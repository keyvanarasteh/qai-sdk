#[cfg(test)]
mod tests {
    use super::*;
    use qai_core::types::{Role, Content, Message};

    #[test]
    fn test_anthropic_request_mapping() {
        let model = AnthropicModel::new("test-key".to_string());
        let prompt = Prompt {
            messages: vec![
                Message {
                    role: Role::User,
                    content: vec![Content::Text { text: "Hello".to_string() }],
                },
            ],
        };
        let options = GenerateOptions {
            model_id: "claude-3-opus-20240229".to_string(),
            max_tokens: Some(100),
            temperature: Some(0.5),
            top_p: None,
            stop_sequences: None,
            tools: None,
        };

        let (request, _) = model.prepare_request(prompt, &options).unwrap();
        
        assert_eq!(request.model, "claude-3-opus-20240229");
        assert_eq!(request.max_tokens, 100);
        assert_eq!(request.temperature, Some(0.5));
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.messages[0].role, "user");
    }
}
