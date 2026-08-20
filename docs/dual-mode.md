# Dual mode (DOM + Vision)

> v0.1.5 — `crates/vision-mod` adds screenshot diff + similarity.

## The two modes

browser-use-rs runs the agent loop against one of two "views" of the page:

| Mode | Input to the agent | Cost | When to use |
|------|-------------------|------|-------------|
| **DOM** | Extracted DOM tree, compressed by `crates/compress` | Cheap: one serialization | Form fills, navigation, predictable structure |
| **Vision** | Screenshot PNG + bounding boxes | Expensive: LLM vision call | Visual layouts, dynamic content, "looks right?" checks |

The agent can switch between modes per iteration: DOM for cheap scraping,
Vision for verification.

## vision-mod's role

After every action, the agent takes a screenshot. `vision-mod::compare`
scores the new screenshot against the previous one:

- `similarity` ∈ [0, 1] — 1.0 = identical, 0.0 = completely different
- `changed_fraction` — fraction of pixels that differ beyond the threshold

The agent uses this to decide:

- `similarity > 0.99` → the action had **no visible effect**. Try again or
  fall back to Vision to understand why.
- `similarity < 0.5` → the page **changed dramatically**. Worth a Vision
  check before the next action.
- In between → normal "did the click register?" signal.

## Implementation details

### Downscaling

Comparing full-resolution screenshots pixel-by-pixel is both slow and
over-sensitive to JPEG noise. `vision-mod` downscales to 64×36 (the
16:9-ish thumbnail) before comparing.

### Mean-squared diff

The similarity score is derived from the mean-squared per-pixel
difference, normalized to [0, 1] where 1.0 = identical. This is
intentionally **coarse** — it's a quick "did anything change" signal,
not a perceptual hash.

### Threshold

`CHANNEL_THRESH = 24` — any per-channel difference below this is treated
as "the same". This avoids counting JPEG re-encoding noise as a change.

## Future work

- SSIM / perceptual hash for more accurate "looks different" detection
- Region-of-interest diff (only compare the part of the page the agent is
  interacting with)
- Color-space aware comparison (HSV diff handles light changes better)
