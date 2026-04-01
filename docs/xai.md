# xAI Provider (`qai_sdk::xai`)

The xAI module provides access to Grok models developed by xAI (`grok-beta`, `grok-vision`, etc.).

## Implemented Traits
- `LanguageModel`

## Initialization

```rust
use qai_sdk::prelude::*;

let provider = create_xai(ProviderSettings {
    api_key: Some(std::env::var("XAI_API_KEY").unwrap()),
    ..Default::default()
});

use qai_sdk::XAIModel;
let model = XAIModel::new(api_key);
```

## Supported Features
- **Base URL**: Targets `https://api.x.ai/v1`.
- **System Instructions**: Highly compliant with `Role::System` routing.
- **Streaming**: Emits standard text deltas mirroring standard behavior.
