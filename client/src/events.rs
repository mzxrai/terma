use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_textarea::{Input, Key};

pub fn handle_key_event(app: &mut crate::app::App, key: KeyEvent) -> Option<String> {
    if matches!(key.code, KeyCode::Char('c'))
        && (key.modifiers.contains(KeyModifiers::CONTROL)
            || key.modifiers.contains(KeyModifiers::SUPER))
    {
        if app.has_selection() {
            if let Err(err) = app.copy_selection() {
                app.add_system_message(format!("Copy failed: {}", err));
            }
            return None;
        }
    }

    // Convert crossterm KeyEvent to tui_textarea Input first
    let input = Input::from(key);

    // Match on the tui_textarea Input struct to properly detect Shift+Enter
    match input {
        // Ctrl+C: quit application
        Input {
            key: Key::Char('c'),
            ctrl: true,
            shift: false,
            ..
        } => {
            app.quit();
            None
        }
        // Shift+Enter or Ctrl+J: insert newline (multi-line message)
        // Note: Many terminals send Shift+Enter as Ctrl+J
        Input {
            key: Key::Enter,
            shift: true,
            ..
        }
        | Input {
            key: Key::Char('j'),
            ctrl: true,
            ..
        } => {
            app.input.insert_newline();
            None
        }
        // Plain Enter (without shift): send message
        Input {
            key: Key::Enter,
            shift: false,
            ..
        } => {
            let message = app.input_take();
            if !message.trim().is_empty() {
                Some(message)
            } else {
                None
            }
        }
        // Ctrl+M: also send message
        Input {
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
