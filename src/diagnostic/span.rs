use super::position::Position;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    pub start_position: Position,
    pub end_position: Position,
}

impl Span {
    pub fn new(start_position: Position, end_position: Position) -> Self {
        Self {
            start_position,
            end_position,
        }
    }

    pub fn range(start_line_number: usize, end_line_number: usize) -> Self {
        Self::multiple_line(start_line_number, 0, end_line_number, 0)
    }

    pub fn single_line(line_number: usize, start: usize, end: usize) -> Self {
        Self {
            start_position: Position::new(line_number, start),
            end_position: Position::new(line_number, end),
        }
    }

    pub fn multiple_line(
        start_line_number: usize,
        start: usize,
        end_line_number: usize,
        end: usize,
    ) -> Self {
        Self {
            start_position: Position::new(start_line_number, start),
            end_position: Position::new(end_line_number, end),
        }
    }

    pub fn expand(&self, other: Span) -> Span {
        let copied = self.clone();

        if other.end_position.line < self.start_position.line {
            return copied;
        } else if other.end_position.line == self.start_position.line {
            if other.end_position.column < self.start_position.column {
                return copied;
            }
        }

        let start_position = self.start_position;
        let end_position = other.end_position;

        Span::new(start_position, end_position)
    }

    pub fn is_multiline(&self) -> bool {
        self.start_position.line != self.end_position.line
    }

    pub fn is_in(&self, line: usize) -> bool {
        self.start_position.line <= line && line <= self.end_position.line
    }

    pub fn offset(&self) -> usize {
        if self.start_position.line != self.end_position.line
            || self.start_position.column > self.end_position.column
        {
            return usize::MAX;
        }

        self.end_position.column - self.start_position.column
    }
}
