use anyhow::{Context, Result};
use std::io::{self, Write};

/// Write the provided text to the terminal clipboard using OSC-52.
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    if text.is_empty() {
        return Ok(());
    }

    let encoded = base64_encode(text.as_bytes());
    let mut stdout = io::stdout();
    write!(stdout, "\x1b]52;c;{}\x07", encoded).context("failed to emit OSC-52 sequence")?;
    stdout.flush().ok();
    Ok(())
}

fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut encoded = String::with_capacity(((data.len() + 2) / 3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0];
        let b1 = *chunk.get(1).unwrap_or(&0);
        let b2 = *chunk.get(2).unwrap_or(&0);

        encoded.push(TABLE[(b0 >> 2) as usize] as char);
        encoded.push(TABLE[((b0 & 0b0000_0011) << 4 | (b1 >> 4)) as usize] as char);

        if chunk.len() > 1 {
            encoded.push(TABLE[((b1 & 0b0000_1111) << 2 | (b2 >> 6)) as usize] as char);
        } else {
            encoded.push('=');
        }

        if chunk.len() > 2 {
            encoded.push(TABLE[(b2 & 0b0011_1111) as usize] as char);
        } else {
            encoded.push('=');
        }
    }

    encoded
}
