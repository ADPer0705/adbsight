use anyhow::{Result, anyhow};
use qrcode::{Color, QrCode};
use rand::RngExt;

const ALNUM: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

#[derive(Debug, Clone)]
pub struct QrPairingSession {
    pub service_name: String,
    pub password: String,
    pub payload: String,
    pub qr_lines: Vec<String>,
}

impl QrPairingSession {
    pub fn generate() -> Result<Self> {
        let mut rng = rand::rng();
        let suffix = random_alnum(&mut rng, 10);
        let service_name = format!("adbsight-{suffix}");
        let password = random_alnum(&mut rng, 12);
        let payload = format!("WIFI:T:ADB;S:{service_name};P:{password};;");
        let qr_lines = render_qr_lines(&payload)?;
        Ok(Self {
            service_name,
            password,
            payload,
            qr_lines,
        })
    }
}

fn random_alnum(rng: &mut impl RngExt, len: usize) -> String {
    (0..len)
        .map(|_| {
            let idx = rng.random_range(0..ALNUM.len());
            ALNUM[idx] as char
        })
        .collect()
}

/// Render a QR code into compact half-block Unicode lines (`▀`, `▄`, `█`, ` `)
/// with a 2-module white quiet zone around the matrix for reliable smartphone scanning.
/// Dark modules become dark (` `) on light (`█`) or inverted — here we emit dark foreground
/// blocks (`█`) on white background or vice versa. Specifically:
/// Using a bright background (`█` for light, ` ` for dark) makes dark squares clear
/// in dark terminals without needing ANSI background colors.
pub fn render_qr_lines(payload: &str) -> Result<Vec<String>> {
    let code = QrCode::new(payload.as_bytes())
        .map_err(|e| anyhow!("failed to encode QR payload: {e:?}"))?;
    let w = code.width();
    let quiet = 2usize;
    let total = w + quiet * 2;

    let is_light = |x: usize, y: usize| -> bool {
        if x < quiet || y < quiet || x >= quiet + w || y >= quiet + w {
            true
        } else {
            code[(x - quiet, y - quiet)] == Color::Light
        }
    };

    let mut lines = Vec::with_capacity(total.div_ceil(2));
    let mut y = 0usize;
    while y < total {
        let mut line = String::with_capacity(total);
        for x in 0..total {
            let top = is_light(x, y);
            let bottom = if y + 1 < total {
                is_light(x, y + 1)
            } else {
                true
            };
            let ch = match (top, bottom) {
                (true, true) => '█',
                (true, false) => '▀',
                (false, true) => '▄',
                (false, false) => ' ',
            };
            line.push(ch);
        }
        lines.push(line);
        y += 2;
    }

    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_valid_android_adb_wifi_qr_payload() {
        let session = QrPairingSession::generate().expect("qr generation should succeed");
        assert!(session.service_name.starts_with("adbsight-"));
        assert_eq!(session.password.len(), 12);
        assert!(session.payload.starts_with("WIFI:T:ADB;S:adbsight-"));
        assert!(session.payload.ends_with(";;"));
        assert!(session.qr_lines.len() > 10);
        let width = session.qr_lines[0].chars().count();
        assert!(width > 20);
        assert!(session.qr_lines.iter().all(|l| l.chars().count() == width));
    }
}
