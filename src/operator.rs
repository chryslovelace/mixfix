use fixity::Fixity;

pub type NamePart = String;

#[derive(Debug, Clone)]
pub struct Operator {
    pub fixity: Fixity,
    pub name_parts: Vec<NamePart>,
}

impl Operator {
    pub fn new<S: Into<String>, I: IntoIterator<Item = S>>(fixity: Fixity, name_parts: I) -> Self {
        Operator {
            fixity,
            name_parts: name_parts.into_iter().map(|s| s.into()).collect(),
        }
    }

    pub fn arity(&self) -> usize {
        let inner = self.name_parts.len() - 1;
        match self.fixity {
            Fixity::Closed => inner,
            Fixity::Prefix | Fixity::Postfix => inner + 1,
            Fixity::Infix(_) => inner + 2,
        }
    }
}
