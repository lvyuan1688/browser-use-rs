//! Static-replay backend skeleton. Replays a recorded sequence of actions
//! and DOM snapshots — useful for offline agent-loop testing and demos.

use anyhow::Result;
use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{Action, ActionResult, Browser, DomNode};

pub struct ReplayBackend {
    pub script: Vec<(Action, ActionResult)>,
    cursor: AtomicUsize,
}

impl ReplayBackend {
    pub fn new(script: Vec<(Action, ActionResult)>) -> Self {
        Self { script, cursor: AtomicUsize::new(0) }
    }
}

#[async_trait]
impl Browser for ReplayBackend {
    fn name(&self) -> &str { "replay" }
    fn live(&self) -> bool { false }

    async fn execute(&self, action: &Action) -> Result<ActionResult> {
        let i = self.cursor.fetch_add(1, Ordering::SeqCst);
        if i >= self.script.len() {
            return Ok(ActionResult::Err("replay script exhausted".into()));
        }
        let (expected, result) = &self.script[i];
        // Best-effort match — return the canned result regardless.
        let _ = expected;
        Ok(result.clone())
    }

    async fn dom(&self) -> Result<Vec<DomNode>> {
        Ok(vec![])
    }
}
