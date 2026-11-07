# External scripts reach the JS lexer undecoded, and invalid UTF-8 swallows the bytes after it

- **Found by:** `js_eval` fuzz target, round 2, 2026-11-07. It found a `debug_assert!` in
  `regress` (`codepointset.rs:143`, `iv.last <= CODE_POINT_MAX`) on the 8-byte script
  `/[\xFF\xFF\xFF ]/`. Following that assertion led to the two causes below.
- **Components:**
  - `rust/js-loader/src/fetch.rs:177`: an external script's response body is passed to
    `eval_script_source` as raw bytes, then to `Scope::compile_script` and
    `Source::from_bytes`. No charset decoding or UTF-8 validation happens on the way.
  - `third_party/boa_parser/src/source/utf8.rs` (`UTF8Input::next_char`) decodes UTF-8
    without checking it. A lead byte takes the next 1–3 bytes as continuation bytes,
    whatever they are, and lead bytes `0xF8`–`0xFF` produce values up to 0x1FFFFF.
- **Impact:** a parser differential with security consequences. Text that every other
  browser treats as inert string or comment content can become live code in Southstar,
  and code can silently disappear. A site that embeds user-controlled bytes inside a
  JavaScript string literal in a generated external script (JSON-P, configuration
  scripts, templated JS) and escapes quotes with a backslash can be attacked with
  cross-site scripting in Southstar alone. Legacy scripts in Latin-1 or windows-1252
  break. Debug builds abort on the fuzz input.
- **Scope:** HTML documents are always made valid UTF-8 (`html_util::decode_body`), so
  inline `<script>` elements are not affected. External classic scripts and module
  scripts fetched over the network are.
- **Severity:** medium.

## Reproducers

These were evaluated with Boa from this tree in a release build (no debug assertions).
"Correct" is the same script with each invalid byte replaced by U+FFFD, as the Encoding
Standard's UTF-8 decoder and every other browser do.

| Script (bytes) | Southstar | Correct decoding |
|---|---|---|
| `var X = 0; var name = '\xC0\'; X = 1; //'; X` | `1`: the lead byte `C0` takes the escaping backslash, the quote ends the string and `X = 1` runs | `0` |
| `var X = 0; var s = '\xE0'+'; X = 1; //'; X` | `1` | `0` |
| `var Y = 0; // \xF0` newline `Y = 1; Y` | `undefined`: `F0` takes the newline, so `Y = 1` is part of the comment | `1` |
| `var s = 'caf\xE9'; s.length` (Latin-1 "café") | `SyntaxError: unterminated string literal` | `4` |
| `/[\xFF\xFF\xFF ]/` (the fuzz input) | a character class holding a code point above U+10FFFF; debug builds abort on `regress`'s `debug_assert!` | a class holding U+FFFD |

The first row is the classic multi-byte escape-swallowing attack. If a server emits
`var name = '<user input with ' escaped as \'>';` into a script, and the user's input is
the byte `0xC0` followed by `'; payload(); //`, the output is
`var name = '\xC0\'; payload(); //';`. Every other browser sees one string. Southstar runs
`payload()`.

## Cause

The HTML Standard fetches a classic script and decodes it with the encoding from its
`Content-Type` charset, falling back to the `<script charset>` attribute or the
document's encoding, and module scripts always as UTF-8. Decoding replaces invalid
sequences with U+FFFD. Southstar skips that step for external scripts. Boa's UTF-8 reader
was written for trusted, valid input and does no validation, so invalid bytes turn into
arbitrary code points and consume their neighbours.

## Suggested fix

1. Decode external and module script bodies before compiling them. For classic scripts,
   use the `Content-Type` charset, else the `charset` attribute, else the document's
   encoding. Module scripts are always UTF-8. In every case, replace invalid sequences
   with U+FFFD; `html_util::decode_body` and `utf8_make_valid` already do this for
   documents.
2. Defence in depth: make `UTF8Input::next_char` in the Boa fork validate. On an invalid
   lead byte, a missing or invalid continuation byte, an overlong form, a surrogate, or a
   value above U+10FFFF, it should emit U+FFFD and consume only the bytes that belong to
   the bad sequence, so no caller can feed it bytes that swallow their neighbours.
3. Keep the `js_eval` fuzz target: once either fix lands, the `regress` assertion should
   no longer be reachable.

## Verify

Serve the first row as an external script (`Content-Type: text/javascript`); `X` must
stay `0`. Serve the Latin-1 row with `Content-Type: text/javascript; charset=iso-8859-1`;
`s.length` must be `4`.
