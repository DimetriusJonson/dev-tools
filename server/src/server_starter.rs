use std::{env, net::SocketAddr};
use std::thread;

use dotenvy::dotenv;
use leptos::prelude::*;
use log::{error, info};
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::app_router::build_app_router::build_app_router;

pub async fn start_axum_server(custom_addr: Option<SocketAddr>) -> anyhow::Result<()> {
    let environment = env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string());
    let env_file_name = format!(".env.{}", environment);
    println!("environment={}, env_file_name={}", environment, env_file_name);

    dotenv().ok();
    dotenvy::from_filename_override(env_file_name).ok();

    LogTracer::init().expect("Failed to set logger");

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

    let conf = get_configuration(None)?;
    let addr = match custom_addr {
        Some(custom_addr) => custom_addr,
        None => conf.leptos_options.site_addr,
    };

    let app = build_app_router(conf).await?;
    info!("listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
