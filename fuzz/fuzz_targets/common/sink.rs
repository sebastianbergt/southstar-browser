//! sink.rs — a tree sink for the HTML fuzz targets that records parents only.

use southstar_html5::{Attrs, ElementName, Position, TreeSink};

pub struct Sink {
    parents: Vec<Option<usize>>,
}

impl Sink {
    pub fn new() -> Sink {
        Sink {
            parents: vec![None],
        }
    }

    fn node(&mut self) -> usize {
        self.parents.push(None);
        self.parents.len() - 1
    }
}

impl TreeSink for Sink {
    type Handle = usize;

    fn document(&self) -> usize {
        0
    }

    fn create_element(&mut self, _name: ElementName<'_>, attrs: Attrs<'_>, _position: Position) -> usize {
        std::hint::black_box(attrs.count());
        self.node()
    }

    fn create_comment(&mut self, text: &[u8]) -> usize {
        std::hint::black_box(text.len());
        self.node()
    }

    fn create_processing_instruction(&mut self, _target: &[u8], _data: &[u8]) -> usize {
        self.node()
    }

    fn append_doctype(&mut self, _name: &[u8], _public_id: &[u8], _system_id: &[u8]) {}

    fn append(&mut self, parent: usize, child: usize) {
        self.parents[child] = Some(parent);
    }

    fn insert_before(&mut self, sibling: usize, child: usize) {
        self.parents[child] = self.parents[sibling];
    }

    fn append_text(&mut self, _parent: usize, text: &[u8]) {
        std::hint::black_box(text.len());
    }

    fn insert_text_before(&mut self, _sibling: usize, _text: &[u8]) {}

    fn parent(&self, node: usize) -> Option<usize> {
        self.parents[node]
    }

    fn remove(&mut self, node: usize) {
        self.parents[node] = None;
    }

    fn reparent_children(&mut self, from: usize, to: usize) {
        for parent in self.parents.iter_mut().filter(|p| **p == Some(from)) {
            *parent = Some(to);
        }
    }

    fn template_contents(&mut self, _template: usize) -> usize {
        self.node()
    }

    fn add_missing_attributes(&mut self, _element: usize, attrs: Attrs<'_>) {
        std::hint::black_box(attrs.count());
    }
}
