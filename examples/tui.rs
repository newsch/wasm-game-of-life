use std::error::Error;
use std::io::{stdin, Read};
use std::thread::sleep;
use std::time::Duration;

use wasm_game_of_life::{parse_plaintext, Grid, Universe};

fn main() -> Result<(), Box<dyn Error>> {
    let mut contents = String::new();
    stdin().read_to_string(&mut contents)?;
    let Grid {
        width,
        height,
        cells,
    } = parse_plaintext(&contents).map_err(|e| e.to_string())?;
    let mut universe = Universe::of_cells(width.try_into()?, height.try_into()?, cells);

    for _ in 0..10 {
        println!("{}", universe);
        universe.tick();
        sleep(Duration::from_millis(500));
    }

    Ok(())
}
