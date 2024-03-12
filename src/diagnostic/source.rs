use super::line::Line;
use std::{collections::HashMap, path::PathBuf};

pub static mut SOURCE_CACHE: Option<HashMap<PathBuf, Source>> = None;

#[derive(Debug, Clone)]
pub struct Source {
    pub lines: Vec<Line>,
}

impl Source {
    pub fn new(lines: Vec<Line>) -> Self {
        Self { lines }
    }

    pub fn from_pathbuf(path: PathBuf) -> Option<Self> {
        let source = std::fs::read_to_string(&path).ok()?;
        let lines = source
            .lines()
            .enumerate()
            .map(|(line_number, line)| Line::new(line_number, 0, line.len(), line.to_string()))
            .collect();
        Some(Self::new(lines))
    }

    pub fn sub_list(&self, from: usize, to: usize) -> Vec<Line> {
        self.lines[from..to].to_vec()
    }

    pub fn from_string(source: String) -> Self {
        let lines = source
            .lines()
            .enumerate()
            .map(|(line_number, line)| Line::new(line_number, 0, line.len(), line.to_string()))
            .collect();
        Self::new(lines)
    }
}
