use super::{
    char_set::{CharacterSet, UNICODE},
    label::Label,
    position::Position,
    report::{Report, ReportType},
    source::{Source, SOURCE_CACHE},
};
use anstyle::{AnsiColor, Color, Reset, Style};
use ilog::IntLog;
use linked_hash_map::LinkedHashMap;
use rand::Rng;
use std::{
    cmp::max,
    collections::LinkedList,
    path::{Path, PathBuf},
};

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
    fn new_file(source_file: &Path) -> FileReportBuilder {
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

    fn new_source(source: String) -> FileReportBuilder {
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

    fn source_file(source_file: &Path) -> FileReportBuilder {
        FileReportBuilder::new_file(source_file)
    }

    fn source(source: String) -> FileReportBuilder {
        FileReportBuilder::new_source(source)
    }

    fn print(&mut self, print_stream: &mut dyn std::io::Write) {
        let mut source: Source;

        if let Some(source_file) = &self.source_file {
            source = SOURCE_CACHE
                .get(source_file)
                .expect("Source file not found in cache")
                .clone();
        } else {
            source = Source::from_string(self.source.clone().unwrap());
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

            let mut occupied_multiline_labels = LinkedHashMap::<Label, bool>::new();
            let segment = source.sub_list(
                report.common_span.start_position.line,
                report.common_span.end_position.line,
            );

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
                Reset.render().to_string(),
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
                let mut applied_labels = LinkedList::<Label>::new();
                let (mut inserted_length, mut most_last_position) = (0, line.length + 1);
                render_source = false;

                for label in &report.labels {
                    if label.is_in(line.line_number) {
                        if !label.is_multiline() {
                            if label.format.is_some() {
                                let mut original_string_pos = label.span.start_position.column;
                                let color_code = self.generate_color_code();
                                line_builder.insert_str(
                                    inserted_length + original_string_pos,
                                    color_code.render_fg().to_string().as_str(),
                                );
                                inserted_length += color_code.render_fg().to_string().len();
                                original_string_pos += label.span.offset();
                                line_builder.insert_str(
                                    inserted_length + original_string_pos,
                                    Reset.render().to_string().as_str(),
                                );
                                inserted_length += Reset.render().to_string().len();

                                if current_dominant_label.is_some()
                                    && current_dominant_label.clone().unwrap().format.is_some()
                                {
                                    line_builder.insert_str(
                                        inserted_length + original_string_pos,
                                        self.generate_color_code().render_fg().to_string().as_str(),
                                    );
                                }
                            }

                            most_last_position =
                                max(most_last_position, label.span.end_position.column + 2);
                            applied_labels.push_back(label.clone());
                        } else {
                            if label.format.is_some() {
                                let color_code = self.generate_color_code();
                                let start_pos =
                                    if label.span.start_position.line == line.line_number {
                                        label.span.start_position.column
                                    } else {
                                        0
                                    };
                                line_builder.insert_str(
                                    inserted_length + start_pos,
                                    color_code.render_fg().to_string().as_str(),
                                );
                                inserted_length += color_code.render_fg().to_string().len();
                                line_builder.insert_str(
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
                            Reset.render().to_string(),
                            width = max_number_of_digit + 2,
                        )
                        .unwrap();

                        self.write_multi_line_label(
                            print_stream,
                            -1,
                            &occupied_multiline_labels,
                            None,
                            self.character_set.vertical_ellipsis,
                        );
                        writeln!(print_stream).unwrap();
                        previous_line_rendered = false;
                    }
                    continue;
                }
            }
        }
    }

    fn write_multi_line_label(
        &mut self,
        print_stream: &mut dyn std::io::Write,
        line_number: i32,
        label_map: &LinkedHashMap<Label, bool>,
        terminated_label: Option<Label>,
        vertical_bar_variant: char,
    ) -> Label {
        let mut entries = Vec::new();
        for (label, is_occupied) in label_map {
            entries.push((label.clone(), is_occupied.clone()));
        }
        let mut should_print = true;
        let mut last_index = 0;
        let mut ended_label = None;

        for i in 0..entries.len() {
            let label: Label = entries[i].0.clone();

            if entries[i].1 {
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

        ended_label.unwrap()
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
            "{:width$}{}[",
            "",
            self.character_set.left_top,
            width = max_line_digit + 2
        )
        .unwrap();
        write!(
            print_stream,
            "{}{}:{}:{}",
            Reset.render().to_string(),
            self.source_name,
            start_position.line,
            start_position.column
        )
        .unwrap();
        Style::new()
            .fg_color(Some(Color::Ansi(AnsiColor::BrightBlack)))
            .write_to(print_stream)
            .unwrap();
        write!(print_stream, "]{}\n", Reset.render().to_string()).unwrap();
    }
}
