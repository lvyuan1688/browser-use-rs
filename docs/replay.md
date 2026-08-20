# Action replay (v0.1.6)

> `crates/action-replay` — record + replay browser action sequences.

## Why

A "trace" is the ordered list of `Action`s an agent took plus the
`ActionResult`s it observed. Traces unlock three workflows:

1. **Regression tests** — replay a recorded trace, assert nothing changed.
2. **Offline demos** — replay a trace without a live browser.
3. **Debugging** — diff two traces of the "same" scenario to find where
   they diverged.

## The three pieces

```rust
Trace { entries: Vec<TraceEntry>, meta: BTreeMap<String,String> }
Recorder<B: Browser>     // wraps a Browser, records every (action, result)
replay(browser, trace)   // replays a trace, returns diff list
```

## Recorder

`Recorder` implements `Browser` by delegating to an inner `Browser` and
storing every `(action, result)` pair into an internal `Trace`:

```rust
let recorder = Recorder::new(my_browser)
    .with_meta("scenario", "login-flow");
// ... run the agent loop against `recorder` ...
let trace = recorder.finish();
trace.save(Path::new("login-flow.trace.json"))?;
```

## Replay + diff

`replay(browser, trace)` re-runs every `Action` in the trace against the
supplied `Browser` and returns a list of `(seq, recorded_result, actual_result)`
for the entries that **differ**. An empty diff list means the trace
reproduced exactly.

```rust
let trace = Trace::load(Path::new("login-flow.trace.json"))?;
let diffs = replay(&my_browser, &trace).await?;
assert!(diffs.is_empty(), "regression: {diffs:?}");
```

## Trace format

JSON, schema:

```json
{
  "entries": [
    { "seq": 0, "action": { "kind": "goto", "url": "https://x" },
      "result": { "kind": "ok" } }
  ],
  "meta": { "scenario": "login-flow", "recorded_at": "2026-08-20" }
}
```

`Trace::save` writes pretty-printed JSON to disk; `Trace::load` reads it
back.

## What's NOT in v0.1.6

- DOM-snapshot diffing (only `ActionResult` Debug string is compared)
- Tolerant replay (action retries, fuzzy result matching)
- Trace compression (large traces can be MBs; consider gzip on save)
- Trace editor / merge / split tools
