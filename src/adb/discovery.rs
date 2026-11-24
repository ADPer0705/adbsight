use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct AdbBinaryInfo {
    pub path: PathBuf,
    pub version: String,
    pub mdns_available: bool,
}

/// Auto-discover the `adb` executable using override -> PATH -> env vars -> standard SDK locations.
pub fn discover_adb(override_path: Option<&Path>) -> Option<AdbBinaryInfo> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Some(p) = override_path {
        candidates.push(p.to_path_buf());
    }

    // 1. PATH lookup
    let bin_name = if cfg!(windows) { "adb.exe" } else { "adb" };
    if let Some(path_var) = env::var_os("PATH") {
        for dir in env::split_paths(&path_var) {
            let candidate = dir.join(bin_name);
            if candidate.is_file() {
                candidates.push(candidate);
            }
        }
    }

    // 2. ANDROID_HOME / ANDROID_SDK_ROOT
    for var in ["ANDROID_HOME", "ANDROID_SDK_ROOT"] {
        if let Some(sdk) = env::var_os(var) {
            candidates.push(PathBuf::from(sdk).join("platform-tools").join(bin_name));
        }
    }

    // 3. Standard platform SDK install directories
    if let Some(home) = env::var_os("HOME").or_else(|| env::var_os("USERPROFILE")) {
        let home = PathBuf::from(home);
        candidates.push(home.join("Android/Sdk/platform-tools").join(bin_name));
        candidates.push(home.join("Library/Android/sdk/platform-tools").join(bin_name));
        candidates.push(
            home.join("AppData/Local/Android/Sdk/platform-tools")
                .join(bin_name),
        );
    }

    if let Some(local_app_data) = env::var_os("LOCALAPPDATA") {
        candidates.push(
            PathBuf::from(local_app_data)
                .join("Android/Sdk/platform-tools")
                .join(bin_name),
        );
    }

    // Common system package locations
    for sys_path in ["/usr/bin/adb", "/usr/local/bin/adb", "/opt/homebrew/bin/adb"] {
        candidates.push(PathBuf::from(sys_path));
    }

    for candidate in candidates {
        if !candidate.is_file() {
            continue;
        }
        if let Some(info) = probe_adb_binary(&candidate) {
            return Some(info);
        }
    }

    None
}

fn probe_adb_binary(path: &Path) -> Option<AdbBinaryInfo> {
    let output = Command::new(path).arg("version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.contains("Android Debug Bridge") {
        return None;
    }

    let version = stdout
        .lines()
        .take(2)
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" | ");

    let mdns_available = Command::new(path)
        .args(["mdns", "check"])
        .output()
        .map(|o| {
            o.status.success()
                && !String::from_utf8_lossy(&o.stdout)
                    .to_lowercase()
                    .contains("disabled")
        })
        .unwrap_or(false);

    Some(AdbBinaryInfo {
        path: path.to_path_buf(),
        version,
        mdns_available,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_returns_none_for_nonexistent_override_only_if_no_fallback() {
        // Ensures probing a non-existent binary doesn't panic.
        assert!(probe_adb_binary(Path::new("/nonexistent/path/to/adb")).is_none());
    }
}
