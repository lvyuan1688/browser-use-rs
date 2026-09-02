# Session Persistence

Long browser tasks must survive crashes and be resumable. browser-use-rs snapshots state at
every step so a session can be restarted from the last good checkpoint.

## Snapshot contents

Each step checkpoint captures:
- `url`, `title`, `dom_hash` (structural hash, not full DOM)
- `localStorage` / `sessionStorage` key-value map
- cookie jar (path-scoped, secrets redacted by default)
- open-tab list and active tab id
- step index + token budget consumed so far

## Checkpoint policy

- Write-ahead: the checkpoint is persisted *before* the action executes, so a crash mid-action
  rolls back to the pre-action state (at-least-once, idempotent actions only).
- Retention: keep the last N (default 20) checkpoints; drop older ones LRU.
- Location: `<data_dir>/sessions/<session_id>/step-<n>.snap`.

## Resume

`Session::resume(id)` loads the newest checkpoint, restores storage/cookies, navigates to the
saved URL, waits for the `dom_hash` to re-stabilize, then continues from the next step.
If the page drifted (hash mismatch beyond threshold), the agent re-grounds by re-capturing the
element tree rather than blindly replaying coordinates.

## Budget accounting

Token spend accumulates in the checkpoint, so a resumed session continues from the same budget
and can still enforce a hard stop.
