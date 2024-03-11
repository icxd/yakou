use super::span::Span;
use anstyle::Color;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Label {
    pub format: Option<Color>,
    pub span: Span,
    pub message: String,
    pub hint: Option<String>,
}

impl Label {
    pub fn new(span: Span, message: String) -> Self {
        Self {
            format: None,
            span,
            message,
            hint: None,
        }
    }

    pub fn set_hint(&mut self, hint: Option<String>) -> Self {
        self.hint = hint;
        self.clone()
    }

    pub fn set_format(&mut self, format: Option<Color>) -> Self {
        self.format = format;
        self.clone()
    }

    pub fn is_multiline(&self) -> bool {
        self.span.is_multiline()
    }

    pub fn is_in(&self, line: usize) -> bool {
        self.span.is_in(line)
    }
}
