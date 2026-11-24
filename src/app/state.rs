use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::adb::wireless::QrPairingSession;
use crate::models::{
    AccountRecord, CallLogRecord, ContactRecord, DeviceIdentity, DeviceSummary, FsEntry,
    FullDeviceSnapshot, LogEntry, LogLevel, MdnsService, PackageDetail, PackageSummary,
    PersonalDataResult, ProcessEntry, PropEntry, RebootMode, RootStatus, SensorEntry, ServiceEntry,
    SettingEntry, SmsRecord, SnapshotMeta, TelemetrySnapshot,
};
use crate::storage::config::AppConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    System,
    Processes,
    Packages,
    Logcat,
    Files,
    Personal,
    Shell,
}

impl Tab {
    pub const ALL: [Tab; 8] = [
        Tab::Dashboard,
        Tab::System,
        Tab::Processes,
        Tab::Packages,
        Tab::Logcat,
        Tab::Files,
        Tab::Personal,
        Tab::Shell,
    ];

    pub fn index(&self) -> usize {
        match self {
            Self::Dashboard => 0,
            Self::System => 1,
            Self::Processes => 2,
            Self::Packages => 3,
            Self::Logcat => 4,
            Self::Files => 5,
            Self::Personal => 6,
            Self::Shell => 7,
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Self::Dashboard => "1 Dashboard",
            Self::System => "2 System",
            Self::Processes => "3 Processes",
            Self::Packages => "4 Packages",
            Self::Logcat => "5 Logcat",
            Self::Files => "6 Files",
            Self::Personal => "7 Personal",
            Self::Shell => "8 Shell",
        }
    }

