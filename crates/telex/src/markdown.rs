//! Markdown rendering for Telex.
//!
//! Converts markdown text into View nodes for display.

use crate::theme::current_theme;
use crate::view::{Align, BoxNode, HStackNode, Justify, LayoutMode, TextNode, VStackNode, View};
use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};

/// Render markdown text into a View.
///
/// Handles common markdown elements:
/// - Paragraphs
/// - Headers (h1-h6)
/// - Code blocks (fenced and indented)
/// - Inline code
/// - Bold and italic text
/// - Lists (ordered and unordered)
/// - Blockquotes
///
/// # Example
/// ```ignore
/// let view = telex::markdown::render("# Hello\n\nThis is **bold** text.");
/// ```
pub fn render(markdown: &str) -> View {
    let parser = Parser::new(markdown);
    let renderer = MarkdownRenderer::new();
    renderer.render(parser)
}

/// Internal state for markdown rendering.
struct MarkdownRenderer {
    /// Stack of views being built (for nested structures).
    view_stack: Vec<Vec<View>>,
    /// Current inline text being accumulated.
    current_text: String,
    /// Current inline styles.
    bold: bool,
    italic: bool,
    /// Are we inside a code block?
    in_code_block: bool,
    /// Code block content accumulator.
    code_block_content: String,
    /// Code block language (if specified).
    code_block_lang: Option<String>,
    /// Current list item content.
    in_list_item: bool,
    /// List markers for nested lists.
    list_markers: Vec<ListMarker>,
    /// Blockquote depth.
    blockquote_depth: usize,
}

#[derive(Clone)]
enum ListMarker {
    Unordered,
    Ordered(u64),
}

impl MarkdownRenderer {
    fn new() -> Self {
        Self {
            view_stack: vec![vec![]],
            current_text: String::new(),
            bold: false,
            italic: false,
            in_code_block: false,
            code_block_content: String::new(),
            code_block_lang: None,
            in_list_item: false,
            list_markers: vec![],
            blockquote_depth: 0,
        }
    }

    fn render(mut self, parser: Parser) -> View {
        for event in parser {
            self.handle_event(event);
        }

        // Flush any remaining text
        self.flush_text();

        // Build final view from top-level children
        let children = self.view_stack.pop().unwrap_or_default();
        if children.is_empty() {
            View::Empty
        } else if children.len() == 1 {
            children.into_iter().next().unwrap()
        } else {
            View::VStack(VStackNode {
                children,
                spacing: 0,
                justify: Justify::Start,
                align: Align::Stretch,
                layout_mode: LayoutMode::Flex,
            })
        }
    }

    fn handle_event(&mut self, event: Event) {
        match event {
            // Block-level start tags
            Event::Start(Tag::Paragraph) => {
                self.flush_text();
            }
            Event::Start(Tag::Heading { level, .. }) => {
                self.flush_text();
                // Headers are bold by default
                self.bold = true;
                // Could also set color based on level
                let _ = level; // TODO: vary styling by level
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                self.flush_text();
                self.in_code_block = true;
                self.code_block_content.clear();
                self.code_block_lang = match kind {
                    CodeBlockKind::Fenced(lang) => {
                        let lang = lang.to_string();
                        if lang.is_empty() {
                            None
                        } else {
                            Some(lang)
                        }
                    }
                    CodeBlockKind::Indented => None,
                };
            }
            Event::Start(Tag::List(start)) => {
                self.flush_text();
                let marker = match start {
                    Some(n) => ListMarker::Ordered(n),
                    None => ListMarker::Unordered,
                };
                self.list_markers.push(marker);
            }
            Event::Start(Tag::Item) => {
                self.flush_text();
                self.in_list_item = true;
            }
            Event::Start(Tag::BlockQuote) => {
                self.flush_text();
                self.blockquote_depth += 1;
            }

            // Inline style tags
            Event::Start(Tag::Strong) => {
                // TODO: nested inline styles - currently we just set the flag
                // If we're already in another style, this will override
                self.bold = true;
            }
            Event::Start(Tag::Emphasis) => {
                // TODO: nested inline styles
                self.italic = true;
            }
            Event::Code(text) => {
                // Inline code - flush current text, add code styled, continue
                self.flush_text();
                self.push_inline_code(&text);
            }

            // Text content
            Event::Text(text) => {
                if self.in_code_block {
                    self.code_block_content.push_str(&text);
                } else {
                    self.current_text.push_str(&text);
                }
            }
            Event::SoftBreak => {
                if self.in_code_block {
                    self.code_block_content.push('\n');
                } else {
                    // Preserve newlines in TUI - unlike web markdown which collapses to space
                    self.current_text.push('\n');
                }
            }
            Event::HardBreak => {
                if self.in_code_block {
                    self.code_block_content.push('\n');
                } else {
                    self.flush_text();
                }
            }

            // Block-level end tags
            Event::End(TagEnd::Paragraph) => {
                self.flush_text();
                self.push_spacing();
            }
            Event::End(TagEnd::Heading(_)) => {
                self.flush_text();
                self.bold = false;
                self.push_spacing();
            }
            Event::End(TagEnd::CodeBlock) => {
                self.push_code_block();
                self.in_code_block = false;
                self.code_block_content.clear();
                self.code_block_lang = None;
                self.push_spacing();
            }
            Event::End(TagEnd::List(_)) => {
                self.list_markers.pop();
            }
            Event::End(TagEnd::Item) => {
                self.flush_list_item();
                self.in_list_item = false;
            }
            Event::End(TagEnd::BlockQuote) => {
                self.blockquote_depth = self.blockquote_depth.saturating_sub(1);
            }

            // Inline style end tags
            Event::End(TagEnd::Strong) => {
                // TODO: nested inline styles - proper stack-based approach
                self.bold = false;
            }
            Event::End(TagEnd::Emphasis) => {
                // TODO: nested inline styles
                self.italic = false;
            }

            // Ignored events (for now)
            Event::Start(Tag::Link { .. })
            | Event::End(TagEnd::Link)
            | Event::Start(Tag::Image { .. })
            | Event::End(TagEnd::Image) => {
                // TODO: links and images
            }

            _ => {}
        }
    }

