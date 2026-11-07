# Third-party crates by security impact

Date: 2026-11-07. This ranks the 237 crates.io packages in `Cargo.lock` by how directly
a hostile web page or network peer can reach them in the browser. The dependency
graph comes from `cargo metadata --filter-platform x86_64-unknown-linux-gnu`; the
`unsafe` column counts the keyword in each crate's `src/` (comments included, so
it is an upper bound); "forbid" means `lib.rs` carries `forbid(unsafe_code)`,
possibly with `unsafe` behind an opt-in feature.

| Tier | Meaning | Crates |
|------|---------|--------|
| 1 | Parses attacker-controlled bytes: page images, fonts, compressed bodies, TLS, wasm, regexes. A memory-safety bug here is a renderer or network-stack compromise. | 51 |
| 2 | Inside the JS engine and Intl/Temporal: reached by any script, mostly with typed values rather than raw bytes. Bugs are more likely denial of service or logic errors. | 130 |
| 3 | Linked but sees no attacker input: compiled-in ICU data and code-generation helpers. | 8 |
| Build | Runs only on the build machine: proc macros, build scripts, `cc`. No runtime exposure, but a supply-chain risk because they execute during the build. | 34 |
| Other OS | Not compiled on Linux (Windows import libraries, WASI, Redox, UEFI). | 14 |

## Notes on the riskiest crates

- **`zlib-rs`** (the `flate2` backend for gzip/deflate bodies, PNG-independent) has
  the most `unsafe` among tier-1 crates (about 450 uses) and sees every compressed
  response and `DecompressionStream` input. `brotli-decompressor`, `ruzstd` and
  `zune-jpeg` also carry `unsafe` on paths fed by the network.
- **`ring`** contains C and assembly as well as Rust; with `rustls` and
  `rustls-webpki` it processes bytes from any server before the certificate is
  checked.
- **`wasmi`** interprets page-supplied WebAssembly with about 300 `unsafe` uses;
  a bug there is reachable from any page that can run script.
- **`regress`** is a backtracking engine with no step limit. Besides JS `RegExp`,
  it runs the HTML `pattern` attribute (`rust/css/src/pattern.rs`), which the
  style system evaluates for `:valid`/`:invalid` and which is compiled on every
  call, so a page can make style recalculation backtrack without running script.
- **Fonts** (`read-fonts`, `skrifa`, `harfrust`) are pure Rust and forbid
  `unsafe` in their parsers; downloaded web fonts never reach FreeType/HarfBuzz.
- **`rustc-hash`** (FxHash) keys Boa's sparse array elements
  (`third_party/boa_engine/src/object/property_map.rs`). FxHash is not
  collision-resistant, so script-chosen indices can degrade those maps; script can
  already burn CPU, so this is low impact.
- Tier-1 parsers that are **in-tree** rather than crates (HTML, CSS, URL, XML,
  HTTP/1.1 and HTTP/2 framing, HPACK, WOFF, BMP/ICO, MPEG-1, MP3, WebVTT, CSP)
  are covered by the fuzz targets in `fuzz/`; see `docs/fuzzing/README.md`.

## Tier 1: parses untrusted input — Images from pages

Decodes attacker-supplied image bytes in the renderer.

| Crate | `unsafe` | forbid | Lines | Used directly by |
|-------|---------:|:------:|------:|------------------|
| adler2-2.0.1 | 1 | yes | 442 | (indirect) |
| byteorder-lite-0.1.0 | 2 | yes | 5049 | (indirect) |
| crc32fast-1.5.4 | 20 |  | 1919 | (indirect) |
| fdeflate-0.3.7 | 0 | yes | 2276 | (indirect) |
| gif-0.14.2 | 0 | yes | 3146 | raster |
| image-webp-0.2.4 | 0 | yes | 8670 | image-decoders |
| miniz_oxide-0.8.9 | 3 | yes | 6948 | (indirect) |
| png-0.18.1 | 0 | yes | 12674 | raster, text |
| simd-adler32-0.3.10 | 36 |  | 1958 | (indirect) |
| weezl-0.1.12 | 0 | yes | 3298 | (indirect) |
| zune-core-0.5.3 | 10 |  | 3206 | (indirect) |
| zune-jpeg-0.5.15 | 85 | yes | 9737 | raster |

## Tier 1: parses untrusted input — Web fonts

Parses, shapes and rasterizes downloaded @font-face fonts (WOFF/WOFF2 unwrap included).

