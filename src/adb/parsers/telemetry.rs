use std::collections::HashMap;

use chrono::Utc;

use crate::models::{
    BatteryInfo, CpuSample, NetworkInfo, StorageMount, TelemetrySnapshot, ThermalZone,
};

#[derive(Debug, Clone, Default)]
pub struct CpuRawSample {
    pub total: Option<(u64, u64)>, // (busy, total)
    pub cores: Vec<(u64, u64)>,
}

#[derive(Debug, Clone, Default)]
pub struct NetRawSample {
    pub timestamp_ms: i64,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

/// Split batched shell output marked by `===SECTION===` lines into sections.
pub fn split_sections(raw: &str) -> HashMap<&str, String> {
    let mut map: HashMap<&str, String> = HashMap::new();
    let mut current_key: Option<&str> = None;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("===") && trimmed.ends_with("===") && trimmed.len() > 6 {
            let key = &trimmed[3..trimmed.len() - 3];
            current_key = Some(key);
            map.entry(key).or_default();
        } else if let Some(k) = current_key {
            let buf = map.entry(k).or_default();
            buf.push_str(line);
            buf.push('\n');
        }
    }
    map
}

pub fn parse_telemetry_batch(
    raw: &str,
    prev_cpu: Option<&CpuRawSample>,
    prev_net: Option<&NetRawSample>,
) -> (TelemetrySnapshot, CpuRawSample, NetRawSample) {
    let sections = split_sections(raw);
    let now = Utc::now();

    let (cpu, cpu_raw) = parse_proc_stat(
        sections.get("STAT").map(String::as_str).unwrap_or(""),
        prev_cpu,
    );
    let mem = parse_meminfo(sections.get("MEM").map(String::as_str).unwrap_or(""));
    let battery = parse_battery(sections.get("BAT").map(String::as_str).unwrap_or(""));
    let thermals = parse_thermals(sections.get("THERM").map(String::as_str).unwrap_or(""));
    let storage = parse_df(sections.get("DF").map(String::as_str).unwrap_or(""));
    let (mut network, net_raw) = parse_net_dev(
        sections.get("NET").map(String::as_str).unwrap_or(""),
        now.timestamp_millis(),
        prev_net,
    );
    enrich_wifi_info(
        &mut network,
        sections.get("WIFI").map(String::as_str).unwrap_or(""),
    );
    let (uptime_secs, load_avg) =
        parse_uptime_load(sections.get("UPTIME").map(String::as_str).unwrap_or(""));

    let snap = TelemetrySnapshot {
        captured_at: now,
        cpu,
        mem_total_kb: mem.total_kb,
        mem_available_kb: mem.available_kb,
        mem_free_kb: mem.free_kb,
        mem_buffers_kb: mem.buffers_kb,
        mem_cached_kb: mem.cached_kb,
        swap_total_kb: mem.swap_total_kb,
        swap_free_kb: mem.swap_free_kb,
        battery,
        thermals,
        storage,
        network,
        uptime_secs,
        load_avg,
    };

    (snap, cpu_raw, net_raw)
}

fn parse_cpu_counters(line: &str) -> Option<(u64, u64)> {
    let mut parts = line.split_whitespace();
    let _label = parts.next()?;
    let nums: Vec<u64> = parts.filter_map(|s| s.parse::<u64>().ok()).collect();
    if nums.len() < 4 {
        return None;
    }
    let idle = nums[3] + nums.get(4).copied().unwrap_or(0); // idle + iowait
    let total: u64 = nums.iter().sum();
    let busy = total.saturating_sub(idle);
    Some((busy, total))
}

fn pct_delta(prev: (u64, u64), cur: (u64, u64)) -> f64 {
    let d_total = cur.1.saturating_sub(prev.1);
    let d_busy = cur.0.saturating_sub(prev.0);
    if d_total == 0 {
        0.0
    } else {
        ((d_busy as f64 / d_total as f64) * 100.0).clamp(0.0, 100.0)
    }
}

