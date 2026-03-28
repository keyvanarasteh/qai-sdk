use std::sync::{Arc, Mutex};
use reqwest::header::HeaderMap;
use qai_core::types::Usage;

pub struct MockClient {
    pub responses: Arc<Mutex<Vec<serde_json::Value>>>,
}

impl MockClient {
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn push_response(&self, response: serde_json::Value) {
        self.responses.lock().unwrap().push(response);
    }
}

pub fn extract_usage(headers: &HeaderMap) -> Option<Usage> {
    Usage::from_headers(headers)
}
