use crate::PersonaRole;
use signal_persona_mind::{
    ClaimAcceptance, ClaimEntry, ClaimRejection, MindReply, RoleClaim, RoleName, RoleObservation,
    RoleRelease, RoleSnapshot, RoleStatus, ScopeConflict, ScopeReference,
};

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

pub struct ClaimLedger {
    entries: Vec<ClaimRecord>,
}

impl ClaimLedger {
    pub fn open() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn apply_claim(&mut self, claim: RoleClaim) -> MindReply {
        let conflicts = self.conflicts_for(&claim);
        if !conflicts.is_empty() {
            return MindReply::ClaimRejection(ClaimRejection {
                role: claim.role,
                conflicts,
            });
        }

        for scope in &claim.scopes {
            if self.role_already_owns(&claim.role, scope) {
                continue;
            }
            self.entries
                .retain(|entry| entry.role != claim.role || !scope_contains(scope, &entry.scope));
            self.entries.push(ClaimRecord {
                role: claim.role,
                scope: scope.clone(),
                reason: claim.reason.clone(),
            });
        }

        MindReply::ClaimAcceptance(ClaimAcceptance {
            role: claim.role,
            scopes: claim.scopes,
        })
    }

    pub fn apply_release(&mut self, release: RoleRelease) -> MindReply {
        let mut released_scopes = Vec::new();
        self.entries.retain(|entry| {
            if entry.role == release.role {
                released_scopes.push(entry.scope.clone());
                false
            } else {
                true
            }
        });

        MindReply::ReleaseAcknowledgment(signal_persona_mind::ReleaseAcknowledgment {
            role: release.role,
            released_scopes,
        })
    }

    pub fn observe(&self, _observation: RoleObservation) -> MindReply {
        let roles = RoleName::ALL
            .into_iter()
            .map(|role| RoleStatus {
                role,
                claims: self.claims_for(role),
            })
            .collect();

        MindReply::RoleSnapshot(RoleSnapshot {
            roles,
            recent_activity: Vec::new(),
        })
    }

    fn conflicts_for(&self, claim: &RoleClaim) -> Vec<ScopeConflict> {
        claim
            .scopes
            .iter()
            .flat_map(|scope| {
                self.entries
                    .iter()
                    .filter(move |entry| {
                        entry.role != claim.role && scopes_overlap(scope, &entry.scope)
                    })
                    .map(move |entry| ScopeConflict {
                        scope: scope.clone(),
                        held_by: entry.role,
                        held_reason: entry.reason.clone(),
                    })
            })
            .collect()
    }

    fn role_already_owns(&self, role: &RoleName, scope: &ScopeReference) -> bool {
        self.entries
            .iter()
            .any(|entry| entry.role == *role && scope_contains(&entry.scope, scope))
    }

    fn claims_for(&self, role: RoleName) -> Vec<ClaimEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.role == role)
            .map(|entry| ClaimEntry {
                scope: entry.scope.clone(),
                reason: entry.reason.clone(),
            })
            .collect()
    }
}

impl Default for ClaimLedger {
    fn default() -> Self {
        Self::open()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ClaimRecord {
    role: RoleName,
    scope: ScopeReference,
    reason: signal_persona_mind::ScopeReason,
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

fn scopes_overlap(left: &ScopeReference, right: &ScopeReference) -> bool {
    scope_contains(left, right) || scope_contains(right, left)
}

fn scope_contains(left: &ScopeReference, right: &ScopeReference) -> bool {
    match (left, right) {
        (ScopeReference::Path(left), ScopeReference::Path(right)) => {
            path_contains(left.as_str(), right.as_str())
        }
        (ScopeReference::Task(left), ScopeReference::Task(right)) => left == right,
        _ => false,
    }
}

fn path_contains(left: &str, right: &str) -> bool {
    left == "/"
        || left == right
        || right
            .strip_prefix(left)
            .is_some_and(|tail| tail.starts_with('/'))
}
