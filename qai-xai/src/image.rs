use async_trait::async_trait;
use qai_core::types::{ImageGenerateOptions, ImageGenerateResult};
use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// xAI image generation model.
pub struct XaiImageModel {
    pub api_key: String,
    pub base_url: String,
    pub client: Client,
}

impl XaiImageModel {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.x.ai/v1".to_string(),
            client: Client::new(),
        }
    }
}

#[derive(Serialize)]
struct XaiImageRequest {
    model: String,
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<String>,
    response_format: String,
}

#[derive(Deserialize)]
struct XaiImageResponse {
    data: Vec<XaiImageData>,
}

#[derive(Deserialize)]
struct XaiImageData {
    #[serde(default)]
    b64_json: Option<String>,
    #[serde(default)]
    url: Option<String>,
}

#[async_trait]
impl qai_core::ImageModel for XaiImageModel {
    async fn generate(&self, options: ImageGenerateOptions) -> Result<ImageGenerateResult> {
        let response_format = options.response_format.clone().unwrap_or_else(|| "b64_json".to_string());

        let request = XaiImageRequest {
            model: options.model_id,
            prompt: options.prompt,
            n: options.n,
            size: options.size,
            response_format,
        };

        let resp = self.client.post(&format!("{}/images/generations", self.base_url))
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await?;
            return Err(anyhow!("xAI Image API error: {}", error_text));
        }

        let img_resp: XaiImageResponse = resp.json().await?;

        let images: Vec<String> = img_resp.data.iter().map(|d| {
            d.b64_json.clone().or_else(|| d.url.clone()).unwrap_or_default()
        }).collect();

        Ok(ImageGenerateResult {
            images,
            revised_prompt: None,
        })
    }
}
