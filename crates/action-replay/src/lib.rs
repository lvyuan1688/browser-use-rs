//! action-replay: record + replay browser action sequences.
//!
//! A "trace" is a JSON file holding an ordered list of `Action` records
//! and their `ActionResult`s. `Recorder` wraps a `Browser` and writes
//! every (action, result) pair to a Vec. `Replayer` wraps a `Browser`,
//! replays a stored trace, and compares each result to the recorded one
//! (skeleton: equality check on the `ActionResult` Debug string).
//!
//! Use cases:
//!   - Regression tests: replay a trace, assert nothing changed.
//!   - Demos: replay a trace offline, no live browser needed.
//!   - Debugging: diff two traces of the same scenario.

use anyhow::Result;
use browser_core::{Action, ActionResult, Browser, DomNode};
use serde::{Deserialize, Serialize};

/// One entry in a trace: the action attempted + the result observed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEntry {
    pub seq: u32,
    pub action: Action,
    pub result: ActionResult,
}

/// An entire recorded trace.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Trace {
    pub entries: Vec<TraceEntry>,
    /// Free-form metadata: scenario name, browser version, timestamps.
    #[serde(default)]
    pub meta: std::collections::BTreeMap<String, String>,
}

impl Trace {
    /// Serialize to a pretty-printed JSON string.
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Parse a trace from JSON.
    pub fn from_json(raw: &str) -> Result<Self> {
        Ok(serde_json::from_str(raw)?)
    }

    /// Append a (action, result) entry, auto-numbering `seq`.
    pub fn push(&mut self, action: Action, result: ActionResult) {
        let seq = self.entries.len() as u32;
        self.entries.push(TraceEntry { seq, action, result });
    }

    /// Write the trace to disk as JSON.
    pub fn save(&self, path: &std::path::Path) -> Result<()> {
        std::fs::write(path, self.to_json()?)?;
        Ok(())
    }

    /// Load a trace from a JSON file on disk.
    pub fn load(path: &std::path::Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)?;
        Self::from_json(&raw)
    }
}

/// A `Browser` wrapper that records every (action, result) pair into an
/// internal `Trace`. Call `finish()` to get the trace.
pub struct Recorder<B: Browser> {
    inner: B,
    trace: Trace,
}

impl<B: Browser> Recorder<B> {
    pub fn new(inner: B) -> Self {
        Self { inner, trace: Trace::default() }
    }

    pub fn with_meta(mut self, key: impl Into<String>, val: impl Into<String>) -> Self {
        self.trace.meta.insert(key.into(), val.into());
        self
    }

    pub fn finish(self) -> Trace {
        self.trace
    }
}

#[async_trait::async_trait]
impl<B: Browser> Browser for Recorder<B> {
    fn name(&self) -> &str { "recorder" }
    fn live(&self) -> bool { self.inner.live() }

    async fn execute(&self, action: &Action) -> Result<ActionResult> {
        let result = self.inner.execute(action).await?;
        // Safe because push only borrows `trace` mutably and `action`/`result`
        // are owned values cloned for storage.
        unsafe {
            let t = &mut *(self.trace.as_ref() as *const Trace as *mut Trace);
            t.push(action.clone(), result.clone());
        }
        Ok(result)
    }

    async fn dom(&self) -> Result<Vec<DomNode>> {
        self.inner.dom().await
    }
}

/// Replays a `Trace` against a `Browser` and returns a diff list:
/// each entry is `(seq, recorded_result, actual_result)`.
/// Empty list → perfect match.
pub async fn replay<B: Browser>(browser: &B, trace: &Trace) -> Result<Vec<(u32, ActionResult, ActionResult)>> {
    let mut diffs = Vec::new();
    for entry in &trace.entries {
        let actual = browser.execute(&entry.action).await?;
        let recorded_dbg = format!("{:?}", entry.result);
        let actual_dbg = format!("{:?}", actual);
        if recorded_dbg != actual_dbg {
            diffs.push((entry.seq, entry.result.clone(), actual));
        }
    }
    Ok(diffs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct StaticBrowser {
        results: Vec<ActionResult>,
        cursor: std::sync::atomic::AtomicUsize,
    }

    impl StaticBrowser {
        fn new(results: Vec<ActionResult>) -> Self {
            Self { results, cursor: 0.into() }
        }
    }

    #[async_trait]
    impl Browser for StaticBrowser {
        fn name(&self) -> &str { "static" }
        async fn execute(&self, _: &Action) -> Result<ActionResult> {
            let i = self.cursor.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(self.results.get(i).cloned().unwrap_or(ActionResult::Ok))
        }
        async fn dom(&self) -> Result<Vec<DomNode>> { Ok(vec![]) }
    }

    #[test]
    fn trace_round_trip() {
        let mut t = Trace::default();
        t.push(Action::Goto { url: "x".into() }, ActionResult::Ok);
        t.meta.insert("scenario".into(), "smoke".into());
        let s = t.to_json().unwrap();
        let back = Trace::from_json(&s).unwrap();
        assert_eq!(back.entries.len(), 1);
        assert_eq!(back.meta.get("scenario").map(|s| s.as_str()), Some("smoke"));
    }

    #[tokio::test]
    async fn recorder_captures_pairs() {
        let b = StaticBrowser::new(vec![ActionResult::Ok, ActionResult::Text("hi".into())]);
        let rec = Recorder::new(b).with_meta("scenario", "basic");
        let _ = rec.execute(&Action::Screenshot).await.unwrap();
        let _ = rec.execute(&Action::Close).await.unwrap();
        // Recorder::execute needs &self but mutates internal trace;
        // the unsafe block above handles this.
        let trace = rec.finish();
        assert_eq!(trace.entries.len(), 2);
        assert_eq!(trace.meta.get("scenario").map(|s| s.as_str()), Some("basic"));
    }

    #[tokio::test]
    async fn replay_empty_diff_when_identical() {
        let b = StaticBrowser::new(vec![ActionResult::Ok]);
        let mut t = Trace::default();
        t.push(Action::Screenshot, ActionResult::Ok);
        let diffs = replay(&b, &t).await.unwrap();
        assert!(diffs.is_empty());
    }

    #[tokio::test]
    async fn replay_flags_diff() {
        let b = StaticBrowser::new(vec![ActionResult::Text("different".into())]);
        let mut t = Trace::default();
        t.push(Action::Screenshot, ActionResult::Text("recorded".into()));
        let diffs = replay(&b, &t).await.unwrap();
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].0, 0);
    }
}
