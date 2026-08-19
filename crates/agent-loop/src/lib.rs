//! agent-loop: Think → Act → Observe state machine wired on top of
//! `browser-core::Browser`.

use anyhow::Result;
use browser_core::{Action, ActionResult, Browser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Think,
    Act,
    Observe,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub phase: Phase,
    pub action: Option<Action>,
    pub result: Option<ActionResult>,
}

/// Run the loop: produce one Action per iteration, execute it on the
/// browser, observe the result, and stop when `decide` returns Phase::Done.
pub async fn run_loop<B, F, Fut>(browser: &B, mut decide: F) -> Result<Vec<Step>>
where
    B: Browser,
    F: FnMut(&[Step]) -> Fut,
    Fut: std::future::Future<Output = Result<Phase>>,
{
    let mut history = Vec::new();
    for _ in 0..100 {
        let phase = decide(&history).await?;
        if phase == Phase::Done {
            history.push(Step { phase, action: None, result: None });
            break;
        }
        let action = Action::Screenshot;
        let result = browser.execute(&action).await?;
        history.push(Step { phase, action: Some(action), result: Some(result) });
    }
    Ok(history)
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct Stub;
    #[async_trait]
    impl Browser for Stub {
        fn name(&self) -> &str { "stub" }
        async fn execute(&self, _: &Action) -> Result<ActionResult> {
            Ok(ActionResult::Ok)
        }
        async fn dom(&self) -> Result<Vec<browser_core::DomNode>> { Ok(vec![]) }
    }

    #[tokio::test]
    async fn loop_terminates() {
        let b = Stub;
        let r = run_loop(&b, |_: &[Step]| async move { Ok(Phase::Done) }).await.unwrap();
        assert_eq!(r.len(), 1);
    }
}
