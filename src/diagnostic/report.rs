use super::{label::Label, span::Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum ReportType {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct Report {
    pub tag: String,
    pub common_span: Span,
    pub labels: Vec<Label>,
    pub report_type: ReportType,
    pub message: String,
}

impl Report {
    pub fn new(tag: Option<String>, span: Span, report_type: ReportType, message: String) -> Self {
        Self {
            tag: if tag.is_some() {
                tag.unwrap()
            } else {
                "".to_string()
            },
            common_span: span,
            labels: vec![],
            report_type,
            message,
        }
    }

    pub fn error(span: Span, message: String) -> Self {
        Self::new(Some("error".to_string()), span, ReportType::Error, message)
    }

    pub fn warning(span: Span, message: String) -> Self {
        Self::new(
            Some("warning".to_string()),
            span,
            ReportType::Warning,
            message,
        )
    }

    pub fn add_label(&mut self, label: Label) {
        self.labels.push(label);
    }
}
