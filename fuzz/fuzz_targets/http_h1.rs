//! http_h1.rs — fuzzes HTTP/1.1 status lines, header lines, chunk sizes and the chunked body decoder.
#![no_main]

use southstar_http_wire::h1;

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    for line in data.split(|&b| b == b'\n') {
        std::hint::black_box(h1::status_code(line));
        std::hint::black_box(h1::split_header(line));
        std::hint::black_box(h1::chunk_size(line));
        std::hint::black_box(h1::strtoll(line, 10));
        std::hint::black_box(h1::strtoll(line, 16));
    }
    let mut chunked = h1::Chunked::new();
    let mut out = Vec::new();
    let step = usize::from(data.first().copied().unwrap_or(1).max(1));
    for piece in data.chunks(step) {
        if chunked.feed(piece, &mut out).is_err() || chunked.done() {
            break;
        }
    }
});
