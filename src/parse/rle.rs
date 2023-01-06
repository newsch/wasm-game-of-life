//! Parsing of run length encoded (.cells) gol files
//!
//! Based on information from <https://conwaylife.com/wiki/Run_Length_Encoded>.

use super::*;

use std::{iter::repeat, str::FromStr};

use nom::{
    branch::alt,
    character::complete::{
        alphanumeric1, char, digit1, line_ending, multispace0, not_line_ending, space0,
    },
    combinator::{map, map_res, opt, peek, rest, value},
    error::{context, ErrorKind, FromExternalError},
    multi::{separated_list0, separated_list1},
    sequence::{pair, preceded, separated_pair, terminated, tuple},
    Finish, Parser,
};

pub struct RleParser();

    const FILE_EXTENSIONS: &'static [&'static str] = &["rle"];

impl LifeParser for RleParser {

    fn file_extensions(&self) -> &[&str] {
        FILE_EXTENSIONS
    }

    fn sniff(&self, input: &str) -> bool {
        input.starts_with(['!', '.', 'O'])
    }

    fn parse(&self, input: &str) -> Result<Grid, ParseError> {
        parse_rle(input)
    }
}

pub fn parse_rle(input: &str) -> Result<Grid, ParseError> {
    let (_rest, rle) = rle(input).finish().map_err(|e| ParseError::new(e, input))?;
    let grid = rle
        .try_into()
        .map_err(|e| VerboseError::from_external_error(input, ErrorKind::Fail, e))
        .map_err(|e| ParseError::new(e, input))?;
    Ok(grid)
}

type RuleSet<'a> = (&'a str, &'a str);

struct Rle {
    width: usize,
    height: usize,
    cell_seq: Vec<Vec<(usize, Cell)>>,
}

impl TryInto<Grid> for Rle {
    type Error = ParseError;

    fn try_into(self) -> Result<Grid, Self::Error> {
        let Rle {
            width,
            height,
            cell_seq,
        } = self;

        // TODO: turn into error or handle softly...
        assert!(cell_seq.len() <= height);

        let mut cells = Vec::with_capacity(width * height);

        for line in cell_seq {
            let start = cells.len();
            for (i, state) in line {
                for _ in 0..i {
                    cells.push(state);
                }
            }
            // fill to end of line
            assert!(cells.len() >= start);
            let pushed = cells.len() - start;
            assert!(width >= pushed);
            let rest = width - pushed;
            for _ in 0..rest {
                cells.push(Cell::Dead);
            }
        }

        // fill remaining rows
        for _ in cells.len()..(width * height) {
            cells.push(Cell::Dead);
        }

        Ok(Grid {
            width,
            height,
            cells,
        })
    }
}

fn rle(i: &str) -> VIResult<&str, Rle> {
    let (i, _comments) = context("comments", separated_list0(line_ending, hash_comment))(i)?;
    let (i, (width, height, _rule_set)) = terminated(header, line_ending)(i)?;
    let (i, cell_seq) = cells(i)?;
    let (i, _trailing_comments) = opt(preceded(line_ending, rest))(i)?;

    let rle = Rle {
        width,
        height,
        cell_seq,
    };
    Ok((i, rle))
}

fn hash_comment(i: &str) -> VIResult<&str, ()> {
    context("comment", value((), pair(char('#'), not_line_ending)))(i)
}

fn header(i: &str) -> VIResult<&str, (usize, usize, Option<RuleSet>)> {
    let width = map_res(kv('x', digit1), usize::from_str);
    let height = map_res(kv('y', digit1), usize::from_str);
    let rule = kv('r', rule_set);

    let (i, (w, h)) = separated_pair(width, char(','), height)(i)?;
    let (i, r) = opt(preceded(char(','), rule))(i)?;
    let (i, _) = peek(line_ending)(i)?;

    Ok((i, (w, h, r)))
}

fn cell(i: &str) -> VIResult<&str, Cell> {
    let alive = value(Cell::Alive, char('b'));
    let dead = value(Cell::Dead, char('o'));
    alt((alive, dead))(i)
}

fn cells(i: &str) -> VIResult<&str, Vec<Vec<(usize, Cell)>>> {
    let end_line = char('$');
    let end_cells = char('!');

    let count = map_res(digit1, usize::from_str);

    let repeated_cell = count.and(cell);
    let single_cell = map(cell, |v| (1, v));

    let line = separated_list0(multispace0, repeated_cell.or(single_cell));

    let mut lines = terminated(
        separated_list1(end_line, line),
        preceded(multispace0, end_cells),
    );

    lines(i)
}

fn rule_set(i: &str) -> VIResult<&str, RuleSet> {
    context(
        "rule",
        separated_pair(alphanumeric1, char('/'), alphanumeric1),
    )(i)
}

/// ` {key} = {value} ` with whitespace handling
fn kv<'a, F, O>(key: char, value: F) -> impl FnMut(&'a str) -> VIResult<&'a str, O>
where
    F: FnMut(&'a str) -> VIResult<&'a str, O>,
{
    preceded(tuple((space0, char(key), space0, char('='), space0)), value)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_comment() {
        assert_eq!(
            Ok(("\nx = 3, y = 3\n", ())),
            hash_comment("# x = 1, y = 3\nx = 3, y = 3\n")
        );
    }

    #[test]
    fn test_header_no_rules() {
        let input = " x = 1, y = 3\nblah blah";
        let rest = "\nblah blah";
        let output = (1, 3, None);
        assert_eq!(Ok((rest, output)), header(input));
    }

    #[test]
    fn test_header_rules() {
        let input = " x = 1, y = 3, r=3B/4a\nblah blah";
        let rest = "\nblah blah";
        let output = (1, 3, Some(("3B", "4a")));
        assert_eq!(Ok((rest, output)), header(input));
    }
}
