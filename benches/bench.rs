#![feature(test)]

extern crate test;
extern crate wasm_game_of_life;

#[bench]
fn universe_ticks(b: &mut test::Bencher) {
    let mut universe = wasm_game_of_life::Universe::new(64, 64);
    universe.reset_fancy();

    b.iter(|| {
        universe.tick();
    });
}
