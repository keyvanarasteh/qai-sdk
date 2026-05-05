//! # xAI Video Generation
//!
//! xAI Video generation provider implementation.

use crate::core::types::{VideoGenerateOptions, VideoGenerateResult};
use anyhow::anyhow;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

/// xAI video generation model.
pub struct XaiVideoModel {
    pub api_key: String,
    pub base_url: String,
    pub client: Client,
}

impl XaiVideoModel {
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
struct XaiVideoRequest {
    model: String,
    prompt: String,
}

#[derive(Deserialize)]
struct XaiVideoResponse {
    request_id: String,
}

#[derive(Deserialize)]
struct XaiVideoStatusResponse {
    status: String,
    video: Option<XaiVideoDetail>,
}

#[derive(Deserialize)]
struct XaiVideoDetail {
    url: String,
}

#[async_trait]
impl crate::core::VideoModel for XaiVideoModel {
    async fn generate(&self, options: VideoGenerateOptions) -> crate::core::Result<VideoGenerateResult> {
        let request = XaiVideoRequest {
            model: options.model_id,
            prompt: options.prompt,
        };

        let resp = self
            .client
            .post(format!("{}/videos/generations", self.base_url))
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await?;
            return Err(anyhow!("xAI Video Generation API error: {error_text}").into());
        }

        let submission: XaiVideoResponse = resp.json().await?;
        let request_id = submission.request_id;

        // Poll for completion (up to 2 minutes)
        for _ in 0..24 {
            sleep(Duration::from_secs(5)).await;

            let status_resp = self
                .client
                .get(format!("{}/videos/{}", self.base_url, request_id))
                .header("Authorization", &format!("Bearer {}", self.api_key))
                .send()
                .await?;

            if !status_resp.status().is_success() {
                continue;
            }

            let status: XaiVideoStatusResponse = status_resp.json().await?;
            if status.status == "done" {
                if let Some(video) = status.video {
                    return Ok(VideoGenerateResult {
                        url: Some(video.url),
                        data: None,
                        revision: Some(request_id),
                    });
                }
            } else if status.status == "failed" {
                return Err(anyhow!("xAI Video Generation failed for ID: {}", request_id).into());
            }
        }

        Err(anyhow!("xAI Video Generation timed out for ID: {}", request_id).into())
    }
}
