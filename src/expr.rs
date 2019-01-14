use operator::Operator;

#[derive(Debug)]
pub struct Expr {
    pub operator: Operator,
    pub args: Vec<Expr>,
}

impl Expr {
    pub fn new(operator: Operator, args: Vec<Expr>) -> Expr {
        Expr { operator, args }
    }

    pub fn well_formed(&self) -> bool {
        self.args.len() == self.operator.arity() && self.args.iter().all(Expr::well_formed)
    }
}
