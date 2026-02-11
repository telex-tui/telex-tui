# telex-ai

A multi-provider AI chat app built with Telex. Streams responses token-by-token with markdown rendering.

Supports Anthropic, OpenAI, Google Gemini, and Ollama.

## Quick start

The fastest way in is Gemini's free tier:

1. Get an API key from [Google AI Studio](https://aistudio.google.com/apikey)
2. Run:
   ```bash
   export GEMINI_API_KEY="your-key"
   cargo run -p chat
   ```

No API key at all? It falls back to mock responses so you can still see the UI.

## Configuration

Create `~/.config/telex-ai/config.json`:

```json
{
  "model": "gemini/gemini-2.5-flash"
}
```

The `model` field uses `provider/model` format:

| Provider | Prefix | Example | Env var |
|----------|--------|---------|---------|
| Anthropic | `anthropic/` or `claude/` | `anthropic/claude-sonnet-4-20250514` | `ANTHROPIC_API_KEY` |
| OpenAI | `openai/` or `gpt/` | `openai/gpt-4o` | `OPENAI_API_KEY` |
| Google | `gemini/` or `google/` | `gemini/gemini-2.5-flash` | `GEMINI_API_KEY` |
| Ollama | `ollama/` or `local/` | `ollama/llama3.2` | *(none)* |

You can also just specify a model name (e.g., `"model": "gpt-4o"`) and the provider will be inferred.

### Advanced

```json
{
  "model": "openai/gpt-4o",
  "api_key": "sk-...",
  "base_url": "https://api.openai.com"
}
```

The `base_url` field lets you point at any OpenAI-compatible API (LM Studio, Together, Groq, etc.).

### Without a config file

Auto-detects from environment variables, checking: Anthropic, OpenAI, Gemini, Ollama (in that order). If nothing is found and Ollama isn't running, falls back to mock responses.

## Controls

- Type a message and press **Enter** to send
- **Ctrl+Q** to quit

## How it works

The app uses Telex's `use_text_stream_with_restart` hook to stream LLM responses. Each provider implements SSE or NDJSON parsing to yield tokens as they arrive. Assistant messages are rendered as markdown.

```
examples/chat/src/
├── main.rs        # UI component, state management
├── config.rs      # Config file + env var loading
├── anthropic.rs   # Anthropic streaming client
├── openai.rs      # OpenAI streaming client
├── gemini.rs      # Gemini streaming client
├── ollama.rs      # Ollama streaming client
└── debug.rs       # Optional debug logging
```
