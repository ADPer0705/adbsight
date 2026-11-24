use crate::models::{FsEntry, FsKind};

#[derive(Debug, Clone)]
pub struct LsParseResult {
    pub entries: Vec<FsEntry>,
    pub permission_denied: bool,
}

/// Parse `ls -la <dir>` from Android toybox/mksh.
/// Handles symlinks (`name -> target`), filenames with spaces, and permission denials.
pub fn parse_ls_la(dir_path: &str, output: &str) -> LsParseResult {
    let mut entries = Vec::new();
    let mut permission_denied = false;
    let base = if dir_path == "/" {
        String::new()
    } else {
        dir_path.trim_end_matches('/').to_string()
    };

    for line in output.lines() {
        let trimmed = line.trim_end_matches('\r');
        if trimmed.is_empty() || trimmed.starts_with("total ") {
            continue;
        }
        if trimmed.contains("Permission denied") {
            permission_denied = true;
            continue;
        }

        let first = trimmed.chars().next().unwrap_or(' ');
        if !matches!(first, 'd' | '-' | 'l' | 'c' | 'b' | 'p' | 's') {
            continue;
        }

        // Standard Android `ls -la` columns:
        // perms links owner group size YYYY-MM-DD HH:MM name [-> target]
        // Example:
        // drwxrwx--x 12 root sdcard_rw 4096 2026-11-20 14:32 Download
        // lrwxrwxrwx  1 root root        19 1970-01-01 00:00 sdcard -> /storage/self/primary
        let Some((cols, name_part)) = split_ls_columns(trimmed) else {
            continue;
        };

        let perms = cols[0].to_string();
        let owner = cols[2].to_string();
        let group = cols[3].to_string();
        let size = cols[4].parse::<u64>().unwrap_or(0);
        let modified = format!("{} {}", cols[5], cols[6]);

        let kind = match first {
            'd' => FsKind::Directory,
            'l' => FsKind::Symlink,
            '-' => FsKind::File,
            _ => FsKind::Other,
        };

        let (name, symlink_target) = if kind == FsKind::Symlink {
            if let Some((n, tgt)) = name_part.split_once(" -> ") {
                (n.to_string(), Some(tgt.to_string()))
            } else {
                (name_part.to_string(), None)
            }
        } else {
            (name_part.to_string(), None)
        };

        if name == "." || name == ".." || name.is_empty() {
            continue;
        }

        let full_path = format!("{base}/{name}");
        entries.push(FsEntry {
            name,
            path: full_path,
            kind,
            permissions: perms,
            owner,
            group,
            size,
            modified,
            symlink_target,
        });
    }

    // Directories/symlinks first, then alphabetical
    entries.sort_by(|a, b| {
        let rank = |k: FsKind| match k {
            FsKind::Directory => 0,
            FsKind::Symlink => 1,
            FsKind::File => 2,
            FsKind::Other => 3,
        };
        rank(a.kind)
            .cmp(&rank(b.kind))
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    LsParseResult {
        entries,
        permission_denied,
    }
}

fn split_ls_columns(line: &str) -> Option<(Vec<&str>, &str)> {
    let bytes = line.as_bytes();
    let mut cols = Vec::with_capacity(7);
    let mut idx = 0;

    for _ in 0..7 {
        while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
            idx += 1;
        }
        if idx >= bytes.len() {
            return None;
        }
        let start = idx;
        while idx < bytes.len() && !bytes[idx].is_ascii_whitespace() {
            idx += 1;
        }
        cols.push(&line[start..idx]);
    }

    while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
        idx += 1;
    }
    if idx >= bytes.len() {
        return None;
    }

    Some((cols, &line[idx..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_directories_files_spaces_and_symlinks() {
        let sample = "\
total 24
drwxrwx--x 12 root sdcard_rw 4096 2026-11-20 14:32 .
drwxr-xr-x  5 root root      4096 2026-11-20 14:32 ..
drwxrwx--x  4 root sdcard_rw 4096 2026-11-20 14:32 My Documents
-rw-rw----  1 u0_a210 sdcard_rw 1048576 2026-11-21 09:15 bugreport 2026.zip
lrwxrwxrwx  1 root root        21 1970-01-01 00:00 sdcard -> /storage/self/primary
";
        let res = parse_ls_la("/storage/emulated/0", sample);
        assert!(!res.permission_denied);
        assert_eq!(res.entries.len(), 3);
        assert_eq!(res.entries[0].name, "My Documents");
        assert_eq!(res.entries[0].kind, FsKind::Directory);
        assert_eq!(res.entries[1].name, "sdcard");
        assert_eq!(
            res.entries[1].symlink_target.as_deref(),
            Some("/storage/self/primary")
        );
        assert_eq!(res.entries[2].name, "bugreport 2026.zip");
        assert_eq!(res.entries[2].size, 1_048_576);
    }
}
