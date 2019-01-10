use fixity::Fixity;

pub type NamePart = String;

#[derive(Clone)]
pub struct Operator {
    pub fixity: Fixity,
    pub name_parts: Vec<NamePart>,
}
