use crate::PersonaRole;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimScope {
    path: String,
}

impl ClaimScope {
    pub fn new(path: impl Into<String>) -> Self {
        ClaimPath::new(path.into()).into_scope()
    }

    pub fn as_str(&self) -> &str {
        &self.path
    }

    pub fn contains(&self, other: &ClaimScope) -> bool {
        self.path == "/"
            || self.path == other.path
            || other
                .path
                .strip_prefix(self.path.as_str())
                .is_some_and(|suffix| suffix.starts_with('/'))
    }

    pub fn overlaps(&self, other: &ClaimScope) -> bool {
        self.contains(other) || other.contains(self)
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
        if self.scopes.iter().any(|current| current.contains(&scope)) {
            return;
        }

        self.scopes.retain(|current| !scope.contains(current));
        self.scopes.push(scope);
    }

    pub fn owns(&self, scope: &ClaimScope) -> bool {
        self.scopes.iter().any(|current| current.contains(scope))
    }

    pub fn role(&self) -> &PersonaRole {
        &self.role
    }

    pub fn scope_count(&self) -> usize {
        self.scopes.len()
    }
}

struct ClaimPath {
    value: String,
}

impl ClaimPath {
    fn new(value: String) -> Self {
        Self { value }
    }

    fn into_scope(self) -> ClaimScope {
        ClaimScope {
            path: self.normalized(),
        }
    }

    fn normalized(&self) -> String {
        let absolute = if self.value.starts_with('/') {
            self.value.clone()
        } else {
            format!("/{}", self.value)
        };
        let parts = absolute
            .split('/')
            .filter(|part| !part.is_empty() && *part != ".")
            .collect::<Vec<_>>();

        if parts.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", parts.join("/"))
        }
    }
}
