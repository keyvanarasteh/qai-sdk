use crate::core::types::ToolDefinition;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::Stdio;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::error;
use eventsource_stream::Eventsource;
use futures_util::StreamExt;

#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("MCP IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("MCP Protocol Error: {0}")]
    Protocol(String),
    #[error("JSON Error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Server Error: {code} - {message}")]
    ServerError { code: i32, message: String },
    #[error("HTTP Error: {0}")]
    Http(#[from] reqwest::Error),
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: usize,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcNotification {
    jsonrpc: String,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    jsonrpc: String,
    #[serde(default)]
    id: Option<usize>,
    #[serde(default)]
    result: Option<Value>,
    #[serde(default)]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[allow(dead_code)]
    #[serde(default)]
    data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpPromptArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpPrompt {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Option<Vec<McpPromptArgument>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpPromptMessage {
    pub role: crate::core::types::Role,
    pub content: crate::core::types::Content,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub mime_type: Option<String>,
    pub size: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpResourceTemplate {
    pub uri_template: String,
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpResourceContent {
    pub uri: String,
    pub name: Option<String>,
    pub title: Option<String>,
    pub mime_type: Option<String>,
    pub text: Option<String>,
    pub blob: Option<String>,
}

/// A transport configuration for connecting to an MCP (Model Context Protocol) server.
#[derive(Debug, Clone)]
pub enum McpTransport {
    Stdio {
        command: String,
        args: Vec<String>,
        envs: std::collections::HashMap<String, String>,
    },
    Sse {
        url: String,
        headers: std::collections::HashMap<String, String>,
    },
}

impl McpTransport {
    pub fn stdio(command: impl Into<String>, args: Vec<impl Into<String>>) -> Self {
        Self::Stdio {
            command: command.into(),
            args: args.into_iter().map(|s| s.into()).collect(),
            envs: Default::default(),
        }
    }

    pub fn sse(url: impl Into<String>) -> Self {
        Self::Sse {
            url: url.into(),
            headers: Default::default(),
        }
    }
}

/// A lightweight client for connecting to an MCP (Model Context Protocol) server.
pub struct McpClient {
    next_id: Arc<AtomicUsize>,
    tx_req: mpsc::Sender<(JsonRpcRequest, oneshot::Sender<Result<Value, McpError>>)>,
    tx_notif: mpsc::Sender<JsonRpcNotification>,
    _child: Option<Child>,
}

impl McpClient {
    /// Connects to an MCP server using the specified transport and performs the initialization handshake.
    pub async fn connect(transport: McpTransport) -> Result<Self, McpError> {
        let client = match transport {
            McpTransport::Stdio { command, args, envs } => {
                let mut cmd = Command::new(&command);
                cmd.args(&args)
                    .envs(&envs)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::inherit()); // allow stderr to pass through for server logs

                let mut child = cmd.spawn()?;
                let stdin = child.stdin.take().expect("Failed to open stdin");
                let stdout = child.stdout.take().expect("Failed to open stdout");

                let (tx_req, rx_req) = mpsc::channel(32);
                let (tx_notif, rx_notif) = mpsc::channel(32);
                let next_id = Arc::new(AtomicUsize::new(1));

                Self::spawn_stdio_io_loops(stdin, stdout, rx_req, rx_notif);

                Self {
                    next_id,
                    tx_req,
                    tx_notif,
                    _child: Some(child),
                }
            }
            McpTransport::Sse { url, headers } => {
                let http_client = reqwest::Client::new();
                let mut req = http_client.get(&url).header("Accept", "text/event-stream");
                for (k, v) in headers {
                    req = req.header(&k, &v);
                }
                
                let resp = req.send().await?;
                let mut stream = resp.bytes_stream().eventsource();
                
                // 1. Wait for "endpoint" event
                let endpoint = loop {
                    match stream.next().await {
                        Some(Ok(event)) => {
                            if event.event == "endpoint" {
                                break event.data;
                            }
                        }
                        _ => return Err(McpError::Protocol("Failed to receive endpoint event from SSE server".to_string())),
                    }
                };

                let post_url = if endpoint.starts_with("http") {
                    endpoint
                } else if endpoint.starts_with('/') {
                    let parsed = reqwest::Url::parse(&url).unwrap();
                    format!("{}://{}{}", parsed.scheme(), parsed.host_str().unwrap(), endpoint)
                } else {
                    format!("{}/{}", url.trim_end_matches('/'), endpoint)
                };

                let (tx_req, rx_req) = mpsc::channel(32);
                let (tx_notif, rx_notif) = mpsc::channel(32);
                let next_id = Arc::new(AtomicUsize::new(1));

                Self::spawn_sse_io_loops(stream, rx_req, rx_notif, http_client, post_url);

                Self {
                    next_id,
                    tx_req,
                    tx_notif,
                    _child: None,
                }
            }
        };

        client.initialize().await?;
        Ok(client)
    }

    fn spawn_stdio_io_loops(
        mut stdin: ChildStdin,
        stdout: ChildStdout,
        mut rx_req: mpsc::Receiver<(JsonRpcRequest, oneshot::Sender<Result<Value, McpError>>)>,
        mut rx_notif: mpsc::Receiver<JsonRpcNotification>,
    ) {
        let pending_requests: Arc<Mutex<std::collections::HashMap<usize, oneshot::Sender<Result<Value, McpError>>>>> =
            Arc::new(Mutex::new(std::collections::HashMap::new()));
        let pending_clone = pending_requests.clone();

        // Write Loop
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    res = rx_req.recv() => {
                        match res {
                            Some((req, reply_tx)) => {
                                pending_requests.lock().await.insert(req.id, reply_tx);
                                let mut msg = serde_json::to_string(&req).unwrap();
                                msg.push('\n');
                                if let Err(e) = stdin.write_all(msg.as_bytes()).await {
                                    tracing::error!("Failed to write request: {}", e);
                                    break;
                                }
                            }
                            None => break,
                        }
                    }
                    res = rx_notif.recv() => {
                        match res {
                            Some(notif) => {
                                let mut msg = serde_json::to_string(&notif).unwrap();
                                msg.push('\n');
                                if let Err(e) = stdin.write_all(msg.as_bytes()).await {
                                    tracing::error!("Failed to write notification: {}", e);
                                    break;
                                }
                            }
                            None => break,
                        }
                    }
                }
            }
        });

        // Read Loop
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if line.trim().is_empty() {
                    continue;
                }
                match serde_json::from_str::<JsonRpcResponse>(&line) {
                    Ok(resp) => {
                        if let Some(id) = resp.id {
                            let mut pending = pending_clone.lock().await;
                            if let Some(reply_tx) = pending.remove(&id) {
                                if let Some(err) = resp.error {
                                    let _ = reply_tx.send(Err(McpError::ServerError {
                                        code: err.code,
                                        message: err.message,
                                    }));
                                } else if let Some(res) = resp.result {
                                    let _ = reply_tx.send(Ok(res));
                                } else {
                                    let _ = reply_tx.send(Err(McpError::Protocol("Empty response result".to_string())));
                                }
                            }
                        }
                        // Ignore notifications from server in this simple MWE
                    }
                    Err(e) => {
                        error!("Failed to parse JSON-RPC response from server: {} (line: {})", e, line);
                    }
                }
            }
        });
    }

    fn spawn_sse_io_loops<S>(
        mut stream: S,
        mut rx_req: mpsc::Receiver<(JsonRpcRequest, oneshot::Sender<Result<Value, McpError>>)>,
        mut rx_notif: mpsc::Receiver<JsonRpcNotification>,
        client: reqwest::Client,
        post_url: String,
    ) where
        S: futures_util::Stream<Item = Result<eventsource_stream::Event, eventsource_stream::EventStreamError<reqwest::Error>>> + Unpin + Send + 'static,
    {
        let pending_requests: Arc<Mutex<std::collections::HashMap<usize, oneshot::Sender<Result<Value, McpError>>>>> =
            Arc::new(Mutex::new(std::collections::HashMap::new()));
        let pending_clone = pending_requests.clone();

        // Write Loop (HTTP POST)
        let client_clone = client.clone();
        let post_url_clone = post_url.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    res = rx_req.recv() => {
                        match res {
                            Some((req, reply_tx)) => {
                                pending_requests.lock().await.insert(req.id, reply_tx);
                                let req_json = serde_json::to_string(&req).unwrap();
                                if let Err(e) = client_clone.post(&post_url_clone).header("Content-Type", "application/json").body(req_json).send().await {
                                    tracing::error!("Failed to post request: {}", e);
                                    break;
                                }
                            }
                            None => break,
                        }
                    }
                    res = rx_notif.recv() => {
                        match res {
                            Some(notif) => {
                                let notif_json = serde_json::to_string(&notif).unwrap();
                                if let Err(e) = client_clone.post(&post_url_clone).header("Content-Type", "application/json").body(notif_json).send().await {
                                    tracing::error!("Failed to post notification: {}", e);
                                    break;
                                }
                            }
                            None => break,
                        }
                    }
                }
            }
        });

        // Read Loop (SSE Stream)
        tokio::spawn(async move {
            while let Some(Ok(event)) = stream.next().await {
                if event.event == "message" {
                    if let Ok(resp) = serde_json::from_str::<JsonRpcResponse>(&event.data) {
                        if let Some(id) = resp.id {
                            let mut pending = pending_clone.lock().await;
                            if let Some(reply_tx) = pending.remove(&id) {
                                if let Some(err) = resp.error {
                                    let _ = reply_tx.send(Err(McpError::ServerError {
                                        code: err.code,
                                        message: err.message,
                                    }));
                                } else if let Some(res) = resp.result {
                                    let _ = reply_tx.send(Ok(res));
                                } else {
                                    let _ = reply_tx.send(Err(McpError::Protocol("Empty response result".to_string())));
                                }
                            }
                        }
                    } else {
                        tracing::error!("Failed to parse SSE JSON-RPC response from server: {}", event.data);
                    }
                }
            }
        });
    }

    async fn send_request(&self, method: &str, params: Option<Value>) -> Result<Value, McpError> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        };
        let (tx, rx): (oneshot::Sender<Result<Value, McpError>>, oneshot::Receiver<Result<Value, McpError>>) = oneshot::channel();
        self.tx_req
            .send((req, tx))
            .await
            .map_err(|_| McpError::Protocol("IO loops terminated".to_string()))?;

        rx.await
            .map_err(|_| McpError::Protocol("Request dropped".to_string()))?
    }

    async fn send_notification(&self, method: &str, params: Option<Value>) -> Result<(), McpError> {
        let notif = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
        };
        self.tx_notif
            .send(notif)
            .await
            .map_err(|_| McpError::Protocol("IO loops terminated".to_string()))?;
        Ok(())
    }

    async fn initialize(&self) -> Result<(), McpError> {
        let params = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "clientInfo": {
                "name": "qai-sdk",
                "version": "0.1.0"
            },
            "capabilities": {}
        });

        // 1. Send Initialize Request
        let _init_res = self.send_request("initialize", Some(params)).await?;
        
        // 2. Send Initialized Notification
        self.send_notification("notifications/initialized", None).await?;
        Ok(())
    }

    /// Fetches the list of tools available on the MCP server and converts them to QAI-SDK `ToolDefinition`s.
    /// Returns a tuple of `(tools, next_cursor)`.
    pub async fn get_tools(&self, cursor: Option<String>) -> Result<(Vec<ToolDefinition>, Option<String>), McpError> {
        let mut params = serde_json::Map::new();
        if let Some(c) = cursor {
            params.insert("cursor".to_string(), Value::String(c));
        }
        let params_val = if params.is_empty() { None } else { Some(Value::Object(params)) };

        let res = self.send_request("tools/list", params_val).await?;
        
        let mut sdk_tools = Vec::new();
        if let Some(tools) = res.get("tools").and_then(|t| t.as_array()) {
            for t in tools {
                let name = t.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                let description = t.get("description").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                let parameters = t.get("inputSchema").cloned().unwrap_or(serde_json::json!({
                    "type": "object",
                    "properties": {}
                }));

                sdk_tools.push(ToolDefinition {
                    name,
                    description,
                    parameters,
                });
            }
        }
        
        let next_cursor = res.get("nextCursor").and_then(|v| v.as_str()).map(|s| s.to_string());
        
        Ok((sdk_tools, next_cursor))
    }

    /// Fetches the list of resources available on the MCP server.
    /// Returns a tuple of `(resources, next_cursor)`.
    pub async fn list_resources(&self, cursor: Option<String>) -> Result<(Vec<McpResource>, Option<String>), McpError> {
        let mut params = serde_json::Map::new();
        if let Some(c) = cursor {
            params.insert("cursor".to_string(), Value::String(c));
        }
        let params_val = if params.is_empty() { None } else { Some(Value::Object(params)) };
        
        let res = self.send_request("resources/list", params_val).await?;
        
        let resources_val = res.get("resources").unwrap_or(&Value::Null);
        let resources: Vec<McpResource> = serde_json::from_value(resources_val.clone())?;
        
        let next_cursor = res.get("nextCursor").and_then(|v| v.as_str()).map(|s| s.to_string());
        
        Ok((resources, next_cursor))
    }

    /// Fetches the list of resource templates available on the MCP server.
    /// Returns a tuple of `(templates, next_cursor)`.
    pub async fn list_resource_templates(&self, cursor: Option<String>) -> Result<(Vec<McpResourceTemplate>, Option<String>), McpError> {
        let mut params = serde_json::Map::new();
        if let Some(c) = cursor {
            params.insert("cursor".to_string(), Value::String(c));
        }
        let params_val = if params.is_empty() { None } else { Some(Value::Object(params)) };
        
        let res = self.send_request("resources/templates/list", params_val).await?;
        
        let templates_val = res.get("resourceTemplates").unwrap_or(&Value::Null);
        let templates: Vec<McpResourceTemplate> = serde_json::from_value(templates_val.clone())?;
        
        let next_cursor = res.get("nextCursor").and_then(|v| v.as_str()).map(|s| s.to_string());
        
        Ok((templates, next_cursor))
    }

    /// Reads a specific resource by URI across the MCP server.
    pub async fn read_resource(&self, uri: &str) -> Result<Vec<McpResourceContent>, McpError> {
        let mut params = serde_json::Map::new();
        params.insert("uri".to_string(), Value::String(uri.to_string()));
        
        let res = self.send_request("resources/read", Some(Value::Object(params))).await?;
        
        let contents_val = res.get("contents").unwrap_or(&Value::Null);
        let contents: Vec<McpResourceContent> = serde_json::from_value(contents_val.clone())?;
        
        Ok(contents)
    }

    /// Calls a specific tool on the MCP server.
    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<Value, McpError> {
        let params = serde_json::json!({
            "name": name,
            "arguments": arguments,
        });

        self.send_request("tools/call", Some(params)).await
    }

    /// Fetches the list of prompts available on the MCP server.
    /// Returns a tuple of `(prompts, next_cursor)`.
    pub async fn list_prompts(&self, cursor: Option<String>) -> Result<(Vec<McpPrompt>, Option<String>), McpError> {
        let mut params = serde_json::Map::new();
        if let Some(c) = cursor {
            params.insert("cursor".to_string(), serde_json::Value::String(c));
        }
        let params_val = if params.is_empty() { None } else { Some(serde_json::Value::Object(params)) };
        
        let res = self.send_request("prompts/list", params_val).await?;
        
        let prompts_val = res.get("prompts").unwrap_or(&serde_json::Value::Null);
        let prompts: Vec<McpPrompt> = serde_json::from_value(prompts_val.clone())?;
        
        let next_cursor = res.get("nextCursor").and_then(|v| v.as_str()).map(|s| s.to_string());
        
        Ok((prompts, next_cursor))
    }

    /// Retrieves a specific prompt by name, optionally with arguments.
    /// Returns the prompt description and the generated list of messages.
    pub async fn get_prompt(
        &self,
        name: &str,
        arguments: Option<std::collections::HashMap<String, String>>,
    ) -> Result<(String, Vec<crate::core::types::Message>), McpError> {
        let mut params = serde_json::Map::new();
        params.insert("name".to_string(), serde_json::Value::String(name.to_string()));
        if let Some(args) = arguments {
            let mut args_map = serde_json::Map::new();
            for (k, v) in args {
                args_map.insert(k, serde_json::Value::String(v));
            }
            params.insert("arguments".to_string(), serde_json::Value::Object(args_map));
        }

        let res = self.send_request("prompts/get", Some(serde_json::Value::Object(params))).await?;
        
        let description = res.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
        
        let messages_val = res.get("messages").unwrap_or(&serde_json::Value::Null);
        let mcp_messages: Vec<McpPromptMessage> = serde_json::from_value(messages_val.clone())?;
        
        let mut final_messages = Vec::new();
        for msg in mcp_messages {
            final_messages.push(crate::core::types::Message {
                role: msg.role,
                content: vec![msg.content],
            });
        }
        
        Ok((description, final_messages))
    }
}
