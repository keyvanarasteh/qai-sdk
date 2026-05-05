use crate::core::types::{MusicGenerateOptions, MusicGenerateResult};
use anyhow::anyhow;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Google Generative AI music generation model (Lyria).
pub struct GoogleMusicModel {
    pub api_key: String,
    pub base_url: String,
    pub client: Client,
}

impl GoogleMusicModel {
    #[must_use]
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            api_key,
            base_url,
            client: Client::new(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GoogleMusicRequest {
    instances: Vec<GoogleMusicInstance>,
    parameters: GoogleMusicParameters,
}

#[derive(Serialize)]
struct GoogleMusicInstance {
    prompt: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GoogleMusicParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    sample_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_seconds: Option<u32>,
}

#[derive(Deserialize)]
struct GoogleMusicResponse {
    #[serde(default)]
    predictions: Vec<GoogleMusicPrediction>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoogleMusicPrediction {
    #[serde(default)]
    bytes_base64_encoded: Option<String>,
}

#[async_trait]
impl crate::core::MusicModel for GoogleMusicModel {
    async fn generate(
        &self,
        options: MusicGenerateOptions,
    ) -> crate::core::Result<MusicGenerateResult> {
        let request = GoogleMusicRequest {
            instances: vec![GoogleMusicInstance {
                prompt: options.prompt,
            }],
            parameters: GoogleMusicParameters {
                sample_count: options.n,
                duration_seconds: options.duration,
            },
        };

        let url = format!(
            "{}/models/{}:predict?key={}",
            self.base_url, options.model_id, self.api_key
        );

        let resp = self.client.post(&url).json(&request).send().await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await?;
            return Err(anyhow!("Google Music API error: {error_text}").into());
        }

        let music_resp: GoogleMusicResponse = resp.json().await?;

        let audio: Vec<String> = music_resp
            .predictions
            .iter()
            .filter_map(|p| p.bytes_base64_encoded.clone())
            .collect();

        Ok(MusicGenerateResult { audio })
    }
}
