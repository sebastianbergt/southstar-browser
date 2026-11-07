//! js_builtins.rs — fuzzes the engine's string parsers directly: JSON.parse, RegExp patterns and flags, Date, Temporal and Intl, with the input as a string.
#![no_main]

use boa_engine::{Context, JsString, Source, js_string, property::Attribute};

const PROBES: &str = r#"
const flags = ['', 'u', 'v', 'gimsy', 'd'][selector % 5];
const attempts = [
  () => JSON.parse(input, (k, v) => v),
  () => { const r = new RegExp(input, flags); return [r.test(subject), subject.replace(r, '$1$<n>$&'), subject.split(r).length, r.source]; },
  () => new RegExp(subject, input),
  () => [Date.parse(input), new Date(input).toISOString()],
  () => Temporal.PlainDate.from(input).toString(),
  () => Temporal.PlainDateTime.from(input).toString(),
  () => Temporal.ZonedDateTime.from(input).toString(),
  () => Temporal.Duration.from(input).total({ unit: 'second', relativeTo: '2020-01-01' }),
  () => Temporal.Instant.from(input).toString(),
  () => new Intl.Locale(input).maximize().toString(),
  () => new Intl.DateTimeFormat(input, { timeZone: 'UTC' }).format(0),
  () => new Intl.NumberFormat(input).format(1234.5),
  () => input.normalize(['NFC', 'NFD', 'NFKC', 'NFKD'][selector % 4]),
  () => input.localeCompare(subject),
  () => [encodeURIComponent(input), decodeURIComponent(input), escape(input), unescape(input)],
  () => parseFloat(input) + parseInt(input, selector % 37),
  () => BigInt(input),
  () => String.raw({ raw: [input, subject] }, 1),
];
for (const attempt of attempts) {
  try { attempt(); } catch (e) {}
}
"#;

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let Some((&selector, rest)) = data.split_first() else {
        return;
    };
    let text = String::from_utf8_lossy(rest);
    let (input, subject) = text.split_once('\u{0}').unwrap_or((&text, "aaa-bbb 2020-01-01"));
    let mut context = Context::default();
    let limits = context.runtime_limits_mut();
    limits.set_loop_iteration_limit(100_000);
    limits.set_recursion_limit(256);
    limits.set_stack_size_limit(64 * 1024);
    let _ = context.register_global_property(js_string!("input"), JsString::from(input), Attribute::all());
    let _ = context.register_global_property(js_string!("subject"), JsString::from(subject), Attribute::all());
    let _ = context.register_global_property(js_string!("selector"), i32::from(selector), Attribute::all());
    std::hint::black_box(context.eval(Source::from_bytes(PROBES)).is_ok());
});
