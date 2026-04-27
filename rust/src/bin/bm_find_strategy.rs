use ahash::{HashMap, HashMapExt};
use segment_colouring::simple_state::{find::Visited, state::State};

fn main() {
    let state = State::<4>::new();
    let mut map = HashMap::new();
    let result = state.find_strategy(&mut map, 50, 11);
    println!("{:?}", result);
    if result {
        let mut file = std::fs::File::create("out.txt").unwrap();
        state.print_strategy(&map, &mut file);
    }
    println!("Visited states: {}", map.len());
    println!(
        "Winning states {}",
        map.values()
            .filter(|v| if let Visited::Winning(_) = v {
                true
            } else {
                false
            })
            .count()
    )
}
