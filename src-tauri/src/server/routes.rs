use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use tracing::{error, instrument};
use utoipa::ToSchema;

use crate::ble::device::ScanResult;
use crate::config::AppConfig;
use crate::state::{AppState, ConnectionStatus};

#[derive(Serialize, ToSchema)]
pub struct StatusResponse {
    pub connection: ConnectionStatus,
    /// BPM
    pub current_hr: Option<u16>,
    /// Milliseconds
    pub current_rr: Option<Vec<f64>>,
    pub device_name: Option<String>,
    pub device_id: Option<String>,
    /// 0-100
    pub battery: Option<u8>,
}

#[derive(Serialize, ToSchema)]
pub struct ApiError {
    pub error: String,
}

fn api_error(status: StatusCode, msg: impl Into<String>) -> (StatusCode, Json<ApiError>) {
    (status, Json(ApiError { error: msg.into() }))
}

#[utoipa::path(
    get,
    path = "/api/status",
    tag = "Status",
    summary = "Current connection and heart rate data",
    responses(
        (status = 200, description = "Current status", body = StatusResponse)
    )
)]
#[instrument(level = "trace", skip_all)]
pub async fn get_status(State(state): State<AppState>) -> Json<StatusResponse> {
    let connection = state.connection_status.read().await.clone();
    let current_data = state.current_data.read().await.clone();

    Json(StatusResponse {
        connection,
        current_hr: current_data.as_ref().map(|d| d.heart_rate),
        current_rr: current_data.as_ref().map(|d| d.rr_intervals.clone()),
        device_name: current_data.as_ref().map(|d| d.device_name.clone()),
        device_id: current_data.as_ref().map(|d| d.device_id.clone()),
        battery: current_data.as_ref().and_then(|d| d.battery),
    })
}

#[utoipa::path(
    get,
    path = "/api/devices/scan",
    tag = "Devices",
    summary = "Scan for nearby BLE heart rate devices (5s)",
    responses(
        (status = 200, description = "List of discovered devices", body = Vec<ScanResult>),
        (status = 500, description = "Scan failed", body = ApiError)
    )
)]
#[instrument(level = "info", skip_all)]
pub async fn scan_devices(
    State(state): State<AppState>,
) -> Result<Json<Vec<ScanResult>>, (StatusCode, Json<ApiError>)> {
    *state.connection_status.write().await = ConnectionStatus::Scanning;

    match crate::ble::scan_for_devices(5).await {
        Ok(results) => {
            let current_status = {
                let status = state.connection_status.read().await;
                matches!(*status, ConnectionStatus::Scanning)
            };
            if current_status {
                *state.connection_status.write().await = ConnectionStatus::Disconnected;
            }
            Ok(Json(results))
        }
        Err(e) => {
            *state.connection_status.write().await = ConnectionStatus::Disconnected;
            Err(api_error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct ConnectRequest {
    pub device_id: String,
    pub device_name: String,
}

#[derive(Serialize, ToSchema)]
pub struct ConnectResponse {
    pub status: String,
    pub device: crate::ble::device::DeviceInfo,
    /// 0-100
    pub battery: Option<u8>,
}

#[derive(Serialize, ToSchema)]
pub struct DisconnectResponse {
    pub status: String,
}

#[utoipa::path(
    post,
    path = "/api/devices/connect",
    tag = "Devices",
    summary = "Connect to a BLE heart rate device and start streaming",
    request_body = ConnectRequest,
    responses(
        (status = 200, description = "Successfully connected", body = ConnectResponse),
        (status = 500, description = "Connection failed", body = ApiError)
    )
)]
#[instrument(level = "info", skip_all)]
pub async fn connect_device(
    State(state): State<AppState>,
    Json(body): Json<ConnectRequest>,
) -> Result<Json<ConnectResponse>, (StatusCode, Json<ApiError>)> {
    *state.connection_status.write().await = ConnectionStatus::Connecting;

    let sender = state.hr_broadcast.clone();
    let current_data = state.current_data.clone();
    let connection_status = state.connection_status.clone();
    let config = state.config.clone();

    match crate::ble::connect_device(&body.device_id, &body.device_name, sender.clone()).await {
        Ok(info) => {
            let battery = crate::ble::read_battery().await;

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

            Ok(Json(ConnectResponse {
                status: "connected".to_string(),
                device: crate::ble::device::DeviceInfo {
                    id: info.id,
                    name: info.name,
                    device_type: info.device_type,
                },
                battery,
            }))
        }
        Err(e) => {
            *connection_status.write().await = ConnectionStatus::Disconnected;
            Err(api_error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/devices/disconnect",
    tag = "Devices",
    summary = "Disconnect the current device",
    responses(
        (status = 200, description = "Successfully disconnected", body = DisconnectResponse),
        (status = 500, description = "Disconnect failed", body = ApiError)
    )
)]
#[instrument(level = "info", skip_all)]
pub async fn disconnect_device(
    State(state): State<AppState>,
) -> Result<Json<DisconnectResponse>, (StatusCode, Json<ApiError>)> {
    match crate::ble::disconnect_device().await {
        Ok(()) => {
            *state.connection_status.write().await = ConnectionStatus::Disconnected;
            *state.current_data.write().await = None;
            Ok(Json(DisconnectResponse {
                status: "disconnected".to_string(),
            }))
        }
        Err(e) => Err(api_error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

#[utoipa::path(
    get,
    path = "/api/config",
    tag = "Configuration",
    summary = "Get current configuration",
    responses(
        (status = 200, description = "Current configuration", body = AppConfig)
    )
)]
#[instrument(level = "trace", skip_all)]
pub async fn get_config(State(state): State<AppState>) -> Json<AppConfig> {
    let config = state.config.read().await;
    Json(config.clone())
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateConfigRequest {
    pub port: Option<u16>,
    pub auto_connect: Option<bool>,
    pub auto_launch: Option<bool>,
    pub start_minimized: Option<bool>,
}

#[utoipa::path(
    put,
    path = "/api/config",
    tag = "Configuration",
    summary = "Update configuration (partial)",
    request_body = UpdateConfigRequest,
    responses(
        (status = 200, description = "Updated configuration", body = AppConfig),
        (status = 500, description = "Failed to save", body = ApiError)
    )
)]
#[instrument(level = "info", skip_all)]
pub async fn update_config(
    State(state): State<AppState>,
    Json(body): Json<UpdateConfigRequest>,
) -> Result<Json<AppConfig>, (StatusCode, Json<ApiError>)> {
    let mut config = state.config.write().await;

    if let Some(port) = body.port {
        config.port = port;
    }
    if let Some(auto_connect) = body.auto_connect {
        config.auto_connect = auto_connect;
    }
    if let Some(auto_launch) = body.auto_launch {
        config.auto_launch = auto_launch;
    }
    if let Some(start_minimized) = body.start_minimized {
        config.start_minimized = start_minimized;
    }

    if let Err(e) = config.save() {
        return Err(api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to save config: {}", e),
        ));
    }

    Ok(Json(config.clone()))
}