| Crate | `unsafe` | forbid | Lines | Used directly by |
|-------|---------:|:------:|------:|------------------|
| font-types-0.12.7 | 4 |  | 3795 | (indirect) |
| harfrust-0.15.0 | 29 | yes | 42204 | text |
| miniz_oxide-0.9.1 | 3 | yes | 7299 | woff2 |
| read-fonts-0.46.0 | 0 | yes | 86087 | text |
| skrifa-0.49.0 | 0 | yes | 41381 | text |

## Tier 1: parses untrusted input — HTTP content decoding

Decompresses response bodies (gzip/deflate/br/zstd) and WOFF2 tables, and DecompressionStream in JS.

| Crate | `unsafe` | forbid | Lines | Used directly by |
|-------|---------:|:------:|------:|------------------|
| alloc-no-stdlib-2.0.4 | 10 |  | 951 | (indirect) |
| alloc-stdlib-0.2.4 | 10 | yes | 470 | (indirect) |
| brotli-decompressor-5.0.3 | 100 | yes | 32751 | http, woff2 |
| flate2-1.1.11 | 37 |  | 8064 | http, js-compression |
| ruzstd-0.9.1 | 39 |  | 12642 | http |
| zlib-rs-0.6.8 | 456 |  | 17812 | (indirect) |

## Tier 1: parses untrusted input — TLS and crypto

Handles bytes from any network peer before authentication; key exchange and certificate validation.

| Crate | `unsafe` | forbid | Lines | Used directly by |
|-------|---------:|:------:|------:|------------------|
| cmov-0.5.4 | 28 |  | 1702 | (indirect) |
| cpufeatures-0.3.1 | 11 |  | 589 | (indirect) |
| crypto-common-0.2.2 | 0 | yes | 853 | (indirect) |
| ctutils-0.4.8 | 1 |  | 4004 | (indirect) |
| digest-0.11.3 | 0 | yes | 2857 | (indirect) |
| getrandom-0.2.17 | 58 |  | 1739 | (indirect) |
| hybrid-array-0.4.15 | 43 |  | 3229 | (indirect) |
| keccak-0.2.2 | 10 |  | 664 | (indirect) |
| kem-0.3.0 | 0 | yes | 251 | (indirect) |
| ml-kem-0.3.4 | 0 |  | 2603 | http |
| module-lattice-0.2.4 | 0 |  | 926 | (indirect) |
| ring-0.17.14 | 230 |  | 24681 | digest, http, websocket |
| rustls-0.23.46 | 5 |  | 49208 | http |
| rustls-pki-types-1.15.1 | 1 |  | 4103 | (indirect) |
| rustls-webpki-0.103.16 | 0 |  | 10197 | (indirect) |
| sha3-0.12.0 | 0 | yes | 246 | (indirect) |
| shake-0.1.0 | 0 | yes | 206 | (indirect) |
| sponge-cursor-0.1.0 | 5 |  | 390 | (indirect) |
| subtle-2.6.1 | 2 |  | 1008 | (indirect) |
| typenum-1.20.1 | 0 | yes | 19354 | (indirect) |
| untrusted-0.9.0 | 0 |  | 402 | (indirect) |
| zeroize-1.9.1 | 20 |  | 1061 | http |

## Tier 1: parses untrusted input — WebAssembly

Validates and interprets wasm modules supplied by pages.

| Crate | `unsafe` | forbid | Lines | Used directly by |
|-------|---------:|:------:|------:|------------------|
| wasmi-2.0.0 | 302 |  | 42840 | js-wasm |
| wasmi_collections-2.0.0 | 27 |  | 4295 | (indirect) |
| wasmi_core-2.0.0 | 12 |  | 7630 | (indirect) |
| wasmi_ir-2.0.0 | 3 |  | 2055 | (indirect) |
| wasmparser-0.228.0 | 1 |  | 37120 | (indirect) |

## Tier 1: parses untrusted input — Regular expressions

Compiles and runs page-supplied patterns: JS RegExp, the HTML pattern attribute (also during style matching) and extension rules.

| Crate | `unsafe` | forbid | Lines | Used directly by |
|-------|---------:|:------:|------:|------------------|
| regress-0.12.0 | 59 |  | 45052 | boa-engine, boa-parser, css, extensions |

## Tier 2: reached by page script — JavaScript engine runtime

Data structures, numbers and strings inside the Boa engine; reached by any script but not parsers of raw bytes.

