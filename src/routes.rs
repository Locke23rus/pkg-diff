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

pub async fn index() -> impl IntoResponse {
	let template = get_template("index.html");
	match template.render("") {
		Ok(html) => Html(html).into_response(),
		Err(e) => render_error(
			StatusCode::INTERNAL_SERVER_ERROR,
			format!("Failed to render template: {e}"),
		)
		.into_response(),
	}
}

pub async fn inspect(Path((registry, pkg, version)): Path<(String, String, String)>) -> impl IntoResponse {
	match get_registry(&registry) {
		Ok(registry) => match registry.inspect(&pkg, &version).await {
			Ok((diff, yanked)) => match Patch::from_multiple(&diff) {
				Ok(patches) => {
					let files: Vec<File> = patches.into_iter().map(File::from_patch).collect();
					let ctx = context! { pkg, version, yanked, files };
					let template = get_template("inspect.html");
					match template.render(ctx) {
						Ok(html) => Html(html).into_response(),
						Err(e) => render_error(
							StatusCode::INTERNAL_SERVER_ERROR,
							format!("Failed to render template: {e}"),
						)
						.into_response(),
					}
				}
				Err(e) => render_error(StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse diff: {e}"))
					.into_response(),
			},
			Err(e) => render_error(StatusCode::BAD_REQUEST, format!("{e}")).into_response(),
		},
		Err(_) => render_error(StatusCode::NOT_FOUND, "Registry not found".to_string()).into_response(),
	}
}

pub async fn compare(Path((registry, pkg, v1, v2)): Path<(String, String, String, String)>) -> impl IntoResponse {
	match get_registry(&registry) {
		Ok(registry) => match registry.compare(&pkg, &v1, &v2).await {
			Ok((diff, v1_yanked, v2_yanked)) => match Patch::from_multiple(&diff) {
				Ok(patches) => {
					let files: Vec<File> = patches.into_iter().map(File::from_patch).collect();
					let template = get_template("compare.html");
					let ctx = context! { pkg, v1, v1_yanked, v2, v2_yanked, files };
					match template.render(ctx) {
						Ok(html) => Html(html).into_response(),
						Err(err) => render_error(
							StatusCode::INTERNAL_SERVER_ERROR,
							format!("Failed to render template: {}", err),
						)
						.into_response(),
					}
				}
				Err(err) => render_error(
					StatusCode::INTERNAL_SERVER_ERROR,
					format!("Failed to parse diff: {}", err),
				)
				.into_response(),
			},
			Err(e) => render_error(StatusCode::BAD_REQUEST, format!("{e}")).into_response(),
		},
		Err(_) => render_error(StatusCode::NOT_FOUND, "Registry not found".to_string()).into_response(),
	}
}

pub async fn handler_404() -> impl IntoResponse {
	render_error(StatusCode::NOT_FOUND, "Page not found".to_string())
}

pub fn render_error(code: StatusCode, error: String) -> impl IntoResponse {
	let template = get_template("error.html");
	let ctx = context! { code => code.as_str(), error };
	match template.render(ctx) {
		Ok(html) => Html(html).into_response(),
		Err(e) => (
			StatusCode::INTERNAL_SERVER_ERROR,
			format!("Oops, something went wrong: {e}"),
		)
			.into_response(),
	}
}
