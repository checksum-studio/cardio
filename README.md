# Cardio

A free, open source heart rate monitor bridge for streamers and developers.

[![CI](https://img.shields.io/github/actions/workflow/status/checksum-studio/cardio/ci.yml?branch=main&label=CI&style=flat-square)](https://github.com/checksum-studio/cardio/actions/workflows/ci.yml)
[![Latest Release](https://img.shields.io/github/v/release/checksum-studio/cardio?style=flat-square)](https://github.com/checksum-studio/cardio/releases/latest)
[![License](https://img.shields.io/github/license/checksum-studio/cardio?style=flat-square)](https://github.com/checksum-studio/cardio/blob/main/LICENSE)
[![Downloads](https://img.shields.io/github/downloads/checksum-studio/cardio/total?style=flat-square)](https://github.com/checksum-studio/cardio/releases/latest)

---

Cardio connects to Bluetooth Low Energy heart rate monitors and exposes live heart rate data through a local REST API and WebSocket server. It ships with built-in OBS overlays so you can show your heart rate on stream without relying on a paid service or cloud dependency. Everything runs locally on your machine.

The app scans for nearby BLE devices, connects to the one you pick, and starts broadcasting heart rate, RR intervals, and battery level over HTTP and WebSocket. It sits in your system tray, reconnects to your last device automatically on launch, and updates itself when new versions are available.

Three OBS overlays are included out of the box: a beating heart, a rolling line chart, and a circular ring gauge. All of them have transparent backgrounds, respond to HR zones with color changes, and accept a `?color=FF0000` query parameter if you want to match your stream's theme. Just add a Browser Source in OBS pointed at `http://localhost:2328/overlay` (or `/overlay/chart`, `/overlay/ring`).

Cardio works with any heart rate monitor that advertises the standard Bluetooth Heart Rate Service. Polar, Garmin, Wahoo, Suunto, CooSpo, Magene, WHOOP, Movesense, Scosche, Myzone, Viiiiva, and Moofit devices are recognized by name, but anything that speaks the standard protocol will work and show up as a generic device.

## Download

**[Download the latest release](https://github.com/checksum-studio/cardio/releases/latest)**

Available for Windows (x64), macOS (Apple Silicon and Intel), Linux (x64), and Android (arm64). Cardio will notify you when updates are available and can install them automatically.

## Usage

Open Cardio and click Scan to discover nearby heart rate monitors. Select your device from the list to connect. Your heart rate data is now available at `http://localhost:2328`. The settings panel lets you configure the server host and port, toggle auto-connect to your last device, and on desktop, enable auto-launch at login and start minimized to the system tray.

Full API documentation is available at [`localhost:2328/api/docs`](http://localhost:2328/api/docs) once the app is running.

## Building from Source

You will need [Rust](https://rustup.rs/) (stable), [Node.js](https://nodejs.org/) (v18+), [pnpm](https://pnpm.io/), and the platform-specific dependencies listed in the [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/). On Linux, you also need the `bluez` package for Bluetooth support.

```sh
pnpm install
pnpm tauri dev      # development
pnpm tauri build    # production build
```

## License

[MIT](LICENSE)

The logos, icons, and brand assets in this repository are the property of checksum.studio and are not licensed under the MIT License. You may not use these assets in any way that suggests endorsement or affiliation without prior written permission.
