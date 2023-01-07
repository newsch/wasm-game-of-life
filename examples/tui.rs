use std::error::Error;
use std::io::{stdin, Read};
use std::thread::sleep;
use std::time::Duration;

use wasm_game_of_life::{parse_str, Grid, Universe};

fn main() -> Result<(), Box<dyn Error>> {
    let mut contents = String::new();
    stdin().read_to_string(&mut contents)?;
    let Grid {
        width,
        height,
        cells,
    } = parse_str(&contents).map_err(|e| {
        eprintln!("{}", e);
        e
    })?;
    let mut universe = Universe::of_cells(width.try_into()?, height.try_into()?, cells);

    for _ in 0..1 {
        println!("{}", universe);
        universe.tick();
        sleep(Duration::from_millis(500));
    }

    Ok(())
}
