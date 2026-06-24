use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;

use server::server_starter::start_axum_server;
use tauri::Manager;

// 1. Mock server initialization logic
async fn start_backend_server() {
    // Replace this with your actual Axum, Rocket, or Actix-web startup logic
    println!("Backend server starting up...");

    unsafe {
        let exe_dir = std::env::current_exe().expect("Failed to get exe path").parent().map(|p| p.to_path_buf()).unwrap();
        let mut path = PathBuf::from(exe_dir);
        path.push("_up_");
        path.push("site");
        let site_dir = path.to_str().unwrap();

        std::env::set_var("LEPTOS_OUTPUT_NAME", "dev_tools");
        std::env::set_var("LEPTOS_SITE_ROOT", site_dir);
        std::env::set_var("LEPTOS_SITE_ADDR", "127.0.0.1:3005");
    }

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3005);

    start_axum_server(Some(addr)).await.expect("Failed start backend server");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Initialize the autostart plugin and attach a hidden boot flag
        .plugin(tauri_plugin_autostart::Builder::new().args(["--autostart"]).build())
        .setup(|app| {
            // 2. Spawn the backend server on the Tokio runtime asynchronously
            tauri::async_runtime::spawn(async {
                start_backend_server().await;
            });

            // 3. Inspect command-line arguments passed by the OS
            let args: Vec<String> = std::env::args().collect();

            if args.contains(&"--autostart".to_string()) {
                // System startup boot: keep window hidden, keep tray active
                if let Some(window) = app.get_webview_window("main") {
                    window.hide().unwrap();
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
