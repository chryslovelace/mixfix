use fixity::Fixity;
use operator::Operator;
use petgraph::{
    graph::{DiGraph, NodeIndex},
    visit::Bfs,
};
use std::collections::HashMap;

pub trait PrecedenceGraph {
    type P: Copy;
    fn ops(&self, prec: Self::P, fix: Fixity) -> Vec<&Operator>;
    fn succ(&self, prec: Self::P) -> Vec<Self::P>;
    fn all(&self) -> Vec<Self::P>;
}

impl PrecedenceGraph for HashMap<usize, Vec<Operator>> {
    type P = usize;

    fn ops(&self, prec: Self::P, fix: Fixity) -> Vec<&Operator> {
        if let Some(ops) = self.get(&prec) {
            ops.iter().filter(|o| o.fixity == fix).collect()
        } else {
            vec![]
        }
    }

    fn succ(&self, prec: Self::P) -> Vec<Self::P> {
        self.keys().cloned().filter(|&k| k > prec).collect()
    }

    fn all(&self) -> Vec<Self::P> {
        self.keys().cloned().collect()
    }
}

impl PrecedenceGraph for DiGraph<Vec<Operator>, ()> {
    type P = NodeIndex;

    fn ops(&self, prec: Self::P, fix: Fixity) -> Vec<&Operator> {
        self[prec].iter().filter(|o| o.fixity == fix).collect()
    }

    fn succ(&self, prec: Self::P) -> Vec<Self::P> {
        let mut succ = Vec::new();
        let mut bfs = Bfs::new(self, prec);
        while let Some(n) = bfs.next(self) {
            if n != prec {
                succ.push(n);
            }
        }
        succ
    }

    fn all(&self) -> Vec<Self::P> {
        self.node_indices().collect()
    }
}
