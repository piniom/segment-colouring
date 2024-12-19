use std::ops::{Add, AddAssign};

pub fn find_lowest_missing<I, T>(iter: I) -> T
where
    I: IntoIterator<Item = T>,
    T: PartialEq + Ord + Add<T> + Default + From<u8> + AddAssign<T> + Copy,
{
    let mut numbers: Vec<_> = iter.into_iter().collect();
    numbers.sort_unstable();
    numbers.dedup();

    let mut expected = T::default();
    for &num in &numbers {
        if num == expected {
            expected += 1.into();
        } else if num > expected {
            break;
        }
    }

    expected
}

#[test]
fn find_lowest_missing_test() {
    assert_eq!(2, find_lowest_missing(vec![0, 1]));
    assert_eq!(0, find_lowest_missing(vec![1, 2, 3, 4]));
    assert_eq!(5, find_lowest_missing(vec![0, 1, 2, 3, 4, 6, 19]));
}
