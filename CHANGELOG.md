# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.8] - 2026-05-05

### Added

- **Ollama Management API**: Native implementation of Ollama administrative endpoints (`/api/tags`, `/api/ps`, `/api/show`, `/api/create`, `/api/copy`, `/api/delete`, `/api/pull`, `/api/push`, `/api/version`).
- **Ollama Web Search / Fetch**: Native support for Ollama's `web_search` and `web_fetch` utility endpoints.
- **Ollama Cloud Support**: Intelligent auto-routing to `https://api.ollama.cloud` when an API key is provided, while seamlessly maintaining localhost compatibility for local servers.
- **Full Feature Parity**: Validated comprehensive coverage for Ollama Streaming, Thinking mode (Reasoning Traces), Structured Outputs (JSON Schema), Vision (Base64 encoding via `image_url`), Embeddings, and Tool Calling via the underlying OpenAI-compatible execution layer.

## [0.1.7] - 2026-05-05

- **DeepSeek Caching**: Full native support for DeepSeek KV cache tracking (`cache_hit_tokens`, `cache_miss_tokens` in `Usage` struct).
- **DeepSeek Multi-round**: Validated stateless multi-round chat functionality across the core generation interfaces.
- **DeepSeek Reasoner**: Native support for DeepSeek's Thinking Mode (Reasoner) with zero middleware required. Added `reasoning: Option<String>` to `GenerateResult` and a new `StreamPart::ReasoningDelta` for streaming thought blocks.
- **DeepSeek Utilities**: Added dedicated `get_balance()` and `list_models()` endpoints directly to `DeepSeekProvider`.
- **Structured Output Update**: Hardened `generate_object` logic to gracefully ignore incoming `ReasoningDelta` chunks to prevent JSON parsing errors.

### Changed
- Promoted DeepSeek Reasoner implementation from middleware to first-class SDK feature support.

## [0.1.6] - 2026-05-02

- **Core traits**: `LanguageModel`, `EmbeddingModel`, `ImageModel`, `CompletionModel`, `SpeechModel`, `TranscriptionModel`
- **OpenAI provider** (`qai-openai`): Chat, streaming, tool calling, vision, embeddings, image generation, speech/TTS, transcription/STT, text completion, and Responses API
- **Anthropic provider** (`qai-anthropic`): Chat, streaming, tool calling, vision, and PDF document support
- **Google provider** (`qai-google`): Chat, streaming, tool calling, and vision via Gemini API
- **DeepSeek provider** (`qai-deepseek`): Chat and streaming via DeepSeek API (OpenAI-compatible)
- **xAI provider** (`qai-xai`): Chat and streaming via Grok API (OpenAI-compatible)
- **OpenAI-compatible provider** (`qai-openai-compatible`): Connect to any OpenAI-compatible endpoint (Ollama, LM Studio, vLLM, etc.)
- **Umbrella crate** (`qai-sdk`): Single-import re-export with `prelude` module
- 17 comprehensive examples covering all model types and providers
- Native tool call support in `GenerateResult`
- Streaming with `StreamPart` enum (text deltas, tool call deltas, usage, finish, errors)
- `Usage::from_headers()` for extracting token counts from response headers

[0.1.0]: https://github.com/keyvanarasteh/qai-sdk/releases/tag/v0.1.0
