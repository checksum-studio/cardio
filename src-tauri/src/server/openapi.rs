use utoipa::OpenApi;

use crate::ble::device::{DeviceInfo, DeviceType, ScanResult};
use crate::config::AppConfig;
use crate::state::{ConnectionStatus, HrEvent};

use super::routes::{
    ApiError, ConnectRequest, ConnectResponse, DisconnectResponse, StatusResponse,
    UpdateConfigRequest,
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Cardio",
        description = "REST & WebSocket API for the Cardio BLE heart rate monitor bridge.\n\n\
            Connect to any standard BLE heart rate monitor and stream data over HTTP or WebSocket.\n\n\
            **WebSocket**: Connect to `/api/ws` for real-time heart rate streaming. Each message is a JSON `HrEvent`.\n\n\
            **OBS Overlays**: Use `/overlay` (heart + BPM) or `/overlay/chart` (real-time graph) as browser sources.",
        version = "0.1.0",
        license(name = "MIT")
    ),
    servers(
        (url = "/", description = "Current server")
    ),
    paths(
        super::routes::get_status,
        super::routes::scan_devices,
        super::routes::connect_device,
        super::routes::disconnect_device,
        super::routes::get_config,
        super::routes::update_config,
        super::overlays::standard::overlay_page,
        super::overlays::chart::overlay_chart_page,
        super::overlays::ring::overlay_ring_page,
    ),
    components(schemas(
        StatusResponse,
        ApiError,
        ConnectRequest,
        ConnectResponse,
        DisconnectResponse,
        UpdateConfigRequest,
        ConnectionStatus,
        HrEvent,
        ScanResult,
        DeviceInfo,
        DeviceType,
        AppConfig,
    )),
    tags(
        (name = "Status"),
        (name = "Devices"),
        (name = "Configuration"),
        (name = "Streaming"),
        (name = "Overlays", description = "OBS browser source overlays")
    )
)]
pub struct ApiDoc;
