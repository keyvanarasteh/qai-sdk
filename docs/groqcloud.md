<p align="center">
  <img src="../assets/groq_cover.png" alt="GroqCloud Module Banner" width="100%"/>
</p>

# GroqCloud Provider (`qai_sdk::groqcloud`)

Integration with [GroqCloud](https://console.groq.com/) for lightning-fast AI inference. This provider uses Groq's highly-recommended OpenAI-compatible `/v1` layer to provide complete native support for Chat Completions, Tool Calling, Structured Outputs, Text-to-Speech (TTS), and Speech-to-Text (STT).

---

## Implemented Traits

| Trait | Models |
|---|---|
| `LanguageModel` | Any Groq-supported LLM (e.g. `llama-3.3-70b-versatile`, `mixtral-8x7b-32768`) |
| `SpeechModel` | `canopylabs/orpheus-v1-english`, `canopylabs/orpheus-arabic-saudi` |
| `TranscriptionModel` | `whisper-large-v3`, `whisper-large-v3-turbo` |

*(Note: Groq currently does not support native embedding endpoints or image generation).*

---

## Initialization

You can create the Groq provider simply by providing your API key. If omitted, the SDK will seamlessly fallback to reading the `GROQ_API_KEY` from the system environment.

```rust
use qai_sdk::prelude::*;

// Automatically uses the GROQ_API_KEY environment variable.
let provider = create_groqcloud(ProviderSettings::default());

// Explicit initialization
let provider = create_groqcloud(ProviderSettings {
    api_key: Some("gsk_...".to_string()),
    ..Default::default()
});
```

---

## Chat Generation & LPU-Speed Processing

```rust
let chat_model = provider.chat("llama-3.3-70b-versatile");

let result = chat_model.generate(
    Prompt {
        messages: vec![
            Message { role: Role::System, content: vec![Content::Text { text: "You are a coding assistant.".into() }] },
            Message { role: Role::User, content: vec![Content::Text { text: "Write a binary search in Rust.".into() }] },
        ],
    },
    GenerateOptions {
        model_id: "llama-3.3-70b-versatile".into(),
        max_tokens: Some(2048),
        temperature: Some(0.3),
        ..Default::default()
    },
).await?;

println!("Fast Generation: {}", result.text);
```

### Tool Calling & Structured Outputs
Because the `groqcloud` module wraps the native `openai` traits under the hood, Tool Calling (`tools`) and Native JSON Output enforcement (`response_format: {"type": "json_object"}`) work out-of-the-box exactly like they do with OpenAI and Ollama.

---

## Transcription (Speech to Text)

Groq provides the fastest `whisper-large-v3` inference in the world. Use the `TranscriptionModel` to utilize it natively:

```rust
let stt_model = provider.transcription("whisper-large-v3-turbo");

let audio_bytes = std::fs::read("test.wav").unwrap();
let result = stt_model.transcribe(TranscriptionOptions {
    model_id: "whisper-large-v3-turbo".into(),
    audio: audio_bytes,
    language: Some("en".into()), // Optional language override
    prompt: None,
    temperature: None,
}).await?;

println!("Transcription: {}", result.text);
```

### Which Whisper Model to Use?
- `whisper-large-v3`: High accuracy, multilingual robust support.
- `whisper-large-v3-turbo`: Extreme speed, great price-for-performance.

---

## Speech Synthesis (Text to Speech)

Groq supports ultra-fast TTS using the `canopylabs/orpheus` models. Simply use the `SpeechModel` trait:

```rust
let tts_model = provider.speech("canopylabs/orpheus-v1-english");

let result = tts_model.synthesize(SpeechOptions {
    model_id: "canopylabs/orpheus-v1-english".into(),
    input: "Welcome to Groq text-to-speech. [cheerful] This is an example of high-quality English audio generation.".into(),
    voice: "troy".into(), // Try: troy, hannah, austin
    response_format: Some("wav".into()),
    speed: None,
}).await?;

std::fs::write("output.wav", result.audio).unwrap();
```
