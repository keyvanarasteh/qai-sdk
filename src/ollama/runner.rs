//! # Local Ollama Runner
//!
//! A programmatic wrapper to spawn a local Ollama server process with strongly-typed
//! configuration environment variables. This covers configurations from Linux, MacOS,
//! Windows, and Docker setups for GPU offloading, parallel requests, and context limits.

use std::collections::HashMap;
use std::process::{Child, Command};

/// A strongly-typed builder to configure and spawn a local Ollama server process.
#[derive(Debug, Clone, Default)]
pub struct LocalOllamaRunner {
    env_vars: HashMap<String, String>,
}

impl LocalOllamaRunner {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets `OLLAMA_HOST`. The host:port to bind to (e.g. `0.0.0.0:11434` or `127.0.0.1:11434`).
    pub fn host(mut self, val: &str) -> Self {
        self.env_vars
            .insert("OLLAMA_HOST".to_string(), val.to_string());
        self
    }

    /// Sets `OLLAMA_MODELS`. The path to the models directory.
    pub fn models_dir(mut self, val: &str) -> Self {
        self.env_vars
            .insert("OLLAMA_MODELS".to_string(), val.to_string());
        self
    }

    /// Sets `OLLAMA_KEEP_ALIVE`. The duration that models stay loaded in memory.
    /// E.g. "5m", "24h", or "-1" for infinite.
    pub fn keep_alive(mut self, val: &str) -> Self {
        self.env_vars
            .insert("OLLAMA_KEEP_ALIVE".to_string(), val.to_string());
        self
    }

    /// Sets `OLLAMA_MAX_QUEUE`. The maximum number of requests Ollama will queue when busy before rejecting.
    pub fn max_queue(mut self, val: u32) -> Self {
        self.env_vars
            .insert("OLLAMA_MAX_QUEUE".to_string(), val.to_string());
        self
    }

    /// Sets `OLLAMA_NUM_PARALLEL`. The maximum number of parallel requests each model will process at the same time.
    pub fn num_parallel(mut self, val: u32) -> Self {
        self.env_vars
            .insert("OLLAMA_NUM_PARALLEL".to_string(), val.to_string());
        self
    }

    /// Sets `OLLAMA_MAX_LOADED_MODELS`. The maximum number of models that can be loaded concurrently.
    pub fn max_loaded_models(mut self, val: u32) -> Self {
        self.env_vars
            .insert("OLLAMA_MAX_LOADED_MODELS".to_string(), val.to_string());
        self
    }

    /// Sets `OLLAMA_CONTEXT_LENGTH`. The default maximum context length across the server.
    pub fn context_length(mut self, val: u32) -> Self {
        self.env_vars
            .insert("OLLAMA_CONTEXT_LENGTH".to_string(), val.to_string());
        self
    }

    /// Sets `OLLAMA_NOHISTORY`. Disables history tracking for the interactive CLI if spawned.
    pub fn no_history(mut self, val: bool) -> Self {
        if val {
            self.env_vars
                .insert("OLLAMA_NOHISTORY".to_string(), "1".to_string());
        }
        self
    }

    /// Sets `OLLAMA_NO_CLOUD` (`disable_ollama_cloud` in JSON equivalent).
    /// Disables connecting to Ollama Cloud features natively.
    pub fn disable_cloud(mut self, val: bool) -> Self {
        if val {
            self.env_vars
                .insert("OLLAMA_NO_CLOUD".to_string(), "1".to_string());
        }
        self
    }

    /// Add a custom environment variable to the process.
    pub fn env(mut self, key: &str, val: &str) -> Self {
        self.env_vars.insert(key.to_string(), val.to_string());
        self
    }

    /// Spawns the `ollama serve` process in the background, returning the `Child` process handle.
    /// Ensure the `ollama` executable is installed and available in the system PATH.
    pub fn spawn(self) -> std::io::Result<Child> {
        let mut cmd = Command::new("ollama");
        cmd.arg("serve");

        for (k, v) in self.env_vars {
            cmd.env(k, v);
        }

        cmd.spawn()
    }
}
