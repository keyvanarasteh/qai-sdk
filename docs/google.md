# Google Gemini Provider (`qai_sdk::google`)

The Google module implements the Google Generative AI API, supporting the highly capable Gemini multimodal family (e.g. `gemini-1.5-pro` and `gemini-1.5-flash`).

## Implemented Traits
- `LanguageModel`
- `EmbeddingModel`

## Initialization

```rust
use qai_sdk::prelude::*;

let provider = create_google(ProviderSettings {
    api_key: Some(std::env::var("GOOGLE_API_KEY").unwrap()),
    ..Default::default()
});

// Or specifically:
use qai_sdk::GoogleModel;
let model = GoogleModel::new(api_key);
```

## Supported Features
- **Multimodal**: Supported. Supply internal `Image` content properly and it is mapped to Gemini's `inline_data` blobs.
- **Safety Settings**: Exposed via low-level request payload customization (currently utilizes the default balanced safety settings).
- **Tool Calling**: Supported natively.
- **Streaming**: Unlike standard SSE JSON streams, Gemini streams return arrays of objects. `GoogleModel` bridges this gracefully into the standard `BoxStream<'static, StreamPart>` contract.
