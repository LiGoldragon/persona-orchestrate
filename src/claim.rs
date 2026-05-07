use crate::PersonaRole;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimScope {
    path: String,
}

impl ClaimScope {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }

    pub fn as_str(&self) -> &str {
        &self.path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimState {
    role: PersonaRole,
    scopes: Vec<ClaimScope>,
}

impl ClaimState {
    pub fn new(role: PersonaRole) -> Self {
        Self {
            role,
            scopes: Vec::new(),
        }
    }

    pub fn claim(&mut self, scope: ClaimScope) {
        if !self.scopes.iter().any(|current| current == &scope) {
            self.scopes.push(scope);
        }
    }

    pub fn owns(&self, scope: &ClaimScope) -> bool {
        self.scopes.iter().any(|current| current == scope)
    }

    pub fn role(&self) -> &PersonaRole {
        &self.role
    }
}
