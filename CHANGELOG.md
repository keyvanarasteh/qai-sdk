# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-04-01

### Added

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
