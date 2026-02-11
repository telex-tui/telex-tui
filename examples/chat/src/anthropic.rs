//! Anthropic API client for streaming chat completions.

use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};

use crate::config::Config;
use crate::{Message, Role};

/// A message in the chat history (for API).
#[derive(Serialize)]
struct ApiMessage {
    role: &'static str,
    content: String,
}

/// Request body for the Anthropic messages API.
#[derive(Serialize)]
struct MessagesRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ApiMessage>,
    stream: bool,
}

/// Event types from the SSE stream.
#[derive(Deserialize)]
#[serde(tag = "type")]
#[allow(dead_code)] // Fields needed for serde deserialization
enum StreamEvent {
    #[serde(rename = "message_start")]
    MessageStart { message: MessageInfo },
    #[serde(rename = "content_block_start")]
    ContentBlockStart { content_block: ContentBlock },
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta { delta: Delta },
    #[serde(rename = "content_block_stop")]
    ContentBlockStop {},
    #[serde(rename = "message_delta")]
    MessageDelta { delta: MessageDeltaInfo },
    #[serde(rename = "message_stop")]
    MessageStop {},
    #[serde(rename = "ping")]
    Ping {},
    #[serde(rename = "error")]
    Error { error: ApiError },
}

#[derive(Deserialize)]
struct MessageInfo {
    #[allow(dead_code)]
    id: String,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[allow(dead_code)]
    text: Option<String>,
}

#[derive(Deserialize)]
struct Delta {
    text: Option<String>,
}

#[derive(Deserialize)]
struct MessageDeltaInfo {
    #[allow(dead_code)]
    stop_reason: Option<String>,
}

#[derive(Deserialize)]
struct ApiError {
    message: String,
}

/// Stream a chat completion from the Anthropic API.
pub fn stream_chat(
    config: &Config,
    prompt: &str,
    history: &[Message],
) -> Box<dyn Iterator<Item = String> + Send + 'static> {
    Box::new(StreamingChat::new(
        config.clone(),
        prompt.to_string(),
        history.to_vec(),
    ))
}

/// Iterator that streams tokens from the Anthropic API.
struct StreamingChat {
    reader: Option<BufReader<Box<dyn std::io::Read + Send>>>,
    done: bool,
}

impl StreamingChat {
    fn new(config: Config, prompt: String, history: Vec<Message>) -> Self {
        let api_key = match &config.api_key {
            Some(key) => key.clone(),
            None => return Self::mock(&prompt),
        };

        // Build message history
        let mut messages: Vec<ApiMessage> = history
            .iter()
            .map(|m| ApiMessage {
                role: match m.role {
                    Role::User => "user",
                    Role::Assistant => "assistant",
                },
                content: m.content.clone(),
            })
            .collect();

        // Add current prompt
        messages.push(ApiMessage {
            role: "user",
            content: prompt,
        });

        let request = MessagesRequest {
            model: config.model.clone(),
            max_tokens: 16384, // Anthropic requires max_tokens; set high for streaming
            messages,
            stream: true,
        };

        let base_url = config
            .base_url
            .as_deref()
            .unwrap_or("https://api.anthropic.com");
        let url = format!("{}/v1/messages", base_url);

        let response = ureq::post(&url)
            .set("x-api-key", &api_key)
            .set("anthropic-version", "2023-06-01")
            .set("content-type", "application/json")
            .send_json(&request);

        match response {
            Ok(resp) => {
                let reader = resp.into_reader();
                Self {
                    reader: Some(BufReader::new(Box::new(reader))),
                    done: false,
                }
            }
            Err(e) => {
                eprintln!("API error: {}", e);
                Self {
                    reader: None,
                    done: true,
                }
            }
        }
    }

    /// Create a mock streaming response for testing without API key.
    fn mock(prompt: &str) -> Self {
        let mock_sse = mock_response(prompt).unwrap_or_default();
        let cursor = std::io::Cursor::new(mock_sse.into_bytes());
        Self {
            reader: Some(BufReader::new(Box::new(cursor))),
            done: false,
        }
    }
}

