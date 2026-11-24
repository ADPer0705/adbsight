use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, List, ListItem, ListState, Paragraph, Tabs, Wrap};

use crate::app::state::{
    AppState, ExportChoice, Modal, WirelessField, WirelessModalState, WirelessSubTab,
};
use crate::models::MdnsKind;
use crate::ui::theme;

pub fn render_modal(frame: &mut Frame, area: Rect, state: &AppState) {
    let Some(ref modal) = state.modal else {
        return;
    };

    match modal {
        Modal::Wireless(w) => render_wireless_modal(frame, area, state, w),
        Modal::Export { selected } => render_export_modal(frame, area, *selected),
        Modal::SnapshotSave(inp) => render_single_input_modal(
            frame,
            area,
            "Save Full Device Snapshot (SQLite)",
            "Snapshot label:",
            &inp.value,
        ),
        Modal::SnapshotHistory { list, selected } => {
            render_snapshot_history(frame, area, list, *selected)
        }
        Modal::ConfirmAction { title, message, .. } => {
            render_confirm_modal(frame, area, title, message)
        }
        Modal::InputPrompt {
            title,
            prompt,
            input,
            ..
        } => render_single_input_modal(frame, area, title, prompt, &input.value),
        Modal::Help => render_help_modal(frame, area),
    }
}

