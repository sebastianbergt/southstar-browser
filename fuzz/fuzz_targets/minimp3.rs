//! minimp3.rs — fuzzes the MP3 frame decoder over a stream of frames.
#![no_main]

use southstar_minimp3::{Decoder, MAX_SAMPLES_PER_FRAME};

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let mut decoder = Decoder::default();
    let mut pcm = [0f32; MAX_SAMPLES_PER_FRAME];
    let mut rest = data;
    for _ in 0..256 {
        let (samples, info) = decoder.decode_frame(rest, Some(&mut pcm));
        std::hint::black_box(samples);
        if info.frame_bytes == 0 || info.frame_bytes > rest.len() {
            break;
        }
        rest = &rest[info.frame_bytes..];
    }
});
