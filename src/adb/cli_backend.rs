use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, mpsc};
use tokio::time::timeout;

use crate::adb::parsers::devices::{parse_adb_devices_l, parse_mdns_services};
use crate::adb::parsers::filesystem::{LsParseResult, parse_ls_la};
use crate::adb::parsers::logcat::parse_logcat_line;
use crate::adb::parsers::packages::{parse_package_detail, parse_packages_list};
use crate::adb::parsers::personal::{parse_accounts, parse_call_logs, parse_contacts, parse_sms};
use crate::adb::parsers::processes::{parse_ps_output, parse_running_services};
use crate::adb::parsers::system::{
    parse_dumpsys_list, parse_getprop, parse_sensors, parse_settings, parse_wm_value,
};
use crate::adb::parsers::telemetry::{CpuRawSample, NetRawSample, parse_telemetry_batch};
use crate::models::{
    AccountRecord, CallLogRecord, ContactRecord, DeviceIdentity, DeviceSummary, LogEntry,
    MdnsService, PackageDetail, PackageSummary, PersonalDataResult, ProcessEntry, PropEntry,
    RebootMode, RootStatus, SensorEntry, ServiceEntry, SettingEntry, SettingNamespace, SmsRecord,
    TelemetrySnapshot,
};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(12);
const LONG_TIMEOUT: Duration = Duration::from_secs(45);

#[derive(Debug, Clone)]
pub struct CliAdb {
    adb_path: PathBuf,
    telemetry_prev: Arc<Mutex<HashMap<String, (CpuRawSample, NetRawSample)>>>,
}

