use std::{
    env::{self},
    net::SocketAddr,
};

use app::common::constants::REMOTE_SERVER_HOST;
use clap::Parser;
use dotenvy::dotenv;
use server::server_starter::start_axum_server;
use tracing::info;

#[derive(Parser)]
#[command(name = "Dev Tools Server")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "WebDev Useful Tools Server", long_about = None)]
struct Cli {
    #[arg(long, value_name = "ADDR", help = "Server socket addr. Example \"--ADDR 0.0.0.0:3005\"")]
    addr: Option<String>,
    #[arg(long, value_name = "DATABASE_URL", help = "Postgres connection url")]
    database_url: Option<String>,
    #[arg(
        long,
        value_name = "REMOTE_SERVER_URL",
        help = "Remote server address. Only for the \"Share File\" feature and if the server is running without a database. Defaults to \"https://dev-tools-rust.vercel.app\"."
    )]
    remote_server_url: Option<String>,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let environment = env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string());
    let env_file_name = format!(".env.{}", environment);
    println!("environment={}, env_file_name={}", environment, env_file_name);

    dotenv().ok();
    dotenvy::from_filename_override(env_file_name).ok();

    let database_url = match cli.database_url {
        Some(database_url) => Some(database_url),
        None => std::env::var("DATABASE_URL").ok(),
    };

    let remote_server_url = match cli.remote_server_url {
        Some(remote_server_url) => remote_server_url,
        None => std::env::var("DEVTOOLS_REMOTE_SERVER_URL")
            .unwrap_or(format!("https://{}", REMOTE_SERVER_HOST)),
    };

    let addr_v4 = match cli.addr {
        Some(addr) => match addr.parse::<SocketAddr>() {
            Ok(addr) => Some(addr),
            Err(err) => Err(err)?,
        },
        None => None,
    };

    let rc_max_content_length = match std::env::var("RC_MAX_CONTENT_LENGTH") {
        Ok(s) => s.parse()?,
        Err(_) => u64::MAX,
    };

    let rest_client_proxy_allow_ips = match std::env::var("REST_CLIENT_PROXY_ALLOW_IPS") {
        Ok(s) => s.split(',').map(|v|v.to_owned()).collect(),
        Err(_) => Vec::new(),
    };

    info!("start_axum_server...");
    start_axum_server(addr_v4, Some(remote_server_url), database_url, rc_max_content_length, rest_client_proxy_allow_ips).await
}
