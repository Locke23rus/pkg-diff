use axum::response::{Html, IntoResponse};

use crate::templates::get_template;

pub async fn root() -> impl IntoResponse {
	let template = get_template("root.html");
	let html = template.render("").unwrap();
	Html(html).into_response()
}
