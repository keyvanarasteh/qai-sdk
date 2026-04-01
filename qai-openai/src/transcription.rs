use anyhow::anyhow;
use async_trait::async_trait;
use qai_core::types::{TranscriptionOptions, TranscriptionResult};
use qai_core::Result;
use reqwest::multipart::{Form, Part};
use reqwest::Client;
use serde::Deserialize;

/// OpenAI transcription (speech-to-text) model.
pub struct OpenAITranscriptionModel {
    pub api_key: String,
    pub base_url: String,
    pub client: Client,
}

impl OpenAITranscriptionModel {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
            client: Client::new(),
        }
    }
}

#[derive(Deserialize)]
struct OpenAITranscriptionResponse {
    text: String,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    duration: Option<f64>,
}

#[async_trait]
impl qai_core::TranscriptionModel for OpenAITranscriptionModel {
    async fn transcribe(
        &self,
        options: TranscriptionOptions,
    ) -> qai_core::Result<TranscriptionResult> {
        let audio_part = Part::bytes(options.audio)
            .file_name("audio.mp3")
            .mime_str("audio/mpeg")?;

        let mut form = Form::new()
            .text("model", options.model_id)
            .text("response_format", "verbose_json")
            .part("file", audio_part);

        if let Some(language) = options.language {
            form = form.text("language", language);
        }
        if let Some(prompt) = options.prompt {
            form = form.text("prompt", prompt);
        }
        if let Some(temperature) = options.temperature {
            form = form.text("temperature", temperature.to_string());
        }

        let resp = self
            .client
            .post(format!("{}/audio/transcriptions", self.base_url))
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await?;
            return Err(anyhow!("OpenAI Transcription API error: {}", error_text).into());
        }

        let transcription: OpenAITranscriptionResponse = resp.json().await?;

        Ok(TranscriptionResult {
            text: transcription.text,
            language: transcription.language,
            duration: transcription.duration,
        })
    }
}
