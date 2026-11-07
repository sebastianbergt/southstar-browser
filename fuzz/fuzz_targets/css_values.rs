//! css_values.rs — fuzzes CSS colour parsing, selector serialization, An+B and @property syntax.
#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    std::hint::black_box(southstar_css_color::parse(data));
    std::hint::black_box(southstar_css_prop_syntax::parse(data));
    let text = String::from_utf8_lossy(data);
    std::hint::black_box(southstar_selector_text::serialize_selector_list(&text));
    std::hint::black_box(southstar_selector_text::parse_anb(&text));
    std::hint::black_box(southstar_selector_text::canonical_anb(&text));
});
