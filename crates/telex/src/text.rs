//! Unicode-aware text handling for Telex.
//!
//! This module provides the foundation for proper text handling:
//! - Grapheme cluster awareness (user-perceived characters)
//! - Display width calculation (handles wide characters like CJK, emoji)
//! - Soft wrapping (visual-only, never modifies content)
//! - Cursor positioning (grapheme-indexed, not byte-indexed)

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// A position in text measured in grapheme clusters.
///
/// This is the correct unit for cursor positioning - it represents
/// user-perceived characters, not bytes or code points.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GraphemeIndex(pub usize);

impl GraphemeIndex {
    pub fn new(index: usize) -> Self {
        Self(index)
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }
}

impl From<usize> for GraphemeIndex {
    fn from(index: usize) -> Self {
        Self(index)
    }
}

impl From<GraphemeIndex> for usize {
    fn from(index: GraphemeIndex) -> usize {
        index.0
    }
}

/// Get the display width of a string in terminal columns.
///
/// Handles:
/// - ASCII: 1 column each
/// - CJK characters: 2 columns each
/// - Emoji: typically 2 columns
/// - Zero-width characters: 0 columns
pub fn display_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

/// Get the display width of a single grapheme cluster.
pub fn grapheme_width(grapheme: &str) -> usize {
    UnicodeWidthStr::width(grapheme)
}

/// Count the number of grapheme clusters in a string.
pub fn grapheme_count(s: &str) -> usize {
    s.graphemes(true).count()
}

/// Iterate over grapheme clusters in a string.
pub fn graphemes(s: &str) -> impl Iterator<Item = &str> {
    s.graphemes(true)
}

/// A visual line produced by soft wrapping.
///
/// Contains the byte range into the original string and display info.
#[derive(Debug, Clone)]
pub struct VisualLine<'a> {
    /// The text content of this visual line
    pub text: &'a str,
    /// Starting grapheme index in the original line
    pub grapheme_start: usize,
    /// Ending grapheme index (exclusive) in the original line
    pub grapheme_end: usize,
    /// Display width of this line in columns
    pub width: usize,
}

/// Soft-wrap a single line of text to fit within a given width.
///
/// This is **visual-only** wrapping - it returns references into the original
/// string and never modifies content. The original text is unchanged.
///
/// Wrapping strategy: **Character wrap** - characters flow to where they fit.
/// When a character doesn't fit on the current line, it goes to the next line.
/// No word rearrangement, no jumping.
///
/// # Arguments
/// * `text` - A single line of text (should not contain newlines)
/// * `width` - Maximum display width in columns
///
/// # Returns
/// A vector of `VisualLine`s representing how the text should be displayed.
pub fn soft_wrap_line(text: &str, width: usize) -> Vec<VisualLine<'_>> {
    if width == 0 {
        return vec![];
    }

    // Fast path: if text fits, return as-is
    let text_width = display_width(text);
    if text_width <= width {
        return vec![VisualLine {
            text,
            grapheme_start: 0,
            grapheme_end: grapheme_count(text),
            width: text_width,
        }];
    }

    // Character wrap: fill each line until it's full, then continue on next line
    let mut lines = Vec::new();
    let mut current_start_byte = 0;
    let mut current_start_grapheme = 0;
    let mut current_width = 0;
    let mut grapheme_idx = 0;

    for (byte_idx, grapheme) in text.grapheme_indices(true) {
        let g_width = grapheme_width(grapheme);

        // Check if this grapheme would exceed the width
        if current_width + g_width > width {
            // This grapheme doesn't fit - finish current line, start new one
            if current_start_byte < byte_idx {
                let line_text = &text[current_start_byte..byte_idx];
                lines.push(VisualLine {
                    text: line_text,
                    grapheme_start: current_start_grapheme,
                    grapheme_end: grapheme_idx,
                    width: current_width,
                });
            }
            // Start new line with this grapheme
            current_start_byte = byte_idx;
            current_start_grapheme = grapheme_idx;
            current_width = g_width;
        } else {
            current_width += g_width;
        }

        grapheme_idx += 1;
    }

    // Don't forget the last line
    if current_start_byte < text.len() {
        let line_text = &text[current_start_byte..];
        lines.push(VisualLine {
            text: line_text,
            grapheme_start: current_start_grapheme,
            grapheme_end: grapheme_idx,
            width: display_width(line_text),
        });
    }

    // Handle empty string case
    if lines.is_empty() {
        lines.push(VisualLine {
            text: "",
            grapheme_start: 0,
            grapheme_end: 0,
            width: 0,
        });
    }

    lines
}

