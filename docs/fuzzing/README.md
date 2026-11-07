# Fuzzing

`fuzz/` is a [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) crate with its own
workspace, so it never changes the browser build. Each target feeds libFuzzer input
to one parser that handles untrusted bytes. Targets are built with AddressSanitizer,
debug assertions and overflow checks.

## Running

```sh
./fuzz/run.sh
```

`run.sh` builds the `fuzz` stage of `.devcontainer/Dockerfile` (Rust 1.91.0,
cargo-fuzz 0.13.2, `RUSTC_BOOTSTRAP=1` for the sanitizer flags) and runs
`fuzz/container.sh` in a container with no capabilities, `no-new-privileges`, a
read-only root filesystem and the repository mounted read-only except `fuzz/`.
`container.sh` builds every target and fuzzes them in parallel with libFuzzer's
fork mode, continuing past crashes, timeouts and out-of-memory inputs.

| Variable | Default | Meaning |
|----------|---------|---------|
| `FUZZ_SECONDS` | 1200 | Time per target. |
| `FUZZ_JOBS` | half of `nproc` | Targets fuzzed at once. |
| `FUZZ_TARGETS` | all | Space-separated targets to run. |
| `FUZZ_FORK` | 1 | libFuzzer worker processes per target; the CPU limit is `FUZZ_JOBS × FUZZ_FORK`. |
| `FUZZ_VENDOR` | unset | A `cargo vendor` directory; when set the container has no network. |

Corpora go to `fuzz/corpus/<target>/`, findings to `fuzz/artifacts/<target>/` and
logs to `fuzz/logs/`; all three are ignored by git. Reproduce a finding with
`cargo fuzz run -a --sanitizer address <target> fuzz/artifacts/<target>/<file>`
inside the same container.

Nightly Rust was tried first: nightly-2026-11-01 fails to compile the Boa fork with
a query cycle in rustc's async-drop analysis (`error[E0391]` in
`third_party/boa_engine/src/error/mod.rs`), a compiler regression, so the stable
toolchain with `RUSTC_BOOTSTRAP=1` is used instead.

## Targets

| Target | Code under test |
|--------|-----------------|
| `html5_document` | `rust/html5` tokenizer and tree builder, whole documents |
| `html5_fragment` | `rust/html5` fragment parsing with 16 context elements |
| `url` | `rust/url` parsing, resolution, setters, origins, IDNA; asserts that reparsing a serialized URL is stable |
| `http_h1` | `rust/http-wire` status lines, header lines, chunk sizes, chunked bodies |
| `http_hpack` | `rust/http-wire` HPACK decoder over successive header blocks, Huffman decoder |
| `http_h2` | `rust/http-wire` sans-I/O HTTP/2 connection fed server bytes |
| `css_syntax` | `rust/css-syntax` tokenizer and component values |
| `css_sheet` | `rust/css` whole style sheets |
| `css_values` | `rust/css-color`, `rust/selector-text`, `rust/css-prop-syntax` |
| `xml` | `rust/xml` parser building DOM trees |
| `woff` | `rust/woff2` WOFF and WOFF2 decoders (with `brotli-decompressor`, `miniz_oxide`) |
| `raster` | `rust/raster`: `png`, `gif`, `zune-jpeg` and the in-tree BMP decoder |
| `images` | `rust/image-decoders`: WebP (`image-webp`) still and animated, ICO |
| `decompress` | HTTP content decoding: `southstar-http`'s `Inflate` (`flate2`/`zlib-rs`), brotli and zstd driven as `rust/http` drives them |
| `plmpeg` | `rust/plmpeg` MPEG-1 demuxer, video and MP2 decoders |
| `minimp3` | `rust/minimp3` MP3 frame decoder |
| `bidi` | `rust/bidi` UAX #9 paragraph resolution |
| `text_formats` | `rust/webvtt`, `rust/csp`, `rust/mime` |
| `js_eval` | the Boa fork (`third_party/boa_engine`) parsing and running scripts under loop, recursion and stack limits |
| `tls` | the TLS client (`rustls`, `rustls-webpki`, `ring`) fed server bytes, with the browser's provider: `rust/http/src/x25519_mlkem768.rs` is compiled in and offered first; a fixed random source makes the ClientHello reproducible so the fuzzer can echo its session ID |
| `fonts` | the pure-Rust web font path as `rust/text` uses it: `read-fonts` acceptance (`accepts_web_font`), `skrifa` outlines unhinted and hinted (the TrueType hinting interpreter), COLR color glyphs, advances, and `harfrust` shaping of Latin, Arabic, Devanagari, Thai and emoji |
| `wasm` | `wasmi` validation and compilation with the WebAssembly API's default engine, then instantiation and calls to the exports under a fuel limit |
| `cookies` | `rust/netstore`: `Set-Cookie` parsing, the jar's text format, `Cookie` header building in three contexts, HSTS policies and lists, HTTP and ISO 8601 dates |
| `websocket` | `rust/websocket`'s frame decoder with server bytes in varying chunks |
| `eventsource` | `rust/eventsource`'s response head check and `text/event-stream` parser |
| `js_builtins` | the Boa fork's string parsers with the input as a string: `JSON.parse`, `RegExp` patterns and flags, `Date`, Temporal, `Intl` locales and formats, normalization, URI escaping, number and `BigInt` parsing |
| `svg` | `rust/svg` parsing and rendering SVG documents into a cairo surface, including `<text>` through `rust/text` |
| `ipc` | the shell's side of the renderer protocol: `Conn::read_head`/`read_body` over a pipe and the JSON field helpers |
| `media_commands` | `rust/shell/src/procview/media.rs`'s `open` URL vetting, checked against the audio and video helpers' tokenizers |

