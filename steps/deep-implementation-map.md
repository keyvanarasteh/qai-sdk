# Deep Implementation Map - Phase 4: Advanced Features & Testing

This map covers the detailed implementation steps for expanding the `qai-sdk` with streaming support, tool calling, and multimodal features.

## 1. Core Abstraction Enhancement (`qai-core`)
- [x] Implement `StreamResult` and `StreamPart` types
- [x] Add `generate_stream` method to `LanguageModel` trait
- [x] Standardize `ToolDefinition` and `ToolCall` structures
- [x] Enhance `Message` and `Content` for better multimodal support (metadata, citations)

## 2. Streaming & Advanced Features Implementation (By Provider)

### Anthropic
- [x] Implement `generate_stream` using `Server-Sent Events (SSE)`
- [x] Port `AnthropicTools` and `prompt_caching` features
- [x] Add support for `thinking` (Reasoning) blocks
- [x] Add support for `Image` and `PDF` inputs

### OpenAI / Deepseek / xAI / OpenAI Compatible
- [x] Implement `generate_stream` (OpenAI compatible streaming)
- [x] Standardize OpenAI `tool_calls` implementation
- [x] Add support for `reasoning_content` (Deepseek v3 / Grok / O1)
- [x] Support `vision` (Gpt-4o / Grok-vision)

### Google (Gemini)
- [x] Implement `generate_stream` using Gemini's streaming API
- [x] Implement Gemini `function_calling`
- [x] Add support for `inlineData` (Images/Files) and `fileData`

## 3. Testing Suite Development
- [x] Create `qai-test-utils` for mocking API responses
- [x] Implement unit tests for each provider's conversion logic
- [x] Add integration tests for end-to-end `generate` and `generate_stream` calls

## 4. Documentation & Examples
- [x] Create `examples/` for each provider showing basic and advanced usage
- [x] Generate comprehensive API documentation

## 5. Extended Native Integrations (Phase 5)

### Google (Gemini)
- [ ] **Search Grounding**: Implement built-in Google Search as a native tool (`google_search_retrieval`). ([Docs](https://ai.google.dev/docs/grounding))
- [ ] **Audio Generation**: Add native support for outputting and streaming generated audio responses in Gemini 1.5 Pro/Flash. ([Docs](https://ai.google.dev/docs/audio))

### DeepSeek
- [ ] **FIM (Fill-In-the-Middle) Completions**: Implement the `/beta/completions` API endpoint for code completion tasks using prefix and suffix structures. ([Docs](https://api-docs.deepseek.com/))

### Anthropic
- [ ] **Computer Use Tools**: Implement Anthropic's Beta tools (`computer_20241022`, `bash_20241022`, and `text_editor_20241022`) to allow native interface/terminal manipulation. ([Docs](https://docs.anthropic.com/en/docs/build-with-claude/computer-use))
