//! url.rs — fuzzes the WHATWG URL parser, resolution, setters, origins and IDNA, and checks that serialization round-trips.
#![no_main]

use southstar_url::bytes;

const COMPONENTS: &[&[u8]] = &[
    b"href", b"protocol", b"username", b"password", b"host", b"hostname", b"port", b"pathname",
    b"search", b"hash",
];

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let Some((&selector, rest)) = data.split_first() else {
        return;
    };
    let split = rest.iter().position(|&b| b == 0).unwrap_or(rest.len());
    let (first, second) = (&rest[..split], rest.get(split + 1..).unwrap_or(&[]));
    if let Some(href) = bytes::resolve(None, first) {
        assert_eq!(
            bytes::resolve(None, &href).as_deref(),
            Some(href.as_slice()),
            "reparsing a serialized URL changed it"
        );
        std::hint::black_box(bytes::origin_from(&href));
        std::hint::black_box(bytes::host_from(&href));
        std::hint::black_box(bytes::parts(&href).is_some());
        std::hint::black_box(bytes::to_ascii(&href));
        std::hint::black_box(bytes::resolve(Some(&href), second));
    }
    std::hint::black_box(bytes::origin_from_any(first));
    std::hint::black_box(bytes::same_origin(first, second));
    std::hint::black_box(bytes::blob_url_reachable(first, second));
    std::hint::black_box(bytes::is_valid_absolute(first));
    let component = COMPONENTS[usize::from(selector) % COMPONENTS.len()];
    std::hint::black_box(bytes::set_component(b"https://user:pw@example.com:8080/a/b?c#d", component, first));
    std::hint::black_box(bytes::set_component(first, component, second));
    std::hint::black_box(southstar_url::domain_to_ascii(&String::from_utf8_lossy(first)));
});
