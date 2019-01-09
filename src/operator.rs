use fixity::Fixity;

pub type NamePart = String;

#[derive(Clone)]
pub struct Operator {
    pub fixity: Fixity,
    pub name_parts: Vec<NamePart>,
}

impl Operator {
    pub fn arity(&self) -> usize {
        self.name_parts.len() - 1
    }
}
