use crate::clipboard;
use chrono::{DateTime, Local, Utc};
use terma_shared::ChatMessage;
use tui_textarea::TextArea;

pub struct App {
    pub room_id: String,
    pub user_id: String,
    pub username: String,
    pub messages: Vec<DisplayMessage>,
    pub input: TextArea<'static>,
    pub online_count: usize,
    pub scroll_offset: usize, // Lines scrolled back from bottom (0 = at bottom)
    pub connected: bool,
    pub should_quit: bool,
    pub selection: SelectionState,
    pub render_cache: RenderCache,
}

#[derive(Clone)]
pub struct DisplayMessage {
    pub username: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub is_system: bool,
    pub is_own_message: bool,
}

impl App {
    pub fn new(room_id: String, user_id: String, username: String) -> Self {
        // Line numbers are disabled by default in TextArea
        let input = TextArea::default();

        Self {
            room_id,
            user_id,
            username,
            messages: Vec::new(),
            input,
            online_count: 0,
            scroll_offset: 0,
            connected: false,
            should_quit: false,
            selection: SelectionState::default(),
            render_cache: RenderCache::default(),
        }
    }

    pub fn add_message(&mut self, message: DisplayMessage) {
        self.messages.push(message);
        // Auto-scroll to bottom
        self.scroll_offset = 0;
        self.selection.clear();
    }

    pub fn add_chat_message(&mut self, msg: ChatMessage) {
        let is_own = msg.user_id == self.user_id;
        self.add_message(DisplayMessage {
            username: msg.username,
            content: msg.content,
            timestamp: msg.timestamp,
            is_system: false,
            is_own_message: is_own,
        });
    }

    pub fn add_system_message(&mut self, content: String) {
        self.add_system_message_with_time(content, Utc::now());
    }

    pub fn add_system_message_with_time(&mut self, content: String, timestamp: DateTime<Utc>) {
        self.add_message(DisplayMessage {
            username: "system".to_string(),
            content,
            timestamp,
            is_system: true,
            is_own_message: false,
        });
    }

    pub fn input_take(&mut self) -> String {
        let lines = self.input.lines().to_vec();
        self.input = TextArea::default();
        lines.join("\n")
    }

    pub fn scroll_up(&mut self) {
        // Scroll up by 3 lines for smoother scrolling
        self.scroll_offset = self.scroll_offset.saturating_add(3);
    }

    pub fn scroll_down(&mut self) {
        // Scroll down by 3 lines for smoother scrolling
        self.scroll_offset = self.scroll_offset.saturating_sub(3);
    }

