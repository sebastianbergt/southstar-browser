//! bidi.rs — fuzzes UAX #9 paragraph resolution with the classes and brackets of the input text.
#![no_main]

use southstar_bidi::ParagraphDirection;

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let Some((&selector, rest)) = data.split_first() else {
        return;
    };
    let text = String::from_utf8_lossy(rest);
    let classes: Vec<_> = text.chars().map(|c| southstar_bidi::class(c as u32)).collect();
    let brackets: Vec<_> = text.chars().map(|c| southstar_bidi::bracket(c as u32)).collect();
    let direction = match selector % 4 {
        0 => ParagraphDirection::Ltr,
        1 => ParagraphDirection::Rtl,
        2 => ParagraphDirection::WeakLtr,
        _ => ParagraphDirection::WeakRtl,
    };
    let mut levels = vec![0u8; classes.len()];
    std::hint::black_box(southstar_bidi::resolve_paragraph(&classes, &brackets, direction, &mut levels));
});
