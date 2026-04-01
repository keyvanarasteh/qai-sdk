# Anthropic Provider (`qai_sdk::anthropic`)

The Anthropic module integrates Claude models (Opus, Sonnet, Haiku) and correctly translates the `qai-sdk` standard `Prompt` structures into the highly-specific structural requirements of the Anthropic Messages API.

## Implemented Traits
- `LanguageModel`

## Initialization

```rust
use qai_sdk::prelude::*;

let provider = create_anthropic(ProviderSettings {
    api_key: Some(std::env::var("ANTHROPIC_API_KEY").unwrap()),
    version: Some("2023-06-01".to_string()), // Optional override
    ..Default::default()
});

// Or instantiate the model directly
use qai_sdk::AnthropicModel;
let model = AnthropicModel::new(api_key);
```

## Supported Features
- **Multimodal (Vision)**: Fully supported. `Claude 3` family will parse `Content::Image { media_type, data }` blocks.
- **System Prompts**: Automatically intercepted from `Role::System` and correctly converted into the high-level `system` property outside the messages array to comply with the Anthropic API design.
- **Tool Calling**: Seamless conversion between `qai_sdk` standard tool arguments and Anthropic's `<tool_use>` XML/JSON representations.
- **Streaming**: Fully decodes Anthropic's proprietary SSE events (`message_start`, `content_block_delta`, `message_delta`) back into universal `StreamPart` enum values.
