use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use utoipa::ToSchema;

use crate::config::AppConfig;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HrEvent {
    /// BPM
    pub heart_rate: u16,
    /// Milliseconds
    pub rr_intervals: Vec<f64>,
    /// 0-100
    pub battery: Option<u8>,
    pub signal_quality: Option<String>,
    pub device_name: String,
    pub device_id: String,
    /// ISO 8601
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ConnectionStatus {
    #[default]
    Disconnected,
    Scanning,
    Connecting,
    Connected {
        device_name: String,
        device_id: String,
        battery_level: Option<u8>,
    },
}

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<RwLock<AppConfig>>,
    pub hr_broadcast: broadcast::Sender<HrEvent>,
    pub connection_status: Arc<RwLock<ConnectionStatus>>,
    pub current_data: Arc<RwLock<Option<HrEvent>>>,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let (hr_broadcast, _) = broadcast::channel(100);
        Self {
            config: Arc::new(RwLock::new(config)),
            hr_broadcast,
            connection_status: Arc::new(RwLock::new(ConnectionStatus::default())),
            current_data: Arc::new(RwLock::new(None)),
        }
    }
}
