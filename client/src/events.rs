use crossterm::event::KeyEvent;
use tui_textarea::{Input, Key};

pub fn handle_key_event(app: &mut crate::app::App, key: KeyEvent) -> Option<String> {
    // Convert crossterm KeyEvent to tui_textarea Input first
    let input = Input::from(key);

    // Match on the tui_textarea Input struct to properly detect Shift+Enter
    match input {
        // Ctrl+C: quit application
        Input {
            key: Key::Char('c'),
            ctrl: true,
            ..
        } => {
            app.quit();
            None
        }
        // Shift+Enter: insert newline (multi-line message)
        Input {
            key: Key::Enter,
            shift: true,
            ..
        } => {
            app.input.insert_newline();
            None
        }
        // Plain Enter or Ctrl+M: send message
        Input {
            key: Key::Enter, ..
        }
        | Input {
            key: Key::Char('m'),
            ctrl: true,
            ..
        } => {
            let message = app.input_take();
            if !message.trim().is_empty() {
                Some(message)
            } else {
                None
            }
        }
        // Alt+Up: scroll messages up
        Input {
            key: Key::Up,
            alt: true,
            ..
        } => {
            app.scroll_up();
            None
        }
        // Alt+Down: scroll messages down
        Input {
            key: Key::Down,
            alt: true,
            ..
        } => {
            app.scroll_down();
            None
        }
        // Forward all other inputs to TextArea for normal editing
        input => {
            app.input.input(input);
            None
        }
    }
}
