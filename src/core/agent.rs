//! # Universal Agent
//!
//! A generalized tool-calling loop that works with any `LanguageModel` and
//! any set of tool definitions. The agent iteratively calls the model,
//! detects tool calls, executes them via a user-provided handler, feeds
//! results back, and repeats until no more tool calls or `max_steps` is reached.
//!
//! This is a universal version of the MCP-specific `run_mcp_agent`.
//!
//! # Example
//! ```rust,ignore
//! use qai_sdk::core::agent::*;
//!
//! let agent = Agent::builder()
//!     .model(my_model)
//!     .tools(vec![weather_tool, search_tool])
//!     .tool_handler(|name, args| async move {
//!         match name.as_str() {
//!             "get_weather" => Ok(json!({"temp": "22°C"})),
//!             _ => Err(anyhow::anyhow!("Unknown tool")),
//!         }
//!     })
//!     .max_steps(10)
//!     .system("You are a helpful assistant.")
//!     .build();
//!
//! let result = agent.run("What's the weather in Paris?").await?;
//! println!("{}", result.text);
//! ```

use crate::core::types::{
    Content, GenerateOptions, GenerateResult, Message, Prompt, Role, ToolDefinition,
};
use crate::core::{LanguageModel, Result};
use crate::core::error::ProviderError;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// A snapshot of one agent step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStep {
    /// The step index (0-based).
    pub step: usize,
    /// The model's text response for this step.
    pub text: String,
    /// Tool calls the model made in this step.
    pub tool_calls: Vec<AgentToolCall>,
}

/// A tool call within a step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentToolCall {
    /// Tool name.
    pub name: String,
    /// Tool arguments (JSON).
    pub arguments: serde_json::Value,
    /// The result returned by the tool handler.
    pub result: Option<serde_json::Value>,
}

/// The final result of an agent run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    /// The final text response.
    pub text: String,
    /// All steps taken.
    pub steps: Vec<AgentStep>,
    /// Total steps executed.
    pub total_steps: usize,
    /// The finish reason of the last step.
    pub finish_reason: String,
}

/// Type alias for the tool handler closure.
pub type ToolHandlerFn = Arc<
    dyn Fn(
            String,
            serde_json::Value,
        ) -> Pin<Box<dyn Future<Output = anyhow::Result<serde_json::Value>> + Send>>
        + Send
        + Sync,
>;

/// A universal agent that runs an iterative tool-calling loop.
pub struct Agent {
    model: Box<dyn LanguageModel>,
    tools: Vec<ToolDefinition>,
    tool_handler: ToolHandlerFn,
    max_steps: usize,
    system: Option<String>,
    model_id: String,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
}

/// Builder for creating an `Agent`.
pub struct AgentBuilder {
    model: Option<Box<dyn LanguageModel>>,
    tools: Vec<ToolDefinition>,
    tool_handler: Option<ToolHandlerFn>,
    max_steps: usize,
    system: Option<String>,
    model_id: String,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
}

impl AgentBuilder {
    /// Set the language model.
    #[must_use]
    pub fn model(mut self, model: Box<dyn LanguageModel>) -> Self {
        self.model = Some(model);
        self
    }

    /// Set the available tools.
    #[must_use]
    pub fn tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.tools = tools;
        self
    }

    /// Set the tool execution handler.
    ///
    /// The handler receives `(tool_name, arguments)` and must return the tool result as JSON.
    #[must_use]
    pub fn tool_handler<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(String, serde_json::Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<serde_json::Value>> + Send + 'static,
    {
        self.tool_handler = Some(Arc::new(move |name, args| {
            Box::pin(handler(name, args))
        }));
        self
    }

    /// Set the maximum number of tool-call loop iterations.
    #[must_use]
    pub fn max_steps(mut self, max_steps: usize) -> Self {
        self.max_steps = max_steps;
        self
    }

    /// Set a system prompt.
    #[must_use]
    pub fn system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }

    /// Set the model ID string.
    #[must_use]
    pub fn model_id(mut self, model_id: impl Into<String>) -> Self {
        self.model_id = model_id.into();
        self
    }

    /// Set the temperature.
    #[must_use]
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set the max tokens.
    #[must_use]
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Build the agent.
    pub fn build(self) -> std::result::Result<Agent, String> {
        Ok(Agent {
            model: self.model.ok_or("model is required")?,
            tools: self.tools,
            tool_handler: self.tool_handler.ok_or("tool_handler is required")?,
            max_steps: self.max_steps,
            system: self.system,
            model_id: self.model_id,
            temperature: self.temperature,
            max_tokens: self.max_tokens,
        })
    }
}