| Crate | `unsafe` | forbid | Lines | Used directly by |
|-------|---------:|:------:|------:|------------------|
| aligned-vec-0.6.4 | 57 |  | 1834 | boa-engine |
| allocator-api2-0.2.21 | 277 |  | 8053 | (indirect) |
| arrayvec-0.7.8 | 58 |  | 2413 | boa-engine, boa-gc |
| async-channel-2.5.0 | 0 | yes | 1388 | boa-engine |
| bitflags-2.13.2 | 3 | yes | 6018 | boa-ast, boa-engine, boa-parser |
| bytemuck-1.25.2 | 376 |  | 4162 | boa-engine |
| cfg-if-1.0.5 | 0 |  | 218 | boa-engine |
| chacha20-0.10.2 | 66 |  | 2765 | (indirect) |
| concurrent-queue-2.5.0 | 17 |  | 1821 | (indirect) |
| core_maths-0.1.1 | 0 |  | 1221 | (indirect) |
| cow-utils-0.1.3 | 7 |  | 249 | boa-engine |
| critical-section-1.2.0 | 20 |  | 603 | (indirect) |
| crossbeam-utils-0.8.23 | 80 |  | 4168 | (indirect) |
| dashmap-6.2.1 | 120 |  | 4649 | boa-engine |
| dynify-0.1.2 | 99 |  | 2200 | boa-engine |
| either-1.19.0 | 2 |  | 2477 | (indirect) |
| equator-0.4.2 | 46 |  | 1778 | (indirect) |
| equivalent-1.0.3 | 0 |  | 119 | (indirect) |
| erased-serde-0.4.11 | 110 |  | 4012 | (indirect) |
| event-listener-5.4.2 | 28 |  | 2737 | (indirect) |
| event-listener-strategy-0.5.4 | 1 |  | 549 | (indirect) |
| fast-float2-0.2.4 | 34 |  | 2255 | boa-engine, boa-parser, boa-string |
| fastrand-2.5.0 | 0 | yes | 970 | (indirect) |
| fixed_decimal-0.7.3 | 1 |  | 9442 | boa-engine |
| fixedbitset-0.5.7 | 77 |  | 2454 | (indirect) |
| float16-0.1.7 | 102 |  | 6766 | boa-engine |
| foldhash-0.1.5 | 3 |  | 1068 | (indirect) |
| foldhash-0.2.0 | 11 |  | 1128 | (indirect) |
| futures-concurrency-7.7.1 | 102 |  | 10091 | boa-engine |
| futures-core-0.3.34 | 8 |  | 840 | (indirect) |
| futures-io-0.3.34 | 0 |  | 561 | (indirect) |
| futures-lite-2.6.1 | 2 |  | 7644 | boa-engine |
| getrandom-0.4.3 | 114 |  | 2582 | (indirect) |
| hashbrown-0.14.5 | 326 |  | 22796 | (indirect) |
| hashbrown-0.15.5 | 324 |  | 22213 | (indirect) |
| hashbrown-0.17.1 | 495 |  | 23720 | boa-engine, boa-gc, boa-interner |
| indexmap-2.14.2 | 12 |  | 13676 | boa-ast, boa-engine, boa-interner |
| intrusive-collections-0.10.3 | 418 |  | 11571 | boa-engine |
| itertools-0.15.0 | 10 |  | 15639 | boa-engine |
| itoa-1.0.18 | 13 |  | 488 | boa-string |
| libc-0.2.191 | 810 |  | 133106 | (indirect) |
| libm-0.2.16 | 57 |  | 19676 | boa-engine |
| lock_api-0.4.14 | 302 |  | 5425 | (indirect) |
| memchr-2.8.3 | 340 |  | 15824 | (indirect) |
| num-bigint-0.5.1 | 12 |  | 10768 | boa-ast, boa-engine, boa-parser |
| num-conv-0.2.2 | 0 |  | 354 | (indirect) |
| num-integer-0.1.47 | 0 |  | 1888 | boa-engine |
| num-traits-0.2.19 | 1 |  | 7815 | boa-engine, boa-parser |
| num_enum-0.7.6 | 2 |  | 99 | boa-engine |
| once_cell-1.21.4 | 51 |  | 2589 | boa-interner |
| oneshot-0.2.1 | 118 |  | 2094 | boa-engine |
| parking-2.2.1 | 0 | yes | 449 | (indirect) |
| parking_lot_core-0.9.12 | 149 |  | 3687 | (indirect) |
| phf-0.14.0 | 0 |  | 1407 | boa-interner |
| phf_shared-0.14.0 | 1 |  | 942 | (indirect) |
| pin-project-1.1.13 | 13 |  | 352 | (indirect) |
| pin-project-lite-0.2.17 | 13 |  | 1770 | (indirect) |
| portable-atomic-1.15.0 | 725 |  | 27601 | boa-engine |
| rand-0.10.3 | 20 |  | 10365 | boa-engine |
| rand_core-0.10.1 | 0 |  | 1073 | (indirect) |
| rustc-hash-2.1.3 | 0 |  | 649 | boa-ast, boa-engine, boa-interner, boa-parser, boa-string |
| ryu-1.0.23 | 30 |  | 2763 | (indirect) |
| ryu-js-1.0.3 | 43 |  | 4024 | boa-engine, boa-string |
| scopeguard-1.2.0 | 4 |  | 595 | (indirect) |
| serde-1.0.229 | 2 |  | 17237 | boa-engine |
| serde_core-1.0.229 | 2 |  | 12037 | (indirect) |
| serde_json-1.0.154 | 16 |  | 18387 | boa-engine, js-engine |
| siphasher-1.0.4 | 12 |  | 1794 | (indirect) |
| small_btree-0.1.0 | 0 |  | 990 | boa-engine |
| smallvec-1.16.5 | 83 |  | 4150 | (indirect) |
| spin-0.9.9 | 136 |  | 4773 | (indirect) |
| static_assertions-1.1.0 | 1 |  | 1133 | boa-engine, boa-interner, boa-string |
| string-interner-0.19.0 | 40 |  | 1990 | (indirect) |
| strum-0.28.0 | 0 |  | 362 | boa-ast |
| tag_ptr-0.1.0 | 3 |  | 126 | boa-engine |
| tap-1.0.1 | 0 |  | 1055 | boa-engine |
| thin-vec-0.2.21 | 115 |  | 5055 | boa-engine, boa-gc |
| thiserror-2.0.21 | 0 |  | 466 | boa-engine |
| typeid-1.0.3 | 8 |  | 222 | (indirect) |
| utf16_iter-1.1.0 | 0 | yes | 878 | (indirect) |
| utf8_iter-1.1.0 | 16 |  | 1418 | (indirect) |
| utf_types-1.0.0 | 31 |  | 2636 | (indirect) |
| write16-1.0.0 | 0 |  | 208 | (indirect) |
| writeable-0.6.4 | 3 |  | 2956 | boa-engine |
| xsum-0.1.6 | 0 |  | 1371 | boa-engine |
| zmij-1.0.23 | 70 |  | 2043 | (indirect) |

