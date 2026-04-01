//! # Error Handling Example
//!
//! Demonstrates typed error handling for each provider using the
//! provider-specific error types with `from_response()` parsers.

use qai_sdk::prelude::*;

#[tokio::main]
async fn main() -> qai_core::Result<()> {
    dotenvy::dotenv().ok();

    // ===================================================================
    // 1. OpenAI Error Handling
    // ===================================================================
    println!("=== OpenAI Error Handling ===");
    use qai_sdk::openai::error::OpenAIError;

    // Simulate parsing an API error response
    let error = OpenAIError::from_response(
        401,
        r#"{"error": {"message": "Incorrect API key provided", "type": "invalid_request_error", "code": "invalid_api_key"}}"#,
    );
    match &error {
        OpenAIError::AuthenticationError { message } => {
            println!("✅ Caught auth error: {}", message);
        }
        _ => println!("❌ Unexpected error type: {}", error),
    }

    // Rate limit error
    let error = OpenAIError::from_response(
        429,
        r#"{"error": {"message": "Rate limit exceeded", "type": "rate_limit_error"}}"#,
    );
    match &error {
        OpenAIError::RateLimitError { message } => {
            println!("✅ Caught rate limit error: {}", message);
        }
        _ => println!("❌ Unexpected error type: {}", error),
    }

    // Generic API error
    let error = OpenAIError::from_response(
        400,
        r#"{"error": {"message": "Invalid model", "type": "invalid_request_error", "code": "model_not_found"}}"#,
    );
    match &error {
        OpenAIError::ApiError {
            error_type,
            message,
            code,
            status_code,
        } => {
            println!(
                "✅ Caught API error: type={}, msg={}, code={:?}, status={}",
                error_type, message, code, status_code
            );
        }
        _ => println!("❌ Unexpected error type: {}", error),
    }

    // Non-JSON error body
    let error = OpenAIError::from_response(500, "Internal Server Error");
    match &error {
        OpenAIError::Other(msg) => {
            println!("✅ Caught other error: {}", msg);
        }
        _ => println!("❌ Unexpected error type: {}", error),
    }

    // ===================================================================
    // 2. Anthropic Error Handling
    // ===================================================================
    println!("\n=== Anthropic Error Handling ===");
    use qai_sdk::anthropic::error::AnthropicError;

    let error = AnthropicError::from_response(
        401,
        r#"{"type": "error", "error": {"type": "authentication_error", "message": "invalid x-api-key"}}"#,
    );
    match &error {
        AnthropicError::AuthenticationError { message } => {
            println!("✅ Caught auth error: {}", message);
        }
        _ => println!("❌ Unexpected error type: {}", error),
    }

    let error = AnthropicError::from_response(
        429,
        r#"{"type": "error", "error": {"type": "rate_limit_error", "message": "Too many requests"}}"#,
    );
    match &error {
        AnthropicError::RateLimitError { message } => {
            println!("✅ Caught rate limit: {}", message);
        }
        _ => println!("❌ Unexpected error type: {}", error),
    }

    // ===================================================================
    // 3. Google Error Handling
    // ===================================================================
    println!("\n=== Google Error Handling ===");
    use qai_sdk::google::error::GoogleError;

    let error = GoogleError::from_response(
        403,
        r#"{"error": {"code": 403, "message": "API key not valid", "status": "PERMISSION_DENIED"}}"#,
    );
    match &error {
        GoogleError::ApiError {
            status, message, ..
        } => {
            println!(
                "✅ Caught Google API error: status={}, msg={}",
                status, message
            );
        }
        _ => println!("❌ Unexpected error type: {}", error),
    }

    // ===================================================================
    // 4. DeepSeek Error Handling
    // ===================================================================
    println!("\n=== DeepSeek Error Handling ===");
    use qai_sdk::deepseek::error::DeepSeekError;

    let error = DeepSeekError::from_response(
        401,
        r#"{"error": {"message": "Invalid authentication", "type": "auth_error"}}"#,
    );
    match &error {
        DeepSeekError::AuthenticationError { message } => {
            println!("✅ Caught DeepSeek auth error: {}", message);
        }
        _ => println!("❌ Unexpected error type: {}", error),
    }

    // ===================================================================
    // 5. xAI Error Handling
    // ===================================================================
    println!("\n=== xAI Error Handling ===");
    use qai_sdk::xai::error::XaiError;

    let error = XaiError::from_response(
        429,
        r#"{"error": {"message": "Rate limit exceeded for Grok-3", "type": "rate_limit"}}"#,
    );
    match &error {
        XaiError::RateLimitError { message } => {
            println!("✅ Caught xAI rate limit: {}", message);
        }
        _ => println!("❌ Unexpected error type: {}", error),
    }

    // ===================================================================
    // 6. OpenAI-Compatible Error Handling (with provider name)
    // ===================================================================
    println!("\n=== OpenAI-Compatible Error Handling ===");
    use qai_sdk::openai_compatible::error::OpenAICompatibleError;

    let error = OpenAICompatibleError::from_response(
        "Together AI",
        401,
        r#"{"error": {"message": "Invalid API key", "type": "authentication_error"}}"#,
    );
    match &error {
        OpenAICompatibleError::AuthenticationError { provider, message } => {
            println!("✅ Caught {} auth error: {}", provider, message);
        }
        _ => println!("❌ Unexpected error type: {}", error),
    }

    let error = OpenAICompatibleError::from_response(
        "Groq",
        400,
        r#"{"error": {"message": "Model not found", "type": "invalid_request"}}"#,
    );
    match &error {
        OpenAICompatibleError::ApiError {
            provider,
            error_type,
            message,
            status_code,
            ..
        } => {
            println!(
                "✅ Caught {} API error: type={}, msg={}, status={}",
                provider, error_type, message, status_code
            );
        }
        _ => println!("❌ Unexpected error type: {}", error),
    }

    // ===================================================================
    // 7. Live error handling with actual API call
    // ===================================================================
    println!("\n=== Live Error Handling ===");
    let model = qai_sdk::openai::OpenAIModel::new("sk-invalid-key".to_string());

    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text {
                text: "Hello".to_string(),
            }],
        }],
    };

    let options = GenerateOptions {
        model_id: "gpt-4o-mini".to_string(),
        max_tokens: Some(10),
        temperature: None,
        top_p: None,
        stop_sequences: None,
        tools: None,
    };

    match model.generate(prompt, options).await {
        Ok(result) => println!("Response: {}", result.text),
        Err(e) => println!("✅ Caught live error: {}", e),
    }

    Ok(())
}