    /// Clamp scroll offset based on actual content dimensions
    /// Call this before rendering to ensure scroll_offset is valid
    pub fn clamp_scroll(&mut self, total_lines: usize, visible_height: usize) {
        let max_scroll = if total_lines > visible_height {
            total_lines - visible_height
        } else {
            0
        };
        self.scroll_offset = self.scroll_offset.min(max_scroll);
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn update_render_cache(
        &mut self,
        lines: Vec<RenderedLine>,
        view_offset: usize,
        area: Option<ContentArea>,
    ) {
        self.render_cache.lines = lines;
        self.render_cache.view_offset = view_offset;
        self.render_cache.area = area;
    }

    pub fn message_position_from_mouse(&self, column: u16, row: u16) -> Option<SelectionPosition> {
        let area = self.render_cache.area?;

        // Account for borders: content starts one cell inside.
        let content_x = area.x.saturating_add(1);
        let content_y = area.y.saturating_add(1);
        let content_width = area.width.saturating_sub(2);
        let content_height = area.height.saturating_sub(2);

        if column < content_x || row < content_y {
            return None;
        }
        let rel_x = column - content_x;
        let rel_y = row - content_y;

        if rel_x >= content_width || rel_y >= content_height {
            return None;
        }

        let line_index = self.render_cache.view_offset.saturating_add(rel_y as usize);

        if line_index >= self.render_cache.lines.len() {
            return None;
        }

        let line = &self.render_cache.lines[line_index];
        let column = rel_x as usize;
        let max_column = line.text.chars().count();
        let clamped_column = column.min(max_column);

        Some(SelectionPosition {
            line: line_index,
            column: clamped_column,
        })
    }

    pub fn start_selection(&mut self, position: SelectionPosition) {
        self.selection.start(position);
    }

    pub fn update_selection(&mut self, position: SelectionPosition) {
        self.selection.update(position);
    }

    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    pub fn has_selection(&self) -> bool {
        self.selection.range().is_some()
    }

    pub fn selection_text(&self) -> Option<String> {
        let (start, end) = self.selection.range()?;
        let lines = &self.render_cache.lines;
        if lines.is_empty() {
            return None;
        }

        let end_line = end.line.min(lines.len().saturating_sub(1));
        let start_line = start.line.min(end_line);

        let start_line_len = lines[start_line].text.chars().count();
        let end_line_len = lines[end_line].text.chars().count();

        let start_col = start.column.min(start_line_len);
        let end_col = end.column.min(end_line_len);

        if start_line == end_line {
            let line = &lines[start_line].text;
            return Some(extract_range(line, start_col, end_col));
        }

        let mut collected = Vec::new();
        if let Some(first_line) = lines.get(start_line) {
            collected.push(extract_range(&first_line.text, start_col, usize::MAX));
        }
        for line_idx in (start_line + 1)..end_line {
            if let Some(line) = lines.get(line_idx) {
                collected.push(line.text.clone());
            }
        }
        if let Some(last_line) = lines.get(end_line) {
            collected.push(extract_range(&last_line.text, 0, end_col));
        }

        Some(collected.join("\n"))
    }

    pub fn copy_selection(&self) -> anyhow::Result<()> {
        if let Some(text) = self.selection_text() {
            clipboard::copy_to_clipboard(&text)?;
        }
        Ok(())
    }
}

impl DisplayMessage {
    pub fn format_time(&self) -> String {
        let local: DateTime<Local> = self.timestamp.into();
        local.format("%H:%M:%S").to_string()
    }

    pub fn format_lines_for_display(&self) -> Vec<String> {
        let mut lines = Vec::new();
        let mut content_lines = self.content.split('\n');
        let time = self.format_time();

        if let Some(first_line) = content_lines.next() {
            if self.is_system {
                lines.push(format!("[{}] {}", time, first_line));
            } else {
                lines.push(format!("[{}] {}: {}", time, self.username, first_line));
            }
        }

        lines.extend(content_lines.map(|line| line.to_string()));

        if lines.is_empty() {
            if self.is_system {
                lines.push(format!("[{}] {}", time, ""));
            } else {
                lines.push(format!("[{}] {}: {}", time, self.username, ""));
            }
        }

        lines
    }
}

#[derive(Default, Clone)]
pub struct SelectionState {
    anchor: Option<SelectionPosition>,
    head: Option<SelectionPosition>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SelectionPosition {
    pub line: usize,
    pub column: usize,
}

impl SelectionState {
    pub fn start(&mut self, position: SelectionPosition) {
        self.anchor = Some(position);
        self.head = Some(position);
    }

    pub fn update(&mut self, position: SelectionPosition) {
        if self.anchor.is_none() {
            self.anchor = Some(position);
        }
        self.head = Some(position);
    }

    pub fn clear(&mut self) {
        self.anchor = None;
        self.head = None;
    }

    pub fn range(&self) -> Option<(SelectionPosition, SelectionPosition)> {
        let anchor = self.anchor?;
        let head = self.head?;
        if anchor == head {
            return None;
        }

        if anchor.line < head.line || (anchor.line == head.line && anchor.column <= head.column) {
            Some((anchor, head))
        } else {
            Some((head, anchor))
        }
    }
}

#[derive(Clone)]
pub struct RenderedLine {
    pub text: String,
    pub kind: LineKind,
}

#[derive(Clone, Copy)]
pub enum LineKind {
    System,
    Own,
    Other,
}

#[derive(Default, Clone, Copy)]
pub struct ContentArea {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Default)]
pub struct RenderCache {
    pub lines: Vec<RenderedLine>,
    pub view_offset: usize,
    pub area: Option<ContentArea>,
}

fn extract_range(text: &str, start_col: usize, end_col: usize) -> String {
    let mut result = String::new();
    let mut current_col = 0;
    for ch in text.chars() {
        if current_col >= end_col {
            break;
        }
        if current_col >= start_col {
            result.push(ch);
        }
        current_col += 1;
    }
    result
}
