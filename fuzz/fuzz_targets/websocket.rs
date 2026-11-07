//! websocket.rs — fuzzes the WebSocket frame decoder with server bytes delivered in varying chunks.
#![no_main]

use southstar_websocket::{Decoded, Decoder};

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let Some((&selector, rest)) = data.split_first() else {
        return;
    };
    let mut decoder = Decoder::default();
    let step = usize::from(selector).max(1);
    let mut frames = 0;
    for piece in rest.chunks(step) {
        decoder.push(piece);
        loop {
            match decoder.next_frame() {
                Decoded::Frame(frame) => {
                    frames += 1;
                    std::hint::black_box((frame.opcode, frame.payload.len()));
                    assert!(frames <= rest.len() + 1, "decoder produced more frames than input bytes");
                }
                Decoded::NeedMore => break,
                Decoded::TooBig | Decoded::ProtocolError => return,
            }
        }
    }
});
