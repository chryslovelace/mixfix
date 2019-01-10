use either::*;
use expr::Expr;
use fixity::{Associativity, Fixity};
use graph::Precedence;
use graph::PrecedenceGraph;
use operator::NamePart;
use operator::Operator;

trait Parser {
    type O;
    fn p<'i>(&self, toks: &'i [NamePart]) -> Option<(&'i [NamePart], Self::O)>;
}

impl<'a, O> Parser for &'a dyn Parser<O = O> {
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
        if let Some((next, a)) = self.0.p(toks) {
            Some((next, Left(a)))
        } else if let Some((next, b)) = self.1.p(toks) {
            Some((next, Right(b)))
        } else {
            None
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
        for p in &self.0 {
            if let Some((next, o)) = p.p(toks) {
                return Some((next, o));
            }
        }
        None
    }
}

struct Expr_<'g>(&'g PrecedenceGraph);

impl<'g> Parser for Expr_<'g> {
    type O = Expr;
    fn p<'i>(&self, toks: &'i [NamePart]) -> Option<(&'i [NamePart], Self::O)> {
        Precs(self.0, self.0.all()).p(toks)
    }
}

#[derive(Clone)]
struct Precs<'g>(&'g PrecedenceGraph, Vec<Precedence>);

impl<'g> Parser for Precs<'g> {
    type O = Expr;
    fn p<'i>(&self, toks: &'i [NamePart]) -> Option<(&'i [NamePart], Self::O)> {
        Opts(self.1.iter().map(|&p| Prec(self.0, p)).collect()).p(toks)
    }
}

struct Prec<'g>(&'g PrecedenceGraph, Precedence);

impl<'g> Parser for Prec<'g> {
    type O = Expr;
    fn p<'i>(&self, toks: &'i [NamePart]) -> Option<(&'i [NamePart], Self::O)> {
        Opts::<&Parser<O = Expr>>(vec![&Closed(self.0, self.1), &NonAssoc(self.0, self.1)]).p(toks)
    }
}

struct Closed<'g>(&'g PrecedenceGraph, Precedence);

impl<'g> Parser for Closed<'g> {
    type O = Expr;
    fn p<'i>(&self, toks: &'i [NamePart]) -> Option<(&'i [NamePart], Self::O)> {
        let (toks, (op, exprs)) = Inner(self.0, self.1, Fixity::Closed).p(toks)?;
        Some((toks, Expr::new(op, exprs)))
    }
}

struct NonAssoc<'g>(&'g PrecedenceGraph, Precedence);

impl<'g> Parser for NonAssoc<'g> {
    type O = Expr;
    fn p<'i>(&self, toks: &'i [NamePart]) -> Option<(&'i [NamePart], Self::O)> {
        let mut exprs = Vec::new();
        let succ = Precs(self.0, self.0.succ(self.1));
        let (toks, e) = succ.p(toks)?;
        exprs.push(e);
        let (toks, (op, inner)) =
            Inner(self.0, self.1, Fixity::Infix(Associativity::Non)).p(toks)?;
        exprs.extend(inner);
        let (toks, e) = succ.p(toks)?;
        exprs.push(e);
        Some((toks, Expr::new(op, exprs)))
    }
}

struct Inner<'g>(&'g PrecedenceGraph, Precedence, Fixity);

impl<'g> Parser for Inner<'g> {
    type O = (Operator, Vec<Expr>);
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

struct Backbone<'g>(&'g PrecedenceGraph, Operator);

impl<'g> Parser for Backbone<'g> {
    type O = (Operator, Vec<Expr>);
    fn p<'i>(&self, toks: &'i [NamePart]) -> Option<(&'i [NamePart], Self::O)> {
        let (toks, _) = Tok(&self.1.name_parts[0]).p(toks)?;
        let (toks, exprs) = Seq(self
            .1
            .name_parts
            .iter()
            .skip(1)
            .map(|t| (Expr_(self.0), Tok(t)))
            .collect())
        .p(toks)?;
        let exprs = exprs.into_iter().map(|(expr, ())| expr).collect();
        Some((toks, (self.1.clone(), exprs)))
    }
}
