use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_textarea::Input;

pub fn handle_key_event(app: &mut crate::app::App, key: KeyEvent) -> Option<String> {
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.quit();
            None
        }
        KeyCode::Enter if key.modifiers.contains(KeyModifiers::SHIFT) => {
            // Shift+Enter: add new line (forward to TextArea)
            app.input.input(Input::from(key));
            None
        }
        KeyCode::Enter => {
            // Enter without Shift: send message
            let message = app.input_take();
            if !message.trim().is_empty() {
                Some(message)
            } else {
                None
            }
        }
        KeyCode::Up if key.modifiers.contains(KeyModifiers::ALT) => {
            // Alt+Up: scroll messages up
            app.scroll_up();
            None
        }
        KeyCode::Down if key.modifiers.contains(KeyModifiers::ALT) => {
            // Alt+Down: scroll messages down
            app.scroll_down();
            None
        }
        _ => {
            // Forward all other keys to TextArea for handling
            app.input.input(Input::from(key));
            None
        }
    }
}
