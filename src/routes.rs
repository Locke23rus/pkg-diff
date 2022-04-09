use axum::{
	extract::Path,
	http::StatusCode,
	response::{Html, IntoResponse},
};
use minijinja::context;
use patch::Patch;

use crate::{diff::File, templates::get_template};

pub async fn root() -> impl IntoResponse {
	let template = get_template("root.html");
	let html = template.render("").unwrap();
	Html(html).into_response()
}

pub async fn inspect(Path((registry, pkg, version)): Path<(String, String, String)>) -> impl IntoResponse {
	let diff = include_str!("../examples/minijinja.diff");

	match Patch::from_multiple(diff) {
		Ok(patches) => {
			let files: Vec<File> = patches.into_iter().map(|patch| File::from_patch(patch)).collect();
			let template = get_template("inspect.html");
			let html = template.render(context! { registry, pkg, version, files }).unwrap();
			Html(html).into_response()
		}
		Err(err) => (StatusCode::BAD_REQUEST, format!("Failed to parse diff: {}", err)).into_response(),
	}
}

pub async fn compare(Path((registry, pkg, v1, v2)): Path<(String, String, String, String)>) -> impl IntoResponse {
	let diff = include_str!("../examples/git.diff");

	match Patch::from_multiple(diff) {
		Ok(patches) => {
			let files: Vec<File> = patches.into_iter().map(|patch| File::from_patch(patch)).collect();
			let template = get_template("compare.html");
			let html = template.render(context! { registry, pkg, v1, v2, files }).unwrap();
			Html(html).into_response()
		}
		Err(err) => (StatusCode::BAD_REQUEST, format!("Failed to parse diff: {}", err)).into_response(),
	}
}
