use std::{
    env::{self},
    net::SocketAddr,
};

use clap::Parser;
use dotenvy::dotenv;
use server::server_starter::start_axum_server;

pub const REMOTE_SERVER_HOST: &str = "dev-tools-rust.vercel.app";

#[derive(Parser)]
#[command(name = "Dev Tools Server")]
#[command(version = "0.5.0")]
#[command(about = "WebDev Useful Tools Server", long_about = None)]
struct Cli {
    #[arg(long, value_name = "ADDR", help="Server socket addr. Example \"--ADDR 0.0.0.0:3005\"")]
    addr: Option<String>,
    #[arg(long, value_name = "DATABASE_URL", help="Postgres connection url")]
    database_url: Option<String>,
    #[arg(long, value_name = "REMOTE_SERVER_URL", help="Remote server address. Only for the \"Share File\" feature and if the server is running without a database. Defaults to \"https://dev-tools-rust.vercel.app\".")]
    remote_server_url: Option<String>,
    #[arg(long, value_name = "DIST_DIR", help="Static files directory. Defaults to \"dist\".")]
    dist_dir: Option<String>,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let environment = env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string());
    let env_file_name = format!(".env.{}", environment);
    println!("environment={}, env_file_name={}", environment, env_file_name);

    dotenv().ok();
    dotenvy::from_filename_override(env_file_name).ok();

    let dist_dir = match cli.dist_dir {
        Some(dist_dir) => dist_dir,
        None => std::env::var("DIST_DIR").unwrap_or("dist".to_owned()),
    };

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

    start_axum_server(addr_v4, Some(remote_server_url), database_url, dist_dir).await
}
