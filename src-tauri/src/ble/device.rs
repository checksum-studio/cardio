use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    PolarH10,
    PolarH9,
    PolarOh1,
    PolarVeritySense,
    PolarGeneric,
    GarminHrmPro,
    GarminHrmDual,
    GarminHrmFit,
    GarminGeneric,
    WahooTickr,
    WahooTickrX,
    WahooTickrFit,
    WahooGeneric,
    SuuntoSmartSensor,
    SuuntoGeneric,
    CoospoH808s,
    CoospoH6,
    CoospoHw807,
    CoospoGeneric,
    MageneH303,
    MageneH64,
    MageneGeneric,
    Whoop,
    Movesense,
    ScoscheRhythm,
    Myzone,
    Viiiiva,
    Moofit,
    Generic,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub device_type: DeviceType,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ScanResult {
    pub device_info: DeviceInfo,
    pub rssi: Option<i16>,
    pub is_connectable: bool,
}
