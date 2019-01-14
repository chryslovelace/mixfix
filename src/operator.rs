use fixity::Fixity;

pub type NamePart = String;

#[derive(Debug, Clone)]
pub struct Operator {
    pub fixity: Fixity,
    pub name_parts: Vec<Vec<NamePart>>,
}

impl Operator {
    pub fn arity(&self) -> usize {
        let inner = self.name_parts.len() - 1;
        match self.fixity {
            Fixity::Closed => inner,
            Fixity::Prefix | Fixity::Postfix => inner + 1,
            Fixity::Infix(_) => inner + 2,
        }
    }
}