/// Soft-wrap multi-line text, preserving existing line breaks.
///
/// This wraps each logical line (separated by '\n') independently.
/// The '\n' characters in the content are **preserved** - this function
/// only adds visual breaks, never modifies the content.
pub fn soft_wrap(text: &str, width: usize) -> Vec<VisualLine<'_>> {
    if width == 0 {
        return vec![];
    }

    let mut result = Vec::new();
    let mut line_grapheme_offset = 0;

    for line in text.split('\n') {
        let wrapped = soft_wrap_line(line, width);
        for mut visual_line in wrapped {
            // Adjust grapheme indices to be relative to the full text
            visual_line.grapheme_start += line_grapheme_offset;
            visual_line.grapheme_end += line_grapheme_offset;
            result.push(visual_line);
        }
        // Account for the newline character (1 grapheme)
        line_grapheme_offset += grapheme_count(line) + 1;
    }

    if result.is_empty() {
        result.push(VisualLine {
            text: "",
            grapheme_start: 0,
            grapheme_end: 0,
            width: 0,
        });
    }

    result
}

/// Convert a grapheme index to screen coordinates (column, row) given a width.
///
/// This accounts for soft wrapping to determine which visual line and column
/// the cursor should appear at.
///
/// # Arguments
/// * `text` - The full text content
/// * `grapheme_idx` - Cursor position as a grapheme index
/// * `width` - Display width for wrapping
///
/// # Returns
/// (column, row) where both are 0-indexed
pub fn cursor_to_screen(text: &str, grapheme_idx: usize, width: usize) -> (u16, u16) {
    if width == 0 {
        return (0, 0);
    }

    let mut row = 0u16;
    let mut grapheme_offset = 0;

    for line in text.split('\n') {
        let line_graphemes = grapheme_count(line);

        // Check if cursor is within this logical line
        if grapheme_idx <= grapheme_offset + line_graphemes {
            // Cursor is in this line - now find which visual line
            let cursor_in_line = grapheme_idx - grapheme_offset;
            let wrapped = soft_wrap_line(line, width);

            for visual_line in &wrapped {
                if cursor_in_line < visual_line.grapheme_end {
                    // Cursor is in this visual line
                    let col_grapheme = cursor_in_line - visual_line.grapheme_start;
                    // Calculate display column by measuring width of graphemes before cursor
                    let col = graphemes(visual_line.text)
                        .take(col_grapheme)
                        .map(grapheme_width)
                        .sum::<usize>() as u16;
                    return (col, row);
                }
                row += 1;
            }
            // Cursor at end of line
            if let Some(last) = wrapped.last() {
                let col = last.width as u16;
                return (col, row.saturating_sub(1));
            }
        }

        grapheme_offset += line_graphemes + 1; // +1 for newline
        row += soft_wrap_line(line, width).len() as u16;
    }

    // Cursor past end of text
    (0, row.saturating_sub(1))
}

