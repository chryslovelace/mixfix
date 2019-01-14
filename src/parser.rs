use either::*;
use expr::Expr;
use fixity::{Associativity, Fixity};
use graph::PrecedenceGraph;
use itertools::Itertools;
use operator::{NamePart, Operator};

trait Parser {
    type O;
    fn p<'i>(&self, toks: &'i [NamePart]) -> Option<(&'i [NamePart], Self::O)>;
}

impl<P: Parser> Parser for &P {
    type O = P::O;
    fn p<'i>(&self, toks: &'i [NamePart]) -> Option<(&'i [NamePart], Self::O)> {
        (*self).p(toks)
    }
}

impl<O> Parser for &Parser<O = O> {
    type O = O;
    fn p<'i>(&self, toks: &'i [NamePart]) -> Option<(&'i [NamePart], Self::O)> {
        (*self).p(toks)
    }
}

struct Tok<'a>(&'a str);

impl<'a> Parser for Tok<'a> {
    type O = ();
    fn p<'i>(&self, toks: &'i [NamePart]) -> Option<(&'i [NamePart], Self::O)> {
        if let Some((tok, rest)) = toks.split_first() {
            if tok == self.0 {
                return Some((rest, ()));
            }
        }
        None
    }
}

impl<A: Parser, B: Parser> Parser for (A, B) {
    type O = (A::O, B::O);
    fn p<'i>(&self, toks: &'i [NamePart]) -> Option<(&'i [NamePart], Self::O)> {
        let (toks, a) = self.0.p(toks)?;
        let (toks, b) = self.1.p(toks)?;
        Some((toks, (a, b)))
    }
}

impl<A: Parser, B: Parser, C: Parser> Parser for (A, B, C) {
    type O = (A::O, B::O, C::O);
    fn p<'i>(&self, toks: &'i [NamePart]) -> Option<(&'i [NamePart], Self::O)> {
        let (toks, a) = self.0.p(toks)?;
        let (toks, b) = self.1.p(toks)?;
        let (toks, c) = self.2.p(toks)?;
        Some((toks, (a, b, c)))
    }
}

struct Opt<A, B>(A, B);

impl<A: Parser, B: Parser> Parser for Opt<A, B> {
    type O = Either<A::O, B::O>;
    fn p<'i>(&self, toks: &'i [NamePart]) -> Option<(&'i [NamePart], Self::O)> {
        match (self.0.p(toks), self.1.p(toks)) {
            (Some((next, a)), None) => Some((next, Left(a))),
            (None, Some((next, b))) => Some((next, Right(b))),
            (Some((next_a, a)), Some((next_b, b))) => {
                if next_a.len() < next_b.len() {
                    Some((next_a, Left(a)))
                } else {
                    Some((next_b, Right(b)))
                }
            }
            (None, None) => None,
        }
    }
}

struct Seq<T>(Vec<T>);

impl<P: Parser> Parser for Seq<P> {
    type O = Vec<P::O>;
    fn p<'i>(&self, mut toks: &'i [NamePart]) -> Option<(&'i [NamePart], Self::O)> {
        let mut out = Vec::new();
        for p in &self.0 {
            let (next, o) = p.p(toks)?;
            toks = next;
            out.push(o);
        }
        Some((toks, out))
    }
}

struct Opts<T>(Vec<T>);

impl<P: Parser> Parser for Opts<P> {
    type O = P::O;
    fn p<'i>(&self, toks: &'i [NamePart]) -> Option<(&'i [NamePart], Self::O)> {
        self.0
            .iter()
            .filter_map(|p| p.p(toks))
            .min_by_key(|(toks, _)| toks.len())
    }
}

struct Star<T>(T);

impl<P: Parser> Parser for Star<P> {
    type O = Vec<P::O>;
    fn p<'i>(&self, mut toks: &'i [NamePart]) -> Option<(&'i [NamePart], Self::O)> {
        let mut res = Vec::new();
        while let Some((next, o)) = self.0.p(toks) {
            toks = next;
            res.push(o);
        }
        Some((toks, res))
    }
}

