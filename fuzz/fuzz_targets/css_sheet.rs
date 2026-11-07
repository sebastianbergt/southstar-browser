//! css_sheet.rs — fuzzes whole style sheet parsing: rules, at-rules, selectors and declarations.
#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    std::hint::black_box(southstar_css::parse_url_cached(b"", data));
});
