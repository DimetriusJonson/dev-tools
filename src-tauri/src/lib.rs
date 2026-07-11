use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;

use dotenvy::dotenv;
use log::info;
use server::server_starter::start_axum_server;
use tauri::{AppHandle, Manager};
use tauri::{Url, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_log::{Target, TargetKind};

#[tauri::command]
fn get_resource_dir(app_handle: &AppHandle) -> PathBuf {
    app_handle.path().resource_dir().unwrap()
}

async fn start_backend_server(port: u16, resource_dir: PathBuf, remote_server_url: String) {
    let addr = format!("0.0.0.0:{}", port);

    info!("Backend server starting up on {}...", addr);

    unsafe {
        let mut site_dir = resource_dir;
        site_dir.push("_up_");
        site_dir.push("site");

        std::env::set_var("LEPTOS_OUTPUT_NAME", "dev_tools");
        std::env::set_var("LEPTOS_SITE_ROOT", site_dir);
        std::env::set_var("LEPTOS_SITE_ADDR", addr);
    }

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);

    start_axum_server(Some(addr), Some(remote_server_url))
        .await
        .expect("Failed start backend server");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let environment = env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string());
    let env_file_name = format!(".env.{}", environment);
    println!("environment={}, env_file_name={}", environment, env_file_name);

    dotenv().ok();
    dotenvy::from_filename_override(env_file_name).ok();

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::Builder::new().args(["--autostart"]).build())
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),    
                    Target::new(TargetKind::LogDir { file_name: Some("dev_tools.log".to_owned()) }), 
                    Target::new(TargetKind::Webview),
                ])
                .build(),
        )
        .setup(|app| {
            let args: Vec<String> = std::env::args().collect();

            let port = match get_arg_value(&args, "port") {
                Some(port) => port.parse::<u16>().unwrap(),
                None => 3005,
            };

            let arg_remote_server_url = get_arg_value(&args, "remote-server-url")
                .unwrap_or("https://dev-tools-rust.vercel.app".to_owned());

            let resource_dir = get_resource_dir(app.app_handle());

            tauri::async_runtime::spawn(async move {
                start_backend_server(port, resource_dir, arg_remote_server_url).await;
            });

            if args.contains(&"--autostart".to_string()) {
                if let Some(window) = app.get_webview_window("main") {
                    window.hide().unwrap();
                }
            }

            let server_url = format!("http://127.0.0.1:{}", port);

            let target_url = Url::parse(&server_url).expect("Failed to parse server URL");

            let _window = WebviewWindowBuilder::new(app, "main", WebviewUrl::External(target_url))
                .title("Developer Tools")
                .inner_size(1500.0, 1000.0)
                .enable_clipboard_access()
                .disable_drag_drop_handler()
                .build()
                .expect("Failed to build dynamic window");

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn get_arg_value(args: &Vec<String>, name: &str) -> Option<String> {
    let arg_search_str = format!("--{}=", name);
    args.iter().find_map(|a| {
        if a.starts_with(&arg_search_str) {
            let str = &a[arg_search_str.len()..];
            Some(str.to_owned())
        } else {
            None
        }
    })
}
