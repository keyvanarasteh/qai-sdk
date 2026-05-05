# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.11] - 2026-05-05

### Added

- **GroqCloud Integration**: Full native support for Groq's high-performance API endpoints.
- **Ultra-Fast Generation**: Direct support for Groq's LPU-powered chat generation (`llama-3.3-70b-versatile`, etc.) with Tool Calling and Structured Output enforcement via `response_format`.
- **Vision (Multimodal)**: Direct support for Groq's Vision models (e.g. `meta-llama/llama-4-scout-17b-16e-instruct`) via standard `ImageSource` payload encoding.
- **Speech-to-Text (STT)**: Direct support for Groq's `whisper-large-v3` and `whisper-large-v3-turbo` audio transcription endpoints mapping to `TranscriptionModel`.
- **Text-to-Speech (TTS)**: Direct support for Groq's `canopylabs/orpheus` models for near-instant speech synthesis mapping to `SpeechModel`.
- **Core Registry Expansion**: Upgraded the core SDK `ProviderRegistry` and `Provider` traits to natively resolve `transcription_model` and `speech_model` across all implementations.

## [0.1.9] - 2026-05-05

### Added

- **Ollama Configuration Helpers**: Added strongly-typed `OllamaOptionsBuilder` for easy access to `num_ctx`, `keep_alive`, `mirostat`, `seed`, and other runtime configuration map parameters natively in Rust.
- **Ollama Modelfile Builder**: Introduced `ModelfileBuilder` to programmatically construct Dockerfile-like Modelfiles with `FROM`, `PARAMETER`, `ADAPTER`, and `SYSTEM` blocks for importing and customizing models natively.
- **Local Ollama Runner**: Introduced `LocalOllamaRunner`, a programmatic `std::process` wrapper to configure and execute local Ollama server instances seamlessly. Features builder methods for environment parameters like `OLLAMA_HOST`, `OLLAMA_MAX_QUEUE`, `OLLAMA_NUM_PARALLEL`, and `OLLAMA_NO_CLOUD`.

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
