//! Google Gemini API client for streaming chat completions.

use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};

use crate::config::Config;
use crate::{Message, Role};

/// Content part for Gemini API.
#[derive(Serialize)]
struct Part {
    text: String,
}

/// A message/content block for Gemini API.
#[derive(Serialize)]
struct Content {
    role: &'static str,
    parts: Vec<Part>,
}

/// Request body for Gemini generateContent API.
#[derive(Serialize)]
struct GenerateRequest {
    contents: Vec<Content>,
}

/// Streaming response chunk from Gemini.
#[derive(Deserialize)]
struct StreamChunk {
    candidates: Option<Vec<Candidate>>,
    error: Option<GeminiError>,
}

#[derive(Deserialize)]
struct Candidate {
    content: Option<CandidateContent>,
    #[serde(rename = "finishReason")]
    _finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct CandidateContent {
    parts: Option<Vec<TextPart>>,
}

#[derive(Deserialize)]
struct TextPart {
    text: Option<String>,
}

#[derive(Deserialize)]
struct GeminiError {
    message: String,
}

/// Stream a chat completion from the Gemini API.
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

        // Build contents array (Gemini's format for conversation)
        let mut contents: Vec<Content> = history
            .iter()
            .map(|m| Content {
                role: match m.role {
                    Role::User => "user",
                    Role::Assistant => "model",
                },
                parts: vec![Part {
                    text: m.content.clone(),
                }],
            })
            .collect();

        // Add current prompt
        contents.push(Content {
            role: "user",
            parts: vec![Part { text: prompt }],
        });

        let request = GenerateRequest { contents };

        let base_url = config
            .base_url
            .as_deref()
            .unwrap_or("https://generativelanguage.googleapis.com");
        let url = format!(
            "{}/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
            base_url, config.model, api_key
        );

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
                eprintln!("Gemini API error: {}", e);
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

/// Generate mock SSE response in Gemini's format.
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

    // Generate Gemini-format SSE (one chunk per word)
    let words: Vec<String> = response
        .split_whitespace()
        .map(|w| format!("{} ", w))
        .collect();

    words
        .into_iter()
        .map(|word| {
            format!(
                "data: {{\"candidates\":[{{\"content\":{{\"parts\":[{{\"text\":\"{}\"}}]}}}}]}}\n\n",
                word.replace('"', "\\\"").replace('\n', "\\n")
            )
        })
        .collect()
}

fn parse_sse_line(line: &str) -> Option<String> {
    if !line.starts_with("data: ") {
        return None;
    }

    let json_str = &line[6..];
    crate::debug_log!("gemini raw: {}", json_str);

    let chunk: StreamChunk = match serde_json::from_str(json_str) {
        Ok(c) => c,
        Err(e) => {
            crate::debug_log!("gemini parse error: {}", e);
            return None;
        }
    };

    if let Some(error) = chunk.error {
        crate::debug_log!("gemini api error: {}", error.message);
        eprintln!("Gemini API error: {}", error.message);
        return None;
    }

    let text = chunk
        .candidates?
        .into_iter()
        .next()?
        .content?
        .parts?
        .into_iter()
        .next()?
        .text;

    if let Some(ref t) = text {
        crate::debug_log!("gemini token: {:?}", t);
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
                    let line = line.trim();

                    if let Some(text) = parse_sse_line(line) {
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
