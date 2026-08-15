//! # Modelfile Builder
//!
//! A strongly-typed builder for creating Ollama Modelfile strings programmatically.
//! Used for importing models, setting adapters, and customizing model behavior.

/// A strongly-typed builder for constructing an Ollama Modelfile string.
#[derive(Debug, Clone, Default)]
pub struct ModelfileBuilder {
    lines: Vec<String>,
}

impl ModelfileBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the `FROM` instruction (Required). Defines the base model or path to GGUF/Safetensors.
    /// Example: `FROM llama3.2` or `FROM ./my-model.gguf`
    pub fn from(mut self, base_model: &str) -> Self {
        self.lines.push(format!("FROM {}", base_model));
        self
    }

    /// Set a `PARAMETER` instruction. Sets a model option like temperature or context window.
    /// Example: `.parameter("num_ctx", "4096")`
    pub fn parameter(mut self, key: &str, value: &str) -> Self {
        self.lines.push(format!("PARAMETER {} {}", key, value));
        self
    }

    /// Set the `TEMPLATE` instruction. Defines the prompt template.
    /// The template usually spans multiple lines, so it is wrapped in triple quotes automatically.
    pub fn template(mut self, template: &str) -> Self {
        self.lines
            .push(format!("TEMPLATE \"\"\"{}\"\"\"", template));
        self
    }

    /// Set the `SYSTEM` instruction. Sets the custom system message.
    pub fn system(mut self, system_message: &str) -> Self {
        self.lines
            .push(format!("SYSTEM \"\"\"{}\"\"\"", system_message));
        self
    }

    /// Set the `ADAPTER` instruction. Defines the path to a LoRA adapter.
    pub fn adapter(mut self, adapter_path: &str) -> Self {
        self.lines.push(format!("ADAPTER {}", adapter_path));
        self
    }

    /// Set the `LICENSE` instruction. Includes legal license text.
    pub fn license(mut self, license_text: &str) -> Self {
        self.lines
            .push(format!("LICENSE \"\"\"{}\"\"\"", license_text));
        self
    }

    /// Set a `MESSAGE` instruction. Adds a conversational turn (role and content).
    /// Example: `.message("user", "Is Toronto in Canada?")`
    pub fn message(mut self, role: &str, content: &str) -> Self {
        self.lines.push(format!("MESSAGE {} {}", role, content));
        self
    }

    /// Set the `REQUIRES` instruction. Specifies the required Ollama version.
    pub fn requires(mut self, version: &str) -> Self {
        self.lines.push(format!("REQUIRES {}", version));
        self
    }

    /// Build the Modelfile into a final string, ready to be passed to `OllamaCreateRequest::modelfile`.
    pub fn build(self) -> String {
        self.lines.join("\n")
    }
}
