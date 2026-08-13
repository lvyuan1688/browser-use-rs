//! browser-use-rs - give any LLM the ability to control a web browser
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "browser-use-rs", version, about = "LLM browser control, in Rust")]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Launch a browser session and connect an LLM agent
    Session { url: String },
    /// Replay a recorded trajectory
    Replay { path: String },
    /// List installed browser drivers
    Drivers,
}

fn main() {
    match Cli::parse().cmd.unwrap_or(Cmd::Drivers) {
        Cmd::Session { url } => println!("[session] navigate to {url} (stub)"),
        Cmd::Replay { path } => println!("[replay] {path} (stub)"),
        Cmd::Drivers => println!("[drivers] chromedriver/geckodriver (stub)"),
    }
}