/// Convert screen coordinates (column, row) to a grapheme index.
///
/// This is the inverse of `cursor_to_screen`.
///
/// # Arguments
/// * `text` - The full text content
/// * `col` - Screen column (0-indexed)
/// * `row` - Screen row (0-indexed)
/// * `width` - Display width for wrapping
///
/// # Returns
/// The grapheme index at or nearest to the given screen position
pub fn screen_to_cursor(text: &str, col: u16, row: u16, width: usize) -> usize {
    if width == 0 {
        return 0;
    }

    let mut current_row = 0u16;
    let mut grapheme_offset = 0;

    for line in text.split('\n') {
        let wrapped = soft_wrap_line(line, width);

        for visual_line in &wrapped {
            if current_row == row {
                // Found the target row - now find the column
                let mut current_col = 0usize;

                for (grapheme_in_line, grapheme) in graphemes(visual_line.text).enumerate() {
                    let g_width = grapheme_width(grapheme);
                    if current_col + g_width > col as usize {
                        // Click is on this grapheme
                        return grapheme_offset + visual_line.grapheme_start + grapheme_in_line;
                    }
                    current_col += g_width;
                }

                // Click past end of line
                return grapheme_offset + visual_line.grapheme_end;
            }
            current_row += 1;
        }

        grapheme_offset += grapheme_count(line) + 1; // +1 for newline
    }

    // Click below text
    grapheme_count(text)
}

/// Get the byte offset for a given grapheme index.
///
/// Returns None if the index is out of bounds.
pub fn grapheme_to_byte_offset(text: &str, grapheme_idx: usize) -> Option<usize> {
    text.grapheme_indices(true)
        .nth(grapheme_idx)
        .map(|(byte_idx, _)| byte_idx)
        .or_else(|| {
            // Index might be at the end
            if grapheme_idx == grapheme_count(text) {
                Some(text.len())
            } else {
                None
            }
        })
}

/// Get the grapheme index for a given byte offset.
///
/// Returns the grapheme that contains or starts at this byte offset.
pub fn byte_to_grapheme_offset(text: &str, byte_offset: usize) -> usize {
    text.grapheme_indices(true)
        .take_while(|(idx, _)| *idx < byte_offset)
        .count()
}

/// Insert a string at a grapheme index, returning the new string.
pub fn insert_at_grapheme(text: &str, grapheme_idx: usize, insert: &str) -> String {
    let byte_offset = grapheme_to_byte_offset(text, grapheme_idx).unwrap_or(text.len());
    let mut result = String::with_capacity(text.len() + insert.len());
    result.push_str(&text[..byte_offset]);
    result.push_str(insert);
    result.push_str(&text[byte_offset..]);
    result
}

/// Remove the grapheme at the given index, returning the new string.
///
/// Returns None if the index is out of bounds.
pub fn remove_at_grapheme(text: &str, grapheme_idx: usize) -> Option<String> {
    let mut graphemes: Vec<&str> = text.graphemes(true).collect();
    if grapheme_idx >= graphemes.len() {
        return None;
    }
    graphemes.remove(grapheme_idx);
    Some(graphemes.concat())
}

