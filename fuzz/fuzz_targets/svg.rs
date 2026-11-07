//! svg.rs — fuzzes SVG images as pages load them: parsing, styling and rendering into a texture.
#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    if !southstar_svg::looks_like_svg(data) {
        return;
    }
    std::hint::black_box(southstar_svg::decode_bytes(data).map(|(_, w, h)| (w, h)));
});
