pub mod dashboard;
pub mod files;
pub mod logcat;
pub mod modals;
pub mod packages;
pub mod personal;
pub mod processes;
pub mod shell;
pub mod system;
pub mod theme;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Paragraph, Tabs};

use crate::app::state::{AppState, Tab};

pub fn render(frame: &mut Frame, state: &mut AppState) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(1),
        ])
        .split(area);

    render_header(frame, chunks[0], state);

    match state.active_tab {
        Tab::Dashboard => dashboard::render(frame, chunks[1], state),
        Tab::System => system::render(frame, chunks[1], state),
        Tab::Processes => processes::render(frame, chunks[1], state),
        Tab::Packages => packages::render(frame, chunks[1], state),
        Tab::Logcat => logcat::render(frame, chunks[1], state),
        Tab::Files => files::render(frame, chunks[1], state),
        Tab::Personal => personal::render(frame, chunks[1], state),
        Tab::Shell => shell::render(frame, chunks[1], state),
    }

    render_footer(frame, chunks[2], state);
    modals::render_modal(frame, area, state);
    render_toasts(frame, area, state);
}

fn render_header(frame: &mut Frame, area: Rect, state: &AppState) {
    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(40), Constraint::Length(44)])
        .split(area);

    let tab_titles: Vec<Line> = Tab::ALL.iter().map(|t| Line::from(t.title())).collect();
    let tabs = Tabs::new(tab_titles)
        .select(state.active_tab.index())
        .block(theme::panel_block("ADBSight // Android Deep Inspector"))
        .highlight_style(
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(tabs, split[0]);

    let dev_label = state
        .active_device()
        .map(|d| {
            format!(
                "[{}] {}",
                d.transport.label(),
                d.model.as_deref().unwrap_or(&d.serial)
            )
        })
        .or_else(|| state.active_serial.clone())
        .unwrap_or_else(|| "No Device".into());

    let root_badge = if state.use_root {
        Span::styled(
            " ROOT ",
            Style::default()
                .fg(Color::Black)
                .bg(theme::ERR)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            format!(" {} ", state.root_status.label()),
            Style::default().fg(theme::MUTED),
        )
    };

    let snap_badge = if state.viewing_snapshot.is_some() {
        Span::styled(
            " SNAPSHOT ",
            Style::default()
                .fg(Color::Black)
                .bg(theme::WARN)
                .add_modifier(Modifier::BOLD),
        )
    } else if state.is_demo {
        Span::styled(
            " DEMO ",
            Style::default()
                .fg(Color::Black)
                .bg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::raw("")
    };

    let status = Paragraph::new(Line::from(vec![
        Span::styled(
            dev_label,
            Style::default().fg(theme::TEXT).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        root_badge,
        Span::raw(" "),
        snap_badge,
    ]))
    .block(theme::panel_block("Target"));
    frame.render_widget(status, split[1]);
}

fn render_footer(frame: &mut Frame, area: Rect, state: &AppState) {
    let text = Line::from(vec![
        Span::styled(" [1-8]", Style::default().fg(theme::ACCENT)),
        Span::raw(" Tabs "),
        Span::styled("[w]", Style::default().fg(theme::ACCENT)),
        Span::raw(" WiFi/Devices "),
        Span::styled("[R]", Style::default().fg(theme::ACCENT)),
        Span::raw(" Root "),
        Span::styled("[r]", Style::default().fg(theme::ACCENT)),
        Span::raw(" Refresh "),
        Span::styled("[S/H]", Style::default().fg(theme::ACCENT)),
        Span::raw(" Snap/Hist "),
        Span::styled("[e]", Style::default().fg(theme::ACCENT)),
        Span::raw(" Export "),
        Span::styled("[?]", Style::default().fg(theme::ACCENT)),
        Span::raw(" Help "),
        Span::styled("[q]", Style::default().fg(theme::ACCENT)),
        Span::raw(" Quit   "),
        Span::styled(&state.adb_info_label, Style::default().fg(theme::MUTED)),
    ]);
    frame.render_widget(
        Paragraph::new(text).style(Style::default().bg(theme::BG_HEADER)),
        area,
    );
}

fn render_toasts(frame: &mut Frame, area: Rect, state: &AppState) {
    if state.toasts.is_empty() || area.width < 36 || area.height < 6 {
        return;
    }

    let width = 52.min(area.width.saturating_sub(4));
    let mut y = 3u16;
    for toast in state.toasts.iter().rev().take(3) {
        if y + 3 >= area.height {
            break;
        }
        let rect = Rect {
            x: area.width.saturating_sub(width + 2),
            y,
            width,
            height: 3,
        };
        frame.render_widget(Clear, rect);
        let border_col = if toast.is_error { theme::ERR } else { theme::OK };
        let title = if toast.is_error { " Error " } else { " Notice " };
        let block = ratatui::widgets::Block::bordered()
            .border_style(Style::default().fg(border_col))
            .title(title);
        frame.render_widget(Paragraph::new(toast.message.as_str()).block(block), rect);
        y += 3;
    }
}