struct Plus<T>(T);

impl<P: Parser> Parser for Plus<P> {
    type O = Vec<P::O>;
    fn p<'i>(&self, toks: &'i [NamePart]) -> Option<(&'i [NamePart], Self::O)> {
        Star(&self.0).p(toks).filter(|(_, res)| !res.is_empty())
    }
}

struct Expr_<'g, G: PrecedenceGraph>(&'g G);

impl<'g, G: PrecedenceGraph> Parser for Expr_<'g, G> {
    type O = Expr;
    fn p<'i>(&self, toks: &'i [NamePart]) -> Option<(&'i [NamePart], Self::O)> {
        Precs(self.0, self.0.all()).p(toks)
    }
}

#[derive(Clone)]
struct Precs<'g, G: PrecedenceGraph>(&'g G, Vec<G::P>);

impl<'g, G: PrecedenceGraph> Parser for Precs<'g, G> {
    type O = Expr;
    fn p<'i>(&self, toks: &'i [NamePart]) -> Option<(&'i [NamePart], Self::O)> {
        Opts(self.1.iter().map(|&p| Prec(self.0, p)).collect()).p(toks)
    }
}

struct Prec<'g, G: PrecedenceGraph>(&'g G, G::P);

impl<'g, G: PrecedenceGraph> Parser for Prec<'g, G> {
    type O = Expr;
    fn p<'i>(&self, toks: &'i [NamePart]) -> Option<(&'i [NamePart], Self::O)> {
        Opts::<&Parser<O = Expr>>(vec![
            &Closed(self.0, self.1),
            &NonAssoc(self.0, self.1),
            &PreRight(self.0, self.1),
            &PostLeft(self.0, self.1),
        ])
        .p(toks)
    }
}

struct Closed<'g, G: PrecedenceGraph>(&'g G, G::P);

impl<'g, G: PrecedenceGraph> Parser for Closed<'g, G> {
    type O = Expr;
    fn p<'i>(&self, toks: &'i [NamePart]) -> Option<(&'i [NamePart], Self::O)> {
        Inner(self.0, self.1, Fixity::Closed).p(toks)
    }
}

struct NonAssoc<'g, G: PrecedenceGraph>(&'g G, G::P);

impl<'g, G: PrecedenceGraph> Parser for NonAssoc<'g, G> {
    type O = Expr;
    fn p<'i>(&self, toks: &'i [NamePart]) -> Option<(&'i [NamePart], Self::O)> {
        let succ = Precs(self.0, self.0.succ(self.1));
        let (toks, left) = succ.p(toks)?;
        let (toks, mut expr) = Inner(self.0, self.1, Fixity::Infix(Associativity::Non)).p(toks)?;
        let (toks, right) = succ.p(toks)?;
        expr.args.insert(0, left);
        expr.args.push(right);
        Some((toks, expr))
    }
}

struct PreRight<'g, G: PrecedenceGraph>(&'g G, G::P);

impl<'g, G: PrecedenceGraph> Parser for PreRight<'g, G> {
    type O = Expr;
    fn p<'i>(&self, toks: &'i [NamePart]) -> Option<(&'i [NamePart], Self::O)> {
        let succ = Precs(self.0, self.0.succ(self.1));
        let (toks, (inners, last)) = (
            Plus(Opt(
                Inner(self.0, self.1, Fixity::Prefix),
                (
                    &succ,
                    Inner(self.0, self.1, Fixity::Infix(Associativity::Right)),
                ),
            )),
            &succ,
        )
            .p(toks)?;

        let mut expr = inners
            .into_iter()
            .map(|e| {
                e.map_right(|(first, mut rest)| {
                    rest.args.insert(0, first);
                    rest
                })
                .into_inner()
            })
            .rev()
            .fold1(|right, mut left| {
                left.args.push(right);
                left
            })
            .unwrap();

        expr.args.push(last);

        Some((toks, expr))
    }
}

