//! http_hpack.rs — fuzzes the HPACK decoder over a sequence of header blocks and the Huffman decoder.
#![no_main]

use southstar_http_wire::hpack;

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    std::hint::black_box(hpack::huffman_decode(data).ok());
    let mut decoder = hpack::Decoder::new(4096);
    let mut rest = data;
    while let Some((&len, tail)) = rest.split_first() {
        let len = usize::from(len).min(tail.len());
        let (block, next) = tail.split_at(len);
        if decoder.decode(block).is_err() {
            break;
        }
        rest = next;
    }
});
