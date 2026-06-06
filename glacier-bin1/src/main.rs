#![allow(unused)]

use std::{env, fs, path::PathBuf};

use glacier_bin1::{deserialize, ser::Bin1Serializer, serialize};

#[cfg(feature = "h1")]
use glacier_bin1::game::h1;

#[cfg(feature = "h2")]
use glacier_bin1::game::h2;

#[cfg(feature = "h3")]
use glacier_bin1::game::h3;

#[cfg(feature = "fl")]
use glacier_bin1::game::fl;

macro_rules! impl_convert {
	($resource_type:ident, $feature:literal, $ty:literal, $res:ty) => {
		#[cfg(feature = $feature)]
		if $resource_type.as_deref() == Some($ty) {
			let value: $res =
				deserialize(&fs::read(env::args().nth(4).expect("4th argument must be input path")).unwrap()).unwrap();

			fs::write(
				env::args().nth(5).map_or_else(
					|| PathBuf::from(env::args().nth(4).unwrap()).with_added_extension("json"),
					PathBuf::from
				),
				serde_json::to_vec(&value).unwrap()
			)
			.unwrap();

			return;
		}
	};

	($resource_type:ident, $ty:literal, $res:ty) => {
		#[cfg(feature = $ty)]
		if $resource_type.as_deref() == Some($ty) {
			let value: $res =
				deserialize(&fs::read(env::args().nth(4).expect("4th argument must be input path")).unwrap()).unwrap();

			fs::write(
				env::args().nth(5).map_or_else(
					|| PathBuf::from(env::args().nth(4).unwrap()).with_added_extension("json"),
					PathBuf::from
				),
				serde_json::to_vec(&value).unwrap()
			)
			.unwrap();

			return;
		}
	};
}

macro_rules! impl_generate {
	($resource_type:ident, $feature:literal, $ty:literal, $res:ty) => {
		#[cfg(feature = $feature)]
		if $resource_type.as_deref() == Some($ty) {
			let value: $res = serde_json::from_slice(
				&fs::read(env::args().nth(4).expect("4th argument must be input path")).unwrap()
			)
			.unwrap();

			fs::write(
				env::args().nth(5).map_or_else(
					|| {
						let path = PathBuf::from(env::args().nth(4).unwrap());
						path.with_file_name(path.file_stem().unwrap_or_else(|| path.file_name().unwrap()))
					},
					PathBuf::from
				),
				serialize(&value).unwrap()
			)
			.unwrap();

			return;
		}
	};

	($resource_type:ident, $ty:literal, $res:ty) => {
		#[cfg(feature = $ty)]
		if $resource_type.as_deref() == Some($ty) {
			let value: $res = serde_json::from_slice(
				&fs::read(env::args().nth(4).expect("4th argument must be input path")).unwrap()
			)
			.unwrap();

			fs::write(
				env::args().nth(5).map_or_else(
					|| {
						let path = PathBuf::from(env::args().nth(4).unwrap());
						path.with_file_name(path.file_stem().unwrap_or_else(|| path.file_name().unwrap()))
					},
					PathBuf::from
				),
				serialize(&value).unwrap()
			)
			.unwrap();

			return;
		}
	};
}

macro_rules! impl_generate_norrids {
	($resource_type:ident, $feature:literal, $ty:literal, $res:ty) => {
		#[cfg(feature = $feature)]
		if $resource_type.as_deref() == Some($ty) {
			let value: $res = serde_json::from_slice(
				&fs::read(env::args().nth(4).expect("4th argument must be input path")).unwrap()
			)
			.unwrap();

			fs::write(
				env::args().nth(5).map_or_else(
					|| {
						let path = PathBuf::from(env::args().nth(4).unwrap());
						path.with_file_name(path.file_stem().unwrap_or_else(|| path.file_name().unwrap()))
					},
					PathBuf::from
				),
				Bin1Serializer::new()
					.with_rrids_segment(false)
					.serialize(&value)
					.unwrap()
			)
			.unwrap();

			return;
		}
	};

	($resource_type:ident, $ty:literal, $res:ty) => {
		#[cfg(feature = $ty)]
		if $resource_type.as_deref() == Some($ty) {
			let value: $res = serde_json::from_slice(
				&fs::read(env::args().nth(4).expect("4th argument must be input path")).unwrap()
			)
			.unwrap();

			fs::write(
				env::args().nth(5).map_or_else(
					|| {
						let path = PathBuf::from(env::args().nth(4).unwrap());
						path.with_file_name(path.file_stem().unwrap_or_else(|| path.file_name().unwrap()))
					},
					PathBuf::from
				),
				Bin1Serializer::new()
					.with_rrids_segment(false)
					.serialize(&value)
					.unwrap()
			)
			.unwrap();

			return;
		}
	};
}

