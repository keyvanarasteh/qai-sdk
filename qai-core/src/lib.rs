pub mod types;

use async_trait::async_trait;
use crate::types::{GenerateOptions, GenerateResult, Prompt};
use anyhow::Result;

#[async_trait]
pub trait LanguageModel: Send + Sync {
    async fn generate(&self, prompt: Prompt, options: GenerateOptions) -> Result<GenerateResult>;
}

pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    fn language_model(&self, model_id: &str) -> Box<dyn LanguageModel>;
}