Seeds came from the repository's HTML, CSS, SVG, XML, image, MP3 and JS files, small
fonts subset from DejaVu Sans into WOFF and WOFF2, and hand-written HTTP/1.1, HTTP/2,
HPACK (RFC 7541 appendix C), MPEG-1, WebVTT, CSP, URL and bidi inputs.

Round 2 added seeds for its new targets: DejaVu Sans (hinted TrueType, Arabic shaping),
Ubuntu Mono `[wght]` (a variable font) and Noto Color Emoji (CBDT) subset with fontTools;
a real CA certificate in a TLS 1.2 Certificate message; TLS 1.3 ServerHellos with
X25519MLKEM768 and X25519 key shares and a HelloRetryRequest; WebSocket frames; cookie,
HSTS and date lines; an event stream; a small WebAssembly module with a loop.

The `wasm` target builds `wasmi` with `portable-dispatch`. With `auto-dispatch` on
x86_64, as `rust/js-wasm` builds it, `wasmi` 2.0 dispatches instructions by tail calls
(`return handler(...)`) and relies on LLVM turning each into a jump, which needs
optimization. AddressSanitizer instrumentation prevents that, so the target's own seed
overflowed the stack, one frame per executed instruction. The browser's release profile
(fat LTO, one codegen unit, `panic = "abort"`), also with overflow checks and at
opt-level 2, ran a 150-million-instruction loop in 0.1 s without growing the stack, so the
shipped engine is not affected. The dependency on the optimizer remains: a toolchain or
target where a handler is not compiled to a jump would let a page overflow the renderer's
stack with a short loop. Building `rust/js-wasm` with `portable-dispatch` removes that
dependency at some speed cost.

See `docs/crate-security-impact.md` for which third-party crates these targets reach.

## Campaign of 2026-11-07

17 targets ran for 20 minutes each, 8 at a time on 8 of 16 CPUs; `images` and
`decompress` then ran for 20 minutes each with 4 workers. No crash, sanitizer
report, panic or out-of-memory input was found, and the URL round-trip assertion
never failed. Timeouts appeared in two targets.

