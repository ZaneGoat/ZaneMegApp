# ZaneMegApp Rewrite (V2) - Project State (Rust Edition)

## Context
This project is a high-performance, modular rewrite of the old monolithic `~/ZaneMegaCenter.py` in **Rust**, using `eframe` (egui) and `egui_plot`. The old Python app and its `.desktop` launcher remain completely untouched.

## Architecture
- **Language / Toolchain**: Rust 1.99.0 (`cargo`), `eframe 0.36.2`, `egui_plot 0.37.0`.
- **Theme**: Dark cyber aesthetic (`#0a0a0a` dark background, `#cc0000` red primary accent, custom visuals).
- **Core Modules (`src/core/`)**:
  - `hardware.rs`: Wraps `isw` fan curves/Cooler Boost via `pkexec`, battery charge threshold limits, sysfs telemetry, and `tlp`/`kscreen-doctor` power profiles.
  - `cleaner.rs`: 12 system cleanup modules (Smart RAM Optimizer, Kernel page cache drop, background services, swap reset, apt, pip, journal, trash, thumbnails, flatpak, docker, /tmp). Non-blocking background worker thread with streaming channels.
  - `controller.rs`: Starship Flight Deck launcher (`start.sh`), process lifecycle manager (start/stop/restart), LED matrix launcher (`led.py`), and real-time streaming console.
  - `headset.rs`: SoundPEATS Q3 Pro RFCOMM manager (`~/soundpeats_controller.py`). Explicitly labeled `UNVERIFIED` with Android HCI snoop extraction instructions.
  - `security.rs`: UFW firewall network isolation toggle via `pkexec` with desktop notifications and penetration testing operational impact advisories.
  - `idevice.rs`: iOS utilities (`ideviceinfo`, `idevicesyslog` daemon stream, `iproxy` USB port forwarding).
- **UI Tabs (`src/ui/`)**:
  - `tab_hardware.rs`: Realtime GPU/CPU thermal curves rendered via `egui_plot`, power profiles, fan controls, and battery cap.
  - `tab_controller.rs`: Starship flight deck card, execution controls, links, and terminal log.
  - `tab_cleaner.rs`: Module selection sidebar with hover tooltips, presets, and live output console.
  - `tab_headset.rs`: Earbuds mode dispatch with UNVERIFIED warning banner.
  - `tab_security.rs`: Network isolation status badge, toggle button, and pentest advisory.
  - `tab_idevice.rs`: iOS device tools and console log.
- **Application (`src/app.rs`)**:
  - Top header navigation bar with title and active tab switcher.
  - Central scrollable panel rendering active tab.
  - Bottom status bar with real-time hardware telemetry and status indicators.

## Previous Python Scaffold
Archived inside `./python_prototype/` for reference.
