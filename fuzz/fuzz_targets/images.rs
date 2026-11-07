//! images.rs — fuzzes the WebP (still and animated) and ICO decoders and the raster chain behind them.
#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    std::hint::black_box(southstar_image_decoders::decode_webp(data).is_some());
    std::hint::black_box(southstar_image_decoders::decode_webp_animation(data).map(|(frames, _, _)| frames.len()));
    std::hint::black_box(southstar_image_decoders::decode_ico(data).is_some());
    std::hint::black_box(southstar_image_decoders::decode_raster(data).is_some());
});
