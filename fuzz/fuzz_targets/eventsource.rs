//! eventsource.rs — fuzzes the EventSource response head check and the text/event-stream parser.
#![no_main]

use southstar_eventsource::{Parser, Post, Shared};

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let Some((&selector, rest)) = data.split_first() else {
        return;
    };
    let split = rest.iter().position(|&b| b == 0).unwrap_or(0);
    let (head, body) = rest.split_at(split);
    let shared = Shared::new(b"https://example.com/events".to_vec(), Some(b"https://example.com".to_vec()), b"".to_vec());
    let mut parser = Parser::new();
    let mut posts = 0usize;
    let mut post = |p: Post| {
        posts += 1;
        std::hint::black_box(matches!(p, Post::Open));
    };
    for line in head.split(|&b| b == b'\n') {
        parser.header(line, &mut post);
    }
    for piece in body.chunks(usize::from(selector).max(1)) {
        parser.feed(piece, &shared, &mut post);
    }
    std::hint::black_box((posts, shared.last_event_id()));
});
