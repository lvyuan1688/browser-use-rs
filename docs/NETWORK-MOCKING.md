# Network Mocking

`browser-use-rs` can intercept and mock network traffic inside the
browser context, so agent tests are deterministic and do not depend on
live third-party services.

## Enabling the interceptor

Pass `--mock-network <route-file>` on the command line, or set it in
the profile:

```toml
[network]
mock = "fixtures/api-routes.toml"
capture_unmatched = true   # log requests that miss a route
```

## Route file format

A route file is a list of matchers and responses. Matchers are applied
in order; the first match wins.

```toml
[[routes]]
match.url = "https://api.example.com/login"
match.method = "POST"
response.status = 200
response.headers = { content-type = "application/json" }
response.body = '{"token":"fake-token-123"}'

[[routes]]
match.url_regex = "^https://api\\.example\\.com/users/\\d+$"
response.status = 200
response.body_file = "fixtures/user-42.json"
```

Use `url` for exact matches and `url_regex` for patterns. The response
body can be inline (`body`) or loaded from disk (`body_file`).

## Capturing real traffic

To bootstrap a fixture, run once with capture mode:

```bash
browser-use-rs run task.yaml --capture-network fixtures/real.yaml
```

Every request/response the browser makes is written to `fixtures/real.yaml`
in the route format. Edit it down to the calls you want to mock, then
point `network.mock` at it.

## Latency and failure injection

Routes can simulate slow or failing endpoints:

```toml
[[routes]]
match.url_regex = "^https://api\\.example\\.com/slow"
response.status = 200
response.body = '{}'
response.delay_ms = 3000     # block 3s before responding

[[routes]]
match.url_regex = "^https://api\\.example\\.com/flaky"
response.status = 503
response.body = '{"error":"overloaded"}'
response.repeat = 3          # fail 3 times, then fall through
```

`repeat` makes the route fire N times before yielding to the next
matcher — handy for testing retry logic.

## Verification

```bash
browser-use-rs doctor --check network-mock
```

Loads the route file, reports how many routes compiled, flags any regex
that failed to parse, and prints the unmatched-request count from the
last run.
