pub mod mock_server;

use qai_core::types::Usage;
use reqwest::header::HeaderMap;
use std::sync::{Arc, Mutex};

pub struct MockClient {
    pub responses: Arc<Mutex<Vec<serde_json::Value>>>,
}

impl Default for MockClient {
    fn default() -> Self {
        Self::new()
    }
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
