//! decompress.rs — fuzzes the HTTP content decoders: gzip/zlib/raw deflate through southstar-http, brotli and zstd as southstar-http drives them.
#![no_main]

use brotli_decompressor::{BrotliDecompressStream, BrotliResult, BrotliState, StandardAlloc};
use ruzstd::decoding::FrameDecoder;
use southstar_http::{Inflate, Inflated};

const OUT_CHUNK: usize = 16_384;
const LIMIT: usize = 8 << 20;

fn inflate(window_bits: i32, data: &[u8], step: usize) {
    let Some(mut inflate) = Inflate::new(window_bits) else {
        return;
    };
    let mut out = vec![0u8; OUT_CHUNK];
    let mut total = 0;
    for piece in data.chunks(step) {
        let mut input = piece;
        loop {
            let (status, consumed, produced) = inflate.step(input, &mut out, false);
            total += produced;
            input = &input[consumed.min(input.len())..];
            if matches!(status, Inflated::Failed | Inflated::StreamEnd) || total > LIMIT {
                return;
            }
            if consumed == 0 && produced == 0 {
                break;
            }
        }
    }
    std::hint::black_box(inflate.step(&[], &mut out, true));
}

fn brotli(data: &[u8], step: usize) {
    let mut state = BrotliState::new(StandardAlloc::default(), StandardAlloc::default(), StandardAlloc::default());
    let mut out = [0u8; OUT_CHUNK];
    let mut total = 0;
    for input in data.chunks(step) {
        let mut available_in = input.len();
        let mut input_offset = 0;
        loop {
            let mut available_out = OUT_CHUNK;
            let mut output_offset = 0;
            let mut total_out = 0;
            let result = BrotliDecompressStream(
                &mut available_in,
                &mut input_offset,
                input,
                &mut available_out,
                &mut output_offset,
                &mut out,
                &mut total_out,
                &mut state,
            );
            total += output_offset;
            match result {
                BrotliResult::NeedsMoreOutput if total <= LIMIT => {}
                BrotliResult::NeedsMoreInput => break,
                _ => return,
            }
        }
    }
}

fn zstd(data: &[u8], step: usize) {
    let mut decoder = FrameDecoder::new();
    decoder.set_max_window_size(1 << 23);
    let mut pending = Vec::new();
    let mut in_frame = false;
    let mut out = [0u8; OUT_CHUNK];
    let mut total = 0;
    for piece in data.chunks(step) {
        pending.extend_from_slice(piece);
        loop {
            if !in_frame {
                if pending.is_empty() {
                    break;
                }
                let mut header: &[u8] = &pending;
                match decoder.reset(&mut header) {
                    Ok(()) => {
                        let used = pending.len() - header.len();
                        pending.drain(..used);
                        in_frame = true;
                    }
                    Err(_) if pending.len() < 18 => break,
                    Err(_) => return,
                }
            }
            let Ok((read, written)) = decoder.decode_from_to(&pending, &mut out) else {
                return;
            };
            total += written;
            if read > pending.len() {
                break;
            }
            pending.drain(..read);
            if total > LIMIT {
                return;
            }
            if decoder.is_finished() && decoder.can_collect() == 0 {
                in_frame = false;
                continue;
            }
            if read == 0 && written == 0 {
                break;
            }
        }
    }
}

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let Some((&selector, rest)) = data.split_first() else {
        return;
    };
    let step = usize::from(selector >> 2).max(1) * 97;
    match selector & 3 {
        0 => inflate(15 + 32, rest, step),
        1 => inflate(-15, rest, step),
        2 => brotli(rest, step),
        _ => zstd(rest, step),
    }
});