fn render_wireless_modal(
    frame: &mut Frame,
    area: Rect,
    state: &AppState,
    w: &WirelessModalState,
) {
    let popup = theme::centered_rect(86, 86, area);
    frame.render_widget(Clear, popup);

    let outer = theme::active_panel_block("Wireless Debugging & Device Manager [Tab switch, Esc close]");
    let inner = outer.inner(popup);
    frame.render_widget(outer, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(10)])
        .split(inner);

    let sub_idx = match w.sub_tab {
        WirelessSubTab::DevicesAndMdns => 0,
        WirelessSubTab::QrPair => 1,
        WirelessSubTab::ManualPairConnect => 2,
    };
    let tabs = Tabs::new(vec![
        Line::from("1 Devices & mDNS"),
        Line::from("2 QR Code Pair (Android 11+)"),
        Line::from("3 Pair Code / Direct IP:Port"),
    ])
    .select(sub_idx)
    .block(theme::panel_block("Mode"))
    .highlight_style(Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD));
    frame.render_widget(tabs, chunks[0]);

    match w.sub_tab {
        WirelessSubTab::DevicesAndMdns => {
            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[1]);

            let right = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
                .split(cols[1]);

            // Attached Devices
            let dev_items: Vec<ListItem> = if state.devices.is_empty() {
                vec![ListItem::new("No attached devices. Press [r] to refresh or Tab for QR/IP.")]
            } else {
                state
                    .devices
                    .iter()
                    .map(|d| {
                        let active_mark = if state.active_serial.as_deref() == Some(&d.serial) {
                            "★ "
                        } else {
                            "  "
                        };
                        ListItem::new(format!(
                            "{active_mark}[{}] {} ({}) {}",
                            d.transport.label(),
                            d.serial,
                            d.state.as_str(),
                            d.model.as_deref().unwrap_or("")
                        ))
                    })
                    .collect()
            };
            let mut d_state = ListState::default();
            if !state.devices.is_empty() {
                d_state.select(Some(w.selected_device.min(state.devices.len() - 1)));
            }
            let dev_block = if w.focus_section == 0 {
                theme::active_panel_block("[1] Attached Devices (Enter=Select, t=USB->WiFi, x=Disconnect)")
            } else {
                theme::panel_block("[1] Attached Devices")
            };
            frame.render_stateful_widget(
                List::new(dev_items)
                    .block(dev_block)
                    .highlight_style(theme::row_highlight_style()),
                cols[0],
                &mut d_state,
            );

            // mDNS Discovered
            let mdns_items: Vec<ListItem> = if state.mdns_services.is_empty() {
                vec![ListItem::new("No mDNS wireless services broadcast yet. Enable Wireless Debugging on phone.")]
            } else {
                state
                    .mdns_services
                    .iter()
                    .map(|s| {
                        let kind = match s.kind {
                            MdnsKind::Pairing => "PAIR",
                            MdnsKind::Connect => "CONN",
                            MdnsKind::Other => "MDNS",
                        };
                        ListItem::new(format!("[{kind}] {}:{} ({})", s.address, s.port, s.name))
                    })
                    .collect()
            };
            let mut m_state = ListState::default();
            if !state.mdns_services.is_empty() {
                m_state.select(Some(w.selected_mdns.min(state.mdns_services.len() - 1)));
            }
            let mdns_block = if w.focus_section == 1 {
                theme::active_panel_block("[2] mDNS Auto-Discovered (Enter=Connect)")
            } else {
                theme::panel_block("[2] mDNS Auto-Discovered")
            };
            frame.render_stateful_widget(
                List::new(mdns_items)
                    .block(mdns_block)
                    .highlight_style(theme::row_highlight_style()),
                right[0],
                &mut m_state,
            );

            // Saved Bookmarks
            let bm_items: Vec<ListItem> = if state.config.saved_devices.is_empty() {
                vec![ListItem::new("No saved Wi-Fi bookmarks yet (auto-saved on connect).")]
            } else {
                state
                    .config
                    .saved_devices
                    .iter()
                    .map(|b| ListItem::new(format!("{} -> {}", b.name, b.address)))
                    .collect()
            };
            let mut b_state = ListState::default();
            if !state.config.saved_devices.is_empty() {
                b_state.select(Some(
                    w.selected_bookmark.min(state.config.saved_devices.len() - 1),
                ));
            }
            let bm_block = if w.focus_section == 2 {
                theme::active_panel_block("[3] Saved Wi-Fi Bookmarks (Enter=Reconnect)")
            } else {
                theme::panel_block("[3] Saved Wi-Fi Bookmarks")
            };
            frame.render_stateful_widget(
                List::new(bm_items)
                    .block(bm_block)
                    .highlight_style(theme::row_highlight_style()),
                right[1],
                &mut b_state,
            );
        }
        WirelessSubTab::QrPair => {
            let split = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
                .split(chunks[1]);

            let qr_lines: Vec<Line> = if let Some(ref qr) = w.qr_session {
                qr.qr_lines
                    .iter()
                    .map(|l| Line::from(Span::styled(l.clone(), Style::default().fg(Color::White))))
                    .collect()
            } else {
                vec![Line::from("QR unavailable")]
            };
            frame.render_widget(
                Paragraph::new(qr_lines).block(theme::panel_block("Scan with Phone Camera")),
                split[0],
            );

            let info_lines = if let Some(ref qr) = w.qr_session {
                vec![
                    Line::from(Span::styled(
                        "Android 11+ Wireless QR Pairing",
                        Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from("1. On Android: Settings -> Developer options"),
                    Line::from("2. Enable Wireless debugging"),
                    Line::from("3. Tap 'Pair device with QR code'"),
                    Line::from("4. Scan the QR code on the left"),
                    Line::from(""),
                    Line::from(format!("Service: {}", qr.service_name)),
                    Line::from(format!("Password: {}", qr.password)),
                    Line::from(""),
                    Line::from(Span::styled(
                        format!("Status: {}", w.qr_status),
                        Style::default().fg(theme::OK),
                    )),
                    Line::from(""),
                    Line::from("Press [g] to generate a new QR code."),
                ]
            } else {
                vec![Line::from("Press [g] to generate QR session.")]
            };
            frame.render_widget(
                Paragraph::new(info_lines)
                    .block(theme::panel_block("Pairing Status"))
                    .wrap(Wrap { trim: true }),
                split[1],
            );
        }
        WirelessSubTab::ManualPairConnect => {
            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(4),
                ])
                .split(chunks[1]);

            let ep_block = if w.active_field == WirelessField::Endpoint {
                theme::active_panel_block("Endpoint IP:Port (e.g. 192.168.1.42:37185 or :5555)")
            } else {
                theme::panel_block("Endpoint IP:Port")
            };
            frame.render_widget(
                Paragraph::new(w.endpoint_input.value.as_str()).block(ep_block),
                rows[0],
            );

            let code_block = if w.active_field == WirelessField::PairingCode {
                theme::active_panel_block("6-Digit Pairing Code (leave empty to direct `adb connect`)")
            } else {
                theme::panel_block("6-Digit Pairing Code (optional)")
            };
            frame.render_widget(
                Paragraph::new(w.code_input.value.as_str()).block(code_block),
                rows[1],
            );

            let help = Paragraph::new(vec![
                Line::from("• Use ↑/↓ to switch between Endpoint and Pairing Code fields."),
                Line::from("• Press [Enter] to Pair (if code entered) or Connect (if code empty)."),
                Line::from(format!("• Last status: {}", w.qr_status)),
            ])
            .block(theme::panel_block("Instructions"))
            .wrap(Wrap { trim: true });
            frame.render_widget(help, rows[2]);
        }
    }
}

