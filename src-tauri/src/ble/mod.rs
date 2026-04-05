pub mod device;
pub mod devices;
pub mod gatt;

use chrono::Utc;
use thiserror::Error;
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, info, warn};

use tauri_plugin_blec::{get_handler, models::ScanFilter, OnDisconnectHandler};

use crate::config::AppConfig;
use crate::state::HrEvent;
use device::{DeviceInfo, ScanResult};
use devices::identify_device;
use gatt::{
    parse_hr_measurement, BATTERY_LEVEL_UUID, BATTERY_SERVICE_UUID, HR_MEASUREMENT_UUID,
    HR_SERVICE_UUID,
};

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum BleError {
    #[error("BLE error: {0}")]
    PluginError(#[from] tauri_plugin_blec::Error),
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    #[error("Not connected")]
    NotConnected,
}

impl serde::Serialize for BleError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub async fn scan_for_devices(duration_secs: u64) -> Result<Vec<ScanResult>, BleError> {
    info!(duration_secs, "starting ble scan");

    let handler = get_handler()?;
    let (tx, mut rx) = mpsc::channel::<Vec<tauri_plugin_blec::models::BleDevice>>(16);

    let timeout_ms = duration_secs * 1000;
    handler
        .discover(
            Some(tx),
            timeout_ms,
            ScanFilter::Service(HR_SERVICE_UUID),
            false,
        )
        .await?;

    let mut results = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // discover() spawns a background task that sends batches over the channel
    // for (timeout / 200ms) iterations. We receive until the scan completes.
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_millis(timeout_ms + 500);
    while let Ok(Some(batch)) = tokio::time::timeout_at(deadline, rx.recv()).await {
        for device in batch {
            if device.name.is_empty() || seen.contains(&device.address) {
                continue;
            }
            seen.insert(device.address.clone());
            let device_type = identify_device(&device.name);
            debug!(
                name = %device.name,
                address = %device.address,
                rssi = ?device.rssi,
                "found hr device"
            );
            results.push(ScanResult {
                device_info: DeviceInfo {
                    id: device.address.clone(),
                    name: device.name.clone(),
                    device_type,
                },
                rssi: device.rssi,
                is_connectable: true,
            });
        }
    }

    info!(count = results.len(), "scan complete");
    Ok(results)
}

pub async fn connect_device(
    address: &str,
    device_name: &str,
    sender: broadcast::Sender<HrEvent>,
) -> Result<DeviceInfo, BleError> {
    info!(address, device_name, "connecting to device");

    let handler = get_handler()?;

    let disconnect_addr = address.to_string();
    let on_disconnect = OnDisconnectHandler::Sync(Box::new(move || {
        warn!(address = %disconnect_addr, "device disconnected");
    }));

    handler.connect(address, on_disconnect, false).await?;

    let sender_clone = sender.clone();
    let name = device_name.to_string();
    let addr = address.to_string();
    handler
        .subscribe(
            HR_MEASUREMENT_UUID,
            Some(HR_SERVICE_UUID),
            move |data: Vec<u8>| {
                if let Some((heart_rate, rr_intervals)) = parse_hr_measurement(&data) {
                    let event = HrEvent {
                        heart_rate,
                        rr_intervals,
                        battery: None,
                        signal_quality: None,
                        device_name: name.clone(),
                        device_id: addr.clone(),
                        timestamp: Utc::now(),
                    };
                    let _ = sender_clone.send(event);
                }
            },
        )
        .await?;

    let battery = read_battery().await;
    if let Some(level) = battery {
        debug!(battery_level = level, "read battery level");
    }

    let device_type = identify_device(device_name);
    let info = DeviceInfo {
        id: address.to_string(),
        name: device_name.to_string(),
        device_type,
    };

    info!(
        device_name = %info.name,
        device_id = %info.id,
        "device connected and streaming"
    );
    Ok(info)
}

pub async fn disconnect_device() -> Result<(), BleError> {
    info!("disconnecting device");
    let handler = get_handler()?;
    handler.disconnect().await?;
    Ok(())
}

pub async fn read_battery() -> Option<u8> {
    let handler = get_handler().ok()?;
    match handler
        .recv_data(BATTERY_LEVEL_UUID, Some(BATTERY_SERVICE_UUID))
        .await
    {
        Ok(data) => data.first().copied(),
        Err(e) => {
            debug!(error = %e, "failed to read battery level");
            None
        }
    }
}

pub async fn auto_connect(
    config: &AppConfig,
    sender: broadcast::Sender<HrEvent>,
) -> Result<Option<DeviceInfo>, BleError> {
    let device_id = match &config.last_device_id {
        Some(id) => id.clone(),
        None => return Ok(None),
    };

    let device_name = config
        .last_device_name
        .clone()
        .unwrap_or_else(|| "Unknown".to_string());

    info!(device_id = %device_id, "auto-connecting to last device");

    let results = scan_for_devices(5).await?;
    let found = results
        .iter()
        .any(|r| r.device_info.id == device_id);

    if !found {
        warn!(device_id = %device_id, "device not found during auto-connect scan");
        return Ok(None);
    }

    match connect_device(&device_id, &device_name, sender).await {
        Ok(info) => {
            info!(device_name = %info.name, "auto-connected");
            Ok(Some(info))
        }
        Err(e) => {
            warn!(error = %e, "auto-connect failed");
            Ok(None)
        }
    }
}
