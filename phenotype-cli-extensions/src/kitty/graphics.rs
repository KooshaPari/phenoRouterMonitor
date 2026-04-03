//! Kitty Graphics Protocol implementation
//!
//! Supports displaying images in Kitty-compatible terminals.

use std::io::Write;

/// Display an image in the terminal using Kitty graphics protocol
pub fn display_image(data: &[u8], width: Option<u32>, height: Option<u32>) -> std::io::Result<()> {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    
    // Kitty graphics escape sequence
    write!(handle, "\x1b_G")?;
    
    if let (Some(w), Some(h)) = (width, height) {
        write!(handle, "a=T,f=100,s={},v={},", w, h)?;
    }
    
    // Base64 encode the image data
    let encoded = base64::encode(data);
    write!(handle, "m=1;{}\x1b\\", encoded)?;
    
    handle.flush()
}

/// Clear all graphics from terminal
pub fn clear_graphics() -> std::io::Result<()> {
    print!("\x1b_Ga=d,d=A\x1b\\");
    Ok(())
}
