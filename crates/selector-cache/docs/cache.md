# Selector cache (v0.1.7)

> `crates/selector-cache` — TTL-based CSS selector cache.

## Why

DOM queries are the slowest part of browser automation. When the agent
polls the same selector (e.g. waiting for a "Next" button to appear),
each query is a round trip. `SelectorCache` skips the round trip after
the first hit.

## API

```rust
use selector_cache::SelectorCache;
use std::time::Duration;

let cache = SelectorCache::with_ttl(Duration::from_secs(30));
let hit = cache.get(&browser, ".btn").await?;
// Subsequent get() with same selector hits cache, skips browser.query()
```

`SelectorHit { selector, node, cached_at, hits }` — `hits` counts how
many `get()` calls were served by this cache entry.

## Invalidation

- **TTL** — default 30s; entries older than TTL are refetched
- **`invalidate(sel)`** — drop one entry (use after a known mutation)
- **`invalidate_all()`** — drop everything (use after navigation)

## Edge cases

- Selector matches nothing → `get()` returns `None`, no cache entry created
- Stale entry + refetch → queries browser twice
- Concurrent access — `Mutex<HashMap>` is fine for read-heavy DOM workloads

## What's NOT in v0.1.7

- MutationObserver-driven invalidation (would need a JS hook)
- LRU eviction (currently unbounded — fix if memory matters)
- Negative caching (selector returns `None` → still re-queried)
- Per-selector TTL override
