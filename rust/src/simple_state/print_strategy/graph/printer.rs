use std::{collections::HashSet, io::Write};

use crate::simple_state::state::State;


#[derive(Debug, Clone, Default)]
pub struct StrategyGraphPrinter<const MAX_CLIQUE: u32> {
    pub printed: HashSet<State<MAX_CLIQUE>>,
    pub vertices: Vec<Vec<State<MAX_CLIQUE>>>,
    pub edges: Vec<StrategyGraphEdge<MAX_CLIQUE>>,
}

#[derive(Debug, Clone)]
pub struct StrategyGraphEdge<const MAX_CLIQUE: u32> {
    pub source: State<MAX_CLIQUE>,
    pub target: State<MAX_CLIQUE>,
    pub label: String,
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
                    -(y as f32) * 1.5f32,
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