use super::State;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct CompressedState<const BYTE_SIZE: usize> {
    data: [u8; BYTE_SIZE],
}

impl<const BYTE_SIZE: usize> CompressedState<BYTE_SIZE> {
    pub fn new<const MAX_CLIQUE: u32>(state: State<MAX_CLIQUE>) -> Self {
        Self::from(state)
    }

    pub(super) const fn empty() -> Self {
        Self {
            data: [0; BYTE_SIZE],
        }
    }

    #[inline(always)]
    pub fn len(&self) -> u8 {
        let last_bit = last_one_bit(&self.data);
        match last_bit {
            None => 0,
            Some(bit_index) => count_events(&self.data, bit_index),
        }
    }
}

impl<const MAX_CLIQUE: u32, const BYTE_SIZE: usize> From<State<MAX_CLIQUE>>
    for CompressedState<BYTE_SIZE>
{
    fn from(state: State<MAX_CLIQUE>) -> Self {
        let mut compressed = Self::empty();
        let mut bit_index = 0usize;

        if state.len() == 0 {
            return compressed;
        }
        if BYTE_SIZE <= 8 {
            // Fast path: collect the codes in one word, most significant bit first, then
            // store its top bytes. Same layout as the bit-by-bit path below.
            let mut codes = 0u64;
            let mut bits = 0u32;
            let mut events = state.data() >> 4; // the first event is implicit
            for _ in 1..state.len() {
                let event = (events & 0b1111) as u64;
                events >>= 4;
                if event & 0b1000 == 0 {
                    codes = codes << 4 | event;
                    bits += 4;
                } else {
                    codes = codes << 1 | 1;
                    bits += 1;
                }
            }
            debug_assert!(bits as usize <= BYTE_SIZE * 8, "state does not fit the key");
            if bits > 0 {
                let aligned = (codes << (64 - bits)).to_be_bytes();
                compressed.data.copy_from_slice(&aligned[..BYTE_SIZE]);
            }
            return compressed;
        }

        for i in 0..(state.len() - 1) {
            // First segment is always 0b0000 so we can skip it
            let value = state.get_at_index(i + 1);
            if value & 0b1000 == 0 {
                bit_index = write_bits(&mut compressed.data, bit_index, value as u32, 4);
            } else {
                bit_index = write_bits(&mut compressed.data, bit_index, 1, 1);
            }
        }
        compressed
    }
}

impl<const MAX_CLIQUE: u32, const BYTE_SIZE: usize> From<CompressedState<BYTE_SIZE>>
    for State<MAX_CLIQUE>
{
    fn from(compressed: CompressedState<BYTE_SIZE>) -> Self {
        let len = compressed.len();
        let mut state = State::new();
        if len == 0 {
            return state;
        }
        // Insert the implicit segment
        state.insert_at_index(0, 0);
        let mut bit_index = 0usize;
        let mut start_events: Vec<u8> = vec![0];
        let mut end_index = 0usize;

        for _ in 0..len {
            let is_end = read_bit(&compressed.data, bit_index);
            if is_end {
                bit_index += 1;
                let value = start_events.get(end_index).copied().unwrap_or(0) | 0b1000;
                end_index += 1;
                state.insert_at_index(state.len(), value);
            } else {
                let (value, next) = read_bits(&compressed.data, bit_index, 4);
                bit_index = next;
                let value = value as u8;
                start_events.push(value);
                state.insert_at_index(state.len(), value);
            }
        }

        state.set_limit_front(0);
        state.set_limit_back(state.len());
        state
    }
}

#[inline(always)]
fn read_bit<const BYTE_SIZE: usize>(data: &[u8; BYTE_SIZE], bit_index: usize) -> bool {
    let byte_index = bit_index / 8;
    let bit_in_byte = 7 - (bit_index % 8);
    ((data[byte_index] >> bit_in_byte) & 1) == 1
}

#[inline(always)]
fn read_bits<const BYTE_SIZE: usize>(
    data: &[u8; BYTE_SIZE],
    bit_index: usize,
    bits: usize,
) -> (u32, usize) {
    let mut value = 0u32;
    let mut index = bit_index;
    for _ in 0..bits {
        value = (value << 1) | (read_bit(data, index) as u32);
        index += 1;
    }
    (value, index)
}

#[inline(always)]
fn write_bits<const BYTE_SIZE: usize>(
    data: &mut [u8; BYTE_SIZE],
    bit_index: usize,
    value: u32,
    bits: usize,
) -> usize {
    let mut index = bit_index;
    for i in (0..bits).rev() {
        let bit = ((value >> i) & 1) == 1;
        let byte_index = index / 8;
        let bit_in_byte = 7 - (index % 8);
        if bit {
            data[byte_index] |= 1 << bit_in_byte;
        } else {
            data[byte_index] &= !(1 << bit_in_byte);
        }
        index += 1;
    }
    index
}

#[inline(always)]
fn last_one_bit<const BYTE_SIZE: usize>(data: &[u8; BYTE_SIZE]) -> Option<usize> {
    for bit_index in (0..BYTE_SIZE * 8).rev() {
        if read_bit(data, bit_index) {
            return Some(bit_index);
        }
    }
    None
}

#[inline(always)]
fn count_events<const BYTE_SIZE: usize>(data: &[u8; BYTE_SIZE], last_bit: usize) -> u8 {
    let mut index = 0usize;
    let mut events = 0u8;
    while index <= last_bit {
        if read_bit(data, index) {
            index += 1;
        } else {
            index += 4;
        }
        events += 1;
    }
    events
}

#[cfg(test)]
mod test {
    use super::*;

    /// The word-at-a-time path (keys of up to 8 bytes) must produce exactly the same
    /// bytes as the bit-by-bit path (used for larger keys).
    #[test]
    fn fast_compression_matches_bit_by_bit() {
        let mut states = crate::state::test::reachable_states(6);
        states.extend(crate::state::test::random_states(20_000));
        for mut state in states {
            state.normalize();
            if state.size() > 11 {
                continue; // more than 56 bits: does not fit a 7-byte key
            }
            let fast = CompressedState::<7>::new(state);
            let slow = CompressedState::<9>::new(state);
            assert_eq!(fast.data[..], slow.data[..7], "{state}");
            assert_eq!(slow.data[7..], [0, 0], "{state}");
            assert_eq!(fast.len(), slow.len(), "{state}");
        }
    }
}
