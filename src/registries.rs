use std::error::Error;

use minijinja::{context, value::Value};
use patch::Patch;

use crate::diff::File;

pub struct PackageMetadata {
	yanked: bool,
	checksum: String,
}

pub trait Registry {
	fn get_package_metadata(&self, name: String, version: String) -> Result<PackageMetadata, Box<dyn Error>>;
	fn checksum_algorithm() -> String;
}

struct CratesRegistry {}

impl Registry for CratesRegistry {
	fn get_package_metadata(&self, pkg: String, version: String) -> Result<PackageMetadata, Box<dyn Error>> {
		Ok(PackageMetadata {
			yanked: false,
			checksum: "q1w2e3".to_string(),
		})
	}

	fn checksum_algorithm() -> String {
		"sha256".to_string()
	}
}

pub fn inspect_package(registry: impl Registry, pkg: String, version: String) -> Result<Value, Box<dyn Error>> {
	let meta = registry.get_package_metadata(pkg.clone(), version.clone())?;

	let diff = include_str!("../examples/minijinja.diff");
	let patches = Patch::from_multiple(diff)?;
	let files: Vec<File> = patches.into_iter().map(|patch| File::from_patch(patch)).collect();

	let ctx = context! { pkg, version, yanked => meta.yanked, files };
	Ok(ctx)
}

pub fn get_registry(registry: String) -> Result<impl Registry, Box<dyn Error>> {
	match registry.as_str() {
		"crates" => Ok(CratesRegistry {}),
		_ => Err("Unknown registry".into()),
	}
}
