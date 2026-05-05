//! # xAI Real-time Voice Agent
//!
//! xAI Real-time provider implementation using WebSockets.

use crate::core::types::RealtimeEvent;
use anyhow::anyhow;
use async_trait::async_trait;
use futures_util::StreamExt;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

/// xAI real-time voice agent model.
pub struct XaiRealtimeModel {
    pub api_key: String,
    pub base_url: String,
}

impl XaiRealtimeModel {
    #[must_use]
    pub fn new(api_key: String, base_url: String) -> Self {
        // Convert https to wss for real-time
        let wss_url = base_url.replace("https://", "wss://").replace("http://", "ws://");
        Self {
            api_key,
            base_url: format!("{}/realtime", wss_url),
        }
    }
}

#[async_trait]
impl crate::core::RealtimeModel for XaiRealtimeModel {
    async fn connect(&self) -> crate::core::Result<futures::stream::BoxStream<'static, RealtimeEvent>> {
        let request = http::Request::builder()
            .uri(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .body(())
            .map_err(|e| anyhow!("Failed to build WebSocket request: {}", e))?;

        let (ws_stream, _) = connect_async(request)
            .await
            .map_err(|e| anyhow!("Failed to connect to xAI Realtime: {}", e))?;

        let (_write, read) = ws_stream.split();

        let stream = async_stream::stream! {
            let mut read = read;
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        yield RealtimeEvent::Text { text: text.to_string() };
                    }
                    Ok(Message::Binary(data)) => {
                        yield RealtimeEvent::Audio { data: data.to_vec() };
                    }
                    Ok(_) => {}
                    Err(e) => {
                        yield RealtimeEvent::Error { message: e.to_string() };
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }

    async fn send(&self, _event: RealtimeEvent) -> crate::core::Result<()> {
        // This is a simplified implementation. 
        // Real-time bidirectional communication usually requires keeping the connection open in a stateful object.
        // For the sake of this trait, we might need to rethink the design if 'send' is called independently.
        Err(anyhow!("Independent 'send' not supported yet. Use the stream for bidirectional flow.").into())
    }
}
