use std::{cell::RefCell, collections::HashMap, hash::Hash, io::Write};

use super::{
    clicqued::ClicquedLinearAxis, event::Event, history::History, normalization::NormalizedState,
    LinearAxis,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategyMove {
    Insert { start: usize, end: usize },
    LimitFront,
    LimitBack,
}

impl StrategyMove {
    pub fn string(&self, offset: usize) -> String {
        match self {
            StrategyMove::Insert { start, end } => format!("{} {}", start + offset, end + offset),
            StrategyMove::LimitFront => ">".to_string(),
            StrategyMove::LimitBack => "<".to_string(),
        }
    }
    pub fn history(&self) -> Option<History> {
        match self {
            Self::LimitBack => Some(History::LimitBack),
            Self::LimitFront => Some(History::LimitFront),
            _ => None,
        }
    }
}

impl History {
    pub fn strategy_move(&self) -> Option<StrategyMove> {
        match self {
            History::LimitFront => Some(StrategyMove::LimitFront),
            History::LimitBack => Some(StrategyMove::LimitBack),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct StrategyState {
    front: Vec<Event>,
    actual: Vec<Event>,
    back: Vec<Event>,
}

impl StrategyState {
    pub fn without_boundaries(&self) -> Vec<Event> {
        [&self.front, &self.actual, &self.back]
            .into_iter()
            .flatten()
            .copied()
            .collect()
    }
}

// impl Hash for StrategyState {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         self.without_boundaries().hash(state);
//     }
// }

impl StrategyState {
    pub fn from(&NormalizedState(ref state): &NormalizedState, max_colors: usize) -> Self {
        let mut started = vec![false; max_colors];
        let mut front = vec![];
        for e in state {
            if e.is_start() {
                started[e.colour() as usize] = true;
            } else if !started[e.colour() as usize] {
                started[e.colour() as usize] = true;
                front.push(e.sibling());
            }
        }
        let mut finished = vec![false; max_colors];
        let mut back = vec![];
        for e in state.iter().rev() {
            if !e.is_start() {
                finished[e.colour() as usize] = true;
            } else if !finished[e.colour() as usize] {
                finished[e.colour() as usize] = true;
                back.push(e.sibling());
            }
        }
        back.reverse();
        Self {
            front,
            actual: state.clone(),
            back,
        }
    }
    pub fn to_string(&self) -> String {
        self.front.iter().map(Event::to_char).collect::<String>()
            + "["
            + &self.actual.iter().map(Event::to_char).collect::<String>()
            + "]"
            + &self.back.iter().map(Event::to_char).collect::<String>()
    }
}

#[derive()]
pub struct StrategyConsumer {
    moves: HashMap<StrategyState, StrategyMove>,
    max_colors: usize,
    wt: RefCell<Box<dyn Write>>,
}

impl StrategyConsumer {
    pub fn new(
        max_colors: usize,
        clicque_size: usize,
        force_colors: usize,
        mut wt: Box<dyn Write>,
    ) -> Self {
        writeln!(wt, "{} {}", clicque_size, force_colors).unwrap();
        Self {
            moves: HashMap::new(),
            max_colors,
            wt: RefCell::new(wt),
        }
    }
    pub fn consume(&mut self, state: &NormalizedState, mov: StrategyMove) {
        let state = StrategyState::from(state, self.max_colors);
        if state.to_string() == "A[BCabAcDaC]dc" {
            println!("{mov:?}")
        }
        writeln!(
            self.wt.borrow_mut(),
            "{} {}",
            state.to_string(),
            mov.string(state.front.len())
        )
        .unwrap();
        self.moves.insert(state, mov);
    }
}

impl LinearAxis {
    pub fn from_strategy_string(string: &str) -> Self {
        let [f, eb, ..] = string.split('[').collect::<Vec<&str>>()[..] else {
            panic!()
        };
        let [e, b, ..] = eb.split(']').collect::<Vec<&str>>()[..] else {
            panic!()
        };
        Self {
            events: e.chars().map(Event::from_char).collect(),
            front: f.chars().map(Event::from_char).collect(),
            back: b.chars().map(Event::from_char).collect(),
        }
    }
    pub fn from_strategy_state(state: StrategyState) -> Self {
        Self {
            events: state.actual.into(),
            front: state.front.into(),
            back: state.back.into(),
        }
    }
}

impl ClicquedLinearAxis {
    pub fn from_strategy_string(string: &str, max_clicque: usize) -> Self {
        Self::with_inner(LinearAxis::from_strategy_string(string), max_clicque)
    }
    pub fn from_strategy_state(state: StrategyState, max_clicque: usize) -> Self {
        Self::with_inner(LinearAxis::from_strategy_state(state), max_clicque)
    }
}

#[test]
fn from_str_test() {
    let mut axis = ClicquedLinearAxis::from_strategy_string("[AaAaAaAa]", 5);
    println!("{:?}", axis.uncollisions(0, 1));
    let strategy = StrategyState::from(&axis.strategy_normalize_without_symmetry().flipped(9), 9);
    println!("{:?}", strategy.to_string());
    axis.apply_history(History::LimitBack);
    // println!("{}", StrategyState::from(&axis.strategy_normalize().0, 5).to_string())
}

#[test]
fn from_str_test_2() {
    let mov = (2, 6);
    let mut axis = ClicquedLinearAxis::from_strategy_string("A[BCabAcDaC]dc", 3);
    assert!(axis.valid_new_segments().contains(&mov));
    println!("{:?}", axis.uncollisions(mov.0, mov.1));
    println!("{}", axis.inner.to_string());
    axis.apply_history(History::SegmentInsert {
        start_index: mov.0,
        end_index: mov.1,
        color: 4,
    });
    println!("{}", axis.inner.to_string());
    // println!("{}", StrategyState::from(&axis.strategy_normalize().0, 5).to_string())
}

// Answer for ( [ABaCbB]cb, 0-1, C ): [ABaCbAcC]ac not found in the strategy.

// Answer for ( AB[CabBcACbac] <  ): AB[CabBcA]ba not found in the strategy.
