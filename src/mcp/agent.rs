use crate::core::types::{Content, GenerateOptions, Message, Prompt, Role};
use crate::core::LanguageModel;
use crate::mcp::client::McpClient;
use crate::mcp::McpError;

/// A monolithic LLM Agent execution loop embedded with automatic MCP Tool tracking.
///
/// This loop accepts an instantiated `LanguageModel` and an active `McpClient`.
/// It discovers tools exposed by the MCP server, provides them to the LLM, and 
/// autonomously routes native `ToolCall` events from the LLM back to the MCP server.
/// If an error occurs, it is returned; otherwise the final LLM string response is yielded.
pub async fn run_mcp_agent<M: LanguageModel>(
    model: &M,
    mcp_client: &McpClient,
    mut messages: Vec<Message>,
    mut options: GenerateOptions,
    max_turns: usize,
) -> Result<String, McpError> {
    
    // 1. Auto-discover MCP tools
    let mut all_tools = Vec::new();
    let mut current_cursor = None;
    loop {
        let (tools, next_cursor) = mcp_client.get_tools(current_cursor.clone()).await?;
        all_tools.extend(tools);
        if next_cursor.is_none() {
            break;
        }
        current_cursor = next_cursor;
    }

    // Merge MCP tools into GenerateOptions
    if let Some(mut existing) = options.tools.take() {
        existing.extend(all_tools);
        options.tools = Some(existing);
    } else {
        options.tools = Some(all_tools);
    }

    // 2. Execution Loop
    for _turn in 0..max_turns {
        let result = model
            .generate(
                Prompt {
                    messages: messages.clone(),
                },
                options.clone(),
            )
            .await
            .map_err(|e| McpError::Protocol(format!("LLM Provider Error: {:?}", e)))?;

        // If no tool calls were requested, simply return the final text
        if result.tool_calls.is_empty() {
            return Ok(result.text);
        }

        // Prepare the LLM Assistant's response appending all the tools it attempted to call
        let mut assistant_content = vec![];
        if !result.text.is_empty() {
            assistant_content.push(Content::Text {
                text: result.text.clone(),
            });
        }

        let mut call_idx = 1;
        for tc in &result.tool_calls {
            assistant_content.push(Content::ToolCall {
                id: format!("call_{}", call_idx),
                name: tc.name.clone(),
                arguments: tc.arguments.clone(),
            });
            call_idx += 1;
        }

        messages.push(Message {
            role: Role::Assistant,
            content: assistant_content,
        });

        // 3. Resolve Tool Calls sequentially via MCP JSON-RPC
        let mut tool_results = vec![];
        let mut resp_call_idx = 1;
        for tc in result.tool_calls {
            let res = mcp_client.call_tool(&tc.name, tc.arguments).await?;

            tool_results.push(Content::ToolResult {
                id: format!("call_{}", resp_call_idx),
                result: res,
            });
            resp_call_idx += 1;
        }

        // Append the execution results to the LLM Context
        messages.push(Message {
            role: Role::Tool,
            content: tool_results,
        });
    }

    Err(McpError::Protocol(format!(
        "Agent Loop aborted: Exceeded max_turns ({})",
        max_turns
    )))
}
