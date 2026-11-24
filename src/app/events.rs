use std::path::PathBuf;

use crate::adb::parsers::filesystem::LsParseResult;
use crate::models::{
    AccountRecord, CallLogRecord, ContactRecord, DeviceIdentity, DeviceSummary, FullDeviceSnapshot,
    LogEntry, MdnsService, PackageDetail, PackageSummary, PersonalDataResult, ProcessEntry,
    PropEntry, RebootMode, RootStatus, SensorEntry, ServiceEntry, SettingEntry, SmsRecord,
    SnapshotMeta, TelemetrySnapshot,
};

#[derive(Debug, Clone)]
pub enum WorkerCommand {
    RefreshDevices,
    RefreshMdns,
    PairWifi {
        endpoint: String,
        code: String,
    },
    ConnectWifi {
        endpoint: String,
        alias: Option<String>,
    },
    DisconnectWifi {
        endpoint: String,
    },
    UsbToWifi {
        usb_serial: String,
    },
    WatchQrPairing {
        service_name: String,
        password: String,
    },
    CheckRoot {
        serial: String,
    },
    FetchIdentity {
        serial: String,
    },
    FetchTelemetry {
        serial: String,
    },
    FetchSystemAll {
        serial: String,
    },
    FetchDumpsysService {
        serial: String,
        service: String,
        use_root: bool,
    },
    FetchProcesses {
        serial: String,
        use_root: bool,
    },
    FetchPackages {
        serial: String,
    },
    FetchPackageDetail {
        serial: String,
        package: String,
    },
    ListDir {
        serial: String,
        path: String,
        use_root: bool,
    },
    PreviewFile {
        serial: String,
        path: String,
        use_root: bool,
    },
    PullPath {
        serial: String,
        remote: String,
        local: PathBuf,
    },
    PushPath {
        serial: String,
        local: PathBuf,
        remote: String,
    },
    FetchPersonalAll {
        serial: String,
        use_root: bool,
    },
    RunShell {
        serial: String,
        command: String,
        use_root: bool,
    },
    KillPid {
        serial: String,
        pid: u32,
        use_root: bool,
    },
    LaunchApp {
        serial: String,
        package: String,
    },
    ForceStopApp {
        serial: String,
        package: String,
    },
    ClearAppData {
        serial: String,
        package: String,
    },
    PullApk {
        serial: String,
        apk_remote: String,
        local: PathBuf,
    },
    Screencap {
        serial: String,
        local: PathBuf,
    },
    Reboot {
        serial: String,
        mode: RebootMode,
    },
    SendInputText {
        serial: String,
        text: String,
    },
    SaveFullSnapshot {
        serial: String,
        use_root: bool,
        label: String,
    },
    ListSnapshots,
    LoadSnapshot {
        id: i64,
    },
    DeleteSnapshot {
        id: i64,
    },
    ExportFull {
        serial: String,
        use_root: bool,
        snapshot_override: Box<Option<FullDeviceSnapshot>>,
        format: FullExportFormat,
        out_path: PathBuf,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FullExportFormat {
    Json,
    Markdown,
}

#[derive(Debug)]
pub enum AppEvent {
    Input(crossterm::event::Event),
    Tick,
    LogcatBatch(Vec<LogEntry>),
    DevicesListed(Result<Vec<DeviceSummary>, String>),
    MdnsListed(Result<Vec<MdnsService>, String>),
    WirelessActionDone {
        message: String,
        connected_endpoint: Option<String>,
        refresh_devices: bool,
    },
    WirelessActionFailed(String),
    QrPairingStatus(String),
    RootChecked {
        serial: String,
        status: RootStatus,
    },
    IdentityLoaded {
        serial: String,
        result: Result<DeviceIdentity, String>,
    },
    TelemetryLoaded {
        serial: String,
        result: Result<TelemetrySnapshot, String>,
    },
    SystemAllLoaded {
        serial: String,
        props: Result<Vec<PropEntry>, String>,
        settings: Result<Vec<SettingEntry>, String>,
        sensors: Result<Vec<SensorEntry>, String>,
        dumpsys_list: Result<Vec<String>, String>,
    },
    DumpsysServiceLoaded {
        service: String,
        result: Result<String, String>,
    },
    ProcessesLoaded {
        serial: String,
        procs: Result<Vec<ProcessEntry>, String>,
        services: Result<Vec<ServiceEntry>, String>,
    },
    PackagesLoaded {
        serial: String,
        result: Result<Vec<PackageSummary>, String>,
    },
    PackageDetailLoaded {
        package: String,
        result: Result<PackageDetail, String>,
    },
    DirLoaded {
        path: String,
        result: Result<LsParseResult, String>,
    },
    FilePreviewLoaded {
        path: String,
        result: Result<String, String>,
    },
    PersonalAllLoaded {
        serial: String,
        contacts: Result<PersonalDataResult<ContactRecord>, String>,
        sms: Result<PersonalDataResult<SmsRecord>, String>,
        calls: Result<PersonalDataResult<CallLogRecord>, String>,
        accounts: Result<PersonalDataResult<AccountRecord>, String>,
    },
    ShellOutput {
        command: String,
        result: Result<String, String>,
    },
    ActionToast {
        ok: bool,
        message: String,
        refresh_after: bool,
    },
    SnapshotSaved(Result<(i64, String), String>),
    SnapshotsListed(Result<Vec<SnapshotMeta>, String>),
    SnapshotLoaded(Box<Result<(SnapshotMeta, FullDeviceSnapshot), String>>),
}
