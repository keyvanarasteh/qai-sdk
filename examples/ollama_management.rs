//! # Ollama Management API Example
//!
//! Demonstrates how to use the native Ollama management endpoints to fetch
//! server versions, list models, check running models, and show model details.
//!
//! Make sure your local Ollama server is running before executing this.

use qai_sdk::{
    core::types::ProviderSettings,
    ollama::{create_ollama, types::OllamaShowRequest},
    Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Automatically targets http://localhost:11434/v1 by default,
    // which the management helpers convert to http://localhost:11434/api
    let provider = create_ollama(ProviderSettings::default());

    println!("--- Ollama Management APIs ---\n");

    // 1. Get Ollama Version
    match provider.get_version().await {
        Ok(res) => println!("🟢 Ollama Version: {}", res.version),
        Err(e) => println!("🔴 Failed to get version: {:?}", e),
    }

    // 2. List Installed Models (/api/tags)
    let models = provider.list_models().await?;
    println!("\n📦 Installed Models ({} total):", models.models.len());
    for model in models.models.iter().take(5) {
        let size_gb = model.size.unwrap_or(0) as f64 / 1_000_000_000.0;
        println!("  - {} ({:.2} GB)", model.name, size_gb);
    }
    if models.models.len() > 5 {
        println!("  - ... and {} more", models.models.len() - 5);
    }

    // 3. List Running Models (/api/ps)
    let running = provider.list_running_models().await?;
    println!("\n🏃 Running Models ({} total):", running.models.len());
    for model in running.models {
        let vram_gb = model.size_vram.unwrap_or(0) as f64 / 1_000_000_000.0;
        println!("  - {} (Using {:.2} GB VRAM)", model.name, vram_gb);
    }

    // 4. Show specific model details (if we have any installed)
    if let Some(first_model) = models.models.first() {
        println!("\n🔍 Details for '{}':", first_model.name);

        let details = provider
            .show_model_info(OllamaShowRequest {
                model: first_model.name.clone(),
                verbose: Some(false),
            })
            .await?;

        if let Some(d) = details.details {
            println!("  Family: {:?}", d.family);
            println!("  Parameter Size: {:?}", d.parameter_size);
            println!("  Format: {:?}", d.format);
            println!("  Quantization: {:?}", d.quantization_level);
        }
    }

    Ok(())
}
