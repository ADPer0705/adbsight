use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Gauge, Paragraph, Sparkline, Wrap};

use crate::adb::parsers::{format_bytes, format_kb};
use crate::app::state::AppState;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    if state.active_serial.is_none() && state.dashboard.identity.is_none() {
        let msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "No Android device connected",
                Style::default().fg(theme::WARN).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("• Plug in a device with USB Debugging enabled, OR"),
            Line::from("• Press [w] to open the Wireless Debugging Manager (QR Pair / mDNS / IP:Port), OR"),
            Line::from("• Press [H] to inspect a saved SQLite snapshot offline, OR"),
            Line::from("• Launch with `adbsight --demo` to explore with the built-in Pixel 9 Pro simulator."),
        ])
        .block(theme::active_panel_block("Welcome to ADBSight"))
        .wrap(Wrap { trim: false });
        frame.render_widget(msg, area);
        return;
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Min(6),
        ])
        .split(area);

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[0]);

    let mid = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);

    let bot = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[2]);

    render_identity(frame, top[0], state);
    render_cpu(frame, top[1], state);
    render_memory(frame, mid[0], state);
    render_battery_thermals(frame, mid[1], state);
    render_storage(frame, bot[0], state);
    render_network(frame, bot[1], state);
}

fn render_identity(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = theme::panel_block("Device Identity & Security");
    let Some(id) = state.dashboard.identity.as_ref() else {
        frame.render_widget(
            Paragraph::new("Loading device identity...").block(block),
            area,
        );
        return;
    };

    let selinux_color = if id.selinux.eq_ignore_ascii_case("enforcing") {
        theme::OK
    } else {
        theme::WARN
    };

    let lines = vec![
        kv("Model", &format!("{} {} ({})", id.manufacturer, id.model, id.device)),
        kv("Serial", &id.serial),
        kv("Android", &format!("{} (SDK {}) • Patch {}", id.android_version, id.sdk_int, id.security_patch)),
        kv("Build / ABI", &format!("{} • {}", id.build_id, id.cpu_abi)),
        kv("Screen", &format!("{} @ {} dpi", id.screen_size, id.screen_density)),
        kv("Kernel", &id.kernel_version),
        Line::from(vec![
            Span::styled("SELinux: ", Style::default().fg(theme::MUTED)),
            Span::styled(&id.selinux, Style::default().fg(selinux_color).add_modifier(Modifier::BOLD)),
            Span::styled("   Verified Boot: ", Style::default().fg(theme::MUTED)),
            Span::styled(&id.verified_boot, Style::default().fg(theme::TEXT)),
        ]),
    ];

    frame.render_widget(Paragraph::new(lines).block(block).wrap(Wrap { trim: true }), area);
}

fn render_cpu(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = theme::panel_block("CPU & Load");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(tel) = state.dashboard.telemetry.as_ref() else {
        frame.render_widget(Paragraph::new("Waiting for telemetry..."), inner);
        return;
    };

    let parts = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(3), Constraint::Min(2)])
        .split(inner);

    let header = Line::from(vec![
        Span::styled(
            format!("Total: {:.1}%", tel.cpu.total_pct),
            Style::default()
                .fg(theme::pct_color(tel.cpu.total_pct))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(
                "  •  Cores: {}  •  Load: {}",
                tel.cpu.per_core_pct.len(),
                tel.load_avg.as_deref().unwrap_or("n/a")
            ),
            Style::default().fg(theme::MUTED),
        ),
    ]);
    frame.render_widget(Paragraph::new(header), parts[0]);

    let spark_data: Vec<u64> = state.dashboard.cpu_history.iter().copied().collect();
    let spark = Sparkline::default()
        .data(&spark_data)
        .max(100)
        .style(Style::default().fg(theme::ACCENT));
    frame.render_widget(spark, parts[1]);

    let cores_text = tel
        .cpu
        .per_core_pct
        .iter()
        .enumerate()
        .map(|(i, p)| format!("C{i}:{p:4.0}%"))
        .collect::<Vec<_>>()
        .join("  ");
    frame.render_widget(
        Paragraph::new(cores_text)
            .style(Style::default().fg(theme::TEXT))
            .wrap(Wrap { trim: true }),
        parts[2],
    );
}

fn render_memory(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = theme::panel_block("Memory & Swap");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(tel) = state.dashboard.telemetry.as_ref() else {
        frame.render_widget(Paragraph::new("Waiting for telemetry..."), inner);
        return;
    };

    let parts = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Min(2),
        ])
        .split(inner);

    let ram_pct = tel.mem_used_pct();
    let ram_gauge = Gauge::default()
        .ratio((ram_pct / 100.0).clamp(0.0, 1.0))
        .label(format!(
            "RAM: {} / {} ({:.1}%)",
            format_kb(tel.mem_used_kb()),
            format_kb(tel.mem_total_kb),
            ram_pct
        ))
        .gauge_style(Style::default().fg(theme::pct_color(ram_pct)));
    frame.render_widget(ram_gauge, parts[0]);

    let swap_pct = if tel.swap_total_kb == 0 {
        0.0
    } else {
        (tel.swap_used_kb() as f64 / tel.swap_total_kb as f64) * 100.0
    };
    let swap_gauge = Gauge::default()
        .ratio((swap_pct / 100.0).clamp(0.0, 1.0))
        .label(format!(
            "Swap/zRAM: {} / {} ({:.1}%)",
            format_kb(tel.swap_used_kb()),
            format_kb(tel.swap_total_kb),
            swap_pct
        ))
        .gauge_style(Style::default().fg(theme::ACCENT_ALT));
    frame.render_widget(swap_gauge, parts[1]);

    let detail = Paragraph::new(Line::from(vec![
        Span::styled("Avail: ", Style::default().fg(theme::MUTED)),
        Span::raw(format_kb(tel.mem_available_kb)),
        Span::styled("  Cached: ", Style::default().fg(theme::MUTED)),
        Span::raw(format_kb(tel.mem_cached_kb)),
        Span::styled("  Buffers: ", Style::default().fg(theme::MUTED)),
        Span::raw(format_kb(tel.mem_buffers_kb)),
    ]));
    frame.render_widget(detail, parts[2]);
}

