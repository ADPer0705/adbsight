use crate::models::{LogEntry, LogLevel};

/// Parse a `logcat -v threadtime` line:
/// `11-24 16:54:12.345  1842  1901 I ActivityManager: Start proc ...`
pub fn parse_logcat_line(line: &str) -> Option<LogEntry> {
    let trimmed = line.trim_end_matches('\r');
    if trimmed.is_empty() || trimmed.starts_with("--------- beginning of") {
        return None;
    }

    let mut parts = trimmed.split_whitespace();
    let date = parts.next()?;
    let time = parts.next()?;
    // Basic check that first two tokens resemble MM-DD HH:MM:SS.mmm
    if !date.contains('-') || !time.contains(':') {
        return Some(LogEntry {
            timestamp: String::new(),
            pid: String::new(),
            tid: String::new(),
            level: LogLevel::Info,
            tag: "raw".to_string(),
            message: trimmed.to_string(),
        });
    }

    let pid = parts.next()?.to_string();
    let tid = parts.next()?.to_string();
    let level_tok = parts.next()?;
    let level = level_tok
        .chars()
        .next()
        .map(LogLevel::from_char)
        .unwrap_or(LogLevel::Info);

    // Find remainder after level token in the original string to preserve message spacing
    let level_byte_offset = find_token_after_prefix(trimmed, 5)?;
    let after_level = trimmed[level_byte_offset..].trim_start();

    let (tag, message) = if let Some((t, m)) = after_level.split_once(": ") {
        (t.trim().to_string(), m.to_string())
    } else if let Some((t, m)) = after_level.split_once(':') {
        (t.trim().to_string(), m.trim_start().to_string())
    } else {
        ("unknown".to_string(), after_level.to_string())
    };

    Some(LogEntry {
        timestamp: format!("{date} {time}"),
        pid,
        tid,
        level,
        tag,
        message,
    })
}

fn find_token_after_prefix(s: &str, token_count: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut idx = 0;
    let mut seen = 0;
    while idx < bytes.len() {
        while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
            idx += 1;
        }
        if idx >= bytes.len() {
            break;
        }
        while idx < bytes.len() && !bytes[idx].is_ascii_whitespace() {
            idx += 1;
        }
        seen += 1;
        if seen == token_count {
            return Some(idx);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_threadtime_logcat_lines() {
        let line = "11-24 16:54:12.345  1842  1901 W ActivityManager: Slow operation: 120ms so far";
        let entry = parse_logcat_line(line).unwrap();
        assert_eq!(entry.timestamp, "11-24 16:54:12.345");
        assert_eq!(entry.pid, "1842");
        assert_eq!(entry.tid, "1901");
        assert_eq!(entry.level, LogLevel::Warn);
        assert_eq!(entry.tag, "ActivityManager");
        assert_eq!(entry.message, "Slow operation: 120ms so far");
    }

    #[test]
    fn ignores_logcat_banner_lines() {
        assert!(parse_logcat_line("--------- beginning of main").is_none());
    }
}
