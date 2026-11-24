use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType};

pub const BG_PANEL: Color = Color::Rgb(16, 19, 26);
pub const BG_HEADER: Color = Color::Rgb(22, 27, 36);
pub const BG_SELECTED: Color = Color::Rgb(34, 52, 76);

pub const ACCENT: Color = Color::Cyan;
pub const ACCENT_ALT: Color = Color::Rgb(120, 175, 255);
pub const OK: Color = Color::Rgb(80, 210, 130);
pub const WARN: Color = Color::Rgb(245, 190, 65);
pub const ERR: Color = Color::Rgb(240, 90, 100);
pub const MUTED: Color = Color::Rgb(125, 135, 152);
pub const TEXT: Color = Color::Rgb(228, 232, 240);

pub fn panel_block(title: impl Into<String>) -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(56, 66, 84)))
        .title(format!(" {} ", title.into()))
        .title_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))
}

pub fn active_panel_block(title: impl Into<String>) -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(ACCENT))
        .title(format!(" {} ", title.into()))
        .title_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))
}

pub fn table_header_style() -> Style {
    Style::default()
        .fg(ACCENT_ALT)
        .add_modifier(Modifier::BOLD)
}

pub fn row_highlight_style() -> Style {
    Style::default()
        .bg(BG_SELECTED)
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
}

pub fn pct_color(pct: f64) -> Color {
    if pct >= 85.0 {
        ERR
    } else if pct >= 65.0 {
        WARN
    } else {
        OK
    }
}

pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
