use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Line {
	/// A line added to the old file in the new file
	Add {
		kind: String,
		text: String,
		from_line_number: String,
		to_line_number: String,
	},
	/// A line removed from the old file in the new file
	Remove {
		kind: String,
		text: String,
		from_line_number: String,
		to_line_number: String,
	},
	/// A line provided for context in the diff (unchanged); from both the old and the new file
	Context {
		kind: String,
		text: String,
		from_line_number: String,
		to_line_number: String,
	},
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Chunk {
	header: String,
	lines: Vec<Line>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct File {
	name: String,
	status: String,
	chunks: Vec<Chunk>,
	digest: String,
}

impl Chunk {
	pub fn from_hunk(hunk: patch::Hunk) -> Self {
		let mut from_line_number = hunk.old_range.start;
		let mut to_line_number = hunk.new_range.start;

		let header = format!("@@ -{} +{} @@{}", hunk.old_range, hunk.new_range, hunk.range_hint);
		let lines: Vec<Line> = hunk
			.lines
			.into_iter()
			.map(|patch_line| match patch_line {
				patch::Line::Add(text) => {
					let line = Line::Add {
						kind: "add".to_string(),
						text: text.to_string(),
						from_line_number: "".to_string(),
						to_line_number: to_line_number.to_string(),
					};

					to_line_number += 1;

					line
				}
				patch::Line::Context(text) => {
					let line = Line::Context {
						kind: "context".to_string(),
						text: text.to_string(),
						from_line_number: from_line_number.to_string(),
						to_line_number: to_line_number.to_string(),
					};

					from_line_number += 1;
					to_line_number += 1;

					line
				}
				patch::Line::Remove(text) => {
					let line = Line::Remove {
						kind: "remove".to_string(),
						text: text.to_string(),
						from_line_number: from_line_number.to_string(),
						to_line_number: "".to_string(),
					};

					from_line_number += 1;

					line
				}
			})
			.collect();

		Self { header, lines }
	}
}

impl File {
	pub fn from_patch(patch: patch::Patch) -> Self {
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
			name = format!("{} -> {}", &patch.old.path[2..], &patch.new.path[2..]);
		}

		let chunks: Vec<Chunk> = patch.hunks.into_iter().map(Chunk::from_hunk).collect();
		let digest: String = format!("{:x}", md5::compute(name.clone()));

		Self {
			name,
			status,
			chunks,
			digest,
		}
	}
}
