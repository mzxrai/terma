use chrono::{DateTime, Local, Utc};
use terma_shared::ChatMessage;

pub struct App {
    pub room_id: String,
    pub user_id: String,
    pub username: String,
    pub messages: Vec<DisplayMessage>,
    pub input: String,
    pub online_count: usize,
    pub scroll_offset: usize,
    pub connected: bool,
    pub should_quit: bool,
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
        Self {
            room_id,
            user_id,
            username,
            messages: Vec::new(),
            input: String::new(),
            online_count: 0,
            scroll_offset: 0,
            connected: false,
            should_quit: false,
        }
    }

    pub fn add_message(&mut self, message: DisplayMessage) {
        self.messages.push(message);
        // Auto-scroll to bottom
        self.scroll_offset = 0;
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
        self.add_message(DisplayMessage {
            username: "system".to_string(),
            content,
            timestamp: Utc::now(),
            is_system: true,
            is_own_message: false,
        });
    }

    pub fn input_push(&mut self, c: char) {
        self.input.push(c);
    }

    pub fn input_pop(&mut self) {
        self.input.pop();
    }

    pub fn input_take(&mut self) -> String {
        std::mem::take(&mut self.input)
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset < self.messages.len().saturating_sub(1) {
            self.scroll_offset += 1;
        }
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}

impl DisplayMessage {
    pub fn format_time(&self) -> String {
        let local: DateTime<Local> = self.timestamp.into();
        local.format("%H:%M:%S").to_string()
    }

    pub fn format_for_display(&self) -> String {
        if self.is_system {
            format!("[{}] {}", self.format_time(), self.content)
        } else {
            format!("[{}] {}: {}", self.format_time(), self.username, self.content)
        }
    }
}
