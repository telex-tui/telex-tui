//! telex-ai - AI chat CLI built with Telex
//!
//! The flagship demo app showcasing Telex's streaming capabilities.
//! Supports multiple LLM providers: Anthropic, OpenAI, Gemini, Ollama.
//!
//! Configure via ~/.config/telex-ai/config.json or environment variables.
//! See README for details.

use telex::prelude::*;
use telex::{Color, VStackNode};

mod anthropic;
mod config;
mod debug;
mod gemini;
mod ollama;
mod openai;

use config::{Config, Provider};

#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        $crate::debug::log(&format!($($arg)*))
    };
}

/// Route to the appropriate provider based on config.
fn stream_chat(
    config: &Config,
    prompt: &str,
    history: &[Message],
) -> Box<dyn Iterator<Item = String> + Send + 'static> {
    match config.provider {
        Provider::Anthropic => anthropic::stream_chat(config, prompt, history),
        Provider::OpenAI => openai::stream_chat(config, prompt, history),
        Provider::Gemini => gemini::stream_chat(config, prompt, history),
        Provider::Ollama => ollama::stream_chat(config, prompt, history),
    }
}

fn main() {
    debug::init();
    debug_log!("config: {:?}", Config::load());
    telex::run_with_theme(App, telex::theme::Theme::nord()).unwrap();
}

/// A chat message in the conversation.
#[derive(Clone)]
struct Message {
    role: Role,
    content: String,
}

#[derive(Clone, Copy, PartialEq)]
enum Role {
    User,
    Assistant,
}

/// Main application component.
struct App;

impl Component for App {
    fn render(&self, cx: Scope) -> View {
        let messages = state!(cx, Vec::<Message>::new);
        let input = state!(cx, String::new);
        let request_id = state!(cx, || 0u32);
        let last_started_id = state!(cx, || 0u32);
        let config = state!(cx, Config::load);

        // Detect when we need to start a new stream
        let needs_restart = request_id.get() != last_started_id.get();

        // Stream for AI responses
        let current_request = request_id.get();
        let msgs_for_stream = messages.get();
        let current_config = config.get();
        let stream = cx.use_text_stream_with_restart(needs_restart, move || {
            if current_request > 0 {
                // Find the last user message to use as prompt
                if let Some(last_user_msg) =
                    msgs_for_stream.iter().rev().find(|m| m.role == Role::User)
                {
                    return stream_chat(&current_config, &last_user_msg.content, &msgs_for_stream);
                }
            }
            // No active request, return empty iterator
            Box::new(std::iter::empty())
        });

        // Mark that we've started this request
        if needs_restart {
            last_started_id.set(request_id.get());
        }

        // Check if streaming is done and update messages
        if stream.is_done() && request_id.get() > 0 && request_id.get() == last_started_id.get() {
            let response = stream.get();
            if !response.is_empty() {
                let mut msgs = messages.get();
                // Only add if we haven't already (check last message isn't this response)
                let should_add = msgs
                    .last()
                    .map(|m| m.role != Role::Assistant || m.content != response)
                    .unwrap_or(true);
                if should_add {
                    debug::log_response_end();
                    msgs.push(Message {
                        role: Role::Assistant,
                        content: response,
                    });
                    messages.set(msgs);
                }
            }
        }

        // Handle send action
        let send = with!(input, messages, request_id, last_started_id => move || {
            let text = input.get().trim().to_string();
            // Only send if not empty and not currently streaming
            if !text.is_empty() && request_id.get() == last_started_id.get() {
                debug::log_user_input(&text);
                let mut msgs = messages.get();
                msgs.push(Message {
                    role: Role::User,
                    content: text,
                });
                messages.set(msgs);
                input.set(String::new());
                // Increment request_id to trigger a new stream
                request_id.set(request_id.get() + 1);
            }
        });

        // Check if we're actively streaming (request started but not done)
        let is_streaming = request_id.get() > last_started_id.get() || stream.is_streaming();
        let current_response = stream.get();

        let on_change = with!(input => move |s| input.set(s));

        // Build the messages view
        let messages_view = build_messages_view(&messages.get(), is_streaming, &current_response);

        // Provider/model for header
        let cfg = config.get();
        let header_text = format!("{} ({})", cfg.provider.name(), cfg.model);

        // Build the full layout using builders (not macro) for the dynamic parts
        View::vstack()
            .child(
                View::hstack()
                    .child(
                        View::styled_text("telex-ai")
                            .bold()
                            .color(Color::Cyan)
                            .build(),
                    )
                    .child(View::text(" │ "))
                    .child(View::styled_text(&header_text).color(Color::Yellow).build())
                    .build(),
            )
            .child(
                View::boxed()
                    .border(true)
                    .flex(1)
                    .padding(1)
                    .auto_scroll_bottom(true)
                    .child(messages_view)
                    .build(),
            )
            .child(
                View::boxed()
                    .border(true)
                    .max_height(3)
                    .child(
                        View::text_input()
                            .value(input.get())
                            .placeholder("Type a message and press Enter...")
                            .on_change(on_change)
                            .on_submit(send)
                            .focused(true)
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}

/// Build the messages view.
fn build_messages_view(messages: &[Message], is_streaming: bool, current_response: &str) -> View {
    let mut children: Vec<View> = messages.iter().map(render_message).collect();

    // Add streaming response
    if is_streaming {
        // Render markdown for the streaming content, append cursor
        let content_view = if current_response.is_empty() {
            View::text("\u{258c}")
        } else {
            // Render markdown and add cursor at the end
            let md_view = telex::markdown::render(current_response);
            View::vstack()
                .child(md_view)
                .child(View::text("\u{258c}"))
                .build()
        };

        children.push(
            View::vstack()
                .child(
                    View::styled_text("Assistant")
                        .bold()
                        .color(Color::Green)
                        .build(),
                )
                .child(content_view)
                .child(View::text(""))
                .build(),
        );
    }

    if children.is_empty() {
        View::styled_text("Type a message and press Enter to send")
            .dim()
            .build()
    } else {
        View::VStack(VStackNode {
            children,
            spacing: 0,
            justify: Justify::Start,
            align: Align::Start,
            layout_mode: LayoutMode::Flex,
        })
    }
}

/// Render a single message.
fn render_message(msg: &Message) -> View {
    let (label, color) = match msg.role {
        Role::User => ("You", Color::Blue),
        Role::Assistant => ("Assistant", Color::Green),
    };

    // Use markdown rendering for assistant messages
    let content_view = match msg.role {
        Role::Assistant => telex::markdown::render(&msg.content),
        Role::User => View::text(&msg.content),
    };

    View::vstack()
        .child(View::styled_text(label).bold().color(color).build())
        .child(content_view)
        .child(View::text(""))
        .build()
}
