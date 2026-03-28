# Implementation Roadmap

This roadmap outlines the steps to port the AI providers from TypeScript to Rust.

## Phase 1: Infrastructure Setup
- [x] Initialize Cargo workspace in `qai-sdk` [x]
- [x] Create common utility crate `qai-core` if needed [x]

## Phase 2: Provider Conversion (One by One)
### Anthropic
- [x] Create `qai-anthropic` crate [x]
- [x] Port types and API client logic [x]
- [x] Implement unit tests [x]
- [x] Verify compilation and tests [x]

### Deepseek
- [x] Create `qai-deepseek` crate [x]
- [x] Port types and API client logic [x]
- [x] Implement unit tests [x]
- [x] Verify compilation and tests [x]

### Google
- [x] Create `qai-google` crate [x]
- [x] Port types and API client logic [x]
- [x] Implement unit tests [x]
- [x] Verify compilation and tests [x]

### OpenAI
- [x] Create `qai-openai` crate [x]
- [x] Port types and API client logic [x]
- [x] Implement unit tests [x]
- [x] Verify compilation and tests [x]

### OpenAI Compatible
- [x] Create `qai-openai-compatible` crate [x]
- [x] Port types and API client logic [x]
- [x] Implement unit tests [x]
- [x] Verify compilation and tests [x]

### xAI
- [x] Create `qai-xai` crate [x]
- [x] Port types and API client logic [x]
- [x] Implement unit tests [x]
- [x] Verify compilation and tests [x]

## Phase 3: Final Verification
- [x] Run workspace-wide checks [x]
- [x] Ensure all crates are correctly linked and tested [x]

## Phase 4: Advanced Features & Testing
- [x] Core Abstraction Enhancement (`qai-core`) [x]
- [x] Streaming Support (Anthropic) [x]
- [x] Streaming Support (OpenAI / Deepseek / xAI) [x]
- [x] Streaming Support (Google Gemini) [x]
- [x] Anthropic Tool Calling [x]
- [x] OpenAI / Deepseek / xAI / Compatible Tool Calling [x]
- [x] Google Gemini Tool Calling [x]
- [x] Multimodal Support Enhancement [x]
- [x] Token Usage Header Extraction [x]
- [x] Testing Suite Development [x]

## Phase 5: Provider Factory Pattern & Config
- [x] Add `ProviderSettings` and factory functions to `qai-core` [x]
- [x] Implement provider factories for all 6 providers [x]

## Phase 6: Embedding Models
- [x] Add `EmbeddingModel` trait to `qai-core` [x]
- [x] Implement for OpenAI, Google, and OpenAI-Compatible [x]

## Phase 7: Image Generation Models
- [x] Add `ImageModel` trait to `qai-core` [x]
- [x] Implement for OpenAI, Google, xAI, and OpenAI-Compatible [x]

## Phase 8: Custom Error Types
- [x] Add `AnthropicError`, `OpenAIError`, `GoogleError` [x]

## Phase 9: Server-Defined Tools
- [x] Anthropic tools (19 tools: bash, computer, text-editor, web-search, etc.) [x]
- [x] OpenAI tools (11 tools: code-interpreter, file-search, web-search, etc.) [x]
- [x] Google tools (7 tools: google-search, code-execution, url-context, etc.) [x]
- [x] xAI tools (7 tools: web-search, code-execution, etc.) [x]

## Phase 10: Completion API
- [x] Add `CompletionModel` trait to `qai-core` [x]
- [x] Implement for OpenAI and OpenAI-Compatible [x]

## Phase 11: Speech & Transcription
- [x] Add `SpeechModel` and `TranscriptionModel` traits to `qai-core` [x]
- [x] Implement for OpenAI [x]
