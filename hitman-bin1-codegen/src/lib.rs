use std::{
	collections::{HashMap, VecDeque},
	fs
};

use codegen::{Block, Scope};
use edit_distance::edit_distance;
use inflector::Inflector;
use lazy_regex::{regex_captures, regex_captures_iter, regex_is_match, regex_replace};
use rayon::prelude::*;

enum Member {
	Padding(usize),
	Field(String, String, String)
}

struct Enum {
	name: String,
	type_id: String,
	size: usize,
	members: Vec<(String, i64)>
}

fn parse_enums(classes: &str, enums: &str) -> Vec<Enum> {
	let enums = enums.replace('\r', "");
	let enums = regex_captures_iter!(
		r#"\(\*g_Enums\)\["(.*?)"] = \{\n((?:\s+\{ -?\d+, ".*?" \},\n)*)\s+\};"#,
		&enums
	)
	.collect::<Vec<_>>();

	let classes = classes.replace('\r', "");
	let classes = classes.split_once("#pragma pack(push, 1)\n\n").unwrap().1;
	classes
		.split("};\n\n")
		.filter(|section| section.starts_with("// Size:"))
		.filter_map(|section| {
			let size =
				usize::from_str_radix(regex_captures!(r"// Size: 0x([0-9A-F]+)", section).unwrap().1, 16).unwrap();

			let section = section
				.split('\n')
				.filter(|x| !x.is_empty() && !x.trim_start().starts_with("//"))
				.collect::<Vec<_>>()
				.join("\n");

			if section.starts_with("enum class") {
				let (name, _) = section.split_once("\n{").unwrap();

				let name = regex_captures!("enum class (.*?)(?: : .*)?$", &name).unwrap().1;

				let enum_entry = enums.par_iter().min_by_key(|x| edit_distance(name, &x[1])).unwrap();

				let members = enum_entry[2]
					.lines()
					.map(|x| regex_captures!(r#"\{\s*(-?\d+),\s*"(.+?)"\s*\}"#, x))
					.filter_map(|x| x.map(|x| (x.2.to_owned(), x.1.parse::<i64>().unwrap())))
					.collect::<Vec<_>>();

				Some(Enum {
					name: name.to_owned(),
					type_id: enum_entry[1].to_owned(),
					size,
					members
				})
			} else {
				None
			}
		})
		.collect()
}

fn parse_classes(classes: &str, types: &str) -> Vec<(String, String, Vec<Member>)> {
	let types = types
		.lines()
		.filter(|x| x.trim().starts_with("ZHMTypeInfo "))
		.map(|x| regex_captures!(r#" (.*?)::TypeInfo = ZHMTypeInfo\("(.*)?","#, x).unwrap())
		.map(|(_, x, y)| (x, y))
		.collect::<HashMap<_, _>>();

	let classes = classes.replace('\r', "").replace("#pragma pack(pop)", "");
	let classes = classes.split_once("#pragma pack(push, 1)\n\n").unwrap().1;
	classes
		.split("};\n\n")
		.map(|section| {
			section
				.split('\n')
				.map(|x| x.trim())
				.filter(|x| !x.is_empty() && !x.starts_with("//"))
				.collect::<Vec<_>>()
				.join("\n")
		})
		.filter(|section| section.starts_with("class"))
		.map(|section| {
			let (name, members) = section.split_once("\n{\npublic:\n").unwrap();

			let name = name.trim_start_matches("class ");
			let name = regex_replace!(r" */\*.*?\*/ *", name, "").into_owned();

			let members = members
				.lines()
				.filter(|x| !x.starts_with("static") && !x.starts_with("bool operator"))
				.map(|member| {
					if let Some((_, amount)) = regex_captures!(r"uint8_t _pad\w+\[(\d+)\] \{\};", member) {
						Member::Padding(amount.parse().unwrap())
					} else {
						let (_, type_name, field_name) = regex_captures!(r"^(.+) (.+);.*$", member).unwrap();

						let original_field_name = field_name;

						let field_name = if (field_name.len() != 2 && !regex_is_match!(r"m\d+", field_name))
							|| field_name.chars().next().unwrap().is_uppercase()
						{
							field_name.to_snake_case()
						} else {
							field_name.into()
						};

						let field_name = if let Some((start, rest)) = field_name.split_once('_')
							&& start.len() == 1 && !["x", "y", "z"].contains(&start)
							&& !rest.is_empty()
						{
							rest.into()
						} else {
							field_name
						};

						let field_name = if let Some((start, rest)) = field_name.split_once('_')
							&& start.len() == 1 && !["x", "y", "z"].contains(&start)
							&& !rest.is_empty()
						{
							rest.into()
						} else {
							field_name
						};

						let field_name = match field_name.as_str() {
							"type" => "r#type",
							"ref" => "reference",
							"move" => "r#move",
							x => x
						};

						fn process_type_name(type_name: &str) -> String {
							match type_name {
								"int8" | "int8_t" => "i8".into(),
								"int16" => "i16".into(),
								"int32" => "i32".into(),
								"int64" => "i64".into(),

								"char" | "uint8" | "uint8_t" => "u8".into(),
								"uint16" => "u16".into(),
								"uint32" | "uint32_t" => "u32".into(),
								"size_t" | "uint64" => "u64".into(),

								"float32" => "f32".into(),
								"float64" => "f64".into(),

								"bool" => "bool".into(),

								"ZString" => "EcoString".into(),

								x if x.starts_with("TArray<") => format!(
									"Vec<{}>",
									process_type_name(
										&x["TArray<".len()..]
											.chars()
											.rev()
											.skip(1)
											.collect::<Vec<_>>()
											.into_iter()
											.rev()
											.collect::<String>()
									)
								),

								x if x.starts_with("TFixedArray<") => format!(
									"[{}; {}]",
									process_type_name(regex_captures!(r"TFixedArray<(.*), *(.*)>", x).unwrap().1),
									regex_captures!(r"TFixedArray<(.*), *(.*)>", x)
										.unwrap()
										.2
										.parse::<usize>()
										.unwrap()
								),

								x if x.starts_with("TPair<") => format!(
									"({}, {})",
									process_type_name(regex_captures!(r"TPair<(.*), *(.*)>", x).unwrap().1),
									process_type_name(regex_captures!(r"TPair<(.*), *(.*)>", x).unwrap().2)
								),

								x if x.starts_with("ZHMPtrLen<") => format!(
									"ZHMPtrLen<{}>",
									process_type_name(
										&x["ZHMPtrLen<".len()..]
											.chars()
											.rev()
											.skip(1)
											.collect::<Vec<_>>()
											.into_iter()
											.rev()
											.collect::<String>()
									)
								),

								x => x.into()
							}
						}

						let type_name = process_type_name(type_name);

						Member::Field(
							original_field_name.to_owned(),
							field_name.to_owned(),
							type_name.to_owned()
						)
					}
				})
				.collect::<Vec<_>>();

			(
				name.to_owned(),
				types.get(name.as_str()).map_or(name.as_str(), |v| v).to_owned(),
				members
			)
		})
		.collect()
}

pub fn generate(scope: &mut Scope, classes_code: &str, enums_code: &str, types_code: &str, to_generate: &[&[&str]]) {
	let mut classes = parse_classes(
		&format!("{}\n\n{}", classes_code, fs::read_to_string("../custom.txt").unwrap()),
		types_code
	);

	let mut enums = parse_enums(classes_code, enums_code);

	// Special cased
	classes.remove(classes.iter().position(|x| x.0 == "ZRuntimeResourceID").unwrap());
	classes.remove(classes.iter().position(|x| x.0 == "SEntityTemplateProperty").unwrap());

	let mut class_queue = VecDeque::new();
	let mut enum_queue = vec![];

	// Types known to be used as ZVariant
	const KNOWN_VARIANTS: &[&str] = &[
		"SColorRGB",
		"SColorRGBA",
		"ZGuid",
		"ZGameTime",
		"SVector2",
		"SVector3",
		"SVector4",
		"SMatrix43",
		"SWorldSpaceSettings",
		"S25DProjectionSettings",
		"SBodyPartDamageMultipliers",
		"SCCEffectSet",
		"SSCCuriousConfiguration",
		"ZCurve",
		"SMapMarkerData",
		"ZHUDOccluderTriggerEntity_SBoneTestSetup",
		"SGaitTransitionEntry",
		"SClothVertex",
		"ZSharedSensorDef_SVisibilitySetting",
		"SFontLibraryDefinition",
		"SCamBone",
		"SConversationPart",
		"AI_SFirePattern01",
		"STargetableBoneConfiguration",
		"ZSecuritySystemCameraConfiguration_SHitmanVisibleEscalationRule",
		"AI_SFirePattern02",
		"ZSecuritySystemCameraConfiguration_SDeadBodyVisibleEscalationRule",
		"ZOverlayControllerEntity_SInputData",
		"ZEntityReference"
	];

	for ty in to_generate.concat() {
		if let Some(pos) = classes.iter().position(|x| x.0 == ty) {
			class_queue.push_back(classes.remove(pos));
		}

		if ty == "enums" {
			scope.import("std::str", "FromStr");
			enum_queue.append(&mut enums);
		}
	}

	while let Some((name, type_id, members)) = class_queue.pop_front() {
		for member in &members {
			if let Member::Field(_, _, ty) = member {
				let mut tys = vec![ty.trim_start_matches("Vec<").trim_end_matches(">")];
				for ty in tys.clone() {
					if ty.starts_with('(') {
						let (first, second) = ty
							.trim_start_matches('(')
							.trim_end_matches(')')
							.split_once(',')
							.unwrap();

						tys.push(first.trim());
						tys.push(second.trim());
					}
				}

				for ty in tys {
					if ty == "ZVariant" || ty == "SEntityTemplateProperty" {
						for ty in KNOWN_VARIANTS {
							if let Some(pos) = classes.iter().position(|x| x.0 == *ty) {
								class_queue.push_back(classes.remove(pos));
							}
						}

						// All enums are valid ZVariants
						scope.import("std::str", "FromStr");
						enum_queue.append(&mut enums);
					} else if ty == "TResourcePtr" {
						scope.import("hitman_bin1_core::types::resource", "TResourcePtr");
					} else if ty == "TypeID" {
						scope.import("hitman_bin1_core::types::variant", "TypeID");
					} else if ty.starts_with("ZHMPtrLen<") {
						scope.import("hitman_bin1_core::types::array", "ZHMPtrLen");
					} else if let Some(pos) = classes.iter().position(|x| x.0 == *ty) {
						class_queue.push_back(classes.remove(pos));
					} else if let Some(pos) = enums.iter().position(|x| x.name == *ty) {
						scope.import("std::str", "FromStr");
						enum_queue.push(enums.remove(pos));
					}
				}
			}
		}

		if members
			.iter()
			.any(|x| matches!(x, Member::Field(_, _, ty) if ty.starts_with('[')))
		{
			scope.import("serde_with", "serde_as");
		}

		let cls = scope
			.new_struct(&name)
			.derive("Facet")
			.derive("Debug")
			.derive("Clone")
			.derive("PartialEq")
			.derive("Bin1Serialize")
			.derive("Bin1Deserialize")
			.vis("pub");

		if members
			.iter()
			.any(|x| matches!(x, Member::Field(_, _, ty) if ty.starts_with('[')))
		{
			cls.r#macro("#[serde_as]");
		}

		// Need to use macro instead of derive to ensure serde derives are under serde_as
		cls.r#macro("#[derive(serde::Serialize, serde::Deserialize)]");

		let mut padding = 0;

		let mut last_field = None;

		for member in members {
			match member {
				Member::Padding(amount) => {
					padding = amount;
				}

				Member::Field(orig_name, field_name, type_name) => {
					last_field = Some({
						let field = cls
							.new_field(field_name, &type_name)
							.vis("pub")
							.annotation(format!(r#"#[serde(rename = "{orig_name}")]"#));

						if type_name.starts_with('[') {
							field.annotation(format!(
								r#"#[serde_as(as = "{}")]"#,
								regex_replace!(r"\[.*; (\d+)\]", &type_name, "[_; $1]")
							));
						}

						if type_name == "EcoString" {
							field.annotation(r#"#[facet(opaque, proxy = String)]"#);
						} else if type_name.contains("EcoString") {
							field.annotation(r#"#[facet(opaque)]"#);
						}

						if let Some((_, contained)) = regex_captures!(r"ZHMPtrLen<(.*)>", &type_name) {
							field.ty = format!("Vec<{contained}>").into();
							field.annotation(format!(r#"#[bin1(as = "ZHMPtrLen::<{contained}>")]"#));
						}

						if padding != 0 {
							field.annotation(format!("#[bin1(pad = {padding})]"));
						}

						field
					});

					padding = 0;
				}
			}
		}

		if padding != 0 {
			last_field.unwrap().annotation(format!("#[bin1(pad_end = {padding})]"));
		}

		scope.raw(format!(r#"submit!({name}, "{type_id}");"#));
	}

	for Enum {
		name: enum_name,
		type_id,
		size,
		members
	} in enum_queue
	{
		let item = scope
			.new_enum(&enum_name)
			.derive("Facet")
			.derive("Debug")
			.derive("Clone")
			.derive("Copy")
			.derive("PartialEq")
			.derive("Eq")
			.derive("PartialOrd")
			.derive("Ord")
			.derive("Hash")
			.derive("serde::Serialize")
			.derive("serde::Deserialize")
			.vis("pub");

		let common_prefix = members
			.iter()
			.map(|(name, _)| name.split_once("_").map(|x| x.0))
			.collect::<Option<Vec<_>>>()
			.and_then(|prefixes| {
				let first = prefixes[0];
				if prefixes.iter().all(|x| *x == first) {
					Some(first.to_owned())
				} else {
					None
				}
			});

		let members = members
			.into_iter()
			.map(|(variant_name, value)| {
				let rust_name = if let Some(common_prefix) = &common_prefix {
					let name = variant_name.trim_start_matches(common_prefix).trim_start_matches('_');
					if name.starts_with(|c: char| c.is_ascii_digit()) {
						format!("_{}", name.to_pascal_case())
					} else {
						name.to_pascal_case()
					}
				} else {
					variant_name.to_pascal_case()
				};

				(variant_name, rust_name, value)
			})
			.collect::<Vec<_>>();

		if members.is_empty() {
			// ZST
			item.new_variant("Value").annotation(r#"#[serde(rename = "")]"#);
		} else {
			for (game_name, rust_name, _) in &members {
				item.new_variant(rust_name)
					.annotation(format!(r#"#[serde(rename = "{game_name}")]"#));
			}
		}

		let size_ty = match size {
			1 => "u8",
			2 => "u16",
			4 => "u32",
			8 => "u64",
			_ => panic!("Invalid size")
		};

		let signed_size_ty = match size {
			1 => "i8",
			2 => "i16",
			4 => "i32",
			8 => "i64",
			_ => panic!("Invalid size")
		};

		item.repr(size_ty);

		scope
			.new_impl(&enum_name)
			.impl_trait("Aligned")
			.associate_const("ALIGNMENT", "usize", size.to_string(), "");

		scope
			.new_impl("&'static str")
			.impl_trait(format!("From<{enum_name}>"))
			.new_fn("from")
			.arg("value", &enum_name)
			.ret("&'static str")
			.push_block({
				let mut block = Block::new("match value");
				if members.is_empty() {
					block.line(format!(r#"{enum_name}::Value => """#));
				} else {
					for (game_name, rust_name, _) in &members {
						block.line(format!(r#"{enum_name}::{rust_name} => "{game_name}","#));
					}
				}
				block
			});

		scope
			.new_impl(&enum_name)
			.impl_trait("FromStr")
			.associate_type("Err", "()")
			.new_fn("from_str")
			.arg("value", "&str")
			.ret("Result<Self, ()>")
			.push_block({
				if members.is_empty() {
					let mut block = Block::new("");
					block.line(r#"value.is_empty().then_some(Self::Value).ok_or(())"#);
					block
				} else {
					let mut block = Block::new("Ok(match value");
					for (game_name, rust_name, _) in &members {
						block.line(format!(r#""{game_name}" => Self::{rust_name},"#));
					}
					block.line("_ => return Err(())");
					block.after(")");
					block
				}
			});

		scope
			.new_impl(signed_size_ty)
			.impl_trait(format!("From<{enum_name}>"))
			.new_fn("from")
			.arg("value", &enum_name)
			.ret(signed_size_ty)
			.push_block({
				let mut block = Block::new("match value");
				if members.is_empty() {
					block.line(format!("{enum_name}::Value => 1"));
				} else {
					for (_, rust_name, variant_value) in &members {
						block.line(format!("{enum_name}::{rust_name} => {variant_value},"));
					}
				}
				block
			});

		scope
			.new_impl(&enum_name)
			.impl_trait(format!("TryFrom<{signed_size_ty}>"))
			.associate_type("Error", "()")
			.new_fn("try_from")
			.arg("value", signed_size_ty)
			.ret("Result<Self, ()>")
			.push_block({
				if members.is_empty() {
					let mut block = Block::new("");
					block.line(format!(
						r#"if value != 1 {{ eprintln!("Unexpected value for uninhabited enum {type_id}: {{}}", value); }}"#
					));
					block.line("Ok(Self::Value)");
					block
				} else {
					let mut block = Block::new("Ok(match value");
					for (_, rust_name, variant_value) in &members {
						block.line(format!("{variant_value} => Self::{rust_name},"));
					}
					block.line("_ => return Err(())");
					block.after(")");
					block
				}
			});

		let ser_impl = scope.new_impl(&enum_name).impl_trait("Bin1Serialize");
		ser_impl
			.new_fn("alignment")
			.arg_ref_self()
			.ret("usize")
			.line(size.to_string());
		ser_impl
			.new_fn("write")
			.arg_ref_self()
			.arg("ser", "&mut Bin1Serializer")
			.ret("Result<(), SerializeError>")
			.line(format!(
				"ser.write_unaligned(&{signed_size_ty}::from(*self).to_le_bytes());"
			))
			.line("Ok(())");

		let de_impl = scope.new_impl(&enum_name).impl_trait("Bin1Deserialize");
		de_impl.associate_const("SIZE", "usize", size.to_string(), "");
		de_impl
			.new_fn("read")
			.arg("de", "&mut Bin1Deserializer")
			.ret("Result<Self, DeserializeError>")
			.line(format!(
				r"let value = de.read_{signed_size_ty}()?;
value.try_into().map_err(|_| DeserializeError::InvalidEnumValue(value as i64))"
			));

		scope.raw(format!(r#"submit!({enum_name}, "{type_id}");"#));
	}
}
