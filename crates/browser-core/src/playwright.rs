//! Playwright driver backend skeleton. Shells out to `npx playwright` in a
//! real impl; the skeleton returns the same canned data as `cdp.rs`.

use anyhow::Result;
use async_trait::async_trait;

use crate::{Action, ActionResult, Browser, DomNode, Screenshot};

pub struct PlaywrightBackend {
    pub headless: bool,
}

#[async_trait]
impl Browser for PlaywrightBackend {
    fn name(&self) -> &str { "playwright" }

    async fn execute(&self, action: &Action) -> Result<ActionResult> {
        Ok(match action {
            Action::Goto { url } => ActionResult::Text(format!("[playwright] goto {url}")),
            Action::Click { selector } => {
                ActionResult::Text(format!("[playwright] click {selector}"))
            }
            Action::Type { selector, text } => {
                ActionResult::Text(format!("[playwright] type {text:?} into {selector}"))
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
                ActionResult::Text(format!("[playwright] extract {selector}"))
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
