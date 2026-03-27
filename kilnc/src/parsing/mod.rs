pub mod ast;
pub mod lexer;
pub mod parser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: Location,
    pub end: Location,
    pub start_offset: usize,
    pub end_offset: usize,
}

impl Span {
    pub fn new(start: Location, end: Location) -> Self {
        Self {
            start,
            end,
            start_offset: start.offset,
            end_offset: end.offset,
        }
    }
}
