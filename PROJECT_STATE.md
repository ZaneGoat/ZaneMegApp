# ZaneMegApp Rewrite (V2.5) - Project State (Cyber-Deck Rust Edition)

## Context
This project is a high-performance, modular rewrite of the old monolithic `~/ZaneMegaCenter.py` in **Rust**, using `eframe` (egui) and `egui_plot`. The old Python app and its `.desktop` launcher remain completely untouched.

## Architecture & Upgrades (V2.5 Cyber-Deck)
- **Language / Toolchain**: Rust 2024 (`cargo`), `eframe 0.36.2`, `egui_plot 0.37.0`, `libc 0.2`.
- **Theme & HUD Widgets (`src/theme.rs`)**: Cyber-Deck V2.5 palette with reusable HUD widgets (`cyber_card`, `metric_tile`, `resource_gauge_card`, `status_pill`, `cyber_console`).
- **Single-Instance Protection (`src/main.rs`)**: Linux kernel `flock` advisory lock (`/run/user/<uid>/zanemegapp.lock`) with duplicate process detection and desktop notification.
- **Top Cockpit Telemetry HUD (`src/app.rs`)**: Real-time status pills in header (CPU temp, RAM %, NVMe %, Battery %, Network Shield state).
- **Core Modules (`src/core/`)**:
  - `cleaner.rs`: Native `SystemResources` reader (`/proc/meminfo` & `df`), before/after delta reclaimed space calculation, single-prompt `pkexec` batching for all privileged modules, and 12 cleanup modules.
  - `hardware.rs`: Extended sysfs telemetry (coretemp, NVMe SSD composite, CPU frequency GHz, scaling governor, battery capacity & status), non-blocking background threads for `pkexec` commands, and power profiles.
  - `controller.rs`: Starship Flight Deck launcher (`start.sh`), process lifecycle manager (start/stop/restart), LED matrix launcher (`led.py`), and real-time streaming console.
  - `headset.rs`: SoundPEATS Q3 Pro RFCOMM manager (`~/soundpeats_controller.py`). Explicitly labeled `UNVERIFIED` with Android HCI snoop extraction instructions.
  - `security.rs`: UFW firewall network isolation toggle via `pkexec` with desktop notifications and penetration testing operational impact advisories.
  - `idevice.rs`: iOS utilities (`ideviceinfo`, `idevicesyslog` daemon stream, `iproxy` USB port forwarding).
- **UI Tabs (`src/ui/`)**:
  - `tab_cleaner.rs`: 3-Column live resource telemetry deck (Physical RAM, Swap, NVMe root disk), 1-click Quick RAM Boost, categorized module sidebar with root tags, and matrix console.
  - `tab_hardware.rs`: 4 Telemetry tiles, 3-sensor waveform plot (CPU, GPU/PCH, NVMe), highlighted active power profiles with async execution, EC fan curve, and battery health cap (60/80/100%).
  - `tab_controller.rs`: Starship flight deck card, execution controls, links, and terminal log.
  - `tab_headset.rs`: Earbuds mode dispatch with UNVERIFIED warning banner.
  - `tab_security.rs`: Network isolation status badge, toggle button, and pentest advisory.
  - `tab_idevice.rs`: iOS device tools and console log.
- **Desktop Launcher**: `~/Desktop/ZaneMegApp.desktop` with custom vector cyber SVG icon (`assets/icon.svg`).
