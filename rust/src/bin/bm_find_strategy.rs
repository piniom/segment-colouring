use ahash::{HashMap, HashMapExt};
use segment_colouring::simple_state::state::State;

fn main() {
    let state = State::<2>::new();
    let mut map = HashMap::new();
    let result = state.find_strategy(&mut map, 4);
    println!("{:?}", result);
    if result {
        let mut file = std::fs::File::create("out.txt").unwrap();
        state.print_strategy(&map, &mut file);
    }
}