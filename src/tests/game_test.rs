use crate::game::Game;

#[test]
fn game_one_segment_test() {
    assert!(Game::new(1, 1, 1).simulate());
    assert!(Game::new(1, 20, 1).simulate());
    assert!(Game::new(20, 1, 1).simulate());
}

#[test]
fn game_just_stack_test() {
    assert!(Game::new(2, 2, 2).simulate());
    assert!(Game::new(3, 3, 3).simulate());
    assert!(Game::new(4, 4, 4).simulate());
    assert!(Game::new(5, 5, 5).simulate());
    assert!(Game::new(6, 6, 6).simulate());
}

#[test]
fn game_impossible_simple_clicque_test() {
    assert!(!Game::new(2, 1, 2).simulate());
    assert!(!Game::new(4, 1, 4).simulate());
}

#[test]
fn game_impossible_to_few_segments_test() {
    assert!(!Game::new(1, 6, 2).simulate());
    assert!(!Game::new(3, 4, 4).simulate());
    assert!(!Game::new(2, 3, 3).simulate());
}

#[test]
fn game_2_3_test() {
    assert!(Game::new(4, 2, 3).simulate());

    let mut game = Game::new(3, 2, 3);
    game.insert_coloured_segment(0, 0, 0);
    game.insert_coloured_segment(2, 2, 0);
    println!("{}", game.to_string());
    game.insert_coloured_segment(1, 2, 1);
    assert!(game.simulate());
    assert!(Game::new(3, 2, 3).simulate());
}

#[test]
fn game_3_5_test() {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024) // 16 MB
        .spawn(|| {
            assert!(Game::new(6, 3, 5).simulate());
        })
        .unwrap()
        .join()
        .unwrap();
    
}