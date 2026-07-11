use std::net::SocketAddr;
use std::thread;

use leptos::prelude::*;
use log::{error, info};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::app_router::build_app_router::build_app_router;
use crate::db::create_pool;

pub async fn start_axum_server(custom_addr: Option<SocketAddr>, remote_server_url: Option<String>) -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_ansi(true)
        //.with_file(true)
        .with_line_number(true)
        // Apply the EnvFilter to use RUST_LOG
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Could not set subscriber");

    match thread::available_parallelism() {
        Ok(n) => info!("Available parallelism: {}", n),
        Err(e) => error!("Error getting parallelism: {}", e),
    }

    let mut conf = get_configuration(None)?;
    let addr = match custom_addr {
        Some(custom_addr) => custom_addr,
        None => conf.leptos_options.site_addr,
    };
    conf.leptos_options.site_addr = addr;

    let pool = create_pool().await;

    let app = build_app_router(conf, pool, remote_server_url).await?;
    info!("listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
