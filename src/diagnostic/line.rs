use super::{position::Position, span::Span};

#[derive(Debug, Clone)]
pub struct Line {
    pub line_number: usize,
    pub offset: usize,
    pub length: usize,
    pub chars: String,
}

impl Line {
    pub fn new(line_number: usize, offset: usize, length: usize, chars: String) -> Self {
        Self {
            line_number,
            offset,
            length,
            chars,
        }
    }

    pub fn span(&self) -> Span {
        Span::new(
            Position::new(self.line_number, self.offset),
            Position::new(self.line_number, self.offset + self.length),
        )
    }
}
