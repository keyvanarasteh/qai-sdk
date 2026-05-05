# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.26] - 2026-05-05

### Changed
- **Documentation**: Exhaustive updates to `README.md` and `docs/` covering specialized modalities and agentic features.

## [0.1.25] - 2026-05-05

### Added
- **Gemini Specialized Modalities**:
  - `ImageModel` support for Imagen 3 (`imagen-3.0-generate-001`).
  - `VideoModel` support for Veo 2 (`veo-2.0-generate-001`).
  - `MusicModel` support for Lyria (`lyria-3-pro-001`).
  - `RealtimeModel` support for Multimodal Live API (WebSocket).
  - Spatial Reasoning & Bounding Box support in `Content::Spatial`.
  - Built-in `code_execution` tool support for Gemini.
- **Core Expansion**:
  - `VideoModel`, `MusicModel`, and `RealtimeModel` traits.
  - `VideoGenerateOptions`, `MusicGenerateOptions`, and `RealtimeEvent` types.
  - `BoundingBox` and `Spatial` content variant.

## [0.1.24] - 2026-05-06

### Added

- **xAI Advanced Agentic Tooling**:
  - Implemented `web_search` and `code_execution` server-side tools for xAI Grok models (e.g., `grok-2`).
  - Added support for `collections_search` (RAG on specific URIs) and `remote_mcp` (direct server-side MCP orchestration).
  - Integrated granular control flags: `include_citations`, `include_tool_outputs`, and `max_turns` in `GenerateOptions`.
- **Rich Metadata & Citations**:
  - Introduced the `Citation` structure in `GenerateResult` for detailed source attribution (source name, snippet, index, URI).
  - Added `StreamPart::Citation` for real-time, granular citation delivery during streaming.
  - Updated `GenerateResult` across all providers to support citation metadata.
- **Provider Architecture Refinement**:
  - Refactored `OpenAITool` from a rigid struct to a flexible `enum` to support xAI-specific built-in tools.
  - Hardened `GenerateOptions` initializers across Anthropic, Google, DeepSeek, and OpenAI providers for better forward-compatibility.
- **Examples**:
  - `xai_web_search.rs`: Demonstrates search grounding with real-time citations.
  - `xai_code_execution.rs`: Demonstrates server-side Python execution via Grok.

## [0.1.23] - 2026-05-06

### Added

- **New Core Traits**:
  - `VideoModel`: Formal abstraction for text-to-video generation.
  - `RealtimeModel`: Formal abstraction for WebSocket-based bidirectional voice/text agents.
- **xAI Multimodal Expansion**:
  - Implemented `SpeechModel` (TTS) and `TranscriptionModel` (STT) for xAI Grok.
  - Added `VideoModel` support for `grok-imagine-video` with automatic asynchronous polling.
  - Added `RealtimeModel` skeleton for xAI Voice Agents (OpenAI-compatible WebSocket).
- **Standardized Reasoning & Thinking**:
  - Unified `reasoning_effort` and `reasoning_format` across Anthropic, Google Gemini, and xAI providers.
  - Mapped Anthropic's "Extended Thinking" and Gemini's "Thinking Config" to the core `GenerateOptions`.
- **Groq Built-in Tooling**:
  - Integrated Groq's Responses API built-in tools (`web_search`, `visit_website`, `code_executor`) into the provider registry.
- **Dependencies**:
  - Added `tokio-tungstenite` for WebSocket support.
  - Added `http` crate for low-level request building.

## [0.1.22] - 2026-05-06

### Added

- **xAI Prompt Caching**:
  - Implemented automatic mapping of `extra_headers` down to the HTTP client for OpenAI-compatible providers, enabling `x-grok-conv-id` for sticky routing and optimal prompt caching on xAI.
  - Added `xai_prompt_caching.rs` example demonstrating multi-turn cache hits.

## [0.1.21] - 2026-05-06

### Added

- **Google Gemini Audio Generation**:
  - Implemented `speech_model()` for the Google provider, enabling Text-to-Speech (TTS) capabilities using Gemini models (e.g., `gemini-2.5-flash`).
  - Added `response_modalities` and `speech_config` support to `GoogleGenerationConfig` to correctly request audio modalities.
  - Added `gemini_audio_generation.rs` example demonstrating TTS capabilities.

## [0.1.20] - 2026-05-06

### Added

- **Anthropic Computer Use**:
  - Implemented support for Anthropic's Computer Use Beta.
  - Added native mapping for `computer_20241022`, `bash_20241022`, and `text_editor_20241022` tools.
  - The SDK automatically sets the required `anthropic-beta` header when these tools are present in the request.
  - Added `anthropic_computer_use.rs` example.

## [0.1.19] - 2026-05-06

### Added

- **DeepSeek FIM (Fill-In-the-Middle)**:
  - Added native support for the DeepSeek `/beta/completions` API.
  - Implemented the `CompletionModel` interface for DeepSeek to support `suffix` inputs.
  - Exposed `completion_model` resolving in the `ProviderRegistry` for DeepSeek.
  - Added `deepseek_fim.rs` example.

## [0.1.18] - 2026-05-06

### Added

- **Google Gemini Search Grounding**:
  - Implemented native Web Search Grounding for Gemini models.
  - Added support for parsing `groundingMetadata` from API responses.
  - Exposed grounding results via the standard `executed_tools` array in both batch and streaming generations.
  - Added `gemini_search_grounding.rs` example.

