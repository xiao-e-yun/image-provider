use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use console::style;
use local_ip_address::local_ip;
use log::info;
use qrcode::{render::unicode, QrCode};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

use image_provider::{get_images_router, ResizeConfig};

#[derive(Debug, Clone, Parser)]
pub struct Config {
    #[clap(long, short, default_value = "3000")]
    port: u16,
    #[clap(flatten)]
    resize: ResizeConfig,
    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,
}

#[tokio::main]
async fn main() {
    let config = Config::parse();
    env_logger::builder()
        .filter_level(config.verbose.log_level_filter())
        .format_target(false)
        .init();

    let images_router = get_images_router(config.resize);

    let app = images_router.layer(ServiceBuilder::new().layer(CorsLayer::new().allow_origin(Any)));

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    show_urls(config.port);

    info!(
        "Press {} to stop the server",
        style("Ctrl + C").green().bold()
    );

    axum::serve(listener, app).await.unwrap();
}

pub fn show_urls(port: u16) {
    info!(
        " {} http://localhost:{} ",
        style("Local").green().bold(),
        port
    );

    if let Ok(addr) = local_ip() {
        let url = format!("http://{addr}:{port}");
        info!(" {} {}", style("Network").green().bold(), url);

        let qrcode = QrCode::new(url.clone()).unwrap();
        let qrcode = qrcode
            .render::<unicode::Dense1x2>()
            .dark_color(unicode::Dense1x2::Dark)
            .light_color(unicode::Dense1x2::Light)
            .quiet_zone(false)
            .build();

        info!("");
        for line in qrcode.lines() {
            info!(" {line}");
        }
        info!("");
    }
}
