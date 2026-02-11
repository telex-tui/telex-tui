//! Ollama API client for streaming chat completions.
//!
//! Ollama runs locally - no API key needed.
//! Install: https://ollama.ai
//! Default endpoint: http://localhost:11434

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

/// Request body for the Ollama chat API.
#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ApiMessage>,
    stream: bool,
}

/// Streaming response chunk from Ollama (NDJSON format).
#[derive(Deserialize)]
struct StreamChunk {
    message: Option<ChunkMessage>,
    done: bool,
    error: Option<String>,
}

#[derive(Deserialize)]
struct ChunkMessage {
    content: String,
}

/// Stream a chat completion from the Ollama API.
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
            content: prompt.clone(),
        });

        let request = ChatRequest {
            model: config.model.clone(),
            messages,
            stream: true,
        };

        let base_url = config
            .base_url
            .as_deref()
            .unwrap_or("http://localhost:11434");
        let url = format!("{}/api/chat", base_url);

        let response = ureq::post(&url)
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
                eprintln!("Ollama not available ({}), using mock response", e);
                Self::mock(&prompt)
            }
        }
    }

    fn mock(prompt: &str) -> Self {
        let mock_ndjson = mock_response(prompt);
        Self {
            reader: Some(BufReader::new(Box::new(std::io::Cursor::new(
                mock_ndjson.into_bytes(),
            )))),
            done: false,
        }
    }
}

/// Generate mock NDJSON response in Ollama's format.
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

    // Generate Ollama NDJSON format (one line per word)
    let words: Vec<String> = response
        .split_whitespace()
        .map(|w| format!("{} ", w))
        .collect();

    let mut ndjson: String = words
        .into_iter()
        .map(|word| {
            format!(
                "{{\"message\":{{\"content\":\"{}\"}},\"done\":false}}\n",
                word.replace('"', "\\\"").replace('\n', "\\n")
            )
        })
        .collect();

    ndjson.push_str("{\"message\":{\"content\":\"\"},\"done\":true}\n");
    ndjson
}

fn parse_ndjson_line(line: &str) -> Option<String> {
    let chunk: StreamChunk = match serde_json::from_str(line) {
        Ok(c) => c,
        Err(_) => return None,
    };

    if let Some(error) = chunk.error {
        eprintln!("Ollama error: {}", error);
        return None;
    }

    if chunk.done {
        return None;
    }

    chunk.message.map(|m| m.content)
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
                    if trimmed.is_empty() {
                        continue;
                    }

                    if let Some(text) = parse_ndjson_line(trimmed) {
                        if !text.is_empty() {
                            return Some(text);
                        }
                    }

                    // Check if stream is done
                    if let Ok(chunk) = serde_json::from_str::<StreamChunk>(trimmed) {
                        if chunk.done {
                            self.done = true;
                            return None;
                        }
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
