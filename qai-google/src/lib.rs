pub mod types;

use async_trait::async_trait;
use qai_core::types::{Content, GenerateOptions, GenerateResult, Prompt, Role, Usage, ImageSource};
use crate::types::{GoogleRequest, GoogleResponse, GoogleContent, GooglePart, GoogleGenerationConfig};
use anyhow::{Result, anyhow};
use reqwest::Client;

pub struct GoogleModel {
    pub api_key: String,
    pub base_url: String,
    pub client: Client,
}

impl GoogleModel {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl qai_core::LanguageModel for GoogleModel {
    async fn generate(&self, prompt: Prompt, options: GenerateOptions) -> Result<GenerateResult> {
        let mut contents = Vec::new();
        let mut system_instruction = None;

        for msg in prompt.messages {
            let role = match msg.role {
                Role::System => {
                    let mut parts = Vec::new();
                    for content in msg.content {
                        if let Content::Text { text } = content {
                            parts.push(GooglePart::Text { text });
                        }
                    }
                    system_instruction = Some(GoogleContent {
                        role: "system".to_string(),
                        parts,
                    });
                    continue;
                }
                Role::User => "user",
                Role::Assistant => "model",
                Role::Tool => "user", // Google treats tool results as user parts with functionResponses
            };

            let mut parts = Vec::new();
            for content in msg.content {
                match content {
                    Content::Text { text } => {
                        parts.push(GooglePart::Text { text });
                    }
                    Content::Image { source } => {
                        if let ImageSource::Base64 { media_type, data } = source {
                            parts.push(GooglePart::InlineData {
                                mime_type: media_type,
                                data,
                            });
                        }
                    }
                    Content::ToolCall { name, arguments, .. } => {
                        parts.push(GooglePart::FunctionCall {
                            name,
                            args: arguments,
                        });
                    }
                    Content::ToolResult { id, result } => {
                        // For Google, we need the function name. 
                        // Our core ToolResult only has ID. 
                        // This is a mapping issue in our core types if we want to support Google fully.
                        // For now, let's assume the ID is the name or we need to extend core.
                        parts.push(GooglePart::FunctionResponse {
                            name: id, // Temporary simplification
                            response: result,
                        });
                    }
                }
            }

            contents.push(GoogleContent {
                role: role.to_string(),
                parts,
            });
        }

        let request = GoogleRequest {
            contents,
            system_instruction,
            generation_config: Some(GoogleGenerationConfig {
                max_output_tokens: options.max_tokens,
                temperature: options.temperature,
                top_p: options.top_p,
                top_k: None,
                stop_sequences: options.stop_sequences,
            }),
        };

        let url = format!("{}/models/{}:generateContent?key={}", self.base_url, options.model_id, self.api_key);

        let response = self.client.post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Google API error: {}", error_text));
        }

        let google_response: GoogleResponse = response.json().await?;

        let candidate = google_response.candidates.get(0).ok_or_else(|| anyhow!("No candidates returned from Google"))?;
        
        let text = candidate.content.parts.iter()
            .filter_map(|p| if let GooglePart::Text { text } = p { Some(text.clone()) } else { None })
            .collect::<Vec<_>>()
            .join("");

        Ok(GenerateResult {
            text,
            usage: Usage {
                prompt_tokens: google_response.usage_metadata.prompt_token_count,
                completion_tokens: google_response.usage_metadata.candidates_token_count,
            },
            finish_reason: candidate.finish_reason.clone().unwrap_or_else(|| "unknown".to_string()),
        })
    }
}
