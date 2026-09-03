use std::net::SocketAddr;
use std::thread;

use anyhow::anyhow;
use leptos::prelude::*;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::app_router::build_app_router::build_app_router;
use crate::app_router::dump_receiver::start_dump_receiver;
use crate::db::create_pool;

pub async fn start_axum_server(
    custom_addr: Option<SocketAddr>,
    remote_server_url: Option<String>,
    database_url: Option<String>,
    rc_max_content_length: u64,
    rest_client_proxy_allow_ips: Vec<String>,
) -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_ansi(true)
        //.with_file(true)
        .with_line_number(true)
        // Apply the EnvFilter to use RUST_LOG
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

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

    info!("conf={:?}", conf);

    let pool = match database_url {
        Some(database_url) => Some(create_pool(database_url).await),
        None => None,
    };

    let dump_port = match start_dump_receiver().await {
        Ok(r) => r.1,
        Err(err) => return Err(anyhow!("Cant start dump receiver: {}", err)),
    };

    let app = build_app_router(
        conf,
        pool,
        remote_server_url,
        dump_port,
        rc_max_content_length,
        rest_client_proxy_allow_ips,
    )
    .await?;
    info!("listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    match axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await {
        Ok(_) => Ok(()),
        Err(err) => Err(anyhow!("Cant create axum server: {}", err)),
    }
}
