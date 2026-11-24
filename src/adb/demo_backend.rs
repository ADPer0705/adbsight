use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use anyhow::{Result, bail};
use chrono::Utc;
use tokio::sync::{Mutex, mpsc};
use tokio::task::JoinHandle;

use crate::adb::parsers::filesystem::LsParseResult;
use crate::models::{
    AccountRecord, BatteryInfo, CallLogRecord, ContactRecord, CpuSample, DeviceIdentity,
    DeviceState, DeviceSummary, FsEntry, FsKind, LogEntry, LogLevel, MdnsKind, MdnsService,
    NetworkInfo, PackageDetail, PackageSummary, PersonalDataResult, ProcessEntry, PropEntry,
    RebootMode, RootStatus, SensorEntry, ServiceEntry, SettingEntry, SettingNamespace, SmsRecord,
    StorageMount, TelemetrySnapshot, ThermalZone, TransportKind,
};

#[derive(Debug, Clone)]
pub struct DemoAdb {
    tick: Arc<AtomicU64>,
    extra_devices: Arc<Mutex<Vec<DeviceSummary>>>,
    use_root_unlock: Arc<Mutex<bool>>,
}

impl Default for DemoAdb {
    fn default() -> Self {
        Self::new()
    }
}

impl DemoAdb {
    pub fn new() -> Self {
        Self {
            tick: Arc::new(AtomicU64::new(0)),
            extra_devices: Arc::new(Mutex::new(Vec::new())),
            use_root_unlock: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn list_devices(&self) -> Result<Vec<DeviceSummary>> {
        let mut devs = vec![
            DeviceSummary {
                serial: "demo-pixel9pro-wifi".to_string(),
                state: DeviceState::Device,
                transport: TransportKind::Demo,
                model: Some("Pixel 9 Pro".to_string()),
                product: Some("caiman".to_string()),
                device: Some("caiman".to_string()),
                transport_id: Some("1".to_string()),
            },
            DeviceSummary {
                serial: "demo-emulator-5554".to_string(),
                state: DeviceState::Device,
                transport: TransportKind::Emulator,
                model: Some("sdk_gphone64_x86_64".to_string()),
                product: Some("sdk_gphone64_x86_64".to_string()),
                device: Some("emu64x".to_string()),
                transport_id: Some("2".to_string()),
            },
        ];
        let extra = self.extra_devices.lock().await;
        devs.extend(extra.iter().cloned());
        Ok(devs)
    }

    pub async fn list_mdns_services(&self) -> Result<Vec<MdnsService>> {
        Ok(vec![
            MdnsService {
                name: "adb-demo-pixel9".to_string(),
                service_type: "_adb-tls-connect._tcp".to_string(),
                address: "192.168.1.42".to_string(),
                port: 41893,
                kind: MdnsKind::Connect,
            },
            MdnsService {
                name: "adb-demo-pairing".to_string(),
                service_type: "_adb-tls-pairing._tcp".to_string(),
                address: "192.168.1.42".to_string(),
                port: 37199,
                kind: MdnsKind::Pairing,
            },
        ])
    }

    pub async fn pair(&self, endpoint: &str, code: &str) -> Result<String> {
        Ok(format!(
            "[DEMO] Successfully paired to {endpoint} using code {code}"
        ))
    }

    pub async fn connect(&self, endpoint: &str) -> Result<String> {
        let mut extra = self.extra_devices.lock().await;
        if !extra.iter().any(|d| d.serial == endpoint) {
            extra.push(DeviceSummary {
                serial: endpoint.to_string(),
                state: DeviceState::Device,
                transport: TransportKind::Wifi,
                model: Some("Pixel 9 Pro (Wi-Fi)".to_string()),
                product: Some("caiman".to_string()),
                device: Some("caiman".to_string()),
                transport_id: Some("9".to_string()),
            });
        }
        Ok(format!("[DEMO] connected to {endpoint}"))
    }

    pub async fn disconnect(&self, endpoint: &str) -> Result<String> {
        let mut extra = self.extra_devices.lock().await;
        extra.retain(|d| d.serial != endpoint);
        Ok(format!("[DEMO] disconnected {endpoint}"))
    }

    pub async fn enable_tcpip_and_connect(&self, _serial: &str) -> Result<String> {
        self.connect("192.168.1.42:5555").await
    }

    pub async fn check_root(&self, _serial: &str) -> Result<RootStatus> {
        Ok(RootStatus::SuAvailable)
    }

    pub async fn shell(&self, serial: &str, script: &str, use_root: bool) -> Result<String> {
        if use_root {
            *self.use_root_unlock.lock().await = true;
        }
        let trimmed = script.trim();
        let response = match trimmed {
            "id" => {
                if use_root {
                    "uid=0(root) gid=0(root) groups=0(root) context=u:r:magisk:s0".to_string()
                } else {
                    "uid=2000(shell) gid=2000(shell) groups=2000(shell),1004(input),1007(log) context=u:r:shell:s0".to_string()
                }
            }
            "uname -a" => {
                "Linux localhost 6.1.99-android15-8-g1a2b3c4d #1 SMP PREEMPT aarch64 Toybox"
                    .to_string()
            }
            s if s.starts_with("getprop ") => {
                let key = s.trim_start_matches("getprop ").trim();
                self.fetch_props(serial)
                    .await?
                    .into_iter()
                    .find(|p| p.key == key)
                    .map(|p| p.value)
                    .unwrap_or_default()
            }
            _ => format!(
                "[DEMO shell on {serial} (root={use_root})]$ {trimmed}\nCommand executed cleanly (0 warnings)."
            ),
        };
        Ok(response)
    }

    pub async fn fetch_identity(&self, serial: &str) -> Result<DeviceIdentity> {
        Ok(DeviceIdentity {
            serial: serial.to_string(),
            model: if serial.contains("emulator") {
                "sdk_gphone64_x86_64".to_string()
            } else {
                "Pixel 9 Pro".to_string()
            },
            brand: "google".to_string(),
            manufacturer: "Google".to_string(),
            device: "caiman".to_string(),
            product: "caiman".to_string(),
            android_version: "15".to_string(),
            sdk_int: "35".to_string(),
            security_patch: "2026-11-05".to_string(),
            build_id: "AP3A.241105.008".to_string(),
            fingerprint: "google/caiman/caiman:15/AP3A.241105.008/12485912:user/release-keys"
                .to_string(),
            kernel_version: "6.1.99-android15-8-g1a2b3c4d".to_string(),
            cpu_abi: "arm64-v8a".to_string(),
            screen_size: "1280x2856".to_string(),
            screen_density: "480".to_string(),
            selinux: "Enforcing".to_string(),
            verified_boot: "green".to_string(),
        })
    }

    pub async fn fetch_telemetry(&self, _serial: &str) -> Result<TelemetrySnapshot> {
        let t = self.tick.fetch_add(1, Ordering::Relaxed) as f64;
        let phase = t * 0.65;
        let cpu_total = (28.0 + 18.0 * phase.sin() + 7.0 * (phase * 1.9).cos()).clamp(4.0, 96.0);
        let cores = (0..8)
            .map(|c| {
                let offset = c as f64 * 0.7;
                (cpu_total + 14.0 * (phase + offset).sin()).clamp(2.0, 99.0)
            })
            .collect();

        let mem_total_kb = 16_384_000;
        let mem_used_delta = ((phase * 0.4).sin() * 420_000.0) as i64;
        let mem_available_kb = (7_200_000_i64 + mem_used_delta).max(1_000_000) as u64;

        let rx_rate = (42_000.0 + 35_000.0 * (phase * 0.9).sin().abs()).max(1200.0);
        let tx_rate = (14_000.0 + 12_000.0 * (phase * 1.3).cos().abs()).max(500.0);

        Ok(TelemetrySnapshot {
            captured_at: Utc::now(),
            cpu: CpuSample {
                total_pct: cpu_total,
                per_core_pct: cores,
            },
            mem_total_kb,
            mem_available_kb,
            mem_free_kb: 1_940_000,
            mem_buffers_kb: 240_000,
            mem_cached_kb: 4_850_000,
            swap_total_kb: 4_194_304,
            swap_free_kb: 3_620_000,
            battery: BatteryInfo {
                level: Some(87 - ((t as u8 / 20) % 10)),
                status: "Discharging".to_string(),
                health: "Good".to_string(),
                plugged: "Battery (Wi-Fi Debug)".to_string(),
                temperature_c: Some(32.4 + 1.2 * (phase * 0.2).sin()),
                voltage_mv: Some(4120),
                current_ma: Some(-385),
                technology: "Li-ion".to_string(),
            },
            thermals: vec![
                ThermalZone {
                    name: "soc_big".to_string(),
                    temp_c: 41.2 + 2.5 * phase.sin(),
                },
                ThermalZone {
                    name: "gpu".to_string(),
                    temp_c: 38.6 + 1.8 * phase.cos(),
                },
                ThermalZone {
                    name: "battery".to_string(),
                    temp_c: 32.4,
                },
                ThermalZone {
                    name: "skin_therm".to_string(),
                    temp_c: 34.1,
                },
            ],
            storage: vec![
                StorageMount {
                    filesystem: "/dev/block/dm-48".to_string(),
                    mount_point: "/data".to_string(),
                    size_kb: 240_000_000,
                    used_kb: 98_400_000,
                    avail_kb: 141_600_000,
                    use_pct: 41,
                },
                StorageMount {
                    filesystem: "/dev/fuse".to_string(),
                    mount_point: "/storage/emulated".to_string(),
                    size_kb: 240_000_000,
                    used_kb: 98_400_000,
                    avail_kb: 141_600_000,
                    use_pct: 41,
                },
                StorageMount {
                    filesystem: "/dev/block/dm-7".to_string(),
                    mount_point: "/system".to_string(),
                    size_kb: 4_800_000,
                    used_kb: 4_512_000,
                    avail_kb: 288_000,
                    use_pct: 94,
                },
            ],
            network: NetworkInfo {
                wifi_ssid: Some("Lab-5GHz-Mesh".to_string()),
                wifi_ip: Some("192.168.1.42/24".to_string()),
                wifi_rssi_dbm: Some(-51),
                rx_bytes_per_sec: rx_rate,
                tx_bytes_per_sec: tx_rate,
                total_rx_bytes: 1_482_910_200 + (t as u64 * 42_000),
                total_tx_bytes: 392_104_800 + (t as u64 * 14_000),
                operator: Some("Google Fi".to_string()),
            },
            uptime_secs: Some(14_820 + t as u64 * 2),
            load_avg: Some("1.42 1.28 1.15".to_string()),
        })
    }

    pub async fn fetch_props(&self, _serial: &str) -> Result<Vec<PropEntry>> {
        Ok(vec![
            ("ro.product.brand", "google"),
            ("ro.product.model", "Pixel 9 Pro"),
            ("ro.product.device", "caiman"),
            ("ro.product.cpu.abi", "arm64-v8a"),
            ("ro.build.version.release", "15"),
            ("ro.build.version.sdk", "35"),
            ("ro.build.version.security_patch", "2026-11-05"),
            ("ro.build.id", "AP3A.241105.008"),
            ("ro.boot.verifiedbootstate", "green"),
            ("ro.boot.flash.locked", "1"),
            ("ro.crypto.state", "encrypted"),
            ("persist.adb.wifi.guid", "adb-demo-pixel9"),
            ("service.adb.tcp.port", "41893"),
            ("sys.usb.state", "adb"),
            ("gsm.operator.alpha", "Google Fi"),
            ("dalvik.vm.heapsize", "512m"),
        ]
        .into_iter()
        .map(|(k, v)| PropEntry {
            key: k.to_string(),
            value: v.to_string(),
        })
        .collect())
    }

    pub async fn fetch_settings(&self, _serial: &str) -> Result<Vec<SettingEntry>> {
        Ok(vec![
            SettingEntry {
                namespace: SettingNamespace::System,
                key: "screen_brightness".into(),
                value: "164".into(),
            },
            SettingEntry {
                namespace: SettingNamespace::System,
                key: "screen_off_timeout".into(),
                value: "60000".into(),
            },
            SettingEntry {
                namespace: SettingNamespace::Secure,
                key: "default_input_method".into(),
                value: "com.google.android.inputmethod.latin/.LatinIME".into(),
            },
            SettingEntry {
                namespace: SettingNamespace::Secure,
                key: "location_mode".into(),
                value: "3".into(),
            },
            SettingEntry {
                namespace: SettingNamespace::Global,
                key: "adb_enabled".into(),
                value: "1".into(),
            },
            SettingEntry {
                namespace: SettingNamespace::Global,
                key: "adb_wifi_enabled".into(),
                value: "1".into(),
            },
            SettingEntry {
                namespace: SettingNamespace::Global,
                key: "development_settings_enabled".into(),
                value: "1".into(),
            },
        ])
    }

    pub async fn fetch_sensors(&self, _serial: &str) -> Result<Vec<SensorEntry>> {
        Ok(vec![
            SensorEntry {
                name: "LSM6DSR 6-axis Accelerometer".into(),
                vendor: "STMicroelectronics".into(),
                sensor_type: "android.sensor.accelerometer(1)".into(),
                detail: "maxRate=416Hz | fifo=3000".into(),
            },
            SensorEntry {
                name: "LSM6DSR Gyroscope".into(),
                vendor: "STMicroelectronics".into(),
                sensor_type: "android.sensor.gyroscope(4)".into(),
                detail: "maxRate=416Hz | wakeUp=false".into(),
            },
            SensorEntry {
                name: "MMC56X3X Magnetometer".into(),
                vendor: "MEMSIC".into(),
                sensor_type: "android.sensor.magnetic_field(2)".into(),
                detail: "maxRate=100Hz".into(),
            },
            SensorEntry {
                name: "ICP20100 Barometer".into(),
                vendor: "TDK InvenSense".into(),
                sensor_type: "android.sensor.pressure(6)".into(),
                detail: "maxRate=25Hz".into(),
            },
            SensorEntry {
                name: "TMD3719 Ambient Light & Proximity".into(),
                vendor: "ams-OSRAM".into(),
                sensor_type: "android.sensor.light(5)".into(),
                detail: "wakeUp=true".into(),
            },
        ])
    }

    pub async fn fetch_dumpsys_list(&self, _serial: &str) -> Result<Vec<String>> {
        Ok(vec![
            "SurfaceFlinger",
            "activity",
            "battery",
            "connectivity",
            "cpuinfo",
            "display",
            "meminfo",
            "package",
            "power",
            "sensorservice",
            "thermalservice",
            "wifi",
            "window",
        ]
        .into_iter()
        .map(ToString::to_string)
        .collect())
    }

    pub async fn fetch_dumpsys_service(
        &self,
        _serial: &str,
        service: &str,
        _use_root: bool,
    ) -> Result<String> {
        Ok(format!(
            "DUMP OF SERVICE {service} (DEMO):\nStatus: Healthy\nActive listeners: 14\nLast GC pause: 3.1ms\nMemory footprint: 42.8 MB\nWakeLocks: 0 active\n"
        ))
    }

    pub async fn fetch_processes(&self, _serial: &str, _use_root: bool) -> Result<Vec<ProcessEntry>> {
        Ok(vec![
            ProcessEntry { pid: 1842, ppid: 740, user: "system".into(), rss_kb: 312_400, vsz_kb: 18_420_000, state: "S".into(), cpu_pct: 9.4, name: "system_server".into() },
            ProcessEntry { pid: 1104, ppid: 1, user: "system".into(), rss_kb: 128_000, vsz_kb: 4_120_000, state: "S".into(), cpu_pct: 5.2, name: "/system/bin/surfaceflinger".into() },
            ProcessEntry { pid: 2490, ppid: 740, user: "u0_a185".into(), rss_kb: 245_100, vsz_kb: 16_900_000, state: "S".into(), cpu_pct: 4.1, name: "com.android.systemui".into() },
            ProcessEntry { pid: 5120, ppid: 740, user: "u0_a241".into(), rss_kb: 198_300, vsz_kb: 15_200_000, state: "R".into(), cpu_pct: 3.8, name: "com.android.chrome".into() },
            ProcessEntry { pid: 3320, ppid: 740, user: "u0_a142".into(), rss_kb: 164_800, vsz_kb: 14_800_000, state: "S".into(), cpu_pct: 2.1, name: "com.google.android.gms.persistent".into() },
            ProcessEntry { pid: 6420, ppid: 740, user: "u0_a310".into(), rss_kb: 142_200, vsz_kb: 13_400_000, state: "S".into(), cpu_pct: 1.7, name: "com.termux".into() },
            ProcessEntry { pid: 892, ppid: 1, user: "shell".into(), rss_kb: 14_200, vsz_kb: 2_100_000, state: "S".into(), cpu_pct: 0.9, name: "/apex/com.android.adbd/bin/adbd".into() },
            ProcessEntry { pid: 1, ppid: 0, user: "root".into(), rss_kb: 8_920, vsz_kb: 1_240_000, state: "S".into(), cpu_pct: 0.1, name: "/system/bin/init".into() },
        ])
    }

    pub async fn fetch_services(&self, _serial: &str, _use_root: bool) -> Result<Vec<ServiceEntry>> {
        Ok(vec![
            ServiceEntry {
                package: "com.google.android.gms".into(),
                service_class: ".gcm.GcmService".into(),
                pid: Some(3320),
                uid: Some(10142),
                state: "Running".into(),
            },
            ServiceEntry {
                package: "com.android.systemui".into(),
                service_class: ".SystemUIService".into(),
                pid: Some(2490),
                uid: Some(10185),
                state: "Running".into(),
            },
            ServiceEntry {
                package: "com.termux".into(),
                service_class: ".app.TermuxService".into(),
                pid: Some(6420),
                uid: Some(10310),
                state: "Running".into(),
            },
        ])
    }

    pub async fn fetch_packages(&self, _serial: &str) -> Result<Vec<PackageSummary>> {
        Ok(vec![
            PackageSummary { name: "com.android.chrome".into(), apk_path: "/data/app/~~k9/com.android.chrome/base.apk".into(), uid: Some(10241), version_code: Some(672309033), is_system: false, enabled: true },
            PackageSummary { name: "com.android.settings".into(), apk_path: "/system_ext/priv-app/Settings/Settings.apk".into(), uid: Some(1000), version_code: Some(35), is_system: true, enabled: true },
            PackageSummary { name: "com.android.systemui".into(), apk_path: "/system_ext/priv-app/SystemUI/SystemUI.apk".into(), uid: Some(10185), version_code: Some(35), is_system: true, enabled: true },
            PackageSummary { name: "com.google.android.gms".into(), apk_path: "/product/priv-app/PrebuiltGmsCore/PrebuiltGmsCore.apk".into(), uid: Some(10142), version_code: Some(244432038), is_system: true, enabled: true },
            PackageSummary { name: "com.signal.securesms".into(), apk_path: "/data/app/~~m4/org.thoughtcrime.securesms/base.apk".into(), uid: Some(10288), version_code: Some(148002), is_system: false, enabled: true },
            PackageSummary { name: "com.termux".into(), apk_path: "/data/app/~~a1/com.termux/base.apk".into(), uid: Some(10310), version_code: Some(118), is_system: false, enabled: true },
            PackageSummary { name: "org.mozilla.firefox".into(), apk_path: "/data/app/~~z3/org.mozilla.firefox/base.apk".into(), uid: Some(10294), version_code: Some(2016058911), is_system: false, enabled: true },
        ])
    }

    pub async fn fetch_package_detail(&self, _serial: &str, package: &str) -> Result<PackageDetail> {
        Ok(PackageDetail {
            name: package.to_string(),
            version_name: "1.18.0-demo".into(),
            version_code: "118".into(),
            target_sdk: "35".into(),
            min_sdk: "26".into(),
            uid: "10310".into(),
            apk_path: format!("/data/app/~~demo/{package}/base.apk"),
            data_dir: format!("/data/user/0/{package}"),
            first_install: "2026-09-12 10:15:00".into(),
            last_update: "2026-11-20 18:42:11".into(),
            installer: "com.android.vending".into(),
            granted_permissions: vec![
                "android.permission.INTERNET".into(),
                "android.permission.ACCESS_NETWORK_STATE".into(),
                "android.permission.WAKE_LOCK".into(),
                "android.permission.FOREGROUND_SERVICE".into(),
            ],
            denied_permissions: vec!["android.permission.CAMERA".into()],
            raw_excerpt: format!("Package [{package}] (demo)\n  targetSdk=35\n  installer=com.android.vending\n"),
        })
    }

    pub fn spawn_demo_logcat(&self, tx: mpsc::UnboundedSender<LogEntry>) -> JoinHandle<()> {
        tokio::spawn(async move {
            let samples = [
                (LogLevel::Info, "ActivityManager", "Displayed com.termux/.HomeActivity: +84ms"),
                (LogLevel::Debug, "AdbdWifi", "TLS keepalive ACK from 192.168.1.10:5037"),
                (LogLevel::Info, "NetworkMonitor", "Validated wlan0 IPv4/IPv6 dual-stack"),
                (LogLevel::Warn, "ThermalEngine", "soc_big crossed 43.0C soft threshold"),
                (LogLevel::Debug, "Choreographer", "Frame rendered in 6.2ms (120Hz)"),
                (LogLevel::Error, "MediaProvider", "Skipped hidden .nomedia cache entry"),
                (LogLevel::Verbose, "PowerManager", "Release partial wakelock AudioMix"),
            ];
            let mut idx = 0usize;
            loop {
                let (level, tag, msg) = samples[idx % samples.len()];
                let entry = LogEntry {
                    timestamp: chrono::Local::now().format("%m-%d %H:%M:%S%.3f").to_string(),
                    pid: format!("{}", 1800 + (idx % 9) * 111),
                    tid: format!("{}", 1840 + (idx % 5) * 17),
                    level,
                    tag: tag.to_string(),
                    message: format!("{msg} (#{idx})"),
                };
                if tx.send(entry).is_err() {
                    break;
                }
                idx += 1;
                tokio::time::sleep(Duration::from_millis(850)).await;
            }
        })
    }

    pub async fn list_dir(&self, _serial: &str, path: &str, use_root: bool) -> Result<LsParseResult> {
        let entries = match path {
            "/" => vec![
                FsEntry { name: "sdcard".into(), path: "/sdcard".into(), kind: FsKind::Symlink, permissions: "lrwxrwxrwx".into(), owner: "root".into(), group: "root".into(), size: 21, modified: "2026-01-01 00:00".into(), symlink_target: Some("/storage/emulated/0".into()) },
                FsEntry { name: "data".into(), path: "/data".into(), kind: FsKind::Directory, permissions: "drwxrwx--x".into(), owner: "system".into(), group: "system".into(), size: 4096, modified: "2026-11-24 12:00".into(), symlink_target: None },
                FsEntry { name: "system".into(), path: "/system".into(), kind: FsKind::Directory, permissions: "drwxr-xr-x".into(), owner: "root".into(), group: "root".into(), size: 4096, modified: "2026-11-05 00:00".into(), symlink_target: None },
            ],
            "/data" if !use_root => {
                return Ok(LsParseResult {
                    entries: Vec::new(),
                    permission_denied: true,
                });
            }
            "/data" => vec![
                FsEntry { name: "local".into(), path: "/data/local".into(), kind: FsKind::Directory, permissions: "drwxr-xr-x".into(), owner: "root".into(), group: "root".into(), size: 4096, modified: "2026-11-20 10:00".into(), symlink_target: None },
                FsEntry { name: "anr".into(), path: "/data/anr".into(), kind: FsKind::Directory, permissions: "drwxrwxr-x".into(), owner: "system".into(), group: "system".into(), size: 4096, modified: "2026-11-22 08:15".into(), symlink_target: None },
            ],
            "/data/local/tmp" | "/data/local" => vec![
                FsEntry { name: "tmp".into(), path: "/data/local/tmp".into(), kind: FsKind::Directory, permissions: "drwxrwx--x".into(), owner: "shell".into(), group: "shell".into(), size: 4096, modified: "2026-11-24 14:20".into(), symlink_target: None },
                FsEntry { name: "adbsight_probe.txt".into(), path: "/data/local/tmp/adbsight_probe.txt".into(), kind: FsKind::File, permissions: "-rw-r--r--".into(), owner: "shell".into(), group: "shell".into(), size: 128, modified: "2026-11-24 16:40".into(), symlink_target: None },
            ],
            _ => vec![
                FsEntry { name: "Download".into(), path: "/sdcard/Download".into(), kind: FsKind::Directory, permissions: "drwxrwx---".into(), owner: "u0_a190".into(), group: "everybody".into(), size: 4096, modified: "2026-11-24 15:10".into(), symlink_target: None },
                FsEntry { name: "DCIM".into(), path: "/sdcard/DCIM".into(), kind: FsKind::Directory, permissions: "drwxrwx---".into(), owner: "u0_a190".into(), group: "everybody".into(), size: 4096, modified: "2026-11-23 19:02".into(), symlink_target: None },
                FsEntry { name: "device_notes.txt".into(), path: "/sdcard/device_notes.txt".into(), kind: FsKind::File, permissions: "-rw-rw----".into(), owner: "u0_a190".into(), group: "everybody".into(), size: 256, modified: "2026-11-24 16:12".into(), symlink_target: None },
                FsEntry { name: "crash_trace.log".into(), path: "/sdcard/crash_trace.log".into(), kind: FsKind::File, permissions: "-rw-rw----".into(), owner: "u0_a190".into(), group: "everybody".into(), size: 1024, modified: "2026-11-24 13:45".into(), symlink_target: None },
            ],
        };
        Ok(LsParseResult {
            entries,
            permission_denied: false,
        })
    }

    pub async fn preview_file(&self, _serial: &str, path: &str, _use_root: bool) -> Result<String> {
        Ok(format!(
            "# Demo Preview: {path}\nCaptured by ADBSight Demo Simulator.\nWi-Fi Debugging: Active on 192.168.1.42:41893\nSELinux: Enforcing\n"
        ))
    }

    pub async fn pull_path(&self, _serial: &str, remote: &str, local: &Path) -> Result<String> {
        if let Some(parent) = local.parent().filter(|p| !p.as_os_str().is_empty()) {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(local, format!("[DEMO pulled content from {remote}]\n")).await?;
        Ok(format!("1 file pulled (demo) -> {}", local.display()))
    }

    pub async fn push_path(&self, _serial: &str, local: &Path, remote: &str) -> Result<String> {
        if !local.exists() {
            bail!("local file does not exist: {}", local.display());
        }
        Ok(format!("1 file pushed (demo) -> {remote}"))
    }

    pub async fn fetch_contacts(
        &self,
        _serial: &str,
        _use_root: bool,
    ) -> Result<PersonalDataResult<ContactRecord>> {
        Ok(PersonalDataResult {
            records: vec![
                ContactRecord {
                    name: "Alex Rivera".into(),
                    phone_numbers: vec!["+1-555-0142".into(), "+1-555-0199".into()],
                    emails: vec!["alex.rivera@example.com".into()],
                },
                ContactRecord {
                    name: "Maya Lin".into(),
                    phone_numbers: vec!["+1-555-0175".into()],
                    emails: vec!["maya.lin@example.org".into()],
                },
                ContactRecord {
                    name: "Android Lab Support".into(),
                    phone_numbers: vec!["+1-800-555-0100".into()],
                    emails: vec!["lab@android.dev".into()],
                },
            ],
            permission_hint: None,
            raw_error: None,
        })
    }

    pub async fn fetch_sms(
        &self,
        _serial: &str,
        use_root: bool,
    ) -> Result<PersonalDataResult<SmsRecord>> {
        if !use_root {
            return Ok(PersonalDataResult {
                records: vec![SmsRecord {
                    address: "ADBSight Hint".into(),
                    body: "Press 'R' to toggle Root Mode in Demo to see simulated unlocked SMS rows (on stock Android 10–15+, shell cannot read SMS without root).".into(),
                    timestamp_ms: Some(Utc::now().timestamp_millis()),
                    date_display: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    msg_type: "Inbox".into(),
                }],
                permission_hint: Some("Simulated non-root restriction: toggle Root Mode ('R') to inspect full SMS dataset.".into()),
                raw_error: None,
            });
        }
        Ok(PersonalDataResult {
            records: vec![
                SmsRecord {
                    address: "+1-555-0142".into(),
                    body: "Hey, the Wi-Fi ADB pairing QR code worked on the first try!".into(),
                    timestamp_ms: Some(1_732_450_000_000),
                    date_display: "2026-11-24 15:20:00".into(),
                    msg_type: "Inbox".into(),
                },
                SmsRecord {
                    address: "+1-555-0175".into(),
                    body: "Sent the kernel trace & APK dump, check /sdcard/Download.".into(),
                    timestamp_ms: Some(1_732_442_000_000),
                    date_display: "2026-11-24 13:06:40".into(),
                    msg_type: "Sent".into(),
                },
            ],
            permission_hint: None,
            raw_error: None,
        })
    }

    pub async fn fetch_call_logs(
        &self,
        _serial: &str,
        use_root: bool,
    ) -> Result<PersonalDataResult<CallLogRecord>> {
        if !use_root {
            return Ok(PersonalDataResult {
                records: Vec::new(),
                permission_hint: Some("Simulated non-root restriction: toggle Root Mode ('R') to inspect Call Logs.".into()),
                raw_error: None,
            });
        }
        Ok(PersonalDataResult {
            records: vec![
                CallLogRecord {
                    number: "+1-555-0142".into(),
                    call_type: "Incoming".into(),
                    timestamp_ms: Some(1_732_448_000_000),
                    date_display: "2026-11-24 14:46:40".into(),
                    duration_secs: 245,
                },
                CallLogRecord {
                    number: "+1-555-0175".into(),
                    call_type: "Missed".into(),
                    timestamp_ms: Some(1_732_436_000_000),
                    date_display: "2026-11-24 11:26:40".into(),
                    duration_secs: 0,
                },
            ],
            permission_hint: None,
            raw_error: None,
        })
    }

    pub async fn fetch_accounts(
        &self,
        _serial: &str,
        _use_root: bool,
    ) -> Result<PersonalDataResult<AccountRecord>> {
        Ok(PersonalDataResult {
            records: vec![
                AccountRecord {
                    account_type: "com.google".into(),
                    name: "developer@example.com".into(),
                },
                AccountRecord {
                    account_type: "com.github.android".into(),
                    name: "adper-dev".into(),
                },
            ],
            permission_hint: None,
            raw_error: None,
        })
    }

    pub async fn launch_app(&self, _serial: &str, pkg: &str) -> Result<String> {
        Ok(format!("[DEMO] Launched {pkg}"))
    }

    pub async fn force_stop(&self, _serial: &str, pkg: &str) -> Result<String> {
        Ok(format!("[DEMO] Force-stopped {pkg}"))
    }

    pub async fn clear_app_data(&self, _serial: &str, pkg: &str) -> Result<String> {
        Ok(format!("[DEMO] Cleared data for {pkg}"))
    }

    pub async fn kill_pid(&self, _serial: &str, pid: u32) -> Result<String> {
        Ok(format!("[DEMO] Killed PID {pid}"))
    }

    pub async fn screencap(&self, _serial: &str, local_png: &Path) -> Result<()> {
        if let Some(parent) = local_png.parent().filter(|p| !p.as_os_str().is_empty()) {
            tokio::fs::create_dir_all(parent).await?;
        }
        // Minimal 1x1 PNG header bytes so generated file is valid PNG in demo mode
        let tiny_png: &[u8] = &[
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00,
            0x00, 0x90, 0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08,
            0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, 0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D,
            0xB0, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ];
        tokio::fs::write(local_png, tiny_png).await?;
        Ok(())
    }

    pub async fn reboot(&self, _serial: &str, mode: RebootMode) -> Result<String> {
        Ok(format!("[DEMO] Simulated {}", mode.label()))
    }

    pub async fn send_input_text(&self, _serial: &str, text: &str) -> Result<String> {
        Ok(format!("[DEMO] Sent input text: {text}"))
    }
}
