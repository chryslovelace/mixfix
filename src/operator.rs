use fixity::Fixity;

pub type NamePart = String;

#[derive(Debug, Clone)]
pub struct OperatorPattern(pub Vec<Vec<NamePart>>);

#[derive(Debug, Clone)]
pub struct Operator {
    pub fixity: Fixity,
    pub pattern: OperatorPattern,
}

impl Operator {
    pub fn arity(&self) -> usize {
        self.pattern.0.len() - 1
    }
}

impl OperatorPattern {
    pub fn backbone(&self) -> &[Vec<NamePart>] {
        let first = self.0.iter().position(|v| !v.is_empty());
        let last = self.0.iter().rposition(|v| !v.is_empty());
        if let (Some(first), Some(last)) = (first, last) {
            &self.0[first..=last]
        } else {
            &[]
        }
    }
}

impl From<&str> for OperatorPattern {
    fn from(s: &str) -> Self {
        OperatorPattern(
            s.split('_')
                .map(|sep| (sep.split_whitespace().map(|s| s.to_string()).collect()))
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_backbone() {
        let op: OperatorPattern = "__a__b__".into();
        println!("{:?}", op);
        let b = op.backbone();
        println!("{:?}", b);
    }
}
