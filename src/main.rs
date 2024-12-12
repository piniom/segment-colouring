use segment_colouring::first_fit::FirstFitColourer;

fn main() {
    let mut ff = FirstFitColourer::default();
    let inputs = [(0, 0), (0, 0), (1, 3), (2, 5), (6, 8)];
    for (s, e) in inputs {
        ff.insert_segment(s, e).unwrap();
        println!("{}\n", ff.to_string())
    }
}
