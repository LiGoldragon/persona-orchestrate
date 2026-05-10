use sema::{Schema, SchemaVersion, Sema, Table};
use signal_persona_mind::{RoleName, ScopeReason, ScopeReference};

use crate::{Result, StoreLocation};

const MIND_SCHEMA: Schema = Schema {
    version: SchemaVersion::new(1),
};

const CLAIMS: Table<&'static str, StoredClaim> = Table::new("claims");

pub struct MindTables {
    database: Sema,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct StoredClaim {
    pub role: RoleName,
    pub scope: ScopeReference,
    pub reason: ScopeReason,
}

impl StoredClaim {
    pub fn new(role: RoleName, scope: ScopeReference, reason: ScopeReason) -> Self {
        Self {
            role,
            scope,
            reason,
        }
    }

    pub fn key(&self) -> String {
        ClaimKey::new(self.role, &self.scope).into_string()
    }
}

impl MindTables {
    pub fn open(store: &StoreLocation) -> Result<Self> {
        let database = Sema::open_with_schema(store.as_path(), &MIND_SCHEMA)?;
        database.write(|transaction| {
            CLAIMS.ensure(transaction)?;
            Ok(())
        })?;
        Ok(Self { database })
    }

    pub fn claim_records(&self) -> Result<Vec<StoredClaim>> {
        Ok(self.database.read(|transaction| {
            Ok(CLAIMS
                .iter(transaction)?
                .into_iter()
                .map(|(_key, claim)| claim)
                .collect())
        })?)
    }

    pub fn replace_claims(
        &self,
        remove_keys: &[String],
        insert_claims: &[StoredClaim],
    ) -> Result<()> {
        self.database.write(|transaction| {
            for key in remove_keys {
                CLAIMS.remove(transaction, key.as_str())?;
            }
            for claim in insert_claims {
                let key = claim.key();
                CLAIMS.insert(transaction, key.as_str(), claim)?;
            }
            Ok(())
        })?;
        Ok(())
    }
}

struct ClaimKey {
    role: RoleName,
    scope: String,
}

impl ClaimKey {
    fn new(role: RoleName, scope: &ScopeReference) -> Self {
        Self {
            role,
            scope: ScopeKey::new(scope).into_string(),
        }
    }

    fn into_string(self) -> String {
        format!("{}|{}", self.role.as_wire_token(), self.scope)
    }
}

struct ScopeKey {
    value: String,
}

impl ScopeKey {
    fn new(scope: &ScopeReference) -> Self {
        let value = match scope {
            ScopeReference::Path(path) => format!("path:{}", path.as_str()),
            ScopeReference::Task(task) => format!("task:{}", task.as_str()),
        };
        Self { value }
    }

    fn into_string(self) -> String {
        self.value
    }
}
