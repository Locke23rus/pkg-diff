mod diff;
mod registries;
mod routes;
mod templates;

use axum::{routing::get, Router};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
	let app = Router::new()
		.route("/", get(routes::index))
		.route("/:registry/:pkg/:v1", get(routes::inspect))
		.route("/:registry/:pkg/:v1/:v2", get(routes::compare))
		.fallback(routes::handler_404);
	let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
	axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}
