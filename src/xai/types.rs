//! xAI-specific types and model IDs.

use serde::{Deserialize, Serialize};

// --- Chat Model IDs ---

/// Known xAI chat model IDs.
pub const GROK_3: &str = "grok-3";
pub const GROK_3_MINI: &str = "grok-3-mini";
pub const GROK_2: &str = "grok-2";

// --- Image Model IDs ---

/// Known xAI image model IDs.
pub const GROK_IMAGINE_IMAGE: &str = "grok-imagine-image";
pub const GROK_IMAGINE_IMAGE_PRO: &str = "grok-imagine-image-pro";

/// xAI chat model configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XaiChatSettings {
    /// Model ID to use.
    pub model_id: String,
}

/// xAI image model configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XaiImageSettings {
    /// Model ID to use.
    pub model_id: String,
}
