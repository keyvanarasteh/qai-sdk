use anyhow::anyhow;
use async_trait::async_trait;
use qai_core::types::{SpeechOptions, SpeechResult};
use qai_core::Result;
use reqwest::Client;
use serde::Serialize;

/// OpenAI speech (text-to-speech) model.
pub struct OpenAISpeechModel {
    pub api_key: String,
    pub base_url: String,
    pub client: Client,
}

impl OpenAISpeechModel {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
            client: Client::new(),
        }
    }
}

#[derive(Serialize)]
struct OpenAISpeechRequest {
    model: String,
    input: String,
    voice: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    speed: Option<f32>,
}

#[async_trait]
impl qai_core::SpeechModel for OpenAISpeechModel {
    async fn synthesize(&self, options: SpeechOptions) -> qai_core::Result<SpeechResult> {
        let request = OpenAISpeechRequest {
            model: options.model_id,
            input: options.input,
            voice: options.voice,
            response_format: options.response_format,
            speed: options.speed,
        };

        let resp = self
            .client
            .post(format!("{}/audio/speech", self.base_url))
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await?;
            return Err(anyhow!("OpenAI Speech API error: {}", error_text).into());
        }

        let audio = resp.bytes().await?.to_vec();

        Ok(SpeechResult { audio })
    }
}