macro_rules! impl_generate_noresptrs {
	($resource_type:ident, $feature:literal, $ty:literal, $res:ty) => {
		#[cfg(feature = $feature)]
		if $resource_type.as_deref() == Some($ty) {
			let value: $res = serde_json::from_slice(
				&fs::read(env::args().nth(4).expect("4th argument must be input path")).unwrap()
			)
			.unwrap();

			fs::write(
				env::args().nth(5).map_or_else(
					|| {
						let path = PathBuf::from(env::args().nth(4).unwrap());
						path.with_file_name(path.file_stem().unwrap_or_else(|| path.file_name().unwrap()))
					},
					PathBuf::from
				),
				Bin1Serializer::new()
					.with_resource_ptrs_segment(false)
					.serialize(&value)
					.unwrap()
			)
			.unwrap();

			return;
		}
	};

	($resource_type:ident, $ty:literal, $res:ty) => {
		#[cfg(feature = $ty)]
		if $resource_type.as_deref() == Some($ty) {
			let value: $res = serde_json::from_slice(
				&fs::read(env::args().nth(4).expect("4th argument must be input path")).unwrap()
			)
			.unwrap();

			fs::write(
				env::args().nth(5).map_or_else(
					|| {
						let path = PathBuf::from(env::args().nth(4).unwrap());
						path.with_file_name(path.file_stem().unwrap_or_else(|| path.file_name().unwrap()))
					},
					PathBuf::from
				),
				Bin1Serializer::new()
					.with_resource_ptrs_segment(false)
					.serialize(&value)
					.unwrap()
			)
			.unwrap();

			return;
		}
	};
}

