//! xml.rs — fuzzes the XML parser that builds DOM trees for XHTML and SVG documents.
#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    std::hint::black_box(southstar_xml::parse_document(data).is_ok());
});
