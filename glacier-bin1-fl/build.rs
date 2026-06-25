use std::{env, fs, path::PathBuf};

use anyhow::Result;
use codegen::Scope;
use glacier_bin1_codegen::generate;

pub fn main() -> Result<()> {
	let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

	println!("cargo::rerun-if-changed=build.rs");

	let mut fl = Scope::new();

	generate(
		&mut fl,
		&fs::read_to_string("ZHMTypes.json")?,
		&fs::read_to_string("CustomTypes.json")?,
		&[
			#[cfg(feature = "CBLU")]
			&["SCppEntityBlueprint"],
			#[cfg(feature = "CLRP")]
			&["SColorPalette"],
			#[cfg(feature = "CPPT")]
			&["SCppEntity"],
			#[cfg(feature = "CRMD")]
			&["SCrowdMapData"],
			#[cfg(feature = "ECPB")]
			&["SExtendedCppEntityBlueprint"],
			#[cfg(feature = "ENUM")]
			&["SEnumType"],
			#[cfg(feature = "GFXA")]
			&["SGFxAtlas"],
			#[cfg(feature = "GFXF")]
			&["SGFxMovieResource"],
			#[cfg(feature = "GIDX")]
			&["SResourceIndex"],
			#[cfg(feature = "KWOR")]
			&["SSerializedKeyword"],
			#[cfg(feature = "ORES")]
			&[
				"SActivities",
				"SBlobsConfigResourceEntry",
				"SContractConfigResourceEntry",
				"SEnvironmentConfigResource"
			],
			#[cfg(feature = "TBLU")]
			&["STemplateEntityBlueprint"],
			#[cfg(feature = "TDAT")]
			&["STerrainResource"],
			#[cfg(feature = "TDPK")]
			&["STerrainDataPackage"],
			#[cfg(feature = "TEMP")]
			&["STemplateEntity", "STemplateEntityFactory"],
			#[cfg(feature = "UICB")]
			&["SControlTypeInfo"],
			#[cfg(feature = "WEMD")]
			&["SAudioEventMetadata"],
			#[cfg(feature = "WSGB")]
			&["SAudioStateGroupData"],
			#[cfg(feature = "WSWB")]
			&["SAudioSwitchGroupData"],
			#[cfg(feature = "enums")]
			&["enums"]
		]
	);

	fs::write(out_dir.join("fl.rs"), fl.to_string())?;

	println!("cargo::rerun-if-changed=ZHMTypes.json");
	println!("cargo::rerun-if-changed=CustomTypes.json");

	Ok(())
}
