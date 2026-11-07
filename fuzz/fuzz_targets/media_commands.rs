//! media_commands.rs — fuzzes the shell's checks on a renderer's X-Audio and video commands, and checks that the URL the shell vets is the URL the audio and video helpers would open.
#![no_main]

#[allow(dead_code)]
#[path = "../../rust/shell/src/procview/media.rs"]
mod media;

fn audio_helper_open(line: &[u8]) -> Option<&[u8]> {
    let mut rest = line;
    let mut next = || {
        let start = rest.iter().position(|&c| c != b' ' && c != b'\t')?;
        let s = &rest[start..];
        let end = s.iter().position(|&c| c == b' ' || c == b'\t');
        let (token, tail) = match end {
            Some(end) => (&s[..end], &s[end + 1..]),
            None => (s, &s[s.len()..]),
        };
        rest = tail;
        Some(token)
    };
    let op = next()?;
    next()?;
    if op != b"open" {
        return None;
    }
    let start = rest.iter().position(|&c| c != b' ').unwrap_or(rest.len());
    Some(&rest[start..])
}

fn video_helper_open(line: &[u8]) -> Option<&[u8]> {
    let blank = |b: u8| b == b' ' || b == b'\t';
    let mut rest = line;
    let mut next = || {
        let start = rest.iter().position(|&b| !blank(b))?;
        let tail = &rest[start..];
        let end = tail.iter().position(|&b| blank(b)).unwrap_or(tail.len());
        rest = tail.get(end + 1..).unwrap_or_default();
        Some(&tail[..end])
    };
    let op = next()?;
    next()?;
    if op != b"open" && op != b"reload" {
        return None;
    }
    let start = rest.iter().position(|&b| b != b' ').unwrap_or(rest.len());
    Some(&rest[start..])
}

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    for line in data.split(|&b| b == b'\n') {
        std::hint::black_box(media::chomp(line));
        if !media::line_ok(line) {
            continue;
        }
        let vetted = media::open_url(line);
        if let Some(url) = audio_helper_open(line) {
            assert_eq!(vetted, Some(url), "audio helper opens a URL the shell did not vet");
        }
        if let Some(url) = video_helper_open(line) {
            assert_eq!(vetted, Some(url), "video helper opens a URL the shell did not vet");
        }
        if let Some(url) = vetted {
            if let Some((dir, name)) = media::stream_file_parts(url) {
                assert!(!dir.is_empty() && !name.contains(&b'/') && !name.contains(&b'\\') && name.ends_with(b".dat"));
                assert!(!name.windows(2).any(|w| w == b".."), "stream file name with ..");
            }
        }
        std::hint::black_box(media::parse_rect(line).map(|r| r.fields));
        std::hint::black_box(matches!(media::video_command(line), media::VideoCommand::Other));
        std::hint::black_box(media::truncated(line).len());
        let _ = media::helper_line(line);
        std::hint::black_box(media::scan_token(line));
    }
});
