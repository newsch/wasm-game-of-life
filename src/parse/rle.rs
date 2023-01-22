//! Parsing of run length encoded (.cells) gol files
//!
//! Based on information from <https://conwaylife.com/wiki/Run_Length_Encoded>.

use super::*;

use std::{str::FromStr};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{
        alphanumeric1, char, digit1, line_ending, multispace0, not_line_ending, space0,
    },
    combinator::{map, map_res, opt, peek, rest, value},
    error::{context, ErrorKind, FromExternalError},
    multi::{many0, separated_list0},
    sequence::{pair, preceded, separated_pair, terminated, tuple},
    Finish, Parser,
};

pub struct RleParser();

const FILE_EXTENSIONS: &'static [&'static str] = &["rle"];

mod tags {
    pub const DEAD: char = 'b';
    pub const ALIVE: char = 'o';
    pub const EOL: char = '$';
}

impl LifeParser for RleParser {
    fn file_extensions(&self) -> &[&str] {
        FILE_EXTENSIONS
    }

    fn sniff(&self, input: &str) -> bool {
        input.starts_with(['#', 'x'])
    }

    fn parse(&self, input: &str) -> Result<Grid, ParseError> {
        parse_rle(input)
    }
}

pub fn parse_rle(input: &str) -> Result<Grid, ParseError> {
    let (_rest, rle) = context("rle", rle)(input)
        .finish()
        .map_err(|e| ParseError::new(e, input))?;
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
    tag_seq: Vec<(usize, char)>,
}

impl TryInto<Grid> for Rle {
    type Error = ParseError;

    fn try_into(self) -> Result<Grid, Self::Error> {
        let Rle {
            width,
            height,
            tag_seq,
        } = self;

        // TODO: turn panics into errors or handle softly...
        // TODO: validate tag seq to size?

        let mut cells = Vec::with_capacity(width * height);

        for (i, tag) in tag_seq {
            let state = match tag {
                tags::EOL => {
                    // TODO: if already at end of line, skip?
                    for i in 0..i {
                        // fill to end of line
                        let row = cells.len() % width;
                        if i == 0 && row == 0 {
                            // already at end of line
                            continue;
                        }
                        for _ in 0..width - row {
                            cells.push(Cell::Dead);
                        }
                    }
                    continue;
                }
                tags::ALIVE => Cell::Alive,
                tags::DEAD => Cell::Dead,
                t => panic!("unexpected tag {t:?}"),
            };

            for _ in 0..i {
                cells.push(state);
            }
        }

        // fill remaining rows
        for _ in cells.len()..(width * height) {
            cells.push(Cell::Dead);
        }

        assert_eq!(cells.len(), (width * height));

        Ok(Grid {
            width,
            height,
            cells,
        })
    }
}

fn rle(i: &str) -> VIResult<&str, Rle> {
    let (i, _comments) = context(
        "comments",
        terminated(separated_list0(line_ending, hash_comment), line_ending),
    )(i)?;
    let (i, (width, height, _rule_set)) = context("header", terminated(header, line_ending))(i)?;
    let (i, tag_seq) = context("cells", cells)(i)?;
    let (i, _trailing_comments) =
        context("trailing comments", opt(preceded(line_ending, rest)))(i)?;

    let rle = Rle {
        width,
        height,
        tag_seq,
    };
    Ok((i, rle))
}

fn hash_comment(i: &str) -> VIResult<&str, ()> {
    context("comment", value((), pair(char('#'), opt(not_line_ending))))(i)
}

fn header(i: &str) -> VIResult<&str, (usize, usize, Option<RuleSet>)> {
    let width = context("width", map_res(kv(char('x'), digit1), usize::from_str));
    let height = context("height", map_res(kv(char('y'), digit1), usize::from_str));
    let rule = context("rule_set", kv(tag("rule"), rule_set));

    let (i, (w, h)) = separated_pair(width, char(','), height)(i)?;
    let (i, r) = opt(preceded(char(','), rule))(i)?;
    let (i, _) = peek(line_ending)(i)?;

    Ok((i, (w, h, r)))
}

fn rle_tag(i: &str) -> VIResult<&str, char> {
    let alive = char(tags::ALIVE);
    let dead = char(tags::DEAD);
    let eol = char(tags::EOL);
    context("rle tag", alt((alive, dead, eol)))(i)
}

fn cells(i: &str) -> VIResult<&str, Vec<(usize, char)>> {
    let end_cells = context("cell end", char('!'));

    let count = context("count", map_res(digit1, usize::from_str));

    let repeated_cell = count.and(rle_tag);
    let single_cell = map(rle_tag, |v| (1, v));

    let tags = context(
        "rle tags",
        many0(preceded(multispace0, repeated_cell.or(single_cell))),
    );

    let mut lines = terminated(tags, preceded(multispace0, end_cells));

    lines(i)
}

fn rule_set(i: &str) -> VIResult<&str, RuleSet> {
    context(
        "rule",
        separated_pair(alphanumeric1, char('/'), alphanumeric1),
    )(i)
}

/// ` {key} = {value} ` with whitespace handling, returning `value`
fn kv<'a, K, V, KO, VO>(key: K, value: V) -> impl FnMut(&'a str) -> VIResult<&'a str, VO>
where
    K: FnMut(&'a str) -> VIResult<&'a str, KO>,
    V: FnMut(&'a str) -> VIResult<&'a str, VO>,
{
    context(
        "key-value",
        preceded(tuple((space0, key, space0, char('='), space0)), value),
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse() {
        let input = //"#N p43 glider loop
// #O Mike Playle
// #C A period-43 oscillator based on a stable reflector
// #C Discovered on 25 Apr 2013
// #C www.conwaylife.com/wiki/P43_glider_loop
"x = 65, y = 65, rule = B3/S23
27b2o$27bobo$29bo4b2o$25b4ob2o2bo2bo$25bo2bo3bobob2o$28bobobobo$29b2ob!
";

        let result = parse_rle(input);
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_comment() {
        assert_eq!(
            Ok(("\nx = 3, y = 3\n", ())),
            hash_comment("# x = 1, y = 3\nx = 3, y = 3\n")
        );
    }

    #[test]
    fn test_header_no_rules() {
        let input = "x = 1, y = 3\nblah blah";
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

    #[test]
    fn test_cells() {
        use tags::*;
        let input = "2b2o$bobo!";
        let rest = "";
        let output = vec![
            (2, DEAD),
            (2, ALIVE),
            (1, EOL),
            (1, DEAD),
            (1, ALIVE),
            (1, DEAD),
            (1, ALIVE),
        ];
        assert_eq!(Ok((rest, output)), cells(input));
    }
}
