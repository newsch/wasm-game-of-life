use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

use nom::{
    branch::alt,
    character::complete::{char, line_ending, not_line_ending},
    combinator::{eof, opt, peek, value},
    error::{context, Error},
    multi::{many0, many1, separated_list0},
    sequence::{delimited, pair, separated_pair, terminated},
    Finish, IResult, Parser,
};

use crate::Cell;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Cell>,
}

impl Grid {
    fn xy2i(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }
}

impl Index<(usize, usize)> for Grid {
    type Output = Cell;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let (x, y) = index;
        &self.cells[self.xy2i(x, y)]
    }
}

impl IndexMut<(usize, usize)> for Grid {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        let (x, y) = index;
        let i = self.xy2i(x, y);
        &mut self.cells[i]
    }
}

impl Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                let output = match self[(x, y)] {
                    Cell::Alive => "O",
                    Cell::Dead => ".",
                };
                write!(f, "{}", output)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

pub fn parse_plaintext(input: &str) -> Result<Grid, Error<&str>> {
    let (_rest, rows) = plaintext(input).finish()?;
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

fn plaintext(i: &str) -> IResult<&str, Vec<Vec<Cell>>> {
    let comments = context("comments", separated_list0(line_ending, bang_comment));
    let cell_rows = context(
        "cells",
        separated_list0(line_ending, cell_row.or(empty_cell_row)),
    );
    delimited(
        comments.and(line_ending),
        cell_rows,
        opt(line_ending).and(eof),
    )(i)
}

fn cell_row(i: &str) -> IResult<&str, Vec<Cell>> {
    let alive = value(Cell::Alive, char('O'));
    let dead = value(Cell::Dead, char('.'));
    let cell = alt((alive, dead));
    many1(cell)(i)
}

fn empty_cell_row(i: &str) -> IResult<&str, Vec<Cell>> {
    value(Vec::new(), peek(line_ending))(i)
}

fn bang_comment(i: &str) -> IResult<&str, ()> {
    value((), pair(char('!'), not_line_ending))(i)
}

#[cfg(test)]
mod test {
    use super::*;

    const GLIDER: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/glider.cells"));

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
