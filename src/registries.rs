use std::{
	env,
	fs::{self, rename},
	io,
	path::PathBuf,
};

use anyhow::{bail, Result};
use async_trait::async_trait;
use bytes::Bytes;
use crates_index::Crate;
use rand::{distributions::Alphanumeric, thread_rng, Rng};

#[async_trait]
pub trait Registry {
	async fn inspect(&self, pkg: String, version: String) -> Result<(String, bool)>;
}

struct CratesRegistry {}

#[async_trait]
impl Registry for CratesRegistry {
	async fn inspect(&self, pkg: String, version: String) -> Result<(String, bool)> {
		match Self::find_crate(pkg.clone())? {
			Some(crate_) => match crate_.versions().iter().find(|v| v.version() == version) {
				Some(crate_version) => {
					let tmp_dir = create_tmp_dir()?;
					let crate_dir = tmp_dir.join("b");
					println!("{}", crate_dir.display());

					Self::download_and_extract_crate(
						tmp_dir.clone(),
						crate_dir.clone(),
						pkg.to_string(),
						version.to_string(),
						crate_version.checksum(),
					)
					.await?;

					let diff = include_str!("../examples/minijinja.diff").to_owned();
					let yanked = crate_version.is_yanked();
					Ok((diff, yanked))
				}
				None => bail!("Version not found"),
			},
			None => bail!("Package not found"),
		}
	}
}

impl CratesRegistry {
	fn find_crate(pkg: String) -> Result<Option<Crate>> {
		let index = crates_index::Index::new_cargo_default()?;
		Ok(index.crate_(&pkg))
	}

	async fn download_crate(pkg: String, version: String, checksum: &[u8; 32]) -> Result<Bytes> {
		let download_url = format!("https://crates.io/api/v1/crates/{}/{}/download", pkg, version);
		println!("Downloading crate `{}={}` from {}", pkg, version, download_url);
		let response = reqwest::get(&download_url).await?;

		let content_length = response.content_length();
		let bytes: Bytes = response.bytes().await?;

		println!(
			"Download size: {}",
			content_length.map_or("<unknown>".into(), |cl| format!("{} bytes", cl))
		);
		println!("Crate `{} v{}` downloaded successfully", pkg, version);
		Ok(bytes)
	}

	async fn download_and_extract_crate(
		tmp_dir: PathBuf,
		destination: PathBuf,
		name: String,
		version: String,
		checksum: &[u8; 32],
	) -> Result<()> {
		let crate_bytes = Self::download_crate(name.clone(), version.clone(), checksum).await?;

		println!("Extracting crate archive to {}/", tmp_dir.display());
		let gzip = flate2::read::GzDecoder::new(&crate_bytes[..]);
		let mut archive = tar::Archive::new(gzip);
		archive.unpack(tmp_dir.clone())?;

		let from = tmp_dir.join(format!("{}-{}", name, version));
		rename(from, destination)?;

		Ok(())
	}
}

pub fn get_registry(registry: String) -> Result<impl Registry> {
	match registry.as_str() {
		"crates" => Ok(CratesRegistry {}),
		_ => bail!("Unknown registry"),
	}
}

fn create_tmp_dir() -> io::Result<PathBuf> {
	let dir = env::temp_dir().join(format!("pkg-diff-{}", random_string()));
	fs::create_dir_all(dir.clone())?;
	Ok(dir)
}

fn random_string() -> String {
	thread_rng()
		.sample_iter(&Alphanumeric)
		.take(12)
		.map(char::from)
		.collect()
}