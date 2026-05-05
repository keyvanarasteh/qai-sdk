//! # Ollama Options Builder
//!
//! Helper struct for strongly-typed construction of Ollama-specific model parameters
//! (such as context window size, memory retention, seed, and stop sequences).
//! These options map natively to the `custom` field in `GenerateOptions`.

use serde_json::{Map, Value};

/// Strongly-typed builder for Ollama's model options.
/// Maps to the `options` field in Ollama's API payload.
#[derive(Debug, Clone, Default)]
pub struct OllamaOptionsBuilder {
    options: Map<String, Value>,
}

impl OllamaOptionsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the size of the context window used to generate the next token. (Default: 2048)
    pub fn num_ctx(mut self, val: u32) -> Self {
        self.options.insert("num_ctx".to_string(), val.into());
        self
    }

    /// Controls how long the model will stay loaded into memory following the request.
    /// Default: 5m. Use a duration string ("10m"), seconds (3600), -1 to keep loaded forever, or 0 to unload immediately.
    pub fn keep_alive<T: Into<Value>>(mut self, val: T) -> Self {
        self.options.insert("keep_alive".to_string(), val.into());
        self
    }

    /// Sets the maximum number of tokens to predict when generating text. (Default: 128, -1 = infinite, -2 = fill context)
    pub fn num_predict(mut self, val: i32) -> Self {
        self.options.insert("num_predict".to_string(), val.into());
        self
    }

    /// Sets the temperature of the model. Increasing the temperature will make the model answer more creatively. (Default: 0.8)
    pub fn temperature(mut self, val: f32) -> Self {
        self.options.insert("temperature".to_string(), val.into());
        self
    }

    /// Sets the random number seed to use for generation. Setting this to a specific number will make the model generate the same text for the same prompt. (Default: 0)
    pub fn seed(mut self, val: u32) -> Self {
        self.options.insert("seed".to_string(), val.into());
        self
    }

    /// Reduces the probability of generating nonsense. A higher value (e.g. 100) will give more diverse answers, while a lower value (e.g. 10) will be more conservative. (Default: 40)
    pub fn top_k(mut self, val: u32) -> Self {
        self.options.insert("top_k".to_string(), val.into());
        self
    }

    /// Works together with top-k. A higher value (e.g., 0.95) will lead to more diverse text, while a lower value (e.g., 0.5) will generate more focused and conservative text. (Default: 0.9)
    pub fn top_p(mut self, val: f32) -> Self {
        self.options.insert("top_p".to_string(), val.into());
        self
    }

    /// Sets the stop sequences to use.
    pub fn stop(mut self, val: Vec<String>) -> Self {
        self.options.insert("stop".to_string(), val.into());
        self
    }

    /// Enable Mirostat sampling for controlling perplexity. (default: 0, 0 = disabled, 1 = Mirostat, 2 = Mirostat 2.0)
    pub fn mirostat(mut self, val: u8) -> Self {
        self.options.insert("mirostat".to_string(), val.into());
        self
    }

    /// Sets how strongly to penalize repetitions. A higher value (e.g., 1.5) will penalize repetitions more strongly, while a lower value (e.g., 0.9) will be more lenient. (Default: 1.1)
    pub fn repeat_penalty(mut self, val: f32) -> Self {
        self.options.insert("repeat_penalty".to_string(), val.into());
        self
    }
    
    /// Sets how far back for the model to look back to prevent repetition. (Default: 64, 0 = disabled, -1 = num_ctx)
    pub fn repeat_last_n(mut self, val: i32) -> Self {
        self.options.insert("repeat_last_n".to_string(), val.into());
        self
    }

    /// Return the raw map, typically injected into the `custom` field of `GenerateOptions`.
    pub fn build(self) -> Map<String, Value> {
        self.options
    }
}
