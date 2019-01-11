use fixity::Fixity;
use operator::Operator;
use petgraph::{
    graph::{node_index, DiGraph, NodeIndex},
    visit::{depth_first_search, Bfs, DfsEvent},
    Direction,
};

pub trait PrecedenceGraph {
    type P: Copy;
    fn ops(&self, prec: Self::P, fix: Fixity) -> Vec<Operator>;
    fn succ(&self, prec: Self::P) -> Vec<Self::P>;
    fn all(&self) -> Vec<Self::P>;
}

impl PrecedenceGraph for Vec<Vec<Operator>> {
    type P = usize;

    fn ops(&self, prec: Self::P, fix: Fixity) -> Vec<Operator> {
        self[prec]
            .iter()
            .filter(|o| o.fixity == fix)
            .cloned()
            .collect()
    }

    fn succ(&self, prec: Self::P) -> Vec<Self::P> {
        if prec + 1 < self.len() {
            vec![prec + 1]
        } else {
            vec![]
        }
    }

    fn all(&self) -> Vec<Self::P> {
        (0..self.len()).collect()
    }
}

impl PrecedenceGraph for DiGraph<Vec<Operator>, ()> {
    type P = NodeIndex;

    fn ops(&self, prec: Self::P, fix: Fixity) -> Vec<Operator> {
        self[prec]
            .iter()
            .filter(|o| o.fixity == fix)
            .cloned()
            .collect()
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
