use qai_core::types::{Content, GenerateOptions, Message, Prompt, Role};
use qai_core::LanguageModel;
use qai_openai::OpenAIModel;
use qai_test_utils::mock_server::TestServer;
use serde_json::json;

#[tokio::test]
async fn test_openai_mocked_chat_completion() {
    let mock_server = TestServer::new().await;

    // Define the expected mock response body mapping perfectly to OpenAI spec
    let json_response = json!({
        "id": "chatcmpl-123",
        "object": "chat.completion",
        "created": 1677652288,
        "model": "gpt-4o",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Hello there! This is a mock server response."
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 9,
            "completion_tokens": 12,
            "total_tokens": 21
        }
    });

    // Mount endpoint to wiremock
    mock_server.mock_chat_completion("/chat/completions", json_response).await;

    // Setup the openai model pointing to local wiremock instead of openai.com
    let model = OpenAIModel {
        api_key: "mock-key".to_string(),
        base_url: mock_server.url(),
        client: reqwest::Client::new(),
    };

    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text {
                text: "Hello".to_string(),
            }],
        }],
    };

    let options = GenerateOptions {
        model_id: "gpt-4o".to_string(),
        max_tokens: None,
        temperature: None,
        top_p: None,
        stop_sequences: None,
        tools: None,
    };

    let result = model.generate(prompt, options).await.expect("Failed to call mocked completion");

    assert_eq!(result.text, "Hello there! This is a mock server response.");
    assert_eq!(result.usage.prompt_tokens, 9);
    assert_eq!(result.usage.completion_tokens, 12);
}
