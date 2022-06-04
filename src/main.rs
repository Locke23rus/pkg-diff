mod diff;
mod registries;
mod routes;
mod templates;

use axum::{handler::Handler, routing::get, Router};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
	let app = Router::new()
		.route("/", get(routes::index))
		.route("/inspect/:registry/:pkg/:version", get(routes::inspect))
		.route("/compare/:registry/:pkg/:v1/:v2", get(routes::compare))
		.fallback(routes::handler_404.into_service());
	let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
	axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}
