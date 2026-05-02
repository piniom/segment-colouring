use crate::simple_state::{
    find::{SearchState, Visited},
    state::State,
};
use printer::*;

mod printer;

impl<const MAX_CLIQUE: u32> State<MAX_CLIQUE> {
    pub fn graph_strategy(
        &self,
        search_state: &SearchState<MAX_CLIQUE>,
    ) -> StrategyGraphPrinter<MAX_CLIQUE> {
        let mut printer = StrategyGraphPrinter::default();
        self.graph_strategy_inner(search_state, &mut printer);
        printer
    }

    fn graph_strategy_inner(
        &self,
        search_state: &SearchState<MAX_CLIQUE>,
        printer: &mut StrategyGraphPrinter<MAX_CLIQUE>,
    ) {
        let mut norm = *self;
        norm.normalize();
        let Some(Visited::Winning{move_, ..}) = search_state.get_winning(&norm) else {
            panic!("Strategy incomplete! {}", &norm)
        };
        if printer.printed.contains(&norm) {
            return;
        }

        printer.printed.insert(norm);
        let size = norm.size() as usize;
        while size >= printer.vertices.len() {
            printer.vertices.push(vec![]);
        }
        printer.vertices[size].push(norm);

        let outcomes: Vec<_> = norm.with_move(*move_).outcomes_with_colours().collect();

        if outcomes.is_empty() {
            let mut result = norm;
            let colour = result.colours_used_count();
            result.insert_segment(move_.0, move_.1, colour);
            result.normalize();

            if !printer.printed.contains(&result) {
                printer.printed.insert(result);
                let size = result.size() as usize;
                while size >= printer.vertices.len() {
                    printer.vertices.push(vec![]);
                }
                printer.vertices[size].push(result);
            };

            printer.edges.push(StrategyGraphEdge {
                source: norm,
                target: result,
                label: format!("!({}, {})!", move_.0, move_.1),
            });
        }

        for (child, c) in outcomes {
            let mut child_norm = child;
            child_norm.normalize();
            printer.edges.push(StrategyGraphEdge {
                source: norm,
                target: child_norm,
                label: format!("({}, {}):{}", move_.0, move_.1, ('A' as u8 + c) as char),
            });
            child.graph_strategy_inner(search_state, printer);
        }
    }
}