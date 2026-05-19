use super::State;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct CompressedState<const MAX_SEGMENTS: usize> {
    data: [u8; MAX_SEGMENTS],
}

impl<const MAX_SEGMENTS: usize> CompressedState<MAX_SEGMENTS> {
    pub fn new<const MAX_CLIQUE: u32>(state: State<MAX_CLIQUE>) -> Self {
        Self::from(state)
    }

    pub(super) const fn empty() -> Self {
        Self {
            data: [0; MAX_SEGMENTS],
        }
    }

    #[inline(always)]
    pub fn len(&self) -> u8 {
        for byte_index in (0..MAX_SEGMENTS).rev() {
            let byte = self.data[byte_index];
            let high = byte >> 4;
            if high != 0 {
                return (byte_index * 2 + 2) as u8;
            }
            let low = byte & 0b1111;
            if low != 0 {
                return (byte_index * 2 + 1) as u8;
            }
        }
        0
    }

    #[inline(always)]
    pub fn data(&self) -> &[u8; MAX_SEGMENTS] {
        &self.data
    }

    #[inline(always)]
    pub fn data_mut(&mut self) -> &mut [u8; MAX_SEGMENTS] {
        &mut self.data
    }
}

impl<const MAX_CLIQUE: u32, const MAX_SEGMENTS: usize> From<State<MAX_CLIQUE>>
    for CompressedState<MAX_SEGMENTS>
{
    fn from(state: State<MAX_CLIQUE>) -> Self {
        let mut compressed = Self::empty();
        let bytes = state.data().to_le_bytes();
        let copy_len = MAX_SEGMENTS.min(bytes.len());
        compressed.data[..copy_len].copy_from_slice(&bytes[..copy_len]);
        compressed
    }
}

impl<const MAX_CLIQUE: u32, const MAX_SEGMENTS: usize> From<CompressedState<MAX_SEGMENTS>>
    for State<MAX_CLIQUE>
{
    fn from(compressed: CompressedState<MAX_SEGMENTS>) -> Self {
        let len = compressed.len();
        let mut state = State::new();
        let mut bytes = [0u8; 16];
        let copy_len = MAX_SEGMENTS.min(bytes.len());
        bytes[..copy_len].copy_from_slice(&compressed.data[..copy_len]);
        state.set_data(u128::from_le_bytes(bytes));
        state.set_len(len);
        state.set_limit_front(0);
        state.set_limit_back(len);
        state
    }
}
