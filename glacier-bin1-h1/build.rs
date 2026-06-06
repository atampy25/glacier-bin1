use std::{env, fs, path::PathBuf};

use anyhow::Result;
use codegen::Scope;
use glacier_bin1_codegen::generate;

pub fn main() -> Result<()> {
	let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

	println!("cargo::rerun-if-changed=build.rs");

	let mut h1 = Scope::new();

	generate(
		&mut h1,
		&fs::read_to_string("ZHMTypes.json")?,
		&fs::read_to_string("../CustomTypes.json")?,
		&[
			#[cfg(feature = "TEMP")]
			&["STemplateEntity", "STemplateEntityFactory"],
			#[cfg(feature = "TBLU")]
			&["STemplateEntityBlueprint"],
			#[cfg(feature = "AIRG")]
			&["SReasoningGrid"],
			#[cfg(feature = "ASVA")]
			&["SPackedAnimSetEntry"],
			#[cfg(feature = "ATMD")]
			&["ZAMDTake"],
			#[cfg(feature = "VIDB")]
			&["SVideoDatabaseData"],
			#[cfg(feature = "UICB")]
			&["SControlTypeInfo"],
			#[cfg(feature = "CBLU")]
			&["SCppEntityBlueprint"],
			#[cfg(feature = "CPPT")]
			&["SCppEntity"],
			#[cfg(feature = "CRMD")]
			&["SCrowdMapData"],
			#[cfg(feature = "WSWB")]
			&["SAudioSwitchGroupData"],
			#[cfg(feature = "GFXF")]
			&["SGFxMovieResource"],
			#[cfg(feature = "GIDX")]
			&["SResourceIndex"],
			#[cfg(feature = "WSGB")]
			&["SAudioStateGroupData"],
			#[cfg(feature = "ENUM")]
			&["SEnumType"],
			#[cfg(feature = "ORES")]
			&[
				"SActivities",
				"SBlobsConfigResourceEntry",
				"SContractConfigResourceEntry",
				"SEnvironmentConfigResource"
			],
			#[cfg(feature = "AIBB")]
			&["SBehaviorTreeInfo"],
			#[cfg(feature = "enums")]
			&["enums"]
		]
	);

	fs::write(out_dir.join("h1.rs"), h1.to_string())?;

	println!("cargo::rerun-if-changed=ZHMTypes.json");
	println!("cargo::rerun-if-changed=../CustomTypes.json");

	Ok(())
}
