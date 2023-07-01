use crate::util::{Pos, Size};
use miette::{miette, Diagnostic, Report, SourceSpan};
use std::collections::HashSet;
use thiserror::Error;

#[derive(PartialEq, Debug)]
pub struct ParsedField {
    pub size: Size,
    pub unavailable: HashSet<Pos>,
}

#[derive(Debug, Error, PartialEq, Diagnostic)]
pub enum ParseError {
    #[error("Unexpected character")]
    UnexpectedCharacter {
        #[label("Expected '{char_busy}' for busy or '{char_empty}' for empty")]
        loc: SourceSpan,
        char_busy: char,
        char_empty: char,
    },
    #[error("Fickle row length")]
    FickleRowLength {
        #[label("First row length is {len_reference}")]
        reference_row: SourceSpan,
        #[label("Found row with length {len_actual}")]
        bad_row: SourceSpan,
        len_reference: usize,
        len_actual: usize,
    },
    #[error("Not enough rows, should be at least 2")]
    NotEnoughRows {
        #[label("Found only one row")]
        all_rows_span: SourceSpan,
    },
    #[error("Not enough columns, should be at least 2")]
    NotEnoughColumns {
        #[label("here")]
        short_row_span: SourceSpan,
    },
}

pub struct Parser {
    char_empty: char,
    char_busy: char,
}

impl Parser {
    pub fn new(char_empty: char, char_busy: char) -> Self {
        Self {
            char_empty,
            char_busy,
        }
    }

    pub fn parse(&self, field: impl AsRef<str>) -> Result<ParsedField, Report> {
        let field_str = field.as_ref();

        self.parse_without_source_code(field_str)
            .map_err(|report| report.with_source_code(field_str.to_owned()))
    }

    fn parse_without_source_code(&self, field: impl AsRef<str>) -> Result<ParsedField, Report> {
        let source_code = field.as_ref();

        let mut cols = 0usize;
        let mut rows = 0;
        let mut unavailable = HashSet::new();

        for (row, iter_str_offsets::LineOffset { line, offset }) in
            iter_str_offsets::lines_with_offsets(&source_code).enumerate()
        {
            let line_len = line.len();

            if cols == 0 {
                cols = line_len;
                if cols < 2 {
                    return Err(ParseError::NotEnoughColumns {
                        short_row_span: (offset, line.len()).into(),
                    })?;
                }
            } else if line_len != cols {
                return Err(ParseError::FickleRowLength {
                    reference_row: (0, cols).into(),
                    bad_row: (offset, line_len).into(),
                    len_reference: cols,
                    len_actual: line_len,
                })?;
            }

            for (col, char) in line.chars().enumerate() {
                if char == self.char_busy {
                    unavailable.insert(Pos::new(row, col));
                } else if char != self.char_empty {
                    return Err(ParseError::UnexpectedCharacter {
                        loc: (offset + col, 1).into(),
                        char_empty: self.char_empty,
                        char_busy: self.char_busy,
                    })?;
                }
            }

            rows += 1;
        }

        if rows == 0 {
            return Err(miette!("Empty input"));
        }

        if rows < 2 {
            return Err(ParseError::NotEnoughRows {
                all_rows_span: (0, source_code.len()).into(),
            })?;
        }

        Ok(ParsedField {
            size: Size::new(rows, cols),
            unavailable,
        })
    }
}

mod iter_str_offsets {
    /// Same as [`str::lines`], but also yields line offset
    pub fn lines_with_offsets(source: &str) -> impl Iterator<Item = LineOffset<'_>> {
        source
            .split_inclusive("\n")
            .str_offsets()
            .map(|(offset, line)| {
                fn strip_n_r(line: &str) -> &str {
                    let Some(line) = line.strip_suffix('\n') else { return line };
                    let Some(line) = line.strip_suffix('\r') else { return line };
                    line
                }

                LineOffset {
                    offset,
                    line: strip_n_r(line),
                }
            })
    }

    struct Offset<I> {
        iter: I,
        offset: usize,
    }

    trait IteratorStrOffsetExt<I>
    where
        I: Iterator,
    {
        fn str_offsets(self) -> Offset<I>;
    }

    impl<'a, I> IteratorStrOffsetExt<I> for I
    where
        I: Iterator<Item = &'a str>,
    {
        fn str_offsets(self) -> Offset<I> {
            Offset {
                iter: self,
                offset: 0,
            }
        }
    }

    impl<'a, I> Iterator for Offset<I>
    where
        I: Iterator<Item = &'a str>,
    {
        type Item = (usize, <I as Iterator>::Item);

        fn next(&mut self) -> Option<Self::Item> {
            self.iter.next().map(|a| {
                let i = self.offset;
                self.offset += a.len();
                (i, a)
            })
        }
    }

    pub struct LineOffset<'a> {
        pub line: &'a str,
        pub offset: usize,
    }

    // TODO: unit tests
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
            parser.parse_without_source_code("--\n--"),
            Ok(ParsedField {
                size: Size::new(2, 2),
                unavailable: HashSet::new()
            })
        );
    }

    #[test]
    fn parses_non_empty_3x2() {
        let parser = factory();

        assert_eq!(
            parser.parse_without_source_code("--+\n-+-"),
            Ok(ParsedField {
                size: Size::new(2, 3),
                unavailable: {
                    let mut set = HashSet::new();
                    set.insert(Pos::new(0, 2));
                    set.insert(Pos::new(1, 1));
                    set
                }
            })
        );
    }

    #[test]
    fn unexpected_char_encountered() {
        let parser = factory();

        assert_eq!(
            parser.parse_without_source_code("---\n--#"),
            Err(ParseError::UnexpectedCharacter {
                loc: (6, 1).into(),
                char_busy: '+',
                char_empty: '-',
            })
        );
    }
}
