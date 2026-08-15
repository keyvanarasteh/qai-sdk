use crate::core::types::RealtimeEvent;
use crate::core::Result;
use async_trait::async_trait;
use futures::stream::BoxStream;
use futures_util::StreamExt;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};

/// Google Gemini Realtime model (Multimodal Live API).
pub struct GoogleRealtimeModel {
    pub api_key: String,
    pub model_id: String,
    pub base_url: String, // e.g. "generativelanguage.googleapis.com"
}

impl GoogleRealtimeModel {
    pub fn new(api_key: String, model_id: String, base_url: String) -> Self {
        Self {
            api_key,
            model_id,
            base_url,
        }
    }
}

#[async_trait]
impl crate::core::RealtimeModel for GoogleRealtimeModel {
    async fn connect(&self) -> Result<BoxStream<'static, RealtimeEvent>> {
        let url = format!(
            "wss://{}/ws/google.ai.generativelanguage.v1beta.GenerativeService.BidiGenerateContent?key={}",
            self.base_url, self.api_key
        );

        let (ws_stream, _) = connect_async(url).await.map_err(|e| {
            crate::core::error::ProviderError::Network(format!("WebSocket connection failed: {e}"))
        })?;

        let (mut _write, read) = ws_stream.split();

        // Gemini Realtime protocol setup (sending setup message)
        // For now, we assume the user might want to send it manually or we send a basic one.
        // let setup = json!({
        //     "setup": {
        //         "model": format!("models/{}", self.model_id)
        //     }
        // });
        // write.send(WsMessage::Text(setup.to_string().into())).await.ok();

        let stream = read.filter_map(|msg| async move {
            match msg {
                Ok(WsMessage::Text(text)) => {
                    // Map Gemini JSON to RealtimeEvent
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                        if let Some(text_val) = val.pointer("/serverContent/modelTurn/parts/0/text")
                        {
                            return Some(RealtimeEvent::Text {
                                text: text_val.as_str()?.to_string(),
                            });
                        }
                        if let Some(audio_val) =
                            val.pointer("/serverContent/modelTurn/parts/0/inlineData/data")
                        {
                            use base64::Engine as _;
                            let data = base64::engine::general_purpose::STANDARD
                                .decode(audio_val.as_str()?)
                                .ok()?;
                            return Some(RealtimeEvent::Audio { data });
                        }
                    }
                    None
                }
                _ => None,
            }
        });

        Ok(Box::pin(stream))
    }

    async fn send(&self, _event: RealtimeEvent) -> Result<()> {
        // In a real implementation, we'd need to maintain the write half of the WebSocket.
        // For the trait to be useful, it probably needs a handle to the connection.
        // This is a simplified version.
        Err(crate::core::error::ProviderError::NotSupported(
            "Send not implemented in this simplified trait".to_string(),
        ))
    }
}
