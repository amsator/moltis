# LLM Providers

Moltis supports multiple LLM providers through a trait-based architecture.
Configure providers through the web UI or directly in configuration files.

## Currently Available Providers*

| Provider | Auth | Notes |
|----------|------|-------|
| **OpenAI Codex** | OAuth | Codex-focused cloud models |
| **GitHub Copilot** | OAuth | Requires active Copilot subscription |
| **Local LLM** | Local runtime | Runs models on your machine |

| Provider | Models | Tool Calling | Streaming |
|----------|--------|--------------|-----------|
| **Anthropic** | Claude 4, Claude 3.5, Claude 3 | ✅ | ✅ |
| **OpenAI** | GPT-4o, GPT-4, o1, o3 | ✅ | ✅ |
| **Google** | Gemini 2.0, Gemini 1.5 | ✅ | ✅ |
| **GitHub Copilot** | GPT-4o, Claude | ✅ | ✅ |

### Tier 2 (Good Support)

| Provider | Models | Tool Calling | Streaming |
|----------|--------|--------------|-----------|
| **Mistral** | Mistral Large, Codestral | ✅ | ✅ |
| **Groq** | Llama 3, Mixtral | ✅ | ✅ |
| **xAI** | Grok 3, Grok 2 | ✅ | ✅ |
| **Together** | Various open models | ✅ | ✅ |
| **Fireworks** | Various open models | ✅ | ✅ |
| **DeepSeek** | DeepSeek V3, Coder | ✅ | ✅ |

### Tier 3 (Basic Support)

| Provider | Notes |
|----------|-------|
| **OpenRouter** | Aggregator for 100+ models |
| **Ollama** | Local models |
| **Venice** | Privacy-focused |
| **Cerebras** | Fast inference |
| **SambaNova** | Enterprise |
| **Cohere** | Command models |
| **AI21** | Jamba models |

\*More providers are coming soon.

## Configuration

### Via Web UI (Recommended)

1. Open Moltis in your browser.
2. Go to **Settings** -> **Providers**.
3. Choose a provider card.
4. Complete OAuth or enter your API key.
5. Select your preferred model.

### Via Configuration Files

Provider credentials are stored in `~/.config/moltis/provider_keys.json`:

```json
{
  "openai-codex": {
    "model": "gpt-5.2-codex"
  }
}
```

Enable providers in `moltis.toml`:

```toml
[providers]
default = "openai-codex"

[providers.openai-codex]
enabled = true

[providers.github-copilot]
enabled = true

[providers.local]
enabled = true
model = "qwen2.5-coder-7b-q4_k_m"
```

## Provider Setup

### OpenAI Codex

OpenAI Codex uses OAuth token import and OAuth-based access.

1. Go to **Settings** -> **Providers** -> **OpenAI Codex**.
2. Click **Connect** and complete the auth flow.
3. Choose a Codex model.

### GitHub Copilot

GitHub Copilot uses OAuth authentication.

1. Go to **Settings** -> **Providers** -> **GitHub Copilot**.
2. Click **Connect**.
3. Complete the GitHub OAuth flow.

```admonish info
Requires an active GitHub Copilot subscription.
```

### Local LLM

Local LLM runs models directly on your machine.

1. Go to **Settings** -> **Providers** -> **Local LLM**.
2. Choose a model from the local registry or download one.
3. Save and select it as your active model.

### xAI (Grok)

1. Get an API key from [console.x.ai](https://console.x.ai)
2. Enter it in Settings → Providers → xAI

Default models: `grok-3`, `grok-3-fast`, `grok-3-mini`, `grok-3-mini-fast`, `grok-2`, `grok-2-mini`

```json
{
  "xai": {
    "apiKey": "xai-...",
    "model": "grok-3"
  }
}
```

```admonish tip title="Dynamic Model Discovery"
Moltis can fetch the latest available models from xAI's API at startup.
This ensures you always have access to new models as they're released.
```

### Ollama (Local Models)

1. Install Ollama: `curl -fsSL https://ollama.ai/install.sh | sh`
2. Pull a model: `ollama pull llama3.2`
3. Configure in Moltis:

```json
{
  "ollama": {
    "baseUrl": "http://localhost:11434",
    "model": "llama3.2"
  }
}
```

### OpenRouter

Access 100+ models through one API:

1. Get an API key from [openrouter.ai](https://openrouter.ai)
2. Enter it in Settings → Providers → OpenRouter
3. Specify the model ID you want to use

```json
{
  "openrouter": {
    "apiKey": "sk-or-...",
    "model": "anthropic/claude-3.5-sonnet"
  }
}
```

### Generic OpenAI-Compatible

For any OpenAI-compatible API endpoint not explicitly supported:

```json
{
  "openai-compatible": {
    "apiKey": "your-api-key",
    "baseUrl": "https://your-endpoint.example.com/v1",
    "model": "your-model-id"
  }
}
```

```admonish note
Both `baseUrl` and `model` are required for the generic provider. It won't register without explicit configuration.
```

This works with:
- Self-hosted LLM servers (vLLM, text-generation-inference)
- Enterprise API proxies
- Regional API endpoints
- Any service implementing the OpenAI chat completions API

## Custom Base URLs

For providers with custom endpoints (enterprise, proxies):

```json
{
  "openai": {
    "apiKey": "sk-...",
    "baseUrl": "https://your-proxy.example.com/v1",
    "model": "gpt-4o"
  }
}
```

## Switching Models

- **Per session**: Use the model selector in the chat UI.
- **Per message**: Use `/model <name>` in chat.
- **Global default**: Set `[providers].default` and `[agent].model` in `moltis.toml`.

### Default Provider

Set the default in `moltis.toml`:

```toml
[providers]
default = "anthropic"

[agent]
model = "claude-sonnet-4-20250514"
```

## Model Capabilities

Different models have different strengths:

| Use Case | Recommended Model |
|----------|-------------------|
| General coding | Claude Sonnet 4, GPT-4o |
| Complex reasoning | Claude Opus 4, o1 |
| Fast responses | Claude Haiku, GPT-4o-mini |
| Long context | Claude (200k), Gemini (1M+) |
| Local/private | Llama 3 via Ollama |

## Troubleshooting

### "Model not available"

- Check provider auth is still valid.
- Check model ID spelling.
- Check account access for that model.

### "Rate limited"

- Retry after a short delay.
- Switch provider/model.
- Upgrade provider quota if needed.

### "Invalid API key"

- Verify the key has no extra spaces.
- Verify it is active and has required permissions.