## Tier 2: reached by page script — Dates, Temporal and Intl

Parses date, time-zone and locale strings from scripts and formats them; ICU data is compiled in.

| Crate | `unsafe` | forbid | Lines | Used directly by |
|-------|---------:|:------:|------:|------------------|
| boa_icu_provider-0.22.0 | 0 |  | 140 | boa-engine |
| calendrical_calculations-0.2.4 | 0 |  | 6622 | (indirect) |
| cobs-0.3.0 | 0 |  | 788 | (indirect) |
| databake-0.2.1 | 2 |  | 1052 | (indirect) |
| deranged-0.5.9 | 74 |  | 3049 | (indirect) |
| icu_calendar-2.3.0 | 1 |  | 21835 | boa-engine |
| icu_casemap-2.3.0 | 2 |  | 4761 | boa-engine |
| icu_collator-2.3.1 | 10 |  | 7376 | boa-engine |
| icu_collections-2.3.0 | 16 |  | 6141 | (indirect) |
| icu_datetime-2.3.0 | 6 |  | 31448 | boa-engine |
| icu_decimal-2.3.0 | 0 |  | 2379 | boa-engine |
| icu_list-2.3.0 | 6 |  | 1482 | boa-engine |
| icu_locale-2.3.1 | 0 |  | 6445 | boa-engine |
| icu_locale_core-2.4.0 | 6 |  | 10422 | boa-gc |
| icu_locale_fallback-2.3.0 | 1 |  | 977 | (indirect) |
| icu_normalizer-2.3.0 | 36 |  | 4249 | boa-engine |
| icu_pattern-0.5.1 | 5 |  | 4076 | (indirect) |
| icu_plurals-2.3.0 | 15 |  | 5302 | boa-engine |
| icu_properties-2.3.0 | 0 |  | 14559 | boa-parser |
| icu_provider-2.4.0 | 20 |  | 5828 | boa-engine |
| icu_provider_adapters-2.3.0 | 0 |  | 1624 | (indirect) |
| icu_provider_blob-2.3.0 | 0 |  | 822 | (indirect) |
| icu_time-2.3.0 | 0 |  | 5220 | boa-engine |
| ixdtf-0.6.7 | 0 |  | 4669 | (indirect) |
| litemap-0.8.3 | 0 |  | 3162 | (indirect) |
| num_threads-0.1.8 | 7 |  | 205 | (indirect) |
| postcard-1.1.3 | 12 |  | 5277 | (indirect) |
| potential_utf-0.1.6 | 11 |  | 828 | (indirect) |
| powerfmt-0.2.1 | 5 |  | 1284 | (indirect) |
| quick-error-2.0.1 | 0 |  | 1336 | (indirect) |
| regex-automata-0.4.18 | 57 |  | 65894 | (indirect) |
| regex-syntax-0.8.11 | 0 | yes | 58465 | (indirect) |
| stable_deref_trait-1.2.1 | 28 |  | 202 | (indirect) |
| sys-locale-0.3.2 | 7 |  | 777 | boa-engine |
| temporal_rs-0.2.6 | 3 | yes | 23455 | boa-engine |
| time-0.3.55 | 272 |  | 33184 | boa-engine |
| time-core-0.1.9 | 0 |  | 515 | (indirect) |
| timezone_provider-0.2.6 | 0 |  | 5015 | boa-engine |
| tinystr-0.8.5 | 22 |  | 2366 | boa-engine |
| yoke-0.8.4 | 126 |  | 3502 | boa-engine |
| zerofrom-0.1.8 | 0 |  | 316 | boa-engine |
| zerotrie-0.2.5 | 9 |  | 6797 | (indirect) |
| zerovec-0.11.9 | 249 |  | 15966 | (indirect) |

