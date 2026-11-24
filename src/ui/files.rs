use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Cell, Paragraph, Row, Table, TableState, Wrap};

use crate::adb::parsers::format_bytes;
use crate::app::state::AppState;
use crate::models::FsKind;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5)])
        .split(area);

    let bar = Paragraph::new(format!(
        "Path: {}  •  [Enter/l] Open/Preview  [Backspace/h] Up  [d] /sdcard  [t] /data/local/tmp  [p] Pull  [u] Push  [/] Filter",
        state.files.current_path
    ))
    .block(if state.files.filter.active {
        theme::active_panel_block("Filesystem Explorer (typing filter)")
    } else {
        theme::panel_block("Filesystem Explorer")
    });
    frame.render_widget(bar, chunks[0]);

    if state.files.loading {
        frame.render_widget(
            Paragraph::new("Listing directory...").block(theme::panel_block("Loading")),
            chunks[1],
        );
        return;
    }

    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(chunks[1]);

    if state.files.permission_denied && state.files.entries.is_empty() {
        frame.render_widget(
            Paragraph::new(
                "Permission denied reading this directory.\n\nPress [R] to enable Root Mode if your device has root/su access, or press [d] to jump to /sdcard.",
            )
            .style(Style::default().fg(theme::WARN))
            .block(theme::panel_block("Access Restricted"))
            .wrap(Wrap { trim: false }),
            split[0],
        );
    } else {
        let q = state.files.filter.value.to_ascii_lowercase();
        let filtered: Vec<_> = state
            .files
            .entries
            .iter()
            .filter(|e| q.is_empty() || e.name.to_ascii_lowercase().contains(&q))
            .collect();

        let rows: Vec<Row> = filtered
            .iter()
            .map(|e| {
                let (kind_tag, col) = match e.kind {
                    FsKind::Directory => ("DIR ", theme::ACCENT),
                    FsKind::Symlink => ("LNK ", theme::WARN),
                    FsKind::File => ("FILE", theme::TEXT),
                    FsKind::Other => ("OTH ", theme::MUTED),
                };
                let display_name = if let Some(ref tgt) = e.symlink_target {
                    format!("{} -> {tgt}", e.name)
                } else {
                    e.name.clone()
                };
                Row::new(vec![
                    Cell::from(kind_tag).style(Style::default().fg(col).add_modifier(Modifier::BOLD)),
                    Cell::from(display_name).style(Style::default().fg(col)),
                    Cell::from(if e.kind == FsKind::Directory {
                        "-".to_string()
                    } else {
                        format_bytes(e.size)
                    }),
                    Cell::from(e.permissions.as_str()).style(Style::default().fg(theme::MUTED)),
                ])
            })
            .collect();

        let mut ts = TableState::default();
        if !rows.is_empty() {
            ts.select(Some(state.files.selected.min(rows.len() - 1)));
        }
        let table = Table::new(
            rows,
            [
                Constraint::Length(5),
                Constraint::Min(20),
                Constraint::Length(10),
                Constraint::Length(11),
            ],
        )
        .header(Row::new(["Type", "Name", "Size", "Mode"]).style(theme::table_header_style()))
        .block(theme::panel_block(format!("Entries ({})", filtered.len())))
        .row_highlight_style(theme::row_highlight_style());
        frame.render_stateful_widget(table, split[0], &mut ts);
    }

    let preview_title = if state.files.preview_title.is_empty() {
        "File Preview [PgUp/PgDn scroll]".to_string()
    } else {
        format!("Preview: {}", state.files.preview_title)
    };
    let preview_body = if state.files.preview_content.is_empty() {
        "Select a file and press [Enter] to preview up to 32 KiB, or press [p] to pull any file/folder to your computer."
    } else {
        state.files.preview_content.as_str()
    };
    frame.render_widget(
        Paragraph::new(preview_body)
            .block(theme::panel_block(preview_title))
            .scroll((state.files.preview_scroll, 0))
            .wrap(Wrap { trim: false }),
        split[1],
    );
}
