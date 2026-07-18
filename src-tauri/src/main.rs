// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;

#[derive(Parser)]
#[command(name = "Dev Tools")]
#[command(version = "0.4.0")]
#[command(about = "WebDev Useful Tools", long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "PORT", help="Server port. Example \"--PORT 3005\"")]
    port: Option<u16>,
    #[arg(long, value_name = "REMOTE_SERVER_URL", help="Remote server address. Only for the \"Share File\" feature or if the local server won't start. Defaults to \"https://dev-tools-rust.vercel.app\".")]
    remote_server_url: Option<String>,
    #[arg(long, help="Do not run local server")]
    no_start_server: Option<bool>,
}

fn main() {
    let cli = Cli::parse();

    webdev_useful_tools_lib::run(cli.port, cli.remote_server_url, cli.no_start_server.unwrap_or(false))
}