## [0.1.17] - 2026-05-06

### Added

- **xAI Vision and Image Generation**:
  - Validated and documented support for xAI multimodal vision via the `grok-2-vision-1212` model using the standard `Content::Image` structure.
  - Documented image generation support via the `grok-imagine-image` model using the `ImageModel` interface.
  - Added new examples: `xai_vision.rs` and `xai_image_generation.rs`.

## [0.1.16] - 2026-05-06

### Added

- **Gemini Thinking/Reasoning**: Full support for Gemini 3 and 2.5 thinking models.
  - `GoogleThinkingConfig`: `include_thoughts`, `thinking_level` (Gemini 3), `thinking_budget` (Gemini 2.5).
  - `GooglePart::Text` gains `thought: Option<bool>` flag for identifying thought summaries.
  - `reasoning_effort` auto-maps to `thinking_level` or `thinking_budget`.
  - Thought summaries extracted into `GenerateResult.reasoning`.
  - Streaming emits `StreamPart::ReasoningDelta` for thought parts.
  - New `gemini_thinking.rs` example.

- **Anthropic Extended/Adaptive Thinking**: Full support for Claude extended thinking and adaptive thinking.
  - `AnthropicThinkingConfig`: `type` (adaptive/enabled/disabled), `budget_tokens`, `display` (summarized/omitted).
  - `AnthropicContent::Thinking` variant for thinking content blocks with signature support.
  - `AnthropicDelta::ThinkingDelta` and `SignatureDelta` for streaming thinking events.
  - `reasoning_effort` maps to adaptive mode (`"adaptive"`, named levels) or manual budget (numeric).
  - `reasoning_format` maps to display mode (`"parsed"` → summarized, `"omitted"` → omitted).
  - Temperature/top_p automatically cleared when thinking is enabled (Anthropic requirement).
  - Thinking blocks extracted into `GenerateResult.reasoning`.
  - Streaming emits `StreamPart::ReasoningDelta` for `thinking_delta` events.
  - New `anthropic_thinking.rs` example.

- **xAI Reasoning**: Documentation and example for xAI Grok reasoning models (via OpenAI-compatible pipeline).
  - New `xai_reasoning.rs` example.

### Documentation

- Updated `docs/google.md` with Thinking/Reasoning section and modern model table.
- Updated `docs/anthropic.md` with Extended/Adaptive Thinking section and modern model table.
- Updated `docs/xai.md` with Reasoning section.

## [0.1.15] - 2026-05-06

### Added

- **GroqCloud Tool Use**: Full three-tier tool ecosystem — Built-in Tools (Web Search, Visit Website), Remote MCP server integration, and enhanced Local Tool Calling.
- **`tool_choice`**: Added `tool_choice: Option<serde_json::Value>` to `GenerateOptions` for fine-grained control over tool selection (`"auto"`, `"required"`, `"none"`, or specific function).
- **`parallel_tool_calls`**: Added `parallel_tool_calls: Option<bool>` to `GenerateOptions` for enabling parallel tool execution.
- **`executed_tools`**: Added `executed_tools: Vec<ExecutedTool>` to `GenerateResult` for surfacing metadata about server-side executed tools (web search, MCP calls).
- **`GroqTool` types**: New `groqcloud::tools` module with `GroqTool` enum (Function, WebSearch, VisitWebsite, Mcp), `GroqUserLocation`, `GroqMcpAllowedTool`, and ergonomic constructors.
- **`GroqCloudResponsesModel`**: New Responses API wrapper for agentic workflows via Groq.
- **Compound model support**: New `.compound()` and `.responses()` factory methods on `GroqCloudProvider`.
- **Examples**: `groqcloud_web_search.rs`, `groqcloud_mcp.rs`, `groqcloud_tool_calling.rs`.

### Fixed

- Pre-existing compilation issues in `groqcloud_chat.rs`, `groqcloud_reasoning.rs`, `groqcloud_vision.rs`, `ollama_basic.rs`, `chat_streaming.rs` examples (Usage type access, missing trait imports, non-exhaustive match arms).
- All example files now use `..Default::default()` for forward-compatible `GenerateOptions` construction.

## [0.1.14] - 2026-05-05

### Documented

- **Prompt Caching**: Added documentation detailing GroqCloud Prompt Caching support, which operates seamlessly with zero code changes and exposes hit metrics directly in `usage.cache_hit_tokens`.

## [0.1.13] - 2026-05-05

### Added

- **Structured Outputs (Strict Mode Control)**: Added `strict: Option<bool>` to `ObjectGenerateOptions` allowing users to explicitly enable or disable strict JSON schema enforcement (e.g., using `strict: false` for best-effort schemas on models like `meta-llama/llama-4-scout-17b-16e-instruct` on GroqCloud).

## [0.1.12] - 2026-05-05

### Added

- **Reasoning Configuration**: Added `reasoning_format` and `reasoning_effort` to `GenerateOptions` and `OpenAIRequest` payloads to support advanced logic configuration for reasoning models natively in Groq (e.g., `qwen/qwen3-32b`).
- **Content Moderation**: Added examples and documentation for integrating safety models like `openai/gpt-oss-safeguard-20b` for prompt injection detection and policy enforcement.
- **OpenAI Schema Robustness**: Added `alias = "reasoning"` to the `reasoning_content` field to correctly parse the "reasoning" key returned by certain OpenAI-compatible providers like Groq.

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
