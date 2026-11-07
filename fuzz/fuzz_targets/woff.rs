//! woff.rs — fuzzes the WOFF and WOFF2 web font decoders.
#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    std::hint::black_box(southstar_woff2::to_sfnt(data).is_some());
    std::hint::black_box(southstar_woff2::woff_to_sfnt(data).is_some());
});