macro_rules! impl_all {
	($resource_type:ident, h1, $impl:ident, $impl_norrids:ident, $impl_noresptrs:ident) => {
		impl_all!(generic, $resource_type, h1, $impl, $impl_norrids, $impl_noresptrs);

		$impl!($resource_type, "TEMP", h1::STemplateEntity);
	};

	($resource_type:ident, h2, $impl:ident, $impl_norrids:ident, $impl_noresptrs:ident) => {
		impl_all!(generic, $resource_type, h2, $impl, $impl_norrids, $impl_noresptrs);

		$impl!($resource_type, "TEMP", h2::STemplateEntityFactory);
		$impl!($resource_type, "ECPB", h2::SExtendedCppEntityBlueprint);
	};

	($resource_type:ident, h3, $impl:ident, $impl_norrids:ident, $impl_noresptrs:ident) => {
		impl_all!(generic, $resource_type, h3, $impl, $impl_norrids, $impl_noresptrs);

		$impl!($resource_type, "TEMP", h3::STemplateEntityFactory);
		$impl!($resource_type, "ECPB", h3::SExtendedCppEntityBlueprint);

		$impl_norrids!($resource_type, "ORES", "ORES-activities", h3::SActivities);
	};

	($resource_type:ident, fl, $impl:ident, $impl_norrids:ident, $impl_noresptrs:ident) => {
		$impl!($resource_type, "CBLU", fl::SCppEntityBlueprint);
		$impl!($resource_type, "CPPT", fl::SCppEntity);
		$impl!($resource_type, "CRMD", fl::SCrowdMapData);
		$impl!($resource_type, "ECPB", fl::SExtendedCppEntityBlueprint);
		$impl!($resource_type, "ENUM", fl::SEnumType);
		$impl!($resource_type, "GFXF", fl::SGFxMovieResource);
		$impl!($resource_type, "GIDX", fl::SResourceIndex);
		$impl!($resource_type, "KWOR", fl::SSerializedKeyword);
		$impl!($resource_type, "TBLU", fl::STemplateEntityBlueprint);
		$impl!($resource_type, "TDAT", fl::STerrainResource);
		$impl!($resource_type, "TDPK", fl::STerrainDataPackage);
		$impl!($resource_type, "TEMP", fl::STemplateEntityFactory);
		$impl!($resource_type, "UICB", fl::SControlTypeInfo);
		$impl!($resource_type, "WSGB", fl::SAudioStateGroupData);
		$impl!($resource_type, "WSWB", fl::SAudioSwitchGroupData);

		$impl_norrids!(
			$resource_type,
			"ORES",
			"ORES-blobs",
			Vec<fl::SBlobsConfigResourceEntry>
		);
		$impl_norrids!(
			$resource_type,
			"ORES",
			"ORES-contracts",
			Vec<fl::SContractConfigResourceEntry>
		);
		$impl_norrids!($resource_type, "ORES", "ORES-unlockables", ecow::EcoString);
		$impl_norrids!(
			$resource_type,
			"ORES",
			"ORES-environment",
			fl::SEnvironmentConfigResource
		);
		$impl_norrids!($resource_type, "ORES", "ORES-activities", fl::SActivities);
	};

	(generic, $resource_type:ident, $game:ident, $impl:ident, $impl_norrids:ident, $impl_noresptrs:ident) => {
		$impl!($resource_type, "AIBB", $game::SBehaviorTreeInfo);
		$impl!($resource_type, "AIRG", $game::SReasoningGrid);
		$impl!($resource_type, "ASVA", Vec<$game::SPackedAnimSetEntry>);
		$impl!($resource_type, "ATMD", $game::ZAMDTake);
		$impl!($resource_type, "BMSK", Vec<u32>);
		$impl!($resource_type, "CBLU", $game::SCppEntityBlueprint);
		$impl!($resource_type, "CPPT", $game::SCppEntity);
		$impl!($resource_type, "CRMD", $game::SCrowdMapData);
		$impl!($resource_type, "ENUM", $game::SEnumType);
		$impl!($resource_type, "GFXF", $game::SGFxMovieResource);
		$impl!($resource_type, "GIDX", $game::SResourceIndex);
		$impl!($resource_type, "TBLU", $game::STemplateEntityBlueprint);
		$impl!($resource_type, "UICB", $game::SControlTypeInfo);
		$impl_noresptrs!($resource_type, "VIDB", $game::SVideoDatabaseData);
		$impl!($resource_type, "WSGB", $game::SAudioStateGroupData);
		$impl!($resource_type, "WSWB", $game::SAudioSwitchGroupData);

		$impl_norrids!(
			$resource_type,
			"ORES",
			"ORES-blobs",
			Vec<$game::SBlobsConfigResourceEntry>
		);
		$impl_norrids!(
			$resource_type,
			"ORES",
			"ORES-contracts",
			Vec<$game::SContractConfigResourceEntry>
		);
		$impl_norrids!($resource_type, "ORES", "ORES-unlockables", ecow::EcoString);
		$impl_norrids!(
			$resource_type,
			"ORES",
			"ORES-environment",
			$game::SEnvironmentConfigResource
		);
	};
}