fn render_export_modal(frame: &mut Frame, area: Rect, selected: usize) {
    let popup = theme::centered_rect(60, 42, area);
    frame.render_widget(Clear, popup);

    let items: Vec<ListItem> = ExportChoice::ALL
        .iter()
        .map(|c| ListItem::new(c.label()))
        .collect();
    let mut ls = ListState::default();
    ls.select(Some(selected.min(ExportChoice::ALL.len() - 1)));

    frame.render_stateful_widget(
        List::new(items)
            .block(theme::active_panel_block("Export Data [Enter export, Esc cancel]"))
            .highlight_style(theme::row_highlight_style()),
        popup,
        &mut ls,
    );
}

fn render_snapshot_history(
    frame: &mut Frame,
    area: Rect,
    list: &[crate::models::SnapshotMeta],
    selected: usize,
) {
    let popup = theme::centered_rect(76, 65, area);
    frame.render_widget(Clear, popup);

    let items: Vec<ListItem> = if list.is_empty() {
        vec![ListItem::new("No snapshots saved yet. Press [S] on any device to capture one.")]
    } else {
        list.iter()
            .map(|m| {
                ListItem::new(format!(
                    "#{}  {}  |  {} (Android {})  |  {}  |  root={}",
                    m.id,
                    m.taken_at.format("%Y-%m-%d %H:%M"),
                    m.model,
                    m.android_version,
                    m.label,
                    m.root_used
                ))
            })
            .collect()
    };

    let mut ls = ListState::default();
    if !list.is_empty() {
        ls.select(Some(selected.min(list.len() - 1)));
    }

    frame.render_stateful_widget(
        List::new(items)
            .block(theme::active_panel_block(
                "SQLite Snapshot History [Enter load offline, d delete, Esc close]",
            ))
            .highlight_style(theme::row_highlight_style()),
        popup,
        &mut ls,
    );
}

fn render_confirm_modal(frame: &mut Frame, area: Rect, title: &str, message: &str) {
    let popup = theme::centered_rect(58, 30, area);
    frame.render_widget(Clear, popup);

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(message, Style::default().fg(theme::TEXT))),
        Line::from(""),
        Line::from(vec![
            Span::styled("[Enter / y] Confirm   ", Style::default().fg(theme::ERR).add_modifier(Modifier::BOLD)),
            Span::styled("[Esc / n] Cancel", Style::default().fg(theme::OK)),
        ]),
    ];

    frame.render_widget(
        Paragraph::new(text)
            .block(theme::active_panel_block(format!("CONFIRM: {title}")))
            .wrap(Wrap { trim: true }),
        popup,
    );
}

fn render_single_input_modal(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    prompt: &str,
    value: &str,
) {
    let popup = theme::centered_rect(64, 30, area);
    frame.render_widget(Clear, popup);

    let text = vec![
        Line::from(prompt),
        Line::from(""),
        Line::from(Span::styled(
            format!("> {value}_"),
            Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "[Enter] Submit   [Esc] Cancel",
            Style::default().fg(theme::MUTED),
        )),
    ];

    frame.render_widget(
        Paragraph::new(text)
            .block(theme::active_panel_block(title))
            .wrap(Wrap { trim: true }),
        popup,
    );
}

fn render_help_modal(frame: &mut Frame, area: Rect) {
    let popup = theme::centered_rect(76, 78, area);
    frame.render_widget(Clear, popup);

    let lines = vec![
        Line::from(Span::styled(
            "ADBSight — Keybindings & Deep Inspector Reference",
            Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  1 .. 8          Switch Inspector Tabs (Dashboard..Shell)"),
        Line::from("  w               Wireless & Device Manager (QR Pair, mDNS, USB->WiFi, IP:Port)"),
        Line::from("  R               Toggle Root Mode (unlocks protected providers & /data paths)"),
        Line::from("  r               Refresh active tab (or return to live device from snapshot)"),
        Line::from("  S / H           Save full SQLite snapshot / Browse snapshot history"),
        Line::from("  e               Export active tab or full device dossier (JSON / CSV / Markdown)"),
        Line::from("  j/k or ↑/↓      Navigate rows (PgUp/PgDn scroll detail panes)"),
        Line::from("  / and Esc       Search/filter active view; Esc clears filter or closes modal"),
        Line::from("  F2 / F3 / F4    Screenshot (PNG) / Reboot (confirmed) / Quick Snapshot"),
        Line::from("  q / Ctrl+C      Quit ADBSight"),
        Line::from(""),
        Line::from("Press [Esc], [Enter], or [?] to close."),
    ];

    frame.render_widget(
        Paragraph::new(lines)
            .block(theme::active_panel_block("Help"))
            .wrap(Wrap { trim: true }),
        popup,
    );
}
