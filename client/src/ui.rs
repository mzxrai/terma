use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),      // Messages
            Constraint::Length(5),  // Input (3 lines + 2 for borders)
        ])
        .split(frame.area());

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

    let header = Paragraph::new(Line::from(header_text))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White))
                .title(" Terma "),
        );

    frame.render_widget(header, area);
}

fn render_messages(frame: &mut Frame, app: &App, area: Rect) {
    // Build all messages without truncation
    let messages: Vec<Line> = app
        .messages
        .iter()
        .map(|msg| {
            let style = if msg.is_system {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::ITALIC)
            } else if msg.is_own_message {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::White)
            };

            // No truncation - let Paragraph wrap the text
            Line::from(Span::styled(msg.format_for_display(), style))
        })
        .collect();

    let messages_widget = Paragraph::new(messages)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White))
                .title(" Messages (Alt+↑/↓ to scroll) "),
        )
        .wrap(Wrap { trim: true })
        .scroll((app.scroll_offset as u16, 0));

    frame.render_widget(messages_widget, area);
}

fn render_input(frame: &mut Frame, app: &mut App, area: Rect) {
    // Set textarea block styling
    app.input.set_block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .title(" Type a message (Enter: send, Shift+Enter: new line, Alt+↑/↓: scroll, Ctrl+C: quit) "),
    );

    // Remove cursor line styling (no underline)
    app.input.set_cursor_line_style(Style::default());

    // Enable hard wrapping at widget boundary
    app.input.set_hard_tab_indent(false);

    // TextArea widget handles cursor positioning and multi-line rendering automatically
    frame.render_widget(&app.input, area);
}
