use super::{
    char_set::{CharacterSet, UNICODE},
    label::Label,
    report::{Report, ReportType},
    source::{Source, SOURCE_CACHE},
};
use colored::*;
use ilog::IntLog;
use linked_hash_map::LinkedHashMap;
use std::{
    cmp::max,
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

        for report in &mut self.reports {
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

            let occupied_multiline_labels = LinkedHashMap::<Label, bool>::new();
            let segment = source.sub_list(
                report.common_span.start_position.line,
                report.common_span.end_position.line,
            );

            match report.report_type {
                ReportType::Error => {}
                ReportType::Warning => {}
            }

            write!(
                print_stream,
                "[{}]",
                report.tag.color(match report.report_type {
                    ReportType::Error => "red",
                    ReportType::Warning => "yellow",
                })
            )
            .unwrap();
        }
    }
}
