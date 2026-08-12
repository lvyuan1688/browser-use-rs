# browser-use-rs

> Give any LLM the ability to control a web browser — in Rust.
> Inspired by [Browser Use](https://github.com/browser-use/browser-use) (86k+ stars), rewritten from scratch in Rust with dual mode (DOM + Vision) and built-in trajectory replay.

## Why

Browser Use is the 86k-star Python browser-agent library, but:
- Python GIL + Playwright overhead limits throughput
- DOM-only mode — Canvas/Flash/WebGL scenes get stuck
- Trajectory replay needs external tooling

**browser-use-rs** ships a Rust implementation with:
- **Dual mode**: DOM extraction (fast, cheap) + Vision (Canvas/Flash scenes)
- **Built-in trajectory replay**: JSONL trajectory + TUI replay viewer
- **3 browser backends**: Playwright (default) / Cdp (direct, bypass Playwright) / Selenium (legacy compat)

## Architecture

```
browser-use-rs/
  crates/
    browser-core/        # Browser trait + 3 backends
      src/
        trait.rs         # pub trait Browser
        playwright.rs    # Playwright backend (via playwright-rust)
        cdp.rs           # Chrome DevTools Protocol direct
        selenium.rs      # Selenium WebDriver legacy
    agent-loop/          # Core agent loop
      src/
        loop.rs          # extract → decide → act → verify
        dom_extract.rs   # interactive element extraction + coord tagging
        vision.rs        # screenshot-based vision mode
        trajectory.rs    # JSONL trajectory persistence
    compress/            # DOM → LLM-readable element list
      src/
        filter.rs        # visibility + interactivity filter
        coord.rs         # bbox coordinate extraction
        index.rs         # element index assignment
  examples/
    basic_agent.rs
    vision_mode.rs
    trajectory_replay.rs
  replay-ui/             # TUI replay viewer (ratatui)
```

### Core trait

```rust
#[async_trait]
pub trait Browser: Send + Sync {
    async fn navigate(&self, url: &str) -> Result<()>;
    async fn screenshot(&self) -> Result<Vec<u8>>;          // PNG bytes
    async fn extract_dom(&self) -> Result<DomTree>;          // full DOM
    async fn click(&self, element_index: usize) -> Result<()>;
    async fn type_text(&self, element_index: usize, text: &str) -> Result<()>;
    async fn scroll(&self, dx: i32, dy: i32) -> Result<()>;
    async fn new_tab(&self, url: &str) -> Result<TabId>;
    async fn switch_tab(&self, id: TabId) -> Result<()>;
    async fn close_tab(&self, id: TabId) -> Result<()>;
    async fn get_cookies(&self) -> Result<Vec<Cookie>>;
    async fn wait_for_stable(&self, timeout_ms: u32) -> Result<()>;
}
```

### Agent loop (dual mode)

```
Task (natural language)
  ↓
Mode select:
  - DOM mode: extract_dom → compress → LLM sees element list
  - Vision mode: screenshot → LLM sees image (GPT-4V/Claude Vision)
  ↓
LLM decides action: "click element 5" / "type 'hello' into element 3"
  ↓
Execute (Browser trait)
  ↓
Wait for page stable → screenshot
  ↓
Trajectory append (screenshot + action + result)
  ↓
Task complete? → return extracted data / continue loop
```

### DOM compression (core differentiator)

Full DOM is too large for LLM context. `compress` crate extracts:

```rust
pub struct CompressedElement {
    pub index: usize,           // "element 5"
    pub tag: String,            // "button"
    pub role: Option<String>,   // ARIA role
    pub text: String,           // visible text (truncated to 200 chars)
    pub bbox: BoundingBox,      // (x, y, width, height)
    pub is_visible: bool,
    pub is_interactive: bool,   // click/input/scroll target
    pub attributes: HashMap<String, String>,  // id, href, placeholder...
}
```

Filter: `is_visible && is_interactive` → typically 20-100 elements per page (vs 2000+ full DOM).

### Trajectory persistence (JSONL)

Each step appended to `~/.browser-use-rs/trajectories/{task_id}.jsonl`:

```json
{"step":1,"screenshot":"base64...","action":{"type":"click","element":5},"result":"page navigated","timestamp":"2026-08-12T14:23:01Z"}
{"step":2,"screenshot":"base64...","action":{"type":"type","element":3,"text":"hello"},"result":"text entered","timestamp":"2026-08-12T14:23:03Z"}
```

Replay via TUI:

```bash
browser-use-rs replay ~/.browser-use-rs/trajectories/task_abc123.jsonl
```

### Cookie persistence (cross-run login state)

```rust
// Auto-saved to ~/.browser-use-rs/browser_profile/ on each run
// Restored on next run — skip re-login
pub struct BrowserProfile {
    pub cookies: Vec<Cookie>,
    pub local_storage: HashMap<String, String>,
    pub user_agent: String,
}
```

## Install

```bash
cargo install browser-use-rs
```

## Quick start

```bash
# Set LLM key
export OPENAI_API_KEY=sk-...

# DOM mode (default, fast)
browser-use-rs "go to hackernews and extract top 10 story titles"

# Vision mode (Canvas/Flash scenes, higher cost)
browser-use-rs --mode vision "fill the captcha on this form"
```

## Multi-tab management

```rust
let tab1 = browser.new_tab("https://example.com").await?;
let tab2 = browser.new_tab("https://google.com").await?;
browser.switch_tab(tab1).await?;
// Each tab has independent context
```

## Roadmap

- [x] Browser trait + 3 backends (Playwright/Cdp/Selenium)
- [x] DOM compression + element indexing
- [x] Dual mode (DOM + Vision)
- [x] Trajectory JSONL persistence + TUI replay
- [x] Cookie/session persistence
- [ ] Multi-tab parallel agents (currently sequential)
- [ ] Headless mode (no GUI, for CI/CD)

## License

MIT — see [LICENSE](LICENSE).

## Acknowledgments

- [Browser Use](https://github.com/browser-use/browser-use) — original 86k-star Python browser-agent library that inspired this Rust rewrite
- [Playwright](https://playwright.dev/) — Browser automation framework
- [ratatui](https://github.com/ratatui/ratatui) — TUI framework for replay viewer
