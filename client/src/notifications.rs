#[cfg(target_os = "macos")]
use notify_rust::Notification;

#[cfg(target_os = "macos")]
pub fn send_notification(username: &str, message: &str) {
    let _ = Notification::new()
        .summary(&format!("Terma: {}", username))
        .body(message)
        .show();
}

#[cfg(not(target_os = "macos"))]
pub fn send_notification(_username: &str, _message: &str) {
    // No-op on non-macOS platforms
}
