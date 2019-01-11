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
}
