use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Cell, Paragraph, Row, Table, TableState, Tabs, Wrap};

use crate::app::state::{AppState, PersonalSubTab};
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

    let titles: Vec<Line> = PersonalSubTab::ALL
        .iter()
        .map(|t| Line::from(t.title()))
        .collect();
    let idx = PersonalSubTab::ALL
        .iter()
        .position(|&t| t == state.personal.sub_tab)
        .unwrap_or(0);
    frame.render_widget(
        Tabs::new(titles)
            .select(idx)
            .block(theme::panel_block("Personal Data Inspector [Tab switch, S snapshot, e export]"))
            .highlight_style(Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
        chunks[0],
    );

    let filter_block = if state.personal.filter.active {
        theme::active_panel_block("Filter (typing — Enter/Esc)")
    } else {
        theme::panel_block("Filter [/ search, Esc clear]")
    };
    frame.render_widget(
        Paragraph::new(format!("> {}", state.personal.filter.value)).block(filter_block),
        chunks[1],
    );

    if state.personal.loading {
        frame.render_widget(
            Paragraph::new("Querying Android content providers & accounts...")
                .block(theme::panel_block("Loading")),
            chunks[2],
        );
        return;
    }

    let q = state.personal.filter.value.to_ascii_lowercase();

    match state.personal.sub_tab {
        PersonalSubTab::Contacts => {
            let hint = state.personal.contacts.permission_hint.as_deref();
            let filtered: Vec<_> = state
                .personal
                .contacts
                .records
                .iter()
                .filter(|c| {
                    q.is_empty()
                        || c.name.to_ascii_lowercase().contains(&q)
                        || c.phone_numbers.iter().any(|p| p.contains(&q))
                        || c.emails.iter().any(|e| e.to_ascii_lowercase().contains(&q))
                })
                .collect();
            let rows: Vec<Row> = filtered
                .iter()
                .map(|c| {
                    Row::new(vec![
                        Cell::from(c.name.as_str()).style(Style::default().fg(theme::ACCENT_ALT)),
                        Cell::from(c.phone_numbers.join(", ")),
                        Cell::from(c.emails.join(", ")),
                    ])
                })
                .collect();
            render_with_hint(
                frame,
                chunks[2],
                hint,
                &format!("Contacts ({})", rows.len()),
                ["Name", "Phone Numbers", "Emails"],
                [
                    Constraint::Percentage(30),
                    Constraint::Percentage(35),
                    Constraint::Percentage(35),
                ],
                rows,
                state.personal.selected,
                None,
            );
        }
        PersonalSubTab::Sms => {
            let hint = state.personal.sms.permission_hint.as_deref();
            let filtered: Vec<_> = state
                .personal
                .sms
                .records
                .iter()
                .filter(|s| {
                    q.is_empty()
                        || s.address.to_ascii_lowercase().contains(&q)
                        || s.body.to_ascii_lowercase().contains(&q)
                })
                .collect();
            let selected_body = filtered
                .get(state.personal.selected)
                .map(|s| format!("From/To: {} ({})\nDate: {}\n\n{}", s.address, s.msg_type, s.date_display, s.body));
            let rows: Vec<Row> = filtered
                .iter()
                .map(|s| {
                    let preview = s.body.replace('\n', " ");
                    Row::new(vec![
                        Cell::from(s.msg_type.as_str()).style(Style::default().fg(theme::WARN)),
                        Cell::from(s.address.as_str()).style(Style::default().fg(theme::ACCENT_ALT)),
                        Cell::from(s.date_display.as_str()).style(Style::default().fg(theme::MUTED)),
                        Cell::from(preview),
                    ])
                })
                .collect();
            render_with_hint(
                frame,
                chunks[2],
                hint,
                &format!("SMS Messages ({})", rows.len()),
                ["Type", "Address", "Date", "Body"],
                [
                    Constraint::Length(9),
                    Constraint::Length(18),
                    Constraint::Length(20),
                    Constraint::Min(20),
                ],
                rows,
                state.personal.selected,
                selected_body,
            );
        }
        PersonalSubTab::CallLogs => {
            let hint = state.personal.call_logs.permission_hint.as_deref();
            let filtered: Vec<_> = state
                .personal
                .call_logs
                .records
                .iter()
                .filter(|c| {
                    q.is_empty()
                        || c.number.to_ascii_lowercase().contains(&q)
                        || c.call_type.to_ascii_lowercase().contains(&q)
                })
                .collect();
            let rows: Vec<Row> = filtered
                .iter()
                .map(|c| {
                    Row::new(vec![
                        Cell::from(c.number.as_str()).style(Style::default().fg(theme::ACCENT_ALT)),
                        Cell::from(c.call_type.as_str()).style(Style::default().fg(theme::WARN)),
                        Cell::from(c.date_display.as_str()),
                        Cell::from(format!("{}s", c.duration_secs)),
                    ])
                })
                .collect();
            render_with_hint(
                frame,
                chunks[2],
                hint,
                &format!("Call Logs ({})", rows.len()),
                ["Number", "Type", "Date", "Duration"],
                [
                    Constraint::Percentage(30),
                    Constraint::Percentage(20),
                    Constraint::Percentage(32),
                    Constraint::Percentage(18),
                ],
                rows,
                state.personal.selected,
                None,
            );
        }
        PersonalSubTab::Accounts => {
            let hint = state.personal.accounts.permission_hint.as_deref();
            let filtered: Vec<_> = state
                .personal
                .accounts
                .records
                .iter()
                .filter(|a| {
                    q.is_empty()
                        || a.account_type.to_ascii_lowercase().contains(&q)
                        || a.name.to_ascii_lowercase().contains(&q)
                })
                .collect();
            let rows: Vec<Row> = filtered
                .iter()
                .map(|a| {
                    Row::new(vec![
                        Cell::from(a.account_type.as_str()).style(Style::default().fg(theme::OK)),
                        Cell::from(a.name.as_str()),
                    ])
                })
                .collect();
            render_with_hint(
                frame,
                chunks[2],
                hint,
                &format!("Accounts ({})", rows.len()),
                ["Account Type", "Account Name / Email"],
                [Constraint::Percentage(40), Constraint::Percentage(60)],
                rows,
                state.personal.selected,
                None,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_with_hint<const N: usize>(
    frame: &mut Frame,
    area: Rect,
    hint: Option<&str>,
    title: &str,
    headers: [&str; N],
    widths: [Constraint; N],
    rows: Vec<Row>,
    selected: usize,
    detail: Option<String>,
) {
    let (hint_area, body_area) = if hint.is_some() {
        let split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(4)])
            .split(area);
        (Some(split[0]), split[1])
    } else {
        (None, area)
    };

    if let (Some(ha), Some(msg)) = (hint_area, hint) {
        let banner = Paragraph::new(Line::from(vec![
            Span::styled("NOTE: ", Style::default().fg(theme::WARN).add_modifier(Modifier::BOLD)),
            Span::styled(msg, Style::default().fg(theme::TEXT)),
        ]))
        .block(theme::panel_block("Android Provider Permission Notice"))
        .wrap(Wrap { trim: true });
        frame.render_widget(banner, ha);
    }

    let (table_area, detail_area) = if detail.is_some() {
        let split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
            .split(body_area);
        (split[0], Some(split[1]))
    } else {
        (body_area, None)
    };

    let mut ts = TableState::default();
    if !rows.is_empty() {
        ts.select(Some(selected.min(rows.len() - 1)));
    }
    let table = Table::new(rows, widths)
        .header(Row::new(headers).style(theme::table_header_style()))
        .block(theme::panel_block(title))
        .row_highlight_style(theme::row_highlight_style());
    frame.render_stateful_widget(table, table_area, &mut ts);

    if let (Some(da), Some(body)) = (detail_area, detail) {
        frame.render_widget(
            Paragraph::new(body)
                .block(theme::panel_block("Selected Message Detail"))
                .wrap(Wrap { trim: false }),
            da,
        );
    }
}
