# The `pattern` attribute runs an unbounded backtracking regex during style matching

- **Found by:** code review while ranking crates for fuzzing, 2026-11-07, then
  measured. Not found by a fuzz target: no target exercises `pattern`.
- **Component:** `rust/css/src/pattern.rs`, called from
  `rust/css/src/ffi/element_state.rs:29` (`:valid` / `:invalid` matching,
  `rust/css/src/element_state.rs:688`), `rust/forms/src/lib.rs:225` (form
  validation) and `rust/js-forms/src/validity.rs:88` (`checkValidity()` and
  friends). The engine is the `regress` crate.
- **Impact:** denial of service. A page with no script can make its renderer spend
  seconds to hours in style recalculation.
- **Severity:** low to medium. CPU only, confined to the tab's renderer, and the
  watchdog reports a hang after 60 s; but it needs no JavaScript, so it also
  affects pages viewed with script disabled.

## Reproducer

```html
<style>input:invalid { outline: 2px solid red }</style>
<input pattern="(a+)+" value="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa!">
```

Each extra `a` in `value` roughly doubles the time of every style match of that
element.

## Measurements

`southstar_css::pattern_matches(b"(a+)+", value)`, release build without
sanitizers, one CPU, in the dev container:

| `value` | Time |
|---------|-----:|
| 16 × `a` + `!` | 3 ms |
| 20 × `a` + `!` | 38 ms |
| 22 × `a` + `!` | 149 ms |
| 24 × `a` + `!` | 605 ms |
| 26 × `a` + `!` | 2,412 ms |

At that rate 30 characters take about 40 s and 40 characters several hours.

## Cause

`pattern_matches` builds `^(?:pattern)$` with the `v` flag and runs it with
`regress`, a backtracking engine. `regress` 0.12 has no step or time limit (its
only budget is an optimizer setting). The pattern is compiled again on every
call, and the style system calls it whenever it evaluates `:valid` or `:invalid`
for a text input with a value, so one page can trigger it repeatedly.

## Suggested fix

- Bound the match: add a backtracking step limit to the matcher (in-tree, or in
  the vendored `regress` through the Boa fork's patch set) and treat an exhausted
  limit as "matches", as `pattern_matches` already does for a pattern that fails
  to compile.
- Cache the compiled regex per element or per pattern string, and cache the
  validity result until the value or the attribute changes, instead of
  recomputing it during every style match.
- Add a fuzz target for `pattern_matches` with a short timeout so regressions show
  up as timeouts.

## Verify

Load the reproducer with 40 `a`s; the tab should stay responsive and the input
should match `:invalid` or `:valid` within milliseconds.
