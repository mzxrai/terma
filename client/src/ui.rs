use crate::app::{App, ContentArea, LineKind, RenderedLine};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Messages
            Constraint::Length(5), // Input (3 lines + 2 for borders)
        ])
        .split(frame.area());

    // Clamp scroll before rendering
    let visible_height = chunks[1].height.saturating_sub(2) as usize;
    let available_width = chunks[1].width.saturating_sub(2) as usize;
    let total_lines: usize = app
        .messages
        .iter()
        .flat_map(|msg| msg.format_lines_for_display())
        .map(|line| {
            if available_width == 0 {
                1
            } else {
                ((line.len() + available_width - 1) / available_width).max(1)
            }
        })
        .sum();
    app.clamp_scroll(total_lines, visible_height);

    render_header(frame, app, chunks[0]);
    render_messages(frame, app, chunks[1]);
    render_input(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let status = if app.connected { "●" } else { "○" };
    let status_color = if app.connected {
        Color::Green
    } else {
        Color::Red
    };

    let header_text = vec![
        Span::styled(status, Style::default().fg(status_color)),
        Span::raw(" "),
        Span::styled(
            format!("Room: {} ", app.room_id),
            Style::default().fg(Color::White),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("Online: {} ", app.online_count),
            Style::default().fg(Color::Gray),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("You: {}", app.username),
            Style::default().fg(Color::Cyan),
        ),
    ];

    let header = Paragraph::new(Line::from(header_text)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .title(" Terma "),
    );

    frame.render_widget(header, area);
}

fn render_messages(frame: &mut Frame, app: &mut App, area: Rect) {
    // Calculate scroll position
    // scroll_offset = 0 means "at bottom" (newest messages visible)
    // scroll_offset > 0 means "scrolled N lines back from bottom"
    let visible_height = area.height.saturating_sub(2) as usize; // Subtract borders
    let available_width = area.width.saturating_sub(2) as usize; // Subtract borders
    let mut total_lines: usize = 0;

    // Build all rendered lines with wrapping and style metadata
    let wrap_width = available_width.max(1);
    let mut cache_lines: Vec<RenderedLine> = Vec::new();

    for msg in &app.messages {
        let kind = if msg.is_system {
            LineKind::System
        } else if msg.is_own_message {
            LineKind::Own
        } else {
            LineKind::Other
        };

        for logical_line in msg.format_lines_for_display() {
            if available_width == 0 {
                cache_lines.push(RenderedLine {
                    text: logical_line.clone(),
                    kind,
                });
                total_lines += 1;
                continue;
            }

            let segments = wrap_line(&logical_line, wrap_width);
            for segment in segments {
                cache_lines.push(RenderedLine {
                    text: segment,
                    kind,
                });
                total_lines += 1;
            }
        }
    }

    // Calculate actual scroll: skip lines from top to show the bottom minus scroll_offset
    // When scroll_offset = 0: show bottom (skip most lines)
    // When scroll_offset increases: show older (skip fewer lines)
    // Note: scroll_offset is already clamped in the render() function
    let scroll_value = if total_lines > visible_height {
        total_lines
            .saturating_sub(visible_height)
            .saturating_sub(app.scroll_offset)
    } else {
        0 // All content fits, no scrolling needed
    };

    app.update_render_cache(
        cache_lines,
        scroll_value as usize,
        Some(ContentArea {
            x: area.x,
            y: area.y,
            width: area.width,
            height: area.height,
        }),
    );

    let selection_range = app.selection.range();
    let mut rendered_lines: Vec<Line> = Vec::with_capacity(app.render_cache.lines.len());
    for (idx, rendered_line) in app.render_cache.lines.iter().enumerate() {
        rendered_lines.push(build_line(rendered_line, idx, selection_range));
    }

    let messages_widget = Paragraph::new(rendered_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White))
                .title(" Messages (Click+drag to select • Alt+↑/↓ scroll) "),
        )
        .wrap(Wrap { trim: true })
        .scroll((scroll_value as u16, 0));

    frame.render_widget(messages_widget, area);
}

fn render_input(frame: &mut Frame, app: &mut App, area: Rect) {
    // Set textarea block styling
    app.input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White))
                .title(" Type a message (Enter: send • Shift+Enter: new line • Alt+↑/↓: scroll • Ctrl+C: quit) "),
        );

    // Remove cursor line styling (no underline)
    app.input.set_cursor_line_style(Style::default());

    // Enable hard wrapping at widget boundary
    app.input.set_hard_tab_indent(false);

    // TextArea widget handles cursor positioning and multi-line rendering automatically
    frame.render_widget(&app.input, area);
}

fn build_line(
    rendered_line: &RenderedLine,
    index: usize,
    selection_range: Option<(crate::app::SelectionPosition, crate::app::SelectionPosition)>,
) -> Line<'_> {
    let base_style = match rendered_line.kind {
        LineKind::System => Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::ITALIC),
        LineKind::Own => Style::default().fg(Color::Cyan),
        LineKind::Other => Style::default().fg(Color::White),
    };

    let line_len = rendered_line.text.chars().count();

    if let Some((start, end)) = selection_range {
        if index < start.line || index > end.line {
            return Line::from(Span::styled(rendered_line.text.clone(), base_style));
        }

        let start_col = if index == start.line {
            start.column.min(line_len)
        } else {
            0
        };
        let end_col = if index == end.line {
            end.column.min(line_len)
        } else {
            line_len
        };

        let (pre, selected, post) = split_for_selection(&rendered_line.text, start_col, end_col);
        let mut spans = Vec::new();

        if !pre.is_empty() {
            spans.push(Span::styled(pre, base_style));
        }

        if !selected.is_empty() {
            spans.push(Span::styled(
                selected,
                base_style
                    .bg(Color::Rgb(90, 90, 90))
                    .add_modifier(Modifier::BOLD),
            ));
        }

        if !post.is_empty() {
            spans.push(Span::styled(post, base_style));
        }

        if spans.is_empty() {
            spans.push(Span::styled(String::new(), base_style));
        }

        Line::from(spans)
    } else {
        Line::from(Span::styled(rendered_line.text.clone(), base_style))
    }
}

fn split_for_selection(text: &str, start_col: usize, end_col: usize) -> (String, String, String) {
    let mut pre = String::new();
    let mut selected = String::new();
    let mut post = String::new();

    let mut idx = 0;
    for ch in text.chars() {
        if idx < start_col {
            pre.push(ch);
        } else if idx < end_col {
            selected.push(ch);
        } else {
            post.push(ch);
        }
        idx += 1;
    }

    (pre, selected, post)
}

fn wrap_line(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }

    if text.is_empty() {
        return vec![String::new()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();
    let mut count = 0;

    for ch in text.chars() {
        if count >= width {
            lines.push(current);
            current = String::new();
            count = 0;
        }
        current.push(ch);
        count += 1;
    }

    lines.push(current);
    lines
}
