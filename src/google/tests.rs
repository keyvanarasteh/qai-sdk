#[cfg(test)]
use crate::core::types::*;
#[cfg(test)]
use crate::google::GoogleModel;

#[test]
fn test_google_request_mapping() {
    let model = GoogleModel::new("test-key".to_string());
    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text {
                text: "Hello".to_string(),
            }],
        }],
    };
    let options = GenerateOptions {
        model_id: "gemini-1.5-pro".to_string(),
        max_tokens: Some(100),
        temperature: Some(0.9),
        top_p: None,
        stop_sequences: None,
        tools: None,
        response_format: None,
    };

    let request = model.prepare_request(prompt, &options).unwrap();

    assert_eq!(request.contents.len(), 1);
    assert_eq!(request.contents[0].role, "user");
    assert_eq!(
        request
            .generation_config
            .as_ref()
            .unwrap()
            .max_output_tokens,
        Some(100)
    );
    assert_eq!(
        request.generation_config.as_ref().unwrap().temperature,
        Some(0.9)
    );
}
