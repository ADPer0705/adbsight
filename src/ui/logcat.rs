use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, Paragraph};

use crate::app::state::AppState;
use crate::models::LogLevel;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5)])
        .split(area);

    let status_badge = if state.logcat.paused {
        Span::styled("PAUSED", Style::default().fg(theme::WARN).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("LIVE", Style::default().fg(theme::OK).add_modifier(Modifier::BOLD))
    };

    let bar_line = Line::from(vec![
        status_badge,
        Span::raw(format!(
            "  •  Min Level: {} [v/d/i/w/e/f]  •  Filter: \"{}\" [/]  •  [Space] Pause  [C] Clear  [G] Tail  [E] Export",
            state.logcat.min_level.code(),
            state.logcat.filter.value
        )),
    ]);
    let bar = Paragraph::new(bar_line).block(if state.logcat.filter.active {
        theme::active_panel_block("Live Logcat (typing filter)")
    } else {
        theme::panel_block("Live Logcat Stream")
    });
    frame.render_widget(bar, chunks[0]);

    let q = state.logcat.filter.value.to_ascii_lowercase();
    let filtered: Vec<_> = state
        .logcat
        .entries
        .iter()
        .filter(|e| e.level >= state.logcat.min_level)
        .filter(|e| {
            q.is_empty()
                || e.tag.to_ascii_lowercase().contains(&q)
                || e.message.to_ascii_lowercase().contains(&q)
                || e.pid.contains(&q)
        })
        .collect();

    let visible_height = chunks[1].height.saturating_sub(2) as usize;
    let total = filtered.len();
    let start = if state.logcat.auto_scroll {
        total.saturating_sub(visible_height)
    } else {
        state.logcat.scroll.min(total.saturating_sub(visible_height))
    };

    let items: Vec<ListItem> = filtered
        .into_iter()
        .skip(start)
        .take(visible_height.max(1))
        .map(|e| {
            let lvl_color = match e.level {
                LogLevel::Verbose => theme::MUTED,
                LogLevel::Debug => Color::Blue,
                LogLevel::Info => theme::OK,
                LogLevel::Warn => theme::WARN,
                LogLevel::Error | LogLevel::Fatal => theme::ERR,
                LogLevel::Silent => theme::MUTED,
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{} ", e.timestamp), Style::default().fg(theme::MUTED)),
                Span::styled(format!("{:>5} ", e.pid), Style::default().fg(theme::MUTED)),
                Span::styled(
                    format!(" {} ", e.level.code()),
                    Style::default().fg(Color::Black).bg(lvl_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!(" {}: ", e.tag), Style::default().fg(theme::ACCENT_ALT)),
                Span::styled(e.message.clone(), Style::default().fg(theme::TEXT)),
            ]))
        })
        .collect();

    let list = List::new(items).block(theme::panel_block(format!(
        "Logcat Buffer ({total} matching / {} stored)",
        state.logcat.entries.len()
    )));
    frame.render_widget(list, chunks[1]);
}
