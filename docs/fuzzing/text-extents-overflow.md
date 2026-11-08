# Text extents overflow `i32` Pango units on very wide lines

- **Found by:** `svg` fuzz target, round 3, 2026-11-07 (1 crash, `attempt to add with
  overflow` at `rust/text/src/glyph.rs:92` in `GlyphString::extents_range`, reached from
  SVG `<text>` layout through `Layout::extents_internal`).
- **Component:** `rust/text` (`southstar-text`): `GlyphString::extents_range`,
  `Layout::extents_internal` and `line_extents_layout_coords` (`layout.rs`),
  `layout/extents.rs` (`run_extents_and_height`, `line_extents_and_height`, the strut and
  CSS line-box unions) and the rounding helpers in `types.rs` (`pixels`, `pixels_ceil`,
  `units_ceil`, `units_round`, `extents_to_pixels`).
- **Impact:**
  - Builds with overflow checks (`cargo xtask build --overflow-checks`, which the Linux
    CI uses, and `--debug`) panic, and with `panic = "abort"` the process dies. In the
    headless engine that's the browser process. In the GTK browser it's the renderer.
  - Release builds wrap and compute wrong ink and logical extents. Whatever uses them
    (line boxes, painting bounds, hit testing) then works from wrong values.
- **Reach:** any page. Layout keeps positions in `i32` Pango units (1/1024 px), so a line
  wider, or a layout taller, than about 2.1 million pixels overflows. Font sizes are capped at 65,535 px, but
  the cap doesn't bound line width.
- **Severity:** low. It's a crash of overflow-checked builds and wrong geometry in
  release builds. No memory-safety impact was found.

## Reproducers

Overflow-checked release build (no LTO) of `main` 01d267c2, headless
(`southstar --headless --url=file://… --dump=png:…`):

| Page | `main` | fixed |
|---|---|---|
| `<pre>` holding 300,000 × `W` (default 16 px monospace) | panic, `glyph.rs:92` | renders |
| `<p style="font-size:65535px;white-space:nowrap">` + 40 × `W` | panic, `glyph.rs:92` | renders |
| `<svg><text font-size="99999">` + 40 × `W` | panic, `glyph.rs:92` | renders |
| a web font whose `H` is a 300-point contour 9 million units wide, at 1000 px and 65,535 px | panic, `types.rs:88` (`extents_to_pixels`) | renders |
| a 595-character paragraph broken after every character (`word-break: break-all`) at `font-size: 2048px` in a `fonts` round-4 input (`unitsPerEm` 72) | panic, `layout.rs:975` (`Layout::extents_internal`) | renders |
| ordinary text, Arabic, SVG text | unchanged | unchanged |

The fuzz input is an SVG with `font-size=9999…9` on a `<text>` element.

## Cause

The port kept Pango's `int` arithmetic. In C the overflow is undefined behaviour that
in practice wraps. In Rust it panics with overflow checks and wraps without them.
`GlyphString::get_width` already used `wrapping_add` for the same reason, but the extents
code did not.

## Fix

Rectangle unions go through one saturating helper, `span_union`. The running x
positions, offsets and the pixel and unit rounding helpers use saturating arithmetic.
Extents clamp at `i32::MAX`, as Blink's `LayoutUnit` clamps, and are unchanged whenever
nothing overflowed. Branch `fix/text-extents-overflow`.

## Verify

The four pages above render in an overflow-checked build, and the `svg` crash input
runs clean in the `svg` fuzz target.
