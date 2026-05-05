//! # QAI `Ollama`
//!
//! `Ollama` provider for the QAI SDK. Uses Ollama's recommended `/v1`
//! OpenAI-compatible API layer, guaranteeing native support for tool calls,
//! structured outputs, and streaming across both local servers and Ollama Cloud.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use qai_sdk::ollama::create_ollama;
//! use qai_sdk::core::types::ProviderSettings;
//!
//! // Automatically targets http://localhost:11434/v1 by default
//! let provider = create_ollama(ProviderSettings::default());
//!
//! let model = provider.chat("llama3.2");
//! ```

pub mod types;

use crate::core::types::{GenerateOptions, GenerateResult, Prompt, ProviderSettings, StreamPart};
use crate::openai::OpenAIModel;
use async_trait::async_trait;
use futures::stream::BoxStream;
use reqwest::Client;

pub struct OllamaModel {
    pub inner: OpenAIModel,
}

impl OllamaModel {
    #[must_use]
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            inner: OpenAIModel {
                api_key,
                base_url,
                client: Client::new(),
            },
        }
    }
}

#[async_trait]
impl crate::core::LanguageModel for OllamaModel {
    #[tracing::instrument(skip(self, prompt), fields(model = options.model_id))]
    async fn generate(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
    ) -> crate::core::Result<GenerateResult> {
        self.inner.generate(prompt, options).await
    }

    async fn generate_stream(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
    ) -> crate::core::Result<BoxStream<'static, StreamPart>> {
        self.inner.generate_stream(prompt, options).await
    }
}

#[async_trait]
impl crate::core::EmbeddingModel for OllamaModel {
    #[tracing::instrument(skip(self, texts), fields(model = options.model_id))]
    async fn embed(
        &self,
        texts: Vec<String>,
        options: crate::core::types::EmbeddingOptions,
    ) -> crate::core::Result<crate::core::types::EmbeddingResult> {
        let embedding_model = crate::openai::embedding::OpenAIEmbeddingModel {
            api_key: self.inner.api_key.clone(),
            base_url: self.inner.base_url.clone(),
            client: self.inner.client.clone(),
        };
        embedding_model.embed(texts, options).await
    }
}

// --- Provider Factory ---

/// `Ollama` provider with configurable settings.
pub struct OllamaProvider {
    settings: ProviderSettings,
}

impl OllamaProvider {
    /// Creates a chat language model.
    #[must_use]
    pub fn chat(&self, _model_id: &str) -> OllamaModel {
        let api_key = self
            .settings
            .api_key
            .clone()
            .or_else(|| std::env::var("OLLAMA_API_KEY").ok())
            .unwrap_or_default();

        let base_url = self
            .settings
            .base_url
            .clone()
            .or_else(|| std::env::var("OLLAMA_BASE_URL").ok())
            .unwrap_or_else(|| {
                if !api_key.is_empty() {
                    types::DEFAULT_OLLAMA_CLOUD_URL.to_string()
                } else {
                    types::DEFAULT_OLLAMA_LOCAL_URL.to_string()
                }
            });

        OllamaModel::new(api_key, base_url)
    }

    /// Alias for `chat`.
    #[must_use]
    pub fn language_model(&self, model_id: &str) -> OllamaModel {
        self.chat(model_id)
    }

    /// Creates an embedding model.
    #[must_use]
    pub fn embedding(&self, model_id: &str) -> OllamaModel {
        self.chat(model_id)
    }

    /// Internal helper to construct the base URL for native `/api/` endpoints
    /// by stripping the `/v1` compatibility suffix if present.
    fn get_api_base(&self) -> String {
        let base = self
            .settings
            .base_url
            .clone()
            .or_else(|| std::env::var("OLLAMA_BASE_URL").ok())
            .unwrap_or_else(|| types::DEFAULT_OLLAMA_LOCAL_URL.to_string());
        
        if base.ends_with("/v1") {
            base[..base.len() - 3].to_string()
        } else if base.ends_with("/v1/") {
            base[..base.len() - 4].to_string()
        } else {
            base
        }
    }

