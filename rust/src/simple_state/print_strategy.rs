use std::io::Write;

use ahash::{HashSet, HashSetExt};

use crate::simple_state::{
    find::{SearchState, Visited},
    state::State,
};

impl<const MAX_CLIQUE: u32> State<MAX_CLIQUE> {
    pub fn print_strategy(&self, search_state: &SearchState<MAX_CLIQUE>, w: &mut impl Write) {
        writeln!(w, "{} {}", MAX_CLIQUE, Self::EXPECTED_COLOURS).unwrap();
        self.print_strategy_inner(search_state, w, &mut HashSet::new());
    }
    fn print_strategy_inner(
        &self,
        search_state: &SearchState<MAX_CLIQUE>,
        w: &mut impl Write,
        printed: &mut HashSet<State<MAX_CLIQUE>>,
    ) {
        let mut norm = *self;
        norm.normalize();
        let Some(Visited::Winning(move_)) = search_state.map.get(&norm) else {
            panic!("Strategy incomplete! {}", &norm)
        };
        if printed.contains(&norm) {
            return;
        }
        writeln!(w, "{} {} {}", norm.to_string(), move_.0, move_.1).unwrap();
        printed.insert(norm);
        for child in norm.with_move(*move_).outcomes() {
            child.print_strategy_inner(search_state, w, printed);
        }
    }

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
        let Some(Visited::Winning(move_)) = search_state.map.get(&norm) else {
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
                label: format!("[{}, {}] - {}", move_.0, move_.1, ('A' as u8 + colour) as char),
            });
        }

        for (child, c) in outcomes {
            let mut child_norm = child;
            child_norm.normalize();
            printer.edges.push(StrategyGraphEdge {
                source: norm,
                target: child_norm,
                label: format!("[{}, {}] - {}", move_.0, move_.1, ('A' as u8 + c) as char),
            });
            child.graph_strategy_inner(search_state, printer);
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct StrategyGraphPrinter<const MAX_CLIQUE: u32> {
    printed: HashSet<State<MAX_CLIQUE>>,
    vertices: Vec<Vec<State<MAX_CLIQUE>>>,
    edges: Vec<StrategyGraphEdge<MAX_CLIQUE>>,
}

#[derive(Debug, Clone)]
pub struct StrategyGraphEdge<const MAX_CLIQUE: u32> {
    source: State<MAX_CLIQUE>,
    target: State<MAX_CLIQUE>,
    label: String,
}

impl<const MAX_CLIQUE: u32> StrategyGraphPrinter<MAX_CLIQUE> {
    pub fn print_tikz(&self, w: &mut impl Write) -> std::io::Result<()> {
        writeln!(w, "\\documentclass[tikz,margin=5mm]{{standalone}}")?;
        writeln!(w, "\\begin{{document}}")?;
        writeln!(w, "\\begin{{tikzpicture}}[x=2cm, y=1.5cm]")?;

        // Output styles
        writeln!(w, "  \\tikzset{{vertex/.style={{inner sep=2pt}}}}")?;

        let mut node_idx_by_state: std::collections::HashMap<State<MAX_CLIQUE>, usize> =
            std::collections::HashMap::new();
        let mut idx = 0;

        for (y, layer) in self.vertices.iter().enumerate() {
            if layer.is_empty() {
                continue;
            }
            let x_base = -((layer.len() - 1) as f32) / 2.0;
            for (x_offset, state) in layer.iter().enumerate() {
                let x = x_base + (x_offset * 2) as f32;
                writeln!(
                    w,
                    "  \\node[vertex] ({}) at ({}, {}) {{{}}};",
                    idx,
                    x,
                    -(y as f32) * 2f32,
                    state
                )?;
                node_idx_by_state.insert(*state, idx);
                idx += 1;
            }
        }

        for edge in &self.edges {
            let src = node_idx_by_state[&edge.source];
            let tgt = node_idx_by_state[&edge.target];
            writeln!(
                w,
                "  \\draw[->] ({}) -- node[auto] {{{}}} ({});",
                src, edge.label, tgt
            )?;
        }
        writeln!(w, "\\end{{tikzpicture}}")?;
        writeln!(w, "\\end{{document}}")?;
        Ok(())
    }
}