fn parse_proc_stat(text: &str, prev: Option<&CpuRawSample>) -> (CpuSample, CpuRawSample) {
    let mut raw = CpuRawSample::default();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("cpu ") {
            raw.total = parse_cpu_counters(trimmed);
        } else if trimmed.starts_with("cpu")
            && trimmed
                .as_bytes()
                .get(3)
                .is_some_and(|b| b.is_ascii_digit())
            && let Some(c) = parse_cpu_counters(trimmed)
        {
            raw.cores.push(c);
        }
    }

    let total_pct = match (prev.and_then(|p| p.total), raw.total) {
        (Some(p), Some(c)) => pct_delta(p, c),
        (None, Some((b, t))) if t > 0 => ((b as f64 / t as f64) * 100.0).clamp(0.0, 100.0),
        _ => 0.0,
    };

    let per_core_pct = raw
        .cores
        .iter()
        .enumerate()
        .map(|(i, &cur)| {
            if let Some(p) = prev.and_then(|pr| pr.cores.get(i).copied()) {
                pct_delta(p, cur)
            } else if cur.1 > 0 {
                ((cur.0 as f64 / cur.1 as f64) * 100.0).clamp(0.0, 100.0)
            } else {
                0.0
            }
        })
        .collect();

    (
        CpuSample {
            total_pct,
            per_core_pct,
        },
        raw,
    )
}

#[derive(Default)]
struct MemParsed {
    total_kb: u64,
    available_kb: u64,
    free_kb: u64,
    buffers_kb: u64,
    cached_kb: u64,
    swap_total_kb: u64,
    swap_free_kb: u64,
}

fn parse_meminfo(text: &str) -> MemParsed {
    let mut m = MemParsed::default();
    for line in text.lines() {
        let Some((k, rest)) = line.split_once(':') else {
            continue;
        };
        let val = rest
            .split_whitespace()
            .next()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);
        match k.trim() {
            "MemTotal" => m.total_kb = val,
            "MemAvailable" => m.available_kb = val,
            "MemFree" => m.free_kb = val,
            "Buffers" => m.buffers_kb = val,
            "Cached" => m.cached_kb = val,
            "SwapTotal" => m.swap_total_kb = val,
            "SwapFree" => m.swap_free_kb = val,
            _ => {}
        }
    }
    if m.available_kb == 0 && m.free_kb > 0 {
        m.available_kb = m.free_kb + m.buffers_kb + m.cached_kb;
    }
    m
}

fn parse_battery(text: &str) -> BatteryInfo {
    let mut b = BatteryInfo::default();
    let mut ac = false;
    let mut usb = false;
    let mut wireless = false;

    for line in text.lines() {
        let Some((k, v)) = line.split_once(':') else {
            continue;
        };
        let key = k.trim().to_ascii_lowercase();
        let val = v.trim();
        match key.as_str() {
            "level" => b.level = val.parse::<u8>().ok(),
            "status" => {
                b.status = match val {
                    "2" => "Charging".into(),
                    "3" => "Discharging".into(),
                    "4" => "Not charging".into(),
                    "5" => "Full".into(),
                    _ => val.to_string(),
                };
            }
            "health" => {
                b.health = match val {
                    "2" => "Good".into(),
                    "3" => "Overheat".into(),
                    "4" => "Dead".into(),
                    "5" => "Over voltage".into(),
                    "7" => "Cold".into(),
                    _ => val.to_string(),
                };
            }
            "ac powered" => ac = val == "true",
            "usb powered" => usb = val == "true",
            "wireless powered" => wireless = val == "true",
            "temperature" => {
                if let Ok(t) = val.parse::<f64>() {
                    b.temperature_c = Some(t / 10.0);
                }
            }
            "voltage" => b.voltage_mv = val.parse::<u32>().ok(),
            "current now" => b.current_ma = val.parse::<i32>().ok().map(|u| u / 1000),
            "technology" => b.technology = val.to_string(),
            _ => {}
        }
    }

    let mut sources = Vec::new();
    if ac {
        sources.push("AC");
    }
    if usb {
        sources.push("USB");
    }
    if wireless {
        sources.push("Wireless");
    }
    b.plugged = if sources.is_empty() {
        "Battery".to_string()
    } else {
        sources.join("+")
    };
    b
}

