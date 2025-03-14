pub mod config;
pub mod images;

use clap::Parser;
use config::Config;
use images::get_images_router;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let config = Config::parse();
    let images_router = get_images_router(&config);

    let app = images_router.layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(CorsLayer::new().allow_origin(Any)),
    );

    let port = config.port;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("Listening on http://localhost:{}", port);

    axum::serve(listener, app).await.unwrap();
}
