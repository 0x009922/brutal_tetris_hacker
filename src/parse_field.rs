use crate::util::{Pos, Size};
use std::collections::HashSet;

#[derive(PartialEq, Debug)]
pub struct ParsedField {
    pub size: Size,
    pub unavailable: HashSet<Pos>,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ParseError {
    #[error("unexpected character in row {row}; expected for empty: '{char_empty}', expected for buse: '{char_busy}', got: '{char}'")]
    UnexpectedCharacter {
        char: char,
        row: usize,
        char_busy: char,
        char_empty: char,
    },
    #[error("fickle length of the row no. {row}; expected: {expected}, actual: {actual}")]
    FickleRowLength {
        row: usize,
        expected: usize,
        actual: usize,
    },
    #[error("not enough rows, should be at least 2")]
    NotEnoughRows,
    #[error("not enough columns, should be at least 2")]
    NotEnoughColumns,
}

pub struct Parser {
    char_empty: char,
    char_busy: char,
}

impl Parser {
    pub fn new(char_empty: char, char_busy: char) -> Self {
        Self {
            char_busy,
            char_empty,
        }
    }

    pub fn parse(&self, field: impl AsRef<str>) -> Result<ParsedField, ParseError> {
        let lines = field.as_ref().lines();

        let mut cols = 0usize;
        let mut rows = 0;
        let mut unavailable = HashSet::new();

        for (row, line) in lines.enumerate() {
            let line_len = line.len();

            if cols == 0 {
                cols = line_len;
                if cols < 2 {
                    return Err(ParseError::NotEnoughColumns);
                }
            } else if line_len != cols {
                return Err(ParseError::FickleRowLength {
                    row,
                    expected: cols,
                    actual: line_len,
                });
            }

            for (col, char) in line.chars().enumerate() {
                if char == self.char_busy {
                    unavailable.insert(Pos::new(row, col));
                } else if char != self.char_empty {
                    return Err(ParseError::UnexpectedCharacter {
                        row,
                        char,
                        char_empty: self.char_empty,
                        char_busy: self.char_busy,
                    });
                }
            }

            rows += 1;
        }

        if rows < 2 {
            return Err(ParseError::NotEnoughRows);
        }

        Ok(ParsedField {
            size: Size::new(rows, cols),
            unavailable,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn factory() -> Parser {
        Parser::new('-', '+')
    }

    #[test]
    fn parses_empty_2x2_field() {
        let parser = factory();

        assert_eq!(
            parser.parse("--\n--"),
            Ok(ParsedField {
                size: Size::new(2, 2),
                unavailable: HashSet::new()
            })
        )
    }

    #[test]
    fn parses_non_empty_3x2() {
        let parser = factory();

        assert_eq!(
            parser.parse("--+\n-+-"),
            Ok(ParsedField {
                size: Size::new(2, 3),
                unavailable: {
                    let mut set = HashSet::new();
                    set.insert(Pos::new(0, 2));
                    set.insert(Pos::new(1, 1));
                    set
                }
            })
        )
    }

    #[test]
    fn unexpected_char_encountered() {
        let parser = factory();

        assert_eq!(
            parser.parse("---\n--#"),
            Err(ParseError::UnexpectedCharacter {
                char: '#',
                char_busy: '+',
                char_empty: '-',
                row: 1
            })
        )
    }
}
