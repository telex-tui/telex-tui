//! OpenAI API client for streaming chat completions.

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

/// Request body for the OpenAI chat completions API.
#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ApiMessage>,
    stream: bool,
}

/// Streaming response chunk from OpenAI.
#[derive(Deserialize)]
struct StreamChunk {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    delta: Delta,
    _finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct Delta {
    content: Option<String>,
}

/// Stream a chat completion from the OpenAI API.
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

        let request = ChatRequest {
            model: config.model.clone(),
            messages,
            stream: true,
        };

        let base_url = config
            .base_url
            .as_deref()
            .unwrap_or("https://api.openai.com");
        let url = format!("{}/v1/chat/completions", base_url);

        let response = ureq::post(&url)
            .set("Authorization", &format!("Bearer {}", api_key))
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
                eprintln!("OpenAI API error: {}", e);
                Self {
                    reader: None,
                    done: true,
                }
            }
        }
    }

    fn mock(prompt: &str) -> Self {
        let mock_sse = mock_response(prompt);
        Self {
            reader: Some(BufReader::new(Box::new(std::io::Cursor::new(
                mock_sse.into_bytes(),
            )))),
            done: false,
        }
    }
}

/// Generate mock SSE response in OpenAI's format.
fn mock_response(prompt: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

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
            The key insight is to approach it step by step.",
            prompt
        ),
        format!(
            "I'd be happy to help with **{}**!\n\n\
            Here's my perspective:\n\n\
            1. Start with the basics\n\
            2. Build understanding incrementally\n\
            3. Apply what you learn\n\n\
            The *key* is to stay curious!",
            prompt
        ),
    ];

    let response = &responses[seed % responses.len()];

    // Generate OpenAI-format SSE (one chunk per word)
    let words: Vec<String> = response
        .split_whitespace()
        .map(|w| format!("{} ", w))
        .collect();

    let mut sse: String = words
        .into_iter()
        .map(|word| {
            format!(
                "data: {{\"choices\":[{{\"delta\":{{\"content\":\"{}\"}}}}]}}\n\n",
                word.replace('"', "\\\"").replace('\n', "\\n")
            )
        })
        .collect();

    sse.push_str("data: [DONE]\n\n");
    sse
}

fn parse_sse_line(line: &str) -> Option<String> {
    if !line.starts_with("data: ") {
        return None;
    }

    let json_str = &line[6..];
    if json_str == "[DONE]" {
        return None;
    }

    crate::debug_log!("openai raw: {}", json_str);

    let chunk: StreamChunk = match serde_json::from_str(json_str) {
        Ok(c) => c,
        Err(e) => {
            crate::debug_log!("openai parse error: {}", e);
            return None;
        }
    };

    let text = chunk.choices.into_iter().next()?.delta.content;
    if let Some(ref t) = text {
        crate::debug_log!("openai token: {:?}", t);
    }
    text
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
                    self.done = true;
                    return None;
                }
                Ok(_) => {
                    let trimmed = line.trim();

                    // Check for done signal
                    if trimmed == "data: [DONE]" {
                        self.done = true;
                        return None;
                    }

                    if let Some(text) = parse_sse_line(trimmed) {
                        return Some(text);
                    }
                }
                Err(_) => {
                    self.done = true;
                    return None;
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    }
}
