use crate::featurez::syntax::SyntaxKind;

#[derive(Clone)]
pub struct Marker {
    index: usize,
    disabled: bool,
}

pub struct CompletedMarker {
    start: usize,
    end: usize,
    kind: SyntaxKind,
}

impl Marker {
    pub fn new(index: usize) -> Marker {
        Marker {
            index,
            disabled: false,
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn defuse(mut self, end: usize, kind: SyntaxKind) -> CompletedMarker {
        self.disabled = true;

        CompletedMarker {
            start: self.index,
            end,
            kind,
        }
    }

    pub fn abandon(&mut self) {
        self.disabled = true;
    }
}

impl Drop for Marker {
    fn drop(&mut self) {
        if !self.disabled {
            //panic!("You must disable or abandon the marker!")
        }
    }
}

impl CompletedMarker {
    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn kind(&self) -> SyntaxKind {
        self.kind
    }
}
