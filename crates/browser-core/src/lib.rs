//! browser-core: the `Browser` trait + 3 backend skeletons (CDP / Playwright
//! driver / static-replay). The trait is intentionally small so new backends
//! are easy to add.

mod cdp;
mod playwright;
mod replay;

pub use cdp::CdpBackend;
pub use playwright::PlaywrightBackend;
pub use replay::ReplayBackend;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// A DOM node extracted from a page. The skeleton keeps only the fields
/// needed for agent decisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomNode {
    pub tag: String,
    pub text: Option<String>,
    pub attrs: std::collections::BTreeMap<String, String>,
    pub children: Vec<DomNode>,
}

/// A screenshot returned by `Browser::screenshot`.
#[derive(Debug, Clone)]
pub struct Screenshot {
    pub width: u32,
    pub height: u32,
    pub png: Vec<u8>,
}

/// The action a controller can perform on a page.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Action {
    Goto { url: String },
    Click { selector: String },
    Type { selector: String, text: String },
    Screenshot,
    Wait { ms: u64 },
    Extract { selector: String },
    Close,
}

/// The result of executing an `Action`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ActionResult {
    Ok,
    Screenshot(Screenshot),
    Text(String),
    Dom(Vec<DomNode>),
    Err(String),
}

/// The Browser trait. Every backend implements this; the agent loop is
/// agnostic to which browser is wired up.
#[async_trait]
pub trait Browser: Send + Sync {
    /// Backend name, e.g. `"cdp"`, `"playwright"`, `"replay"`.
    fn name(&self) -> &str;
    /// Execute an action and return the result.
    async fn execute(&self, action: &Action) -> Result<ActionResult>;
    /// Return the current DOM (best-effort). Backends that can't do this
    /// return `Ok(vec![])`.
    async fn dom(&self) -> Result<Vec<DomNode>>;
    /// Whether the backend supports live navigation (replay backends don't).
    fn live(&self) -> bool { true }
}
