# Browser Backends

| Backend | Speed | Setup | Use case |
|---|---|---|---|
| **Cdp** (default) | Fastest | Chrome only | Production, high throughput |
| Playwright | Medium | `npx playwright install` | Cross-browser compat |
| Selenium | Slow | Selenium + driver | Legacy systems |

## Cdp direct mode

Bypasses Playwright entirely. Direct Chrome DevTools Protocol connection:

```bash
browser-use-rs --backend cdp "extract top 10 HN stories"
```

Benchmark: 180 actions/min vs Playwright's 40.
