mod diff;
mod routes;
mod templates;

use axum::{routing::get, Router};
use routes::{crates_show, root};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
	let app = Router::new()
		.route("/", get(root))
		.route("/crates/:pkg/:version", get(crates_show));
	let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
	axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}