| Target | Coverage | Execs/s | Timeouts |
|--------|---------:|--------:|---------:|
| `js_eval` | 22,443 | 89 | 0 |
| `css_sheet` | 7,829 | 44 | 4 + 1 slow unit |
| `images` | 4,801 | 899 | 0 |
| `decompress` | 4,111 | 204 | 8 |
| `html5_fragment` | 3,425 | 1,440 | 0 |
| `html5_document` | 3,076 | 708 | 0 |
| `woff` | 2,896 | 1,395 | 0 |
| `raster` | 2,796 | 252 | 0 |
| `url` | 2,327 | 4,863 | 0 |
| `css_values` | 2,273 | 6,337 | 0 |
| `plmpeg` | 1,045 | 402 | 0 |
| `bidi` | 1,011 | 2,815 | 0 |
| `xml` | 1,006 | 7,660 | 0 |
| `minimp3` | 911 | 225 | 0 |
| `http_h2` | 776 | 4,392 | 0 |
| `text_formats` | 619 | 19,180 | 0 |
| `css_syntax` | 543 | 454 | 0 |
| `http_hpack` | 311 | 10,465 | 0 |
| `http_h1` | 277 | 12,104 | 0 |

## Campaign of 2026-11-07, round 2

Round 2 ran on upstream `main` (026b2b09) with the three fixes for the round-1 findings
merged in (PRs #307, #308, #309), so the known bugs could not hide new ones. All 25
targets ran for 20 minutes each, 8 at a time. No sanitizer report (out-of-bounds access,
use after free) was found. The `cookies` target found an integer overflow, `js_eval`
found a debug assertion that led to the script-encoding issue, and `decompress` found
two timeouts. Those timeouts came from the harness, which still mirrored the zstd loop
from before #307; it has been updated, and the real decoder handles both inputs at once.

| Target | Coverage | Execs/s | Findings |
|--------|---------:|--------:|---------:|
| `js_eval` | 22,954 | 44 | 1 crash (debug assertion) |
| `fonts` | 9,375 | 498 | 0 |
| `css_sheet` | 8,506 | 58 | 0 |
| `images` | 6,030 | 1,267 | 0 |
| `wasm` | 5,890 | 14,948 | 0 |
| `decompress` | 4,121 | 81 | 2 timeouts (harness) |
| `tls` | 3,723 | 398 | 0 |
| `html5_fragment` | 3,580 | 751 | 0 |
| `html5_document` | 3,185 | 581 | 0 |
| `raster` | 3,076 | 204 | 0 |
| `woff` | 3,005 | 1,026 | 0 |
| `url` | 2,433 | 2,817 | 0 |
| `css_values` | 2,356 | 10,860 | 0 |
| `cookies` | 2,052 | 3,561 | 2 crashes (overflow) |
| `plmpeg` | 1,048 | 444 | 0 |
| `bidi` | 1,011 | 4,668 | 0 |
| `xml` | 1,010 | 2,304 | 0 |
| `minimp3` | 932 | 130 | 0 |
| `http_h2` | 777 | 6,229 | 0 |
| `text_formats` | 705 | 16,257 | 0 |
| `css_syntax` | 543 | 649 | 0 |
| `eventsource` | 361 | 16,704 | 0 |
| `http_hpack` | 311 | 5,577 | 0 |
| `http_h1` | 277 | 11,338 | 0 |
| `websocket` | 156 | 22,591 | 0 |

## Campaign of 2026-11-07, round 3

Round 3 ran on upstream `main` (72b4cd78) with the five fixes from rounds 1 and 2 merged
in. It added two targets, `js_builtins` (the engine's string parsers: JSON, RegExp, dates, Temporal, Intl) and
`svg` (the SVG renderer, including text layout through `rust/text`), plus libFuzzer
dictionaries for the JS, HTML, CSS and URL targets. 24 of the 27 targets ran for 40
minutes each, 6 at a time on 6 CPUs. `text_formats`, `bidi` and `minimp3` were stopped
early; they run again in round 4. The corpora started from round 2's.

No sanitizer report was found. `svg` found an integer overflow in text extents, and
`fonts` found three overflows in `skrifa`'s autohinter, all from fonts with
`unitsPerEm` 0.

| Target | Coverage | Execs/s | Findings |
|--------|---------:|--------:|---------:|
| `js_eval` | 24,577 | 67 | 0 |
| `js_builtins` | 23,217 | 39 | 0 |
| `svg` | 10,945 | 220 | 1 crash (overflow) |
| `fonts` | 10,625 | 491 | 3 crashes (overflow) |
| `wasm` | 10,534 | 13,538 | 0 |
| `css_sheet` | 10,070 | 81 | 0 |
| `images` | 6,313 | 616 | 0 |
| `tls` | 4,379 | 661 | 0 |
| `decompress` | 4,166 | 131 | 0 |
| `html5_fragment` | 3,938 | 974 | 0 |
| `html5_document` | 3,906 | 756 | 0 |
| `raster` | 3,144 | 234 | 0 |
| `woff` | 3,045 | 1,206 | 0 |
| `css_values` | 2,705 | 8,723 | 0 |
| `url` | 2,482 | 4,296 | 0 |
| `cookies` | 2,385 | 4,776 | 0 |
| `xml` | 1,095 | 3,926 | 0 |
| `plmpeg` | 1,048 | 337 | 0 |
| `bidi` | 1,011 | 5,701 | 0 |
| `minimp3` | 934 | 137 | 0 |
| `http_h2` | 777 | 5,531 | 0 |
| `text_formats` | 706 | 15,888 | 0 |
| `css_syntax` | 543 | 837 | 0 |
| `eventsource` | 361 | 14,569 | 0 |
| `http_hpack` | 311 | 7,002 | 0 |
| `http_h1` | 277 | 15,158 | 0 |
| `websocket` | 156 | 31,929 | 0 |

Two targets for the renderer-to-shell boundary ran separately for about 100 minutes
each on 2 CPUs and found nothing:

- `ipc` is the shell's reading of renderer messages (`Conn::read_head`/`read_body` and
  the JSON helpers).
- `media_commands` checks the shell's `open` URL vetting against the audio and video
  helpers' own tokenizers, as a differential.

## Findings

| Report | Found by | Impact |
|--------|----------|--------|
| [zstd-checksum-infinite-loop.md](zstd-checksum-infinite-loop.md) | `decompress` | HTTP decoder thread spins forever on zstd bodies, including valid ones split near the checksum |
| [css-nesting-selector-blowup.md](css-nesting-selector-blowup.md) | `css_sheet` | ~1 s and ~250–840 MB per style sheet from under 200 bytes of nested CSS |
| [pattern-attribute-redos.md](pattern-attribute-redos.md) | review | exponential regex backtracking in `:valid`/`:invalid` style matching, no script needed |
| [script-utf8-escape-swallowing.md](script-utf8-escape-swallowing.md) | `js_eval` (round 2) | external scripts reach Boa undecoded; an invalid UTF-8 lead byte swallows a following escape or quote, so escaped user input can become live code in Southstar only |
| [http-date-year-overflow.md](http-date-year-overflow.md) | `cookies` (round 2) | a huge year in `Expires` or other HTTP dates overflows: a panic in overflow-checked builds, a wrong timestamp in release |
| [text-extents-overflow.md](text-extents-overflow.md) | `svg` (round 3) | lines wider than ~2.1 million px overflow `i32` Pango units in text extents: a panic in overflow-checked builds (a 300,000-character `<pre>` is enough), wrong extents in release |
| [web-font-units-per-em.md](web-font-units-per-em.md) | `fonts` (round 3) | web fonts with `unitsPerEm` 0 or 1 are installed: `skrifa` autohinter overflow (panic in overflow-checked builds), runaway layout and a 10 GB renderer |
