use axum::{extract::Path, http::StatusCode, response::IntoResponse};

pub async fn crates_show(Path((_pkg, _version)): Path<(String, String)>) -> impl IntoResponse {
    // (StatusCode::OK, format!("{} {}", pkg, version))

    let diff = include_str!("../../static/git.diff").to_string();
    (StatusCode::OK, diff)
}
