//! html5_document.rs — fuzzes the HTML tokenizer and tree builder with a whole document.
#![no_main]

#[path = "common/sink.rs"]
mod sink;

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let scripting = data.first().is_some_and(|b| b & 1 == 1);
    let mut sink = sink::Sink::new();
    southstar_html5::parse_document(data, scripting, &mut sink);
});
