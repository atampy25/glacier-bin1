use std::{env, fs, path::PathBuf};

use anyhow::{Context, Result};

fn main() -> Result<()> {
	let out_dir = PathBuf::from(env::var_os("OUT_DIR").context("OUT_DIR not set")?);

	fs::write(
		out_dir.join("properties-crc32.txt"),
		fs::read_to_string("properties.txt")?
			.lines()
			.map(|x| crc32fast::hash(x.trim().as_bytes()).to_string())
			.collect::<Vec<_>>()
			.join("\n")
	)?;

	println!("cargo::rerun-if-changed=build.rs");
	println!("cargo::rerun-if-changed=properties.txt");

	Ok(())
}
