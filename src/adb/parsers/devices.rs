use crate::models::{DeviceState, DeviceSummary, MdnsKind, MdnsService, TransportKind};

/// Parse output of `adb devices -l`.
pub fn parse_adb_devices_l(output: &str) -> Vec<DeviceSummary> {
    let mut devices = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with("List of devices")
            || trimmed.starts_with('*')
        {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let Some(serial) = parts.next() else {
            continue;
        };
        let Some(state_raw) = parts.next() else {
            continue;
        };

        let mut model = None;
        let mut product = None;
        let mut device = None;
        let mut transport_id = None;

        for token in parts {
            if let Some((k, v)) = token.split_once(':') {
                match k {
                    "model" => model = Some(v.replace('_', " ")),
                    "product" => product = Some(v.to_string()),
                    "device" => device = Some(v.to_string()),
                    "transport_id" => transport_id = Some(v.to_string()),
                    _ => {}
                }
            }
        }

        devices.push(DeviceSummary {
            serial: serial.to_string(),
            state: DeviceState::from_adb(state_raw),
            transport: TransportKind::infer(serial),
            model,
            product,
            device,
            transport_id,
        });
    }
    devices
}

/// Parse output of `adb mdns services`.
/// Format example:
/// `adb-2A181FDH200012-abcDEF  _adb-tls-connect._tcp.  192.168.1.42:43821`
pub fn parse_mdns_services(output: &str) -> Vec<MdnsService> {
    let mut services = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("List of discovered") {
            continue;
        }
        let cols: Vec<&str> = trimmed.split_whitespace().collect();
        if cols.len() < 3 {
            continue;
        }
        let name = cols[0].to_string();
        let service_type = cols[1].trim_end_matches('.').to_string();
        let endpoint = cols[2];

        let Some((addr, port_str)) = endpoint.rsplit_once(':') else {
            continue;
        };
        let Ok(port) = port_str.parse::<u16>() else {
            continue;
        };

        let kind = if service_type.contains("_adb-tls-pairing") {
            MdnsKind::Pairing
        } else if service_type.contains("_adb-tls-connect") {
            MdnsKind::Connect
        } else {
            MdnsKind::Other
        };

        services.push(MdnsService {
            name,
            service_type,
            address: addr.trim_matches(|c| c == '[' || c == ']').to_string(),
            port,
            kind,
        });
    }
    services
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_usb_wifi_and_unauthorized_devices() {
        let sample = "\
* daemon not running; starting now at tcp:5037
* daemon started successfully
List of devices attached
2A181FDH200012         device usb:1-2 product:panther model:Pixel_7 device:panther transport_id:1
192.168.1.50:5555      device product:husky model:Pixel_8_Pro device:husky transport_id:4
emulator-5554          unauthorized transport_id:7
";
        let devs = parse_adb_devices_l(sample);
        assert_eq!(devs.len(), 3);
        assert_eq!(devs[0].transport, TransportKind::Usb);
        assert_eq!(devs[0].model.as_deref(), Some("Pixel 7"));
        assert_eq!(devs[1].transport, TransportKind::Wifi);
        assert_eq!(devs[1].state, DeviceState::Device);
        assert_eq!(devs[2].transport, TransportKind::Emulator);
        assert_eq!(devs[2].state, DeviceState::Unauthorized);
    }

    #[test]
    fn parses_mdns_pairing_and_connect_services() {
        let sample = "\
List of discovered mdns services
adbsight-9x4k2m1p0q\t_adb-tls-pairing._tcp.\t192.168.1.42:37185
adb-2A181FDH200012-xyz\t_adb-tls-connect._tcp\t192.168.1.42:41903
";
        let services = parse_mdns_services(sample);
        assert_eq!(services.len(), 2);
        assert_eq!(services[0].kind, MdnsKind::Pairing);
        assert_eq!(services[0].address, "192.168.1.42");
        assert_eq!(services[0].port, 37185);
        assert_eq!(services[1].kind, MdnsKind::Connect);
        assert_eq!(services[1].port, 41903);
    }
}
