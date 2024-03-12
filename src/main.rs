use std::{collections::HashMap, io::stdout, path::Path};

use anstyle::{AnsiColor, Color};
use yakou::diagnostic::{report_builder::FileReportBuilder, source::SOURCE_CACHE, span::Span};

fn main() {
    unsafe { SOURCE_CACHE = Some(HashMap::new()) }

    let path = Path::new("src/main.rs");

    FileReportBuilder::source_file(path)
        .error(Span::multiple_line(1, 0, 6, 0), "test or smth".to_string())
        .tag("E01".to_string())
        .label(Span::multiple_line(1, 0, 3, 8), "LOL".to_string())
        .color(Color::Ansi(AnsiColor::Red))
        .build()
        .label(Span::multiple_line(2, 0, 5, 0), "KEK".to_string())
        .color(Color::Ansi(AnsiColor::BrightMagenta))
        .build()
        .label(Span::single_line(1, 6, 11), "L".to_string())
        .color(Color::Ansi(AnsiColor::BrightCyan))
        .hint("Hint L".to_string())
        .build()
        .label(Span::single_line(3, 0, 4), "impl dude".to_string())
        .color(Color::Ansi(AnsiColor::BrightCyan))
        .hint("impl it!".to_string())
        .build()
        .label(Span::multiple_line(4, 0, 6, 0), "kek".to_string())
        .hint("omegalul".to_string())
        .build()
        .build()
        .print(&mut stdout());
}
