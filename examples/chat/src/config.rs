//! Configuration management for telex-ai.
//!
//! Loads settings from `~/.config/telex-ai/config.json` with env var fallbacks.

use serde::Deserialize;
use std::path::PathBuf;

/// Application configuration.
#[derive(Debug, Clone)]
pub struct Config {
    pub provider: Provider,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

/// Supported providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    Anthropic,
    OpenAI,
    Gemini,
    Ollama,
}

/// Raw config file format.
#[derive(Deserialize, Default)]
struct ConfigFile {
    /// Model string in "provider/model" format, e.g. "gemini/gemini-1.5-flash"
    model: Option<String>,
    /// Optional API key (prefer env vars for security)
    api_key: Option<String>,
    /// Optional base URL for OpenAI-compatible APIs
    base_url: Option<String>,
}

impl Config {
    /// Load configuration from file and environment.
    pub fn load() -> Self {
        let file_config = Self::load_file().unwrap_or_default();

        // Parse model string (e.g., "gemini/gemini-1.5-flash")
        let (provider, model) = if let Some(model_str) = &file_config.model {
            parse_model_string(model_str)
        } else {
            // Auto-detect from env vars
            detect_from_env()
        };

        // Get API key: config file -> env var
        let api_key = file_config
            .api_key
            .or_else(|| std::env::var(provider.env_var()).ok());

        Config {
            provider,
            model,
            api_key,
            base_url: file_config.base_url,
        }
    }

    fn load_file() -> Option<ConfigFile> {
        let path = config_path()?;
        let contents = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&contents).ok()
    }
}

impl Provider {
    /// Environment variable name for this provider's API key.
    pub fn env_var(&self) -> &'static str {
        match self {
            Provider::Anthropic => "ANTHROPIC_API_KEY",
            Provider::OpenAI => "OPENAI_API_KEY",
            Provider::Gemini => "GEMINI_API_KEY",
            Provider::Ollama => "OLLAMA_HOST",
        }
    }

    /// Human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            Provider::Anthropic => "Anthropic",
            Provider::OpenAI => "OpenAI",
            Provider::Gemini => "Gemini",
            Provider::Ollama => "Ollama",
        }
    }

    /// Default model for this provider.
    pub fn default_model(&self) -> &'static str {
        match self {
            Provider::Anthropic => "claude-sonnet-4-20250514",
            Provider::OpenAI => "gpt-4o",
            Provider::Gemini => "gemini-2.5-flash",
            Provider::Ollama => "llama3.2",
        }
    }
}

/// Get the config file path: ~/.config/telex-ai/config.json
fn config_path() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(PathBuf::from(home).join(".config/telex-ai/config.json"))
}

/// Parse a "provider/model" string.
fn parse_model_string(s: &str) -> (Provider, String) {
    if let Some((provider_str, model)) = s.split_once('/') {
        let provider = match provider_str.to_lowercase().as_str() {
            "anthropic" | "claude" => Provider::Anthropic,
            "openai" | "gpt" => Provider::OpenAI,
            "gemini" | "google" => Provider::Gemini,
            "ollama" | "local" => Provider::Ollama,
            _ => Provider::Ollama, // Default to ollama for unknown
        };
        (provider, model.to_string())
    } else {
        // No slash - assume it's just a model name, try to infer provider
        let provider = infer_provider_from_model(s);
        (provider, s.to_string())
    }
}

/// Try to infer provider from model name.
fn infer_provider_from_model(model: &str) -> Provider {
    let m = model.to_lowercase();
    if m.starts_with("claude") {
        Provider::Anthropic
    } else if m.starts_with("gpt") || m.starts_with("o1") {
        Provider::OpenAI
    } else if m.starts_with("gemini") {
        Provider::Gemini
    } else {
        Provider::Ollama
    }
}

/// Detect provider from environment variables.
fn detect_from_env() -> (Provider, String) {
    if std::env::var("ANTHROPIC_API_KEY").is_ok() {
        (
            Provider::Anthropic,
            Provider::Anthropic.default_model().to_string(),
        )
    } else if std::env::var("OPENAI_API_KEY").is_ok() {
        (
            Provider::OpenAI,
            Provider::OpenAI.default_model().to_string(),
        )
    } else if std::env::var("GEMINI_API_KEY").is_ok() {
        (
            Provider::Gemini,
            Provider::Gemini.default_model().to_string(),
        )
    } else {
        (
            Provider::Ollama,
            Provider::Ollama.default_model().to_string(),
        )
    }
}
