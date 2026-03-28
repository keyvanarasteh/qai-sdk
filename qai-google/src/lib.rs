pub mod types;
#[cfg(test)]
mod tests;

use async_trait::async_trait;
use qai_core::types::{Content, GenerateOptions, GenerateResult, Prompt, Role, Usage, ImageSource, FileSource, StreamPart};
use crate::types::{GoogleRequest, GoogleResponse, GoogleContent, GooglePart, GoogleGenerationConfig, GoogleTool, GoogleFunctionDeclaration};
use anyhow::{Result, anyhow};
use reqwest::Client;
use futures::stream::BoxStream;
use futures_util::StreamExt;
use eventsource_stream::Eventsource;

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
        let request = self.prepare_request(prompt, &options)?;

        let url = format!("{}/models/{}:generateContent?key={}", self.base_url, options.model_id, self.api_key);

        let response = self.client.post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Google API error: {}", error_text));
        }

        let headers = response.headers().clone();
        let google_response: GoogleResponse = response.json().await?;

        let mut usage = Usage {
            prompt_tokens: google_response.usage_metadata.prompt_token_count,
            completion_tokens: google_response.usage_metadata.candidates_token_count,
        };

        // Header extraction as fallback/supplement
        if let Some(header_usage) = Usage::from_headers(&headers) {
            usage = header_usage;
        }

        let candidate = google_response.candidates.get(0).ok_or_else(|| anyhow!("No candidates returned from Google"))?;
        
        let text = candidate.content.parts.iter()
            .filter_map(|p| if let GooglePart::Text { text } = p { Some(text.clone()) } else { None })
            .collect::<Vec<_>>()
            .join("");

        Ok(GenerateResult {
            text,
            usage,
            finish_reason: candidate.finish_reason.clone().unwrap_or_else(|| "stop".to_string()),
        })
    }

    async fn generate_stream(
        &self,
        prompt: Prompt,
        options: GenerateOptions,
    ) -> Result<BoxStream<'static, StreamPart>> {
        let request = self.prepare_request(prompt, &options)?;
        let url = format!("{}/models/{}:streamGenerateContent?alt=sse&key={}", self.base_url, options.model_id, self.api_key);

        let response = self.client.post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Google API error: {}", error_text));
        }

        let mut event_stream = response.bytes_stream().eventsource();

        let stream = async_stream::stream! {
            while let Some(event) = event_stream.next().await {
                match event {
                    Ok(event) => {
                        let parsed: Result<GoogleResponse, _> = serde_json::from_str(&event.data);
                        match parsed {
                            Ok(google_response) => {
                                // Gemini sends usage in the last chunk or sometimes in every chunk
                                yield StreamPart::Usage { 
                                    usage: Usage { 
                                        prompt_tokens: google_response.usage_metadata.prompt_token_count, 
                                        completion_tokens: google_response.usage_metadata.candidates_token_count 
                                    } 
                                };

                                if let Some(candidate) = google_response.candidates.get(0) {
                                    for part in &candidate.content.parts {
                                        match part {
                                            GooglePart::Text { text } => {
                                                yield StreamPart::TextDelta { delta: text.clone() };
                                            }
                                            GooglePart::FunctionCall { name, args } => {
                                                yield StreamPart::ToolCallDelta {
                                                    index: 0,
                                                    id: None,
                                                    name: Some(name.clone()),
                                                    arguments_delta: Some(args.to_string()),
                                                };
                                            }
                                            _ => {}
                                        }
                                    }

                                    if let Some(reason) = &candidate.finish_reason {
                                        yield StreamPart::Finish { finish_reason: reason.clone() };
                                    }
                                }
                            }
                            Err(e) => {
                                yield StreamPart::Error { message: e.to_string() };
                            }
                        }
                    }
                    Err(e) => {
                        yield StreamPart::Error { message: e.to_string() };
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }
}

impl GoogleModel {
    fn prepare_request(&self, prompt: Prompt, options: &GenerateOptions) -> Result<GoogleRequest> {
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
                Role::Tool => "user",
            };

            let mut parts = Vec::new();
            for content in msg.content {
                match content {
                    Content::Text { text } => {
                        parts.push(GooglePart::Text { text });
                    }
                    Content::Image { source } => {
                        let (mime_type, data) = match source {
                            ImageSource::Base64 { media_type, data } => (media_type, data),
                            _ => return Err(anyhow!("Unsupported image source for Google")),
                        };
                        parts.push(GooglePart::InlineData {
                            mime_type,
                            data,
                        });
                    }
                    Content::File { source } => {
                        let FileSource::Base64 { media_type, data } = source;
                        parts.push(GooglePart::InlineData {
                            mime_type: media_type,
                            data,
                        });
                    }
                    Content::ToolCall { name, arguments, .. } => {
                        parts.push(GooglePart::FunctionCall {
                            name,
                            args: arguments,
                        });
                    }
                    Content::ToolResult { id, result } => {
                        parts.push(GooglePart::FunctionResponse {
                            name: id,
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

        let google_tools = if options.tools.as_ref().map(|t| !t.is_empty()).unwrap_or(false) {
            Some(vec![GoogleTool {
                function_declarations: options.tools.as_ref().unwrap().iter().map(|t| GoogleFunctionDeclaration {
                    name: t.name.clone(),
                    description: t.description.clone(),
                    parameters: t.parameters.clone(),
                }).collect()
            }])
        } else {
            None
        };

        Ok(GoogleRequest {
            contents,
            system_instruction,
            generation_config: Some(GoogleGenerationConfig {
                max_output_tokens: options.max_tokens,
                temperature: options.temperature,
                top_p: options.top_p,
                top_k: None,
                stop_sequences: options.stop_sequences.clone(),
            }),
            tools: google_tools,
        })
    }
}
