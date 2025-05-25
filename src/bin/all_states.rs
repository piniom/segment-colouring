use rand::{seq::SliceRandom, Rng};
use segment_colouring::linear_axis::{clicqued::ClicquedLinearAxis, event::Event, game::{self, Game}, normalization::NormalizedState, strategy::StrategyState};

const MAX_COLORS: usize = 9;
const MAX_CLICQUE: usize = 5;

#[tokio::main]
async fn main() {
    // test().await;
    let mut axis = ClicquedLinearAxis::new(5);
    let states = axis.generate_all_states_async(9, 6).await;
    let mut agr = vec![vec![]; 10];
    for s in states {
        agr[s.colors_used()].push(s);
    }
    println!("Generated states!");
    let mut rng = rand::rng();
    let mut select = agr[7].clone();
    select.shuffle(&mut rng);
    dbg!(select.len());
    run_parallel_loop(&select).await;
}

async fn process_state(index: usize, state: NormalizedState) -> (bool, usize) {
    let mut sum = 0;
    let mut axis = ClicquedLinearAxis::from_strategy_state(StrategyState::from(&state, MAX_COLORS), MAX_CLICQUE);
    // let result = Game::with_axis(axis, 24, MAX_COLORS, None).simulate(3);
    let result = axis.check_if_winning(2, &mut sum);
    (result, sum)
}

async fn run_parallel_loop(select: &[NormalizedState]) {
    let mut futures = Vec::new();

    for (index, state) in select.iter().enumerate() {
        let state_clone = state.clone();
        futures.push(tokio::spawn(async move {
            process_state(index, state_clone).await
        }));
    }
    
    let mut successes = 0;
    let mut failures = 0;

    for future in futures {
        let (result, sum) = future.await.unwrap();
        if result {
            successes += 1;
        } else {
            failures += 1;
        }
        let total = successes + failures;
        if total % 100 == 0 || total < 20 {
            println!("Total  : {}", total);
            println!("Succ   : {}", successes);
            println!("Fail   : {}", failures);
            println!("Cursum : {}", sum);
            println!()
        }
    }
    println!("\n\n----------\n\n");
    println!("Total: {}", select.len());
    println!("Succ : {}", successes);
    println!("Fail : {}", failures)
}

async fn test() {
    let state = vec![
        Event::new_start(0),
        Event::new_start(1),
        Event::new_start(2),
        Event::new_start(3),
        Event::new_end(0),
        Event::new_end(1),
        Event::new_end(2),
        Event::new_end(3),
        Event::new_start(4),
        Event::new_start(5),
        Event::new_start(6),
        Event::new_start(7),
        Event::new_end(4),
        Event::new_end(5),
        Event::new_end(6),
        Event::new_end(7),
    ];
    dbg!(process_state(0, NormalizedState(state)).await);
}