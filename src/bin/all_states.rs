use segment_colouring::linear_axis::clicqued::ClicquedLinearAxis;

#[tokio::main]
async fn main() {
    let mut axis = ClicquedLinearAxis::new(5);
    let states = axis.generate_all_states_async(9, 0).await;
    dbg!(states.len());
}
