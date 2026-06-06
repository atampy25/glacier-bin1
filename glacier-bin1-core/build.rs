use std::{env, fs, path::PathBuf};

use anyhow::{Context, Result};

fn main() -> Result<()> {
	let out_dir = PathBuf::from(env::var_os("OUT_DIR").context("OUT_DIR not set")?);

	let properties = fs::read_to_string("properties.txt")?
		.lines()
		.map(|x| {
			format!(
				"({}, \"{}\"),",
				crc32fast::hash(x.trim().as_bytes()),
				x.trim().replace("\"", "\\\"")
			)
		})
		.collect::<Vec<_>>();

	fs::write(
		out_dir.join("properties.rs"),
		format!(
			"static PROPERTIES_HASHES: [(u32, &str); {}] = [{}];",
			properties.len(),
			properties.join("\n")
		)
	)?;

	println!("cargo::rerun-if-changed=build.rs");
	println!("cargo::rerun-if-changed=properties.txt");

	Ok(())
}
