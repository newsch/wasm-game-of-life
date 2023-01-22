use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

use nom::{
    error::{convert_error, VerboseError},
    Err,
};

use crate::Cell;

/// Wrapper to include files from the root patterns/ directory
macro_rules! include_pattern {
    ($pattern_file:expr) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/patterns/",
            $pattern_file,
        ))
    };
}

mod plaintext;
pub use plaintext::*;
mod rle;
pub use rle::*;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Immutable pattern storage
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

trait LifeParser {
    /// File extensions that may indicate the parser can be used
    // const MIME_TYPES: [&'static str];
    fn file_extensions(&self) -> &[&str];

    /// Quick, low-overhead method to detect if the file is the format
    ///
    /// Must return `true` for any input for which [`parse`] returns `Ok(_)`.
    // TODO: decide on goals here
    fn sniff(&self, input: &str) -> bool {
        self.parse(input).is_ok()
        // None
    }

    fn parse(&self, input: &str) -> Result<Grid, ParseError>;
}

const PARSERS: &[&'static dyn LifeParser] = &[&PlaintextParser(), &RleParser()];

pub fn parse_str(input: &str) -> Result<Grid, ParseError> {
    for parser in PARSERS {
        if parser.sniff(input) {
            return parser.parse(input);
        }
    }
    panic!() // TODO: error
}

/// Drop-in replacement to IResult that holds VerboseErrors
type VIResult<I, O, E = VerboseError<I>> = Result<(I, O), Err<E>>;

#[derive(Debug, Clone, PartialEq)]
/// Wrapper of VerboseError pretty printing
pub struct ParseError(String);

impl ParseError {
    fn new(error: VerboseError<&str>, source: &str) -> Self {
        // convert to owned string (.to_owned() doesn't seem to work?)
        Self(format!(
            "Error parsing input:\n{}",
            convert_error(source, error)
        ))
    }
}

impl std::error::Error for ParseError {}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
