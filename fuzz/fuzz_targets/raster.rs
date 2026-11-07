//! raster.rs — fuzzes the PNG/APNG, GIF, JPEG and BMP decoders behind page images.
#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    std::hint::black_box(southstar_raster::decode(data).is_some());
    std::hint::black_box(southstar_raster::decode_animation(data).map(|frames| frames.len()));
});
