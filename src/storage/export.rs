use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use crate::adb::parsers::format_kb;
use crate::models::{
    AccountRecord, CallLogRecord, ContactRecord, FullDeviceSnapshot, LogEntry, PackageSummary,
    ProcessEntry, PropEntry, SmsRecord,
};

pub fn export_json<T: Serialize + ?Sized>(data: &T, path: &Path) -> Result<()> {
    ensure_parent(path)?;
    let pretty = serde_json::to_string_pretty(data)?;
    fs::write(path, pretty)?;
    Ok(())
}

pub fn export_props_csv(props: &[PropEntry], path: &Path) -> Result<()> {
    ensure_parent(path)?;
    let mut wtr = csv::Writer::from_path(path)?;
    wtr.write_record(["key", "value"])?;
    for p in props {
        wtr.write_record([&p.key, &p.value])?;
    }
    wtr.flush()?;
    Ok(())
}

pub fn export_packages_csv(packages: &[PackageSummary], path: &Path) -> Result<()> {
    ensure_parent(path)?;
    let mut wtr = csv::Writer::from_path(path)?;
    wtr.write_record(["name", "is_system", "enabled", "uid", "version_code", "apk_path"])?;
    for p in packages {
        wtr.write_record([
            p.name.as_str(),
            &p.is_system.to_string(),
            &p.enabled.to_string(),
            &p.uid.map(|u| u.to_string()).unwrap_or_default(),
            &p.version_code.map(|v| v.to_string()).unwrap_or_default(),
            p.apk_path.as_str(),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

pub fn export_processes_csv(procs: &[ProcessEntry], path: &Path) -> Result<()> {
    ensure_parent(path)?;
    let mut wtr = csv::Writer::from_path(path)?;
    wtr.write_record(["pid", "ppid", "user", "rss_kb", "vsz_kb", "state", "cpu_pct", "name"])?;
    for p in procs {
        wtr.write_record([
            &p.pid.to_string(),
            &p.ppid.to_string(),
            p.user.as_str(),
            &p.rss_kb.to_string(),
            &p.vsz_kb.to_string(),
            p.state.as_str(),
            &format!("{:.1}", p.cpu_pct),
            p.name.as_str(),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

pub fn export_logcat_csv(logs: &[LogEntry], path: &Path) -> Result<()> {
    ensure_parent(path)?;
    let mut wtr = csv::Writer::from_path(path)?;
    wtr.write_record(["timestamp", "pid", "tid", "level", "tag", "message"])?;
    for l in logs {
        wtr.write_record([
            l.timestamp.as_str(),
            l.pid.as_str(),
            l.tid.as_str(),
            &l.level.code().to_string(),
            l.tag.as_str(),
            l.message.as_str(),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

pub fn export_contacts_csv(contacts: &[ContactRecord], path: &Path) -> Result<()> {
    ensure_parent(path)?;
    let mut wtr = csv::Writer::from_path(path)?;
    wtr.write_record(["name", "phone_numbers", "emails"])?;
    for c in contacts {
        wtr.write_record([
            c.name.as_str(),
            &c.phone_numbers.join("; "),
            &c.emails.join("; "),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

pub fn export_sms_csv(sms: &[SmsRecord], path: &Path) -> Result<()> {
    ensure_parent(path)?;
    let mut wtr = csv::Writer::from_path(path)?;
    wtr.write_record(["address", "type", "date", "body"])?;
    for s in sms {
        wtr.write_record([
            s.address.as_str(),
            s.msg_type.as_str(),
            s.date_display.as_str(),
            s.body.as_str(),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

pub fn export_call_logs_csv(calls: &[CallLogRecord], path: &Path) -> Result<()> {
    ensure_parent(path)?;
    let mut wtr = csv::Writer::from_path(path)?;
    wtr.write_record(["number", "call_type", "date", "duration_secs"])?;
    for c in calls {
        wtr.write_record([
            c.number.as_str(),
            c.call_type.as_str(),
            c.date_display.as_str(),
            &c.duration_secs.to_string(),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

pub fn export_accounts_csv(accounts: &[AccountRecord], path: &Path) -> Result<()> {
    ensure_parent(path)?;
    let mut wtr = csv::Writer::from_path(path)?;
    wtr.write_record(["account_type", "name"])?;
    for a in accounts {
        wtr.write_record([a.account_type.as_str(), a.name.as_str()])?;
    }
    wtr.flush()?;
    Ok(())
}

pub fn render_markdown_report(snap: &FullDeviceSnapshot) -> String {
    let mut out = String::new();
    let id = &snap.identity;
    let tel = &snap.telemetry;

    let _ = writeln!(out, "# ADBSight Inspection Report — {} ({})", id.model, id.serial);
    let _ = writeln!(out, "\n- **Captured**: {}", snap.taken_at.to_rfc3339());
    let _ = writeln!(out, "- **Root Mode Used**: {}", snap.root_used);

    let _ = writeln!(out, "\n## Device Identity & Security Posture\n");
    let _ = writeln!(out, "| Field | Value |");
    let _ = writeln!(out, "|---|---|");
    for (k, v) in [
        ("Serial", id.serial.as_str()),
        ("Brand / Manufacturer", &format!("{} / {}", id.brand, id.manufacturer)),
        ("Model / Device", &format!("{} ({})", id.model, id.device)),
        ("Android / SDK", &format!("Android {} (API {})", id.android_version, id.sdk_int)),
        ("Security Patch", id.security_patch.as_str()),
        ("Build ID", id.build_id.as_str()),
        ("Kernel", id.kernel_version.as_str()),
        ("CPU ABI", id.cpu_abi.as_str()),
        ("Screen", &format!("{} @ {} dpi", id.screen_size, id.screen_density)),
        ("SELinux", id.selinux.as_str()),
        ("Verified Boot", id.verified_boot.as_str()),
    ] {
        let _ = writeln!(out, "| {k} | `{v}` |");
    }

    let _ = writeln!(out, "\n## Live Telemetry Summary\n");
    let _ = writeln!(out, "- **CPU**: {:.1}% ({} cores)", tel.cpu.total_pct, tel.cpu.per_core_pct.len());
    let _ = writeln!(
        out,
        "- **RAM**: {} / {} ({:.1}%)",
        format_kb(tel.mem_used_kb()),
        format_kb(tel.mem_total_kb),
        tel.mem_used_pct()
    );
    let _ = writeln!(
        out,
        "- **Battery**: {}% ({}, {}, {:.1}°C)",
        tel.battery.level.map(|l| l.to_string()).unwrap_or_else(|| "?".into()),
        tel.battery.status,
        tel.battery.health,
        tel.battery.temperature_c.unwrap_or(0.0)
    );
    if let Some(ip) = &tel.network.wifi_ip {
        let _ = writeln!(
            out,
            "- **Network**: Wi-Fi `{}` ({ip}, {} dBm)",
            tel.network.wifi_ssid.as_deref().unwrap_or("unknown"),
            tel.network.wifi_rssi_dbm.unwrap_or(0)
        );
    }

    if !tel.storage.is_empty() {
        let _ = writeln!(out, "\n### Storage Mounts\n");
        let _ = writeln!(out, "| Mount | Size | Used | Free | Use% |");
        let _ = writeln!(out, "|---|---|---|---|---|");
        for m in &tel.storage {
            let _ = writeln!(
                out,
                "| `{}` | {} | {} | {} | {}% |",
                m.mount_point,
                format_kb(m.size_kb),
                format_kb(m.used_kb),
                format_kb(m.avail_kb),
                m.use_pct
            );
        }
    }

    let user_pkgs: Vec<&PackageSummary> = snap.packages.iter().filter(|p| !p.is_system).collect();
    let _ = writeln!(
        out,
        "\n## Installed Packages ({} total, {} user apps)\n",
        snap.packages.len(),
        user_pkgs.len()
    );
    for pkg in user_pkgs.iter().take(40) {
        let _ = writeln!(
            out,
            "- `{}` (uid: {}, v{})",
            pkg.name,
            pkg.uid.map(|u| u.to_string()).unwrap_or_else(|| "?".into()),
            pkg.version_code.map(|v| v.to_string()).unwrap_or_else(|| "?".into())
        );
    }

    let _ = writeln!(out, "\n## Personal Data Summary\n");
    let _ = writeln!(out, "- **Contacts**: {}", snap.contacts.records.len());
    let _ = writeln!(out, "- **SMS Messages**: {}", snap.sms.records.len());
    let _ = writeln!(out, "- **Call Logs**: {}", snap.call_logs.records.len());
    let _ = writeln!(out, "- **Accounts**: {}", snap.accounts.records.len());
    for a in &snap.accounts.records {
        let _ = writeln!(out, "  - `{}` — {}", a.account_type, a.name);
    }

    out
}

pub fn export_markdown_report(snap: &FullDeviceSnapshot, path: &Path) -> Result<()> {
    ensure_parent(path)?;
    fs::write(path, render_markdown_report(snap))?;
    Ok(())
}

fn ensure_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adb::demo_backend::DemoAdb;
    use crate::adb::AdbClient;

    #[tokio::test]
    async fn exports_json_csv_and_markdown_reports() {
        let tmp_dir = std::env::temp_dir().join(format!("adbsight_exp_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp_dir);

        let client = AdbClient::Demo(DemoAdb::new());
        let snap = client
            .collect_full_snapshot("demo-pixel9pro-wifi", true)
            .await
            .unwrap();

        let json_path = tmp_dir.join("snap.json");
        let md_path = tmp_dir.join("report.md");
        let csv_path = tmp_dir.join("sms.csv");

        export_json(&snap, &json_path).unwrap();
        export_markdown_report(&snap, &md_path).unwrap();
        export_sms_csv(&snap.sms.records, &csv_path).unwrap();

        let json_body = fs::read_to_string(&json_path).unwrap();
        assert!(json_body.contains("Pixel 9 Pro"));
        let md_body = fs::read_to_string(&md_path).unwrap();
        assert!(md_body.contains("# ADBSight Inspection Report"));
        let csv_body = fs::read_to_string(&csv_path).unwrap();
        assert!(csv_body.starts_with("address,type,date,body"));

        let _ = fs::remove_dir_all(&tmp_dir);
    }
}
