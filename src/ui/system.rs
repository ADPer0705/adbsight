use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Cell, Paragraph, Row, Table, TableState, Tabs, Wrap};

use crate::app::state::{AppState, SystemSubTab};
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(5),
        ])
        .split(area);

    // Sub-tabs
    let titles: Vec<Line> = SystemSubTab::ALL
        .iter()
        .map(|t| Line::from(t.title()))
        .collect();
    let sub_idx = SystemSubTab::ALL
        .iter()
        .position(|&t| t == state.system.sub_tab)
        .unwrap_or(0);
    let tabs = Tabs::new(titles)
        .select(sub_idx)
        .block(theme::panel_block("System & Hardware [Tab to switch]"))
        .highlight_style(
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(tabs, chunks[0]);

    // Filter bar
    let filter_title = if state.system.filter.active {
        "Filter (typing — Enter/Esc to finish)"
    } else {
        "Filter [/ to search, Esc to clear]"
    };
    let filter_block = if state.system.filter.active {
        theme::active_panel_block(filter_title)
    } else {
        theme::panel_block(filter_title)
    };
    let filter_text =
        Paragraph::new(format!("> {}", state.system.filter.value)).block(filter_block);
    frame.render_widget(filter_text, chunks[1]);

    if state.system.loading {
        frame.render_widget(
            Paragraph::new("Querying Android system properties, settings, sensors, and dumpsys...")
                .block(theme::panel_block("Loading")),
            chunks[2],
        );
        return;
    }

    let q = state.system.filter.value.to_ascii_lowercase();
    match state.system.sub_tab {
        SystemSubTab::Properties => {
            let rows: Vec<Row> = state
                .system
                .props
                .iter()
                .filter(|p| {
                    q.is_empty()
                        || p.key.to_ascii_lowercase().contains(&q)
                        || p.value.to_ascii_lowercase().contains(&q)
                })
                .map(|p| {
                    Row::new(vec![
                        Cell::from(p.key.as_str()).style(Style::default().fg(theme::ACCENT_ALT)),
                        Cell::from(p.value.as_str()),
                    ])
                })
                .collect();
            render_table(
                frame,
                chunks[2],
                &format!("System Properties ({})", rows.len()),
                vec!["Property Key", "Value"],
                [Constraint::Percentage(45), Constraint::Percentage(55)],
                rows,
                state.system.selected,
            );
        }
        SystemSubTab::Settings => {
            let rows: Vec<Row> = state
                .system
                .settings
                .iter()
                .filter(|s| {
                    q.is_empty()
                        || s.key.to_ascii_lowercase().contains(&q)
                        || s.value.to_ascii_lowercase().contains(&q)
                })
                .map(|s| {
                    Row::new(vec![
                        Cell::from(s.namespace.as_str()).style(Style::default().fg(theme::WARN)),
                        Cell::from(s.key.as_str()).style(Style::default().fg(theme::ACCENT_ALT)),
                        Cell::from(s.value.as_str()),
                    ])
                })
                .collect();
            render_table(
                frame,
                chunks[2],
                &format!("Settings ({})", rows.len()),
                vec!["Namespace", "Key", "Value"],
                [
                    Constraint::Length(10),
                    Constraint::Percentage(40),
                    Constraint::Percentage(50),
                ],
                rows,
                state.system.selected,
            );
        }
        SystemSubTab::Sensors => {
            let rows: Vec<Row> = state
                .system
                .sensors
                .iter()
                .filter(|s| q.is_empty() || s.name.to_ascii_lowercase().contains(&q))
                .map(|s| {
                    Row::new(vec![
                        Cell::from(s.name.as_str()).style(Style::default().fg(theme::OK)),
                        Cell::from(s.vendor.as_str()),
                        Cell::from(s.sensor_type.as_str()),
                        Cell::from(s.detail.as_str()),
                    ])
                })
                .collect();
            render_table(
                frame,
                chunks[2],
                &format!("Hardware Sensors ({})", rows.len()),
                vec!["Name", "Vendor", "Type", "Specs"],
                [
                    Constraint::Percentage(30),
                    Constraint::Percentage(20),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                ],
                rows,
                state.system.selected,
            );
        }
        SystemSubTab::Dumpsys => {
            let split = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(32), Constraint::Percentage(68)])
                .split(chunks[2]);

            let svcs: Vec<Row> = state
                .system
                .dumpsys_list
                .iter()
                .filter(|s| q.is_empty() || s.to_ascii_lowercase().contains(&q))
                .map(|s| Row::new(vec![Cell::from(s.as_str())]))
                .collect();
            render_table(
                frame,
                split[0],
                &format!("Services ({}) [Enter to dump]", svcs.len()),
                vec!["Service"],
                [Constraint::Percentage(100)],
                svcs,
                state.system.selected,
            );

            let svc_title = state
                .system
                .dumpsys_active_service
                .as_deref()
                .map(|s| format!("dumpsys {s} [PgUp/PgDn scroll]"))
                .unwrap_or_else(|| "Select a service and press Enter".into());
            let body = if state.system.dumpsys_output.is_empty() {
                "Press Enter on any service on the left to inspect its live dumpsys output."
            } else {
                state.system.dumpsys_output.as_str()
            };
            let para = Paragraph::new(body)
                .block(theme::panel_block(svc_title))
                .scroll((state.system.dumpsys_scroll, 0))
                .wrap(Wrap { trim: false });
            frame.render_widget(para, split[1]);
        }
    }
}

fn render_table(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    headers: Vec<&str>,
    widths: impl IntoIterator<Item = Constraint>,
    rows: Vec<Row>,
    selected: usize,
) {
    let header = Row::new(
        headers
            .into_iter()
            .map(|h| Cell::from(Span::styled(h.to_string(), theme::table_header_style())))
            .collect::<Vec<_>>(),
    )
    .height(1);

    let mut ts = TableState::default();
    if !rows.is_empty() {
        ts.select(Some(selected.min(rows.len() - 1)));
    }

    let table = Table::new(rows, widths)
        .header(header)
        .block(theme::panel_block(title))
        .row_highlight_style(theme::row_highlight_style());
    frame.render_stateful_widget(table, area, &mut ts);
}
