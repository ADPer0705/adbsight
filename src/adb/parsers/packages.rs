use std::collections::HashSet;

use crate::models::{PackageDetail, PackageSummary};

/// Parse combined package listings:
/// - `all_output`: `pm list packages -f -U --show-versioncode` (or `-f -U` fallback)
/// - `system_names`: `pm list packages -s`
/// - `disabled_names`: `pm list packages -d`
pub fn parse_packages_list(
    all_output: &str,
    system_output: &str,
    disabled_output: &str,
) -> Vec<PackageSummary> {
    let system_set = parse_simple_package_set(system_output);
    let disabled_set = parse_simple_package_set(disabled_output);

    let mut packages = Vec::new();
    for line in all_output.lines() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix("package:") else {
            continue;
        };

        // Typical format:
        // `/data/app/~~abc/com.example-xyz/base.apk=com.example versionCode:123 uid:10245`
        let tokens: Vec<&str> = rest.split_whitespace().collect();
        let Some(path_and_pkg) = tokens.first() else {
            continue;
        };

        let (apk_path, name) = if let Some((path, pkg)) = path_and_pkg.rsplit_once('=') {
            (path.to_string(), pkg.to_string())
        } else {
            (String::new(), (*path_and_pkg).to_string())
        };

        let mut uid = None;
        let mut version_code = None;
        for tok in tokens.iter().skip(1) {
            if let Some(v) = tok.strip_prefix("uid:") {
                uid = v.parse::<u32>().ok();
            } else if let Some(v) = tok.strip_prefix("versionCode:") {
                version_code = v.parse::<i64>().ok();
            }
        }

        packages.push(PackageSummary {
            is_system: system_set.contains(&name),
            enabled: !disabled_set.contains(&name),
            name,
            apk_path,
            uid,
            version_code,
        });
    }

    packages.sort_by(|a, b| a.name.cmp(&b.name));
    packages
}

fn parse_simple_package_set(output: &str) -> HashSet<String> {
    output
        .lines()
        .filter_map(|l| l.trim().strip_prefix("package:"))
        .map(|s| {
            s.split_whitespace()
                .next()
                .unwrap_or("")
                .rsplit('=')
                .next()
                .unwrap_or("")
                .to_string()
        })
        .filter(|s| !s.is_empty())
        .collect()
}

/// Parse `dumpsys package <pkg>` output into structured metadata + permissions.
pub fn parse_package_detail(pkg_name: &str, output: &str) -> PackageDetail {
    let mut detail = PackageDetail {
        name: pkg_name.to_string(),
        ..Default::default()
    };

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum PermSection {
        None,
        Granted,
        Runtime,
    }
    let mut section = PermSection::None;

    for line in output.lines() {
        let trimmed = line.trim();
        if let Some(v) = trimmed.strip_prefix("versionName=") {
            detail.version_name = v.to_string();
        } else if let Some(v) = trimmed.strip_prefix("versionCode=") {
            for tok in v.split_whitespace() {
                if detail.version_code.is_empty() {
                    detail.version_code = tok.to_string();
                } else if let Some(min) = tok.strip_prefix("minSdk=") {
                    detail.min_sdk = min.to_string();
                } else if let Some(tgt) = tok.strip_prefix("targetSdk=") {
                    detail.target_sdk = tgt.to_string();
                }
            }
        } else if let Some(v) = trimmed.strip_prefix("userId=") {
            detail.uid = v.to_string();
        } else if let Some(v) = trimmed.strip_prefix("appId=") {
            if detail.uid.is_empty() {
                detail.uid = v.to_string();
            }
        } else if let Some(v) = trimmed.strip_prefix("codePath=") {
            detail.apk_path = v.to_string();
        } else if let Some(v) = trimmed.strip_prefix("dataDir=") {
            detail.data_dir = v.to_string();
        } else if let Some(v) = trimmed.strip_prefix("firstInstallTime=") {
            detail.first_install = v.to_string();
        } else if let Some(v) = trimmed.strip_prefix("lastUpdateTime=") {
            detail.last_update = v.to_string();
        } else if let Some(v) = trimmed.strip_prefix("installerPackageName=") {
            detail.installer = v.to_string();
        } else if trimmed.starts_with("install permissions:") {
            section = PermSection::Granted;
            continue;
        } else if trimmed.starts_with("runtime permissions:") {
            section = PermSection::Runtime;
            continue;
        } else if trimmed.ends_with(':') && !trimmed.contains("granted=") {
            section = PermSection::None;
        } else if section != PermSection::None && trimmed.contains(':') {
            let (perm, rest) = trimmed.split_once(':').unwrap_or((trimmed, ""));
            if rest.contains("granted=true") || section == PermSection::Granted {
                detail.granted_permissions.push(perm.trim().to_string());
            } else if rest.contains("granted=false") {
                detail.denied_permissions.push(perm.trim().to_string());
            }
        }
    }

    detail.raw_excerpt = output.lines().take(120).collect::<Vec<_>>().join("\n");
    detail
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_package_list_and_detail() {
        let all = "\
package:/data/app/~~123/com.termux/base.apk=com.termux versionCode:118 uid:10321
package:/system/app/Settings/Settings.apk=com.android.settings versionCode:35 uid:1000
";
        let sys = "package:com.android.settings\n";
        let dis = "";
        let pkgs = parse_packages_list(all, sys, dis);
        assert_eq!(pkgs.len(), 2);
        assert!(pkgs[0].is_system);
        assert!(!pkgs[1].is_system);
        assert_eq!(pkgs[1].uid, Some(10321));
        assert_eq!(pkgs[1].version_code, Some(118));

        let dump = "\
  Package [com.termux] (1234):
    userId=10321
    codePath=/data/app/~~123/com.termux
    versionCode=118 minSdk=24 targetSdk=28
    versionName=0.118.0
    dataDir=/data/user/0/com.termux
    install permissions:
      android.permission.INTERNET: granted=true
    runtime permissions:
      android.permission.POST_NOTIFICATIONS: granted=false, flags=[ USER_SET ]
";
        let detail = parse_package_detail("com.termux", dump);
        assert_eq!(detail.version_name, "0.118.0");
        assert_eq!(detail.target_sdk, "28");
        assert_eq!(detail.granted_permissions, vec!["android.permission.INTERNET"]);
        assert_eq!(
            detail.denied_permissions,
            vec!["android.permission.POST_NOTIFICATIONS"]
        );
    }
}
