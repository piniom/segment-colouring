use std::{collections::HashMap, hash::Hash, io::Write};

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
            _ => None
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

#[derive(Debug)]
pub struct StrategyConsumer {
    moves: HashMap<StrategyState, Vec<StrategyMove>>,
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
        let state = StrategyState::from(state, self.max_colors);
        self.moves.entry(state).or_default().push(mov);
    }
    pub fn write(&self, wt: &mut dyn Write) -> std::io::Result<()> {
        writeln!(wt, "{} {}", self.clicque_size, self.force_colors)?;
        for (s, ms) in &self.moves {
            let (ins, lims) = ms.into_iter().partition::<Vec<&StrategyMove>, _>(|m|if let StrategyMove::Insert {..} = m {true} else {false});
            if !ins.is_empty() {
                writeln!(wt, "{} {}", s.to_string(), ins[0].string(s.front.len()))?;
                continue;
            } 
            let mut found = false;
            for m in &lims {
                let mut axis = ClicquedLinearAxis::from_strategy_state(s.clone(), self.clicque_size);
                axis.apply_history(m.history().unwrap()).unwrap();
                let norm = axis.strategy_normalize();
                let st = StrategyState::from(&norm.0, self.max_colors);
                if let Some(_) = self.moves.get(&st) {
                    writeln!(wt, "{} {}", s.to_string(), m.string(s.front.len()))?;
                    found = true;
                    break;
                }
            }
            if !found {
                println!("{}", s.to_string());
                writeln!(wt, "{} {}", s.to_string(), lims.last().unwrap().string(s.front.len()))?;
            }
        }
        Ok(())
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
        Self::with_inner(
            LinearAxis::from_strategy_string(string),
            max_clicque,
        )
    }
    pub fn from_strategy_state(state: StrategyState, max_clicque: usize) -> Self {
        Self::with_inner(
            LinearAxis::from_strategy_state(state),
            max_clicque,
        )
    }
}

#[test]
fn from_str_test() {
    let (norm, f) = ClicquedLinearAxis::from_strategy_string("AB[aCAbDcad]", 3).strategy_normalize();
    dbg!(f);
    println!("{}", StrategyState::from(&norm, 5).to_string())
}
