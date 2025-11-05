# Provider Configuration and Routing Guide

This guide explains how to configure LLM providers and set up intelligent routing in the claude-code-router.

## Table of Contents

- [Configuration Overview](#configuration-overview)
- [Provider Configuration](#provider-configuration)
- [Router Configuration](#router-configuration)
- [Transformers](#transformers)
- [Local Providers](#local-providers)
  - [Ollama](#ollama)
  - [vLLM](#vllm)
  - [llama.cpp](#llamacpp)
- [Cloud Providers](#cloud-providers)
  - [Groq](#groq)
  - [OpenRouter](#openrouter)
  - [Anthropic](#anthropic)
  - [OpenAI](#openai)
  - [Google Gemini](#google-gemini)
- [Complete Examples](#complete-examples)

## Configuration Overview

The router uses a JSON configuration file located at `~/.claude-code-router/config.json`. The configuration has four main sections:

```json
{
  "Providers": [],    // List of LLM provider configurations
  "Router": {},       // Routing rules and logic
  "APIKEY": "",       // Authentication key for the router API
  "HOST": "",         // Server bind address (default: 0.0.0.0:8080)
  "LOG": false        // Enable debug logging
}
```

## Provider Configuration

Each provider in the `Providers` array has the following structure:

```json
{
  "name": "provider-name",
  "api_base_url": "http://localhost:11434/v1",
  "api_key": "your-api-key-or-placeholder",
  "models": ["model-name-1", "model-name-2"],
  "transformer": {
    "use": ["transformer1", "transformer2"]
  }
}
```

### Provider Fields

- **name** (required): Unique identifier for the provider
- **api_base_url** (required): Base URL for the provider's API endpoint
  - For OpenAI-compatible APIs, include `/v1` suffix
  - Example: `http://localhost:11434/v1` for Ollama
- **api_key** (required): Authentication key for the provider
  - Use `"not-needed"` or `"EMPTY"` for local providers without authentication
- **models** (required): Array of model identifiers available from this provider
  - Can be specific model names like `"llama3.2"` or `"mixtral-8x7b-32768"`
- **transformer** (optional): Configuration for request/response transformations
  - `use`: Array of transformer names to apply in order
  - Available transformers: `"openrouter"`, `"gemini"`, `"maxtoken"`

## Router Configuration

The router intelligently selects providers based on request characteristics:

```json
{
  "Router": {
    "default": "provider-name,model-name",
    "background": "fast-provider,fast-model",
    "think": "reasoning-provider,reasoning-model",
    "longContext": "long-context-provider,model",
    "webSearch": "web-search-provider,model"
  }
}
```

### Routing Logic Priority

The router evaluates requests in this order:

1. **Direct Model Specification**: If the request includes a model with a comma (e.g., `"groq,mixtral-8x7b-32768"`), use that provider/model combination
2. **Long Context** (>60,000 tokens): Route to `longContext` if specified
3. **Haiku Model**: If request specifies `claude-3-5-haiku`, route to `background` if configured
4. **Thinking Mode**: If `thinking: true` in request, route to `think` provider
5. **Web Search**: If tools include `web_search*`, route to `webSearch` provider
6. **Default**: Fall back to the `default` route

### Router Fields

- **default** (required): Default provider and model as `"provider,model"`
- **background** (optional): Fast/cheap model for background tasks and haiku requests
- **think** (optional): Provider for reasoning/thinking mode requests
- **longContext** (optional): Provider for requests >60k tokens
- **webSearch** (optional): Provider that supports web search tools

## Transformers

Transformers modify API requests to ensure compatibility with different provider formats.

### Available Transformers

#### `"openrouter"`
- **Use Case**: Groq, OpenRouter, and other providers without system field support
- **Functionality**:
  - Converts Claude tool format to OpenAI function calling format
  - Removes system field (incompatible with Groq)
  - Transforms tools from `{name, description, input_schema}` to OpenAI's `{type: "function", function: {...}}` format

#### `"gemini"`
- **Use Case**: Google Gemini API
- **Functionality**:
  - Similar to openrouter but includes system field support
  - Converts tools to OpenAI format
  - Preserves system prompts

#### `"maxtoken"`
- **Use Case**: Override maximum token limits
- **Functionality**:
  - Sets or overrides `max_tokens` parameter
- **Configuration**:
  ```json
  "transformer": {
    "use": [["maxtoken", {"max_tokens": 4096}]]
  }
  ```

### When to Use Transformers

- **No Transformer**: Native Claude API, Anthropic-compatible APIs
- **OpenRouter**: Groq, most OpenRouter models, providers without system field
- **Gemini**: Google Gemini API specifically
- **MaxToken**: Any provider where you want to enforce token limits

## Local Providers

### Ollama

[Ollama](https://ollama.ai/) provides a local OpenAI-compatible API for running models.

**Installation**:
```bash
# Install Ollama
curl -fsSL https://ollama.ai/install.sh | sh

# Pull a model
ollama pull llama3.2
ollama pull mixtral
ollama pull qwen2.5-coder
```

**Configuration**:
```json
{
  "Providers": [
    {
      "name": "ollama",
      "api_base_url": "http://localhost:11434/v1",
      "api_key": "not-needed",
      "models": [
        "llama3.2",
        "llama3.2:70b",
        "mixtral",
        "qwen2.5-coder:32b"
      ],
      "transformer": {
        "use": ["openrouter"]
      }
    }
  ],
  "Router": {
    "default": "ollama,qwen2.5-coder:32b"
  }
}
```

**Notes**:
- Default port: `11434`
- Use `openrouter` transformer for proper tool format conversion
- API key is not required; use placeholder value
- Model names match Ollama model tags exactly

### vLLM

[vLLM](https://github.com/vllm-project/vllm) is a high-performance inference server with OpenAI-compatible API.

**Installation**:
```bash
# Install vLLM
pip install vllm

# Start server with a model
python -m vllm.entrypoints.openai.api_server \
  --model meta-llama/Meta-Llama-3-70B-Instruct \
  --port 8000 \
  --served-model-name llama3-70b
```

**Configuration**:
```json
{
  "Providers": [
    {
      "name": "vllm",
      "api_base_url": "http://localhost:8000/v1",
      "api_key": "EMPTY",
      "models": [
        "llama3-70b",
        "mistral-7b",
        "codellama-34b"
      ],
      "transformer": {
        "use": ["openrouter"]
      }
    }
  ],
  "Router": {
    "default": "vllm,llama3-70b",
    "background": "vllm,mistral-7b"
  }
}
```

**Advanced vLLM Configuration**:
```bash
# Start with tensor parallelism for multi-GPU
python -m vllm.entrypoints.openai.api_server \
  --model meta-llama/Meta-Llama-3-70B-Instruct \
  --tensor-parallel-size 4 \
  --port 8000 \
  --served-model-name llama3-70b \
  --max-model-len 8192

# Start multiple models (requires multiple instances on different ports)
python -m vllm.entrypoints.openai.api_server \
  --model mistralai/Mistral-7B-Instruct-v0.2 \
  --port 8001 \
  --served-model-name mistral-7b
```

**Notes**:
- Default port: `8000` (customizable)
- Use `EMPTY` or `"not-needed"` for api_key
- Model name is set via `--served-model-name` flag
- Supports high-throughput inference with PagedAttention
- Use `openrouter` transformer for compatibility

### llama.cpp

[llama.cpp](https://github.com/ggerganov/llama.cpp) provides efficient local inference with OpenAI-compatible server.

**Installation**:
```bash
# Clone and build
git clone https://github.com/ggerganov/llama.cpp
cd llama.cpp
make

# Build with GPU support (CUDA)
make LLAMA_CUDA=1

# Build with Metal (macOS)
make LLAMA_METAL=1
```

**Starting the Server**:
```bash
# Start server with a GGUF model
./server \
  -m models/llama-3-8b-instruct.Q4_K_M.gguf \
  --port 8080 \
  --ctx-size 4096 \
  --n-gpu-layers 35 \
  --host 0.0.0.0

# The OpenAI-compatible endpoint will be at http://localhost:8080/v1
```

**Configuration**:
```json
{
  "Providers": [
    {
      "name": "llamacpp",
      "api_base_url": "http://localhost:8080/v1",
      "api_key": "not-needed",
      "models": [
        "llama3-8b",
        "mistral-7b",
        "codellama-13b"
      ],
      "transformer": {
        "use": ["openrouter"]
      }
    }
  ],
  "Router": {
    "default": "llamacpp,llama3-8b"
  }
}
```

**Notes**:
- Default port: `8080` (customizable with `--port`)
- Model name is arbitrary; server only runs one model at a time
- Use quantized GGUF models for better performance
- `--n-gpu-layers` offloads layers to GPU (adjust based on VRAM)
- Use `openrouter` transformer for proper tool handling

**Multiple llama.cpp Instances**:
```json
{
  "Providers": [
    {
      "name": "llamacpp-large",
      "api_base_url": "http://localhost:8080/v1",
      "api_key": "not-needed",
      "models": ["llama3-70b"],
      "transformer": {"use": ["openrouter"]}
    },
    {
      "name": "llamacpp-small",
      "api_base_url": "http://localhost:8081/v1",
      "api_key": "not-needed",
      "models": ["llama3-8b"],
      "transformer": {"use": ["openrouter"]}
    }
  ],
  "Router": {
    "default": "llamacpp-large,llama3-70b",
    "background": "llamacpp-small,llama3-8b"
  }
}
```

## Cloud Providers

### Groq

[Groq](https://groq.com/) provides ultra-fast inference with LPU (Language Processing Unit) technology.

**Setup**:
1. Sign up at https://console.groq.com/
2. Create API key at https://console.groq.com/keys

**Configuration**:
```json
{
  "Providers": [
    {
      "name": "groq",
      "api_base_url": "https://api.groq.com/openai/v1",
      "api_key": "gsk_...",
      "models": [
        "llama-3.3-70b-versatile",
        "llama-3.1-70b-versatile",
        "mixtral-8x7b-32768",
        "gemma2-9b-it"
      ],
      "transformer": {
        "use": ["openrouter"]
      }
    }
  ],
  "Router": {
    "default": "groq,llama-3.3-70b-versatile",
    "background": "groq,gemma2-9b-it"
  }
}
```

**Available Models** (as of 2025):
- `llama-3.3-70b-versatile`: Latest Llama 3.3 70B
- `llama-3.1-70b-versatile`: Llama 3.1 70B with 128k context
- `mixtral-8x7b-32768`: Mixtral 8x7B with 32k context
- `gemma2-9b-it`: Google Gemma 2 9B

**Notes**:
- Extremely fast inference (hundreds of tokens/sec)
- Use `openrouter` transformer (Groq doesn't support system field in body)
- Free tier available with rate limits
- System prompts are prepended to first user message by transformer

### OpenRouter

[OpenRouter](https://openrouter.ai/) provides unified access to multiple LLM providers.

**Setup**:
1. Sign up at https://openrouter.ai/
2. Get API key from https://openrouter.ai/keys
3. Add credits at https://openrouter.ai/credits

**Configuration**:
```json
{
  "Providers": [
    {
      "name": "openrouter",
      "api_base_url": "https://openrouter.ai/api/v1",
      "api_key": "sk-or-v1-...",
      "models": [
        "anthropic/claude-3.5-sonnet",
        "google/gemini-pro-1.5",
        "meta-llama/llama-3.1-70b-instruct",
        "openai/gpt-4-turbo"
      ],
      "transformer": {
        "use": ["openrouter"]
      }
    }
  ],
  "Router": {
    "default": "openrouter,anthropic/claude-3.5-sonnet"
  }
}
```

**Notes**:
- Access to 100+ models through single API
- Model names use `provider/model` format
- Pay-as-you-go pricing
- Use `openrouter` transformer for compatibility

### Anthropic

[Anthropic](https://anthropic.com/) provides Claude models directly.

**Setup**:
1. Sign up at https://console.anthropic.com/
2. Create API key at https://console.anthropic.com/settings/keys

**Configuration**:
```json
{
  "Providers": [
    {
      "name": "anthropic",
      "api_base_url": "https://api.anthropic.com/v1",
      "api_key": "sk-ant-...",
      "models": [
        "claude-3-5-sonnet-20241022",
        "claude-3-5-haiku-20241022",
        "claude-3-opus-20240229"
      ]
    }
  ],
  "Router": {
    "default": "anthropic,claude-3-5-sonnet-20241022",
    "background": "anthropic,claude-3-5-haiku-20241022"
  }
}
```

**Notes**:
- **No transformer needed** - native Claude API format
- Direct access to Claude models
- Best for Claude-specific features
- Note: This router is primarily useful when mixing Anthropic with other providers

### OpenAI

[OpenAI](https://openai.com/) provides GPT models.

**Setup**:
1. Sign up at https://platform.openai.com/
2. Create API key at https://platform.openai.com/api-keys

**Configuration**:
```json
{
  "Providers": [
    {
      "name": "openai",
      "api_base_url": "https://api.openai.com/v1",
      "api_key": "sk-...",
      "models": [
        "gpt-4-turbo-preview",
        "gpt-4",
        "gpt-3.5-turbo"
      ],
      "transformer": {
        "use": ["openrouter"]
      }
    }
  ],
  "Router": {
    "default": "openai,gpt-4-turbo-preview",
    "background": "openai,gpt-3.5-turbo"
  }
}
```

**Notes**:
- Use `openrouter` transformer for Claude→OpenAI format conversion
- Requires paid account with credits
- Rate limits apply based on tier

### Google Gemini

[Google Gemini](https://ai.google.dev/) provides Gemini models.

**Setup**:
1. Get API key from https://makersuite.google.com/app/apikey

**Configuration**:
```json
{
  "Providers": [
    {
      "name": "gemini",
      "api_base_url": "https://generativelanguage.googleapis.com/v1beta/openai",
      "api_key": "AIza...",
      "models": [
        "gemini-1.5-pro",
        "gemini-1.5-flash"
      ],
      "transformer": {
        "use": ["gemini"]
      }
    }
  ],
  "Router": {
    "default": "gemini,gemini-1.5-pro",
    "background": "gemini,gemini-1.5-flash"
  }
}
```

**Notes**:
- Use `gemini` transformer specifically
- Supports system field (unlike Groq)
- Free tier available with limits

## Complete Examples

### Example 1: Local-First with Cloud Backup

```json
{
  "Providers": [
    {
      "name": "ollama",
      "api_base_url": "http://localhost:11434/v1",
      "api_key": "not-needed",
      "models": ["qwen2.5-coder:32b", "llama3.2"],
      "transformer": {"use": ["openrouter"]}
    },
    {
      "name": "groq",
      "api_base_url": "https://api.groq.com/openai/v1",
      "api_key": "gsk_...",
      "models": ["llama-3.3-70b-versatile", "mixtral-8x7b-32768"],
      "transformer": {"use": ["openrouter"]}
    }
  ],
  "Router": {
    "default": "ollama,qwen2.5-coder:32b",
    "longContext": "groq,mixtral-8x7b-32768",
    "think": "groq,llama-3.3-70b-versatile"
  },
  "APIKEY": "your-router-auth-key",
  "HOST": "0.0.0.0:8080",
  "LOG": false
}
```

### Example 2: Multi-Provider with Specialized Routing

```json
{
  "Providers": [
    {
      "name": "llamacpp",
      "api_base_url": "http://localhost:8080/v1",
      "api_key": "not-needed",
      "models": ["codellama-70b"],
      "transformer": {"use": ["openrouter"]}
    },
    {
      "name": "vllm",
      "api_base_url": "http://localhost:8000/v1",
      "api_key": "EMPTY",
      "models": ["llama3-70b"],
      "transformer": {"use": ["openrouter"]}
    },
    {
      "name": "groq",
      "api_base_url": "https://api.groq.com/openai/v1",
      "api_key": "gsk_...",
      "models": ["llama-3.3-70b-versatile", "gemma2-9b-it"],
      "transformer": {"use": ["openrouter"]}
    },
    {
      "name": "openrouter",
      "api_base_url": "https://openrouter.ai/api/v1",
      "api_key": "sk-or-v1-...",
      "models": ["anthropic/claude-3.5-sonnet"],
      "transformer": {"use": ["openrouter"]}
    }
  ],
  "Router": {
    "default": "llamacpp,codellama-70b",
    "background": "groq,gemma2-9b-it",
    "think": "openrouter,anthropic/claude-3.5-sonnet",
    "longContext": "vllm,llama3-70b",
    "webSearch": "groq,llama-3.3-70b-versatile"
  },
  "APIKEY": "secure-key-here",
  "HOST": "127.0.0.1:8080",
  "LOG": true
}
```

### Example 3: All-Local Development Setup

```json
{
  "Providers": [
    {
      "name": "ollama-coding",
      "api_base_url": "http://localhost:11434/v1",
      "api_key": "not-needed",
      "models": ["qwen2.5-coder:32b", "deepseek-coder:33b"],
      "transformer": {"use": ["openrouter"]}
    },
    {
      "name": "llamacpp-fast",
      "api_base_url": "http://localhost:8081/v1",
      "api_key": "not-needed",
      "models": ["llama3-8b-q4"],
      "transformer": {"use": ["openrouter"]}
    },
    {
      "name": "llamacpp-quality",
      "api_base_url": "http://localhost:8082/v1",
      "api_key": "not-needed",
      "models": ["llama3-70b-q4"],
      "transformer": {"use": ["openrouter"]}
    }
  ],
  "Router": {
    "default": "ollama-coding,qwen2.5-coder:32b",
    "background": "llamacpp-fast,llama3-8b-q4",
    "think": "llamacpp-quality,llama3-70b-q4"
  },
  "APIKEY": "local-dev-key",
  "HOST": "127.0.0.1:8080",
  "LOG": true
}
```

## Testing Your Configuration

After setting up your configuration, test it:

```bash
# Start the router
cargo run --bin ccr -- start

# Check status
cargo run --bin ccr -- status

# Test with a simple request
curl http://localhost:8080/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: your-router-auth-key" \
  -d '{
    "model": "default",
    "messages": [
      {"role": "user", "content": "Hello, how are you?"}
    ],
    "max_tokens": 100
  }'

# Test routing to specific provider
curl http://localhost:8080/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: your-router-auth-key" \
  -d '{
    "model": "ollama,llama3.2",
    "messages": [
      {"role": "user", "content": "What is Rust?"}
    ],
    "max_tokens": 200
  }'

# Use with Claude Code CLI
cargo run --bin ccr -- code "list files in current directory"
```

## Troubleshooting

### Common Issues

**Provider Not Responding**:
- Check provider server is running (e.g., `curl http://localhost:11434/v1/models`)
- Verify `api_base_url` includes `/v1` suffix for OpenAI-compatible APIs
- Check API key is correct

**Tool Calling Not Working**:
- Ensure using correct transformer (`openrouter` for most providers)
- Check provider model supports function calling
- Enable logging with `"LOG": true` to see request/response

**Routing Not Working as Expected**:
- Check Router configuration field names (case-sensitive: `longContext`, `webSearch`)
- Verify model names match exactly what's in provider's `models` array
- Use format `"provider,model"` for route values

**Authentication Errors**:
- Verify `APIKEY` in config matches header (`x-api-key` or `Authorization: Bearer`)
- Check provider API key is valid and has credits/quota

### Debug Mode

Enable detailed logging:

```json
{
  "LOG": true
}
```

Then check logs when starting the router:
```bash
cargo run --bin ccr -- start
```

---

For more information, see the main [README.md](README.md) and [CLAUDE.md](CLAUDE.md).
