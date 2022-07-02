use std::cell::Cell;

use crate::{grapheme::Graphemes, register::Register};

/// Store the candidates to choose the items from.
#[derive(Debug, Clone, Default)]
pub struct SelectBox {
    pub data: Vec<Graphemes>,
    pub idx: Cell<usize>,
}

impl<T: Into<String>> Register<T> for SelectBox {
    fn register(&mut self, item: T) {
        self.data.push(Graphemes::from(item.into()))
    }
}

impl SelectBox {
    pub fn pos(&self) -> usize {
        self.idx.get()
    }

    pub fn prev(&self) -> bool {
        if 0 < self.idx.get() {
            self.idx.set(self.idx.get() - 1);
            return true;
        }
        false
    }

    pub fn next(&self) -> bool {
        if !self.data.is_empty() && self.idx.get() < self.data.len() - 1 {
            self.idx.set(self.idx.get() + 1);
            return true;
        }
        false
    }

    pub fn to_head(&self) {
        self.idx.set(0)
    }

    pub fn to_tail(&self) {
        self.idx.set(self.data.len() - 1)
    }

    pub fn get_with(&self, i: usize) -> Option<&Graphemes> {
        self.data.get(i)
    }

    pub fn get(&self) -> Graphemes {
        self.data
            .get(self.pos())
            .map(|v| v.to_owned())
            .unwrap_or_default()
    }
}

#[test]
fn prev() {
    let mut b = SelectBox::default();
    b.register_all(vec!["a", "b", "c"]);
    assert!(!b.prev());
    b.idx.set(1);
    assert!(b.prev());
}

#[test]
fn next() {
    let mut b = SelectBox::default();
    b.register_all(vec!["a", "b", "c"]);
    assert!(b.next());
    b.idx.set(b.data.len() - 1);
    assert!(!b.next());
}