fn render_battery_thermals(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = theme::panel_block("Battery & Thermals");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(tel) = state.dashboard.telemetry.as_ref() else {
        frame.render_widget(Paragraph::new("Waiting for telemetry..."), inner);
        return;
    };

    let parts = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(3)])
        .split(inner);

    let level = tel.battery.level.unwrap_or(0);
    let bat_color = if level <= 15 {
        theme::ERR
    } else if level <= 35 {
        theme::WARN
    } else {
        theme::OK
    };
    let gauge = Gauge::default()
        .percent(u16::from(level.min(100)))
        .label(format!(
            "{}% • {} ({}) • {:.1}°C",
            level,
            tel.battery.status,
            tel.battery.plugged,
            tel.battery.temperature_c.unwrap_or(0.0)
        ))
        .gauge_style(Style::default().fg(bat_color));
    frame.render_widget(gauge, parts[0]);

    let therm_str = if tel.thermals.is_empty() {
        "No readable thermal zones".to_string()
    } else {
        tel.thermals
            .iter()
            .take(6)
            .map(|z| format!("{}: {:.1}°C", z.name, z.temp_c))
            .collect::<Vec<_>>()
            .join("  |  ")
    };
    let info = vec![
        kv(
            "Health / Volt",
            &format!(
                "{} • {} mV • {} mA",
                tel.battery.health,
                tel.battery.voltage_mv.unwrap_or(0),
                tel.battery.current_ma.unwrap_or(0)
            ),
        ),
        kv("Thermals", &therm_str),
    ];
    frame.render_widget(Paragraph::new(info).wrap(Wrap { trim: true }), parts[1]);
}

fn render_storage(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = theme::panel_block("Storage Mounts");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(tel) = state.dashboard.telemetry.as_ref() else {
        frame.render_widget(Paragraph::new("Waiting for storage info..."), inner);
        return;
    };

    if tel.storage.is_empty() {
        frame.render_widget(Paragraph::new("No storage mounts reported"), inner);
        return;
    }

    let lines: Vec<Line> = tel
        .storage
        .iter()
        .take(inner.height as usize)
        .map(|m| {
            let col = theme::pct_color(f64::from(m.use_pct));
            Line::from(vec![
                Span::styled(
                    format!("{:<18} ", m.mount_point),
                    Style::default().fg(theme::ACCENT_ALT).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{:>3}% ", m.use_pct),
                    Style::default().fg(col).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{} / {} ({} free)", format_kb(m.used_kb), format_kb(m.size_kb), format_kb(m.avail_kb)),
                    Style::default().fg(theme::TEXT),
                ),
            ])
        })
        .collect();
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_network(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = theme::panel_block("Network & Throughput");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(tel) = state.dashboard.telemetry.as_ref() else {
        frame.render_widget(Paragraph::new("Waiting for network stats..."), inner);
        return;
    };

    let parts = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(2)])
        .split(inner);

    let n = &tel.network;
    let lines = vec![
        kv(
            "Wi-Fi",
            &format!(
                "{} • IP {} • {} dBm",
                n.wifi_ssid.as_deref().unwrap_or("n/a"),
                n.wifi_ip.as_deref().unwrap_or("n/a"),
                n.wifi_rssi_dbm.map(|r| r.to_string()).unwrap_or_else(|| "?".into())
            ),
        ),
        kv(
            "Rate",
            &format!(
                "RX {}/s   TX {}/s   (Total {}/{})",
                format_bytes(n.rx_bytes_per_sec as u64),
                format_bytes(n.tx_bytes_per_sec as u64),
                format_bytes(n.total_rx_bytes),
                format_bytes(n.total_tx_bytes)
            ),
        ),
        kv(
            "Uptime / Carrier",
            &format!(
                "{}s • {}",
                tel.uptime_secs.unwrap_or(0),
                n.operator.as_deref().unwrap_or("n/a")
            ),
        ),
    ];
    frame.render_widget(Paragraph::new(lines), parts[0]);

    let rx_data: Vec<u64> = state.dashboard.rx_history.iter().copied().collect();
    let spark = Sparkline::default()
        .data(&rx_data)
        .style(Style::default().fg(theme::OK));
    frame.render_widget(spark, parts[1]);
}

fn kv(label: &str, val: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label}: "), Style::default().fg(theme::MUTED)),
        Span::styled(val.to_string(), Style::default().fg(theme::TEXT)),
    ])
}
