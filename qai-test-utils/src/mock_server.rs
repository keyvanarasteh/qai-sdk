use serde_json::Value;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

pub struct TestServer {
    pub server: MockServer,
}

impl TestServer {
    pub async fn new() -> Self {
        let server = MockServer::start().await;
        Self { server }
    }

    pub fn url(&self) -> String {
        self.server.uri()
    }

    /// Mount a mock endpoint that expects a POST at `path` and returns `response_body`
    pub async fn mock_chat_completion(&self, endpoint_path: &str, response_body: Value) {
        Mock::given(method("POST"))
            .and(path(endpoint_path))
            .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
            .mount(&self.server)
            .await;
    }
}
