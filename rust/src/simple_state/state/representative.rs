use super::State;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Representative<const MAX_CLIQUE: u32>(pub State<MAX_CLIQUE>);

impl<const MAX_CLIQUE: u32> Representative<MAX_CLIQUE> {
    pub fn new(mut state: State<MAX_CLIQUE>) -> Self {
        state.set_limit_front(0);
        state.set_limit_back(state.len());
        Self(state)
    }
}
