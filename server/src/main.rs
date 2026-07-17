use std::{
    env::{self, Args},
    net::SocketAddr,
};

use dotenvy::dotenv;
use log::info;
use server::server_starter::start_axum_server;
use tracing_log::LogTracer;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let environment = env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string());
    let env_file_name = format!(".env.{}", environment);
    println!("environment={}, env_file_name={}", environment, env_file_name);

    dotenv().ok();
    dotenvy::from_filename_override(env_file_name).ok();

    LogTracer::init().expect("Failed to set logger");

    let database_url = std::env::var("DATABASE_URL").ok();
    let remote_server_url = std::env::var("DEVTOOLS_REMOTE_SERVER_URL")
        .unwrap_or("https://dev-tools-rust.vercel.app".to_owned());

    info!("start_axum_server...");

    let args = std::env::args();
    let addr_arg = get_arg_value(args, "addr");

    let addr_v4 = match addr_arg {
        Some(addr) => match addr.parse::<SocketAddr>() {
            Ok(addr) => Some(addr),
            Err(err) => Err(err)?,
        },
        None => None,
    };

    start_axum_server(addr_v4, Some(remote_server_url), database_url).await
}

fn get_arg_value(mut args: Args, name: &str) -> Option<String> {
    let arg_search_str = format!("--{}=", name);
    args.find_map(|a| {
        if a.starts_with(&arg_search_str) {
            let str = &a[arg_search_str.len()..];
            Some(str.to_owned())
        } else {
            None
        }
    })
}
