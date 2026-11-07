# zstd responses can make the HTTP decoder loop forever

- **Found by:** `decompress` fuzz target, 2026-11-07 (8 timeouts, inputs of 11–192
  bytes, all zstd).
- **Component:** `rust/http/src/zstd.rs` (`Zstd::feed`) together with
  `ruzstd` 0.9.1, `FrameDecoder::decode_from_to`
  (`src/decoding/frame_decoder.rs`, lines 465–477 of the published crate).
- **Impact:** a thread decoding an HTTP response body spins at 100% CPU forever
  and the response never completes. It happens with **well-formed** zstd
  streams, not only malicious ones, whenever a network read ends inside the
  frame's 4-byte content checksum or right before it.
- **Severity:** high for availability. zstd is enabled in every build
  (`rust/xtask/src/detect.rs:159`) and advertised in every request
  (`Accept-Encoding: gzip, deflate, br, zstd`, `rust/http/src/decode.rs:33`), and
  `zstd` writes a content checksum by default. Any server, or anyone who can
  answer a plain-HTTP request, can trigger it, and a page can open many such
  requests to pin many threads. No memory-safety issue: memory use stays flat.

## Reproducer

A zstd frame whose last block is complete but whose checksum is cut short; this
10-byte body (the minimized fuzz input without its selector byte) hangs the
decoder:

```
28 b5 2f fd 04 60 33 8a 00 09
```

A well-formed frame hangs too when split. A 56-byte frame of 3,322 bytes of HTML
(`zstd` level 3 with checksum) was fed in two chunks with the loop from
`Zstd::feed`, release build:

| First chunk ends at | Result |
|--------------------:|--------|
| 56 / 56 (whole frame) | decoded, 1 call |
| 55 / 56 | still looping after 2 s, about 80 million calls |
| 54 / 56 | still looping after 2 s |
| 53 / 56 | still looping after 2 s |
| 52 / 56 (just before the checksum) | still looping after 2 s |
| 51 / 56 | decoded, 2 calls |

All 3,322 bytes of content were emitted before the loop started, so the page
receives its data but the response never finishes.

## Cause

When the last block of a frame has been decoded, the frame declares a content
checksum and the checksum has not been read yet, `decode_from_to` runs:

```rust
if mt_source.len() >= 4 {
    /* store the checksum, count 4 bytes read */
}
return Ok((4, 0));
```

It returns "4 bytes read" even when fewer than 4 were available and nothing was
consumed, and `is_finished()` stays false because no checksum was stored.

`Zstd::feed` then drains `read.min(pending.len())` bytes, which discards the 0–3
checksum bytes that did arrive, and loops. It only stops on
`read == 0 && written == 0` or a finished frame, neither of which can happen
again, so `feed` never returns, not even to wait for the rest of the body.

## Suggested fix

- In `Zstd::feed`, treat a call that writes nothing and cannot have consumed
  `read` bytes (`read > pending.len()`, or a repeated `(read, 0)` with no change
  in `pending`) as "need more input": return `Ok(())` without draining, and let
  the next network chunk supply the checksum. At end of body, a frame that is
  not finished is `Failure::Corrupt`.
- Report the bug upstream to `ruzstd`: `decode_from_to` should return `(0, 0)`
  when it cannot read the checksum.
- Bound every decoder loop in `rust/http` by progress, not by the decoder's own
  counters, so a misbehaving third-party decoder cannot hang a thread.

## Verify

Rerun the `decompress` target (the 8 inputs are in `fuzz/artifacts/decompress/`
after a run) and feed a checksummed zstd body split one byte before its end;
decoding should finish and report the content.
