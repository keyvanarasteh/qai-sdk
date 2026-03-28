use async_trait::async_trait;
use qai_core::types::{ImageGenerateOptions, ImageGenerateResult};
use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// OpenAI image generation model.
pub struct OpenAIImageModel {
    pub api_key: String,
    pub base_url: String,
    pub client: Client,
}

impl OpenAIImageModel {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
            client: Client::new(),
        }
    }
}

#[derive(Serialize)]
struct OpenAIImageRequest {
    model: String,
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    quality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<String>,
}

#[derive(Deserialize)]
struct OpenAIImageResponse {
    data: Vec<OpenAIImageData>,
}

#[derive(Deserialize)]
struct OpenAIImageData {
    #[serde(default)]
    b64_json: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    revised_prompt: Option<String>,
}

#[async_trait]
impl qai_core::ImageModel for OpenAIImageModel {
    async fn generate(&self, options: ImageGenerateOptions) -> Result<ImageGenerateResult> {
        let response_format = options.response_format.clone().unwrap_or_else(|| "b64_json".to_string());

        let request = OpenAIImageRequest {
            model: options.model_id,
            prompt: options.prompt,
            n: options.n,
            size: options.size,
            quality: options.quality,
            response_format: Some(response_format.clone()),
        };

        let resp = self.client.post(&format!("{}/images/generations", self.base_url))
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await?;
            return Err(anyhow!("OpenAI Image API error: {}", error_text));
        }

        let img_resp: OpenAIImageResponse = resp.json().await?;

        let images: Vec<String> = img_resp.data.iter().map(|d| {
            d.b64_json.clone().or_else(|| d.url.clone()).unwrap_or_default()
        }).collect();

        let revised_prompt = img_resp.data.first().and_then(|d| d.revised_prompt.clone());

        Ok(ImageGenerateResult {
            images,
            revised_prompt,
        })
    }
}
