//! browser-use-rs CLI entry point.
//! Skeleton: build a backend, run an empty action loop, print a stub result.

use anyhow::Result;
use browser_core::{Action, Browser, CdpBackend};
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "browser-use-rs", version, about = "Give any LLM the ability to control a browser")]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Navigate to a URL and screenshot.
    Go { url: String },
    /// Print the resolved backend.
    Info,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let backend = CdpBackend { endpoint: "http://localhost:9222".into() };
    match cli.cmd.unwrap_or(Command::Info) {
        Command::Go { url } => {
            let _ = backend.execute(&Action::Goto { url }).await?;
            let _ = backend.execute(&Action::Screenshot).await?;
            println!("done");
        }
        Command::Info => {
            println!("backend = {} ({})", backend.name(), backend.live());
        }
    }
    Ok(())
}
