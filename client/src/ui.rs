use ratatui::{
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),      // Messages
            Constraint::Length(3),  // Input
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
    // Calculate maximum width for messages (accounting for borders)
    let max_width = area.width.saturating_sub(2) as usize;

    let messages: Vec<Line> = app
        .messages
        .iter()
        .rev()
        .skip(app.scroll_offset)
        .take(area.height.saturating_sub(2) as usize)
        .rev()
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

            // Truncate message if it's too long
            let formatted = msg.format_for_display();
            let truncated = if formatted.len() > max_width {
                format!("{}…", &formatted[..max_width.saturating_sub(1)])
            } else {
                formatted
            };

            Line::from(Span::styled(truncated, style))
        })
        .collect();

    let messages_widget = Paragraph::new(messages)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White))
                .title(" Messages "),
        );

    frame.render_widget(messages_widget, area);
}

fn render_input(frame: &mut Frame, app: &App, area: Rect) {
    let input = Paragraph::new(app.input.as_str())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White))
                .title(" Type a message (Ctrl+C to quit) "),
        );

    frame.render_widget(input, area);

    // Set cursor position to show where user is typing
    // Position cursor at the end of the input text, accounting for the border
    frame.set_cursor_position(Position::new(
        area.x + app.input.len() as u16 + 1,
        area.y + 1,
    ));
}