## Tier 2: reached by page script — Text segmentation

Breaks page text into lines and words (Thai, Lao, Khmer, Myanmar dictionaries).

| Crate | `unsafe` | forbid | Lines | Used directly by |
|-------|---------:|:------:|------:|------------------|
| icu_segmenter-2.3.0 | 2 |  | 10399 | boa-engine, text |

## Tier 3: no attacker input at run time — Compiled-in data

Static tables generated at publish time, and code-generation helpers linked through databake but not called at run time.

| Crate | `unsafe` | forbid | Lines | Used directly by |
|-------|---------:|:------:|------:|------------------|
| icu_calendar_data-2.3.0 | 0 |  | 17 | (indirect) |
| icu_locale_fallback_data-2.3.0 | 0 |  | 17 | (indirect) |
| icu_normalizer_data-2.3.0 | 0 |  | 17 | (indirect) |
| icu_properties_data-2.3.0 | 0 |  | 17 | (indirect) |
| icu_segmenter_data-2.3.0 | 0 |  | 17 | (indirect) |
| proc-macro2-1.0.107 | 6 |  | 6029 | (indirect) |
| quote-1.0.47 | 0 |  | 2702 | (indirect) |
| unicode-ident-1.0.27 | 2 |  | 976 | (indirect) |

## Build time only

`autocfg-1.5.1`, `boa_macros-0.22.0`, `bytemuck_derive-1.12.1`, `cc-1.11.1`, `databake-derive-0.2.2`, `displaydoc-0.2.7`, `dynify-macros-0.1.2`, `equator-macro-0.4.2`, `find-msvc-tools-0.1.15`, `heck-0.5.0`, `num_enum_derive-0.7.6`, `pastey-0.2.3`, `phf_generator-0.14.0`, `phf_macros-0.14.0`, `pin-project-internal-1.1.13`, `proc-macro-crate-3.5.0`, `rustc_version-0.4.1`, `rustversion-1.0.23`, `semver-1.0.28`, `serde_derive-1.0.229`, `shlex-2.0.1`, `strum_macros-0.28.0`, `syn-2.0.119`, `syn-3.0.9`, `synstructure-0.14.0`, `thiserror-impl-2.0.21`, `time-macros-0.2.32`, `toml_datetime-1.1.3+spec-1.1.0`, `toml_edit-0.25.19+spec-1.1.0`, `toml_parser-1.1.5+spec-1.1.0`, `winnow-1.0.5`, `yoke-derive-0.8.4`, `zerofrom-derive-0.1.8`, `zerovec-derive-0.11.7`.

## Not compiled on Linux

`r-efi-6.0.0`, `redox_syscall-0.5.18`, `wasi-0.11.1+wasi-snapshot-preview1`, `windows-link-0.2.1`, `windows-sys-0.52.0`, `windows-targets-0.52.6`, `windows_aarch64_gnullvm-0.52.6`, `windows_aarch64_msvc-0.52.6`, `windows_i686_gnu-0.52.6`, `windows_i686_gnullvm-0.52.6`, `windows_i686_msvc-0.52.6`, `windows_x86_64_gnu-0.52.6`, `windows_x86_64_gnullvm-0.52.6`, `windows_x86_64_msvc-0.52.6`. On Windows the `windows_*` crates are
import libraries for the system DLLs and `windows-sys` is their bindings.
