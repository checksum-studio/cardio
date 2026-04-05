use uuid::Uuid;

pub const HR_SERVICE_UUID: Uuid = Uuid::from_u128(0x0000180D_0000_1000_8000_00805F9B34FB);
pub const HR_MEASUREMENT_UUID: Uuid = Uuid::from_u128(0x00002A37_0000_1000_8000_00805F9B34FB);
pub const BATTERY_SERVICE_UUID: Uuid = Uuid::from_u128(0x0000180F_0000_1000_8000_00805F9B34FB);
pub const BATTERY_LEVEL_UUID: Uuid = Uuid::from_u128(0x00002A19_0000_1000_8000_00805F9B34FB);

/// Returns `(bpm, rr_intervals_ms)` per the BLE Heart Rate Measurement spec.
pub fn parse_hr_measurement(data: &[u8]) -> Option<(u16, Vec<f64>)> {
    if data.is_empty() {
        return None;
    }

    let flags = data[0];
    let hr_format_16bit = (flags & 0x01) != 0;
    let _sensor_contact_supported = (flags & 0x02) != 0;
    let _sensor_contact_detected = (flags & 0x04) != 0;
    let energy_expended_present = (flags & 0x08) != 0;
    let rr_interval_present = (flags & 0x10) != 0;

    let mut offset = 1;

    let heart_rate = if hr_format_16bit {
        if data.len() < offset + 2 {
            return None;
        }
        let hr = u16::from_le_bytes([data[offset], data[offset + 1]]);
        offset += 2;
        hr
    } else {
        if data.len() < offset + 1 {
            return None;
        }
        let hr = data[offset] as u16;
        offset += 1;
        hr
    };

    if energy_expended_present {
        offset += 2;
    }

    let mut rr_intervals = Vec::new();
    if rr_interval_present {
        while offset + 1 < data.len() {
            let rr_raw = u16::from_le_bytes([data[offset], data[offset + 1]]);
            let rr_ms = rr_raw as f64 / 1024.0 * 1000.0;
            rr_intervals.push(rr_ms);
            offset += 2;
        }
    }

    Some((heart_rate, rr_intervals))
}
