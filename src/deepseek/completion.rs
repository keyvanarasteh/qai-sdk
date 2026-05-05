use crate::core::types::{CompletionOptions, CompletionResult, Usage};
use crate::deepseek::types::DeepSeekCompletionRequest;
use anyhow::anyhow;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

/// DeepSeek completion model (FIM completions API).
pub struct DeepSeekCompletionModel {
    pub api_key: String,
    pub base_url: String,
    pub client: Client,
}

impl DeepSeekCompletionModel {
    #[must_use]
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            api_key,
            base_url,
            client: Client::new(),
        }
    }
}

#[derive(Deserialize)]
struct DeepSeekCompletionResponse {
    choices: Vec<DeepSeekCompletionChoice>,
    usage: DeepSeekCompletionUsage,
}

#[derive(Deserialize)]
struct DeepSeekCompletionChoice {
    text: String,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct DeepSeekCompletionUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    #[serde(default)]
    prompt_cache_hit_tokens: u32,
    #[serde(default)]
    prompt_cache_miss_tokens: u32,
}

#[async_trait]
impl crate::core::CompletionModel for DeepSeekCompletionModel {
    async fn complete(&self, options: CompletionOptions) -> crate::core::Result<CompletionResult> {
        let request = DeepSeekCompletionRequest {
            model: options.model_id.clone(),
            prompt: options.prompt.clone(),
            suffix: options.suffix.clone(),
            max_tokens: options.max_tokens,
            temperature: options.temperature,
            top_p: options.top_p,
            stop: options.stop.clone(),
            stream: Some(false),
        };

        let url = format!("{}/beta/completions", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("DeepSeek API Error: {}", error_text).into());
        }

        let mut ds_response: DeepSeekCompletionResponse = response.json().await?;

        if let Some(choice) = ds_response.choices.pop() {
            Ok(CompletionResult {
                text: choice.text,
                usage: Usage {
                    prompt_tokens: ds_response.usage.prompt_tokens,
                    completion_tokens: ds_response.usage.completion_tokens,
                    cache_hit_tokens: Some(ds_response.usage.prompt_cache_hit_tokens),
                    cache_miss_tokens: Some(ds_response.usage.prompt_cache_miss_tokens),
                },
                finish_reason: choice.finish_reason.unwrap_or_else(|| "stop".to_string()),
            })
        } else {
            Err(anyhow!("No choices in DeepSeek response").into())
        }
    }
}
