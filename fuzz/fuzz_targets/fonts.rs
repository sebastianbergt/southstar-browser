//! fonts.rs — fuzzes the pure-Rust web font path as rust/text uses it: read-fonts acceptance, skrifa outlines (unhinted and hinted) and color glyphs, and harfrust shaping.
#![no_main]

use harfrust::font::Blob;
use harfrust::{Buffer, ClusterLevel, Direction, GlyphId as ShapeGlyphId, ShapeOptions, ShapePlan, ShaperFont};
use read_fonts::{FileRef, TableProvider};
use skrifa::color::{Brush, ColorPainter, CompositeMode, Transform};
use skrifa::instance::{LocationRef, Size};
use skrifa::raw::types::BoundingBox;
use skrifa::outline::{DrawSettings, HintingInstance, HintingOptions, OutlinePen};
use skrifa::{FontRef, GlyphId, MetadataProvider};

const SAMPLES: &[(&str, Direction)] = &[
    ("Hello, fuzzing ffi fl Å é", Direction::LeftToRight),
    ("مرحبا بالعالم لا", Direction::RightToLeft),
    ("नमस्ते दुनिया क्षि", Direction::LeftToRight),
    ("สวัสดีชาวโลก", Direction::LeftToRight),
    ("👩‍👩‍👧 🏳️‍🌈 a\u{301}", Direction::LeftToRight),
];

#[derive(Default)]
struct Pen(usize);

impl OutlinePen for Pen {
    fn move_to(&mut self, _x: f32, _y: f32) {
        self.0 += 1;
    }
    fn line_to(&mut self, _x: f32, _y: f32) {
        self.0 += 1;
    }
    fn quad_to(&mut self, _cx0: f32, _cy0: f32, _x: f32, _y: f32) {
        self.0 += 1;
    }
    fn curve_to(&mut self, _cx0: f32, _cy0: f32, _cx1: f32, _cy1: f32, _x: f32, _y: f32) {
        self.0 += 1;
    }
    fn close(&mut self) {}
}

#[derive(Default)]
struct Painter(usize);

impl ColorPainter for Painter {
    fn push_transform(&mut self, _transform: Transform) {
        self.0 += 1;
    }
    fn pop_transform(&mut self) {}
    fn push_clip_glyph(&mut self, _glyph_id: GlyphId) {
        self.0 += 1;
    }
    fn push_clip_box(&mut self, _clip_box: BoundingBox<f32>) {}
    fn pop_clip(&mut self) {}
    fn fill(&mut self, brush: Brush<'_>) {
        std::hint::black_box(matches!(brush, Brush::Solid { .. }));
    }
    fn push_layer(&mut self, _composite_mode: CompositeMode) {}
}

fn accepted(data: &[u8]) -> bool {
    let Ok(file) = FileRef::new(data) else {
        return false;
    };
    let mut fonts = file.fonts().peekable();
    fonts.peek().is_some()
        && fonts.all(|font| {
            font.is_ok_and(|font| {
                font.head().is_ok_and(|head| (16..=16384).contains(&head.units_per_em())) && font.maxp().is_ok() && font.cmap().is_ok() && font.hhea().is_ok() && font.hmtx().is_ok()
            })
        })
}

fn glyphs(font: &FontRef<'_>) {
    let outlines = font.outline_glyphs();
    let hinters: Vec<HintingInstance> = [16.0, 2048.0]
        .into_iter()
        .filter_map(|ppem| HintingInstance::new(&outlines, Size::new(ppem), LocationRef::default(), HintingOptions::default()).ok())
        .collect();
    let charmap = font.charmap();
    let mut ids: Vec<GlyphId> = (0..24u32).map(GlyphId::new).collect();
    ids.extend(SAMPLES.iter().flat_map(|(text, _)| text.chars()).filter_map(|c| charmap.map(c)));
    ids.truncate(64);
    for gid in ids {
        if let Some(glyph) = outlines.get(gid) {
            let mut pen = Pen::default();
            let _ = glyph.draw(DrawSettings::unhinted(Size::new(16.0), LocationRef::default()), &mut pen);
            for hinter in &hinters {
                let _ = glyph.draw(DrawSettings::hinted(hinter, false), &mut pen);
            }
            std::hint::black_box(pen.0);
        }
        if let Some(color) = font.color_glyphs().get(gid) {
            let mut painter = Painter::default();
            let _ = color.paint(LocationRef::default(), &mut painter);
            std::hint::black_box(painter.0);
        }
        std::hint::black_box(font.glyph_metrics(Size::new(16.0), LocationRef::default()).advance_width(gid));
    }
}

fn shape(data: &[u8]) {
    let Some(face) = harfrust::Font::new(Blob::from(data.to_vec()), 0) else {
        return;
    };
    for &(text, direction) in SAMPLES {
        let plan = ShapePlan::new(&face, direction, None, None, &[]);
        let shaper = ShaperFont::new(&face);
        let mut buffer = Buffer::new();
        buffer.set_direction(direction);
        buffer.set_cluster_level(ClusterLevel::MonotoneCharacters);
        buffer.set_invisible_glyph(Some(ShapeGlyphId::new(0)));
        for (offset, ch) in text.char_indices() {
            buffer.push(u32::from(ch), offset as u32);
        }
        let options = ShapeOptions::new().plan(Some(&plan)).point_size(Some(16.0));
        std::hint::black_box(harfrust::shape(&shaper, &mut buffer, options).is_ok());
    }
}

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    if !accepted(data) {
        return;
    }
    if let Ok(font) = FontRef::new(data) {
        glyphs(&font);
    }
    shape(data);
});