    /// List models that are available locally.
    pub async fn list_models(&self) -> crate::core::Result<types::OllamaListResponse> {
        let url = format!("{}/api/tags", self.get_api_base());
        let client = Client::new();
        let res = client.get(&url).send().await.map_err(|e| crate::core::error::ProviderError::Network(e.to_string()))?;
        if !res.status().is_success() {
            return Err(crate::core::error::ProviderError::InvalidResponse(format!("Ollama API error: {}", res.status())));
        }
        res.json().await.map_err(|e| crate::core::error::ProviderError::InvalidResponse(e.to_string()))
    }

    /// List models that are currently running in memory.
    pub async fn list_running_models(&self) -> crate::core::Result<types::OllamaPsResponse> {
        let url = format!("{}/api/ps", self.get_api_base());
        let client = Client::new();
        let res = client.get(&url).send().await.map_err(|e| crate::core::error::ProviderError::Network(e.to_string()))?;
        if !res.status().is_success() {
            return Err(crate::core::error::ProviderError::InvalidResponse(format!("Ollama API error: {}", res.status())));
        }
        res.json().await.map_err(|e| crate::core::error::ProviderError::InvalidResponse(e.to_string()))
    }

    /// Show detailed information for a specific model.
    pub async fn show_model_info(&self, req: types::OllamaShowRequest) -> crate::core::Result<types::OllamaShowResponse> {
        let url = format!("{}/api/show", self.get_api_base());
        let client = Client::new();
        let res = client.post(&url).json(&req).send().await.map_err(|e| crate::core::error::ProviderError::Network(e.to_string()))?;
        if !res.status().is_success() {
            return Err(crate::core::error::ProviderError::InvalidResponse(format!("Ollama API error: {}", res.status())));
        }
        res.json().await.map_err(|e| crate::core::error::ProviderError::InvalidResponse(e.to_string()))
    }

    /// Copy a model. Creates a model with another name from an existing model.
    pub async fn copy_model(&self, req: types::OllamaCopyRequest) -> crate::core::Result<()> {
        let url = format!("{}/api/copy", self.get_api_base());
        let client = Client::new();
        let res = client.post(&url).json(&req).send().await.map_err(|e| crate::core::error::ProviderError::Network(e.to_string()))?;
        if !res.status().is_success() {
            return Err(crate::core::error::ProviderError::InvalidResponse(format!("Ollama API error: {}", res.status())));
        }
        Ok(())
    }

    /// Delete a model and its data.
    pub async fn delete_model(&self, req: types::OllamaDeleteRequest) -> crate::core::Result<()> {
        let url = format!("{}/api/delete", self.get_api_base());
        let client = Client::new();
        let res = client.delete(&url).json(&req).send().await.map_err(|e| crate::core::error::ProviderError::Network(e.to_string()))?;
        if !res.status().is_success() {
            return Err(crate::core::error::ProviderError::InvalidResponse(format!("Ollama API error: {}", res.status())));
        }
        Ok(())
    }

    /// Create a model from a Modelfile (blocking non-streaming implementation).
    pub async fn create_model(&self, mut req: types::OllamaCreateRequest) -> crate::core::Result<()> {
        req.stream = Some(false); // Force non-streaming for simplicity in this helper
        let url = format!("{}/api/create", self.get_api_base());
        let client = Client::new();
        let res = client.post(&url).json(&req).send().await.map_err(|e| crate::core::error::ProviderError::Network(e.to_string()))?;
        if !res.status().is_success() {
            return Err(crate::core::error::ProviderError::InvalidResponse(format!("Ollama API error: {}", res.status())));
        }
        Ok(())
    }

