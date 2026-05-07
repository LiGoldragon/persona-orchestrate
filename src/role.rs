#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PersonaRole {
    value: String,
}

impl PersonaRole {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    pub fn operator() -> Self {
        Self::new("operator")
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}