fn parse_thermals(text: &str) -> Vec<ThermalZone> {
    let mut zones = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Expected format from our probe: `zone_name:41200` (millidegrees C or degrees C)
        if let Some((name, raw_temp)) = trimmed.split_once(':')
            && let Ok(num) = raw_temp.trim().parse::<f64>()
        {
            let temp_c = if num > 200.0 { num / 1000.0 } else { num };
            if temp_c > -40.0 && temp_c < 150.0 {
                zones.push(ThermalZone {
                    name: name.trim().to_string(),
                    temp_c,
                });
            }
        }
    }
    zones
}

fn parse_df(text: &str) -> Vec<StorageMount> {
    let mut mounts = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("Filesystem") {
            continue;
        }
        let cols: Vec<&str> = trimmed.split_whitespace().collect();
        if cols.len() < 6 {
            continue;
        }
        let fs = cols[0];
        let mount = cols[5];
        if !matches!(
            mount,
            "/" | "/data" | "/sdcard" | "/storage/emulated" | "/system" | "/vendor" | "/cache"
        ) && !mount.starts_with("/storage/")
        {
            continue;
        }
        let size_kb = cols[1].parse::<u64>().unwrap_or(0);
        let used_kb = cols[2].parse::<u64>().unwrap_or(0);
        let avail_kb = cols[3].parse::<u64>().unwrap_or(0);
        let use_pct = cols[4].trim_end_matches('%').parse::<u8>().unwrap_or(0);
        if size_kb == 0 {
            continue;
        }
        mounts.push(StorageMount {
            filesystem: fs.to_string(),
            mount_point: mount.to_string(),
            size_kb,
            used_kb,
            avail_kb,
            use_pct,
        });
    }
    mounts
}

fn parse_net_dev(
    text: &str,
    now_ms: i64,
    prev: Option<&NetRawSample>,
) -> (NetworkInfo, NetRawSample) {
    let mut rx_total: u64 = 0;
    let mut tx_total: u64 = 0;

    for line in text.lines() {
        let Some((iface, rest)) = line.split_once(':') else {
            continue;
        };
        let iface = iface.trim();
        if iface == "lo" || iface.is_empty() {
            continue;
        }
        let nums: Vec<u64> = rest
            .split_whitespace()
            .filter_map(|s| s.parse::<u64>().ok())
            .collect();
        if nums.len() >= 9 {
            rx_total = rx_total.saturating_add(nums[0]);
            tx_total = tx_total.saturating_add(nums[8]);
        }
    }

    let (rx_rate, tx_rate) = if let Some(p) = prev {
        let dt_secs = ((now_ms - p.timestamp_ms).max(1) as f64) / 1000.0;
        let d_rx = rx_total.saturating_sub(p.rx_bytes) as f64;
        let d_tx = tx_total.saturating_sub(p.tx_bytes) as f64;
        (d_rx / dt_secs, d_tx / dt_secs)
    } else {
        (0.0, 0.0)
    };

    let info = NetworkInfo {
        rx_bytes_per_sec: rx_rate,
        tx_bytes_per_sec: tx_rate,
        total_rx_bytes: rx_total,
        total_tx_bytes: tx_total,
        ..Default::default()
    };

    let raw = NetRawSample {
        timestamp_ms: now_ms,
        rx_bytes: rx_total,
        tx_bytes: tx_total,
    };

    (info, raw)
}

fn enrich_wifi_info(net: &mut NetworkInfo, text: &str) {
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(val) = trimmed.strip_prefix("IP=") {
            let v = val.trim();
            if !v.is_empty() {
                net.wifi_ip = Some(v.to_string());
            }
        } else if let Some(val) = trimmed.strip_prefix("OP=") {
            let v = val.trim();
            if !v.is_empty() {
                net.operator = Some(v.to_string());
            }
        } else if let Some(val) = trimmed.strip_prefix("SSID=") {
            let v = val.trim().trim_matches('"');
            if !v.is_empty() && v != "<unknown ssid>" {
                net.wifi_ssid = Some(v.to_string());
            }
        } else if let Some(val) = trimmed.strip_prefix("RSSI=") {
            if let Ok(r) = val.trim().parse::<i32>() {
                net.wifi_rssi_dbm = Some(r);
            }
        } else if trimmed.contains("SSID:") && net.wifi_ssid.is_none() {
            // Fallback from `dumpsys wifi | grep mWifiInfo`
            for part in trimmed.split(',') {
                let p = part.trim();
                if let Some(s) = p.strip_prefix("SSID:") {
                    let clean = s.trim().trim_matches('"');
                    if !clean.is_empty() && clean != "<unknown ssid>" {
                        net.wifi_ssid = Some(clean.to_string());
                    }
                } else if let Some(r) = p.strip_prefix("RSSI:")
                    && let Ok(rssi) = r.trim().parse::<i32>()
                {
                    net.wifi_rssi_dbm = Some(rssi);
                }
            }
        }
    }
}