struct PostLeft<'g, G: PrecedenceGraph>(&'g G, G::P);

impl<'g, G: PrecedenceGraph> Parser for PostLeft<'g, G> {
    type O = Expr;
    fn p<'i>(&self, toks: &'i [NamePart]) -> Option<(&'i [NamePart], Self::O)> {
        let succ = Precs(self.0, self.0.succ(self.1));

        let (toks, (first, inners)) = (
            &succ,
            Plus(Opt(
                Inner(self.0, self.1, Fixity::Postfix),
                (
                    Inner(self.0, self.1, Fixity::Infix(Associativity::Left)),
                    &succ,
                ),
            )),
        )
            .p(toks)?;

        let mut expr = inners
            .into_iter()
            .map(|e| {
                e.map_right(|(mut rest, last)| {
                    rest.args.push(last);
                    rest
                })
                .into_inner()
            })
            .fold1(|left, mut right| {
                right.args.insert(0, left);
                right
            })
            .unwrap();

        expr.args.insert(0, first);

        Some((toks, expr))
    }
}

struct Inner<'g, G: PrecedenceGraph>(&'g G, G::P, Fixity);

impl<'g, G: PrecedenceGraph> Parser for Inner<'g, G> {
    type O = Expr;
    fn p<'i>(&self, toks: &'i [NamePart]) -> Option<(&'i [NamePart], Self::O)> {
        Opts(
            self.0
                .ops(self.1, self.2)
                .into_iter()
                .map(|o| Backbone(self.0, o))
                .collect(),
        )
        .p(toks)
    }
}

struct Backbone<'g, G: PrecedenceGraph>(&'g G, &'g Operator);

impl<'g, G: PrecedenceGraph> Parser for Backbone<'g, G> {
    type O = Expr;
    fn p<'i>(&self, toks: &'i [NamePart]) -> Option<(&'i [NamePart], Self::O)> {
        let (first, rest) = self.1.name_parts.split_first().unwrap();
        let (toks, (_, exprs)) = (
            Tok(first),
            Seq(rest.iter().map(|t| (Expr_(self.0), Tok(t))).collect()),
        )
            .p(toks)?;
        let exprs = exprs.into_iter().map(|(expr, ())| expr).collect();
        Some((toks, Expr::new(self.1.clone(), exprs)))
    }
}

pub fn parse_expr<G: PrecedenceGraph>(graph: &G, tokens: &[NamePart]) -> Option<Expr> {
    let (unparsed, expr) = Expr_(graph).p(tokens)?;
    if unparsed.len() == 0 {
        Some(expr)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::graph::DiGraph;

    #[test]
    fn test_tok() {
        let input = [":".to_string()];
        let (toks, ()) = Tok(":").p(&input).unwrap();
        assert_eq!(toks.len(), 0);
    }

    fn simple_graph() -> impl PrecedenceGraph {
        let atom = Operator::new(Fixity::Closed, vec!["•"]);
        let plus = Operator::new(Fixity::Infix(Associativity::Left), vec!["+"]);
        let well_typed = Operator::new(Fixity::Postfix, vec!["⊢", ":"]);
        let mut g = DiGraph::new();
        let a = g.add_node(vec![atom]);
        let pl = g.add_node(vec![plus]);
        let wt = g.add_node(vec![well_typed.clone()]);
        g.add_edge(pl, a, ());
        g.add_edge(wt, a, ());
        g.add_edge(wt, pl, ());
        g
    }

    #[test]
    fn test_simple_parse() {
        let input: Vec<_> = "•+•⊢•:".chars().map(|c| c.to_string()).collect();
        println!("{:?}", input);
        let expr = parse_expr(&simple_graph(), &input).unwrap();
        assert!(expr.well_formed());
        println!("{:#?}", expr);
    }
}
