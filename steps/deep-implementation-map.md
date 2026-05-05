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
- [x] **Search Grounding**: Implement built-in Google Search as a native tool (`google_search_retrieval`). ([Docs](https://ai.google.dev/docs/grounding))
- [x] **Audio Generation**: Add native support for outputting and streaming generated audio responses in Gemini 1.5 Pro/Flash. ([Docs](https://ai.google.dev/docs/audio))

### DeepSeek
- [x] **FIM (Fill-In-the-Middle) Completions**: Implement the `/beta/completions` API endpoint for code completion tasks using prefix and suffix structures. ([Docs](https://api-docs.deepseek.com/))

### Anthropic
- [x] **Computer Use Tools**: Implement Anthropic's Beta tools (`computer_20241022`, `bash_20241022`, and `text_editor_20241022`) to allow native interface/terminal manipulation. ([Docs](https://docs.anthropic.com/en/docs/build-with-claude/computer-use))

## 6. Extended Integrations (Phase 6)

### Prompt Caching & Reasoning
- [x] **xAI Prompt Caching**: Implement `x-grok-conv-id` headers for prompt caching. ([How it works](https://docs.x.ai/developers/advanced-api-usage/prompt-caching/how-it-works), [Maximizing hits](https://docs.x.ai/developers/advanced-api-usage/prompt-caching/maximizing-cache-hits))
- [x] **xAI Reasoning**: Support xAI reasoning parameters. ([Reasoning](https://docs.x.ai/developers/model-capabilities/text/reasoning))
- [x] **Gemini Reasoning**: Support Gemini thought signatures. ([Thinking](https://ai.google.dev/gemini-api/docs/thinking), [Signatures](https://ai.google.dev/gemini-api/docs/thought-signatures))
- [x] **Anthropic Reasoning**: Support Claude extended and adaptive thinking. ([Extended](https://platform.claude.com/docs/en/build-with-claude/extended-thinking), [Adaptive](https://platform.claude.com/docs/en/build-with-claude/adaptive-thinking))

### Multimodal Expansion (xAI)
- [x] **xAI Audio/Voice**: Implement TTS, STT, and voice agents for xAI. ([Voice](https://docs.x.ai/developers/model-capabilities/audio/voice), [TTS](https://docs.x.ai/developers/model-capabilities/audio/text-to-speech), [STT](https://docs.x.ai/developers/model-capabilities/audio/speech-to-text))
- [x] **xAI Video Generation**: Implement video generation for xAI. ([Video Generation](https://docs.x.ai/developers/model-capabilities/video/generation))
- [x] **xAI Vision**: Implement image understanding and generation. ([Understanding](https://docs.x.ai/developers/model-capabilities/images/understanding), [Generation](https://docs.x.ai/developers/model-capabilities/images/generation))

### Groq Tool Use
- [x] **Groq Tools**: Implement built-in web search, visit-website, and remote MCP. ([Web Search](https://console.groq.com/docs/tool-use/built-in-tools/web-search), [MCP](https://console.groq.com/docs/tool-use/remote-mcp))

### Multi-Agent
- [ ] **xAI Multi-agent**: Support multi-agent text generation. ([Multi-agent](https://docs.x.ai/developers/model-capabilities/text/multi-agent))

## 7. Advanced Agentic Features & Specialized Modalities (Phase 7)

### xAI Advanced Tooling
- [x] **xAI Web Search**: Integrate native `web_search` tool. Support `search_results` and `search_queries` in response metadata. ([Docs](https://docs.x.ai/developers/tools/web-search))
- [x] **xAI Code Execution**: Support `code_execution` tool. Handle `code_execution_call` and `code_execution_result` blocks in message history. ([Docs](https://docs.x.ai/developers/tools/code-execution))
- [x] **xAI Collections Search**: Implement `collections_search` tool. Handle `collection_uris` and citation indices. ([Docs](https://docs.x.ai/developers/tools/collections-search))
- [x] **xAI Remote MCP**: Support `remote_mcp` tool configuration with `server_url` and `allowed_tools`. ([Docs](https://docs.x.ai/developers/tools/remote-mcp))
- [x] **xAI Citations**: Implement `include_citations` parameter and parse `citations` array with `source`, `snippet`, and `index` mapping. ([Docs](https://docs.x.ai/developers/tools/citations))
- [x] **xAI Streaming & Sync**: Implement `include_tool_outputs` for real-time observability of tool execution during streaming. ([Docs](https://docs.x.ai/developers/tools/streaming-and-sync))
- [x] **xAI Tool Usage Details**: Expose `tool_calls` and `server_side_tool_usage` in usage metrics. ([Docs](https://docs.x.ai/developers/tools/tool-usage-details))
- [x] **xAI Advanced Usage**: Implement hybrid tool orchestration (server-side + client-side) and `max_turns` limit. ([Docs](https://docs.x.ai/developers/tools/advanced-usage))

### Google Gemini Specialized Modalities
- [ ] **Gemini Image Generation**: Support `imagen` models (e.g., `imagen-3.0-generate-001`) via REST. Handle `Image` output parts. ([Docs](https://ai.google.dev/gemini-api/docs/image-generation#rest))
- [ ] **Gemini Video Generation**: Support `veo` models (e.g., `veo-2.0-generate-001`) via REST. Implement `GenerateVideoConfig` (resolution, duration). ([Docs](https://ai.google.dev/gemini-api/docs/video))
- [ ] **Gemini Music Generation**: Support `lyria` models (e.g., `lyria-3-pro-001`) via REST. Handle `audio_config` and `response_modalities: ["AUDIO"]`. ([Docs](https://ai.google.dev/gemini-api/docs/music-generation#rest))
- [ ] **Gemini Realtime Music**: Implement WebSocket-based streaming for `lyria-realtime-exp`. Handle control messages and steerable prompts. ([Docs](https://ai.google.dev/gemini-api/docs/realtime-music-generation#rest))
- [ ] **Gemini Robotics**: Support `robotics-er-1.6` models. Implement spatial reasoning parts (bounding boxes, point tracking). ([Docs](https://ai.google.dev/gemini-api/docs/robotics-overview#rest))
- [ ] **Gemini Video Understanding**: Enhance `file_data` support for large videos. Implement `video_metadata` with clipping and custom FPS. ([Docs](https://ai.google.dev/gemini-api/docs/video-understanding#rest))
