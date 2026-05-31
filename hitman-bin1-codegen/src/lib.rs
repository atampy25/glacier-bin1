use std::collections::VecDeque;

use codegen::{Block, Scope};
use inflector::Inflector;
use lazy_regex::{regex_captures, regex_is_match, regex_replace};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Enum {
	name: String,
	size: usize,
	values: Vec<Variant>
}

#[derive(Serialize, Deserialize)]
struct Variant {
	name: String,
	value: i64
}

#[derive(Serialize, Deserialize)]
struct Struct {
	name: String,
	alignment: Option<usize>,
	fields: Vec<Field>
}

#[derive(Serialize, Deserialize)]
struct Field {
	name: String,

	#[serde(rename = "type")]
	ty: String
}

#[derive(Serialize, Deserialize)]
struct Types {
	structs: Vec<Struct>,
	enums: Vec<Enum>
}

#[derive(Serialize, Deserialize)]
struct RustEnum {
	type_id: String,
	rust_name: String,
	size: usize,
	values: Vec<RustVariant>
}

#[derive(Serialize, Deserialize)]
struct RustVariant {
	variant_name: String,
	rust_name: String,
	value: i64
}

#[derive(Serialize, Deserialize)]
struct RustStruct {
	type_id: String,
	rust_name: String,
	alignment: Option<usize>,
	fields: Vec<RustField>
}

#[derive(Serialize, Deserialize)]
struct RustField {
	field_name: String,
	rust_name: String,

	#[serde(rename = "type")]
	ty: String
}

#[derive(Serialize, Deserialize)]
struct RustTypes {
	structs: Vec<RustStruct>,
	enums: Vec<RustEnum>
}

fn process_types(types: Types) -> RustTypes {
	RustTypes {
		structs: types
			.structs
			.into_par_iter()
			.map(|s| RustStruct {
				rust_name: s.name.replace('.', "_"),
				type_id: s.name,
				alignment: s.alignment,
				fields: s
					.fields
					.into_iter()
					.map(|f| RustField {
						rust_name: {
							let field_name = &f.name;

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

							field_name.into()
						},
						field_name: f.name,
						ty: {
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

									x => x.replace('.', "_")
								}
							}

							process_type_name(&f.ty)
						}
					})
					.collect()
			})
			.collect(),
		enums: types
			.enums
			.into_par_iter()
			.map(|e| {
				let common_prefix = e
					.values
					.iter()
					.map(|v| v.name.split_once("_").map(|x| x.0))
					.collect::<Option<Vec<_>>>()
					.and_then(|prefixes| {
						let first = prefixes[0];
						if prefixes.iter().all(|x| *x == first) {
							Some(first.to_owned())
						} else {
							None
						}
					});

				RustEnum {
					rust_name: e.name.replace('.', "_"),
					type_id: e.name,
					size: e.size,
					values: e
						.values
						.into_iter()
						.map(|v| RustVariant {
							rust_name: {
								if let Some(common_prefix) = &common_prefix {
									let name = v.name.trim_start_matches(common_prefix).trim_start_matches('_');
									if name.starts_with(|c: char| c.is_ascii_digit()) {
										format!(
											"_{}",
											if name.chars().all(|c| c.is_uppercase() || !c.is_ascii_alphabetic()) {
												name.to_pascal_case()
											} else {
												name.into()
											}
										)
									} else {
										if name.chars().all(|c| c.is_uppercase() || !c.is_ascii_alphabetic()) {
											name.to_pascal_case()
										} else {
											name.into()
										}
									}
								} else {
									if v.name.chars().all(|c| c.is_uppercase() || !c.is_ascii_alphabetic()) {
										v.name.to_pascal_case()
									} else {
										v.name.to_owned()
									}
								}
							},
							variant_name: v.name,
							value: v.value
						})
						.collect()
				}
			})
			.collect()
	}
}

