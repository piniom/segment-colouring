pub fn find_lowest_missing<I>(iter: I) -> u32
where
    I: IntoIterator<Item = u32>,
{
    let mut numbers: Vec<_> = iter.into_iter().collect();
    numbers.sort_unstable();
    numbers.dedup();

    let mut expected = 0;
    for &num in &numbers {
        if num == expected {
            expected += 1;
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