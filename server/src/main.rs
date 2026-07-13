use std::env;

use dotenvy::dotenv;
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

    start_axum_server(None, Some("https://dev-tools-rust.vercel.app".to_owned()), database_url).await
}
