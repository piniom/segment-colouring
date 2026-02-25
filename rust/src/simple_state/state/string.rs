use super::*;

impl State {
    pub fn to_string(&self) -> String {
        let mut result = String::new();
        for i in 0..=self.len() {
            if self.limit_front() == i {
                result.push('[');
            }
            if self.limit_back() == i {
                result.push(']');
            }
            if self.len() <= i {
                break;
            }
            let value = self.get_at_index(i);
            let base = if value & 0b1000 == 0 {
                'A' as u8
            } else {
                'a' as u8
            };
            result.push((base + (value & 0b0111)) as char);
        }
        result
    }
    pub fn from_string(s: &str) -> Self {
        let mut result = Self::new();
        for c in s.chars() {
            match c {
                '[' => result.set_limit_front(result.len()),
                ']' => result.set_limit_back(result.len()),
                'A'..='H' => result.insert_at_index(result.len(), (c as u8 - 'A' as u8) & 0b0111),
                'a'..='h' => {
                    result.insert_at_index(result.len(), ((c as u8 - 'a' as u8) & 0b0111) | 0b1000)
                }
                _ => panic!("Invalid character in string representation of state"),
            }
        }
        result
    }
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State")
            .field("data", &self.to_string().replace("[", "").replace("]", ""))
            .field("len", &self.len())
            .field("limit_front", &self.limit_front())
            .field("limit_back", &self.limit_back())
            .finish()
    }
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[allow(dead_code)]
pub(super) fn print_grouped(n: u128) {
    let s = format!("{:0128b}", n);
    let grouped = s
        .as_bytes()
        .chunks(4)
        .map(std::str::from_utf8)
        .map(Result::unwrap)
        .collect::<Vec<_>>()
        .join(" ");
    println!("{}", grouped);
}

#[cfg(test)]
mod test {
    use std::vec;

    use super::*;

    #[test]
    fn test_string_conversion() {
        let mut state = State::new();
        state.insert_at_index(0, 0);
        state.insert_at_index(1, 1);
        state.insert_at_index(2, 2);
        state.insert_at_index(3, 3);
        state.insert_at_index(4, 4);
        state.set_limit_front(1);
        state.set_limit_back(4);
        let s = state.to_string();
        assert_eq!(s, "A[BCD]E");
        let parsed = State::from_string(&s);
        assert_eq!(state, parsed);
    }
    #[test]
    fn test_string_conversion_empty() {
        let state = State::new();
        let s = state.to_string();
        assert_eq!(s, "[]");
        let parsed = State::from_string(&s);
        assert_eq!(state, parsed);
    }
    #[test]
    fn test_complex() {
        let segments = vec![
            (0, 0, 0),
            (1, 1, 1),
            (2, 2, 2),
            (3, 3, 3),
            (4, 4, 4),
            (0, 1, 5),
            (2, 3, 6),
            (1, 2, 7),
            (0, 2, 4),
        ];
        let mut state = State::new();
        for i in segments {
            state.insert_segment(i.0, i.1, i.2);
        }
        state.move_limit_front();
        state.move_limit_back();
        let parsed = State::from_string(&state.to_string());
        // assert_eq!(state.limit_back, parsed.limit_back);
        // assert_eq!(state.to_string(), parsed.to_string());
        dbg!(state.to_string());
        assert_eq!(state, parsed);
    }
}