/// Generate a mock SSE response string for testing without API keys.
/// Used by all providers as a fallback.
pub fn mock_response(prompt: &str) -> Option<String> {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Seed based on prompt + time for variety
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as usize)
        .unwrap_or(0);

    let responses = [
        format!(
            "That's a great question about **{}**!\n\n\
            Let me think about this carefully. The answer involves several aspects:\n\n\
            1. First, consider the *fundamentals*\n\
            2. Then, explore the implications\n\
            3. Finally, draw conclusions\n\n\
            Here's a code example:\n\n\
            ```rust\nfn main() {{\n    println!(\"Hello!\");\n}}\n```\n\n\
            The key insight is to approach it step by step.",
            prompt
        ),
        format!(
            "Ah, **{}**! This is fascinating.\n\n\
            Did you know that `inline code` can highlight technical terms?\n\n\
            Here are some key points:\n\n\
            - The universe is full of *wonderful mysteries*\n\
            - Each discovery leads to **more questions**\n\
            - Learning never stops\n\n\
            Let me show you an example:\n\n\
            ```python\ndef explore():\n    return \"knowledge\"\n```",
            prompt
        ),
        format!(
            "I'd be happy to help with **{}**!\n\n\
            Here's my perspective:\n\n\
            1. Start with the basics\n\
            2. Build understanding incrementally\n\
            3. Apply what you learn\n\n\
            A simple example in code:\n\n\
            ```javascript\nconst answer = 42;\nconsole.log(answer);\n```\n\n\
            The *key* is to stay curious!",
            prompt
        ),
        format!(
            "Great question about **{}**!\n\n\
            There are multiple perspectives to consider:\n\n\
            - Some say *this*\n\
            - Others argue **that**\n\
            - The truth is often `somewhere in between`\n\n\
            Here's how you might approach it:\n\n\
            ```\nStep 1: Observe\nStep 2: Analyze\nStep 3: Conclude\n```",
            prompt
        ),
        format!(
            "Interesting topic: **{}**\n\n\
            This is one of my *favorite* subjects. Let me share:\n\n\
            1. The short answer is **nuanced**\n\
            2. The long answer is even more fascinating\n\n\
            Code to illustrate:\n\n\
            ```rust\nlet insight = \"clarity\";\nprintln!(\"{{}}\", insight);\n```\n\n\
            Hope that helps!",
            prompt
        ),
    ];

    let response = &responses[seed % responses.len()];

    // Create a mock reader that yields the response word by word
    let words: Vec<String> = response
        .split_whitespace()
        .map(|w| format!("{} ", w))
        .collect();

    let mock_sse: String = words
        .into_iter()
        .map(|word| {
            format!(
                "event: content_block_delta\ndata: {{\"type\":\"content_block_delta\",\"delta\":{{\"text\":\"{}\"}}}}\n\n",
                word.replace('"', "\\\"")
            )
        })
        .chain(std::iter::once(
            "event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n".to_string(),
        ))
        .collect();

    Some(mock_sse)
}

/// Parse a single SSE line and extract any text delta.
fn parse_sse_line(line: &str) -> Option<String> {
    if !line.starts_with("data: ") {
        return None;
    }

    let json_str = &line[6..];
    if json_str == "[DONE]" {
        return None;
    }

    crate::debug_log!("anthropic raw: {}", json_str);

    let event: StreamEvent = match serde_json::from_str(json_str) {
        Ok(e) => e,
        Err(e) => {
            crate::debug_log!("anthropic parse error: {}", e);
            return None;
        }
    };

    match event {
        StreamEvent::ContentBlockDelta { delta } => {
            if let Some(ref t) = delta.text {
                crate::debug_log!("anthropic token: {:?}", t);
            }
            delta.text
        }
        StreamEvent::Error { error } => {
            crate::debug_log!("anthropic api error: {}", error.message);
            eprintln!("API error: {}", error.message);
            None
        }
        _ => None,
    }
}

impl Iterator for StreamingChat {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let reader = self.reader.as_mut()?;
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    // EOF
                    self.done = true;
                    return None;
                }
                Ok(_) => {
                    let line = line.trim();

                    // Check for message_stop event
                    if line.contains("message_stop") {
                        self.done = true;
                        return None;
                    }

                    // Try to parse as SSE data line
                    if let Some(text) = parse_sse_line(line) {
                        return Some(text);
                    }
                    // Otherwise continue reading
                }
                Err(_) => {
                    self.done = true;
                    return None;
                }
            }

            // Small delay to show streaming effect (real APIs have natural latency)
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    }
}
