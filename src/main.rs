use std::thread;
use std::time::Instant;

use clap::*;
use segment_colouring::linear_axis::game::Game;

/// This is to prevent stack overflow.
/// We currently clone the state of the game to be restored when another simulation branch is evaluated.
/// Due to possible reductions max tree depth may be quite large.
const STACK_SIZE: usize = 8 * 1024 * 1024 * 1024;

#[derive(Parser, Debug)]
#[command(
    name = "Segment colouring",
    about = "Simulates an on-line proper segment colouring game between a builder and a colouring algorithm. 
Try to force ANY colouring algorithm to use a desired number of colours without exceeding the max clicque size."
)]
struct Args {
    /// The simulation will "win" when a strategy is discovered
    /// forcing any colouring algorithm to use this number of colours
    desired_number_of_colours: usize,
    /// Max allowed clicque size.
    max_clicque: u32,
    /// The simulation will reduce the game when the <MAX_EVENTS> number of events is reached.
    /// The higher this number is the longer it takes to simulate but the chance of success is higher
    max_events: usize,
}

fn run(args: Args) {
    let mut game = Game::new(
        args.max_events,
        args.max_clicque,
        args.desired_number_of_colours,
    );
    let start = Instant::now();
    let result = game.simulate();
    let elapsed = start.elapsed();
    if result {
        println!("SUCCESS!");
        println!("It IS possible to force any colouring algorithm to use {} colours whithout creating a clicque larger than {}.", args.desired_number_of_colours, args.max_clicque);
        println!(
            "The simulation was confined to states with at most {} events.",
            args.max_events
        );
    } else {
        println!("FAILURE!");
        println!(
            "It is NOT possible to force any colouring algorithm to use {} colours whithout creating a clicque larger than {} when the simulation is confined to states with at most {} events.", 
            args.desired_number_of_colours,
            args.max_clicque,
            args.max_events
        )
    }
    println!(
        "\nThe simulation discovered {} states in {:?}.",
        game.number_of_states(),
        elapsed
    );
}

fn main() {
    let args = Args::parse();
    let child = thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(move || run(args))
        .unwrap();

    child.join().unwrap();
}
