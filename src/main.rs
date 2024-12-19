// use segment_colouring::first_fit::FirstFitColourer;

// fn main() {
//     let mut ff = FirstFitColourer::default();
//     let inputs = [(0, 0), (0, 0), (1, 3), (5, 6), (2, 6)];
//     // let inputs = [(0, 0), (0, 0), (1, 3), (2, 5), (6, 8)];
//     for (s, e) in inputs {
//         ff.insert_segment(s, e).unwrap();
//         println!("{}\n", ff.to_string())
//     }
// }

use segment_colouring::game::Game;

use std::thread;

const STACK_SIZE: usize = 1024 * 1024 * 1024;

fn run() {
    let mut game = Game::new(8, 4, 7);
    dbg!(game.simulate());
}

fn main() {
    let child = thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(run)
        .unwrap();

    child.join().unwrap();
}