pub fn generate(scope: &mut Scope, types_json: &str, custom_types_json: &str, to_generate: &[&[&str]]) {
	let RustTypes { mut structs, mut enums } = process_types(serde_json::from_str(types_json).unwrap());
	let RustTypes {
		structs: mut custom_structs,
		enums: mut custom_enums
	} = process_types(serde_json::from_str(custom_types_json).unwrap());
	structs.append(&mut custom_structs);
	enums.append(&mut custom_enums);

	// Special cased
	structs.remove(
		structs
			.iter()
			.position(|x| x.rust_name == "ZRuntimeResourceID")
			.unwrap()
	);
	structs.remove(
		structs
			.iter()
			.position(|x| x.rust_name == "SEntityTemplateProperty")
			.unwrap()
	);

	let mut struct_queue = VecDeque::new();
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
		"ZHUDOccluderTriggerEntity.SBoneTestSetup",
		"SGaitTransitionEntry",
		"SClothVertex",
		"ZSharedSensorDef.SVisibilitySetting",
		"SFontLibraryDefinition",
		"SCamBone",
		"SConversationPart",
		"AI.SFirePattern01",
		"STargetableBoneConfiguration",
		"ZSecuritySystemCameraConfiguration.SHitmanVisibleEscalationRule",
		"AI.SFirePattern02",
		"ZSecuritySystemCameraConfiguration.SDeadBodyVisibleEscalationRule",
		"ZOverlayControllerEntity.SInputData",
		"ZEntityReference",
		"SLayerBehaviorConfiguration",
		"ZHM5CrowdGenericEventConsumer.SCrowdSoundGenericEventData",
		"ZHM5FootstepEventConsumer.SFootstepSoundEventData",
		"ZHM5AudioEventConsumer.SAudioAnimationEventData",
		"ZHM5HIKEventConsumer.SZHM5HIKEventData",
		"ZInteractionEventConsumer.SInteractionEventData"
	];

	for ty in to_generate.concat() {
		if let Some(pos) = structs.iter().position(|x| x.type_id == ty) {
			struct_queue.push_back(structs.remove(pos));
		}

		if ty == "enums" {
			scope.import("std::str", "FromStr");
			enum_queue.append(&mut enums);
		}
	}

	while let Some(RustStruct {
		rust_name,
		type_id,
		fields,
		alignment,
		..
	}) = struct_queue.pop_front()
	{
		for RustField { ty, .. } in &fields {
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
						if let Some(pos) = structs.iter().position(|x| x.type_id == *ty) {
							struct_queue.push_back(structs.remove(pos));
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
				} else if let Some(pos) = structs.iter().position(|x| x.rust_name == *ty) {
					struct_queue.push_back(structs.remove(pos));
				} else if let Some(pos) = enums.iter().position(|x| x.rust_name == *ty) {
					scope.import("std::str", "FromStr");
					enum_queue.push(enums.remove(pos));
				}
			}
		}

		if fields.iter().any(|x| x.ty.starts_with('[')) {
			scope.import("serde_with", "serde_as");
		}

		let cls = scope
			.new_struct(&rust_name)
			.derive("Facet")
			.derive("Debug")
			.derive("Clone")
			.derive("PartialEq")
			.derive("Bin1Serialize")
			.derive("Bin1Deserialize")
			.vis("pub");

		if let Some(alignment) = alignment {
			cls.r#macro(format!("#[bin1(alignment = {alignment})]"));
		}

		if fields.iter().any(|x| x.ty.starts_with('[')) {
			cls.r#macro("#[serde_as]");
		}

		// Need to use macro instead of derive to ensure serde derives are under serde_as
		cls.r#macro("#[derive(serde::Serialize, serde::Deserialize)]");

		for RustField {
			rust_name,
			field_name,
			ty
		} in fields.iter()
		{
			let field = cls
				.new_field(rust_name, ty)
				.vis("pub")
				.annotation(format!(r#"#[serde(rename = "{field_name}")]"#))
				.annotation(format!(r#"#[facet(rename = "{field_name}")]"#));

			if ty.starts_with('[') {
				field.annotation(format!(
					r#"#[serde_as(as = "{}")]"#,
					regex_replace!(r"\[.*; (\d+)\]", &ty, "[_; $1]")
				));
			}

			if ty == "EcoString" {
				field.annotation(r#"#[facet(opaque, proxy = String)]"#);
			} else if ty.contains("EcoString") {
				field.annotation(r#"#[facet(opaque)]"#);
			}

			if let Some((_, contained)) = regex_captures!(r"ZHMPtrLen<(.*)>", &ty) {
				field.ty = format!("Vec<{contained}>").into();
				field.annotation(format!(r#"#[bin1(as = "ZHMPtrLen::<{contained}>")]"#));
			}
		}

		scope.raw(format!(r#"submit!({rust_name}, "{type_id}");"#));
	}

	for RustEnum {
		rust_name: enum_name,
		type_id,
		size,
		values
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

		if values.is_empty() {
			// ZST
			item.new_variant("Value")
				.annotation(r#"#[serde(rename = "")]"#)
				.annotation(r#"#[facet(rename = "")]"#);
		} else {
			for RustVariant {
				rust_name,
				variant_name,
				..
			} in &values
			{
				item.new_variant(rust_name)
					.annotation(format!(r#"#[serde(rename = "{variant_name}")]"#))
					.annotation(format!(r#"#[facet(rename = "{variant_name}")]"#));
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
				if values.is_empty() {
					block.line(format!(r#"{enum_name}::Value => """#));
				} else {
					for RustVariant {
						rust_name,
						variant_name,
						..
					} in &values
					{
						block.line(format!(r#"{enum_name}::{rust_name} => "{variant_name}","#));
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
				if values.is_empty() {
					let mut block = Block::new("");
					block.line(r#"value.is_empty().then_some(Self::Value).ok_or(())"#);
					block
				} else {
					let mut block = Block::new("Ok(match value");
					for RustVariant {
						rust_name,
						variant_name,
						..
					} in &values
					{
						block.line(format!(r#""{variant_name}" => Self::{rust_name},"#));
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
				if values.is_empty() {
					block.line(format!("{enum_name}::Value => 1"));
				} else {
					for RustVariant { rust_name, value, .. } in &values {
						block.line(format!("{enum_name}::{rust_name} => {value},"));
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
				if values.is_empty() {
					let mut block = Block::new("");
					block.line(format!(
						r#"if value != 1 {{ eprintln!("Unexpected value for uninhabited enum {type_id}: {{}}", value); }}"#
					));
					block.line("Ok(Self::Value)");
					block
				} else {
					let mut block = Block::new("Ok(match value");
					for RustVariant { rust_name, value, .. } in &values {
						block.line(format!("{value} => Self::{rust_name},"));
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
