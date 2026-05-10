use crate::{MindTables, PersonaRole, Result, StoreLocation, StoredClaim};
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
    tables: MindTables,
}

impl ClaimLedger {
    pub fn open(store: &StoreLocation) -> Result<Self> {
        Ok(Self {
            tables: MindTables::open(store)?,
        })
    }

    pub fn apply_claim(&self, claim: RoleClaim) -> Result<MindReply> {
        let entries = self.tables.claim_records()?;
        let conflicts = Self::conflicts_for(&entries, &claim);
        if !conflicts.is_empty() {
            return Ok(MindReply::ClaimRejection(ClaimRejection {
                role: claim.role,
                conflicts,
            }));
        }

        let mut next_entries = entries.clone();
        for scope in &claim.scopes {
            if Self::role_already_owns(&next_entries, &claim.role, scope) {
                continue;
            }
            next_entries
                .retain(|entry| entry.role != claim.role || !scope_contains(scope, &entry.scope));
            next_entries.push(StoredClaim::new(
                claim.role,
                scope.clone(),
                claim.reason.clone(),
            ));
        }

        let remove_keys = entries
            .iter()
            .filter(|entry| entry.role == claim.role)
            .map(StoredClaim::key)
            .collect::<Vec<_>>();
        let insert_claims = next_entries
            .iter()
            .filter(|entry| entry.role == claim.role)
            .cloned()
            .collect::<Vec<_>>();
        self.tables.replace_claims(&remove_keys, &insert_claims)?;

        Ok(MindReply::ClaimAcceptance(ClaimAcceptance {
            role: claim.role,
            scopes: claim.scopes,
        }))
    }

    pub fn apply_release(&self, release: RoleRelease) -> Result<MindReply> {
        let entries = self.tables.claim_records()?;
        let released_scopes = entries
            .iter()
            .filter(|entry| entry.role == release.role)
            .map(|entry| entry.scope.clone())
            .collect::<Vec<_>>();
        let remove_keys = entries
            .iter()
            .filter(|entry| entry.role == release.role)
            .map(StoredClaim::key)
            .collect::<Vec<_>>();
        self.tables.replace_claims(&remove_keys, &[])?;

        Ok(MindReply::ReleaseAcknowledgment(
            signal_persona_mind::ReleaseAcknowledgment {
                role: release.role,
                released_scopes,
            },
        ))
    }

    pub fn observe(&self, _observation: RoleObservation) -> Result<MindReply> {
        let entries = self.tables.claim_records()?;
        let roles = RoleName::ALL
            .into_iter()
            .map(|role| RoleStatus {
                role,
                claims: Self::claims_for(&entries, role),
            })
            .collect();

        Ok(MindReply::RoleSnapshot(RoleSnapshot {
            roles,
            recent_activity: Vec::new(),
        }))
    }

    fn conflicts_for(entries: &[StoredClaim], claim: &RoleClaim) -> Vec<ScopeConflict> {
        claim
            .scopes
            .iter()
            .flat_map(|scope| {
                entries
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

    fn role_already_owns(entries: &[StoredClaim], role: &RoleName, scope: &ScopeReference) -> bool {
        entries
            .iter()
            .any(|entry| entry.role == *role && scope_contains(&entry.scope, scope))
    }

    fn claims_for(entries: &[StoredClaim], role: RoleName) -> Vec<ClaimEntry> {
        entries
            .iter()
            .filter(|entry| entry.role == role)
            .map(|entry| ClaimEntry {
                scope: entry.scope.clone(),
                reason: entry.reason.clone(),
            })
            .collect()
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
