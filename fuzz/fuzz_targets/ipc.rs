//! ipc.rs — fuzzes the renderer protocol as the shell reads it from a renderer: HTTP/1.1 heads with the X-* headers and bodies through a real pipe, and the flat JSON helpers.
#![no_main]

use std::io::Write;
use std::os::fd::AsRawFd;

use southstar_ipc::{Conn, Head, json_double, json_string, json_value};

const KEYS: &[&[u8]] = &[b"url", b"title", b"x", b"y", b"width", b"height", b"text", b"value", b"kind", b"id", b"scale", b"tiles", b"gen"];

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    if data.len() > 60_000 {
        return;
    }
    let Ok((reader, mut writer)) = std::io::pipe() else {
        return;
    };
    if writer.write_all(data).is_err() {
        return;
    }
    drop(writer);
    let mut conn = Conn::boxed(reader.as_raw_fd());
    let mut head = Head::boxed();
    for _ in 0..4 {
        if !conn.read_head(&mut head) {
            break;
        }
        std::hint::black_box((head.status, head.x_w, head.x_h, head.x_stride, head.x_tiles));
        let n = head.content_length;
        if n <= 0 || n > 1 << 20 {
            continue;
        }
        let mut body = vec![0u8; n as usize];
        if !conn.read_body(&mut body) {
            break;
        }
        for key in KEYS {
            if let Some(value) = json_value(&body, key) {
                std::hint::black_box(json_double(value));
                std::hint::black_box(json_string(value));
            }
        }
    }
    drop(conn);
    drop(reader);
});
