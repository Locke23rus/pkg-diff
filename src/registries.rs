use std::{
	env,
	path::PathBuf,
	process::{Command, Stdio},
};

use anyhow::{bail, Result};
use async_trait::async_trait;
use bytes::Bytes;
use crates_index::Crate;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sha2::{Digest, Sha256};
use tokio::fs::{create_dir_all, read_to_string, rename};

#[async_trait]
pub trait Registry {
	async fn inspect(&self, pkg: &str, version: &str) -> Result<(String, bool)>;
}

struct CratesRegistry {}

#[async_trait]
impl Registry for CratesRegistry {
	async fn inspect(&self, pkg: &str, version: &str) -> Result<(String, bool)> {
		match Self::find_crate(&pkg)? {
			Some(crate_) => match crate_.versions().iter().find(|v| v.version() == version) {
				Some(crate_version) => {
					let tmp_dir = create_tmp_dir().await?;
					create_dir_all(tmp_dir.join("a")).await?;

					Self::download_and_extract_crate(
						&tmp_dir,
						tmp_dir.join("b"),
						&pkg,
						&version,
						crate_version.checksum(),
					)
					.await?;

					let diff = git_diff(&tmp_dir).await?;
					let yanked = crate_version.is_yanked();

					Ok((diff, yanked))
				}
				None => bail!("Version not found"),
			},
			None => bail!("Crate not found"),
		}
	}
}

impl CratesRegistry {
	fn find_crate(pkg: &str) -> Result<Option<Crate>> {
		let index = crates_index::Index::new_cargo_default()?;
		Ok(index.crate_(pkg))
	}

	async fn download_and_verify_crate(pkg: &str, version: &str, checksum: &[u8; 32]) -> Result<Bytes> {
		let download_url = format!("https://crates.io/api/v1/crates/{}/{}/download", pkg, version);
		let response = reqwest::get(&download_url).await?;
		let bytes = response.bytes().await?;
		let hash = Sha256::digest(bytes.as_ref());

		if hash[..] == checksum[..] {
			Ok(bytes)
		} else {
			bail!("Crate {} v{} checksum mismatch", pkg, version)
		}
	}

	async fn download_and_extract_crate(
		tmp_dir: &PathBuf,
		destination: PathBuf,
		name: &str,
		version: &str,
		checksum: &[u8; 32],
	) -> Result<()> {
		let crate_bytes = Self::download_and_verify_crate(name, version, checksum).await?;
		let gzip = flate2::read::GzDecoder::new(crate_bytes.as_ref());
		let mut archive = tar::Archive::new(gzip);
		archive.unpack(tmp_dir)?;

		let from = tmp_dir.join(format!("{}-{}", name, version));
		rename(from, destination).await?;

		Ok(())
	}
}

pub fn get_registry(registry: &str) -> Result<impl Registry> {
	match registry {
		"crates" => Ok(CratesRegistry {}),
		_ => bail!("Unknown registry"),
	}
}

async fn create_tmp_dir() -> Result<PathBuf> {
	let dir = env::temp_dir().join(format!("pkg-diff-{}", random_string()));
	create_dir_all(&dir).await?;
	Ok(dir)
}

fn random_string() -> String {
	thread_rng()
		.sample_iter(&Alphanumeric)
		.take(12)
		.map(char::from)
		.collect()
}

async fn git_diff(tmp_dir: &PathBuf) -> Result<String> {
	let diff_path = tmp_dir.join("pkg.diff");
	Command::new("git")
		.current_dir(tmp_dir)
		.stdout(Stdio::null())
		.stderr(Stdio::null())
		.arg("-c")
		.arg("diff.algorithm=histogram")
		.arg("diff")
		.arg("--no-color")
		.arg("--no-index")
		.arg("--no-prefix")
		.arg("--output")
		.arg(diff_path.as_os_str())
		.arg("a")
		.arg("b")
		.output()?;

	let diff = read_to_string(diff_path).await?;
	Ok(diff)
}
