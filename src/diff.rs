use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Line {
	/// A line added to the old file in the new file
	Add {
		text: String,
		from_line_number: String,
		to_line_number: String,
	},
	/// A line removed from the old file in the new file
	Remove {
		text: String,
		from_line_number: String,
		to_line_number: String,
	},
	/// A line provided for context in the diff (unchanged); from both the old and the new file
	Context {
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
}

impl Chunk {
	pub fn from_hunk(hunk: patch::Hunk) -> Self {
		let context: &str;
		let old_lines_count = hunk
			.clone()
			.lines
			.into_iter()
			.filter(|line| match line {
				patch::Line::Add(_) => false,
				_ => true,
			})
			.count() as u64;

		let mut patch_lines = hunk.lines;

		if old_lines_count > hunk.old_range.count {
			if let patch::Line::Context(ctx) = patch_lines[0] {
				context = ctx;
				patch_lines.remove(0);
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

		let mut from_line_number = hunk.old_range.start;
		let mut to_line_number = hunk.new_range.start;

		let lines: Vec<Line> = patch_lines
			.into_iter()
			.map(|patch_line| match patch_line {
				patch::Line::Add(text) => {
					let line = Line::Add {
						text: text.to_string(),
						from_line_number: "".to_string(),
						to_line_number: to_line_number.to_string(),
					};

					to_line_number += 1;

					line
				}
				patch::Line::Context(text) => {
					let line = Line::Context {
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
