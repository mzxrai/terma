use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_key_event(app: &mut crate::app::App, key: KeyEvent) -> Option<String> {
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.quit();
            None
        }
        KeyCode::Char(c) => {
            app.input_push(c);
            None
        }
        KeyCode::Backspace => {
            app.input_pop();
            None
        }
        KeyCode::Enter => {
            let message = app.input_take();
            if !message.trim().is_empty() {
                Some(message)
            } else {
                None
            }
        }
        KeyCode::Up => {
            app.scroll_up();
            None
        }
        KeyCode::Down => {
            app.scroll_down();
            None
        }
        _ => None,
    }
}
