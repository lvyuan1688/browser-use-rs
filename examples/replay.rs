//! Run a recorded agent session end-to-end.
//! `cargo run --example replay -- path/to/session.json`

use anyhow::Result;
use browser_use_rs::agent_loop::{run_loop, Phase, Step};
use browser_core::{Action, ActionResult, ReplayBackend};

#[tokio::main]
async fn main() -> Result<()> {
    let script = vec![
        (Action::Goto { url: "https://example.com".into() }, ActionResult::Ok),
        (Action::Screenshot, ActionResult::Ok),
        (Action::Close, ActionResult::Ok),
    ];
    let backend = ReplayBackend::new(script);
    let history: Vec<Step> = run_loop(&backend, |_: &[Step]| async move { Ok(Phase::Done) }).await?;
    println!("replay finished, {} steps", history.len());
    Ok(())
}
