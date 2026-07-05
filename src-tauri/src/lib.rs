use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;

use server::server_starter::start_axum_server;
use tauri::{AppHandle, Manager};
use tauri::{Url, WebviewUrl, WebviewWindowBuilder};

#[tauri::command]
fn get_resource_dir(app_handle: &AppHandle) -> PathBuf {
    app_handle
        .path()
        .resource_dir().unwrap()
}

async fn start_backend_server(port: u16, resource_dir: PathBuf) {
    let addr = format!("127.0.0.1:{}", port);

    println!("Backend server starting up on {}...", addr);

    unsafe {
        let mut site_dir = resource_dir;
        site_dir.push("_up_");
        site_dir.push("site");

        std::env::set_var("LEPTOS_OUTPUT_NAME", "dev_tools");
        std::env::set_var("LEPTOS_SITE_ROOT", site_dir);
        std::env::set_var("LEPTOS_SITE_ADDR", addr);
    }

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);

    start_axum_server(Some(addr)).await.expect("Failed start backend server");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::Builder::new().args(["--autostart"]).build())
        .setup(|app| {
            let args: Vec<String> = std::env::args().collect();
            let arg_port = args.iter().find_map(|a| {
                if a.starts_with("--port=") {
                    let str = &a[7..].parse::<u16>().unwrap();
                    Some(*str)
                } else {
                    None
                }
            });

            let port = arg_port.unwrap_or(3005);
            let resource_dir = get_resource_dir(app.app_handle());
            println!("resource_dir={}", resource_dir.to_str().unwrap());

            tauri::async_runtime::spawn(async move {
                start_backend_server(port, resource_dir).await;
            });

            if args.contains(&"--autostart".to_string()) {
                if let Some(window) = app.get_webview_window("main") {
                    window.hide().unwrap();
                }
            }

            let server_url = format!("http://127.0.0.1:{}", port);

            let target_url = Url::parse(&server_url).expect("Failed to parse server URL");

            let _window = WebviewWindowBuilder::new(
                app,
                "main", 
                WebviewUrl::External(target_url),
            )
            .title("Developer Tools")
            .inner_size(1500.0, 1000.0)
            .enable_clipboard_access()
            .build()
            .expect("Failed to build dynamic window");

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
