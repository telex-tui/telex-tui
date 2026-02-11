//! Example 30: Image Widget
//!
//! Demonstrates the image widget for displaying images using the Kitty
//! graphics protocol.
//!
//! Features:
//! - Display PNG, JPEG, and GIF images
//! - Embed images at compile time with include_bytes!
//! - Load images from file path at runtime
//!
//! NOTE: Requires a Kitty-protocol compatible terminal:
//! - Kitty
//! - Ghostty
//! - WezTerm
//!
//! Run with: cargo run -p telex-tui --example 30_image

use crossterm::event::KeyCode;
use telex::prelude::*;

telex::require_api!(0, 1);

fn main() {
    telex::run_with_theme(App, telex::theme::Theme::nord()).unwrap();
}

struct App;

impl Component for App {
    fn render(&self, cx: Scope) -> View {
        let show_help = state!(cx, || false);

        // F1 toggles help
        cx.use_command(
            KeyBinding::key(KeyCode::F(1)),
            with!(show_help => move || show_help.update(|v| *v = !*v)),
        );
        View::vstack()
            .spacing(1)
            .child(
                View::styled_text("Image Widget Demo (Kitty Graphics)")
                    .bold()
                    .build(),
            )
            .child(View::text("Requires Kitty, Ghostty, or WezTerm terminal"))
            .child(View::text(""))
            // Load image from file path
            .child(View::text("Logo (from file path):"))
            .child(View::image().file("assets/telex-tui.png").build())
            .child(View::text(""))
            .child(View::styled_text("F1 help • Ctrl+Q quit").dim().build())
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 30: Image")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• Image display via Kitty protocol"))
                            .child(View::text("• PNG/JPEG/GIF support"))
                            .child(View::text("• Loaded from file path"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• View::image() displays images"))
                            .child(View::text("• .file(\"path\") loads from disk"))
                            .child(View::text("• .bytes(data) for embedded images"))
                            .child(View::text("• Works in Kitty/Ghostty/WezTerm"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Run in compatible terminal"))
                            .child(View::text("• See the Telex logo rendered"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("→ 31_animated_canvas: animations"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
