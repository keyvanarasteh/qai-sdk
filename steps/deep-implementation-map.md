# Deep Implementation Map - Phase 4: Advanced Features & Testing

This map covers the detailed implementation steps for expanding the `qai-sdk` with streaming support, tool calling, and multimodal features.

## 1. Core Abstraction Enhancement (`qai-core`)
- [ ] Implement `StreamResult` and `StreamPart` types [ ]
- [ ] Add `generate_stream` method to `LanguageModel` trait [ ]
- [ ] Standardize `ToolDefinition` and `ToolCall` structures [ ]
- [ ] Enhance `Message` and `Content` for better multimodal support (metadata, citations) [ ]

## 2. Streaming & Advanced Features Implementation (By Provider)

### Anthropic
- [ ] Implement `generate_stream` using `Server-Sent Events (SSE)` [ ]
- [ ] Port `AnthropicTools` and `prompt_caching` features [ ]
- [ ] Add support for `thinking` (Reasoning) blocks [ ]
- [ ] Add support for `Image` and `PDF` inputs [ ]

### OpenAI / Deepseek / xAI / OpenAI Compatible
- [ ] Implement `generate_stream` (OpenAI compatible streaming) [ ]
- [ ] Standardize OpenAI `tool_calls` implementation [ ]
- [ ] Add support for `reasoning_content` (Deepseek v3 / Grok / O1) [ ]
- [ ] Support `vision` (Gpt-4o / Grok-vision) [ ]

### Google (Gemini)
- [ ] Implement `generate_stream` using Gemini's streaming API [ ]
- [ ] Implement Gemini `function_calling` [ ]
- [ ] Add support for `inlineData` (Images/Files) and `fileData` [ ]

## 3. Testing Suite Development
- [ ] Create `qai-test-utils` for mocking API responses [ ]
- [ ] Implement unit tests for each provider's conversion logic [ ]
- [ ] Add integration tests for end-to-end `generate` and `generate_stream` calls (using recorded responses) [ ]

## 4. Documentation & Examples
- [ ] Create `examples/` for each provider showing basic and advanced usage [ ]
- [ ] Generate comprehensive API documentation [ ]
