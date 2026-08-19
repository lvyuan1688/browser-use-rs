//! CDP (Chrome DevTools Protocol) backend skeleton.
//! In a real impl this would speak WebSocket to a Chromium instance. The
//! skeleton returns canned data so the agent loop can be exercised offline.

use anyhow::Result;
use async_trait::async_trait;

use crate::{Action, ActionResult, Browser, DomNode, Screenshot};

pub struct CdpBackend {
    pub endpoint: String,
}

#[async_trait]
impl Browser for CdpBackend {
    fn name(&self) -> &str { "cdp" }

    async fn execute(&self, action: &Action) -> Result<ActionResult> {
        Ok(match action {
            Action::Goto { url } => ActionResult::Text(format!("[cdp] navigated to {url}")),
            Action::Click { selector } => ActionResult::Text(format!("[cdp] clicked {selector}")),
            Action::Type { selector, text } => {
                ActionResult::Text(format!("[cdp] typed {text:?} into {selector}"))
            }
            Action::Screenshot => ActionResult::Screenshot(Screenshot {
                width: 1280,
                height: 720,
                png: Vec::new(),
            }),
            Action::Wait { ms } => {
                tokio::time::sleep(std::time::Duration::from_millis(*ms)).await;
                ActionResult::Ok
            }
            Action::Extract { selector } => {
                ActionResult::Text(format!("[cdp] extracted {selector}"))
            }
            Action::Close => ActionResult::Ok,
        })
    }

    async fn dom(&self) -> Result<Vec<DomNode>> {
        Ok(vec![DomNode {
            tag: "html".into(),
            text: None,
            attrs: Default::default(),
            children: vec![],
        }])
    }
}
