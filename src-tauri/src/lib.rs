mod ble;
mod config;
mod foreground_service;
mod server;
mod state;

use state::{AppState, ConnectionStatus};
use tauri::Manager;
use tracing::{debug, error, info, instrument};

#[cfg(desktop)]
use tauri::{RunEvent, WindowEvent};

#[cfg(desktop)]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(desktop)]
static EXIT_REQUESTED: AtomicBool = AtomicBool::new(false);

#[instrument(skip_all)]
#[tauri::command]
async fn get_status(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let connection = state.connection_status.read().await.clone();
    let current_data = state.current_data.read().await.clone();

    Ok(serde_json::json!({
        "connection": connection,
        "current_data": current_data,
    }))
}

#[instrument(skip_all)]
#[tauri::command]
async fn scan_devices(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    *state.connection_status.write().await = ConnectionStatus::Scanning;

    match ble::scan_for_devices(5).await {
        Ok(results) => {
            let is_scanning = matches!(
                *state.connection_status.read().await,
                ConnectionStatus::Scanning
            );
            if is_scanning {
                *state.connection_status.write().await = ConnectionStatus::Disconnected;
            }
            serde_json::to_value(&results).map_err(|e| e.to_string())
        }
        Err(e) => {
            *state.connection_status.write().await = ConnectionStatus::Disconnected;
            Err(e.to_string())
        }
    }
}

#[instrument(skip(state))]
#[tauri::command]
async fn connect_device(
    device_id: String,
    device_name: String,
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    *state.connection_status.write().await = ConnectionStatus::Connecting;

    let sender = state.hr_broadcast.clone();
    let current_data = state.current_data.clone();
    let connection_status = state.connection_status.clone();
    let config = state.config.clone();

    match ble::connect_device(&device_id, &device_name, sender.clone()).await {
        Ok(info) => {
            let battery = ble::read_battery().await;

            *connection_status.write().await = ConnectionStatus::Connected {
                device_name: info.name.clone(),
                device_id: info.id.clone(),
                battery_level: battery,
            };

            {
                let mut cfg = config.write().await;
                cfg.last_device_id = Some(info.id.clone());
                cfg.last_device_name = Some(info.name.clone());
                if let Err(e) = cfg.save() {
                    error!(error = %e, "failed to save config");
                }
            }

            let mut rx = sender.subscribe();
            tokio::spawn(async move {
                while let Ok(event) = rx.recv().await {
                    *current_data.write().await = Some(event);
                }
            });

            Ok(serde_json::json!({
                "status": "connected",
                "device": {
                    "id": info.id,
                    "name": info.name,
                    "device_type": info.device_type,
                },
                "battery": battery,
            }))
        }
        Err(e) => {
            *connection_status.write().await = ConnectionStatus::Disconnected;
            Err(e.to_string())
        }
    }
}

#[instrument(skip_all)]
#[tauri::command]
async fn disconnect_device(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    match ble::disconnect_device().await {
        Ok(()) => {
            *state.connection_status.write().await = ConnectionStatus::Disconnected;
            *state.current_data.write().await = None;
            Ok(serde_json::json!({ "status": "disconnected" }))
        }
        Err(e) => Err(e.to_string()),
    }
}

#[instrument(skip_all)]
#[tauri::command]
async fn get_config(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let config = state.config.read().await;
    serde_json::to_value(config.clone()).map_err(|e| e.to_string())
}

