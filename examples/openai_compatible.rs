//! # OpenAI-Compatible Provider Example
//!
//! Demonstrates using the OpenAI-Compatible provider to connect to any
//! third-party API endpoint that follows the OpenAI API format.
//! Examples include: Together AI, Groq, Fireworks, Perplexity, Mistral,
//! Anyscale, OpenRouter, local Ollama, and vLLM.

use futures::StreamExt;
use qai_sdk::openai_compatible::OpenAICompatibleProviderSettings;
use qai_sdk::types::EmbeddingOptions;
use qai_sdk::EmbeddingModel;
use qai_sdk::*;

use qai_sdk::LanguageModel;
#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    // ===================================================================
    // 1. Together AI
    // ===================================================================
    println!("=== Together AI ===");
    let together = create_openai_compatible(OpenAICompatibleProviderSettings {
        base_url: "https://api.together.xyz/v1".to_string(),
        name: "together".to_string(),
        api_key: Some(std::env::var("TOGETHER_API_KEY").unwrap_or_default()),
        headers: None,
    });

    // Chat
    let model = together.chat("meta-llama/Llama-3-70b-chat-hf");
    let prompt = Prompt {
        messages: vec![Message {
            role: Role::User,
            content: vec![Content::Text {
                text: "What is Rust? One sentence.".to_string(),
            }],
        }],
    };
    let options = GenerateOptions {
        model_id: "meta-llama/Llama-3-70b-chat-hf".to_string(),
        max_tokens: Some(100),
        temperature: Some(0.7),
        top_p: None,
        stop_sequences: None,
        tools: None,
    };
    let result = model.generate(prompt.clone(), options.clone()).await?;
    println!("Together Chat: {}", result.text);

    // Embedding
    let model = together.embedding("togethercomputer/m2-bert-80M-8k-retrieval");
    let result = model
        .embed(
            vec!["Hello world".to_string()],
            EmbeddingOptions {
                model_id: "togethercomputer/m2-bert-80M-8k-retrieval".to_string(),
                dimensions: None,
            },
        )
        .await?;
    println!(
        "Together Embedding: {} dimensions\n",
        result.embeddings[0].len()
    );

    // ===================================================================
    // 2. Groq (Ultra-fast inference)
    // ===================================================================
    println!("=== Groq ===");
    let groq = create_openai_compatible(OpenAICompatibleProviderSettings {
        base_url: "https://api.groq.com/openai/v1".to_string(),
        name: "groq".to_string(),
        api_key: Some(std::env::var("GROQ_API_KEY").unwrap_or_default()),
        headers: None,
    });

    let model = groq.chat("llama-3.1-70b-versatile");
    let options = GenerateOptions {
        model_id: "llama-3.1-70b-versatile".to_string(),
        max_tokens: Some(100),
        temperature: Some(0.5),
        top_p: None,
        stop_sequences: None,
        tools: None,
    };
    let result = model.generate(prompt.clone(), options.clone()).await?;
    println!("Groq response: {}\n", result.text);

    // ===================================================================
    // 3. Fireworks AI
    // ===================================================================
    println!("=== Fireworks AI ===");
    let fireworks = create_openai_compatible(OpenAICompatibleProviderSettings {
        base_url: "https://api.fireworks.ai/inference/v1".to_string(),
        name: "fireworks".to_string(),
        api_key: Some(std::env::var("FIREWORKS_API_KEY").unwrap_or_default()),
        headers: None,
    });

    let model = fireworks.chat("accounts/fireworks/models/llama-v3p1-70b-instruct");
    let options = GenerateOptions {
        model_id: "accounts/fireworks/models/llama-v3p1-70b-instruct".to_string(),
        max_tokens: Some(100),
        temperature: None,
        top_p: None,
        stop_sequences: None,
        tools: None,
    };
    let result = model.generate(prompt.clone(), options.clone()).await?;
    println!("Fireworks response: {}\n", result.text);

    // ===================================================================
    // 4. OpenRouter (Multi-provider gateway)
    // ===================================================================
    println!("=== OpenRouter ===");
    let openrouter = create_openai_compatible(OpenAICompatibleProviderSettings {
        base_url: "https://openrouter.ai/api/v1".to_string(),
        name: "openrouter".to_string(),
        api_key: Some(std::env::var("OPENROUTER_API_KEY").unwrap_or_default()),
        headers: None,
    });

    let model = openrouter.chat("meta-llama/llama-3.1-8b-instruct:free");
    let options = GenerateOptions {
        model_id: "meta-llama/llama-3.1-8b-instruct:free".to_string(),
        max_tokens: Some(100),
        temperature: None,
        top_p: None,
        stop_sequences: None,
        tools: None,
    };
    let result = model.generate(prompt.clone(), options.clone()).await?;
    println!("OpenRouter response: {}\n", result.text);

    // ===================================================================
    // 5. Mistral AI
    // ===================================================================
    println!("=== Mistral AI ===");
    let mistral = create_openai_compatible(OpenAICompatibleProviderSettings {
        base_url: "https://api.mistral.ai/v1".to_string(),
        name: "mistral".to_string(),
        api_key: Some(std::env::var("MISTRAL_API_KEY").unwrap_or_default()),
        headers: None,
    });

    let model = mistral.chat("mistral-small-latest");
    let options = GenerateOptions {
        model_id: "mistral-small-latest".to_string(),
        max_tokens: Some(100),
        temperature: None,
        top_p: None,
        stop_sequences: None,
        tools: None,
    };
    let result = model.generate(prompt.clone(), options.clone()).await?;
    println!("Mistral response: {}\n", result.text);

    // ===================================================================
    // 6. Local Ollama
    // ===================================================================
    println!("=== Local Ollama ===");
    let ollama = create_openai_compatible(OpenAICompatibleProviderSettings {
        base_url: "http://localhost:11434/v1".to_string(),
        name: "ollama".to_string(),
        api_key: Some("ollama".to_string()), // Ollama doesn't require a key
        headers: None,
    });

    let model = ollama.chat("llama3.2");
    let options = GenerateOptions {
        model_id: "llama3.2".to_string(),
        max_tokens: Some(100),
        temperature: None,
        top_p: None,
        stop_sequences: None,
        tools: None,
    };

    // Ollama may not be running, so handle errors gracefully
    match model.generate(prompt.clone(), options.clone()).await {
        Ok(result) => println!("Ollama response: {}", result.text),
        Err(e) => println!("Ollama not available: {} (start with `ollama serve`)", e),
    }

    // ===================================================================
    // 7. Streaming with Compatible Provider
    // ===================================================================
    println!("\n=== Streaming with Compatible Provider ===");
    let model = groq.chat("llama-3.1-8b-instant");
    let options = GenerateOptions {
        model_id: "llama-3.1-8b-instant".to_string(),
        max_tokens: Some(100),
        temperature: None,
        top_p: None,
        stop_sequences: None,
        tools: None,
    };

    let mut stream = model.generate_stream(prompt.clone(), options).await?;
    print!("Streaming: ");
    while let Some(part) = stream.next().await {
        if let StreamPart::TextDelta { delta } = part {
            print!("{}", delta);
        }
    }
    println!("\n");

    // ===================================================================
    // Summary of supported endpoints
    // ===================================================================
    println!("--- Supported OpenAI-Compatible Endpoints ---");
    println!("Together AI:  api.together.xyz/v1");
    println!("Groq:         api.groq.com/openai/v1");
    println!("Fireworks:    api.fireworks.ai/inference/v1");
    println!("OpenRouter:   openrouter.ai/api/v1");
    println!("Mistral:      api.mistral.ai/v1");
    println!("Perplexity:   api.perplexity.ai");
    println!("Ollama:       localhost:11434/v1");
    println!("vLLM:         localhost:8000/v1");
    println!("LM Studio:    localhost:1234/v1");

    Ok(())
}