    pub fn from_digit(d: char) -> Option<Self> {
        match d {
            '1' => Some(Self::Dashboard),
            '2' => Some(Self::System),
            '3' => Some(Self::Processes),
            '4' => Some(Self::Packages),
            '5' => Some(Self::Logcat),
            '6' => Some(Self::Files),
            '7' => Some(Self::Personal),
            '8' => Some(Self::Shell),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TextInput {
    pub value: String,
    pub active: bool,
}

impl TextInput {
    pub fn with_value(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            active: false,
        }
    }

    pub fn push(&mut self, ch: char) {
        self.value.push(ch);
    }

    pub fn pop(&mut self) {
        self.value.pop();
    }

    pub fn clear(&mut self) {
        self.value.clear();
    }
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub is_error: bool,
    pub expires_at: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemSubTab {
    Properties,
    Settings,
    Sensors,
    Dumpsys,
}

impl SystemSubTab {
    pub const ALL: [Self; 4] = [Self::Properties, Self::Settings, Self::Sensors, Self::Dumpsys];

    pub fn title(&self) -> &'static str {
        match self {
            Self::Properties => "Properties",
            Self::Settings => "Settings",
            Self::Sensors => "Sensors & HW",
            Self::Dumpsys => "Dumpsys",
        }
    }

    pub fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|&x| x == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    pub fn prev(self) -> Self {
        let idx = Self::ALL.iter().position(|&x| x == self).unwrap_or(0);
        Self::ALL[(idx + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessSubTab {
    Processes,
    Services,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessSort {
    CpuDesc,
    RamDesc,
    PidAsc,
    NameAsc,
}

impl ProcessSort {
    pub fn next(self) -> Self {
        match self {
            Self::CpuDesc => Self::RamDesc,
            Self::RamDesc => Self::PidAsc,
            Self::PidAsc => Self::NameAsc,
            Self::NameAsc => Self::CpuDesc,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::CpuDesc => "CPU%",
            Self::RamDesc => "RAM",
            Self::PidAsc => "PID",
            Self::NameAsc => "Name",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageFilter {
    All,
    UserOnly,
    SystemOnly,
}

impl PackageFilter {
    pub fn next(self) -> Self {
        match self {
            Self::All => Self::UserOnly,
            Self::UserOnly => Self::SystemOnly,
            Self::SystemOnly => Self::All,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::All => "All",
            Self::UserOnly => "User",
            Self::SystemOnly => "System",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersonalSubTab {
    Contacts,
    Sms,
    CallLogs,
    Accounts,
}

impl PersonalSubTab {
    pub const ALL: [Self; 4] = [Self::Contacts, Self::Sms, Self::CallLogs, Self::Accounts];

    pub fn title(&self) -> &'static str {
        match self {
            Self::Contacts => "Contacts",
            Self::Sms => "SMS",
            Self::CallLogs => "Call Logs",
            Self::Accounts => "Accounts",
        }
    }

    pub fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|&x| x == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    pub fn prev(self) -> Self {
        let idx = Self::ALL.iter().position(|&x| x == self).unwrap_or(0);
        Self::ALL[(idx + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WirelessSubTab {
    DevicesAndMdns,
    QrPair,
    ManualPairConnect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WirelessField {
    Endpoint,
    PairingCode,
}

#[derive(Debug, Clone)]
pub struct WirelessModalState {
    pub sub_tab: WirelessSubTab,
    pub selected_device: usize,
    pub selected_mdns: usize,
    pub selected_bookmark: usize,
    pub focus_section: usize, // 0=devices, 1=mdns, 2=bookmarks
    pub qr_session: Option<QrPairingSession>,
    pub qr_status: String,
    pub endpoint_input: TextInput,
    pub code_input: TextInput,
    pub active_field: WirelessField,
}

impl Default for WirelessModalState {
    fn default() -> Self {
        Self {
            sub_tab: WirelessSubTab::DevicesAndMdns,
            selected_device: 0,
            selected_mdns: 0,
            selected_bookmark: 0,
            focus_section: 0,
            qr_session: None,
            qr_status: "Generate a QR code to pair Android 11+ over Wi-Fi.".into(),
            endpoint_input: TextInput::default(),
            code_input: TextInput::default(),
            active_field: WirelessField::Endpoint,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportChoice {
    ActiveTabJson,
    ActiveTabCsv,
    FullSnapshotJson,
    FullMarkdownReport,
}

impl ExportChoice {
    pub const ALL: [Self; 4] = [
        Self::ActiveTabJson,
        Self::ActiveTabCsv,
        Self::FullSnapshotJson,
        Self::FullMarkdownReport,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::ActiveTabJson => "Export Active Tab -> JSON",
            Self::ActiveTabCsv => "Export Active Tab -> CSV",
            Self::FullSnapshotJson => "Export Full Device Snapshot -> JSON",
            Self::FullMarkdownReport => "Export Full Inspection Dossier -> Markdown (.md)",
        }
    }
}

#[derive(Debug, Clone)]
pub enum PendingAction {
    KillPid { pid: u32, name: String },
    ForceStopApp(String),
    ClearAppData(String),
    PushFile { local: PathBuf, remote: String },
    Reboot(RebootMode),
}

#[derive(Debug, Clone)]
pub enum PromptKind {
    PullFile { remote: String },
    PushFile { remote_dir: String },
    SendInputText,
}

#[derive(Debug, Clone)]
pub enum Modal {
    Wireless(WirelessModalState),
    Export { selected: usize },
    SnapshotSave(TextInput),
    SnapshotHistory { list: Vec<SnapshotMeta>, selected: usize },
    ConfirmAction { title: String, message: String, action: PendingAction },
    InputPrompt { title: String, prompt: String, input: TextInput, kind: PromptKind },
    Help,
}

#[derive(Debug, Default)]
pub struct DashboardState {
    pub identity: Option<DeviceIdentity>,
    pub telemetry: Option<TelemetrySnapshot>,
    pub cpu_history: VecDeque<u64>,
    pub rx_history: VecDeque<u64>,
    pub tx_history: VecDeque<u64>,
}

#[derive(Debug)]
pub struct SystemState {
    pub sub_tab: SystemSubTab,
    pub props: Vec<PropEntry>,
    pub settings: Vec<SettingEntry>,
    pub sensors: Vec<SensorEntry>,
    pub dumpsys_list: Vec<String>,
    pub dumpsys_output: String,
    pub dumpsys_active_service: Option<String>,
    pub dumpsys_scroll: u16,
    pub selected: usize,
    pub filter: TextInput,
    pub loading: bool,
}

impl Default for SystemState {
    fn default() -> Self {
        Self {
            sub_tab: SystemSubTab::Properties,
            props: Vec::new(),
            settings: Vec::new(),
            sensors: Vec::new(),
            dumpsys_list: Vec::new(),
            dumpsys_output: String::new(),
            dumpsys_active_service: None,
            dumpsys_scroll: 0,
            selected: 0,
            filter: TextInput::default(),
            loading: false,
        }
    }
}

#[derive(Debug)]
pub struct ProcessesState {
    pub sub_tab: ProcessSubTab,
    pub processes: Vec<ProcessEntry>,
    pub services: Vec<ServiceEntry>,
    pub sort: ProcessSort,
    pub selected: usize,
    pub filter: TextInput,
    pub loading: bool,
}

impl Default for ProcessesState {
    fn default() -> Self {
        Self {
            sub_tab: ProcessSubTab::Processes,
            processes: Vec::new(),
            services: Vec::new(),
            sort: ProcessSort::CpuDesc,
            selected: 0,
            filter: TextInput::default(),
            loading: false,
        }
    }
}

#[derive(Debug)]
pub struct PackagesState {
    pub packages: Vec<PackageSummary>,
    pub filter_mode: PackageFilter,
    pub filter: TextInput,
    pub selected: usize,
    pub detail: Option<PackageDetail>,
    pub detail_scroll: u16,
    pub loading: bool,
    pub detail_loading: bool,
}

impl Default for PackagesState {
    fn default() -> Self {
        Self {
            packages: Vec::new(),
            filter_mode: PackageFilter::All,
            filter: TextInput::default(),
            selected: 0,
            detail: None,
            detail_scroll: 0,
            loading: false,
            detail_loading: false,
        }
    }
}

#[derive(Debug)]
pub struct LogcatState {
    pub entries: VecDeque<LogEntry>,
    pub max_lines: usize,
    pub min_level: LogLevel,
    pub paused: bool,
    pub auto_scroll: bool,
    pub scroll: usize,
    pub filter: TextInput,
    pub streaming: bool,
}

impl LogcatState {
    pub fn new(max_lines: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(max_lines.min(1024)),
            max_lines,
            min_level: LogLevel::Verbose,
            paused: false,
            auto_scroll: true,
            scroll: 0,
            filter: TextInput::default(),
            streaming: false,
        }
    }
}

#[derive(Debug)]
pub struct FilesState {
    pub current_path: String,
    pub entries: Vec<FsEntry>,
    pub selected: usize,
    pub permission_denied: bool,
    pub preview_title: String,
    pub preview_content: String,
    pub preview_scroll: u16,
    pub filter: TextInput,
    pub loading: bool,
}

impl Default for FilesState {
    fn default() -> Self {
        Self {
            current_path: "/sdcard".to_string(),
            entries: Vec::new(),
            selected: 0,
            permission_denied: false,
            preview_title: String::new(),
            preview_content: String::new(),
            preview_scroll: 0,
            filter: TextInput::default(),
            loading: false,
        }
    }
}

#[derive(Debug)]
pub struct PersonalState {
    pub sub_tab: PersonalSubTab,
    pub contacts: PersonalDataResult<ContactRecord>,
    pub sms: PersonalDataResult<SmsRecord>,
    pub call_logs: PersonalDataResult<CallLogRecord>,
    pub accounts: PersonalDataResult<AccountRecord>,
    pub selected: usize,
    pub filter: TextInput,
    pub loading: bool,
}

impl Default for PersonalState {
    fn default() -> Self {
        Self {
            sub_tab: PersonalSubTab::Contacts,
            contacts: PersonalDataResult::default(),
            sms: PersonalDataResult::default(),
            call_logs: PersonalDataResult::default(),
            accounts: PersonalDataResult::default(),
            selected: 0,
            filter: TextInput::default(),
            loading: false,
        }
    }
}

#[derive(Debug)]
pub struct ShellState {
    pub input: TextInput,
    pub history: Vec<String>,
    pub history_idx: Option<usize>,
    pub output_lines: Vec<String>,
    pub scroll: u16,
    pub running: bool,
}

impl Default for ShellState {
    fn default() -> Self {
        let mut s = Self {
            input: TextInput {
                value: String::new(),
                active: true,
            },
            history: Vec::new(),
            history_idx: None,
            output_lines: vec![
                "ADBSight Interactive Shell & Device Action Deck".into(),
                "Type an Android shell command and press Enter. Quick keys: [F2] Screenshot, [F3] Reboot, [F4] Snapshot, [Ctrl+I] Input Text".into(),
            ],
            scroll: 0,
            running: false,
        };
        s.input.active = true;
        s
    }
}

pub struct AppState {
    pub active_tab: Tab,
    pub should_quit: bool,
    pub is_demo: bool,
    pub adb_info_label: String,
    pub config: AppConfig,
    pub devices: Vec<DeviceSummary>,
    pub mdns_services: Vec<MdnsService>,
    pub active_serial: Option<String>,
    pub root_status: RootStatus,
    pub use_root: bool,
    pub loaded_tabs_for_serial: HashMap<String, Vec<Tab>>,
    pub viewing_snapshot: Option<SnapshotMeta>,
    pub live_snapshot_cache: Option<FullDeviceSnapshot>,
    pub dashboard: DashboardState,
    pub system: SystemState,
    pub processes: ProcessesState,
    pub packages: PackagesState,
    pub logcat: LogcatState,
    pub files: FilesState,
    pub personal: PersonalState,
    pub shell: ShellState,
    pub modal: Option<Modal>,
    pub toasts: VecDeque<Toast>,
}

impl AppState {
    pub fn new(config: AppConfig, is_demo: bool, adb_info_label: String) -> Self {
        let max_lines = config.logcat_max_lines;
        Self {
            active_tab: Tab::Dashboard,
            should_quit: false,
            is_demo,
            adb_info_label,
            config,
            devices: Vec::new(),
            mdns_services: Vec::new(),
            active_serial: None,
            root_status: RootStatus::None,
            use_root: false,
            loaded_tabs_for_serial: HashMap::new(),
            viewing_snapshot: None,
            live_snapshot_cache: None,
            dashboard: DashboardState::default(),
            system: SystemState::default(),
            processes: ProcessesState::default(),
            packages: PackagesState::default(),
            logcat: LogcatState::new(max_lines),
            files: FilesState::default(),
            personal: PersonalState::default(),
            shell: ShellState::default(),
            modal: None,
            toasts: VecDeque::new(),
        }
    }

    pub fn push_toast(&mut self, message: impl Into<String>, is_error: bool) {
        self.toasts.push_back(Toast {
            message: message.into(),
            is_error,
            expires_at: Instant::now() + Duration::from_secs(4),
        });
        while self.toasts.len() > 4 {
            self.toasts.pop_front();
        }
    }

    pub fn prune_toasts(&mut self) {
        let now = Instant::now();
        while self.toasts.front().is_some_and(|t| t.expires_at <= now) {
            self.toasts.pop_front();
        }
    }

    pub fn active_device(&self) -> Option<&DeviceSummary> {
        let serial = self.active_serial.as_deref()?;
        self.devices.iter().find(|d| d.serial == serial)
    }

    pub fn is_tab_loaded(&self, serial: &str, tab: Tab) -> bool {
        self.loaded_tabs_for_serial
            .get(serial)
            .is_some_and(|tabs| tabs.contains(&tab))
    }

    pub fn mark_tab_loaded(&mut self, serial: &str, tab: Tab) {
        let entry = self
            .loaded_tabs_for_serial
            .entry(serial.to_string())
            .or_default();
        if !entry.contains(&tab) {
            entry.push(tab);
        }
    }

    pub fn reset_device_caches(&mut self) {
        self.loaded_tabs_for_serial.clear();
        self.dashboard = DashboardState::default();
        self.system = SystemState::default();
        self.processes = ProcessesState::default();
        self.packages = PackagesState::default();
        self.files = FilesState::default();
        self.personal = PersonalState::default();
        self.root_status = RootStatus::None;
        self.use_root = false;
    }

    pub fn apply_loaded_snapshot(&mut self, meta: SnapshotMeta, snap: FullDeviceSnapshot) {
        self.viewing_snapshot = Some(meta);
        self.live_snapshot_cache = Some(snap.clone());
        self.dashboard.identity = Some(snap.identity);
        self.dashboard.telemetry = Some(snap.telemetry);
        self.system.props = snap.props;
        self.packages.packages = snap.packages;
        self.processes.processes = snap.processes;
        self.personal.contacts = snap.contacts;
        self.personal.sms = snap.sms;
        self.personal.call_logs = snap.call_logs;
        self.personal.accounts = snap.accounts;
    }
}
