use crate::models::{PropEntry, SensorEntry, SettingEntry, SettingNamespace};

/// Parse `adb shell getprop` output (`[key]: [value]`).
pub fn parse_getprop(output: &str) -> Vec<PropEntry> {
    let mut props = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('[') {
            continue;
        }
        if let Some((k_part, v_part)) = trimmed.split_once("]: [") {
            let key = k_part.trim_start_matches('[').trim().to_string();
            let value = v_part.trim_end_matches(']').to_string();
            props.push(PropEntry { key, value });
        }
    }
    props.sort_by(|a, b| a.key.cmp(&b.key));
    props
}

/// Parse `adb shell settings list <ns>` (`key=value`).
pub fn parse_settings(output: &str, namespace: SettingNamespace) -> Vec<SettingEntry> {
    let mut entries = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some((k, v)) = trimmed.split_once('=') {
            entries.push(SettingEntry {
                namespace,
                key: k.trim().to_string(),
                value: v.trim().to_string(),
            });
        }
    }
    entries.sort_by(|a, b| a.key.cmp(&b.key));
    entries
}

/// Parse `dumpsys sensorservice` Sensor List section.
pub fn parse_sensors(output: &str) -> Vec<SensorEntry> {
    let mut sensors = Vec::new();
    let mut in_list = false;

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Sensor List:") {
            in_list = true;
            continue;
        }
        if in_list && (trimmed.is_empty() || trimmed.starts_with("Fusion States")) {
            if !sensors.is_empty() {
                break;
            }
            continue;
        }
        if !in_list {
            continue;
        }
        // Typical line:
        // 0x00000001) LSM6DSR Accelerometer     | STMicroelectronics | ver: 1 | type: android.sensor.accelerometer(1) | ...
        if trimmed.contains('|') {
            let parts: Vec<&str> = trimmed.split('|').map(str::trim).collect();
            let raw_name = parts.first().copied().unwrap_or("");
            let name = raw_name
                .split_once(')')
                .map(|(_, rest)| rest.trim())
                .unwrap_or(raw_name)
                .to_string();
            let vendor = parts.get(1).copied().unwrap_or("").to_string();
            let sensor_type = parts
                .iter()
                .find(|p| p.starts_with("type:"))
                .map(|p| p.trim_start_matches("type:").trim().to_string())
                .unwrap_or_default();
            let detail = parts.into_iter().skip(2).collect::<Vec<_>>().join(" | ");
            if !name.is_empty() {
                sensors.push(SensorEntry {
                    name,
                    vendor,
                    sensor_type,
                    detail,
                });
            }
        }
    }
    sensors
}

pub fn parse_wm_value(output: &str) -> String {
    output
        .lines()
        .filter_map(|l| l.split_once(':').map(|(_, v)| v.trim()))
        .next_back()
        .unwrap_or_else(|| output.trim())
        .to_string()
}

pub fn parse_dumpsys_list(output: &str) -> Vec<String> {
    let mut services: Vec<String> = output
        .lines()
        .map(str::trim)
        .filter(|l| {
            !l.is_empty()
                && !l.starts_with("Currently running services:")
                && !l.starts_with("Can't find")
        })
        .map(ToString::to_string)
        .collect();
    services.sort();
    services.dedup();
    services
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_getprop_settings_and_sensors() {
        let props = parse_getprop(
            "[ro.product.model]: [Pixel 9 Pro]\n[ro.build.version.release]: [15]\n",
        );
        assert_eq!(props.len(), 2);
        assert_eq!(props[1].key, "ro.product.model");
        assert_eq!(props[1].value, "Pixel 9 Pro");

        let settings = parse_settings("screen_brightness=120\nvolume_ring=5\n", SettingNamespace::System);
        assert_eq!(settings.len(), 2);

        let sensor_dump = "\
Sensor List:
0x00000001) LSM6DSR Accelerometer | STMicroelectronics | ver: 1 | type: android.sensor.accelerometer(1) | maxRate=400.00Hz
0x00000002) AK09918 Magnetometer  | AKM | ver: 1 | type: android.sensor.magnetic_field(2) | maxRate=100.00Hz
Fusion States:
";
        let sensors = parse_sensors(sensor_dump);
        assert_eq!(sensors.len(), 2);
        assert_eq!(sensors[0].name, "LSM6DSR Accelerometer");
        assert_eq!(sensors[0].vendor, "STMicroelectronics");
    }
}
