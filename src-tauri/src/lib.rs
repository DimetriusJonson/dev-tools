use std::net::SocketAddr;
use std::path::PathBuf;
use std::{env, fs};

use log::{LevelFilter, error, info};
use server::server_starter::start_axum_server;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, WindowEvent};
use tauri::{Url, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_updater::UpdaterExt;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
struct ServerTask(tauri::async_runtime::JoinHandle<()>);

fn start_backend_server(
    app_handle: &AppHandle,
    port: u16,
    resource_dir: PathBuf,
    remote_server_url: String,
) -> Result<(), String> {
    let addr = format!("0.0.0.0:{}", port);
    let addr_v4 = match addr.parse::<SocketAddr>() {
        Ok(addr) => Some(addr),
        Err(err) => Err(err.to_string())?,
    };

    info!("Backend server starting up on {}...", addr);

    let mut site_dir = resource_dir;
    site_dir.push("_up_");
    site_dir.push("site");

    let app_handle = app_handle.clone();

    unsafe {
        std::env::set_var("LEPTOS_OUTPUT_NAME", "dev_tools");
        std::env::set_var("LEPTOS_SITE_ADDR", &addr);
        std::env::set_var("LEPTOS_SITE_ROOT", site_dir);
    };

    let server_task = tauri::async_runtime::spawn(async move {
        start_axum_server(
            addr_v4,
            Some(remote_server_url),
            None,
            u64::MAX,
            vec!["127.0.0.1".to_owned()],
        )
        .await
        .unwrap();
    });
    app_handle.manage(ServerTask(server_task));

    // Wait for the server to be ready before navigating
    tauri::async_runtime::block_on(async {
        for _ in 0..50 {
            if tokio::net::TcpStream::connect(&addr).await.is_ok() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    });

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run(port: Option<u16>, remote_server_url: Option<String>, no_start_server: bool) {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_autostart::Builder::new().args(["--autostart"]).build())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(LevelFilter::Info)
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir {
                        file_name: Some("webdev_useful_tools.log".to_owned()),
                    }),
                    Target::new(TargetKind::Webview),
                ])
                .build(),
        )
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(move |app| {
            let port = port.unwrap_or(3005);
            let remote_server_url =
                remote_server_url.unwrap_or("https://dev-tools-rust.vercel.app".to_owned());

            let resource_dir = app.app_handle().path().resource_dir()?;

            let server_url;
            if no_start_server {
                server_url = remote_server_url.to_owned();
            } else {
                server_url = format!("http://127.0.0.1:{}", port);
                start_backend_server(app.app_handle(), port, resource_dir, remote_server_url)?;
            }

            let app_title = format!(
                "{} {}",
                app.config().product_name.as_ref().ok_or("No product name in config!")?,
                APP_VERSION
            );

            let target_url = Url::parse(&server_url)?;
            let _window = WebviewWindowBuilder::new(app, "main", WebviewUrl::External(target_url))
                .title(app_title.to_owned())
                .inner_size(1500.0, 1000.0)
                .enable_clipboard_access()
                .disable_drag_drop_handler()
                .build()?;

            let quit_i = MenuItem::with_id(app, "quit", "Exit", true, None::<&str>)?;
            let open_i = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .tooltip(&app_title)
                .icon(app.default_window_icon().ok_or("Failed get default window icon")?.clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event({
                    move |app, event| match event.id.as_ref() {
                        "quit" => {
                            app.exit(0);
                        }
                        "open" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.unminimize();
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::DoubleClick { button: MouseButton::Left, .. } = event {
                        let app_handle = tray.app_handle();
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Spawn background task to check for update automatically on start
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                match app_handle
                    .updater_builder()
                    .on_before_exit({
                        let app_handle = app_handle.clone();
                        move || {
                            if let Some(task) = app_handle.try_state::<ServerTask>() {
                                task.0.abort();
                                println!("Backend server terminated successfully.");
                            }
                            info!("Clear cache...");
                            clear_webview_cache(&app_handle);
                        }
                    })
                    .build()
                {
                    Ok(updater) => {
                        if let Ok(Some(update)) = updater.check().await {
                            match update.download_and_install(|_, _| {}, || {}).await {
                                Ok(_) => {
                                    info!("Update downloaded successfully. Restarting...");
                                    app_handle.restart();
                                }
                                Err(err) => error!("Failed to download update: {}", err),
                            }
                        }
                    }
                    Err(err) => error!("Cant build updater {}", err),
                };
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                if let Some(task) = window.try_state::<ServerTask>() {
                    task.0.abort();
                    println!("Backend server terminated successfully.");
                }
                //let _ = window.minimize();
                let _ = window.hide();
            }
        });

    app.run(tauri::generate_context!())
        .unwrap_or_else(|err| error!("error while running tauri application: {}", err));
}

fn clear_webview_cache(app: &AppHandle) {
    if let Ok(cache_dir) = app.path().app_cache_dir() {
        // 1. Clear Chromium / WebKit Network Caches
        let target_cache = cache_dir.join("Cache");
        let target_code_cache = cache_dir.join("Code Cache");

        if target_cache.exists() {
            let _ = fs::remove_dir_all(target_cache);
        }
        if target_code_cache.exists() {
            let _ = fs::remove_dir_all(target_code_cache);
        }

        #[cfg(target_os = "windows")]
        {
            let wvv2_cache = cache_dir.join("EBWebView").join("Default").join("Cache");
            if wvv2_cache.exists() {
                let _ = fs::remove_dir_all(wvv2_cache);
                info!("Removed Cache");
            }
            let wvv2_cache_code = cache_dir.join("EBWebView").join("Default").join("Code Cache");
            if wvv2_cache_code.exists() {
                let _ = fs::remove_dir_all(wvv2_cache_code);
                info!("Removed Code Cache");
            }
        }
    }
}
