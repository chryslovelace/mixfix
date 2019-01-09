use fixity::Fixity;
use operator::Operator;
use petgraph::graph::{DiGraph, NodeIndex};

#[derive(Default)]
pub struct PrecedenceGraph(DiGraph<Vec<Operator>, ()>);

pub type Precedence = NodeIndex;

impl PrecedenceGraph {
    pub fn ops(&self, prec: Precedence, fix: Fixity) -> Vec<Operator> {
        self.0
            .node_weight(prec)
            .unwrap()
            .iter()
            .filter(|o| o.fixity == fix)
            .cloned()
            .collect()
    }

    pub fn succ(&self, prec: Precedence) -> Vec<Precedence> {
        self.0.neighbors(prec).collect()
    }

    pub fn all(&self) -> Vec<Precedence> {
        self.0.node_indices().collect()
    }
}
