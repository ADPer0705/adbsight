use crate::models::{ProcessEntry, ServiceEntry};

/// Parse `ps -A -o PID,PPID,USER,RSS,VSZ,S,PCPU,NAME` (or fallback `ps -A` / `ps -ef`).
pub fn parse_ps_output(output: &str) -> Vec<ProcessEntry> {
    let mut procs = Vec::new();
    let mut header_seen = false;
    let mut has_pcpu = false;

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !header_seen && (trimmed.starts_with("PID") || trimmed.starts_with("USER")) {
            header_seen = true;
            has_pcpu = trimmed.contains("%CPU") || trimmed.contains("PCPU");
            continue;
        }

        let cols: Vec<&str> = trimmed.split_whitespace().collect();
        if has_pcpu && cols.len() >= 8 {
            if let Ok(pid) = cols[0].parse::<u32>() {
                let ppid = cols[1].parse::<u32>().unwrap_or(0);
                let user = cols[2].to_string();
                let rss_kb = parse_mem_col(cols[3]);
                let vsz_kb = parse_mem_col(cols[4]);
                let state = cols[5].to_string();
                let cpu_pct = cols[6].parse::<f64>().unwrap_or(0.0);
                let name = cols[7..].join(" ");
                procs.push(ProcessEntry {
                    pid,
                    ppid,
                    user,
                    rss_kb,
                    vsz_kb,
                    state,
                    cpu_pct,
                    name,
                });
            }
        } else if cols.len() >= 9 {
            // Standard toybox `ps -A`: USER PID PPID VSZ RSS WCHAN ADDR S NAME
            if let Ok(pid) = cols[1].parse::<u32>() {
                procs.push(ProcessEntry {
                    pid,
                    ppid: cols[2].parse::<u32>().unwrap_or(0),
                    user: cols[0].to_string(),
                    vsz_kb: parse_mem_col(cols[3]),
                    rss_kb: parse_mem_col(cols[4]),
                    state: cols[7].to_string(),
                    cpu_pct: 0.0,
                    name: cols[8..].join(" "),
                });
            }
        }
    }
    procs
}

fn parse_mem_col(raw: &str) -> u64 {
    if let Some(m) = raw.strip_suffix('M').or_else(|| raw.strip_suffix('m')) {
        (m.parse::<f64>().unwrap_or(0.0) * 1024.0) as u64
    } else if let Some(k) = raw.strip_suffix('K').or_else(|| raw.strip_suffix('k')) {
        k.parse::<u64>().unwrap_or(0)
    } else if let Some(g) = raw.strip_suffix('G').or_else(|| raw.strip_suffix('g')) {
        (g.parse::<f64>().unwrap_or(0.0) * 1024.0 * 1024.0) as u64
    } else {
        raw.parse::<u64>().unwrap_or(0)
    }
}

/// Parse `dumpsys activity services` for active service records.
pub fn parse_running_services(output: &str) -> Vec<ServiceEntry> {
    let mut services = Vec::new();
    let mut cur_pkg = String::new();
    let mut cur_class = String::new();
    let mut cur_uid = None;
    let mut cur_pid = None;

    for line in output.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("* ServiceRecord{") {
            if !cur_pkg.is_empty() {
                services.push(ServiceEntry {
                    package: std::mem::take(&mut cur_pkg),
                    service_class: std::mem::take(&mut cur_class),
                    pid: cur_pid.take(),
                    uid: cur_uid.take(),
                    state: "Running".into(),
                });
            }
            // Example: `* ServiceRecord{abc u0 com.whatsapp/.messaging.MessageService}`
            let inner = rest.trim_end_matches('}');
            if let Some(comp) = inner.split_whitespace().last()
                && let Some((pkg, cls)) = comp.split_once('/')
            {
                cur_pkg = pkg.to_string();
                cur_class = cls.to_string();
            }
        } else if trimmed.starts_with("uid=") {
            // e.g. `uid=10245 packageName=com.whatsapp processName=com.whatsapp`
            for tok in trimmed.split_whitespace() {
                if let Some(u) = tok.strip_prefix("uid=") {
                    cur_uid = u.parse::<u32>().ok();
                }
            }
        } else if trimmed.starts_with("app=ProcessRecord{")
            && let Some((before_colon, _)) = trimmed.split_once(':')
            && let Some(pid_str) = before_colon.split_whitespace().last()
        {
            cur_pid = pid_str.trim().parse::<u32>().ok();
        }
    }

    if !cur_pkg.is_empty() {
        services.push(ServiceEntry {
            package: cur_pkg,
            service_class: cur_class,
            pid: cur_pid,
            uid: cur_uid,
            state: "Running".into(),
        });
    }

    services
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ps_with_pcpu_and_fallback() {
        let sample = "\
  PID  PPID USER            RSS    VSZ S %CPU NAME
    1     0 root           8420  24100 S  0.1 init
 1842   712 system       192400 1850200 S 12.4 system_server
 4210   712 u0_a210      128500 1520100 R  3.8 com.example.app
";
        let procs = parse_ps_output(sample);
        assert_eq!(procs.len(), 3);
        assert_eq!(procs[1].pid, 1842);
        assert_eq!(procs[1].rss_kb, 192400);
        assert!((procs[1].cpu_pct - 12.4).abs() < 0.01);
        assert_eq!(procs[2].name, "com.example.app");
    }

    #[test]
    fn parses_running_services_records() {
        let sample = "\
  * ServiceRecord{a1b2c3 u0 com.google.android.gms/.gcm.GcmService}
    uid=10123 packageName=com.google.android.gms
    app=ProcessRecord{def456 2450:com.google.android.gms.persistent/u0a123}
";
        let svcs = parse_running_services(sample);
        assert_eq!(svcs.len(), 1);
        assert_eq!(svcs[0].package, "com.google.android.gms");
        assert_eq!(svcs[0].pid, Some(2450));
        assert_eq!(svcs[0].uid, Some(10123));
    }
}
