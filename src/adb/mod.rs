pub mod cli_backend;
pub mod demo_backend;
pub mod discovery;
pub mod parsers;
pub mod wireless;

use std::path::Path;

use anyhow::Result;
use tokio::process::Child;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::adb::cli_backend::CliAdb;
use crate::adb::demo_backend::DemoAdb;
use crate::adb::parsers::filesystem::LsParseResult;
use crate::models::{
    AccountRecord, CallLogRecord, ContactRecord, DeviceIdentity, DeviceSummary, FullDeviceSnapshot,
    LogEntry, MdnsService, PackageDetail, PackageSummary, PersonalDataResult, ProcessEntry,
    PropEntry, RebootMode, RootStatus, SensorEntry, ServiceEntry, SettingEntry, SmsRecord,
    TelemetrySnapshot,
};

#[derive(Debug)]
pub enum LogcatHandle {
    Child(Child),
    DemoTask(JoinHandle<()>),
}

impl LogcatHandle {
    pub fn stop(self) {
        match self {
            Self::Child(mut child) => {
                let _ = child.start_kill();
            }
            Self::DemoTask(handle) => {
                handle.abort();
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum AdbClient {
    Cli(CliAdb),
    Demo(DemoAdb),
}

impl AdbClient {
    pub fn is_demo(&self) -> bool {
        matches!(self, Self::Demo(_))
    }

    pub async fn list_devices(&self) -> Result<Vec<DeviceSummary>> {
        match self {
            Self::Cli(c) => c.list_devices().await,
            Self::Demo(d) => d.list_devices().await,
        }
    }

    pub async fn list_mdns_services(&self) -> Result<Vec<MdnsService>> {
        match self {
            Self::Cli(c) => c.list_mdns_services().await,
            Self::Demo(d) => d.list_mdns_services().await,
        }
    }

    pub async fn pair(&self, endpoint: &str, code: &str) -> Result<String> {
        match self {
            Self::Cli(c) => c.pair(endpoint, code).await,
            Self::Demo(d) => d.pair(endpoint, code).await,
        }
    }

    pub async fn connect(&self, endpoint: &str) -> Result<String> {
        match self {
            Self::Cli(c) => c.connect(endpoint).await,
            Self::Demo(d) => d.connect(endpoint).await,
        }
    }

    pub async fn disconnect(&self, endpoint: &str) -> Result<String> {
        match self {
            Self::Cli(c) => c.disconnect(endpoint).await,
            Self::Demo(d) => d.disconnect(endpoint).await,
        }
    }

    pub async fn enable_tcpip_and_connect(&self, usb_serial: &str) -> Result<String> {
        match self {
            Self::Cli(c) => c.enable_tcpip_and_connect(usb_serial).await,
            Self::Demo(d) => d.enable_tcpip_and_connect(usb_serial).await,
        }
    }

    pub async fn check_root(&self, serial: &str) -> Result<RootStatus> {
        match self {
            Self::Cli(c) => c.check_root(serial).await,
            Self::Demo(d) => d.check_root(serial).await,
        }
    }

    pub async fn shell(&self, serial: &str, script: &str, use_root: bool) -> Result<String> {
        match self {
            Self::Cli(c) => c.shell(serial, script, use_root).await,
            Self::Demo(d) => d.shell(serial, script, use_root).await,
        }
    }

    pub async fn fetch_identity(&self, serial: &str) -> Result<DeviceIdentity> {
        match self {
            Self::Cli(c) => c.fetch_identity(serial).await,
            Self::Demo(d) => d.fetch_identity(serial).await,
        }
    }

    pub async fn fetch_telemetry(&self, serial: &str) -> Result<TelemetrySnapshot> {
        match self {
            Self::Cli(c) => c.fetch_telemetry(serial).await,
            Self::Demo(d) => d.fetch_telemetry(serial).await,
        }
    }

    pub async fn fetch_props(&self, serial: &str) -> Result<Vec<PropEntry>> {
        match self {
            Self::Cli(c) => c.fetch_props(serial).await,
            Self::Demo(d) => d.fetch_props(serial).await,
        }
    }

    pub async fn fetch_settings(&self, serial: &str) -> Result<Vec<SettingEntry>> {
        match self {
            Self::Cli(c) => c.fetch_settings(serial).await,
            Self::Demo(d) => d.fetch_settings(serial).await,
        }
    }

    pub async fn fetch_sensors(&self, serial: &str) -> Result<Vec<SensorEntry>> {
        match self {
            Self::Cli(c) => c.fetch_sensors(serial).await,
            Self::Demo(d) => d.fetch_sensors(serial).await,
        }
    }

    pub async fn fetch_dumpsys_list(&self, serial: &str) -> Result<Vec<String>> {
        match self {
            Self::Cli(c) => c.fetch_dumpsys_list(serial).await,
            Self::Demo(d) => d.fetch_dumpsys_list(serial).await,
        }
    }

    pub async fn fetch_dumpsys_service(
        &self,
        serial: &str,
        service: &str,
        use_root: bool,
    ) -> Result<String> {
        match self {
            Self::Cli(c) => c.fetch_dumpsys_service(serial, service, use_root).await,
            Self::Demo(d) => d.fetch_dumpsys_service(serial, service, use_root).await,
        }
    }

    pub async fn fetch_processes(&self, serial: &str, use_root: bool) -> Result<Vec<ProcessEntry>> {
        match self {
            Self::Cli(c) => c.fetch_processes(serial, use_root).await,
            Self::Demo(d) => d.fetch_processes(serial, use_root).await,
        }
    }

    pub async fn fetch_services(&self, serial: &str, use_root: bool) -> Result<Vec<ServiceEntry>> {
        match self {
            Self::Cli(c) => c.fetch_services(serial, use_root).await,
            Self::Demo(d) => d.fetch_services(serial, use_root).await,
        }
    }

    pub async fn fetch_packages(&self, serial: &str) -> Result<Vec<PackageSummary>> {
        match self {
            Self::Cli(c) => c.fetch_packages(serial).await,
            Self::Demo(d) => d.fetch_packages(serial).await,
        }
    }

    pub async fn fetch_package_detail(&self, serial: &str, package: &str) -> Result<PackageDetail> {
        match self {
            Self::Cli(c) => c.fetch_package_detail(serial, package).await,
            Self::Demo(d) => d.fetch_package_detail(serial, package).await,
        }
    }

    pub async fn spawn_logcat(
        &self,
        serial: &str,
        tx: mpsc::UnboundedSender<LogEntry>,
    ) -> Result<LogcatHandle> {
        match self {
            Self::Cli(c) => Ok(LogcatHandle::Child(c.spawn_logcat(serial, tx).await?)),
            Self::Demo(d) => Ok(LogcatHandle::DemoTask(d.spawn_demo_logcat(tx))),
        }
    }

    pub async fn list_dir(
        &self,
        serial: &str,
        path: &str,
        use_root: bool,
    ) -> Result<LsParseResult> {
        match self {
            Self::Cli(c) => c.list_dir(serial, path, use_root).await,
            Self::Demo(d) => d.list_dir(serial, path, use_root).await,
        }
    }

    pub async fn preview_file(&self, serial: &str, path: &str, use_root: bool) -> Result<String> {
        match self {
            Self::Cli(c) => c.preview_file(serial, path, use_root).await,
            Self::Demo(d) => d.preview_file(serial, path, use_root).await,
        }
    }

    pub async fn pull_path(&self, serial: &str, remote: &str, local: &Path) -> Result<String> {
        match self {
            Self::Cli(c) => c.pull_path(serial, remote, local).await,
            Self::Demo(d) => d.pull_path(serial, remote, local).await,
        }
    }

    pub async fn push_path(&self, serial: &str, local: &Path, remote: &str) -> Result<String> {
        match self {
            Self::Cli(c) => c.push_path(serial, local, remote).await,
            Self::Demo(d) => d.push_path(serial, local, remote).await,
        }
    }

    pub async fn fetch_contacts(
        &self,
        serial: &str,
        use_root: bool,
    ) -> Result<PersonalDataResult<ContactRecord>> {
        match self {
            Self::Cli(c) => c.fetch_contacts(serial, use_root).await,
            Self::Demo(d) => d.fetch_contacts(serial, use_root).await,
        }
    }

    pub async fn fetch_sms(
        &self,
        serial: &str,
        use_root: bool,
    ) -> Result<PersonalDataResult<SmsRecord>> {
        match self {
            Self::Cli(c) => c.fetch_sms(serial, use_root).await,
            Self::Demo(d) => d.fetch_sms(serial, use_root).await,
        }
    }

    pub async fn fetch_call_logs(
        &self,
        serial: &str,
        use_root: bool,
    ) -> Result<PersonalDataResult<CallLogRecord>> {
        match self {
            Self::Cli(c) => c.fetch_call_logs(serial, use_root).await,
            Self::Demo(d) => d.fetch_call_logs(serial, use_root).await,
        }
    }

    pub async fn fetch_accounts(
        &self,
        serial: &str,
        use_root: bool,
    ) -> Result<PersonalDataResult<AccountRecord>> {
        match self {
            Self::Cli(c) => c.fetch_accounts(serial, use_root).await,
            Self::Demo(d) => d.fetch_accounts(serial, use_root).await,
        }
    }

    pub async fn launch_app(&self, serial: &str, package: &str) -> Result<String> {
        match self {
            Self::Cli(c) => c.launch_app(serial, package).await,
            Self::Demo(d) => d.launch_app(serial, package).await,
        }
    }

    pub async fn force_stop(&self, serial: &str, package: &str) -> Result<String> {
        match self {
            Self::Cli(c) => c.force_stop(serial, package).await,
            Self::Demo(d) => d.force_stop(serial, package).await,
        }
    }

    pub async fn clear_app_data(&self, serial: &str, package: &str) -> Result<String> {
        match self {
            Self::Cli(c) => c.clear_app_data(serial, package).await,
            Self::Demo(d) => d.clear_app_data(serial, package).await,
        }
    }

    pub async fn pull_apk(&self, serial: &str, remote: &str, local: &Path) -> Result<String> {
        match self {
            Self::Cli(c) => c.pull_apk(serial, remote, local).await,
            Self::Demo(d) => d.pull_path(serial, remote, local).await,
        }
    }

    pub async fn kill_pid(&self, serial: &str, pid: u32, use_root: bool) -> Result<String> {
        match self {
            Self::Cli(c) => c.kill_pid(serial, pid, use_root).await,
            Self::Demo(d) => d.kill_pid(serial, pid).await,
        }
    }

    pub async fn screencap(&self, serial: &str, local_png: &Path) -> Result<()> {
        match self {
            Self::Cli(c) => c.screencap(serial, local_png).await,
            Self::Demo(d) => d.screencap(serial, local_png).await,
        }
    }

    pub async fn reboot(&self, serial: &str, mode: RebootMode) -> Result<String> {
        match self {
            Self::Cli(c) => c.reboot(serial, mode).await,
            Self::Demo(d) => d.reboot(serial, mode).await,
        }
    }

    pub async fn send_input_text(&self, serial: &str, text: &str) -> Result<String> {
        match self {
            Self::Cli(c) => c.send_input_text(serial, text).await,
            Self::Demo(d) => d.send_input_text(serial, text).await,
        }
    }

    pub async fn collect_full_snapshot(
        &self,
        serial: &str,
        use_root: bool,
    ) -> Result<FullDeviceSnapshot> {
        let (identity, telemetry, props, packages, processes, contacts, sms, call_logs, accounts) = tokio::join!(
            self.fetch_identity(serial),
            self.fetch_telemetry(serial),
            self.fetch_props(serial),
            self.fetch_packages(serial),
            self.fetch_processes(serial, use_root),
            self.fetch_contacts(serial, use_root),
            self.fetch_sms(serial, use_root),
            self.fetch_call_logs(serial, use_root),
            self.fetch_accounts(serial, use_root),
        );

        Ok(FullDeviceSnapshot {
            taken_at: chrono::Utc::now(),
            root_used: use_root,
            identity: identity?,
            telemetry: telemetry?,
            props: props.unwrap_or_default(),
            packages: packages.unwrap_or_default(),
            processes: processes.unwrap_or_default(),
            contacts: contacts.unwrap_or_default(),
            sms: sms.unwrap_or_default(),
            call_logs: call_logs.unwrap_or_default(),
            accounts: accounts.unwrap_or_default(),
        })
    }
}
