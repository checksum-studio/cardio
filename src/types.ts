export type ConnectionStatus =
  | { status: "disconnected" }
  | { status: "scanning" }
  | { status: "connecting" }
  | {
      status: "connected";
      device_name: string;
      device_id: string;
      battery_level: number | null;
    };

export interface HrEvent {
  heart_rate: number;
  rr_intervals: number[];
  battery: number | null;
  signal_quality: string | null;
  device_name: string;
  device_id: string;
  timestamp: string;
}

export interface ScanResult {
  device_info: DeviceInfo;
  rssi: number | null;
}

export interface DeviceInfo {
  id: string;
  name: string;
  device_type: string;
}

export interface AppConfig {
  host: string;
  port: number;
  auto_connect: boolean;
  auto_launch: boolean;
  start_minimized: boolean;
  last_device_id: string | null;
  last_device_name: string | null;
}

export type Platform = "windows" | "macos" | "linux" | "android" | "ios" | "unknown";

export interface AppStatus {
  connection: ConnectionStatus;
  current_data: HrEvent | null;
}
