#![cfg(feature = "fl")]

use std::{
	fs,
	path::Path,
	sync::atomic::{AtomicUsize, Ordering}
};

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rpkg_rs::resource::{
	partition_manager::{PartitionManager, PartitionManagerPar},
	pdefs::{PackageDefinitionParser, PackageDefinitionSource, bond_parser::BondParser}
};

const GAME_PATH: &str = "/home/user/.local/share/Steam/steamapps/common/007 First Light/Retail";

#[test]
fn roundtrip_all_resources() {
	let game_path = Path::new(GAME_PATH);
	println!("Testing against 007: First Light at {}", game_path.display());

	let mut game_files = PartitionManager::new(game_path.join("../Runtime"), &{
		let mut partitions = BondParser::parse(
			&fs::read(game_path.join("../Runtime/packagedefinition.txt")).expect("Couldn't read packagedefinition")
		)
		.expect("Couldn't parse packagedefinition");

		for partition in &mut partitions {
			partition.set_max_patch_level(9);
		}

		PackageDefinitionSource::Custom(partitions)
	})
	.expect("Couldn't create partition manager");

	game_files
		.mount_partitions_par(|_, _| {})
		.expect("Couldn't load game files");

	for partition in &game_files.partitions {
		let resources = partition.latest_resources();
		let total = resources.len();
		let progress = AtomicUsize::new(0);

		let partition_name = partition
			.partition_info()
			.name
			.to_owned()
			.unwrap_or_else(|| partition.partition_info().id.to_string());

		println!("Testing partition {partition_name}: 0/{total}");
		resources.into_par_iter().for_each(|(info, _)| {
			if ![
				#[cfg(feature = "CBLU")]
				"CBLU",
				#[cfg(feature = "CLRP")]
				"CLRP",
				#[cfg(feature = "CPPT")]
				"CPPT",
				#[cfg(feature = "CRMD")]
				"CRMD",
				#[cfg(feature = "ECPB")]
				"ECPB",
				#[cfg(feature = "ENUM")]
				"ENUM",
				#[cfg(feature = "GFXA")]
				"GFXA",
				#[cfg(feature = "GFXF")]
				"GFXF",
				#[cfg(feature = "GIDX")]
				"GIDX",
				#[cfg(feature = "KWOR")]
				"KWOR",
				#[cfg(feature = "ORES")]
				"ORES",
				#[cfg(feature = "TBLU")]
				"TBLU",
				#[cfg(feature = "TDAT")]
				"TDAT",
				#[cfg(feature = "TDPK")]
				"TDPK",
				#[cfg(feature = "TEMP")]
				"TEMP",
				#[cfg(feature = "UICB")]
				"UICB",
				#[cfg(feature = "WEMD")]
				"WEMD",
				#[cfg(feature = "WSGB")]
				"WSGB",
				#[cfg(feature = "WSWB")]
				"WSWB",
				#[cfg(feature = "WSWB")]
				"DSWB",
				""
			]
			.into_iter()
			.any(|x| x == info.data_type())
			{
				let progress = progress.fetch_add(1, Ordering::Relaxed) + 1;
				if progress.is_multiple_of(total / 10) {
					println!("Testing partition {partition_name}: {progress}/{total}");
				}
				return;
			}

			let data = partition
				.read_resource(info.rrid())
				.unwrap_or_else(|e| panic!("Couldn't read resource {}: {e}", info.rrid().to_hex_string()));

			let initial_size = data.len();

			#[allow(unused_macros)]
			macro_rules! roundtrip {
				($data:expr, $round:literal, $ty:ty) => {{
					#[allow(unused_imports)]
					use glacier_bin1::game::fl::*;
					glacier_bin1::serialize(&glacier_bin1::deserialize::<$ty>(&$data).unwrap_or_else(|e| {
						panic!(
							"Couldn't deserialize {} data for {}.{} in {partition_name}: {e}",
							$round,
							info.rrid().to_hex_string(),
							info.data_type()
						)
					}))
					.unwrap_or_else(|e| {
						panic!(
							"Couldn't serialize deserialized resource for {}.{} in {partition_name}: {e}",
							info.rrid().to_hex_string(),
							info.data_type()
						)
					})
				}};
			}

			let data = match info.data_type().as_str() {
				#[cfg(feature = "CBLU")]
				"CBLU" => roundtrip!(data, "original", SCppEntityBlueprint),
				#[cfg(feature = "CLRP")]
				"CLRP" => roundtrip!(data, "original", SColorPalette),
				#[cfg(feature = "CPPT")]
				"CPPT" => roundtrip!(data, "original", SCppEntity),
				#[cfg(feature = "CRMD")]
				"CRMD" => roundtrip!(data, "original", SCrowdMapData),
				#[cfg(feature = "ECPB")]
				"ECPB" => roundtrip!(data, "original", SExtendedCppEntityBlueprint),
				#[cfg(feature = "ENUM")]
				"ENUM" => roundtrip!(data, "original", SEnumType),
				#[cfg(feature = "GFXA")]
				"GFXA" => roundtrip!(data, "original", SGFxAtlas),
				#[cfg(feature = "GFXF")]
				"GFXF" => roundtrip!(data, "original", SGFxMovieResource),
				#[cfg(feature = "GIDX")]
				"GIDX" => roundtrip!(data, "original", SResourceIndex),
				#[cfg(feature = "KWOR")]
				"KWOR" => roundtrip!(data, "original", SSerializedKeyword),
				#[cfg(feature = "TBLU")]
				"TBLU" => roundtrip!(data, "original", STemplateEntityBlueprint),
				#[cfg(feature = "TDAT")]
				"TDAT" => roundtrip!(data, "original", STerrainResource),
				#[cfg(feature = "TDPK")]
				"TDPK" => roundtrip!(data, "original", STerrainDataPackage),
				#[cfg(feature = "TEMP")]
				"TEMP" => roundtrip!(data, "original", STemplateEntityFactory),
				#[cfg(feature = "UICB")]
				"UICB" => roundtrip!(data, "original", SControlTypeInfo),
				#[cfg(feature = "WEMD")]
				"WEMD" => roundtrip!(data, "original", Vec<SAudioEventMetadata>),
				#[cfg(feature = "WSGB")]
				"WSGB" => roundtrip!(data, "original", SAudioStateGroupData),
				#[cfg(feature = "WSWB")]
				"WSWB" => roundtrip!(data, "original", SAudioSwitchGroupData),
				#[cfg(feature = "WSWB")]
				"DSWB" => roundtrip!(data, "original", SAudioSwitchGroupData),
				_ => data
			};

			if data.len() > initial_size {
				println!(
					"Roundtripped data is larger for resource {}.{} in partition {partition_name}",
					info.rrid(),
					info.data_type()
				);
			}

			let second_data = match info.data_type().as_str() {
				#[cfg(feature = "CBLU")]
				"CBLU" => roundtrip!(data, "roundtripped", SCppEntityBlueprint),
				#[cfg(feature = "CLRP")]
				"CLRP" => roundtrip!(data, "roundtripped", SColorPalette),
				#[cfg(feature = "CPPT")]
				"CPPT" => roundtrip!(data, "roundtripped", SCppEntity),
				#[cfg(feature = "CRMD")]
				"CRMD" => roundtrip!(data, "roundtripped", SCrowdMapData),
				#[cfg(feature = "ECPB")]
				"ECPB" => roundtrip!(data, "roundtripped", SExtendedCppEntityBlueprint),
				#[cfg(feature = "ENUM")]
				"ENUM" => roundtrip!(data, "roundtripped", SEnumType),
				#[cfg(feature = "GFXA")]
				"GFXA" => roundtrip!(data, "roundtripped", SGFxAtlas),
				#[cfg(feature = "GFXF")]
				"GFXF" => roundtrip!(data, "roundtripped", SGFxMovieResource),
				#[cfg(feature = "GIDX")]
				"GIDX" => roundtrip!(data, "roundtripped", SResourceIndex),
				#[cfg(feature = "KWOR")]
				"KWOR" => roundtrip!(data, "roundtripped", SSerializedKeyword),
				#[cfg(feature = "TBLU")]
				"TBLU" => roundtrip!(data, "roundtripped", STemplateEntityBlueprint),
				#[cfg(feature = "TDAT")]
				"TDAT" => roundtrip!(data, "roundtripped", STerrainResource),
				#[cfg(feature = "TDPK")]
				"TDPK" => roundtrip!(data, "roundtripped", STerrainDataPackage),
				#[cfg(feature = "TEMP")]
				"TEMP" => roundtrip!(data, "roundtripped", STemplateEntityFactory),
				#[cfg(feature = "UICB")]
				"UICB" => roundtrip!(data, "roundtripped", SControlTypeInfo),
				#[cfg(feature = "WEMD")]
				"WEMD" => roundtrip!(data, "roundtripped", Vec<SAudioEventMetadata>),
				#[cfg(feature = "WSGB")]
				"WSGB" => roundtrip!(data, "roundtripped", SAudioStateGroupData),
				#[cfg(feature = "WSWB")]
				"WSWB" => roundtrip!(data, "roundtripped", SAudioSwitchGroupData),
				#[cfg(feature = "WSWB")]
				"DSWB" => roundtrip!(data, "roundtripped", SAudioSwitchGroupData),
				_ => data.to_owned()
			};

			assert!(
				data == second_data,
				"Second roundtrip doesn't match first for resource {}.{} in partition {partition_name}",
				info.rrid(),
				info.data_type()
			);

			let progress = progress.fetch_add(1, Ordering::Relaxed) + 1;
			if progress.is_multiple_of(total / 10) {
				println!("Testing partition {partition_name}: {progress}/{total}");
			}
		});
	}
}
