# ADBSight (`adbsight`) v1.0.0

A fast, cross-platform **Rust + Ratatui TUI Android Deep Inspector** with first-class **Wi-Fi Wireless Debugging** (Android 11+ QR code pairing, mDNS auto-discovery, pair code, and USB-to-Wi-Fi handoff), live telemetry, embedded SQLite snapshot history, and JSON/CSV/Markdown exports.

> Evolved from the original `.NET` WinForms ADB Data Fetcher (`git log` preserves the C# history at commit `2ce03b2`).

---

## Features

- **First-Class Wireless Debugging (`w`)**
  - **Terminal QR Code Pairing (Android 11+)**: Generates `WIFI:T:ADB;S:...;P:...;;`, renders a compact scannable Unicode half-block QR code inside the TUI, watches `adb mdns services`, and automatically runs `adb pair` + `adb connect`.
  - **mDNS Auto-Discovery & Bookmarks**: Lists `_adb-tls-connect._tcp` and `_adb-tls-pairing._tcp` devices on your LAN and remembers Wi-Fi endpoints.
  - **One-Key USB → Wi-Fi (`t`)**: Reads `wlan0` IPv4, switches `adbd` to `tcpip 5555`, and connects over Wi-Fi.
  - **Manual Pair / Direct Connect**: Pair with 6-digit code (`adb pair`) or connect directly (`adb connect <IP:PORT>`).
- **8 Deep Inspector Modules**
  1. **`1 Dashboard`**: Device identity, SELinux/verified-boot posture, live CPU total & per-core sparkline, RAM/Swap gauges, battery vitals & thermal zones, storage mounts, and Wi-Fi RX/TX throughput sparkline (fetched in a single batched ADB probe per tick).
  2. **`2 System`**: Searchable `getprop`, `settings` (`system`/`secure`/`global`), hardware sensors (`dumpsys sensorservice`), and an interactive `dumpsys` service browser.
  3. **`3 Processes`**: Live sortable `ps` table (CPU%, RAM, PID, Name) and active services, with confirmed `SIGKILL` (`K`).
  4. **`4 Packages`**: All/User/System package browser + deep `dumpsys package` inspector (SDKs, UID, installer, granted & denied permissions), plus launch (`l`), pull APK (`p`), confirmed force-stop (`x`), and clear data (`c`).
  5. **`5 Logcat`**: Live streaming `logcat -v threadtime` with level toggles (`v`/`d`/`i`/`w`/`e`/`f`), text filter (`/`), pause (`Space`), clear (`C`), and CSV/JSON export.
  6. **`6 Files`**: Dual-pane filesystem explorer (`/sdcard`, `/data/local/tmp`, `/`) with permissions, symlinks, 32 KiB file preview, `pull` (`p`), and confirmed `push` (`u`).
  7. **`7 Personal`**: Contacts, SMS (comma/newline-safe parser), Call Logs, and Accounts — with automatic Android 10–15+ permission diagnostics and Root Mode unlock.
  8. **`8 Shell`**: Interactive ADB shell console with command history + quick actions (`F2` PNG screenshot, `F3` reboot menu, `F4` full SQLite snapshot, `Ctrl+I` remote text input).
- **Root-Aware Unlock (`R`)**: Detects `adb root` or `su` automatically and routes protected queries/directories through root only when toggled on.
- **Zero-Setup SQLite Snapshots (`S` / `H`) & Reports (`e`)**: Embedded WAL-mode SQLite (`rusqlite` bundled) stores full browsable device snapshots for offline review, plus JSON, CSV, and Markdown dossier exports.
- **Built-in Simulator (`--demo`)**: Explore every screen, chart, logcat stream, and export on a simulated Pixel 9 Pro (Android 15) without a physical phone attached.

---

## Prerequisites & ADB Auto-Discovery

- **Rust toolchain** (`cargo` / `rustc` 1.88+ for building from source)
- **Android SDK Platform-Tools (`adb`)**

`adbsight` automatically searches for `adb` in:
1. `--adb-path <PATH>` or `config.toml`
2. `PATH`
3. `$ANDROID_HOME/platform-tools/adb` and `$ANDROID_SDK_ROOT/platform-tools/adb`
4. Standard Linux (`~/Android/Sdk/platform-tools/adb`), macOS (`~/Library/Android/sdk/platform-tools/adb`), and Windows (`%LOCALAPPDATA%\Android\Sdk\platform-tools\adb.exe`) SDK locations

---

## Installation

### Install via Cargo (Recommended)

```bash
# Install the latest release from crates.io
cargo install adbsight

# Or install directly from GitHub
cargo install --git https://github.com/ADPer0705/adbsight
```

### Prebuilt Linux x86_64 Binary

Download `adbsight-v1.0.0-x86_64-unknown-linux-gnu.tar.gz` from [GitHub Releases](https://github.com/ADPer0705/adbsight/releases/tag/v1.0.0).

---

## Quick Start

```bash
# Launch with the built-in simulated Android 15 device (no phone needed)
adbsight --demo

# Launch against real USB / Wi-Fi devices
adbsight

# Connect directly to a wireless ADB endpoint on startup
adbsight --connect 192.168.1.42:5555

# List USB, Wi-Fi, and mDNS devices headlessly
adbsight devices

# Capture a full Markdown or JSON inspection dossier headlessly
adbsight --demo dump --format md --out report.md --save-db
```

> Full wiki & documentation: [adbsight.dearanayji.rocks](https://adbsight.dearanayji.rocks) • Licensed under [MIT](LICENSE).

---

## Keybindings

| Key | Action |
|---|---|
| `1` – `8` | Switch between the 8 inspector tabs |
| `Tab` / `Shift+Tab` | Cycle sub-tabs (System, Processes, Personal, Wireless modal) |
| `w` | Open Wireless Debugging & Device Manager (QR Pair / mDNS / IP:Port / USB→Wi-Fi) |
| `R` | Toggle Root Mode (`adb root` / `su`) |
| `r` | Refresh active tab (or return to live device when viewing an offline snapshot) |
| `j` / `k` or `↑` / `↓` | Move selection (`PgUp`/`PgDn` scroll detail panes) |
| `/` and `Esc` | Search/filter active tab; `Esc` clears filter or closes modal |
| `S` / `H` | Save full SQLite snapshot / Browse & load snapshot history |
| `e` | Export active tab or full device snapshot (`JSON` / `CSV` / `Markdown`) |
| `F2` / `F3` / `F4` | Capture PNG screenshot / Reboot (confirmed) / Quick snapshot |
| `?` | Show keybindings help overlay |
| `q` / `Ctrl+C` | Quit |