#[instrument(skip_all)]
#[tauri::command]
async fn update_config(
    config: crate::config::AppConfig,
    state: tauri::State<'_, AppState>,
    #[allow(unused)] app: tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    let auto_launch = config.auto_launch;
    let mut current = state.config.write().await;
    current.host = config.host;
    current.port = config.port;
    current.auto_connect = config.auto_connect;
    current.auto_launch = config.auto_launch;
    current.start_minimized = config.start_minimized;

    current.save().map_err(|e| e.to_string())?;

    #[cfg(desktop)]
    {
        use tauri_plugin_autostart::ManagerExt;
        let autolaunch = app.autolaunch();
        if auto_launch {
            let _ = autolaunch.enable();
        } else {
            let _ = autolaunch.disable();
        }
    }

    #[cfg(not(desktop))]
    let _ = auto_launch;

    serde_json::to_value(current.clone()).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_platform() -> String {
    #[cfg(target_os = "android")]
    return "android".to_string();
    #[cfg(target_os = "ios")]
    return "ios".to_string();
    #[cfg(target_os = "windows")]
    return "windows".to_string();
    #[cfg(target_os = "macos")]
    return "macos".to_string();
    #[cfg(target_os = "linux")]
    return "linux".to_string();
    #[cfg(not(any(
        target_os = "android",
        target_os = "ios",
        target_os = "windows",
        target_os = "macos",
        target_os = "linux"
    )))]
    return "unknown".to_string();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("starting cardio");

    let app_config = config::AppConfig::load();
    let server_host = app_config.host.clone();
    let port = app_config.port;
    let auto_connect = app_config.auto_connect;
    let app_state = AppState::new(app_config);

    let server_state = app_state.clone();
    let auto_connect_state = app_state.clone();

    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_blec::init())
        .plugin(tauri_plugin_opener::init());

    #[cfg(target_os = "android")]
    {
        builder = builder.plugin(foreground_service::init());
    }

    #[cfg(desktop)]
    {
        use tauri_plugin_autostart::MacosLauncher;

        static TRAY_NOTIFICATION_SHOWN: AtomicBool = AtomicBool::new(false);

        builder = builder
            .plugin(tauri_plugin_autostart::init(
                MacosLauncher::LaunchAgent,
                Some(vec!["--minimized"]),
            ))
            .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                }
            }))
            .plugin(tauri_plugin_updater::Builder::new().build())
            .plugin(tauri_plugin_process::init())
            .plugin(tauri_plugin_notification::init());

        builder = builder.on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();

                if !TRAY_NOTIFICATION_SHOWN.swap(true, Ordering::Relaxed) {
                    use tauri_plugin_notification::NotificationExt;
                    let _ = window
                        .app_handle()
                        .notification()
                        .builder()
                        .title("Cardio is still running")
                        .body("The app has been minimized to the system tray.")
                        .show();
                }
            }
        });
    }

    let app = builder
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_status,
            scan_devices,
            connect_device,
            disconnect_device,
            get_config,
            update_config,
            get_platform,
        ])
        .setup(move |app| {
            if let Ok(config_dir) = app.path().app_config_dir() {
                config::init_config_dir(config_dir);
            }

            tauri::async_runtime::spawn(async move {
                server::start_server(server_state, &server_host, port).await;
            });

            if auto_connect {
                tauri::async_runtime::spawn(async move {
                    let config = auto_connect_state.config.read().await.clone();
                    if config.last_device_id.is_some() {
                        info!("attempting auto-connect");
                        *auto_connect_state.connection_status.write().await =
                            ConnectionStatus::Connecting;

                        let sender = auto_connect_state.hr_broadcast.clone();
                        let current_data = auto_connect_state.current_data.clone();
                        let connection_status = auto_connect_state.connection_status.clone();

                        match ble::auto_connect(&config, sender.clone()).await {
                            Ok(Some(info)) => {
                                let battery = ble::read_battery().await;
                                *connection_status.write().await = ConnectionStatus::Connected {
                                    device_name: info.name.clone(),
                                    device_id: info.id.clone(),
                                    battery_level: battery,
                                };

                                let mut rx = sender.subscribe();
                                tokio::spawn(async move {
                                    while let Ok(event) = rx.recv().await {
                                        *current_data.write().await = Some(event);
                                    }
                                });

                                debug!(device_name = %info.name, "auto-connected");
                            }
                            Ok(None) => {
                                *connection_status.write().await = ConnectionStatus::Disconnected;
                                debug!("no device found for auto-connect");
                            }
                            Err(e) => {
                                *connection_status.write().await = ConnectionStatus::Disconnected;
                                error!(error = %e, "auto-connect failed");
                            }
                        }
                    }
                });
            }

            #[cfg(desktop)]
            {
                use tauri::menu::{MenuBuilder, MenuItemBuilder};
                use tauri::tray::TrayIconBuilder;

                let start_minimized = std::env::args().any(|a| a == "--minimized");
                if start_minimized {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.hide();
                    }
                }

                let show_item = MenuItemBuilder::with_id("show", "Show").build(app)?;
                let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

                let menu = MenuBuilder::new(app)
                    .item(&show_item)
                    .separator()
                    .item(&quit_item)
                    .build()?;

                let _tray = TrayIconBuilder::new()
                    .icon(app.default_window_icon().unwrap().clone())
                    .tooltip("Cardio")
                    .menu(&menu)
                    .show_menu_on_left_click(false)
                    .on_menu_event(|app, event| match event.id().as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            EXIT_REQUESTED.store(true, Ordering::Relaxed);
                            app.exit(0);
                        }
                        _ => {}
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let tauri::tray::TrayIconEvent::Click {
                            button: tauri::tray::MouseButton::Left,
                            ..
                        } = event
                        {
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    })
                    .build(app)?;
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    #[cfg(desktop)]
    app.run(|_app, event| {
        if let RunEvent::ExitRequested { api, .. } = event {
            if !EXIT_REQUESTED.load(Ordering::Relaxed) {
                api.prevent_exit();
            }
        }
    });

    #[cfg(not(desktop))]
    app.run(|_app, _event| {});
}
