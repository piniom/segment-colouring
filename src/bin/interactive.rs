use segment_colouring::linear_axis::{clicqued::ClicquedLinearAxis, history::History};

fn main() {
    let mut axis = ClicquedLinearAxis::new(4);
    use History::*;
    let moves = [
        SegmentInsert {
            start_index: 0,
            end_index: 0,
            color: 3,
        },
        SegmentInsert {
            start_index: 1,
            end_index: 2,
            color: 1,
        },
        SegmentInsert {
            start_index: 1,
            end_index: 5,
            color: 0,
        },
        LimitFront,
        LimitFront,
        LimitFront,
        SegmentInsert {
            start_index: 3,
            end_index: 3,
            color: 0,
        },
        SegmentInsert {
            start_index: 4,
            end_index: 5,
            color: 1,
        },
        SegmentInsert {
            start_index: 6,
            end_index: 7,
            color: 0,
        },
        LimitBack,
        LimitBack,
        // SegmentInsert {
        //     start_index: 1,
        //     end_index: 7,
        //     color: 2,
        // },
    ];
    for m in moves {
        axis.apply_history(m);
        println!("{}", axis.inner.to_string());
    }
    println!("{:?}", axis.valid_new_segment_ends(0));

    println!("{:?}", axis.intersections);
}
