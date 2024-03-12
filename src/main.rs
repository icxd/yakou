use anstyle::{AnsiColor, Color};
use std::{collections::HashMap, io::stdout, path::Path};
use yakou::diagnostic::{report_builder::FileReportBuilder, source::SOURCE_CACHE, span::Span};

fn main() {
    unsafe { SOURCE_CACHE = Some(HashMap::new()) }

    let path = Path::new("tests/test.yk");

    FileReportBuilder::source_file(path)
        .error(Span::single_line(1, 0, 6), "test or smth".to_string())
        .tag("E01".to_string())
        .label(Span::single_line(1, 6, 11), "L".to_string())
        .color(Color::Ansi(AnsiColor::BrightCyan))
        .hint("Hint L".to_string())
        .build()
        .label(Span::single_line(3, 0, 4), "impl dude".to_string())
        .color(Color::Ansi(AnsiColor::BrightCyan))
        .hint("impl it!".to_string())
        .build()
        .build()
        .print(&mut stdout());
}
