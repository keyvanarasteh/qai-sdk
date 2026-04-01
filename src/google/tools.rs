//! Google server-defined tools.
//!
//! These tools are Google-specific capabilities that can be used alongside
//! user-defined tools. Each function returns a `serde_json::Value`.

use serde_json::{json, Value};

/// Creates a Google Search grounding tool.
#[must_use]
pub fn google_search() -> Value {
    json!({ "google_search": {} })
}

/// Creates an Enterprise Web Search tool (Vertex AI only).
#[must_use]
pub fn enterprise_web_search() -> Value {
    json!({ "enterprise_web_search": {} })
}

/// Creates a Google Maps grounding tool.
#[must_use]
pub fn google_maps() -> Value {
    json!({ "google_maps": {} })
}

/// Creates a URL context tool for real-time web content.
#[must_use]
pub fn url_context() -> Value {
    json!({ "url_context": {} })
}

/// Creates a file search tool for Gemini RAG.
#[must_use]
pub fn file_search(
    store_names: Vec<String>,
    metadata_filter: Option<String>,
    top_k: Option<u32>,
) -> Value {
    let mut tool = json!({
        "file_search": {
            "store_names": store_names,
        }
    });
    if let Some(mf) = metadata_filter {
        tool["file_search"]["metadata_filter"] = json!(mf);
    }
    if let Some(k) = top_k {
        tool["file_search"]["top_k"] = json!(k);
    }
    tool
}

/// Creates a code execution tool for running Python.
#[must_use]
pub fn code_execution() -> Value {
    json!({ "code_execution": {} })
}

/// Creates a Vertex RAG Store tool.
#[must_use]
pub fn vertex_rag_store(rag_corpora: Vec<String>) -> Value {
    json!({
        "vertex_rag_store": {
            "rag_corpora": rag_corpora,
        }
    })
}