    /// Pull a model from a registry (blocking non-streaming implementation).
    pub async fn pull_model(&self, mut req: types::OllamaPullRequest) -> crate::core::Result<types::OllamaPullResponse> {
        req.stream = Some(false); // Force non-streaming for simplicity
        let url = format!("{}/api/pull", self.get_api_base());
        let client = Client::new();
        let res = client.post(&url).json(&req).send().await.map_err(|e| crate::core::error::ProviderError::Network(e.to_string()))?;
        if !res.status().is_success() {
            return Err(crate::core::error::ProviderError::InvalidResponse(format!("Ollama API error: {}", res.status())));
        }
        res.json().await.map_err(|e| crate::core::error::ProviderError::InvalidResponse(e.to_string()))
    }

    /// Push a model to a registry (blocking non-streaming implementation).
    pub async fn push_model(&self, mut req: types::OllamaPushRequest) -> crate::core::Result<()> {
        req.stream = Some(false); // Force non-streaming for simplicity
        let url = format!("{}/api/push", self.get_api_base());
        let client = Client::new();
        let res = client.post(&url).json(&req).send().await.map_err(|e| crate::core::error::ProviderError::Network(e.to_string()))?;
        if !res.status().is_success() {
            return Err(crate::core::error::ProviderError::InvalidResponse(format!("Ollama API error: {}", res.status())));
        }
        Ok(())
    }

    /// Get the version of the Ollama server.
    pub async fn get_version(&self) -> crate::core::Result<types::OllamaVersionResponse> {
        let url = format!("{}/api/version", self.get_api_base());
        let client = Client::new();
        let res = client.get(&url).send().await.map_err(|e| crate::core::error::ProviderError::Network(e.to_string()))?;
        if !res.status().is_success() {
            return Err(crate::core::error::ProviderError::InvalidResponse(format!("Ollama API error: {}", res.status())));
        }
        res.json().await.map_err(|e| crate::core::error::ProviderError::InvalidResponse(e.to_string()))
    }

    /// Perform a web search.
    pub async fn web_search(&self, req: types::WebSearchRequest) -> crate::core::Result<types::WebSearchResponse> {
        let url = format!("{}/api/web_search", self.get_api_base());
        let client = Client::new();
        
        let mut request_builder = client.post(&url);
        let api_key = self.resolve_api_key();
        if !api_key.is_empty() {
            request_builder = request_builder.bearer_auth(api_key);
        }

        let res = request_builder.json(&req).send().await.map_err(|e| crate::core::error::ProviderError::Network(e.to_string()))?;
        if !res.status().is_success() {
            return Err(crate::core::error::ProviderError::InvalidResponse(format!("Ollama API error: {}", res.status())));
        }
        res.json().await.map_err(|e| crate::core::error::ProviderError::InvalidResponse(e.to_string()))
    }

    /// Fetch a web page's content.
    pub async fn web_fetch(&self, req: types::WebFetchRequest) -> crate::core::Result<types::WebFetchResponse> {
        let url = format!("{}/api/web_fetch", self.get_api_base());
        let client = Client::new();

        let mut request_builder = client.post(&url);
        let api_key = self.resolve_api_key();
        if !api_key.is_empty() {
            request_builder = request_builder.bearer_auth(api_key);
        }

        let res = request_builder.json(&req).send().await.map_err(|e| crate::core::error::ProviderError::Network(e.to_string()))?;
        if !res.status().is_success() {
            return Err(crate::core::error::ProviderError::InvalidResponse(format!("Ollama API error: {}", res.status())));
        }
        res.json().await.map_err(|e| crate::core::error::ProviderError::InvalidResponse(e.to_string()))
    }
}

/// Create an `Ollama` provider instance with the given settings.
#[must_use]
pub fn create_ollama(settings: ProviderSettings) -> OllamaProvider {
    OllamaProvider { settings }
}

impl crate::core::registry::Provider for OllamaProvider {
    fn language_model(&self, model_id: &str) -> Option<Box<dyn crate::core::LanguageModel>> {
        Some(Box::new(self.chat(model_id)))
    }

    fn embedding_model(&self, model_id: &str) -> Option<Box<dyn crate::core::EmbeddingModel>> {
        Some(Box::new(self.embedding(model_id)))
    }
}
