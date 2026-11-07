//! js_eval.rs — fuzzes the in-tree Boa fork by parsing and running scripts under runtime limits.
#![no_main]

use boa_engine::{Context, Source};

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let mut context = Context::default();
    let limits = context.runtime_limits_mut();
    limits.set_loop_iteration_limit(100_000);
    limits.set_recursion_limit(256);
    limits.set_stack_size_limit(64 * 1024);
    std::hint::black_box(context.eval(Source::from_bytes(data)).is_ok());
    std::hint::black_box(context.run_jobs().is_ok());
});
