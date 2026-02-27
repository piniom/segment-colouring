use segment_colouring::simple_state::state::State;

fn main() {
    let state = State::<3>::new();
    let result = state.generate_all(10);
    println!("{}", result.len());
}
