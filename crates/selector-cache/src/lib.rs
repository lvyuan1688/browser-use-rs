//! selector-cache: cache CSS selector → element resolution.
//!
//! DOM queries are the slowest part of browser automation. When the
//! same `document.querySelector(sel)` is issued repeatedly (e.g. an
//! agent polling a "Next" button), `SelectorCache` avoids the round
//! trip after the first hit.
//!
//! The cache is invalidated by:
//!   - explicit `invalidate(sel)` call
//!   - `invalidate_all()` after a navigation event
//!   - TTL (configurable, default 30s) — DOM mutates under us
//!
//! Lookups return a `SelectorHit` which carries the resolved node +
//! the age of the cache entry.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use browser_core::{Browser, DomNode, Selector};

/// One cached selector resolution.
#[derive(Debug, Clone)]
pub struct SelectorHit {
    pub selector: String,
    pub node: DomNode,
    /// When this entry was last validated.
    pub cached_at: Instant,
    pub hits: u32,
}

/// A TTL-based selector cache.
pub struct SelectorCache {
    inner: Mutex<HashMap<String, SelectorHit>>,
    ttl: Duration,
}

impl SelectorCache {
    pub fn new() -> Self {
        Self::with_ttl(Duration::from_secs(30))
    }

    pub fn with_ttl(ttl: Duration) -> Self {
        Self { inner: Mutex::new(HashMap::new()), ttl }
    }

    /// Try the cache first. If miss or stale, query the browser and
    /// store the result. Returns `None` if the selector matches nothing.
    pub async fn get<B: Browser>(
        &self,
        browser: &B,
        sel: &str,
    ) -> Option<SelectorHit> {
        if let Some(hit) = self.check(sel) {
            return Some(hit);
        }
        let node = browser.query(Selector::css(sel)).await.ok()?;
        let hit = SelectorHit {
            selector: sel.to_string(),
            node,
            cached_at: Instant::now(),
            hits: 1,
        };
        self.inner.lock().unwrap().insert(sel.to_string(), hit.clone());
        Some(hit)
    }

    /// Return a fresh, non-stale entry if present.
    fn check(&self, sel: &str) -> Option<SelectorHit> {
        let mut map = self.inner.lock().unwrap();
        let hit = map.get_mut(sel)?;
        if hit.cached_at.elapsed() > self.ttl {
            map.remove(sel);
            return None;
        }
        hit.hits += 1;
        Some(hit.clone())
    }

    pub fn invalidate(&self, sel: &str) {
        self.inner.lock().unwrap().remove(sel);
    }

    pub fn invalidate_all(&self) {
        self.inner.lock().unwrap().clear();
    }

    pub fn len(&self) -> usize {
        self.inner.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for SelectorCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct CountingBrowser {
        queries: AtomicUsize,
    }

    #[async_trait]
    impl Browser for CountingBrowser {
        async fn query(&self, _sel: browser_core::Selector) -> anyhow::Result<DomNode> {
            self.queries.fetch_add(1, Ordering::SeqCst);
            Ok(DomNode::element("div", "cached"))
        }
        async fn execute(&self, _a: &browser_core::Action) -> anyhow::Result<browser_core::ActionResult> {
            Ok(browser_core::ActionResult::Ok)
        }
    }

    #[tokio::test]
    async fn cache_hits_avoid_browser_query() {
        let b = CountingBrowser { queries: 0.into() };
        let cache = SelectorCache::with_ttl(Duration::from_secs(60));
        let _ = cache.get(&b, ".btn").await.unwrap();
        let _ = cache.get(&b, ".btn").await.unwrap();
        assert_eq!(b.queries.load(Ordering::SeqCst), 1);
        assert_eq!(cache.get(&b, ".btn").await.unwrap().hits, 3);
    }

    #[tokio::test]
    async fn stale_entry_refetched() {
        let b = CountingBrowser { queries: 0.into() };
        let cache = SelectorCache::with_ttl(Duration::from_millis(1));
        let _ = cache.get(&b, ".btn").await;
        tokio::time::sleep(Duration::from_millis(5)).await;
        let _ = cache.get(&b, ".btn").await;
        assert_eq!(b.queries.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn missing_selector_returns_none() {
        struct EmptyBrowser;
        #[async_trait]
        impl Browser for EmptyBrowser {
            async fn query(&self, _s: browser_core::Selector) -> anyhow::Result<DomNode> {
                Err(anyhow::anyhow!("no match"))
            }
            async fn execute(&self, _a: &browser_core::Action) -> anyhow::Result<browser_core::ActionResult> {
                Ok(browser_core::ActionResult::Ok)
            }
        }
        let cache = SelectorCache::new();
        assert!(cache.get(&EmptyBrowser, "nope").await.is_none());
    }

    #[tokio::test]
    async fn invalidate_forces_refetch() {
        let b = CountingBrowser { queries: 0.into() };
        let cache = SelectorCache::with_ttl(Duration::from_secs(60));
        let _ = cache.get(&b, ".btn").await;
        cache.invalidate(".btn");
        let _ = cache.get(&b, ".btn").await;
        assert_eq!(b.queries.load(Ordering::SeqCst), 2);
    }
}
