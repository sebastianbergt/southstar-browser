//! text_formats.rs — fuzzes WebVTT cue parsing, Content-Security-Policy headers and MIME sniffing.
#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    std::hint::black_box(southstar_webvtt::parse(data).len());
    std::hint::black_box(southstar_csp::Csp::parse(data).is_some());
    std::hint::black_box(southstar_mime::guess(b"file", Some(data)));
    std::hint::black_box(southstar_mime::guess(data, None));
});
