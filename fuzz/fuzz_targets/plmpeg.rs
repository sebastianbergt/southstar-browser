//! plmpeg.rs — fuzzes the MPEG-1 program stream demuxer and the MPEG-1 video and MP2 audio decoders.
#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let mut plm = southstar_plmpeg::Plm::new(data.to_vec());
    if !plm.has_headers() {
        return;
    }
    std::hint::black_box((plm.width(), plm.height(), plm.samplerate(), plm.duration()));
    for _ in 0..32 {
        let video = plm.decode_video().is_some();
        let audio = plm.decode_audio().is_some();
        if !video && !audio {
            break;
        }
    }
});
