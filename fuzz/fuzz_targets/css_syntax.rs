//! css_syntax.rs — fuzzes the CSS tokenizer and component-value parser.
#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    std::hint::black_box(southstar_css_syntax::parse(data));
    std::hint::black_box(southstar_css_syntax::value_valid(data));
    std::hint::black_box(southstar_css_syntax::scan(data, b";}"));
    std::hint::black_box(southstar_css_syntax::is_self_contained(data));
});
