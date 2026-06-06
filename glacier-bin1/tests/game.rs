use std::{
	fs,
	sync::atomic::{AtomicUsize, Ordering}
};

use hitman_commons::game_detection::GameInstall;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rpkg_rs::resource::{
	partition_manager::{PartitionManager, PartitionManagerPar},
	pdefs::{GamePaths, PackageDefinitionSource}
};

#[static_init::dynamic]
static GAMES: Vec<GameInstall> = hitman_commons::game_detection::detect_installs()
	.expect("Couldn't detect game installs")
	.into_iter()
	.filter(|x| {
		[
			#[cfg(feature = "h1")]
			hitman_commons::game::GameVersion::H1,
			#[cfg(feature = "h2")]
			hitman_commons::game::GameVersion::H2,
			#[cfg(feature = "h3")]
			hitman_commons::game::GameVersion::H3
		]
		.contains(&x.version)
	})
	.collect();

#[test]
fn roundtrip_all_resources() {
	for game in GAMES.iter() {
		println!("Testing against {} at {}", game.version, game.path.display());

		let game_paths: GamePaths =
			GamePaths::from_retail_directory(game.path.to_owned()).expect("Couldn't find game paths");

		let mut game_files = PartitionManager::new(game_paths.runtime_path.to_owned(), &{
			let mut partitions = PackageDefinitionSource::from_version(
				game.version.into(),
				fs::read(game_paths.runtime_path.join("packagedefinition.txt"))
					.expect("Couldn't read packagedefinition")
			)
			.read()
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
					#[cfg(feature = "TEMP")]
					"TEMP",
					#[cfg(feature = "TBLU")]
					"TBLU",
					#[cfg(feature = "UICB")]
					"UICB",
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
					($data:expr, $round:literal, $h1_ty:ty, $h2_ty:ty, $h3_ty:ty) => {
						match game.version {
							#[cfg(feature = "h1")]
							hitman_commons::game::GameVersion::H1 => {
								#[allow(unused_imports)]
								use glacier_bin1::game::h1::*;
								glacier_bin1::serialize(&glacier_bin1::deserialize::<$h1_ty>(&$data).unwrap_or_else(
									|e| {
										panic!(
											"Couldn't deserialize {} data for {}.{} in {partition_name}: {e}",
											$round,
											info.rrid().to_hex_string(),
											info.data_type()
										)
									}
								))
								.unwrap_or_else(|e| {
									panic!(
										"Couldn't serialize deserialized resource for {}.{} in {partition_name}: {e}",
										info.rrid().to_hex_string(),
										info.data_type()
									)
								})
							}

							#[cfg(feature = "h2")]
							hitman_commons::game::GameVersion::H2 => {
								#[allow(unused_imports)]
								use glacier_bin1::game::h2::*;
								glacier_bin1::serialize(&glacier_bin1::deserialize::<$h2_ty>(&$data).unwrap_or_else(
									|e| {
										panic!(
											"Couldn't deserialize {} data for {}.{} in {partition_name}: {e}",
											$round,
											info.rrid().to_hex_string(),
											info.data_type()
										)
									}
								))
								.unwrap_or_else(|e| {
									panic!(
										"Couldn't serialize deserialized resource for {}.{} in {partition_name}: {e}",
										info.rrid().to_hex_string(),
										info.data_type()
									)
								})
							}

							#[cfg(feature = "h3")]
							hitman_commons::game::GameVersion::H3 => {
								#[allow(unused_imports)]
								use glacier_bin1::game::h3::*;
								glacier_bin1::serialize(&glacier_bin1::deserialize::<$h3_ty>(&$data).unwrap_or_else(
									|e| {
										panic!(
											"Couldn't deserialize {} data for {}.{} in {partition_name}: {e}",
											$round,
											info.rrid().to_hex_string(),
											info.data_type()
										)
									}
								))
								.unwrap_or_else(|e| {
									panic!(
										"Couldn't serialize deserialized resource for {}.{} in {partition_name}: {e}",
										info.rrid().to_hex_string(),
										info.data_type()
									)
								})
							}

							_ => panic!("Unsupported game version")
						}
					};

					($data:expr, $round:literal, $ty:ty) => {
						roundtrip!($data, $round, $ty, $ty, $ty)
					};
				}

				let data = match info.data_type().as_str() {
					#[cfg(feature = "AIBB")]
					"AIBB" => roundtrip!(data, "original", SBehaviorTreeInfo),
					#[cfg(feature = "AIRG")]
					"AIRG" => roundtrip!(data, "original", SReasoningGrid),
					#[cfg(feature = "ASVA")]
					"ASVA" => roundtrip!(data, "original", Vec<SPackedAnimSetEntry>),
					#[cfg(feature = "ATMD")]
					"ATMD" => roundtrip!(data, "original", ZAMDTake),
					#[cfg(feature = "BMSK")]
					"BMSK" => roundtrip!(data, "original", Vec<u32>),
					#[cfg(feature = "CBLU")]
					"CBLU" => roundtrip!(data, "original", SCppEntityBlueprint),
					#[cfg(feature = "CPPT")]
					"CPPT" => roundtrip!(data, "original", SCppEntity),
					#[cfg(feature = "CRMD")]
					"CRMD" => roundtrip!(data, "original", SCrowdMapData),
					#[cfg(feature = "ECPB")]
					"ECPB" => roundtrip!(
						data,
						"original",
						(),
						SExtendedCppEntityBlueprint,
						SExtendedCppEntityBlueprint
					),
					#[cfg(feature = "ENUM")]
					"ENUM" => roundtrip!(data, "original", SEnumType),
					#[cfg(feature = "GFXF")]
					"GFXF" => roundtrip!(data, "original", SGFxMovieResource),
					#[cfg(feature = "GIDX")]
					"GIDX" => roundtrip!(data, "original", SResourceIndex),
					#[cfg(feature = "TEMP")]
					"TEMP" => roundtrip!(
						data,
						"original",
						STemplateEntity,
						STemplateEntityFactory,
						STemplateEntityFactory
					),
					#[cfg(feature = "TBLU")]
					"TBLU" => roundtrip!(data, "original", STemplateEntityBlueprint),
					#[cfg(feature = "UICB")]
					"UICB" => roundtrip!(data, "original", SControlTypeInfo),
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
					#[cfg(feature = "AIBB")]
					"AIBB" => roundtrip!(data, "roundtripped", SBehaviorTreeInfo),
					#[cfg(feature = "AIRG")]
					"AIRG" => roundtrip!(data, "roundtripped", SReasoningGrid),
					#[cfg(feature = "ASVA")]
					"ASVA" => roundtrip!(data, "roundtripped", Vec<SPackedAnimSetEntry>),
					#[cfg(feature = "ATMD")]
					"ATMD" => roundtrip!(data, "roundtripped", ZAMDTake),
					#[cfg(feature = "BMSK")]
					"BMSK" => roundtrip!(data, "roundtripped", Vec<u32>),
					#[cfg(feature = "CBLU")]
					"CBLU" => roundtrip!(data, "roundtripped", SCppEntityBlueprint),
					#[cfg(feature = "CPPT")]
					"CPPT" => roundtrip!(data, "roundtripped", SCppEntity),
					#[cfg(feature = "CRMD")]
					"CRMD" => roundtrip!(data, "roundtripped", SCrowdMapData),
					#[cfg(feature = "ECPB")]
					"ECPB" => roundtrip!(
						data,
						"roundtripped",
						(),
						SExtendedCppEntityBlueprint,
						SExtendedCppEntityBlueprint
					),
					#[cfg(feature = "ENUM")]
					"ENUM" => roundtrip!(data, "roundtripped", SEnumType),
					#[cfg(feature = "GFXF")]
					"GFXF" => roundtrip!(data, "roundtripped", SGFxMovieResource),
					#[cfg(feature = "GIDX")]
					"GIDX" => roundtrip!(data, "roundtripped", SResourceIndex),
					#[cfg(feature = "TEMP")]
					"TEMP" => roundtrip!(
						data,
						"roundtripped",
						STemplateEntity,
						STemplateEntityFactory,
						STemplateEntityFactory
					),
					#[cfg(feature = "TBLU")]
					"TBLU" => roundtrip!(data, "roundtripped", STemplateEntityBlueprint),
					#[cfg(feature = "UICB")]
					"UICB" => roundtrip!(data, "roundtripped", SControlTypeInfo),
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
}
