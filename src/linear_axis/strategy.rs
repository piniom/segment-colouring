use std::{collections::HashMap, hash::Hash, io::Write};

use super::{event::Event, history::History, normalization::NormalizedState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategyMove {
    Insert { start: usize, end: usize },
    LimitFront,
    LimitBack,
}

impl StrategyMove {
    pub fn string(&self) -> String {
        match self {
            StrategyMove::Insert { start, end } => format!("{start} {end}"),
            StrategyMove::LimitFront => ">".to_string(),
            StrategyMove::LimitBack => "<".to_string(),
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

#[derive(Debug, Clone)]
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

impl PartialEq for StrategyState {
    fn eq(&self, other: &Self) -> bool {
        self.without_boundaries() == other.without_boundaries()
    }
}

impl Eq for StrategyState {}

impl Hash for StrategyState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.without_boundaries().hash(state);
    }
}

impl StrategyState {
    fn from(&NormalizedState(ref state): &NormalizedState, max_colors: usize) -> Self {
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

#[derive(Debug)]
pub struct StrategyConsumer {
    moves: HashMap<StrategyState, StrategyMove>,
    max_colors: usize,
    clicque_size: usize,
    force_colors: usize,
}

impl StrategyConsumer {
    pub fn new(max_colors: usize, clicque_size: usize, force_colors: usize) -> Self {
        Self {
            moves: HashMap::new(),
            max_colors,
            clicque_size,
            force_colors,
        }
    }
    pub fn consume(&mut self, state: &NormalizedState, mov: StrategyMove) {
        self.moves
            .insert(StrategyState::from(state, self.max_colors), mov);
    }
    pub fn write(&self, wt: &mut dyn Write) -> std::io::Result<()> {
        writeln!(wt, "{} {}", self.clicque_size, self.force_colors)?;
        for (s, m) in &self.moves {
            writeln!(wt, "{} {}", s.to_string(), m.string())?;
        }
        Ok(())
    }
}
