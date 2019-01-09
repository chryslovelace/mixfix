use fixity::Fixity;
use operator::Operator;
use petgraph::graph::{DiGraph, NodeIndex};

pub trait PrecedenceGraph {
    type P;
    fn ops(&self, Self::P, Fixity) -> Vec<Operator>;
    fn succ(&self, Self::P) -> Vec<Self::P>;
    fn all(&self) -> Vec<Self::P>;
}

#[derive(Default)]
pub struct Graph(DiGraph<Vec<Operator>, ()>);

impl PrecedenceGraph for Graph {
    type P = NodeIndex;

    fn ops(&self, prec: Self::P, fix: Fixity) -> Vec<Operator> {
        self.0
            .node_weight(prec)
            .unwrap()
            .iter()
            .filter(|o| o.fixity == fix)
            .cloned()
            .collect()
    }

    fn succ(&self, prec: Self::P) -> Vec<Self::P> {
        self.0.neighbors(prec).collect()
    }

    fn all(&self) -> Vec<Self::P> {
        self.0.node_indices().collect()
    }
}