impl Agent {
    /// Create a new agent builder.
    #[must_use]
    pub fn builder() -> AgentBuilder {
        AgentBuilder {
            model: None,
            tools: Vec::new(),
            tool_handler: None,
            max_steps: 10,
            system: None,
            model_id: String::new(),
            temperature: None,
            max_tokens: None,
        }
    }

    /// Run the agent to completion, returning the full result with all steps.
    pub async fn run(&self, prompt_text: &str) -> Result<AgentResult> {
        let mut messages = Vec::new();

        // System prompt
        if let Some(ref sys) = self.system {
            messages.push(Message {
                role: Role::System,
                content: vec![Content::Text {
                    text: sys.clone(),
                }],
            });
        }

        // User message
        messages.push(Message {
            role: Role::User,
            content: vec![Content::Text {
                text: prompt_text.to_string(),
            }],
        });

        let mut steps = Vec::new();
        let mut last_result: Option<GenerateResult> = None;

        for step_idx in 0..self.max_steps {
            let prompt = Prompt {
                messages: messages.clone(),
            };

            let options = GenerateOptions {
                model_id: self.model_id.clone(),
                max_tokens: self.max_tokens,
                temperature: self.temperature,
                top_p: None,
                stop_sequences: None,
                tools: if self.tools.is_empty() {
                    None
                } else {
                    Some(self.tools.clone())
                },
            };

            let result = self.model.generate(prompt, options).await?;

            // Record step
            let mut step = AgentStep {
                step: step_idx,
                text: result.text.clone(),
                tool_calls: Vec::new(),
            };

            if result.tool_calls.is_empty() {
                // No tool calls — we're done
                steps.push(step);
                last_result = Some(result);
                break;
            }

            // Add assistant message with tool calls
            let mut assistant_content = Vec::new();
            if !result.text.is_empty() {
                assistant_content.push(Content::Text {
                    text: result.text.clone(),
                });
            }
            for tc in &result.tool_calls {
                assistant_content.push(Content::ToolCall {
                    id: tc.name.clone(),
                    name: tc.name.clone(),
                    arguments: tc.arguments.clone(),
                });
            }
            messages.push(Message {
                role: Role::Assistant,
                content: assistant_content,
            });

            // Execute tool calls
            for tc in &result.tool_calls {
                let tool_result = (self.tool_handler)(
                    tc.name.clone(),
                    tc.arguments.clone(),
                )
                .await;

                let result_value = match tool_result {
                    Ok(v) => v,
                    Err(e) => serde_json::json!({ "error": e.to_string() }),
                };

                step.tool_calls.push(AgentToolCall {
                    name: tc.name.clone(),
                    arguments: tc.arguments.clone(),
                    result: Some(result_value.clone()),
                });

                // Add tool result to conversation
                messages.push(Message {
                    role: Role::Tool,
                    content: vec![Content::ToolResult {
                        id: tc.name.clone(),
                        result: result_value,
                    }],
                });
            }

            steps.push(step);
            last_result = Some(result);
        }

        let final_result = last_result.ok_or_else(|| {
            ProviderError::InvalidResponse("Agent produced no results".to_string())
        })?;

        let total_steps = steps.len();
        Ok(AgentResult {
            text: final_result.text,
            steps,
            total_steps,
            finish_reason: final_result.finish_reason,
        })
    }
}
