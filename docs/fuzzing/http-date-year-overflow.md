# HTTP dates with a huge year overflow

- **Found by:** `cookies` fuzz target, round 2, 2026-11-07 (2 crashes, both
  `attempt to multiply with overflow` at `rust/netstore/src/http_date.rs:238`).
- **Component:** `southstar_netstore::http_date::parse`. It's used for the
  `Expires` attribute of `Set-Cookie` (`rust/netstore/src/cookies.rs:236`), for HTTP
  cache dates (`rust/http-cache/src/ffi.rs:254`) and in `rust/net/src/ffi/url.rs:14`.
  Every one of these parses a value sent by a server.
- **Impact:**
  - Builds with overflow checks (`cargo xtask build --overflow-checks`, which the Linux
    CI uses, and `--debug`) panic, and with `panic = "abort"` the process dies. Any
    server can crash such a build with one header.
  - Release builds without overflow checks wrap silently and get a wrong timestamp, for
    example far in the past, so a cookie or cache entry can be expired or kept by
    mistake.
- **Severity:** low. A server already controls its own cookies' lifetimes and its
  responses' freshness, so the wrong value in release builds gains it little. The crash
  affects the builds that turn overflow checks on.

## Reproducer

`southstar_netstore::http_date::parse`, run in the dev container:

| Input | Release (no overflow checks) | With overflow checks |
|---|---|---|
| `Sun, 06 Nov 1994 08:49:37 GMT` | `784111777` | `784111777` |
| `Sun, 06 Nov 99999999999 08:49:37 GMT` | `3155695137827974177` | same |
| `Sun, 06 Nov 9999999999999999 08:49:37 GMT` | `1069130988528479265` (wrapped) | **panic** |
| `Sun, 06 Nov 999999999999999999 08:49:37 GMT` | `-3767359434378826719` (wrapped to the past) | **panic** |
| `06 Jun 99999999999999 00:00:00` | `1301963333481396864` (wrapped) | **panic** |

The header that crashes a CI build, for example:

```
Set-Cookie: id=1; Expires=Sun, 06 Nov 9999999999999999 08:49:37 GMT
```

## Cause

`parse` rejects `year < 1583` but has no upper bound. `days_from_civil(year, month, day)`
grows with the year, and `days * 86_400 + hour * 3600 + …` (line 238) overflows `i64`
once the year passes roughly 2.9 × 10¹¹.

## Suggested fix

Bound the year, as other parsers do (for example, reject years above 9999, or clamp to a
maximum timestamp), and compute the result with `checked_mul`/`checked_add`, returning
`None` on overflow. The `parse_iso8601` path should get the same check.

## Verify

The three panicking inputs above should return `None`, or a clamped maximum, in both
build modes, and the `cookies` fuzz target should run clean.
