//! Example 06: Log Viewer
//!
//! Demonstrates auto-scrolling streaming text, simulating a log tail.
//!
//! Run with: cargo run -p telex-tui --example 06_log_viewer

use crossterm::event::KeyCode;
use crossterm::style::Color;
use std::time::Duration;
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

        // Stream log entries
        let logs = cx.use_text_stream(|| {
            let log_messages = vec![
                "[INFO]  Application started",
                "[INFO]  Loading configuration...",
                "[OK]    Config loaded successfully",
                "[INFO]  Connecting to database...",
                "[OK]    Database connected",
                "[INFO]  Starting web server on :8080",
                "[OK]    Server listening",
                "[INFO]  Processing request GET /api/users",
                "[OK]    Response 200 in 45ms",
                "[INFO]  Processing request POST /api/login",
                "[OK]    Response 200 in 120ms",
                "[WARN]  High memory usage detected: 85%",
                "[INFO]  Running garbage collection...",
                "[OK]    Memory freed: 200MB",
                "[INFO]  Processing request GET /api/data",
                "[ERROR] Database timeout after 5000ms",
                "[INFO]  Retrying database connection...",
                "[OK]    Database reconnected",
                "[OK]    Response 200 in 5045ms",
                "[INFO]  Scheduled backup starting...",
                "[OK]    Backup completed: 1.2GB",
                "[INFO]  Processing request GET /api/health",
                "[OK]    Response 200 in 12ms",
                "[INFO]  Processing request PUT /api/users/123",
                "[OK]    Response 200 in 89ms",
                "[INFO]  Cache invalidation triggered",
                "[OK]    Cache cleared: 50MB",
                "[WARN]  Slow query detected: 1200ms",
                "[INFO]  Query optimization suggested",
                "[INFO]  Processing request DELETE /api/sessions",
                "[OK]    Response 204 in 34ms",
                "[INFO]  SSL certificate check",
                "[OK]    Certificate valid for 45 days",
                "[INFO]  Processing request POST /api/upload",
                "[OK]    File uploaded: 15MB",
                "[WARN]  Disk usage at 78%",
                "[INFO]  Processing request GET /api/reports",
                "[OK]    Response 200 in 230ms",
                "[INFO]  Metrics exported to monitoring",
                "[OK]    Heartbeat sent successfully",
                "------- Log stream completed -------",
            ];

            log_messages.into_iter().map(|msg| {
                std::thread::sleep(Duration::from_millis(500));
                format!("{}\n", msg)
            })
        });

        let log_text = logs.get();
        let is_streaming = logs.is_loading();

        // Color the status indicator
        let status = if is_streaming {
            View::styled_text(" [LIVE]")
                .color(Color::Green)
                .bold()
                .build()
        } else {
            View::styled_text(" [END]").dim().build()
        };

        View::vstack()
            .child(
                View::hstack()
                    .child(
                        View::styled_text("Log Viewer")
                            .color(Color::Cyan)
                            .bold()
                            .build(),
                    )
                    .child(status)
                    .build(),
            )
            .child(View::gap(1))
            .child(
                View::boxed()
                    .scroll(true)
                    .auto_scroll_bottom(true)
                    .min_height(15)
                    .max_height(15)
                    .child(View::text(&log_text))
                    .build(),
            )
            .child(View::gap(1))
            .child(View::styled_text("F1 help • Ctrl+Q quit").dim().build())
            .child(
                View::modal()
                    .visible(show_help.get())
                    .title("Example 06: Log Viewer")
                    .on_dismiss(with!(show_help => move || show_help.set(false)))
                    .child(
                        View::vstack()
                            .child(View::styled_text("What you're seeing").bold().build())
                            .child(View::text("• cx.use_text_stream() for accumulating text"))
                            .child(View::text("• Auto-scrolling box that follows new content"))
                            .child(View::text("• Live/End indicator based on stream state"))
                            .child(View::gap(1))
                            .child(View::styled_text("Key concepts").bold().build())
                            .child(View::text("• use_text_stream concatenates yielded strings"))
                            .child(View::text(
                                "• auto_scroll_bottom(true) keeps newest visible",
                            ))
                            .child(View::text("• Simulates tailing a log file"))
                            .child(View::gap(1))
                            .child(View::styled_text("Try this").bold().build())
                            .child(View::text("• Watch the [LIVE] indicator change to [END]"))
                            .child(View::text("• Notice auto-scroll keeps up with new entries"))
                            .child(View::gap(1))
                            .child(View::styled_text("Next up").bold().build())
                            .child(View::text("→ 07_file_browser: real filesystem navigation"))
                            .child(View::gap(1))
                            .child(View::styled_text("Press Escape to close").dim().build())
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}
