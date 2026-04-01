#[cfg(test)]
use crate::core::types::*;
#[cfg(test)]
use reqwest::header::{HeaderMap, HeaderValue};

#[test]
fn test_usage_from_headers_openai() {
    let mut headers = HeaderMap::new();
    headers.insert(
        "x-openai-usage-prompt-tokens",
        HeaderValue::from_static("10"),
    );
    headers.insert(
        "x-openai-usage-completion-tokens",
        HeaderValue::from_static("20"),
    );

    let usage = Usage::from_headers(&headers).unwrap();
    assert_eq!(usage.prompt_tokens, 10);
    assert_eq!(usage.completion_tokens, 20);
}

#[test]
fn test_usage_from_headers_anthropic() {
    let mut headers = HeaderMap::new();
    // Test both formats
    headers.insert(
        "x-anthropic-usage-input-tokens",
        HeaderValue::from_static("15"),
    );
    headers.insert(
        "x-anthropic-usage-output-tokens",
        HeaderValue::from_static("25"),
    );

    let usage = Usage::from_headers(&headers).unwrap();
    assert_eq!(usage.prompt_tokens, 15);
    assert_eq!(usage.completion_tokens, 25);
}
