use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use crate::app::state::AppState;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(72), Constraint::Percentage(28)])
        .split(area);

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3)])
        .split(split[0]);

    let out_text = state.shell.output_lines.join("\n");
    let visible_h = left[0].height.saturating_sub(2) as usize;
    let total_lines = state.shell.output_lines.len();
    let scroll = if total_lines > visible_h {
        state
            .shell
            .scroll
            .min(total_lines.saturating_sub(visible_h) as u16)
    } else {
        0
    };

    let output = Paragraph::new(out_text)
        .block(theme::panel_block("ADB Shell Console"))
        .scroll((scroll, 0))
        .wrap(Wrap { trim: false });
    frame.render_widget(output, left[0]);

    let prompt_title = if state.shell.running {
        "Running command..."
    } else if state.shell.input.active {
        "Shell Command (Enter to run, Esc for global keys)"
    } else {
        "Shell Command (press Enter/i to type)"
    };
    let prompt_block = if state.shell.input.active {
        theme::active_panel_block(prompt_title)
    } else {
        theme::panel_block(prompt_title)
    };
    let prefix = if state.use_root { "# " } else { "$ " };
    frame.render_widget(
        Paragraph::new(format!("{prefix}{}", state.shell.input.value)).block(prompt_block),
        left[1],
    );

    // Quick actions sidebar
    let actions = vec![
        Line::from(Span::styled(
            "Device Action Deck",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        action_line("F2", "Capture Screenshot (PNG)"),
        action_line("F3", "Reboot Device (confirm)"),
        action_line("F4", "Save Full SQLite Snapshot"),
        action_line("Ctrl+I", "Send Text Input to Device"),
        action_line("R", "Toggle Root Shell (# / $)"),
        action_line("w", "Wireless & QR Manager"),
        action_line("e", "Export Report (JSON/CSV/MD)"),
        Line::from(""),
        Line::from(Span::styled(
            "Suggested Probes:",
            Style::default().fg(theme::MUTED),
        )),
        Line::from("  id"),
        Line::from("  uname -a"),
        Line::from("  ip addr show wlan0"),
        Line::from("  dumpsys battery"),
        Line::from("  wm size"),
    ];
    frame.render_widget(
        Paragraph::new(actions)
            .block(theme::panel_block("Quick Actions"))
            .wrap(Wrap { trim: true }),
        split[1],
    );
}

fn action_line(key: &str, desc: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("[{key}] "),
            Style::default()
                .fg(theme::WARN)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(desc.to_string(), Style::default().fg(theme::TEXT)),
    ])
}
