use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceState {
    Device,
    Offline,
    Unauthorized,
    Authorizing,
    Recovery,
    Sideload,
    Unknown(String),
}

impl DeviceState {
    pub fn from_adb(raw: &str) -> Self {
        match raw.trim() {
            "device" => Self::Device,
            "offline" => Self::Offline,
            "unauthorized" => Self::Unauthorized,
            "authorizing" => Self::Authorizing,
            "recovery" => Self::Recovery,
            "sideload" => Self::Sideload,
            other => Self::Unknown(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Device => "device",
            Self::Offline => "offline",
            Self::Unauthorized => "unauthorized",
            Self::Authorizing => "authorizing",
            Self::Recovery => "recovery",
            Self::Sideload => "sideload",
            Self::Unknown(s) => s.as_str(),
        }
    }

    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Device)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportKind {
    Usb,
    Wifi,
    Emulator,
    Demo,
}

impl TransportKind {
    pub fn infer(serial: &str) -> Self {
        if serial.starts_with("demo-") {
            Self::Demo
        } else if serial.starts_with("emulator-") {
            Self::Emulator
        } else if serial.contains(':') || serial.contains("_adb-tls-connect") {
            Self::Wifi
        } else {
            Self::Usb
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Usb => "USB",
            Self::Wifi => "Wi-Fi",
            Self::Emulator => "EMU",
            Self::Demo => "DEMO",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSummary {
    pub serial: String,
    pub state: DeviceState,
    pub transport: TransportKind,
    pub model: Option<String>,
    pub product: Option<String>,
    pub device: Option<String>,
    pub transport_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MdnsKind {
    Pairing,
    Connect,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdnsService {
    pub name: String,
    pub service_type: String,
    pub address: String,
    pub port: u16,
    pub kind: MdnsKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RootStatus {
    #[default]
    None,
    AdbRoot,
    SuAvailable,
}

impl RootStatus {
    pub fn is_available(&self) -> bool {
        !matches!(self, Self::None)
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::None => "USER",
            Self::AdbRoot => "ADB-ROOT",
            Self::SuAvailable => "SU-ROOT",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CpuSample {
    pub total_pct: f64,
    pub per_core_pct: Vec<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BatteryInfo {
    pub level: Option<u8>,
    pub status: String,
    pub health: String,
    pub plugged: String,
    pub temperature_c: Option<f64>,
    pub voltage_mv: Option<u32>,
    pub current_ma: Option<i32>,
    pub technology: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalZone {
    pub name: String,
    pub temp_c: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMount {
    pub filesystem: String,
    pub mount_point: String,
    pub size_kb: u64,
    pub used_kb: u64,
    pub avail_kb: u64,
    pub use_pct: u8,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub wifi_ssid: Option<String>,
    pub wifi_ip: Option<String>,
    pub wifi_rssi_dbm: Option<i32>,
    pub rx_bytes_per_sec: f64,
    pub tx_bytes_per_sec: f64,
    pub total_rx_bytes: u64,
    pub total_tx_bytes: u64,
    pub operator: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TelemetrySnapshot {
    pub captured_at: DateTime<Utc>,
    pub cpu: CpuSample,
    pub mem_total_kb: u64,
    pub mem_available_kb: u64,
    pub mem_free_kb: u64,
    pub mem_buffers_kb: u64,
    pub mem_cached_kb: u64,
    pub swap_total_kb: u64,
    pub swap_free_kb: u64,
    pub battery: BatteryInfo,
    pub thermals: Vec<ThermalZone>,
    pub storage: Vec<StorageMount>,
    pub network: NetworkInfo,
    pub uptime_secs: Option<u64>,
    pub load_avg: Option<String>,
}

impl TelemetrySnapshot {
    pub fn mem_used_kb(&self) -> u64 {
        self.mem_total_kb.saturating_sub(self.mem_available_kb)
    }

    pub fn mem_used_pct(&self) -> f64 {
        if self.mem_total_kb == 0 {
            0.0
        } else {
            (self.mem_used_kb() as f64 / self.mem_total_kb as f64) * 100.0
        }
    }

    pub fn swap_used_kb(&self) -> u64 {
        self.swap_total_kb.saturating_sub(self.swap_free_kb)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceIdentity {
    pub serial: String,
    pub model: String,
    pub brand: String,
    pub manufacturer: String,
    pub device: String,
    pub product: String,
    pub android_version: String,
    pub sdk_int: String,
    pub security_patch: String,
    pub build_id: String,
    pub fingerprint: String,
    pub kernel_version: String,
    pub cpu_abi: String,
    pub screen_size: String,
    pub screen_density: String,
    pub selinux: String,
    pub verified_boot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropEntry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettingNamespace {
    System,
    Secure,
    Global,
}

impl SettingNamespace {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Secure => "secure",
            Self::Global => "global",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingEntry {
    pub namespace: SettingNamespace,
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorEntry {
    pub name: String,
    pub vendor: String,
    pub sensor_type: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessEntry {
    pub pid: u32,
    pub ppid: u32,
    pub user: String,
    pub rss_kb: u64,
    pub vsz_kb: u64,
    pub state: String,
    pub cpu_pct: f64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEntry {
    pub package: String,
    pub service_class: String,
    pub pid: Option<u32>,
    pub uid: Option<u32>,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSummary {
    pub name: String,
    pub apk_path: String,
    pub uid: Option<u32>,
    pub version_code: Option<i64>,
    pub is_system: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackageDetail {
    pub name: String,
    pub version_name: String,
    pub version_code: String,
    pub target_sdk: String,
    pub min_sdk: String,
    pub uid: String,
    pub apk_path: String,
    pub data_dir: String,
    pub first_install: String,
    pub last_update: String,
    pub installer: String,
    pub granted_permissions: Vec<String>,
    pub denied_permissions: Vec<String>,
    pub raw_excerpt: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    Verbose,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
    Silent,
}

impl LogLevel {
    pub fn from_char(c: char) -> Self {
        match c {
            'V' => Self::Verbose,
            'D' => Self::Debug,
            'I' => Self::Info,
            'W' => Self::Warn,
            'E' => Self::Error,
            'F' | 'A' => Self::Fatal,
            _ => Self::Silent,
        }
    }

    pub fn code(&self) -> char {
        match self {
            Self::Verbose => 'V',
            Self::Debug => 'D',
            Self::Info => 'I',
            Self::Warn => 'W',
            Self::Error => 'E',
            Self::Fatal => 'F',
            Self::Silent => 'S',
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub pid: String,
    pub tid: String,
    pub level: LogLevel,
    pub tag: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FsKind {
    Directory,
    File,
    Symlink,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsEntry {
    pub name: String,
    pub path: String,
    pub kind: FsKind,
    pub permissions: String,
    pub owner: String,
    pub group: String,
    pub size: u64,
    pub modified: String,
    pub symlink_target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactRecord {
    pub name: String,
    pub phone_numbers: Vec<String>,
    pub emails: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsRecord {
    pub address: String,
    pub body: String,
    pub timestamp_ms: Option<i64>,
    pub date_display: String,
    pub msg_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallLogRecord {
    pub number: String,
    pub call_type: String,
    pub timestamp_ms: Option<i64>,
    pub date_display: String,
    pub duration_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountRecord {
    pub account_type: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalDataResult<T> {
    pub records: Vec<T>,
    pub permission_hint: Option<String>,
    pub raw_error: Option<String>,
}

impl<T> Default for PersonalDataResult<T> {
    fn default() -> Self {
        Self {
            records: Vec::new(),
            permission_hint: None,
            raw_error: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMeta {
    pub id: i64,
    pub device_serial: String,
    pub model: String,
    pub android_version: String,
    pub label: String,
    pub taken_at: DateTime<Utc>,
    pub root_used: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FullDeviceSnapshot {
    pub taken_at: DateTime<Utc>,
    pub root_used: bool,
    pub identity: DeviceIdentity,
    pub telemetry: TelemetrySnapshot,
    pub props: Vec<PropEntry>,
    pub packages: Vec<PackageSummary>,
    pub processes: Vec<ProcessEntry>,
    pub contacts: PersonalDataResult<ContactRecord>,
    pub sms: PersonalDataResult<SmsRecord>,
    pub call_logs: PersonalDataResult<CallLogRecord>,
    pub accounts: PersonalDataResult<AccountRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RebootMode {
    System,
    Recovery,
    Bootloader,
}

impl RebootMode {
    pub fn adb_arg(&self) -> Option<&'static str> {
        match self {
            Self::System => None,
            Self::Recovery => Some("recovery"),
            Self::Bootloader => Some("bootloader"),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::System => "System Reboot",
            Self::Recovery => "Reboot to Recovery",
            Self::Bootloader => "Reboot to Bootloader",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedWifiDevice {
    pub name: String,
    pub address: String,
    pub last_connected: Option<DateTime<Utc>>,
}
