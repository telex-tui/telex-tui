//! Image widget for displaying images using the Kitty graphics protocol.
//!
//! Supports PNG, JPEG, and GIF formats. Kitty handles format detection
//! and GIF animation natively.
//!
//! # Example
//!
//! ```rust,ignore
//! // Embed image at compile time
//! View::image()
//!     .data(include_bytes!("logo.png"))
//!     .build()
//!
//! // Or load from file at runtime
//! View::image()
//!     .file("assets/animation.gif")
//!     .build()
//! ```

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use std::sync::atomic::{AtomicU32, Ordering};

/// Source of image data.
#[derive(Clone)]
pub enum ImageSource {
    /// Raw image bytes (PNG, JPEG, GIF, etc.)
    Data(Vec<u8>),
    /// Path to image file (loaded at render time)
    File(String),
}

/// A pending image to be rendered via Kitty protocol.
#[derive(Clone)]
pub struct PendingImage {
    /// Cell column position
    pub cell_x: u16,
    /// Cell row position
    pub cell_y: u16,
    /// Image data
    pub data: Vec<u8>,
    /// Unique ID for this image
    pub id: u32,
    /// Width in cells (for placeholder sizing)
    pub cell_width: u16,
    /// Height in cells (for placeholder sizing)
    pub cell_height: u16,
}

/// Generate a unique image ID.
pub fn next_image_id() -> u32 {
    static NEXT_ID: AtomicU32 = AtomicU32::new(10000); // Start high to avoid collision with canvas IDs
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

/// Encode raw image file data as Kitty graphics protocol escape sequences.
///
/// Uses `f=0` for auto-detection of format (PNG, JPEG, GIF, etc.).
/// Kitty handles GIF animation natively.
pub fn encode_kitty_image(data: &[u8], cell_x: u16, cell_y: u16, image_id: u32) -> String {
    if data.is_empty() {
        return String::new();
    }

    // Encode image data as base64
    let b64_data = BASE64.encode(data);

    // Kitty protocol uses chunked transmission for large images
    const CHUNK_SIZE: usize = 4096;

    let mut result = String::new();
    let chunks: Vec<&str> = b64_data
        .as_bytes()
        .chunks(CHUNK_SIZE)
        .map(|c| std::str::from_utf8(c).unwrap_or(""))
        .collect();

    let total_chunks = chunks.len();

    for (i, chunk) in chunks.iter().enumerate() {
        let is_first = i == 0;
        let is_last = i == total_chunks - 1;
        let more = if is_last { 0 } else { 1 };

        if is_first {
            // First chunk includes full header
            // a=T means transmit and display
            // f=100 means PNG format (Kitty auto-detects from data)
            // t=d means direct (data follows in payload)
            // i=ID for image identification
            // q=2 suppresses responses
            result.push_str(&format!(
                "\x1b_Ga=T,f=100,i={},t=d,m={},q=2;{}\x1b\\",
                image_id, more, chunk
            ));
        } else {
            // Continuation chunks
            result.push_str(&format!("\x1b_Gm={};{}\x1b\\", more, chunk));
        }
    }

    // Position the image at the specified cell
    result.insert_str(0, &format!("\x1b[{};{}H", cell_y + 1, cell_x + 1));

    result
}

/// Try to detect image dimensions from file header.
/// Returns (width, height) in pixels, or None if unknown.
pub fn detect_image_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    // GIF: GIF87a or GIF89a (check first since it has the shortest header)
    if data.len() >= 10 && (data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a")) {
        let width = u16::from_le_bytes([data[6], data[7]]) as u32;
        let height = u16::from_le_bytes([data[8], data[9]]) as u32;
        return Some((width, height));
    }

    // PNG: 89 50 4E 47 0D 0A 1A 0A
    // PNG dimensions are at bytes 16-23 (width: 16-19, height: 20-23)
    if data.len() >= 24 && data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        let width = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
        let height = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
        return Some((width, height));
    }

    // JPEG: FF D8 FF
    if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        // JPEG dimensions require parsing markers - simplified approach
        // Look for SOF0 (0xFFC0) or SOF2 (0xFFC2) marker
        let mut i = 2;
        while i + 9 < data.len() {
            if data[i] == 0xFF {
                let marker = data[i + 1];
                if marker == 0xC0 || marker == 0xC2 {
                    // SOF marker found: height at i+5, width at i+7
                    let height = u16::from_be_bytes([data[i + 5], data[i + 6]]) as u32;
                    let width = u16::from_be_bytes([data[i + 7], data[i + 8]]) as u32;
                    return Some((width, height));
                }
                // Skip to next marker
                if i + 3 < data.len() {
                    let len = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
                    i += 2 + len;
                } else {
                    break;
                }
            } else {
                i += 1;
            }
        }
    }

    None
}

/// Estimate cell dimensions from pixel dimensions.
/// Assumes roughly 10 pixels per cell width and 20 pixels per cell height.
pub fn pixels_to_cells(width: u32, height: u32) -> (u16, u16) {
    let cell_width = ((width as f32) / 10.0).ceil() as u16;
    let cell_height = ((height as f32) / 20.0).ceil() as u16;
    (cell_width.max(1), cell_height.max(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_png_detection() {
        // Minimal PNG header with dimensions 100x50
        let mut png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        png_data.extend_from_slice(&[0, 0, 0, 13]); // IHDR length
        png_data.extend_from_slice(b"IHDR");
        png_data.extend_from_slice(&100u32.to_be_bytes()); // width
        png_data.extend_from_slice(&50u32.to_be_bytes()); // height

        assert_eq!(detect_image_dimensions(&png_data), Some((100, 50)));
    }

    #[test]
    fn test_gif_detection() {
        // Minimal GIF header with dimensions 200x100
        let mut gif_data = b"GIF89a".to_vec();
        gif_data.extend_from_slice(&200u16.to_le_bytes()); // width
        gif_data.extend_from_slice(&100u16.to_le_bytes()); // height

        assert_eq!(detect_image_dimensions(&gif_data), Some((200, 100)));
    }

    #[test]
    fn test_pixels_to_cells() {
        assert_eq!(pixels_to_cells(100, 100), (10, 5));
        assert_eq!(pixels_to_cells(50, 20), (5, 1));
        assert_eq!(pixels_to_cells(1, 1), (1, 1));
    }

    #[test]
    fn test_next_image_id_increments() {
        let id1 = next_image_id();
        let id2 = next_image_id();
        assert!(id2 > id1);
    }
}