    /// Flush accumulated text as a styled text node.
    fn flush_text(&mut self) {
        if self.current_text.is_empty() {
            return;
        }

        let text = std::mem::take(&mut self.current_text);
        let theme = current_theme();

        let view = View::Text(TextNode {
            content: text,
            color: Some(theme.foreground),
            bg_color: None,
            bold: self.bold,
            italic: self.italic,
            underline: false,
            dim: false,
        });

        self.push_view(view);
    }

    /// Push inline code as a styled segment.
    fn push_inline_code(&mut self, text: &str) {
        let theme = current_theme();

        // Inline code: different color, maybe background
        let view = View::Text(TextNode {
            content: format!("`{}`", text),
            color: Some(theme.primary),
            bg_color: None,
            bold: false,
            italic: false,
            underline: false,
            dim: false,
        });

        self.push_view(view);
    }

    /// Push a code block.
    fn push_code_block(&mut self) {
        let content = self.code_block_content.trim_end().to_string();
        let theme = current_theme();

        // Code block: boxed with border
        let text_view = View::Text(TextNode {
            content,
            color: Some(theme.foreground),
            bg_color: None,
            bold: false,
            italic: false,
            underline: false,
            dim: false,
        });

        let code_box = View::Box(BoxNode {
            child: Some(Box::new(text_view)),
            border: true,
            padding: 1,
            flex: 0,
            scroll: false,
            auto_scroll_bottom: false,
            focusable: false,
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
        });

        self.push_view(code_box);
    }

    /// Flush a list item with its marker.
    fn flush_list_item(&mut self) {
        self.flush_text();

        // Get the appropriate marker
        let marker = match self.list_markers.last_mut() {
            Some(ListMarker::Unordered) => "  • ".to_string(),
            Some(ListMarker::Ordered(n)) => {
                let marker = format!("{:>2}. ", n);
                *n += 1;
                marker
            }
            None => "  • ".to_string(),
        };

        // Indent based on list depth
        let indent = "    ".repeat(self.list_markers.len().saturating_sub(1));

        // Get the content that was just pushed
        if let Some(views) = self.view_stack.last_mut() {
            if let Some(last_view) = views.pop() {
                // Wrap content in a flex box so it takes remaining width after marker
                let content_box = View::Box(BoxNode {
                    child: Some(Box::new(last_view)),
                    border: false,
                    padding: 0,
                    flex: 1,
                    scroll: false,
                    auto_scroll_bottom: false,
                    focusable: false,
                    min_width: None,
                    max_width: None,
                    min_height: None,
                    max_height: None,
                });

                // Wrap with marker
                let marked_view = View::HStack(HStackNode {
                    children: vec![View::text(format!("{}{}", indent, marker)), content_box],
                    spacing: 0,
                    justify: Justify::Start,
                    align: Align::Start,
                    layout_mode: LayoutMode::Flex,
                });
                views.push(marked_view);
            }
        }
    }

    /// Push an empty line for spacing between blocks.
    fn push_spacing(&mut self) {
        // Add a blank line between blocks
        self.push_view(View::text(""));
    }

    /// Push a view onto the current stack level.
    fn push_view(&mut self, view: View) {
        if let Some(views) = self.view_stack.last_mut() {
            views.push(view);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_text() {
        let view = render("Hello world");
        assert!(matches!(view, View::VStack(_) | View::Text(_)));
    }

    #[test]
    fn test_code_block() {
        let view = render("```rust\nfn main() {}\n```");
        // Should produce a view containing a Box (code block)
        assert!(matches!(view, View::VStack(_)));
    }

    #[test]
    fn test_bold_text() {
        let view = render("This is **bold** text");
        assert!(matches!(view, View::VStack(_)));
    }
}
