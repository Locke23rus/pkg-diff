use std::{env, path::PathBuf, process::Stdio};

use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use bytes::Bytes;
use crates_index::{Crate, Version};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sha2::{Digest, Sha256};
use tokio::{
	fs::{create_dir_all, read_to_string, remove_dir_all, rename},
	process::Command,
	try_join,
};

static USER_AGENT: &str = "pkg-diff (https://github.com/locke23rus/pkg-diff)";

#[async_trait]
pub trait Registry {
	async fn inspect(&self, pkg: &str, version: &str) -> Result<(String, bool)>;
	async fn compare(&self, pkg: &str, v1: &str, v2: &str) -> Result<(String, bool, bool)>;
}

struct CratesRegistry {}

#[async_trait]
impl Registry for CratesRegistry {
	async fn inspect(&self, pkg: &str, version: &str) -> Result<(String, bool)> {
		let crate_ = Self::find_crate(pkg)?;
		let crate_version = Self::find_version(&crate_, version)?;

		let tmp_dir = create_tmp_dir().await?;
		create_dir_all(tmp_dir.join("a")).await?;

		Self::download_and_extract_crate(&tmp_dir, tmp_dir.join("b"), pkg, version, crate_version.checksum()).await?;

		let diff = git_diff(&tmp_dir).await?;
		remove_dir_all(tmp_dir).await?;

		Ok((diff, crate_version.is_yanked()))
	}

	async fn compare(&self, pkg: &str, v1: &str, v2: &str) -> Result<(String, bool, bool)> {
		let crate_ = Self::find_crate(pkg)?;
		let crate_v1 = Self::find_version(&crate_, v1)?;
		let crate_v2 = Self::find_version(&crate_, v2)?;

		let tmp_dir = create_tmp_dir().await?;

		try_join!(
			Self::download_and_extract_crate(&tmp_dir, tmp_dir.join("a"), pkg, v1, crate_v1.checksum()),
			Self::download_and_extract_crate(&tmp_dir, tmp_dir.join("b"), pkg, v2, crate_v2.checksum()),
		)?;

		let diff = git_diff(&tmp_dir).await?;
		remove_dir_all(&tmp_dir).await?;

		Ok((diff, crate_v1.is_yanked(), crate_v2.is_yanked()))
	}
}

impl CratesRegistry {
	fn find_crate(pkg: &str) -> Result<Crate> {
		let index = crates_index::Index::new_cargo_default()?;
		index.crate_(pkg).ok_or_else(|| anyhow!("Crate '{}' not found", pkg))
	}

	fn find_version<'a>(crate_: &'a Crate, version: &str) -> Result<&'a Version> {
		crate_
			.versions()
			.iter()
			.find(|v| v.version() == version)
			.ok_or_else(|| {
				anyhow!(
					"Version '{}' not found\nAvailable versions:\n{}",
					version,
					crate_
						.versions()
						.iter()
						.map(|v| format!(
							"<a href=\"/crates/{}/{}\">{}</a>",
							crate_.name(),
							v.version(),
							v.version()
						))
						.collect::<Vec<_>>()
						.join("\n")
				)
			})
	}

	async fn download_and_verify_crate(pkg: &str, version: &str, checksum: &[u8; 32]) -> Result<Bytes> {
		let download_url = format!("https://crates.io/api/v1/crates/{}/{}/download", pkg, version);
		let client = build_http_client()?;
		let response = client.get(&download_url).send().await?;
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
		.arg("core.quotepath=false")
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
		.status()
		.await?;

	let diff = read_to_string(diff_path).await?;
	Ok(diff)
}

fn build_http_client() -> Result<reqwest::Client, reqwest::Error> {
	reqwest::Client::builder().user_agent(USER_AGENT).build()
}
