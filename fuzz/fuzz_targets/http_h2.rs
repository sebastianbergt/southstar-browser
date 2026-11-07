//! http_h2.rs — fuzzes the sans-I/O HTTP/2 client connection with server bytes after one request.
#![no_main]

use southstar_http_wire::h2::Connection;

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let mut connection = Connection::new();
    connection.submit(
        &[
            (b":method", b"POST"),
            (b":scheme", b"https"),
            (b":authority", b"example.com"),
            (b":path", b"/"),
        ],
        vec![0; 70_000],
    );
    std::hint::black_box(connection.take_output());
    let step = usize::from(data.first().copied().unwrap_or(1).max(1)) * 7;
    for piece in data.chunks(step) {
        if connection.receive(piece).is_err() {
            break;
        }
        while let Some(event) = connection.next_event() {
            std::hint::black_box(event);
        }
        std::hint::black_box(connection.take_output());
        if connection.can_open() && connection.active() < 4 {
            connection.submit(&[(b":method", b"GET"), (b":path", b"/x")], Vec::new());
        }
    }
});
