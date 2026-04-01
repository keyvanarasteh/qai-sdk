#[cfg(test)]
use crate::types::OpenAIMessage;
#[cfg(test)]
use crate::OpenAIModel;
#[cfg(test)]
use qai_core::types::*;

#[test]
fn test_openai_request_mapping() {
    let model = OpenAIModel::new("test-key".to_string());
    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text {
                text: "Hello".to_string(),
            }],
        }],
    };
    let options = GenerateOptions {
        model_id: "gpt-4-turbo".to_string(),
        max_tokens: Some(100),
        temperature: Some(0.7),
        top_p: None,
        stop_sequences: None,
        tools: None,
    };

    let request = model.prepare_request(prompt, options).unwrap();

    assert_eq!(request.model, "gpt-4-turbo");
    assert_eq!(request.max_tokens, Some(100));
    assert_eq!(request.temperature, Some(0.7));
    assert_eq!(request.messages.len(), 1);

    match &request.messages[0] {
        OpenAIMessage::User { .. } => {}
        _ => panic!("Expected User message"),
    }
}
