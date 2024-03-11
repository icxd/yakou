use super::span::Span;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Label {
    pub span: Span,
    pub message: String,
    pub hint: Option<String>,
}

impl Label {
    pub fn new(span: Span, message: String) -> Self {
        Self {
            span,
            message,
            hint: None,
        }
    }

    pub fn with_hint(&mut self, hint: String) -> Self {
        self.hint = Some(hint);
        self.clone()
    }
}
