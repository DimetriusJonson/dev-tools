use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::thread;

use tracing::{error, info};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::app_router::build_app_router::build_app_router;
use crate::db::create_pool;

pub async fn start_axum_server(
    custom_addr: Option<SocketAddr>,
    remote_server_url: Option<String>,
    database_url: Option<String>,
    dist_dir: String,
) -> anyhow::Result<()> {
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

    let addr = match custom_addr {
        Some(custom_addr) => custom_addr,
        None => SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 3000),
    };

    let pool = match database_url {
        Some(database_url) => Some(create_pool(database_url).await),
        None => None,
    };

    let app = build_app_router(addr, pool, remote_server_url, dist_dir).await?;
    info!("listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
