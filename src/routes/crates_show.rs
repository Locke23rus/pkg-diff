use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};
use minijinja::context;

use crate::templates::get_template;

pub async fn crates_show(Path((pkg, version)): Path<(String, String)>) -> impl IntoResponse {
    let diff = include_str!("../../static/git.diff").to_string();
    let template = get_template("crates_show.html");
    let html = template.render(context! { pkg, version, diff }).unwrap();
    Html(html).into_response()
}
