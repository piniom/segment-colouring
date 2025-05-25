use segment_colouring::linear_axis::clicqued::ClicquedLinearAxis;


fn main() {
    let mut axis = ClicquedLinearAxis::new(5);
    let states = axis.generate_all_states(8);
    dbg!(states.len());
}
