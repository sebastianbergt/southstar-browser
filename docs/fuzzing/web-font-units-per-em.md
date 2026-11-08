# Web fonts with `unitsPerEm` 0 or 1 are installed

- **Found by:** `fonts` fuzz target, round 3, 2026-11-07. There were 3 crashes, all in
  `skrifa` 0.49.0's autohinter: `attempt to subtract with overflow` at
  `src/outline/autohint/hint/outline.rs:128` (twice) and `:365` (`iup_interpolate`).
- **Component:** `southstar_text::accepts_web_font` (`rust/text/src/engine.rs`). It's the
  gate for `@font-face` fonts on the pure-Rust path, called from `rust/font/src/lib.rs`.
  It checked only that `head`, `maxp`, `cmap`, `hhea` and `hmtx` parse. All three crash
  inputs have `head.unitsPerEm = 0`.
- **Impact:** a web font with a `unitsPerEm` that OpenType forbids is installed and used:
  - `unitsPerEm` 0:
    - `skrifa` falls back to a scale of 1.0 and hints raw font units in 26.6 fixed
      point. Long `glyf` contours, whose 16-bit deltas add up to millions of units, then
      overflow the autohinter's `i32` coordinates.
    - Builds with overflow checks (the Linux CI build, `--debug`) abort the renderer.
      Release builds draw mangled glyphs.
    - Layout divides metrics by zero, and a test page grew past cairo's 32,767-pixel
      surface limit.
  - `unitsPerEm` 1: every glyph is scaled by the full font size. 16 px text in a DejaVu
    Sans subset took the renderer past 10 GB, and it was killed. This happens with and
    without overflow checks.
- **Reach:** any page can load a web font. The browser hints web fonts with
  `Engine::AutoFallback`, or with `Engine::Auto` for light hinting, so fonts without
  TrueType instructions go through the autohinter.
- **Severity:** low to medium. It's a renderer crash in overflow-checked builds, and
  memory exhaustion of the renderer in all builds. No memory-safety impact was found
  (`skrifa` is safe Rust).

## Reproducers

These pages use a DejaVu Sans subset with hinting removed (`fontTools.subset`,
`hinting=False`) and `head.unitsPerEm` patched. They were loaded through `@font-face` in
an overflow-checked build of `main` 01d267c2:

| `unitsPerEm` | `main` | fixed |
|---|---|---|
| 2048 (unchanged) | renders | renders (same) |
| 0 | page outgrows cairo's surface limit | font rejected, fallback font |
| 1 | renderer passes 10 GB and is killed | font rejected, fallback font |
| 8 | renders (scaled 256×) | font rejected, fallback font |

The fuzz harness hints glyph IDs 0–23 directly, so it reaches the `skrifa` overflows
with fonts whose `cmap` maps none of the page's characters.

## Cause

OpenType requires `unitsPerEm` in 16..16384. OTS, the sanitizer Chrome and Firefox run
on every web font, rejects fonts outside that range. Southstar's gate didn't check it,
and `skrifa` copes with `unitsPerEm` 0 by not scaling at all.

## Fix

`accepts_web_font` also requires `head.unitsPerEm` in 16..16384, so such fonts are not
installed and the page falls back to the next family. All 411 fonts on the test machine
have `unitsPerEm` 256, 1000, 1024 or 2048. Branch `fix/web-font-units-per-em`.

## Upstream

`skrifa`'s autohinter does its coordinate arithmetic in `i32` without checks, both in
0.49.0 (the latest release) and on fontations `main`. FreeType uses `FT_Pos`, which is
64-bit on LP64 Linux and macOS. Other overflows may be reachable with a valid
`unitsPerEm`, so the arithmetic is worth reporting to fontations.

Round 4 confirmed it. Three inputs with a valid `unitsPerEm` of 72 overflow at
`outline.rs:365` (`iup_interpolate`) when hinted at 2,048 px, but not at 1,024 px or
below. In the browser these fonts at `font-size: 2048px` hit Southstar's layout overflow
first (fixed in #315). After that fix the pages render, and the crashing glyph could not
be reached from page text. A fix belongs in `skrifa`: 64-bit or saturating coordinates
in the autohinter, as FreeType's `FT_Pos` gives on LP64.
