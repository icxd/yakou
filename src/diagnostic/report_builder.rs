use super::{
    char_set::{CharacterSet, UNICODE},
    label::Label,
    position::Position,
    report::{Report, ReportType},
    source::{Source, SOURCE_CACHE},
    span::Span,
};
use anstyle::{AnsiColor, Color, Reset, Style};
use ilog::IntLog;
use rand::Rng;
use std::{
    cmp::max,
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct FileReportBuilder {
    file_path: Option<String>,
    source_name: String,
    source_file: Option<PathBuf>,
    source: Option<String>,
    enable_color: bool,
    character_set: CharacterSet,
    reports: Vec<Report>,
}

impl FileReportBuilder {
    pub fn new_file(source_file: &Path) -> FileReportBuilder {
        let file_path = source_file.to_string_lossy().to_string();

        FileReportBuilder {
            file_path: Some(file_path.clone()),
            source_name: file_path.clone(),
            source_file: Some(source_file.to_path_buf()),
            source: None,
            enable_color: true,
            character_set: UNICODE.clone(),
            reports: Vec::new(),
        }
    }

    pub fn new_source(source: String) -> FileReportBuilder {
        FileReportBuilder {
            file_path: None,
            source_name: "Unknown".to_string(),
            source_file: None,
            source: Some(if source.ends_with('\n') {
                source
            } else {
                source + "\n"
            }),
            enable_color: true,
            character_set: UNICODE.clone(),
            reports: Vec::new(),
        }
    }

    pub fn source_file(source_file: &Path) -> FileReportBuilder {
        FileReportBuilder::new_file(source_file)
    }

    pub fn source(source: String) -> FileReportBuilder {
        FileReportBuilder::new_source(source)
    }

    pub fn print(&mut self, print_stream: &mut dyn std::io::Write) {
        let source: Source;
        unsafe {
            let mut cache = SOURCE_CACHE.clone().unwrap();

            if let Some(source_file) = &self.source_file {
                match cache.insert(
                    source_file.clone(),
                    Source::from_pathbuf(source_file.to_path_buf()).unwrap(),
                ) {
                    Some(src) => {
                        source = src;
                    }
                    None => {
                        source = cache.get(source_file).unwrap().clone();
                    }
                }
            } else {
                source = Source::from_string(self.source.clone().unwrap());
            }
        }

        for report in &mut self.reports.clone() {
            report.labels.sort_by(|a, b| {
                a.span
                    .start_position
                    .column
                    .cmp(&b.span.start_position.column)
            });
            let max_number_of_digit = max(
                usize::log10(report.common_span.start_position.line) + 1,
                usize::log10(report.common_span.end_position.line) + 1,
            ) as usize;

            let mut occupied_multiline_labels = HashMap::<Label, bool>::new();
            let segment = source.sub_list(
                report.common_span.start_position.line,
                report.common_span.end_position.line,
            );
            dbg!(segment.clone());
            dbg!(source.clone().lines);

            match report.report_type {
                ReportType::Error => Style::new()
                    .fg_color(Some(Color::Ansi(AnsiColor::Red)))
                    .write_to(print_stream)
                    .unwrap(),
                ReportType::Warning => Style::new()
                    .fg_color(Some(Color::Ansi(AnsiColor::Yellow)))
                    .write_to(print_stream)
                    .unwrap(),
            }

            write!(
                print_stream,
                "[{}] {}{}\n",
                report.tag,
                Reset.render(),
                report.message
            )
            .unwrap();

            self.write_source_location(
                print_stream,
                max_number_of_digit,
                &report.common_span.start_position,
            );

            for label in &report.labels {
                if label.is_multiline() {
                    occupied_multiline_labels.insert(label.clone(), false);
                }
            }

            let mut previous_line_rendered = true;
            let mut render_source = false;
            let mut current_dominant_label: Option<Label> = None;
            for line in &segment {
                let mut line_builder = String::new();
                let mut applied_labels: Vec<Option<Label>> = Vec::new();
                let (mut inserted_length, mut most_last_position) = (0, line.length + 1);
                render_source = false;

                for label in &report.labels {
                    if label.is_in(line.line_number) {
                        if !label.is_multiline() {
                            if label.format.is_some() {
                                let mut original_string_pos = label.span.start_position.column;
                                let color_code = self.generate_color_code();
                                self.insert_str(
                                    &mut line_builder,
                                    inserted_length + original_string_pos,
                                    color_code.render_fg().to_string().as_str(),
                                );
                                inserted_length += color_code.render_fg().to_string().len();
                                original_string_pos += label.span.offset();
                                self.insert_str(
                                    &mut line_builder,
                                    inserted_length + original_string_pos,
                                    Reset.render().to_string().as_str(),
                                );
                                inserted_length += Reset.render().to_string().len();

                                if current_dominant_label.is_some()
                                    && current_dominant_label.clone().unwrap().format.is_some()
                                {
                                    self.insert_str(
                                        &mut line_builder,
                                        inserted_length + original_string_pos,
                                        self.generate_color_code().render_fg().to_string().as_str(),
                                    );
                                }
                            }

                            most_last_position =
                                max(most_last_position, label.span.end_position.column + 2);
                            applied_labels.push(Some(label.clone()));
                        } else {
                            if label.format.is_some() {
                                let color_code = self.generate_color_code();
                                let start_pos =
                                    if label.span.start_position.line == line.line_number {
                                        label.span.start_position.column
                                    } else {
                                        0
                                    };
                                self.insert_str(
                                    &mut line_builder,
                                    inserted_length + start_pos,
                                    color_code.render_fg().to_string().as_str(),
                                );
                                inserted_length += color_code.render_fg().to_string().len();
                                self.insert_str(
                                    &mut line_builder,
                                    inserted_length + (line.chars.len() - start_pos),
                                    Reset.render().to_string().as_str(),
                                );
                            }

                            occupied_multiline_labels.insert(label.clone(), true);
                            current_dominant_label = Some(label.clone());
                        }
                    }

                    if (label.span.start_position.line >= line.line_number - 1
                        && label.span.start_position.line <= line.line_number + 1)
                        || (label.span.end_position.line >= line.line_number - 1
                            && label.span.end_position.line <= line.line_number + 1)
                    {
                        render_source = true;
                    }
                }

                if !render_source {
                    if previous_line_rendered {
                        Style::new()
                            .fg_color(Some(Color::Ansi(AnsiColor::BrightBlack)))
                            .write_to(print_stream)
                            .unwrap();
                        write!(
                            print_stream,
                            "{:width$}{}{}",
                            "",
                            self.character_set.vertical_ellipsis,
                            Reset.render(),
                            width = max_number_of_digit + 2,
                        )
                        .unwrap();

                        self.write_multi_line_label(
                            print_stream,
                            usize::MAX,
                            &occupied_multiline_labels,
                            None,
                            self.character_set.vertical_ellipsis,
                        );
                        writeln!(print_stream).unwrap();
                        previous_line_rendered = false;
                    }
                    continue;
                }

                previous_line_rendered = true;

                self.write_line_number(print_stream, line.line_number, max_number_of_digit, false);
                let ended_label = self.write_multi_line_label(
                    print_stream,
                    line.line_number,
                    &occupied_multiline_labels,
                    None,
                    self.character_set.vertical_bar,
                );

                write!(print_stream, "{}", line_builder).unwrap();

                inserted_length = 0;
                if !applied_labels.is_empty() {
                    self.write_line_number(print_stream, usize::MAX, max_number_of_digit, true);
                    self.write_multi_line_label(
                        print_stream,
                        usize::MAX,
                        &occupied_multiline_labels,
                        None,
                        self.character_set.vertical_bar,
                    );

                    for label in &applied_labels {
                        let label = match label {
                            Some(label) => label,
                            None => continue,
                        };
                        let space_length = label.span.start_position.column - inserted_length;
                        if space_length > 0 {
                            write!(print_stream, "{:width$}", "", width = space_length).unwrap();
                        }

                        let offset = label.span.offset();
                        let mut underline_builder = String::new();
                        for k in 0..offset {
                            if offset / 2 == k {
                                underline_builder.push(self.character_set.under_bar);
                            } else {
                                underline_builder.push(self.character_set.underline);
                            }
                        }

                        Style::new()
                            .fg_color(label.format)
                            .write_to(print_stream)
                            .unwrap();
                        write!(print_stream, "{}{}", underline_builder, Reset.render()).unwrap();
                        inserted_length += space_length + offset;
                    }

                    writeln!(print_stream, "").unwrap();

                    for j in 1..=applied_labels.len() * 2 {
                        self.write_line_number(print_stream, usize::MAX, max_number_of_digit, true);
                        self.write_multi_line_label(
                            print_stream,
                            usize::MAX,
                            &occupied_multiline_labels,
                            None,
                            self.character_set.vertical_bar,
                        );

                        inserted_length = 0;
                        for k in 0..=applied_labels.len() {
                            let tmp = applied_labels.clone();
                            let label = match tmp.get(k) {
                                Some(label) => match label {
                                    Some(label) => label,
                                    None => continue,
                                },
                                None => continue,
                            };

                            let space_length = label.span.start_position.column - inserted_length;
                            let offset = label.span.offset() / 2;

                            write!(print_stream, "{}", " ".repeat(space_length + offset)).unwrap();
                            inserted_length += space_length + offset + 1;

                            if j % 2 == 1 {
                                applied_labels[k] = None;
                                Style::new()
                                    .fg_color(label.format)
                                    .write_to(print_stream)
                                    .unwrap();
                                write!(
                                    print_stream,
                                    "{}{}{} {}",
                                    self.character_set.left_bottom,
                                    self.character_set
                                        .horizontal_bar
                                        .to_string()
                                        .repeat(most_last_position - inserted_length),
                                    Reset.render(),
                                    label.message
                                )
                                .unwrap();

                                if label.hint.is_some() {
                                    writeln!(print_stream, "").unwrap();

                                    self.write_line_number(
                                        print_stream,
                                        usize::MAX,
                                        max_number_of_digit,
                                        true,
                                    );
                                    self.write_multi_line_label(
                                        print_stream,
                                        usize::MAX,
                                        &occupied_multiline_labels,
                                        None,
                                        self.character_set.vertical_bar,
                                    );

                                    write!(
                                        print_stream,
                                        "{}",
                                        " ".repeat(
                                            space_length + offset + most_last_position
                                                - inserted_length
                                                + 1
                                        )
                                    )
                                    .unwrap();

                                    Style::new()
                                        .fg_color(Some(Color::Ansi(AnsiColor::BrightBlue)))
                                        .write_to(print_stream)
                                        .unwrap();
                                    write!(
                                        print_stream,
                                        "!hint: {}{}",
                                        label.hint.clone().unwrap(),
                                        Reset.render()
                                    )
                                    .unwrap();
                                }

                                break;
                            }

                            Style::new()
                                .fg_color(label.format)
                                .write_to(print_stream)
                                .unwrap();
                            write!(
                                print_stream,
                                "{}{}",
                                self.character_set.vertical_bar,
                                Reset.render(),
                            )
                            .unwrap();
                        }

                        writeln!(print_stream, "").unwrap();
                    }
                }

                if ended_label.is_some() {
                    self.write_line_number(print_stream, usize::MAX, max_number_of_digit, true);
                    self.write_multi_line_label(
                        print_stream,
                        usize::MAX,
                        &occupied_multiline_labels,
                        None,
                        self.character_set.vertical_bar,
                    );
                    writeln!(print_stream, "").unwrap();
                    self.write_line_number(print_stream, usize::MAX, max_number_of_digit, true);
                    self.write_multi_line_label(
                        print_stream,
                        usize::MAX,
                        &occupied_multiline_labels,
                        ended_label.clone(),
                        self.character_set.vertical_bar,
                    );
                    occupied_multiline_labels.insert(ended_label.clone().unwrap(), false);
                    Style::new()
                        .fg_color(ended_label.clone().unwrap().format)
                        .write_to(print_stream)
                        .unwrap();
                    write!(
                        print_stream,
                        "{}{} {}",
                        self.character_set
                            .horizontal_bar
                            .to_string()
                            .repeat(most_last_position),
                        Reset.render(),
                        ended_label.clone().unwrap().message
                    )
                    .unwrap();

                    if ended_label.clone().unwrap().hint.is_some() {
                        writeln!(print_stream, "").unwrap();

                        self.write_line_number(print_stream, usize::MAX, max_number_of_digit, true);
                        self.write_multi_line_label(
                            print_stream,
                            usize::MAX,
                            &occupied_multiline_labels,
                            None,
                            self.character_set.vertical_bar,
                        );

                        write!(print_stream, "{}", " ".repeat(most_last_position)).unwrap();

                        Style::new()
                            .fg_color(Some(Color::Ansi(AnsiColor::BrightBlue)))
                            .write_to(print_stream)
                            .unwrap();
                        write!(
                            print_stream,
                            "!hint: {}{}",
                            ended_label.clone().unwrap().hint.clone().unwrap(),
                            Reset.render()
                        )
                        .unwrap();
                    }

                    writeln!(print_stream, "").unwrap();
                }
            }

            Style::new()
                .fg_color(Some(Color::Ansi(AnsiColor::BrightBlack)))
                .write_to(print_stream)
                .unwrap();
            writeln!(
                print_stream,
                "{}{}{}",
                self.character_set
                    .horizontal_bar
                    .to_string()
                    .repeat(max_number_of_digit + 1),
                self.character_set.right_bottom,
                Reset.render()
            )
            .unwrap();
        }
    }

    fn write_line_number(
        &mut self,
        print_stream: &mut dyn std::io::Write,
        line_number: usize,
        max_line_digit: usize,
        is_virtual_line: bool,
    ) {
        Style::new()
            .fg_color(Some(Color::Ansi(AnsiColor::BrightBlack)))
            .write_to(print_stream)
            .unwrap();
        if is_virtual_line {
            write!(
                print_stream,
                "{:width$}{}",
                "",
                self.character_set.vertical_bar_breaking,
                width = max_line_digit
            )
            .unwrap();
        } else {
            write!(
                print_stream,
                "{:width$}{}",
                line_number,
                self.character_set.vertical_bar,
                width = max_line_digit
            )
            .unwrap();
        }
        write!(print_stream, "{}", Reset.render()).unwrap();
    }

    fn write_multi_line_label(
        &mut self,
        print_stream: &mut dyn std::io::Write,
        line_number: usize,
        label_map: &HashMap<Label, bool>,
        terminated_label: Option<Label>,
        vertical_bar_variant: char,
    ) -> Option<Label> {
        let mut entries = Vec::new();
        for (label, is_occupied) in label_map {
            entries.push((label.clone(), is_occupied.clone()));
        }
        let mut should_print = true;
        let mut last_index = 0;
        let mut ended_label = None;

        for i in 0..=entries.len() {
            let label: Label = entries[i].0.clone();

            if entries[i].1 {
                Style::new()
                    .fg_color(label.format)
                    .write_to(print_stream)
                    .unwrap();

                if label.span.start_position.line == line_number {
                    write!(print_stream, "{}", self.character_set.left_top).unwrap();
                    write!(
                        print_stream,
                        "{}",
                        self.character_set
                            .horizontal_bar
                            .to_string()
                            .repeat((entries.len() - i) * 2)
                    )
                    .unwrap();
                    write!(print_stream, "{}", self.character_set.right_arrow).unwrap();
                    should_print = false;
                    break;
                } else if label.span.end_position.line == line_number {
                    write!(print_stream, "{}", self.character_set.left_cross).unwrap();
                    write!(
                        print_stream,
                        "{}",
                        self.character_set
                            .horizontal_bar
                            .to_string()
                            .repeat((entries.len() - i) * 2)
                    )
                    .unwrap();
                    write!(print_stream, "{}", self.character_set.right_arrow).unwrap();
                    should_print = false;
                    ended_label = Some(label);
                    break;
                } else if terminated_label.is_some() && label == *terminated_label.as_ref().unwrap()
                {
                    write!(
                        print_stream,
                        "{}{}",
                        self.character_set.left_bottom,
                        self.character_set
                            .horizontal_bar
                            .to_string()
                            .repeat((entries.len() - i) * 2 + 2)
                    )
                    .unwrap();
                    return None;
                } else {
                    write!(print_stream, "{} ", vertical_bar_variant).unwrap();
                }

                write!(print_stream, "{}", Reset.render()).unwrap();
            } else {
                write!(print_stream, "  ").unwrap();
            }

            last_index = i + 1;
        }

        if should_print {
            let count = (entries.len() - last_index) * 2 + 3;
            write!(print_stream, "{:width$}", "", width = count).unwrap();
        } else {
            write!(print_stream, " ").unwrap();
        }

        ended_label
    }

    fn generate_color_code(&self) -> AnsiColor {
        let colors = vec![
            AnsiColor::BrightRed,
            AnsiColor::BrightGreen,
            AnsiColor::BrightYellow,
            AnsiColor::BrightBlue,
            AnsiColor::BrightMagenta,
            AnsiColor::BrightCyan,
            AnsiColor::BrightWhite,
        ];
        let mut rng = rand::thread_rng();
        colors[rng.gen_range(0..colors.len())]
    }

    fn write_source_location(
        &mut self,
        print_stream: &mut dyn std::io::Write,
        max_line_digit: usize,
        start_position: &Position,
    ) {
        Style::new()
            .fg_color(Some(Color::Ansi(AnsiColor::BrightBlack)))
            .write_to(print_stream)
            .unwrap();
        write!(
            print_stream,
            "{:>width$}{}[",
            self.character_set.left_top,
            self.character_set.horizontal_bar,
            width = max_line_digit + 2
        )
        .unwrap();
        write!(
            print_stream,
            "{}{}:{}:{}",
            Reset.render(),
            self.source_name,
            start_position.line,
            start_position.column
        )
        .unwrap();
        Style::new()
            .fg_color(Some(Color::Ansi(AnsiColor::BrightBlack)))
            .write_to(print_stream)
            .unwrap();
        writeln!(print_stream, "]{}", Reset.render()).unwrap();
    }

    fn insert_str(&self, string: &mut String, index: usize, value: &str) {
        let mut new_string = String::new();
        for (i, c) in string.chars().enumerate() {
            if i == index {
                new_string.push_str(value);
            }
            new_string.push(c);
        }
        *string = new_string;
    }

    pub fn error(&mut self, span: Span, message: String) -> ReportBuilder {
        ReportBuilder::new(self.clone(), Report::error(span, message))
    }

    pub fn warning(&mut self, span: Span, message: String) -> ReportBuilder {
        ReportBuilder::new(self.clone(), Report::warning(span, message))
    }

    pub fn source_name(&mut self, source_name: String) -> FileReportBuilder {
        self.source_name = source_name;
        self.clone()
    }
}

#[derive(Debug, Clone)]
pub struct ReportBuilder {
    parent_builder: FileReportBuilder,
    report: Report,
}

impl ReportBuilder {
    pub fn new(parent_builder: FileReportBuilder, report: Report) -> Self {
        Self {
            parent_builder,
            report,
        }
    }

    pub fn tag(&mut self, tag: String) -> Self {
        self.report.tag = tag;
        self.clone()
    }

    pub fn label(&mut self, span: Span, message: String) -> LabelBuilder {
        LabelBuilder::new(self, Label::new(span, message))
    }

    pub fn build(&mut self) -> FileReportBuilder {
        self.parent_builder.reports.push(self.report.clone());
        self.parent_builder.clone()
    }
}

#[derive(Debug, Clone)]
pub struct LabelBuilder {
    parent_builder: ReportBuilder,
    label: Label,
}

impl LabelBuilder {
    pub fn new(parent_builder: &ReportBuilder, label: Label) -> Self {
        Self {
            parent_builder: parent_builder.clone(),
            label,
        }
    }

    pub fn color(&mut self, color: Color) -> Self {
        self.label.format = Some(color);
        self.clone()
    }

    pub fn hint(&mut self, hint: String) -> Self {
        self.label.hint = Some(hint);
        self.clone()
    }

    pub fn build(&mut self) -> ReportBuilder {
        self.parent_builder.report.labels.push(self.label.clone());
        self.parent_builder.clone()
    }
}
