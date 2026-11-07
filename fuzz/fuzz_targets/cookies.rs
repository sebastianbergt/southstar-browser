//! cookies.rs — fuzzes Set-Cookie parsing, the cookie jar's text format, Cookie header building, HSTS policies and lists, and HTTP dates.
#![no_main]

use southstar_netstore::{cookies, hsts, http_date};
use southstar_url::bytes;

const TARGETS: &[&[u8]] = &[
    b"https://example.com/a/b",
    b"http://sub.example.co.uk/",
    b"https://[::1]:8443/x?y",
    b"https://xn--bcher-kva.example/",
];

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let Some((&selector, rest)) = data.split_first() else {
        return;
    };
    let target = TARGETS[usize::from(selector) % TARGETS.len()];
    let now = 1_800_000_000;
    let public_suffix = |host: &[u8]| matches!(host, b"com" | b"co.uk" | b"uk" | b"example");
    let mut jar: Option<Vec<u8>> = (selector & 0x80 != 0).then(|| rest.to_vec());
    for line in rest.split(|&b| b == b'\n').take(16) {
        if let Some(cookie) = cookies::parse(target, line, now, public_suffix) {
            std::hint::black_box(cookies::has_live_httponly(jar.as_deref().unwrap_or(b""), &cookie));
            std::hint::black_box(cookies::settable_in(&cookie, cookies::Context::CrossSite));
            jar = Some(cookies::store(jar.as_deref(), &cookie, selector & 1 == 0));
        }
        let policy = hsts::parse_policy(line);
        std::hint::black_box((policy.max_age, policy.subdomains));
        std::hint::black_box(http_date::parse(line));
        std::hint::black_box(http_date::parse_iso8601(line));
        std::hint::black_box(hsts::parse_expiry(line));
    }
    if let (Some(jar), Some(parts)) = (jar.as_deref(), bytes::parts(target)) {
        for context in [cookies::Context::SameSite, cookies::Context::TopLevelSafeNavigation, cookies::Context::CrossSite] {
            std::hint::black_box(cookies::header(&parts, [jar], selector & 2 == 0, context, now));
        }
    }
    let hosts = hsts::parse_list(rest);
    std::hint::black_box(hsts::lookup(&hosts, b"sub.example.com", now));
});
