# Element Resolution

browser-use-rs needs to interact with specific DOM elements (click, type, read). Resolution
is the process of finding the right element from a natural-language or structured description.

## Resolution hierarchy

| Priority | Strategy | When to use |
|----------|----------|-------------|
| 1 | Role + Name | Accessibility-labeled elements (`button "Submit"`, `link "Home"`) |
| 2 | CSS selector | Stable selectors (`#login-form input[type=email]`) |
| 3 | Text content | Visible text match (`"Sign in"` button) |
| 4 | Coordinate | Last resort — pixel (x, y) from bounding box center |

Role+Name is preferred because it survives layout changes and is closest to how a human
identifies an element.

## Re-grounding

When a cached element reference goes stale (DOM mutation, page navigation), the resolver
re-captures the accessibility tree and re-resolves by role+name. If the element is truly gone,
it reports `ElementNotFound` with a suggested alternative (nearest match by role).

## Shadow DOM

Elements inside shadow roots are resolved by recursively walking `shadowRoot` on each host
element. The accessibility tree includes shadow DOM nodes when the browser exposes them
(Chromium does; Firefox support is partial).

## Iframes

Cross-origin iframes require a separate page context. The resolver switches context by
frame index or name, resolves inside, then switches back. Same-origin iframes are traversed
inline.
