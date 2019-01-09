use operator::Operator;

pub struct Expr {
    operator: Operator,
    args: Vec<Expr>,
}
