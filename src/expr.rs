use operator::Operator;

pub struct Expr {
    pub operator: Operator,
    pub args: Vec<Expr>,
}

impl Expr {
    pub fn new(operator: Operator, args: Vec<Expr>) -> Expr {
        Expr { operator, args }
    }
}
