use patch::{Hunk, Line, Patch};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Chunk {
	header: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct File {
	name: String,
	status: String,
	chunks: Vec<Chunk>,
}

impl Chunk {
	pub fn from_hunk(hunk: Hunk) -> Self {
		let context: &str;
		let old_lines_count = hunk
			.clone()
			.lines
			.into_iter()
			.filter(|line| match line {
				Line::Add(_) => false,
				_ => true,
			})
			.count() as u64;

		if old_lines_count > hunk.old_range.count {
			if let Line::Context(ctx) = hunk.lines[0] {
				context = ctx;
			} else {
				context = "";
			}
		} else {
			context = "";
		}

		let header = format!(
			"@@ -{},{} +{},{} @@ {}",
			hunk.old_range.start, hunk.old_range.count, hunk.new_range.start, hunk.new_range.count, context
		);

		Self { header }
	}
}

impl File {
	pub fn from_patch(patch: Patch) -> Self {
		let status: String;
		let name: String;

		if patch.old.path == "/dev/null" {
			status = "added".to_string();
			name = patch.new.path[2..].to_string();
		} else if patch.new.path == "/dev/null" {
			status = "removed".to_string();
			name = patch.old.path[2..].to_string();
		} else if patch.old.path[2..] == patch.new.path[2..] {
			status = "changed".to_string();
			name = patch.new.path[2..].to_string();
		} else {
			status = "renamed".to_string();
			name = format!(
				"{} -> {}",
				patch.old.path[2..].to_string(),
				patch.new.path[2..].to_string()
			);
		}

		let chunks: Vec<Chunk> = patch.hunks.into_iter().map(|hunk| Chunk::from_hunk(hunk)).collect();

		Self { name, status, chunks }
	}
}
