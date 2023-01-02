mod utils;
use utils::Timer;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wasm")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

impl Cell {
    fn toggle(&mut self) {
        *self = match *self {
            Cell::Dead => Cell::Alive,
            Cell::Alive => Cell::Dead,
        }
    }
}

// TODO: use fixedbitset for storing cells
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
}

/// public methods for JS
#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl Universe {
    // TODO: random generation of initial layout with js-sys
    pub fn new(width: u32, height: u32) -> Universe {
        #[cfg(feature = "wasm")]
        utils::set_panic_hook();

        let cells = vec![Cell::Dead; (width * height) as usize];

        Universe {
            width,
            height,
            cells,
        }
    }

    pub fn reset_blank(&mut self) {
        for i in 0..(self.height * self.width) as usize {
            self.cells[i] = Cell::Dead;
        }
    }

    pub fn reset_fancy(&mut self) {
        for i in 0..(self.height * self.width) as usize {
            self.cells[i] = if i % 2 == 0 || i % 7 == 0 {
                Cell::Alive
            } else {
                Cell::Dead
            };
        }
    }

    #[cfg(feature = "wasm")]
    pub fn reset_random(&mut self) {
        for i in 0..(self.height * self.width) as usize {
            self.cells[i] = if js_sys::Math::random() < 0.5 {
                Cell::Alive
            } else {
                Cell::Dead
            };
        }
    }

    /// Returns a Unicode grid in a string, representing the Universe.
    pub fn render(&self) -> String {
        self.to_string()
    }

    /// Updates the Universe, bringing cells into and out of existence.
    pub fn tick(&mut self) {
        let _timer = Timer::new("Universe::tick");
        let mut next = {
            let _timer = Timer::new("allocate next cells");
            self.cells.clone()
        }; // future universe

        {
            let _timer = Timer::new("new generation");
            for row in 0..self.height {
                for col in 0..self.width {
                    let idx = self.get_index(row, col);
                    let cell = self.cells[idx];
                    let live_neighbors = self.live_neighbor_count(row, col);

                    // log!("cell[{row}, {col}] is initially {cell:?} and has {live_neighbors} live neighbors");

                    let next_cell = match (cell, live_neighbors) {
                        // Rule 1: Any live cell with fewer than two neighbors dies.
                        (Cell::Alive, x) if x < 2 => Cell::Dead,
                        // Rule 2: Any live cell with two or three live neighbours
                        // lives on to the next generation.
                        (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                        // Rule 3: Any live cell with more than three neighbours
                        // dies.
                        (Cell::Alive, x) if x > 3 => Cell::Dead,
                        // Rule 4: Any dead cell with exactly three neighbours
                        // becomes a live cell.
                        (Cell::Dead, 3) => Cell::Alive,
                        // All other cells remain in the same state.
                        (otherwise, _) => otherwise,
                    };

                    // log!("it becomes {next_cell:?}");

                    next[idx] = next_cell;
                }
            }
        }

        let _timer2 = Timer::new("free old cells");
        self.cells = next;
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Set the width of the universe.
    ///
    /// Resets all cells to the dead state.
    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        self.make_cells();
    }

    /// Set the height of the universe.
    ///
    /// Resets all cells to the dead state.
    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        self.make_cells();
    }

    /// Returns a pointer to the cells buffer.
    ///
    /// Cells are laid out as a linear stack of rows.
    /// Mapping from (row, col) coordinates to an index into the linear stack
    /// can be done with: `idx = (row_num * width + col_num)`
    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }

    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells[idx].toggle();
    }
}

/// non-JS-exported methods
impl Universe {
    /// Get the dead and alive values of the entire universe.
    pub fn get_cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Set cells to be alive in a universe by passing the row and column
    /// of each cell as an array.
    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells[idx] = Cell::Alive;
        }
    }

    fn get_index(&self, row: u32, col: u32) -> usize {
        debug_assert!(row < self.height);
        debug_assert!(col < self.width);
        (row * self.width + col) as usize
    }

    /// Sets the cells buffer to match height and width dimensions.
    ///
    /// Resets all cells to the dead state.
    #[inline]
    fn make_cells(&mut self) {
        self.cells = (0..self.width * self.height).map(|_i| Cell::Dead).collect();
    }

    fn live_neighbor_count(&self, row: u32, col: u32) -> u8 {
        let north = if row == 0 { self.height - 1 } else { row - 1 };

        let south = if row == self.height - 1 { 0 } else { row + 1 };

        let west = if col == 0 { self.width - 1 } else { col - 1 };

        let east = if col == self.width - 1 { 0 } else { col + 1 };

        let neighbors = [
            (north, west),
            (north, col),
            (north, east),
            (row, west),
            (row, east),
            (south, west),
            (south, col),
            (south, east),
        ];

        let count = neighbors
            .into_iter()
            .map(|(r, c)| self.cells[self.get_index(r, c)] as u8)
            .sum();

        return count;
    }
}

use std::fmt;

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == Cell::Dead { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}
