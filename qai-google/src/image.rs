use anyhow::{anyhow, Result};
use async_trait::async_trait;
use qai_core::types::{ImageGenerateOptions, ImageGenerateResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Google Generative AI image generation model.
pub struct GoogleImageModel {
    pub api_key: String,
    pub base_url: String,
    pub client: Client,
}

impl GoogleImageModel {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            client: Client::new(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GoogleImageRequest {
    instances: Vec<GoogleImageInstance>,
    parameters: GoogleImageParameters,
}

#[derive(Serialize)]
struct GoogleImageInstance {
    prompt: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GoogleImageParameters {
    sample_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    aspect_ratio: Option<String>,
}

#[derive(Deserialize)]
struct GoogleImageResponse {
    #[serde(default)]
    predictions: Vec<GoogleImagePrediction>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoogleImagePrediction {
    #[serde(default)]
    bytes_base64_encoded: Option<String>,
}

#[async_trait]
impl qai_core::ImageModel for GoogleImageModel {
    async fn generate(&self, options: ImageGenerateOptions) -> Result<ImageGenerateResult> {
        let n = options.n.unwrap_or(1);

        let request = GoogleImageRequest {
            instances: vec![GoogleImageInstance {
                prompt: options.prompt,
            }],
            parameters: GoogleImageParameters {
                sample_count: n,
                aspect_ratio: options.size, // Map size to aspect_ratio for Google
            },
        };

        let url = format!(
            "{}/models/{}:predict?key={}",
            self.base_url, options.model_id, self.api_key
        );

        let resp = self.client.post(&url).json(&request).send().await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await?;
            return Err(anyhow!("Google Image API error: {}", error_text));
        }

        let img_resp: GoogleImageResponse = resp.json().await?;

        let images: Vec<String> = img_resp
            .predictions
            .iter()
            .filter_map(|p| p.bytes_base64_encoded.clone())
            .collect();

        Ok(ImageGenerateResult {
            images,
            revised_prompt: None,
        })
    }
}
