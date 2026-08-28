//! Killer demo: browser-use-rs navigates to a URL, snapshots the DOM,
//! and runs a 4-step action trajectory — all offline with a stub CDP backend.
//!
//! Run:  cargo run --example demo
//!
//! What you'll see: a stub CDP backend "navigates" to example.com,
//! executes Click/Type/Screenshot actions, and reports per-step timing.

use anyhow::Result;
use browser_core::{Action, Browser, CdpBackend, DomNode};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🌐  browser-use-rs  —  demo run");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 1. Build a stub CDP backend (no real browser needed).
    let browser = CdpBackend { endpoint: "ws://localhost:9222".into() };
    println!("\n①  Backend: {}  endpoint: {}", browser.name(), browser.endpoint);

    // 2. Navigate to example.com.
    println!("\n②  Navigate to https://example.com ...");
    let t0 = Instant::now();
    let r = browser.execute(&Action::Goto { url: "https://example.com".into() }).await?;
    println!("   ✅  {:?}  in {:?}", r, t0.elapsed());

    // 3. Snapshot the DOM.
    println!("\n③  Snapshot DOM tree ...");
    let t1 = Instant::now();
    let dom: Vec<DomNode> = browser.dom().await?;
    println!("   ✅  {} top-level node(s) captured in {:?}", dom.len(), t1.elapsed());
    for (i, node) in dom.iter().enumerate().take(3) {
        println!("      {}. <{}>  text={:?}",
            i + 1,
            node.tag,
            node.text.as_deref().unwrap_or("").chars().take(40).collect::<String>(),
        );
    }

    // 4. Run a 4-step action trajectory.
    let trajectory = vec![
        Action::Click { selector: "nav a:first-child".into() },
        Action::Type  { selector: "input#search".into(), text: "rust browser agent".into() },
        Action::Screenshot,
        Action::Wait  { ms: 500 },
    ];
    println!("\n④  Execute {}-step action trajectory ...", trajectory.len());
    let t2 = Instant::now();
    for (i, action) in trajectory.iter().enumerate() {
        let step_t = Instant::now();
        let r = browser.execute(action).await?;
        let label = match action {
            Action::Click { .. }    => "Click",
            Action::Type { .. }     => "Type",
            Action::Screenshot       => "Screenshot",
            Action::Wait { .. }     => "Wait",
            Action::Goto { .. }     => "Goto",
            Action::Extract { .. }  => "Extract",
            Action::Close            => "Close",
        };
        println!("   step {}  {:<10}  {:?}  in {:?}", i + 1, label, r, step_t.elapsed());
    }
    println!("   ✅  trajectory completed in {:?}", t2.elapsed());

    // 5. Summary.
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊  Summary");
    println!("   Backend           : {} (stub)", browser.name());
    println!("   DOM nodes         : {}", dom.len());
    println!("   Actions executed  : {}", trajectory.len());
    println!("   Total demo time   : {:?}", t0.elapsed());
    println!();
    println!("⭐  Star browser-use-rs for more:");
    println!("     https://github.com/lvyuan1688/browser-use-rs");
    Ok(())
}
