//! Parsing of plaintext (.cells) files
//!
//! Based on information from <https://conwaylife.com/wiki/Plaintext>.

use super::*;

use nom::{
    branch::alt,
    character::complete::{char, line_ending, not_line_ending},
    combinator::{eof, opt, peek, value},
    error::context,
    multi::{many1, separated_list0},
    sequence::{delimited, pair},
    Finish, Parser,
};

pub struct PlaintextParser();

const FILE_EXTENSIONS: &'static [&'static str] = &["cells"];

impl LifeParser for PlaintextParser {
    fn file_extensions(&self) -> &[&str] {
        FILE_EXTENSIONS
    }

    fn sniff(&self, input: &str) -> bool {
        input.starts_with(['!', '.', 'O'])
    }

    fn parse(&self, input: &str) -> Result<Grid, ParseError> {
        parse_plaintext(input)
    }
}

pub fn parse_plaintext(input: &str) -> Result<Grid, ParseError> {
    let (_rest, rows) = context("plaintext", plaintext)(input)
        .finish()
        .map_err(|e| ParseError::new(e, input))?;
    let grid = normalize_rows(rows);
    Ok(grid)
}

fn normalize_rows(rows: Vec<Vec<Cell>>) -> Grid {
    let height = rows.len();
    let width = rows.iter().map(|r| r.len()).max().unwrap_or_default();
    let cells = vec![Cell::Dead; height * width];
    let mut grid = Grid {
        height,
        width,
        cells,
    };

    for (y, row) in rows.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            grid[(x, y)] = *cell;
        }
    }

    grid
}

fn plaintext(i: &str) -> VIResult<&str, Vec<Vec<Cell>>> {
    let comments = context("comments", separated_list0(line_ending, bang_comment));
    let cell_rows = context(
        "cells",
        separated_list0(line_ending, cell_row.or(empty_cell_row)),
    );
    delimited(
        comments.and(line_ending),
        cell_rows,
        context("end of file", opt(line_ending).and(eof)),
    )(i)
}

fn cell_row(i: &str) -> VIResult<&str, Vec<Cell>> {
    let alive = value(Cell::Alive, char('O'));
    let dead = value(Cell::Dead, char('.'));
    let cell = alt((alive, dead));
    context("cell row", many1(cell))(i)
}

fn empty_cell_row(i: &str) -> VIResult<&str, Vec<Cell>> {
    context("empty cell row", value(Vec::new(), peek(line_ending)))(i)
}

fn bang_comment(i: &str) -> VIResult<&str, ()> {
    context("comment", value((), pair(char('!'), not_line_ending)))(i)
}

#[cfg(test)]
mod test {
    use super::*;

    const GLIDER: &str = include_pattern!("glider.cells");

    #[test]
    fn test_parse() {
        let parsed = parse_plaintext(GLIDER);
        let expected = String::from(".O.\n..O\nOOO\n");
        assert_eq!(Ok(expected), parsed.map(|g| g.to_string()))
    }

    #[test]
    fn test_plaintext_glider() {
        assert!(GLIDER.ends_with("\n"));
        let parsed = plaintext(GLIDER);
        assert!(parsed.is_ok());
        let (rest, parsed) = parsed.unwrap();
        assert_eq!(rest, "");
        assert_eq!(3, parsed.len());
    }

    #[test]
    fn test_plaintext() {
        assert_eq!(
            Ok(("", vec![vec![Cell::Dead]])),
            plaintext("!Name: Foo\n!\n.")
        )
    }

    #[test]
    fn test_plaintext_empty_row() {
        assert_eq!(
            Ok(("", vec![vec![Cell::Dead], vec![], vec![Cell::Alive]])),
            plaintext("!Name: Foo\n!\n.\n\nO\n")
        )
    }

    #[test]
    fn test_cell_row_no_endine() {
        assert_eq!(Ok(("", vec![Cell::Dead, Cell::Alive])), cell_row(".O"));
    }

    #[test]
    fn test_cell_row_endine() {
        assert_eq!(Ok(("\n", vec![Cell::Dead, Cell::Alive])), cell_row(".O\n"));
    }

    #[test]
    fn test_cell_row_multiple_rows() {
        assert_eq!(
            Ok(("\nO.\n", vec![Cell::Dead, Cell::Alive])),
            cell_row(".O\nO.\n")
        );
    }

    #[test]
    fn test_comment() {
        assert_eq!(Ok(("\nO.\n", ())), bang_comment("!Name: Foo\nO.\n"));
    }
}
