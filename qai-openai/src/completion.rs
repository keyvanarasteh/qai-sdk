use anyhow::anyhow;
use async_trait::async_trait;
use qai_core::types::{CompletionOptions, CompletionResult, Usage};
use qai_core::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// OpenAI completion model (legacy completions API).
pub struct OpenAICompletionModel {
    pub api_key: String,
    pub base_url: String,
    pub client: Client,
}

impl OpenAICompletionModel {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
            client: Client::new(),
        }
    }
}

#[derive(Serialize)]
struct OpenAICompletionRequest {
    model: String,
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    suffix: Option<String>,
}

#[derive(Deserialize)]
struct OpenAICompletionResponse {
    choices: Vec<OpenAICompletionChoice>,
    usage: OpenAICompletionUsage,
}

#[derive(Deserialize)]
struct OpenAICompletionChoice {
    text: String,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct OpenAICompletionUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

#[async_trait]
impl qai_core::CompletionModel for OpenAICompletionModel {
    async fn complete(&self, options: CompletionOptions) -> qai_core::Result<CompletionResult> {
        let request = OpenAICompletionRequest {
            model: options.model_id,
            prompt: options.prompt,
            max_tokens: options.max_tokens,
            temperature: options.temperature,
            top_p: options.top_p,
            stop: options.stop,
            suffix: options.suffix,
        };

        let resp = self
            .client
            .post(format!("{}/completions", self.base_url))
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await?;
            return Err(anyhow!("OpenAI Completion API error: {}", error_text).into());
        }

        let completion_resp: OpenAICompletionResponse = resp.json().await?;
        let choice =
            completion_resp
                .choices
                .first()
                .ok_or_else(|| -> qai_core::ProviderError {
                    qai_core::ProviderError::Other(anyhow::anyhow!(
                        "No completion choices returned"
                    ))
                })?;

        Ok(CompletionResult {
            text: choice.text.clone(),
            usage: Usage {
                prompt_tokens: completion_resp.usage.prompt_tokens,
                completion_tokens: completion_resp.usage.completion_tokens,
            },
            finish_reason: choice
                .finish_reason
                .clone()
                .unwrap_or_else(|| "stop".to_string()),
        })
    }
}
