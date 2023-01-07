use log::trace;

mod utils;
use utils::Timer;
mod parse;
pub use parse::*;

#[cfg(feature = "wasm")]
use js_sys::TypeError;
#[cfg(feature = "wasm")]
use wasm_bindgen::{prelude::*, JsValue};

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

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgeBehavior {
    Wrap,
    Dead,
    Alive,
    // Grow,
}

// TODO: use fixedbitset for storing cells
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
    old_cells: Vec<Cell>,
    delta_alive: Vec<u32>,
    delta_dead: Vec<u32>,
    edge_behavior: EdgeBehavior,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
    console_log::init()
        // console_log::init_with_level(Level::Trace)
        .expect("error initializing log");
    log::info!("Hello from wasm!");
}

/// public methods for JS
#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl Universe {
    // TODO: random generation of initial layout with js-sys
    pub fn new(width: u32, height: u32) -> Universe {
        let cells = vec![Cell::Dead; (width * height) as usize];
        Self::of_cells(width, height, cells)
    }

    #[cfg(feature = "wasm")]
    pub fn reset_from_file(&mut self, f: &[u8]) -> Result<(), JsValue> {
        *self = Self::of_file(f).map_err(|e| TypeError::new(e.to_string().as_ref()))?;
        Ok(())
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
        {
            let _timer = Timer::new("new generation");
            for row in 0..self.height {
                for col in 0..self.width {
                    let idx = self.get_index(row, col);
                    let cell = self.cells[idx];
                    let live_neighbors = self.live_neighbor_count(row, col);

                    trace!("cell[{row}, {col}] is initially {cell:?} and has {live_neighbors} live neighbors");
                    let next_cell = Self::update_status(cell, live_neighbors);
                    trace!("it becomes {next_cell:?}");

                    self.old_cells[idx] = next_cell;
                }
            }
        }

        let _timer2 = Timer::new("swap cell buffers");
        mem::swap(&mut self.old_cells, &mut self.cells);
    }

    /// Updates the Universe, bringing cells into and out of existence.
    pub fn tick_delta(&mut self) {
        let _timer = Timer::new("Universe::tick_delta");
        self.delta_alive.clear();
        self.delta_dead.clear();
        {
            let _timer = Timer::new("new generation");
            for row in 0..self.height {
                for col in 0..self.width {
                    let idx = self.get_index(row, col);
                    let cell = self.cells[idx];
                    let live_neighbors = self.live_neighbor_count(row, col);

                    trace!("cell[{row}, {col}] is initially {cell:?} and has {live_neighbors} live neighbors");
                    let next_cell = Self::update_status(cell, live_neighbors);
                    trace!("it becomes {next_cell:?}");

                    if cell != next_cell {
                        self.buffer_delta(row, col, next_cell);
                    }
                    self.old_cells[idx] = next_cell;
                }
            }
        }
        mem::swap(&mut self.old_cells, &mut self.cells);
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen(getter))]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen(getter))]
    pub fn height(&self) -> u32 {
        self.height
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen(setter))]
    /// Set the width of the universe.
    ///
    /// Resets all cells to the dead state.
    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        self.make_cells();
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen(setter))]
    /// Set the height of the universe.
    ///
    /// Resets all cells to the dead state.
    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        self.make_cells();
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen(getter))]
    pub fn edge_behavior(&self) -> EdgeBehavior {
        self.edge_behavior
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen(setter))]
    pub fn set_edge_behavior(&mut self, edge_behavior: EdgeBehavior) {
        self.edge_behavior = edge_behavior;
    }

    /// Returns a pointer to the cells buffer.
    ///
    /// Cells are laid out as a linear stack of rows.
    /// Mapping from (row, col) coordinates to an index into the linear stack
    /// can be done with: `idx = (row_num * width + col_num)`
    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }

    pub fn cells_born(&self) -> *const u32 {
        self.delta_alive.as_ptr()
    }

    pub fn cells_died(&self) -> *const u32 {
        self.delta_dead.as_ptr()
    }

    pub fn cells_born_count(&self) -> usize {
        self.delta_alive.len() / 2
    }

    pub fn cells_died_count(&self) -> usize {
        self.delta_dead.len() / 2
    }

    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells[idx].toggle();
        self.buffer_delta(row, column, self.cells[idx]);
    }
}

/// non-JS-exported methods
impl Universe {
    pub fn of_cells(width: u32, height: u32, cells: Vec<Cell>) -> Universe {
        assert_eq!(cells.len(), (width * height) as usize);
        let old_cells = cells.clone();

        Universe {
            width,
            height,
            cells,
            old_cells,
            delta_alive: Vec::new(),
            delta_dead: Vec::new(),
            edge_behavior: EdgeBehavior::Wrap,
        }
    }

    fn of_grid(
        Grid {
            width,
            height,
            cells,
        }: Grid,
    ) -> Result<Self, Box<dyn Error>> {
        Ok(Self::of_cells(width.try_into()?, height.try_into()?, cells))
    }

    pub fn of_file(f: &[u8]) -> Result<Self, Box<dyn Error>> {
        let f = std::str::from_utf8(f)?;
        let grid = parse_str(f)?;
        Self::of_grid(grid)
    }

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

    fn get_rowcol(&self, index: usize) -> (u32, u32) {
        debug_assert!(index < self.cells.len());
        let row = index / self.cells.len();
        let col = index % self.cells.len();
        (row as u32, col as u32)
    }

    /// Sets the cells buffer to match height and width dimensions.
    ///
    /// Resets all cells to the dead state.
    #[inline]
    fn make_cells(&mut self) {
        self.cells = (0..self.width * self.height).map(|_i| Cell::Dead).collect();
        self.old_cells = self.cells.clone();
    }

    fn live_neighbor_count(&self, row: u32, col: u32) -> u8 {
        match self.edge_behavior {
            EdgeBehavior::Wrap => self.live_neighbor_count_wrapping(row, col),
            EdgeBehavior::Dead => self.live_neighbor_count_fixed(row, col, Cell::Dead),
            EdgeBehavior::Alive => self.live_neighbor_count_fixed(row, col, Cell::Alive),
        }
    }

    fn live_neighbor_count_wrapping(&self, row: u32, col: u32) -> u8 {
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

    fn live_neighbor_count_fixed(&self, row: u32, col: u32, boundary: Cell) -> u8 {
        let north = (row != 0).then_some(row - 1);
        let south = (row != self.height - 1).then_some(row + 1);
        let west = (col != 0).then_some(col - 1);
        let east = (col != self.width - 1).then_some(col + 1);

        let neighbors = [
            (north, west),
            (north, Some(col)),
            (north, east),
            (Some(row), west),
            (Some(row), east),
            (south, west),
            (south, Some(col)),
            (south, east),
        ];

        let count = neighbors
            .into_iter()
            .map(|pair| match pair {
                (Some(r), Some(c)) => self.cells[self.get_index(r, c)] as u8,
                _ => boundary as u8,
            })
            .sum();

        return count;
    }

    fn update_status(cell: Cell, live_neighbors: u8) -> Cell {
        match (cell, live_neighbors) {
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
        }
    }

    fn buffer_delta(&mut self, row: u32, col: u32, cell: Cell) {
        let delta_buffer = match cell {
            Cell::Alive => &mut self.delta_alive,
            Cell::Dead => &mut self.delta_dead,
        };
        delta_buffer.push(row);
        delta_buffer.push(col);
    }
}

use std::{error::Error, fmt, mem};

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
