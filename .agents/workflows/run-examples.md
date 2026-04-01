---
description: How to run example files from the examples/ directory
---

# Run Examples

## Steps

1. Copy the environment template:
```bash
cp .env.example .env
```

2. Fill in your API keys in `.env`:
```
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
GOOGLE_API_KEY=AI...
DEEPSEEK_API_KEY=sk-...
XAI_API_KEY=xai-...
```

3. Run any example:
```bash
// turbo
cargo run --example chat_basic
// turbo
cargo run --example streaming_basic
// turbo
cargo run --example tool_calling
// turbo
cargo run --example multimodal_vision
```

4. For MCP examples, add the `mcp` feature flag:
```bash
// turbo
cargo run --example test_mcp --features mcp
```

## Available Examples

| Example | Feature |
|---|---|
| `chat_basic` | Basic chat generation |
| `streaming_basic` | Real-time streaming |
| `tool_calling` | Function calling |
| `multimodal_vision` | Image analysis |
| `embeddings` | Text embeddings |
| `image_generation` | DALL-E image creation |
| `speech_tts` | Text-to-speech |
| `transcription` | Speech-to-text |
| `responses_api` | OpenAI Responses API |
| `error_handling` | Error pattern examples |
| `provider_factory` | Multi-provider factory |
| `openai_compatible` | Local model endpoints |
| `test_mcp` | MCP server integration |
