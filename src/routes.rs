use axum::{
	extract::Path,
	http::StatusCode,
	response::{Html, IntoResponse},
};
use minijinja::context;
use patch::Patch;

use crate::{
	diff::File,
	registries::{get_registry, Registry},
	templates::get_template,
};

pub async fn root() -> impl IntoResponse {
	let template = get_template("root.html");
	let html = template.render("").unwrap();
	Html(html).into_response()
}

pub async fn inspect(Path((registry, pkg, version)): Path<(String, String, String)>) -> impl IntoResponse {
	match get_registry(&registry) {
		Ok(registry) => match registry.inspect(&pkg, &version).await {
			Ok((diff, yanked)) => match Patch::from_multiple(&diff) {
				Ok(patches) => {
					let files: Vec<File> = patches.into_iter().map(|patch| File::from_patch(patch)).collect();
					let ctx = context! { pkg, version, yanked, files };
					let template = get_template("inspect.html");
					match template.render(ctx) {
						Ok(html) => Html(html).into_response(),
						Err(e) => (
							StatusCode::INTERNAL_SERVER_ERROR,
							format!("Failed to render template: {e}"),
						)
							.into_response(),
					}
				}
				Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse diff: {e}")).into_response(),
			},
			Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}")).into_response(),
		},
		Err(_) => (StatusCode::NOT_FOUND, "Page not found").into_response(),
	}
}

pub async fn compare(Path((registry, pkg, v1, v2)): Path<(String, String, String, String)>) -> impl IntoResponse {
	let diff = include_str!("../examples/git.diff");
	let v1_yanked = true;
	let v2_yanked = false;

	match Patch::from_multiple(diff) {
		Ok(patches) => {
			let files: Vec<File> = patches.into_iter().map(|patch| File::from_patch(patch)).collect();
			let template = get_template("compare.html");
			let ctx = context! { registry, pkg, v1, v1_yanked, v2, v2_yanked, files };
			match template.render(ctx) {
				Ok(html) => Html(html).into_response(),
				Err(err) => (
					StatusCode::INTERNAL_SERVER_ERROR,
					format!("Failed to render template: {}", err),
				)
					.into_response(),
			}
		}
		Err(err) => (
			StatusCode::INTERNAL_SERVER_ERROR,
			format!("Failed to parse diff: {}", err),
		)
			.into_response(),
	}
}
