//! # QAI Ollama Types
//!
//! Currently empty, as Ollama relies on the generic OpenAI-compatible payloads
//! under the hood for maximum compatibility.

/// The default local host for an Ollama server.
pub const DEFAULT_OLLAMA_LOCAL_URL: &str = "http://localhost:11434/v1";

/// The default remote host for Ollama Cloud (when an API key is provided).
pub const DEFAULT_OLLAMA_CLOUD_URL: &str = "https://api.ollama.cloud/v1";
