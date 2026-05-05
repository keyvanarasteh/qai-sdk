use crate::core::types::{VideoGenerateOptions, VideoGenerateResult};
use anyhow::anyhow;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Google Generative AI video generation model (Veo).
pub struct GoogleVideoModel {
    pub api_key: String,
    pub base_url: String,
    pub client: Client,
}

impl GoogleVideoModel {
    #[must_use]
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
struct GoogleVideoRequest {
    instances: Vec<GoogleVideoInstance>,
    parameters: GoogleVideoParameters,
}

#[derive(Serialize)]
struct GoogleVideoInstance {
    prompt: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GoogleVideoParameters {
    // Veo parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    sample_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    aspect_ratio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_seconds: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fps: Option<u32>,
}

#[derive(Deserialize)]
struct GoogleVideoResponse {
    #[serde(default)]
    predictions: Vec<GoogleVideoPrediction>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoogleVideoPrediction {
    #[serde(default)]
    bytes_base64_encoded: Option<String>,
    #[serde(default)]
    gcs_uri: Option<String>,
}

#[async_trait]
impl crate::core::VideoModel for GoogleVideoModel {
    async fn generate(
        &self,
        options: VideoGenerateOptions,
    ) -> crate::core::Result<VideoGenerateResult> {
        let request = GoogleVideoRequest {
            instances: vec![GoogleVideoInstance {
                prompt: options.prompt,
            }],
            parameters: GoogleVideoParameters {
                sample_count: options.n,
                aspect_ratio: options.size,
                duration_seconds: options.duration,
                fps: options.fps,
            },
        };

        let url = format!(
            "{}/models/{}:predict?key={}",
            self.base_url, options.model_id, self.api_key
        );

        let resp = self.client.post(&url).json(&request).send().await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await?;
            return Err(anyhow!("Google Video API error: {error_text}").into());
        }

        let video_resp: GoogleVideoResponse = resp.json().await?;

        let videos: Vec<String> = video_resp
            .predictions
            .iter()
            .filter_map(|p| p.bytes_base64_encoded.clone())
            .collect();

        // Note: Some models might return a GCS URI instead of bytes if it's long-running.
        // For now, we assume bytes if available.

        Ok(VideoGenerateResult {
            videos,
            revised_prompt: None,
        })
    }
}
