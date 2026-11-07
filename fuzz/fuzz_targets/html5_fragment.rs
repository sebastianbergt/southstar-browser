//! html5_fragment.rs — fuzzes HTML fragment parsing with the context element picked from the input.
#![no_main]

#[path = "common/sink.rs"]
mod sink;

use southstar_html5::{Context, Ns};

const CONTEXTS: &[(Ns, &[u8])] = &[
    (Ns::Html, b"body"),
    (Ns::Html, b"table"),
    (Ns::Html, b"tr"),
    (Ns::Html, b"select"),
    (Ns::Html, b"template"),
    (Ns::Html, b"textarea"),
    (Ns::Html, b"script"),
    (Ns::Html, b"style"),
    (Ns::Html, b"title"),
    (Ns::Html, b"plaintext"),
    (Ns::Html, b"noscript"),
    (Ns::Html, b"html"),
    (Ns::Svg, b"svg"),
    (Ns::Svg, b"foreignObject"),
    (Ns::MathMl, b"math"),
    (Ns::MathMl, b"annotation-xml"),
];

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let Some((&selector, input)) = data.split_first() else {
        return;
    };
    let (ns, name) = CONTEXTS[usize::from(selector) % CONTEXTS.len()];
    let mut sink = sink::Sink::new();
    southstar_html5::parse_fragment(Context { ns, name }, input, selector & 0x80 != 0, &mut sink);
});
