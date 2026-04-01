//! # Embedding Example
//!
//! Demonstrates text embedding generation using the `EmbeddingModel` trait
//! across OpenAI, Google, and OpenAI-Compatible providers.

use qai_sdk::types::{EmbeddingOptions, ProviderSettings};
use qai_sdk::EmbeddingModel;

use qai_sdk::LanguageModel;
#[tokio::main]
async fn main() -> qai_sdk::Result<()> {
    dotenvy::dotenv().ok();

    let texts = vec![
        "Rust is a systems programming language.".to_string(),
        "Python is great for data science.".to_string(),
        "TypeScript adds types to JavaScript.".to_string(),
    ];

    // ===================================================================
    // 1. OpenAI Embeddings
    // ===================================================================
    println!("=== OpenAI Embeddings ===");
    let provider = qai_sdk::openai::create_openai(ProviderSettings {
        api_key: Some(std::env::var("OPENAI_API_KEY").unwrap_or_default()),
        ..Default::default()
    });
    let model = provider.embedding("text-embedding-3-small");

    let options = EmbeddingOptions {
        model_id: "text-embedding-3-small".to_string(),
        dimensions: Some(256), // Reduce dimensions for cost savings
    };

    let result = model.embed(texts.clone(), options).await?;
    println!("Generated {} embeddings", result.embeddings.len());
    for (i, embedding) in result.embeddings.iter().enumerate() {
        println!(
            "  Text {}: {} dimensions, first 5 values: {:?}",
            i,
            embedding.len(),
            &embedding[..5.min(embedding.len())]
        );
    }
    if let Some(usage) = &result.usage {
        println!("  Usage: {} prompt tokens", usage.prompt_tokens);
    }

    // ===================================================================
    // 2. Google Embeddings
    // ===================================================================
    println!("\n=== Google Embeddings ===");
    let provider = qai_sdk::google::create_google(ProviderSettings {
        api_key: Some(std::env::var("GOOGLE_GENERATIVE_AI_API_KEY").unwrap_or_default()),
        ..Default::default()
    });
    let model = provider.embedding("text-embedding-004");

    let options = EmbeddingOptions {
        model_id: "text-embedding-004".to_string(),
        dimensions: None,
    };

    let result = model.embed(texts.clone(), options).await?;
    println!("Generated {} embeddings", result.embeddings.len());
    for (i, embedding) in result.embeddings.iter().enumerate() {
        println!("  Text {}: {} dimensions", i, embedding.len());
    }

    // ===================================================================
    // 3. OpenAI-Compatible Embeddings (e.g., Together AI)
    // ===================================================================
    println!("\n=== OpenAI-Compatible Embeddings ===");
    use qai_sdk::openai_compatible::OpenAICompatibleProviderSettings;

    let provider =
        qai_sdk::openai_compatible::create_openai_compatible(OpenAICompatibleProviderSettings {
            base_url: "https://api.together.xyz/v1".to_string(),
            name: "together".to_string(),
            api_key: Some(std::env::var("TOGETHER_API_KEY").unwrap_or_default()),
            headers: None,
        });
    let model = provider.embedding("togethercomputer/m2-bert-80M-8k-retrieval");

    let options = EmbeddingOptions {
        model_id: "togethercomputer/m2-bert-80M-8k-retrieval".to_string(),
        dimensions: None,
    };

    let result = model.embed(texts.clone(), options).await?;
    println!("Generated {} embeddings", result.embeddings.len());
    for (i, embedding) in result.embeddings.iter().enumerate() {
        println!("  Text {}: {} dimensions", i, embedding.len());
    }

    // ===================================================================
    // 4. Cosine Similarity between embeddings
    // ===================================================================
    println!("\n=== Cosine Similarity Demo ===");
    let provider = qai_sdk::openai::create_openai(ProviderSettings {
        api_key: Some(std::env::var("OPENAI_API_KEY").unwrap_or_default()),
        ..Default::default()
    });
    let model = provider.embedding("text-embedding-3-small");

    let similarity_texts = vec![
        "I love programming in Rust".to_string(),
        "Rust is my favorite programming language".to_string(),
        "The weather is beautiful today".to_string(),
    ];

    let result = model
        .embed(
            similarity_texts.clone(),
            EmbeddingOptions {
                model_id: "text-embedding-3-small".to_string(),
                dimensions: Some(256),
            },
        )
        .await?;

    println!("Similarity between related texts (should be high):");
    println!(
        "  '{}' vs '{}': {:.4}",
        similarity_texts[0],
        similarity_texts[1],
        cosine_similarity(&result.embeddings[0], &result.embeddings[1])
    );
    println!("Similarity between unrelated texts (should be low):");
    println!(
        "  '{}' vs '{}': {:.4}",
        similarity_texts[0],
        similarity_texts[2],
        cosine_similarity(&result.embeddings[0], &result.embeddings[2])
    );

    Ok(())
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}