fn main() {
	const GAMES: &[&str] = &[
		#[cfg(feature = "h1")]
		"HM2016",
		#[cfg(feature = "h2")]
		"HM2",
		#[cfg(feature = "h3")]
		"HM3",
		#[cfg(feature = "fl")]
		"KNT"
	];

	const RESOURCE_TYPES: &[&str] = &[
		#[cfg(feature = "AIBB")]
		"AIBB",
		#[cfg(feature = "AIRG")]
		"AIRG",
		#[cfg(feature = "ASVA")]
		"ASVA",
		#[cfg(feature = "ATMD")]
		"ATMD",
		#[cfg(feature = "BMSK")]
		"BMSK",
		#[cfg(feature = "CBLU")]
		"CBLU",
		#[cfg(feature = "CPPT")]
		"CPPT",
		#[cfg(feature = "CRMD")]
		"CRMD",
		#[cfg(feature = "ECPB")]
		"ECPB",
		#[cfg(feature = "ENUM")]
		"ENUM",
		#[cfg(feature = "GFXF")]
		"GFXF",
		#[cfg(feature = "GIDX")]
		"GIDX",
		#[cfg(feature = "ORES")]
		"ORES",
		#[cfg(feature = "TBLU")]
		"TBLU",
		#[cfg(feature = "TEMP")]
		"TEMP",
		#[cfg(feature = "UICB")]
		"UICB",
		#[cfg(feature = "VIDB")]
		"VIDB",
		#[cfg(feature = "WSGB")]
		"WSGB",
		#[cfg(feature = "WSWB")]
		"WSWB"
	];

	match env::args().nth(1).as_deref() {
		#[cfg(feature = "h1")]
		Some("HM2016") => match env::args().nth(2).as_deref() {
			Some("convert") => {
				let resource_type = env::args().nth(3);

				impl_all!(resource_type, h1, impl_convert, impl_convert, impl_convert);

				panic!(
					"3rd argument must be one of the following resource types and supported by the given game \
					 version: {}",
					RESOURCE_TYPES.join(", ")
				);
			}

			Some("generate") => {
				let resource_type = env::args().nth(3);

				impl_all!(
					resource_type,
					h1,
					impl_generate,
					impl_generate_norrids,
					impl_generate_noresptrs
				);

				panic!(
					"3rd argument must be one of the following resource types and supported by the given game \
					 version: {}",
					RESOURCE_TYPES.join(", ")
				);
			}

			_ => panic!("2nd argument must be convert or generate")
		},

		#[cfg(feature = "h2")]
		Some("HM2") => match env::args().nth(2).as_deref() {
			Some("convert") => {
				let resource_type = env::args().nth(3);

				impl_all!(resource_type, h2, impl_convert, impl_convert, impl_convert);

				panic!(
					"3rd argument must be one of the following resource types and supported by the given game \
					 version: {}",
					RESOURCE_TYPES.join(", ")
				);
			}

			Some("generate") => {
				let resource_type = env::args().nth(3);

				impl_all!(
					resource_type,
					h2,
					impl_generate,
					impl_generate_norrids,
					impl_generate_noresptrs
				);

				panic!(
					"3rd argument must be one of the following resource types and supported by the given game \
					 version: {}",
					RESOURCE_TYPES.join(", ")
				);
			}

			_ => panic!("2nd argument must be convert or generate")
		},

		#[cfg(feature = "h3")]
		Some("HM3") => match env::args().nth(2).as_deref() {
			Some("convert") => {
				let resource_type = env::args().nth(3);

				impl_all!(resource_type, h3, impl_convert, impl_convert, impl_convert);

				panic!(
					"3rd argument must be one of the following resource types and supported by the given game \
					 version: {}",
					RESOURCE_TYPES.join(", ")
				);
			}

			Some("generate") => {
				let resource_type = env::args().nth(3);

				impl_all!(
					resource_type,
					h3,
					impl_generate,
					impl_generate_norrids,
					impl_generate_noresptrs
				);

				panic!(
					"3rd argument must be one of the following resource types and supported by the given game \
					 version: {}",
					RESOURCE_TYPES.join(", ")
				);
			}

			_ => panic!("2nd argument must be convert or generate")
		},

		#[cfg(feature = "fl")]
		Some("KNT") => match env::args().nth(2).as_deref() {
			Some("convert") => {
				let resource_type = env::args().nth(3);

				impl_all!(resource_type, fl, impl_convert, impl_convert, impl_convert);

				panic!(
					"3rd argument must be one of the following resource types and supported by the given game \
					 version: {}",
					RESOURCE_TYPES.join(", ")
				);
			}

			Some("generate") => {
				let resource_type = env::args().nth(3);

				impl_all!(
					resource_type,
					fl,
					impl_generate,
					impl_generate_norrids,
					impl_generate_noresptrs
				);

				panic!(
					"3rd argument must be one of the following resource types and supported by the given game \
					 version: {}",
					RESOURCE_TYPES.join(", ")
				);
			}

			_ => panic!("2nd argument must be convert or generate")
		},

		_ => panic!("1st argument must be one of the games: {}", GAMES.join(", "))
	}
}