/// Calculate the wrapped height of text (number of visual lines).
pub fn wrapped_height(text: &str, width: usize) -> usize {
    if width == 0 {
        return 0;
    }
    soft_wrap(text, width).len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_width_ascii() {
        assert_eq!(display_width("hello"), 5);
        assert_eq!(display_width(""), 0);
    }

    #[test]
    fn test_display_width_cjk() {
        assert_eq!(display_width("中"), 2);
        assert_eq!(display_width("中文"), 4);
        assert_eq!(display_width("hello中文"), 9); // 5 + 4
    }

    #[test]
    fn test_grapheme_count() {
        assert_eq!(grapheme_count("hello"), 5);
        assert_eq!(grapheme_count("é"), 1); // Single grapheme (composed)
        assert_eq!(grapheme_count("e\u{0301}"), 1); // e + combining acute = 1 grapheme
    }

    #[test]
    fn test_soft_wrap_fits() {
        let lines = soft_wrap_line("hello", 10);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].text, "hello");
    }

    #[test]
    fn test_soft_wrap_character_wrap() {
        // Character wrap: text flows to where it fits, no word rearrangement
        let lines = soft_wrap_line("hello world", 8);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].text, "hello wo"); // 8 chars fit
        assert_eq!(lines[1].text, "rld"); // rest flows to next line
    }

    #[test]
    fn test_soft_wrap_force_break() {
        let lines = soft_wrap_line("abcdefghij", 4);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].text, "abcd");
        assert_eq!(lines[1].text, "efgh");
        assert_eq!(lines[2].text, "ij");
    }

    #[test]
    fn test_cursor_to_screen_simple() {
        let text = "hello";
        assert_eq!(cursor_to_screen(text, 0, 80), (0, 0));
        assert_eq!(cursor_to_screen(text, 2, 80), (2, 0));
        assert_eq!(cursor_to_screen(text, 5, 80), (5, 0));
    }

    #[test]
    fn test_cursor_to_screen_multiline() {
        let text = "hello\nworld";
        assert_eq!(cursor_to_screen(text, 0, 80), (0, 0));
        assert_eq!(cursor_to_screen(text, 5, 80), (5, 0)); // End of "hello"
        assert_eq!(cursor_to_screen(text, 6, 80), (0, 1)); // Start of "world"
        assert_eq!(cursor_to_screen(text, 8, 80), (2, 1)); // "wo|rld"
    }

    #[test]
    fn test_insert_at_grapheme() {
        assert_eq!(insert_at_grapheme("hello", 0, "X"), "Xhello");
        assert_eq!(insert_at_grapheme("hello", 2, "X"), "heXllo");
        assert_eq!(insert_at_grapheme("hello", 5, "X"), "helloX");
    }

    #[test]
    fn test_remove_at_grapheme() {
        assert_eq!(remove_at_grapheme("hello", 0), Some("ello".to_string()));
        assert_eq!(remove_at_grapheme("hello", 2), Some("helo".to_string()));
        assert_eq!(remove_at_grapheme("hello", 4), Some("hell".to_string()));
        assert_eq!(remove_at_grapheme("hello", 5), None);
    }

    // ==========================================================================
    // Tests that prove the auto-wrap bug fix
    // ==========================================================================

    #[test]
    fn test_soft_wrap_returns_references_not_copies() {
        // This proves soft_wrap doesn't modify content - it returns slices
        // into the original string. The content is NEVER modified.
        let original = "here is my text before I resize the screen";
        let wrapped = soft_wrap_line(original, 20);

        // Each visual line's text is a slice of the original
        for line in &wrapped {
            // Verify text is a substring of original
            assert!(original.contains(line.text));
        }

        // Character wrap: concatenating all visual lines directly gives us
        // back the original content (no spaces added/removed)
        let reconstructed: String = wrapped.iter().map(|l| l.text).collect::<Vec<_>>().concat();
        assert_eq!(reconstructed, original);
    }

    #[test]
    fn test_soft_wrap_no_newlines_inserted() {
        // The original bug: typing would insert '\n' into content.
        // This test proves soft_wrap NEVER adds newlines.
        let content = "here is my text before I resize the screen it is lovely";

        // Wrap at narrow width
        let wrapped_narrow = soft_wrap_line(content, 20);
        assert!(wrapped_narrow.len() > 1, "Should wrap at width 20");

        // Wrap at wide width
        let wrapped_wide = soft_wrap_line(content, 80);
        assert_eq!(wrapped_wide.len(), 1, "Should not wrap at width 80");

        // CRITICAL: The content itself has NO newlines
        assert!(!content.contains('\n'), "Content must not contain newlines");

        // Visual lines don't contain newlines either
        for line in &wrapped_narrow {
            assert!(
                !line.text.contains('\n'),
                "Visual line must not contain newlines"
            );
        }
    }

    #[test]
    fn test_resize_reflows_correctly() {
        // The original bug: text wrapped at width 40 would have '\n' baked in,
        // so resizing to 80 wouldn't reflow - lines stayed short.
        //
        // This test proves: same content, different widths = different visual lines.
        let content = "here is my text before I resize the screen";

        // "Narrow window" - 20 cols
        let at_20 = soft_wrap_line(content, 20);

        // "Wide window" - 60 cols
        let at_60 = soft_wrap_line(content, 60);

        // "Very wide window" - 80 cols
        let at_80 = soft_wrap_line(content, 80);

        // Narrow wraps more
        assert!(
            at_20.len() > at_60.len(),
            "Narrower width should have more visual lines"
        );
        assert!(
            at_60.len() >= at_80.len(),
            "Wider width should have fewer visual lines"
        );

        // But content is identical - just different visual presentation
        // (no '\n' characters were added to content)
        assert!(!content.contains('\n'));
    }

    #[test]
    fn test_content_integrity_after_simulated_typing() {
        // Simulate what happens when user types a long string.
        // With the OLD buggy code: content would have '\n' inserted.
        // With the FIX: content stays as one line, wrapping is visual-only.

        let mut content = String::new();

        // Simulate typing 80 characters
        for i in 0..80 {
            content.push(char::from_u32('a' as u32 + (i % 26)).unwrap());
        }

        // Content should be exactly 80 chars, NO newlines
        assert_eq!(content.len(), 80);
        assert_eq!(content.chars().filter(|&c| c == '\n').count(), 0);

        // Even when wrapped at narrow width, content is unchanged
        let visual_lines = soft_wrap_line(&content, 20);
        assert_eq!(
            visual_lines.len(),
            4,
            "80 chars / 20 width = 4 visual lines"
        );

        // Content still has no newlines
        assert!(
            !content.contains('\n'),
            "Content must never be modified by wrapping"
        );
    }

    #[test]
    fn test_cjk_content_wraps_by_display_width() {
        // CJK characters are 2 columns wide.
        // This proves we wrap by DISPLAY width, not byte/char count.
        let content = "中文中文中文"; // 6 CJK chars = 12 display columns

        // Width 6 should fit 3 CJK chars per line
        let wrapped = soft_wrap_line(content, 6);
        assert_eq!(wrapped.len(), 2);
        assert_eq!(display_width(wrapped[0].text), 6);
        assert_eq!(display_width(wrapped[1].text), 6);
    }

    #[test]
    fn test_grapheme_cluster_not_split() {
        // Grapheme clusters (like emoji with modifiers) should not be split
        let content = "hello 👨‍👩‍👧‍👦 world"; // Family emoji is one grapheme

        // Even at narrow width, the emoji stays together
        let wrapped = soft_wrap_line(content, 10);

        // Find which line has the emoji
        let emoji_line = wrapped.iter().find(|l| l.text.contains('👨')).unwrap();

        // The full emoji cluster should be intact
        assert!(emoji_line.text.contains("👨‍👩‍👧‍👦"));
    }

    // ==========================================================================
    // Character wrap specific tests
    // ==========================================================================

    #[test]
    fn test_character_wrap_typing_at_edge() {
        // Simulates: user types "here we are on a thin scre" (26 chars)
        // then types "e" - the "e" should flow to next line
        let before = "here we are on a thin scre";
        let after = "here we are on a thin scree";

        let wrapped_before = soft_wrap_line(before, 26);
        let wrapped_after = soft_wrap_line(after, 26);

        // Before: fits on one line
        assert_eq!(wrapped_before.len(), 1);
        assert_eq!(wrapped_before[0].text, before);

        // After: flows to two lines
        assert_eq!(wrapped_after.len(), 2);
        assert_eq!(wrapped_after[0].text, "here we are on a thin scre");
        assert_eq!(wrapped_after[1].text, "e");
    }

    #[test]
    fn test_character_wrap_typing_continues_on_second_line() {
        // Type "en" after "scre" - both chars should be on line 2
        let content = "here we are on a thin screen";
        let wrapped = soft_wrap_line(content, 26);

        assert_eq!(wrapped.len(), 2);
        assert_eq!(wrapped[0].text, "here we are on a thin scre");
        assert_eq!(wrapped[1].text, "en");
    }

    #[test]
    fn test_resize_wider_reflows_back() {
        // Content that wraps at width 20, but fits at width 40
        let content = "here is my text that wraps";

        let at_20 = soft_wrap_line(content, 20);
        let at_40 = soft_wrap_line(content, 40);

        // At 20: wraps to 2 lines
        assert_eq!(at_20.len(), 2);
        assert_eq!(at_20[0].text, "here is my text that");
        assert_eq!(at_20[1].text, " wraps");

        // At 40: fits on 1 line
        assert_eq!(at_40.len(), 1);
        assert_eq!(at_40[0].text, content);

        // Content unchanged in both cases
        let reconstructed_20: String = at_20.iter().map(|l| l.text).collect();
        let reconstructed_40: String = at_40.iter().map(|l| l.text).collect();
        assert_eq!(reconstructed_20, content);
        assert_eq!(reconstructed_40, content);
    }

    #[test]
    fn test_resize_narrower_reflows_more() {
        let content = "abcdefghijklmnopqrstuvwxyz";

        let at_26 = soft_wrap_line(content, 26);
        let at_13 = soft_wrap_line(content, 13);
        let at_5 = soft_wrap_line(content, 5);

        // At 26: fits on 1 line
        assert_eq!(at_26.len(), 1);

        // At 13: wraps to 2 lines
        assert_eq!(at_13.len(), 2);
        assert_eq!(at_13[0].text, "abcdefghijklm");
        assert_eq!(at_13[1].text, "nopqrstuvwxyz");

        // At 5: wraps to 6 lines
        assert_eq!(at_5.len(), 6);
        assert_eq!(at_5[0].text, "abcde");
        assert_eq!(at_5[5].text, "z");
    }

    #[test]
    fn test_exact_width_no_wrap() {
        // Text exactly matches width - should not wrap
        let content = "12345";
        let wrapped = soft_wrap_line(content, 5);

        assert_eq!(wrapped.len(), 1);
        assert_eq!(wrapped[0].text, "12345");
    }

    #[test]
    fn test_one_over_width_wraps() {
        // Text is exactly one char over - that char wraps
        let content = "123456";
        let wrapped = soft_wrap_line(content, 5);

        assert_eq!(wrapped.len(), 2);
        assert_eq!(wrapped[0].text, "12345");
        assert_eq!(wrapped[1].text, "6");
    }

    #[test]
    fn test_spaces_preserved_in_wrap() {
        // Spaces should be preserved, not eaten by wrap
        let content = "ab cd ef";
        let wrapped = soft_wrap_line(content, 4);

        // "ab c" | "d ef" - spaces preserved
        assert_eq!(wrapped.len(), 2);
        assert_eq!(wrapped[0].text, "ab c");
        assert_eq!(wrapped[1].text, "d ef");

        // Reconstruction gives original
        let reconstructed: String = wrapped.iter().map(|l| l.text).collect();
        assert_eq!(reconstructed, content);
    }

    #[test]
    fn test_grapheme_indices_correct_after_wrap() {
        let content = "abcdefghij";
        let wrapped = soft_wrap_line(content, 4);

        // Line 0: "abcd" graphemes 0-4
        assert_eq!(wrapped[0].grapheme_start, 0);
        assert_eq!(wrapped[0].grapheme_end, 4);

        // Line 1: "efgh" graphemes 4-8
        assert_eq!(wrapped[1].grapheme_start, 4);
        assert_eq!(wrapped[1].grapheme_end, 8);

        // Line 2: "ij" graphemes 8-10
        assert_eq!(wrapped[2].grapheme_start, 8);
        assert_eq!(wrapped[2].grapheme_end, 10);
    }

    #[test]
    fn test_cursor_position_after_wrap() {
        // "abcdefgh" wrapped at width 5 = "abcde" + "fgh"
        // Cursor at grapheme 6 (the 'g') should be at (1, 1) - col 1, row 1
        let content = "abcdefgh";
        let (col, row) = cursor_to_screen(content, 6, 5);

        assert_eq!(row, 1, "Cursor should be on second visual line");
        assert_eq!(col, 1, "Cursor should be at column 1 (after 'f')");
    }

    #[test]
    fn test_cursor_at_wrap_boundary() {
        // Cursor at exactly the wrap point
        let content = "abcdefgh";

        // Cursor at grapheme 5 (the 'f') - first char of second line
        let (col, row) = cursor_to_screen(content, 5, 5);
        assert_eq!(row, 1, "Cursor should be on second line");
        assert_eq!(col, 0, "Cursor should be at start of second line");
    }

    #[test]
    fn test_multiple_resize_cycles() {
        // Simulate resize: 40 -> 20 -> 40 -> 10 -> 40
        // Content should always reconstruct to original
        let content = "the quick brown fox jumps over lazy dog";

        for width in [40, 20, 40, 10, 40, 15, 40] {
            let wrapped = soft_wrap_line(content, width);
            let reconstructed: String = wrapped.iter().map(|l| l.text).collect();
            assert_eq!(
                reconstructed, content,
                "Content corrupted at width {}",
                width
            );
        }
    }

    #[test]
    fn test_emoji_at_wrap_boundary() {
        // User's test case: emojis at boundary
        // Each emoji is 2 display columns wide
        let base = "a b c d e dakl asdl d fox fox badger😊😊😊 😊😊";
        let base_width = display_width(base);

        // Test wrapping at exactly the base width (should fit on one line)
        let wrapped_exact = soft_wrap_line(base, base_width);
        assert_eq!(
            wrapped_exact.len(),
            1,
            "Should fit on one line at exact width"
        );

        // Now add one more emoji (2 cols) - should wrap
        let with_extra = "a b c d e dakl asdl d fox fox badger😊😊😊 😊😊😊";

        // Wrap at original base_width - the extra emoji should go to line 2
        let wrapped_overflow = soft_wrap_line(with_extra, base_width);

        assert_eq!(wrapped_overflow.len(), 2, "Should wrap to two lines");
        assert!(
            wrapped_overflow[1].text.contains('😊'),
            "Second line should have the overflow emoji"
        );

        // Content integrity: reconstruction should match original
        let reconstructed: String = wrapped_overflow.iter().map(|l| l.text).collect();
        assert_eq!(reconstructed, with_extra, "Content must be preserved");
    }

    #[test]
    fn test_wide_char_at_boundary_edge_cases() {
        // Test when a 2-wide character (emoji) would overflow by just 1 column
        // Width 46 means the last emoji (width=2) at position 45-46 doesn't fit

        let text = "a b c d e dakl asdl d fox fox badger😊😊😊 😊😊";
        // Display width = 47
        // Position 45 starts the last 😊 (which needs cols 45-46)

        // At width 47: fits exactly
        let at_47 = soft_wrap_line(text, 47);
        assert_eq!(at_47.len(), 1);

        // At width 46: emoji at position 45 would need cols 45-46, but we only have 0-45
        // So the last emoji should wrap to line 2
        let at_46 = soft_wrap_line(text, 46);
        assert_eq!(at_46.len(), 2);
        assert_eq!(at_46[0].width, 45); // "a b c d e dakl asdl d fox fox badger😊😊😊 😊" = 45
        assert_eq!(at_46[1].text, "😊");

        // At width 45: now the second-to-last emoji also doesn't fit
        let at_45 = soft_wrap_line(text, 45);
        assert_eq!(at_45.len(), 2);

        // Content always preserved
        for width in [47, 46, 45, 44, 43, 42, 41, 40] {
            let wrapped = soft_wrap_line(text, width);
            let reconstructed: String = wrapped.iter().map(|l| l.text).collect();
            assert_eq!(reconstructed, text, "Content corrupted at width {}", width);
        }
    }
}
