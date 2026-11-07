# CSS nesting expands selector lists exponentially

- **Found by:** `css_sheet` fuzz target, 2026-11-07 (four timeouts and one slow unit).
- **Component:** `rust/css/src/nesting.rs` (CSS Nesting flattener), then the selector
  parser in `rust/css`.
- **Impact:** denial of service. A page makes its renderer spend about a second and
  hundreds of megabytes per style sheet, from under 200 bytes of CSS, with no
  script. Memory use is not bounded across sheets.
- **Severity:** medium. The renderer has no memory limit (no `RLIMIT_AS`), so many
  sheets can push the whole machine towards the out-of-memory killer, and the
  watchdog only treats a renderer as hung after 60 s (`HANG_MIN_SECS`).
- **Memory safety:** none found. The fuzzer saw no crash or sanitizer report.

## Reproducer

A rule whose selectors are lists, nested 18 deep (173 bytes):

```python
css = "a, b { " * 18 + "color: red;" + " }" * 18
```

```html
<style>a, b { a, b { a, b { a, b { a, b { a, b { a, b { a, b { a, b { a, b {
a, b { a, b { a, b { a, b { a, b { a, b { a, b { a, b { color: red;
} } } } } } } } } } } } } } } } } }</style>
```

Repeating the `<style>` element multiplies the cost. The fuzzer's inputs, minimized
from 3–9 KB to 0.4–3.6 KB, are the same pattern formed by unbalanced `{` in
ordinary rules such as `div, p, section { … div, p { …`.

## Measurements

`southstar_css::parse_url_cached(b"", css)`, release build without sanitizers, one
CPU, in the dev container:

| Input | Size | Time | Peak RSS |
|-------|-----:|-----:|---------:|
| ordinary rules (80 × `div, p, … { display: block; }`) | 6,320 B | 0.3 ms | — |
| `a, b` nested 12 deep | 119 B | 13 ms | 17 MB |
| `a, b` nested 14 deep | 137 B | 64 ms | 56 MB |
| `a, b` nested 16 deep | 155 B | 260 ms | 213 MB |
| `a, b` nested 18 deep | 173 B | 1,018 ms | 841 MB |
| `a, b, c` nested 12 deep | 155 B | 1,099 ms | 1,173 MB |
| fuzz artifact `timeout-46503e…`, minimized | 427 B | 504 ms | — |
| fuzz artifact `timeout-46503e…` | 7,647 B | 1,883 ms | — |

Parsing the 173-byte sheet four times and keeping the sheets used 0.72–0.80 s each,
and retained memory grew by about 250 MB per sheet (1.09 GB → 1.59 GB). With a
nesting depth of 20 the flattener hits its budget early and the parse is fast
again, so the worst case sits just below the budget.

## Cause

The flattener rewrites nested rules as text. For every comma-separated part of a
nested selector, `combine_selectors` (`nesting.rs:122`) inserts a full copy of the
parent selector wrapped in `:is(…)`. With *n* parts per level the selector text
grows by a factor of *n* per level: 2^18 copies of `a` at depth 18.

The only brake is `NEST_SELECTOR_BUDGET` (`nesting.rs:11`): each call to
`flatten` may emit `16 MiB + 16 × input length` bytes of selector text. That cap
is per sheet, and 16 MiB of selector text is then parsed into selector
structures roughly 50 times larger.

## Suggested fix

- Do not copy the parent selector into each part. Keep the parent's parsed
  selector list and have nested selectors refer to it (`&` as a reference to an
  `:is()` of the parent list), as the CSS Nesting spec describes; the size is then
  linear in the input.
- Until then, make the budget proportional to the input (for example `64 KiB + 4 ×
  input length`) and share it per document, so separate `<style>` elements and
  `@import`s cannot each claim 16 MiB.
- Consider an address-space or cgroup memory limit for renderer processes, so one
  tab cannot exhaust system memory.

## Verify

Rerun the `css_sheet` target and time the reproducer above; it should parse in
well under a millisecond.
