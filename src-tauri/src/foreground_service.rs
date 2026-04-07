use tauri::{
    plugin::{Builder, TauriPlugin},
    Wry,
};

#[cfg(target_os = "android")]
use std::sync::OnceLock;

#[cfg(target_os = "android")]
use tracing::info;

#[cfg(target_os = "android")]
static PLUGIN_HANDLE: OnceLock<tauri::plugin::PluginHandle<Wry>> = OnceLock::new();

#[cfg(target_os = "android")]
pub fn start_service() {
    if let Some(handle) = PLUGIN_HANDLE.get() {
        info!("starting foreground service");
        let _: Result<(), _> = handle.run_mobile_plugin("startService", serde_json::json!({}));
    }
}

#[cfg(target_os = "android")]
pub fn stop_service() {
    if let Some(handle) = PLUGIN_HANDLE.get() {
        info!("stopping foreground service");
        let _: Result<(), _> = handle.run_mobile_plugin("stopService", serde_json::json!({}));
    }
}

#[allow(dead_code)]
pub fn init() -> TauriPlugin<Wry> {
    Builder::new("foreground-service")
        .setup(|_app, _api| {
            #[cfg(target_os = "android")]
            {
                let handle = _api
                    .register_android_plugin("space.checksum.cardio", "ForegroundServicePlugin")?;
                let _ = PLUGIN_HANDLE.set(handle);
                start_service();
            }
            Ok(())
        })
        .build()
}
