use crate::core::types::{SpeechOptions, SpeechResult};
use crate::google::types::{
    GoogleContent, GoogleGenerationConfig, GooglePart, GooglePrebuiltVoiceConfig, GoogleRequest,
    GoogleResponse, GoogleSpeechConfig, GoogleVoiceConfig,
};
use anyhow::anyhow;
use async_trait::async_trait;
use reqwest::Client;

/// Google Gemini audio generation model (TTS).
pub struct GoogleSpeechModel {
    pub api_key: String,
    pub base_url: String,
    pub client: Client,
}

impl GoogleSpeechModel {
    #[must_use]
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            api_key,
            base_url,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl crate::core::SpeechModel for GoogleSpeechModel {
    async fn synthesize(&self, options: SpeechOptions) -> crate::core::Result<SpeechResult> {
        let voice_name = if options.voice.is_empty() {
            "Aoede".to_string()
        } else {
            options.voice.clone()
        };

        let generation_config = GoogleGenerationConfig {
            response_modalities: Some(vec!["AUDIO".to_string()]),
            speech_config: Some(GoogleSpeechConfig {
                voice_config: GoogleVoiceConfig {
                    prebuilt_voice_config: GooglePrebuiltVoiceConfig { voice_name },
                },
            }),
            max_output_tokens: None,
            temperature: None,
            top_p: None,
            top_k: None,
            stop_sequences: None,
            response_mime_type: None,
            response_schema: None,
            thinking_config: None,
        };

        let request = GoogleRequest {
            contents: vec![GoogleContent {
                role: "user".to_string(),
                parts: vec![GooglePart::Text {
                    text: options.input,
                    thought: None,
                }],
            }],
            system_instruction: None,
            generation_config: Some(generation_config),
            tools: None,
        };

        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.base_url, options.model_id, self.api_key
        );

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Google API error: {}", error_text).into());
        }

        let mut g_response: GoogleResponse = response.json().await?;

        if let Some(candidate) = g_response.candidates.pop() {
            for part in candidate.content.parts {
                if let GooglePart::InlineData { mime_type: _, data } = part {
                    use base64::{engine::general_purpose, Engine as _};
                    let audio_bytes = general_purpose::STANDARD
                        .decode(data)
                        .map_err(|e| anyhow!("Failed to decode base64 audio: {}", e))?;

                    return Ok(SpeechResult { audio: audio_bytes });
                }
            }
            Err(anyhow!("No audio inlineData found in Google response").into())
        } else {
            Err(anyhow!("No candidates in Google response").into())
        }
    }
}
