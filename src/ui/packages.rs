use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Cell, Paragraph, Row, Table, TableState, Wrap};

use crate::app::state::AppState;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5)])
        .split(area);

    let bar = Paragraph::new(format!(
        "Scope: {} [f cycle]  •  Filter: \"{}\" [/ search]  •  [Enter] Inspect  [l] Launch  [x] Force-Stop  [p] Pull APK  [c] Clear Data",
        state.packages.filter_mode.label(),
        state.packages.filter.value
    ))
    .block(if state.packages.filter.active {
        theme::active_panel_block("Packages & Apps (typing filter)")
    } else {
        theme::panel_block("Packages & Apps")
    });
    frame.render_widget(bar, chunks[0]);

    if state.packages.loading {
        frame.render_widget(
            Paragraph::new("Enumerating installed packages via `pm list packages`...")
                .block(theme::panel_block("Loading")),
            chunks[1],
        );
        return;
    }

    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(46), Constraint::Percentage(54)])
        .split(chunks[1]);

    let q = state.packages.filter.value.to_ascii_lowercase();
    let filtered: Vec<_> = state
        .packages
        .packages
        .iter()
        .filter(|p| match state.packages.filter_mode {
            crate::app::state::PackageFilter::All => true,
            crate::app::state::PackageFilter::UserOnly => !p.is_system,
            crate::app::state::PackageFilter::SystemOnly => p.is_system,
        })
        .filter(|p| q.is_empty() || p.name.to_ascii_lowercase().contains(&q))
        .collect();

    let rows: Vec<Row> = filtered
        .iter()
        .map(|p| {
            let tag = if p.is_system { "SYS" } else { "USR" };
            let tag_color = if p.is_system { theme::MUTED } else { theme::OK };
            Row::new(vec![
                Cell::from(tag).style(Style::default().fg(tag_color).add_modifier(Modifier::BOLD)),
                Cell::from(p.name.as_str()).style(Style::default().fg(theme::TEXT)),
                Cell::from(p.uid.map(|u| u.to_string()).unwrap_or_else(|| "-".into())),
            ])
        })
        .collect();

    let mut ts = TableState::default();
    if !rows.is_empty() {
        ts.select(Some(state.packages.selected.min(rows.len() - 1)));
    }
    let table = Table::new(
        rows,
        [
            Constraint::Length(5),
            Constraint::Min(20),
            Constraint::Length(8),
        ],
    )
    .header(Row::new(["Type", "Package Name", "UID"]).style(theme::table_header_style()))
    .block(theme::panel_block(format!(
        "Installed ({})",
        filtered.len()
    )))
    .row_highlight_style(theme::row_highlight_style());
    frame.render_stateful_widget(table, split[0], &mut ts);

    // Detail pane
    let detail_block = theme::panel_block("Package Deep Inspector [PgUp/PgDn scroll]");
    if state.packages.detail_loading {
        frame.render_widget(
            Paragraph::new("Running `dumpsys package`...").block(detail_block),
            split[1],
        );
        return;
    }

    if let Some(d) = state.packages.detail.as_ref() {
        let mut lines = vec![
            kv("Package", &d.name),
            kv(
                "Version",
                &format!("{} (code {})", d.version_name, d.version_code),
            ),
            kv(
                "SDK",
                &format!("min {} / target {}", d.min_sdk, d.target_sdk),
            ),
            kv("UID / Installer", &format!("{} • {}", d.uid, d.installer)),
            kv("APK Path", &d.apk_path),
            kv("Data Dir", &d.data_dir),
            kv(
                "Installed / Updated",
                &format!("{} / {}", d.first_install, d.last_update),
            ),
            Line::from(""),
            Line::from(Span::styled(
                format!("Granted Permissions ({}):", d.granted_permissions.len()),
                Style::default().fg(theme::OK).add_modifier(Modifier::BOLD),
            )),
        ];
        for p in &d.granted_permissions {
            lines.push(Line::from(format!("  + {p}")));
        }
        if !d.denied_permissions.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!(
                    "Denied Runtime Permissions ({}):",
                    d.denied_permissions.len()
                ),
                Style::default()
                    .fg(theme::WARN)
                    .add_modifier(Modifier::BOLD),
            )));
            for p in &d.denied_permissions {
                lines.push(Line::from(format!("  - {p}")));
            }
        }
        frame.render_widget(
            Paragraph::new(lines)
                .block(detail_block)
                .scroll((state.packages.detail_scroll, 0))
                .wrap(Wrap { trim: false }),
            split[1],
        );
    } else if let Some(sel) = filtered.get(state.packages.selected) {
        let lines = vec![
            kv("Selected", &sel.name),
            kv("APK Path", &sel.apk_path),
            kv(
                "UID",
                &sel.uid.map(|u| u.to_string()).unwrap_or_else(|| "?".into()),
            ),
            Line::from(""),
            Line::from(
                "Press [Enter] to run full `dumpsys package` inspection (permissions, SDKs, timestamps).",
            ),
        ];
        frame.render_widget(
            Paragraph::new(lines)
                .block(detail_block)
                .wrap(Wrap { trim: true }),
            split[1],
        );
    } else {
        frame.render_widget(
            Paragraph::new("Select a package to inspect.").block(detail_block),
            split[1],
        );
    }
}

fn kv(label: &str, val: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label}: "), Style::default().fg(theme::MUTED)),
        Span::styled(val.to_string(), Style::default().fg(theme::TEXT)),
    ])
}
