pub mod devices;
pub mod filesystem;
pub mod logcat;
pub mod packages;
pub mod personal;
pub mod processes;
pub mod system;
pub mod telemetry;

use std::collections::HashMap;

/// Stateful parser for `adb shell content query` output (`Row: N key=val, key2=val2`).
/// Handles commas and embedded newlines inside values (e.g., SMS bodies) by scanning
/// for known `, <key>=` boundaries and `Row: <N> ` row starts.
pub fn parse_content_query_rows(output: &str, known_keys: &[&str]) -> Vec<HashMap<String, String>> {
    let mut raw_rows: Vec<String> = Vec::new();
    let mut current: Option<String> = None;

    for line in output.lines() {
        let trimmed = line.trim_end_matches('\r');
        if let Some(rest) = strip_row_prefix(trimmed) {
            if let Some(prev) = current.take() {
                raw_rows.push(prev);
            }
            current = Some(rest.to_string());
        } else if let Some(ref mut cur) = current {
            // Continuation line belonging to the last field (e.g., multiline SMS body)
            if !trimmed.is_empty() {
                cur.push('\n');
                cur.push_str(trimmed);
            }
        }
    }
    if let Some(prev) = current.take() {
        raw_rows.push(prev);
    }

    raw_rows
        .into_iter()
        .map(|row_str| parse_single_content_row(&row_str, known_keys))
        .collect()
}

fn strip_row_prefix(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("Row: ")?;
    let space_idx = rest.find(' ')?;
    let num_part = &rest[..space_idx];
    if !num_part.is_empty() && num_part.chars().all(|c| c.is_ascii_digit()) {
        Some(&rest[space_idx + 1..])
    } else {
        None
    }
}

fn parse_single_content_row(row: &str, known_keys: &[&str]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut boundaries: Vec<(usize, &str)> = Vec::new();

    // First key at position 0
    for &key in known_keys {
        if row.starts_with(key) && row[key.len()..].starts_with('=') {
            boundaries.push((0, key));
            break;
        }
    }

    // Subsequent keys preceded by ", "
    let bytes = row.as_bytes();
    let mut i = 0;
    while i + 2 < bytes.len() {
        if bytes[i] == b',' && bytes[i + 1] == b' ' {
            let after = &row[i + 2..];
            for &key in known_keys {
                if after.starts_with(key) && after[key.len()..].starts_with('=') {
                    boundaries.push((i + 2, key));
                    break;
                }
            }
        }
        i += 1;
    }

    boundaries.sort_by_key(|(pos, _)| *pos);
    boundaries.dedup_by_key(|(pos, _)| *pos);

    for idx in 0..boundaries.len() {
        let (start_pos, key) = boundaries[idx];
        let val_start = start_pos + key.len() + 1; // skip `key=`
        let val_end = if idx + 1 < boundaries.len() {
            // exclude the `, ` before the next key
            boundaries[idx + 1].0.saturating_sub(2)
        } else {
            row.len()
        };
        if val_start <= val_end && val_end <= row.len() {
            let val = row[val_start..val_end].trim().to_string();
            if val != "NULL" {
                map.insert(key.to_string(), val);
            }
        }
    }

    map
}

pub fn format_kb(kb: u64) -> String {
    if kb >= 1024 * 1024 {
        format!("{:.2} GiB", kb as f64 / (1024.0 * 1024.0))
    } else if kb >= 1024 {
        format!("{:.1} MiB", kb as f64 / 1024.0)
    } else {
        format!("{kb} KiB")
    }
}

pub fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.2} GiB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MiB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KiB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes} B")
    }
}