impl CliAdb {
    pub fn new(adb_path: PathBuf) -> Self {
        Self {
            adb_path,
            telemetry_prev: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn adb_path(&self) -> &Path {
        &self.adb_path
    }

    async fn run_adb(&self, args: &[&str], limit: Duration) -> Result<String> {
        let mut cmd = Command::new(&self.adb_path);
        cmd.args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let fut = cmd.output();
        let output = timeout(limit, fut)
            .await
            .with_context(|| format!("adb {:?} timed out after {}s", args, limit.as_secs()))?
            .with_context(|| format!("failed to execute {:?}", self.adb_path))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() && stdout.trim().is_empty() {
            let msg = if stderr.trim().is_empty() {
                format!("adb exited with status {}", output.status)
            } else {
                stderr.trim().to_string()
            };
            bail!("{msg}");
        }

        if stdout.trim().is_empty() && !stderr.trim().is_empty() {
            Ok(stderr)
        } else if !stderr.trim().is_empty() {
            Ok(format!("{stdout}\n{stderr}"))
        } else {
            Ok(stdout)
        }
    }

    async fn run_device_adb(&self, serial: &str, args: &[&str], limit: Duration) -> Result<String> {
        let mut full_args = Vec::with_capacity(args.len() + 2);
        full_args.push("-s");
        full_args.push(serial);
        full_args.extend_from_slice(args);
        self.run_adb(&full_args, limit).await
    }

    pub async fn shell(&self, serial: &str, script: &str, use_root: bool) -> Result<String> {
        self.shell_with_timeout(serial, script, use_root, DEFAULT_TIMEOUT)
            .await
    }

    async fn shell_with_timeout(
        &self,
        serial: &str,
        script: &str,
        use_root: bool,
        limit: Duration,
    ) -> Result<String> {
        let wrapped = if use_root {
            let root_status = self.check_root(serial).await.unwrap_or(RootStatus::None);
            match root_status {
                RootStatus::AdbRoot => script.to_string(),
                RootStatus::SuAvailable => {
                    format!("su -c '{}'", escape_single_quotes(script))
                }
                RootStatus::None => script.to_string(),
            }
        } else {
            script.to_string()
        };

        self.run_device_adb(serial, &["shell", &wrapped], limit)
            .await
    }

    pub async fn list_devices(&self) -> Result<Vec<DeviceSummary>> {
        let out = self.run_adb(&["devices", "-l"], DEFAULT_TIMEOUT).await?;
        Ok(parse_adb_devices_l(&out))
    }

    pub async fn list_mdns_services(&self) -> Result<Vec<MdnsService>> {
        let out = self.run_adb(&["mdns", "services"], DEFAULT_TIMEOUT).await?;
        Ok(parse_mdns_services(&out))
    }

    pub async fn pair(&self, endpoint: &str, code: &str) -> Result<String> {
        let out = self
            .run_adb(&["pair", endpoint.trim(), code.trim()], Duration::from_secs(20))
            .await?;
        if out.to_ascii_lowercase().contains("failed") {
            bail!("{}", out.trim());
        }
        Ok(out.trim().to_string())
    }

    pub async fn connect(&self, endpoint: &str) -> Result<String> {
        let out = self
            .run_adb(&["connect", endpoint.trim()], DEFAULT_TIMEOUT)
            .await?;
        let lower = out.to_ascii_lowercase();
        if lower.contains("unable to connect")
            || lower.contains("failed to connect")
            || lower.contains("cannot connect")
        {
            bail!("{}", out.trim());
        }
        Ok(out.trim().to_string())
    }

    pub async fn disconnect(&self, endpoint: &str) -> Result<String> {
        let out = self
            .run_adb(&["disconnect", endpoint.trim()], DEFAULT_TIMEOUT)
            .await?;
        Ok(out.trim().to_string())
    }

    pub async fn enable_tcpip_and_connect(&self, usb_serial: &str) -> Result<String> {
        let ip_out = self
            .shell(
                usb_serial,
                "ip -4 addr show wlan0 | awk '/inet / {print $2}' | cut -d/ -f1 | head -n1",
                false,
            )
            .await?;
        let ip = ip_out
            .lines()
            .map(str::trim)
            .find(|l| !l.is_empty() && l.contains('.'))
            .context("could not detect device Wi-Fi IPv4 address on wlan0 (is Wi-Fi connected?)")?
            .to_string();

        let _ = self
            .run_device_adb(usb_serial, &["tcpip", "5555"], DEFAULT_TIMEOUT)
            .await?;
        tokio::time::sleep(Duration::from_millis(1400)).await;

        let endpoint = format!("{ip}:5555");
        let conn_msg = self.connect(&endpoint).await?;
        Ok(format!("{endpoint} ({conn_msg})"))
    }

    pub async fn check_root(&self, serial: &str) -> Result<RootStatus> {
        let out = self
            .run_device_adb(
                serial,
                &["shell", "id -u; which su 2>/dev/null || true"],
                Duration::from_secs(5),
            )
            .await?;
        let mut lines = out.lines().map(str::trim);
        if lines.next() == Some("0") {
            return Ok(RootStatus::AdbRoot);
        }
        if out.lines().any(|l| l.trim().ends_with("/su")) {
            return Ok(RootStatus::SuAvailable);
        }
        Ok(RootStatus::None)
    }

    pub async fn fetch_identity(&self, serial: &str) -> Result<DeviceIdentity> {
        let script = r#"
echo "MODEL=$(getprop ro.product.model)"
echo "BRAND=$(getprop ro.product.brand)"
echo "MFR=$(getprop ro.product.manufacturer)"
echo "DEVICE=$(getprop ro.product.device)"
echo "PRODUCT=$(getprop ro.product.name)"
echo "ANDROID=$(getprop ro.build.version.release)"
echo "SDK=$(getprop ro.build.version.sdk)"
echo "PATCH=$(getprop ro.build.version.security_patch)"
echo "BUILD=$(getprop ro.build.id)"
echo "FINGERPRINT=$(getprop ro.build.fingerprint)"
echo "ABI=$(getprop ro.product.cpu.abi)"
echo "SELINUX=$(getenforce 2>/dev/null || echo unknown)"
echo "VBOOT=$(getprop ro.boot.verifiedbootstate)"
echo "KERNEL=$(uname -r 2>/dev/null || cat /proc/version)"
echo "WMSIZE=$(wm size 2>/dev/null | tr '\n' ' ')"
echo "WMDENSITY=$(wm density 2>/dev/null | tr '\n' ' ')"
"#;
        let out = self.shell(serial, script, false).await?;
        let mut id = DeviceIdentity {
            serial: serial.to_string(),
            ..Default::default()
        };

        for line in out.lines() {
            if let Some((k, v)) = line.split_once('=') {
                let val = v.trim().to_string();
                match k.trim() {
                    "MODEL" => id.model = val,
                    "BRAND" => id.brand = val,
                    "MFR" => id.manufacturer = val,
                    "DEVICE" => id.device = val,
                    "PRODUCT" => id.product = val,
                    "ANDROID" => id.android_version = val,
                    "SDK" => id.sdk_int = val,
                    "PATCH" => id.security_patch = val,
                    "BUILD" => id.build_id = val,
                    "FINGERPRINT" => id.fingerprint = val,
                    "ABI" => id.cpu_abi = val,
                    "SELINUX" => id.selinux = val,
                    "VBOOT" => id.verified_boot = val,
                    "KERNEL" => id.kernel_version = val,
                    "WMSIZE" => id.screen_size = parse_wm_value(&val),
                    "WMDENSITY" => id.screen_density = parse_wm_value(&val),
                    _ => {}
                }
            }
        }
        Ok(id)
    }

    pub async fn fetch_telemetry(&self, serial: &str) -> Result<TelemetrySnapshot> {
        let script = r#"
echo "===STAT==="
cat /proc/stat 2>/dev/null | grep '^cpu'
echo "===MEM==="
cat /proc/meminfo 2>/dev/null
echo "===BAT==="
dumpsys battery 2>/dev/null
echo "===THERM==="
for z in /sys/class/thermal/thermal_zone*; do
  [ -r "$z/type" ] && [ -r "$z/temp" ] && echo "$(cat "$z/type"):$(cat "$z/temp")"
done | head -n 10
echo "===DF==="
df -k 2>/dev/null
echo "===NET==="
cat /proc/net/dev 2>/dev/null
echo "===WIFI==="
echo "IP=$(ip -4 addr show wlan0 2>/dev/null | awk '/inet / {print $2}' | head -n1)"
echo "OP=$(getprop gsm.operator.alpha)"
dumpsys wifi 2>/dev/null | grep -m1 'mWifiInfo'
echo "===UPTIME==="
echo "UPTIME=$(cat /proc/uptime 2>/dev/null)"
echo "LOAD=$(cat /proc/loadavg 2>/dev/null)"
"#;
        let out = self.shell(serial, script, false).await?;
        let mut guard = self.telemetry_prev.lock().await;
        let prev = guard.get(serial).cloned();
        let (snap, cpu_raw, net_raw) = parse_telemetry_batch(
            &out,
            prev.as_ref().map(|(c, _)| c),
            prev.as_ref().map(|(_, n)| n),
        );
        guard.insert(serial.to_string(), (cpu_raw, net_raw));
        Ok(snap)
    }

    pub async fn fetch_props(&self, serial: &str) -> Result<Vec<PropEntry>> {
        let out = self.shell(serial, "getprop", false).await?;
        Ok(parse_getprop(&out))
    }

    pub async fn fetch_settings(&self, serial: &str) -> Result<Vec<SettingEntry>> {
        let script = r#"
echo "===SYS==="
settings list system 2>/dev/null
echo "===SEC==="
settings list secure 2>/dev/null
echo "===GLB==="
settings list global 2>/dev/null
"#;
        let out = self.shell(serial, script, false).await?;
        let sections = crate::adb::parsers::telemetry::split_sections(&out);
        let mut all = Vec::new();
        all.extend(parse_settings(
            sections.get("SYS").map(String::as_str).unwrap_or(""),
            SettingNamespace::System,
        ));
        all.extend(parse_settings(
            sections.get("SEC").map(String::as_str).unwrap_or(""),
            SettingNamespace::Secure,
        ));
        all.extend(parse_settings(
            sections.get("GLB").map(String::as_str).unwrap_or(""),
            SettingNamespace::Global,
        ));
        Ok(all)
    }

    pub async fn fetch_sensors(&self, serial: &str) -> Result<Vec<SensorEntry>> {
        let out = self
            .shell_with_timeout(serial, "dumpsys sensorservice", false, LONG_TIMEOUT)
            .await?;
        Ok(parse_sensors(&out))
    }

    pub async fn fetch_dumpsys_list(&self, serial: &str) -> Result<Vec<String>> {
        let out = self.shell(serial, "dumpsys -l", false).await?;
        Ok(parse_dumpsys_list(&out))
    }

    pub async fn fetch_dumpsys_service(
        &self,
        serial: &str,
        service: &str,
        use_root: bool,
    ) -> Result<String> {
        let cmd = format!("dumpsys {}", shell_quote(service));
        self.shell_with_timeout(serial, &cmd, use_root, LONG_TIMEOUT)
            .await
    }

    pub async fn fetch_processes(&self, serial: &str, use_root: bool) -> Result<Vec<ProcessEntry>> {
        let out = self
            .shell(
                serial,
                "ps -A -o PID,PPID,USER,RSS,VSZ,S,PCPU,NAME 2>/dev/null || ps -A",
                use_root,
            )
            .await?;
        Ok(parse_ps_output(&out))
    }

    pub async fn fetch_services(&self, serial: &str, use_root: bool) -> Result<Vec<ServiceEntry>> {
        let out = self
            .shell_with_timeout(serial, "dumpsys activity services", use_root, LONG_TIMEOUT)
            .await?;
        Ok(parse_running_services(&out))
    }

    pub async fn fetch_packages(&self, serial: &str) -> Result<Vec<PackageSummary>> {
        let script = r#"
echo "===ALL==="
pm list packages -f -U --show-versioncode 2>/dev/null || pm list packages -f 2>/dev/null
echo "===SYS==="
pm list packages -s 2>/dev/null
echo "===DIS==="
pm list packages -d 2>/dev/null
"#;
        let out = self
            .shell_with_timeout(serial, script, false, LONG_TIMEOUT)
            .await?;
        let sections = crate::adb::parsers::telemetry::split_sections(&out);
        Ok(parse_packages_list(
            sections.get("ALL").map(String::as_str).unwrap_or(""),
            sections.get("SYS").map(String::as_str).unwrap_or(""),
            sections.get("DIS").map(String::as_str).unwrap_or(""),
        ))
    }

    pub async fn fetch_package_detail(
        &self,
        serial: &str,
        package: &str,
    ) -> Result<PackageDetail> {
        let cmd = format!("dumpsys package {}", shell_quote(package));
        let out = self
            .shell_with_timeout(serial, &cmd, false, LONG_TIMEOUT)
            .await?;
        Ok(parse_package_detail(package, &out))
    }

    pub async fn spawn_logcat(
        &self,
        serial: &str,
        tx: mpsc::UnboundedSender<LogEntry>,
    ) -> Result<Child> {
        let mut cmd = Command::new(&self.adb_path);
        cmd.args(["-s", serial, "logcat", "-v", "threadtime", "-T", "200"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);

        let mut child = cmd.spawn().context("failed to spawn adb logcat")?;
        if let Some(stdout) = child.stdout.take() {
            tokio::spawn(async move {
                let mut lines = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    if let Some(entry) = parse_logcat_line(&line)
                        && tx.send(entry).is_err()
                    {
                        break;
                    }
                }
            });
        }
        Ok(child)
    }

    pub async fn list_dir(&self, serial: &str, path: &str, use_root: bool) -> Result<LsParseResult> {
        let cmd = format!("ls -la {}", shell_quote(path));
        let out = self.shell(serial, &cmd, use_root).await?;
        Ok(parse_ls_la(path, &out))
    }

    pub async fn preview_file(&self, serial: &str, path: &str, use_root: bool) -> Result<String> {
        let cmd = format!("head -c 32768 {}", shell_quote(path));
        self.shell(serial, &cmd, use_root).await
    }

    pub async fn pull_path(&self, serial: &str, remote: &str, local: &Path) -> Result<String> {
        let local_str = local.to_string_lossy().to_string();
        let out = self
            .run_device_adb(serial, &["pull", remote, &local_str], LONG_TIMEOUT)
            .await?;
        Ok(out.trim().to_string())
    }

    pub async fn push_path(&self, serial: &str, local: &Path, remote: &str) -> Result<String> {
        let local_str = local.to_string_lossy().to_string();
        let out = self
            .run_device_adb(serial, &["push", &local_str, remote], LONG_TIMEOUT)
            .await?;
        Ok(out.trim().to_string())
    }

    pub async fn fetch_contacts(
        &self,
        serial: &str,
        use_root: bool,
    ) -> Result<PersonalDataResult<ContactRecord>> {
        let out = self
            .shell(
                serial,
                "content query --uri content://com.android.contacts/data --projection display_name:data1:mimetype",
                use_root,
            )
            .await?;
        Ok(parse_contacts(&out))
    }

    pub async fn fetch_sms(
        &self,
        serial: &str,
        use_root: bool,
    ) -> Result<PersonalDataResult<SmsRecord>> {
        let out = self
            .shell(
                serial,
                "content query --uri content://sms --projection address:body:date:type",
                use_root,
            )
            .await?;
        Ok(parse_sms(&out))
    }

    pub async fn fetch_call_logs(
        &self,
        serial: &str,
        use_root: bool,
    ) -> Result<PersonalDataResult<CallLogRecord>> {
        let out = self
            .shell(
                serial,
                "content query --uri content://call_log/calls --projection number:type:date:duration",
                use_root,
            )
            .await?;
        Ok(parse_call_logs(&out))
    }

    pub async fn fetch_accounts(
        &self,
        serial: &str,
        use_root: bool,
    ) -> Result<PersonalDataResult<AccountRecord>> {
        let out = self
            .shell(serial, "dumpsys account", use_root)
            .await?;
        Ok(parse_accounts(&out))
    }

    pub async fn launch_app(&self, serial: &str, package: &str) -> Result<String> {
        let cmd = format!(
            "monkey -p {} -c android.intent.category.LAUNCHER 1",
            shell_quote(package)
        );
        self.shell(serial, &cmd, false).await
    }

    pub async fn force_stop(&self, serial: &str, package: &str) -> Result<String> {
        let cmd = format!("am force-stop {}", shell_quote(package));
        self.shell(serial, &cmd, false).await
    }

    pub async fn clear_app_data(&self, serial: &str, package: &str) -> Result<String> {
        let cmd = format!("pm clear {}", shell_quote(package));
        self.shell(serial, &cmd, false).await
    }

    pub async fn pull_apk(
        &self,
        serial: &str,
        apk_remote: &str,
        local_dest: &Path,
    ) -> Result<String> {
        self.pull_path(serial, apk_remote, local_dest).await
    }

    pub async fn kill_pid(&self, serial: &str, pid: u32, use_root: bool) -> Result<String> {
        let cmd = format!("kill -9 {pid}");
        self.shell(serial, &cmd, use_root).await
    }

    pub async fn screencap(&self, serial: &str, local_png: &Path) -> Result<()> {
        let mut cmd = Command::new(&self.adb_path);
        cmd.args(["-s", serial, "exec-out", "screencap", "-p"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = timeout(LONG_TIMEOUT, cmd.output())
            .await
            .context("screencap timed out")?
            .context("failed to run screencap")?;

        if !output.status.success() || output.stdout.is_empty() {
            let err = String::from_utf8_lossy(&output.stderr);
            bail!("screencap failed: {}", err.trim());
        }

        if let Some(parent) = local_dest_parent(local_png) {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(local_png, &output.stdout).await?;
        Ok(())
    }

    pub async fn reboot(&self, serial: &str, mode: RebootMode) -> Result<String> {
        let mut args = vec!["reboot"];
        if let Some(m) = mode.adb_arg() {
            args.push(m);
        }
        self.run_device_adb(serial, &args, DEFAULT_TIMEOUT).await
    }

    pub async fn send_input_text(&self, serial: &str, text: &str) -> Result<String> {
        let escaped = text.replace(' ', "%s");
        let cmd = format!("input text {}", shell_quote(&escaped));
        self.shell(serial, &cmd, false).await
    }
}

fn local_dest_parent(p: &Path) -> Option<&Path> {
    p.parent().filter(|d| !d.as_os_str().is_empty())
}

fn escape_single_quotes(s: &str) -> String {
    s.replace('\'', "'\"'\"'")
}

fn shell_quote(s: &str) -> String {
    format!("'{}'", escape_single_quotes(s))
}
