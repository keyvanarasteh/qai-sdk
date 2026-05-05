//! This example demonstrates the use of the Ollama-specific strongly-typed helper builders
//! for configuring server runners, generation options, and Modelfiles programmatically.

use qai_sdk::ollama::{LocalOllamaRunner, ModelfileBuilder, OllamaOptionsBuilder};

fn main() {
    println!("=== Ollama Builders Demonstration ===\n");

    // 1. ModelfileBuilder
    let modelfile = ModelfileBuilder::new()
        .from("llama3.2")
        .parameter("temperature", "0.8")
        .system("You are Mario from super mario bros.")
        .message("user", "Hello there!")
        .message("assistant", "It's a-me, Mario!")
        .build();

    println!("--- Generated Modelfile ---");
    println!("{}", modelfile);
    println!("---------------------------\n");

    // 2. OllamaOptionsBuilder
    let options = OllamaOptionsBuilder::new()
        .num_ctx(8192)
        .keep_alive("24h")
        .temperature(0.9)
        .seed(42)
        .build();

    println!("--- Generated API Options JSON ---");
    println!("{}", serde_json::to_string_pretty(&options).unwrap());
    println!("----------------------------------\n");

    // 3. LocalOllamaRunner
    // We won't actually `.spawn()` it to avoid interfering with any running servers,
    // but we can print the environment configuration.
    let runner = LocalOllamaRunner::new()
        .host("0.0.0.0:11434")
        .context_length(64000)
        .num_parallel(4)
        .max_loaded_models(3)
        .disable_cloud(true)
        .keep_alive("-1");

    println!("--- LocalOllamaRunner configured ---");
    println!("{:?}", runner);
    println!("------------------------------------\n");

    println!("To start the server, you would call: runner.spawn().unwrap();");
}
