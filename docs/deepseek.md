# DeepSeek Provider (`qai_sdk::deepseek`)

The DeepSeek module implements the fast, open-weights derived `deepseek-chat` and `deepseek-coder` LLMs. DeepSeek's official API is almost entirely compatible with OpenAI's API. Thus, the implementation acts as a customized wrapping over the internal OpenAI-compatible protocol while explicitly handling DeepSeek's nuanced base URLs.

## Implemented Traits
- `LanguageModel`

## Initialization

```rust
use qai_sdk::prelude::*;

let provider = create_deepseek(ProviderSettings {
    api_key: Some(std::env::var("DEEPSEEK_API_KEY").unwrap()),
    ..Default::default()
});

use qai_sdk::DeepSeekModel;
let model = DeepSeekModel::new(api_key);
```

## Supported Features
- **Models**: Optimized for `deepseek-chat` and `deepseek-coder`.
- **Latency Optimization**: Communicates directly with DeepSeek's specialized hardware endpoints via `https://api.deepseek.com/v1`.
- **Streaming**: Native parsing over the compatible SSE format.
- *Note*: DeepSeek API currently lacks comprehensive native multimodal functionality depending on the exact model string invoked.
