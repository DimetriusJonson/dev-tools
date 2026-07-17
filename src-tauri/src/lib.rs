use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use log::{error, info};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, WindowEvent};
use tauri::{Url, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::{CommandChild, CommandEvent};

#[tauri::command]
fn get_resource_dir(app_handle: &AppHandle) -> PathBuf {
    app_handle.path().resource_dir().unwrap()
}

fn start_backend_server(
    app_handle: &AppHandle,
    port: u16,
    resource_dir: PathBuf,
    remote_server_url: String,
) -> Result<(tokio::sync::mpsc::Receiver<CommandEvent>, CommandChild), String> {
    let addr = format!("0.0.0.0:{}", port);

    info!("Backend server starting up on {}...", addr);

    let mut site_dir = resource_dir;
    site_dir.push("_up_");
    site_dir.push("site");

    let app_handle = app_handle.clone();
    let shell = app_handle.shell();

    match shell.sidecar("webdev_useful_tools_server") {
        Ok(sidecar) => match sidecar
            .env("LEPTOS_OUTPUT_NAME", "dev_tools")
            .env("LEPTOS_SITE_ADDR", addr.to_owned())
            .env("LEPTOS_SITE_ROOT", site_dir)
            .env("DEVTOOLS_REMOTE_SERVER_URL", remote_server_url)
            .arg(format!("--addr={}", addr))
            .spawn()
        {
            Ok(rx) => Ok(rx),
            Err(err) => {
                error!("Error: {}", err);
                Err(err.to_string())
            }
        },
        Err(err) => {
            error!("Error: {}", err);
            Err(err.to_string())
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let server_cmd_child = Arc::new(Mutex::new(None));

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::Builder::new().args(["--autostart"]).build())
        .plugin(tauri_plugin_shell::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: Some("webdev_useful_tools.log".to_owned()) }),
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
        .setup(move |app| {
            let args: Vec<String> = std::env::args().collect();

            let port = match get_arg_value(&args, "port") {
                Some(port) => port.parse::<u16>().unwrap(),
                None => 3005,
            };

            let arg_remote_server_url = get_arg_value(&args, "remote-server-url")
                .unwrap_or("https://dev-tools-rust.vercel.app".to_owned());

            let resource_dir = get_resource_dir(app.app_handle());

            let server_descr = start_backend_server(app.app_handle(), port, resource_dir, arg_remote_server_url)?;
            *server_cmd_child.lock().unwrap() = Some(server_descr.1);

            tauri::async_runtime::spawn(async move {
                let mut rx = server_descr.0;
                while let Some(received) = rx.recv().await {
                    match received {
                        tauri_plugin_shell::process::CommandEvent::Stderr(items) => {
                            error!("server: {}", String::from_utf8_lossy(&items))
                        }
                        tauri_plugin_shell::process::CommandEvent::Stdout(items) => {
                            info!("server: {}", String::from_utf8_lossy(&items))
                        }
                        tauri_plugin_shell::process::CommandEvent::Error(err) => {
                            error!("Error: {}", err)
                        }
                        tauri_plugin_shell::process::CommandEvent::Terminated(_) => break,
                        _ => break,
                    }
                }
            });            

            if args.contains(&"--autostart".to_string()) {
                if let Some(window) = app.get_webview_window("main") {
                    window.hide().unwrap();
                }
            }

            let server_url = format!("http://127.0.0.1:{}", port)/*"https://dev-tools-rust.vercel.app"*/;

            let target_url = Url::parse(&server_url).expect("Failed to parse server URL");

            let _window = WebviewWindowBuilder::new(app, "main", WebviewUrl::External(target_url))
                .title("Developer Tools")
                .inner_size(1500.0, 1000.0)
                .enable_clipboard_access()
                .disable_drag_drop_handler()
                .build()
                .expect("Failed to build dynamic window");

            let quit_i = MenuItem::with_id(app, "quit", "Выход", true, None::<&str>)?;
            let open_i = MenuItem::with_id(app, "open", "Открыть", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "quit" => {
                        let mut managed_child = server_cmd_child.lock().unwrap();
                        if let Some(cmd_child) = managed_child.take() {
                            let _ = cmd_child.kill(); // Best effort termination
                        }
                        app.exit(0);
                    }
                    "open" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {
                        //                        println!("menu item {:?} not handled", event.id);
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                //let _ = window.minimize();
                let _ = window.hide();
            }
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