fn parse_uptime_load(text: &str) -> (Option<u64>, Option<String>) {
    let mut uptime_secs = None;
    let mut load_avg = None;
    for line in text.lines() {
        let t = line.trim();
        if let Some(u) = t.strip_prefix("UPTIME=") {
            uptime_secs = u
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<f64>().ok())
                .map(|v| v as u64);
        } else if let Some(l) = t.strip_prefix("LOAD=") {
            let parts: Vec<&str> = l.split_whitespace().take(3).collect();
            if parts.len() == 3 {
                load_avg = Some(parts.join(" "));
            }
        }
    }
    (uptime_secs, load_avg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_telemetry_and_computes_cpu_net_deltas() {
        let tick1 = "\
===STAT===
cpu  1000 100 500 8400 0 0 0 0 0 0
cpu0 500 50 250 4200 0 0 0 0 0 0
cpu1 500 50 250 4200 0 0 0 0 0 0
===MEM===
MemTotal:        8192000 kB
MemFree:         1024000 kB
MemAvailable:    4096000 kB
Buffers:          128000 kB
Cached:          2048000 kB
SwapTotal:       2048000 kB
SwapFree:        1500000 kB
===BAT===
  AC powered: false
  USB powered: true
  status: 2
  health: 2
  level: 84
  temperature: 315
  voltage: 4210
  technology: Li-ion
===THERM===
soc_therm:42300
skin:33.5
===DF===
Filesystem     1K-blocks    Used Available Use% Mounted on
/dev/block/dm-5 110000000 44000000 66000000 40% /data
===NET===
Inter-|   Receive                                                |  Transmit
 face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets
    lo: 1000 10 0 0 0 0 0 0 1000 10 0 0 0 0 0 0
 wlan0: 1000000 100 0 0 0 0 0 0 500000 80 0 0 0 0 0 0
===WIFI===
IP=192.168.1.77/24
OP=T-Mobile
SSID=\"HomeWiFi\"
RSSI=-54
===UPTIME===
UPTIME=3661.42 12000.00
LOAD=1.25 1.10 0.95 2/840 12345
";

        let (snap1, cpu1, mut net1) = parse_telemetry_batch(tick1, None, None);
        assert_eq!(snap1.mem_total_kb, 8_192_000);
        assert_eq!(snap1.mem_used_kb(), 4_096_000);
        assert_eq!(snap1.battery.level, Some(84));
        assert_eq!(snap1.battery.status, "Charging");
        assert_eq!(snap1.battery.plugged, "USB");
        assert_eq!(snap1.battery.temperature_c, Some(31.5));
        assert_eq!(snap1.thermals.len(), 2);
        assert_eq!(snap1.storage.len(), 1);
        assert_eq!(snap1.network.wifi_ssid.as_deref(), Some("HomeWiFi"));
        assert_eq!(snap1.network.wifi_rssi_dbm, Some(-54));
        assert_eq!(snap1.uptime_secs, Some(3661));
        assert_eq!(snap1.load_avg.as_deref(), Some("1.25 1.10 0.95"));

        // Simulate 2s later with +400 busy, +1000 total CPU ticks -> 40% CPU
        net1.timestamp_ms -= 2000;
        let tick2 = tick1
            .replace("cpu  1000 100 500 8400", "cpu  1300 100 600 9000")
            .replace(
                "wlan0: 1000000 100 0 0 0 0 0 0 500000",
                "wlan0: 1200000 120 0 0 0 0 0 0 600000",
            );
        let (snap2, _, _) = parse_telemetry_batch(&tick2, Some(&cpu1), Some(&net1));
        assert!((snap2.cpu.total_pct - 40.0).abs() < 0.01);
        assert!(snap2.network.rx_bytes_per_sec > 50_000.0);
    }
}
