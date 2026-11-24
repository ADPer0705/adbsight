use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Cell, Paragraph, Row, Table, TableState};

use crate::adb::parsers::format_kb;
use crate::app::App;
use crate::app::state::{AppState, ProcessSubTab};
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5)])
        .split(area);

    let mode_label = match state.processes.sub_tab {
        ProcessSubTab::Processes => "Processes",
        ProcessSubTab::Services => "Running Services",
    };
    let bar = Paragraph::new(format!(
        "View: {mode_label} [Tab switch]  •  Sort: {} [s cycle]  •  Filter: \"{}\" [/ search]  •  [K] Kill PID  •  [r] Refresh",
        state.processes.sort.label(),
        state.processes.filter.value
    ))
    .block(if state.processes.filter.active {
        theme::active_panel_block("Processes & Services (typing filter)")
    } else {
        theme::panel_block("Processes & Services")
    });
    frame.render_widget(bar, chunks[0]);

    if state.processes.loading {
        frame.render_widget(
            Paragraph::new("Fetching process table & active services...")
                .block(theme::panel_block("Loading")),
            chunks[1],
        );
        return;
    }

    match state.processes.sub_tab {
        ProcessSubTab::Processes => {
            let list = filtered_sorted_processes(state);
            let rows: Vec<Row> = list
                .iter()
                .map(|p| {
                    Row::new(vec![
                        Cell::from(p.pid.to_string()),
                        Cell::from(p.user.as_str()).style(Style::default().fg(theme::MUTED)),
                        Cell::from(format!("{:.1}%", p.cpu_pct))
                            .style(Style::default().fg(theme::pct_color(p.cpu_pct))),
                        Cell::from(format_kb(p.rss_kb)),
                        Cell::from(p.state.as_str()),
                        Cell::from(p.name.as_str()).style(Style::default().fg(theme::TEXT)),
                    ])
                })
                .collect();

            let mut ts = TableState::default();
            if !rows.is_empty() {
                ts.select(Some(state.processes.selected.min(rows.len() - 1)));
            }
            let table = Table::new(
                rows,
                [
                    Constraint::Length(7),
                    Constraint::Length(12),
                    Constraint::Length(8),
                    Constraint::Length(11),
                    Constraint::Length(6),
                    Constraint::Min(20),
                ],
            )
            .header(
                Row::new(["PID", "USER", "CPU%", "RSS", "S", "NAME"])
                    .style(theme::table_header_style()),
            )
            .block(theme::panel_block(format!("Processes ({})", list.len())))
            .row_highlight_style(theme::row_highlight_style());
            frame.render_stateful_widget(table, chunks[1], &mut ts);
        }
        ProcessSubTab::Services => {
            let q = state.processes.filter.value.to_ascii_lowercase();
            let rows: Vec<Row> = state
                .processes
                .services
                .iter()
                .filter(|s| {
                    q.is_empty()
                        || s.package.to_ascii_lowercase().contains(&q)
                        || s.service_class.to_ascii_lowercase().contains(&q)
                })
                .map(|s| {
                    Row::new(vec![
                        Cell::from(s.pid.map(|p| p.to_string()).unwrap_or_else(|| "-".into())),
                        Cell::from(s.uid.map(|u| u.to_string()).unwrap_or_else(|| "-".into())),
                        Cell::from(s.package.as_str())
                            .style(Style::default().fg(theme::ACCENT_ALT)),
                        Cell::from(s.service_class.as_str()),
                    ])
                })
                .collect();

            let mut ts = TableState::default();
            if !rows.is_empty() {
                ts.select(Some(state.processes.selected.min(rows.len() - 1)));
            }
            let table = Table::new(
                rows,
                [
                    Constraint::Length(8),
                    Constraint::Length(8),
                    Constraint::Percentage(38),
                    Constraint::Percentage(46),
                ],
            )
            .header(
                Row::new(["PID", "UID", "Package", "Service Class"])
                    .style(theme::table_header_style()),
            )
            .block(theme::panel_block(format!(
                "Running Services ({})",
                state.processes.services.len()
            )))
            .row_highlight_style(theme::row_highlight_style());
            frame.render_stateful_widget(table, chunks[1], &mut ts);
        }
    }
}

fn filtered_sorted_processes(state: &AppState) -> Vec<&crate::models::ProcessEntry> {
    let q = state.processes.filter.value.to_ascii_lowercase();
    let mut list: Vec<_> = state
        .processes
        .processes
        .iter()
        .filter(|p| {
            q.is_empty()
                || p.name.to_ascii_lowercase().contains(&q)
                || p.user.to_ascii_lowercase().contains(&q)
                || p.pid.to_string().contains(&q)
        })
        .collect();
    match state.processes.sort {
        crate::app::state::ProcessSort::CpuDesc => list.sort_by(|a, b| {
            b.cpu_pct
                .partial_cmp(&a.cpu_pct)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        crate::app::state::ProcessSort::RamDesc => {
            list.sort_by_key(|a| std::cmp::Reverse(a.rss_kb))
        }
        crate::app::state::ProcessSort::PidAsc => list.sort_by_key(|a| a.pid),
        crate::app::state::ProcessSort::NameAsc => list.sort_by(|a, b| a.name.cmp(&b.name)),
    }
    let _ = std::marker::PhantomData::<App>;
    list
}
