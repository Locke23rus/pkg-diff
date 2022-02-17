use axum::{
	extract::Path,
	http::StatusCode,
	response::{Html, IntoResponse},
};
use minijinja::context;
use patch::Patch;

use crate::{diff::File, templates::get_template};

pub async fn crates_show(Path((pkg, version)): Path<(String, String)>) -> impl IntoResponse {
	// let diff = include_str!("../../static/git.diff");
	let diff = include_str!("../../static/minijinja.diff");

	match Patch::from_multiple(diff) {
		Ok(patches) => {
			let files: Vec<File> = patches.into_iter().map(|patch| File::from_patch(patch)).collect();
			let template = get_template("crates_show.html");
			let html = template.render(context! { pkg, version, files }).unwrap();
			Html(html).into_response()
		}
		Err(err) => (StatusCode::BAD_REQUEST, format!("Failed to parse diff: {}", err)).into_response(),
	}
}
