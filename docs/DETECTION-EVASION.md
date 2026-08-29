# Detection Evasion

browser-use-rs applies runtime evasion techniques so automated sessions survive bot-detection heuristics without resorting to brittle patch lists.

## Threat model

We assume a target that deploys:
- TLS fingerprinting (JA3/JA4)
- WebDriver flag detection (`navigator.webdriver`)
- Behavioral timing analysis
- Headless Chrome artifact detection

We do **not** attempt to defeat CAPTCHA challenges; that is out of scope for this crate.

## Approach: native, not headless

The single most effective evasion is to use a real browser binary with a real user profile, rather than a headless instance. browser-use-rs launches a persistent Chrome profile (no `--headless`), and drives it via the DevTools Protocol over an isolated port.

```rust
let mut browser = Browser::builder()
    .profile_dir(&profile)           // persistent profile, no --temp
    .port(0)                          // OS-assigned ephemeral port
    .flag("disable-blink-features", "AutomationControlled")
    .build().await?;
```

## WebDriver flag

The `AutomationControlled` flag is removed at launch. We also inject a stealth script at the earliest possible document lifecycle point (`Page.addScriptToEvaluateOnNewDocument`) to normalize `navigator.webdriver`, `navigator.plugins`, and `chrome.runtime` before any page script runs.

## Timing

Detection systems flag sessions where inter-action intervals are perfectly uniform. browser-use-rs applies a Gaussian jitter (μ=850ms, σ=220ms) to:
- Time between page load and first action
- Time between keystrokes
- Time between mouse moves

The jitter distribution is configurable:
```toml
[evasion.timing]
mean_ms = 850
stddev_ms = 220
min_ms = 400
```

## What we don't do

- We do not spoof JA3 fingerprints; that requires a custom TLS stack and is fragile. If you need JA3 evasion, pair browser-use-rs with a TLS proxy (e.g. utls).
- We do not rotate User-Agent strings; modern detection correlates UA with other signals, so rotation buys little.
- We do not proxy through residential IP pools; that is an infrastructure decision left to the operator.

See `docs/NETWORK-MOCKING.md` for how to test evasion logic against mocked network responses.
