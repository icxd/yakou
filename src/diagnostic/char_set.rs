use lazy_static::lazy_static;

#[derive(Debug, Clone, Copy)]
pub struct CharacterSet {
    pub right_arrow: char,
    pub horizontal_bar: char,
    pub vertical_bar: char,
    pub cross_bar: char,
    pub under_bar: char,
    pub left_cross: char,
    pub vertical_bar_breaking: char,
    pub vertical_ellipsis: char,
    pub left_top: char,
    pub left_bottom: char,
    pub right_bottom: char,
    pub underline: char,
}

impl CharacterSet {
    pub fn new(
        right_arrow: char,
        horizontal_bar: char,
        vertical_bar: char,
        cross_bar: char,
        under_bar: char,
        left_cross: char,
        vertical_bar_breaking: char,
        vertical_ellipsis: char,
        left_top: char,
        left_bottom: char,
        right_bottom: char,
        underline: char,
    ) -> Self {
        Self {
            right_arrow,
            horizontal_bar,
            vertical_bar,
            cross_bar,
            under_bar,
            left_cross,
            vertical_bar_breaking,
            vertical_ellipsis,
            left_top,
            left_bottom,
            right_bottom,
            underline,
        }
    }
}

lazy_static! {
    pub static ref UNICODE: CharacterSet =
        CharacterSet::new('▶', '─', '│', '┼', '┬', '├', '·', '⋮', '╭', '╰', '╯', '─');
    pub static ref ASCII: CharacterSet =
        CharacterSet::new('>', '-', '|', '+', '|', '|', '*', ':', ',', '`', '\'', '^');
}
