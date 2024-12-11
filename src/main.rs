use segment_colouring::first_fit::FirstFitColourer;

fn main() {
    let mut ff = FirstFitColourer::default();
    ff.insert_segment(0, 0).unwrap();
    ff.insert_segment(0, 0).unwrap();
    ff.insert_segment(1, 3).unwrap();
    println!("{}", ff.to_string())
}
