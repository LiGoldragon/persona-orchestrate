use nota_codec::{Decoder, Encoder, NotaDecode, NotaEncode, NotaEnum, NotaRecord};
use signal_persona_mind as contract;

use crate::{Error, Result};

#[derive(NotaEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoleNameText {
    Operator,
    OperatorAssistant,
    Designer,
    DesignerAssistant,
    SystemSpecialist,
    SystemAssistant,
    Poet,
    PoetAssistant,
}

impl RoleNameText {
    fn from_contract(role: contract::RoleName) -> Self {
        match role {
            contract::RoleName::Operator => Self::Operator,
            contract::RoleName::OperatorAssistant => Self::OperatorAssistant,
            contract::RoleName::Designer => Self::Designer,
            contract::RoleName::DesignerAssistant => Self::DesignerAssistant,
            contract::RoleName::SystemSpecialist => Self::SystemSpecialist,
            contract::RoleName::SystemAssistant => Self::SystemAssistant,
            contract::RoleName::Poet => Self::Poet,
            contract::RoleName::PoetAssistant => Self::PoetAssistant,
        }
    }

    fn into_contract(self) -> contract::RoleName {
        match self {
            Self::Operator => contract::RoleName::Operator,
            Self::OperatorAssistant => contract::RoleName::OperatorAssistant,
            Self::Designer => contract::RoleName::Designer,
            Self::DesignerAssistant => contract::RoleName::DesignerAssistant,
            Self::SystemSpecialist => contract::RoleName::SystemSpecialist,
            Self::SystemAssistant => contract::RoleName::SystemAssistant,
            Self::Poet => contract::RoleName::Poet,
            Self::PoetAssistant => contract::RoleName::PoetAssistant,
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Path {
    pub path: String,
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub token: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeReferenceText {
    Path(Path),
    Task(Task),
}

impl ScopeReferenceText {
    fn from_contract(scope: contract::ScopeReference) -> Self {
        match scope {
            contract::ScopeReference::Path(path) => Self::Path(Path {
                path: path.as_str().to_string(),
            }),
            contract::ScopeReference::Task(token) => Self::Task(Task {
                token: token.as_str().to_string(),
            }),
        }
    }

    fn into_contract(self) -> Result<contract::ScopeReference> {
        match self {
            Self::Path(path) => Ok(contract::ScopeReference::Path(
                contract::WirePath::from_absolute_path(path.path)?,
            )),
            Self::Task(task) => Ok(contract::ScopeReference::Task(
                contract::TaskToken::from_wire_token(task.token)?,
            )),
        }
    }
}

impl NotaEncode for ScopeReferenceText {
    fn encode(&self, encoder: &mut Encoder) -> nota_codec::Result<()> {
        match self {
            Self::Path(path) => path.encode(encoder),
            Self::Task(task) => task.encode(encoder),
        }
    }
}

impl NotaDecode for ScopeReferenceText {
    fn decode(decoder: &mut Decoder<'_>) -> nota_codec::Result<Self> {
        match decoder.peek_record_head()?.as_str() {
            "Path" => Ok(Self::Path(Path::decode(decoder)?)),
            "Task" => Ok(Self::Task(Task::decode(decoder)?)),
            other => Err(nota_codec::Error::UnknownKindForVerb {
                verb: "ScopeReference",
                got: other.to_string(),
            }),
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RoleClaim {
    pub role: RoleNameText,
    pub scopes: Vec<ScopeReferenceText>,
    pub reason: String,
}

impl RoleClaim {
    fn into_contract(self) -> Result<contract::MindRequest> {
        let scopes = self
            .scopes
            .into_iter()
            .map(ScopeReferenceText::into_contract)
            .collect::<Result<Vec<_>>>()?;
        Ok(contract::MindRequest::RoleClaim(contract::RoleClaim {
            role: self.role.into_contract(),
            scopes,
            reason: contract::ScopeReason::from_text(self.reason)?,
        }))
    }
}

#[derive(NotaRecord, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoleRelease {
    pub role: RoleNameText,
}

impl RoleRelease {
    fn into_contract(self) -> contract::MindRequest {
        contract::MindRequest::RoleRelease(contract::RoleRelease {
            role: self.role.into_contract(),
        })
    }
}

#[derive(NotaRecord, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoleObservation {}

impl RoleObservation {
    fn into_contract(self) -> contract::MindRequest {
        contract::MindRequest::RoleObservation(contract::RoleObservation)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MindTextRequest {
    RoleClaim(RoleClaim),
    RoleRelease(RoleRelease),
    RoleObservation(RoleObservation),
}

impl MindTextRequest {
    pub fn from_nota(text: &str) -> Result<Self> {
        let mut decoder = Decoder::new(text);
        let request = Self::decode(&mut decoder)?;
        MindTextEnd::new(&mut decoder).expect()?;
        Ok(request)
    }

    pub fn into_request(self) -> Result<contract::MindRequest> {
        match self {
            Self::RoleClaim(claim) => claim.into_contract(),
            Self::RoleRelease(release) => Ok(release.into_contract()),
            Self::RoleObservation(observation) => Ok(observation.into_contract()),
        }
    }
}

impl NotaEncode for MindTextRequest {
    fn encode(&self, encoder: &mut Encoder) -> nota_codec::Result<()> {
        match self {
            Self::RoleClaim(claim) => claim.encode(encoder),
            Self::RoleRelease(release) => release.encode(encoder),
            Self::RoleObservation(observation) => observation.encode(encoder),
        }
    }
}

impl NotaDecode for MindTextRequest {
    fn decode(decoder: &mut Decoder<'_>) -> nota_codec::Result<Self> {
        match decoder.peek_record_head()?.as_str() {
            "RoleClaim" => Ok(Self::RoleClaim(RoleClaim::decode(decoder)?)),
            "RoleRelease" => Ok(Self::RoleRelease(RoleRelease::decode(decoder)?)),
            "RoleObservation" => Ok(Self::RoleObservation(RoleObservation::decode(decoder)?)),
            other => Err(nota_codec::Error::UnknownKindForVerb {
                verb: "MindTextRequest",
                got: other.to_string(),
            }),
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ClaimAcceptance {
    pub role: RoleNameText,
    pub scopes: Vec<ScopeReferenceText>,
}

impl ClaimAcceptance {
    fn from_contract(acceptance: contract::ClaimAcceptance) -> Self {
        Self {
            role: RoleNameText::from_contract(acceptance.role),
            scopes: acceptance
                .scopes
                .into_iter()
                .map(ScopeReferenceText::from_contract)
                .collect(),
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ScopeConflict {
    pub scope: ScopeReferenceText,
    pub held_by: RoleNameText,
    pub held_reason: String,
}

impl ScopeConflict {
    fn from_contract(conflict: contract::ScopeConflict) -> Self {
        Self {
            scope: ScopeReferenceText::from_contract(conflict.scope),
            held_by: RoleNameText::from_contract(conflict.held_by),
            held_reason: conflict.held_reason.as_str().to_string(),
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ClaimRejection {
    pub role: RoleNameText,
    pub conflicts: Vec<ScopeConflict>,
}

impl ClaimRejection {
    fn from_contract(rejection: contract::ClaimRejection) -> Self {
        Self {
            role: RoleNameText::from_contract(rejection.role),
            conflicts: rejection
                .conflicts
                .into_iter()
                .map(ScopeConflict::from_contract)
                .collect(),
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ReleaseAcknowledgment {
    pub role: RoleNameText,
    pub released_scopes: Vec<ScopeReferenceText>,
}

impl ReleaseAcknowledgment {
    fn from_contract(acknowledgment: contract::ReleaseAcknowledgment) -> Self {
        Self {
            role: RoleNameText::from_contract(acknowledgment.role),
            released_scopes: acknowledgment
                .released_scopes
                .into_iter()
                .map(ScopeReferenceText::from_contract)
                .collect(),
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ClaimEntry {
    pub scope: ScopeReferenceText,
    pub reason: String,
}

impl ClaimEntry {
    fn from_contract(entry: contract::ClaimEntry) -> Self {
        Self {
            scope: ScopeReferenceText::from_contract(entry.scope),
            reason: entry.reason.as_str().to_string(),
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RoleStatus {
    pub role: RoleNameText,
    pub claims: Vec<ClaimEntry>,
}

impl RoleStatus {
    fn from_contract(status: contract::RoleStatus) -> Self {
        Self {
            role: RoleNameText::from_contract(status.role),
            claims: status
                .claims
                .into_iter()
                .map(ClaimEntry::from_contract)
                .collect(),
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Activity {
    pub role: RoleNameText,
    pub scope: ScopeReferenceText,
    pub reason: String,
    pub stamped_at: u64,
}

impl Activity {
    fn from_contract(activity: contract::Activity) -> Self {
        Self {
            role: RoleNameText::from_contract(activity.role),
            scope: ScopeReferenceText::from_contract(activity.scope),
            reason: activity.reason.as_str().to_string(),
            stamped_at: activity.stamped_at.value(),
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RoleSnapshot {
    pub roles: Vec<RoleStatus>,
    pub recent_activity: Vec<Activity>,
}

impl RoleSnapshot {
    fn from_contract(snapshot: contract::RoleSnapshot) -> Self {
        Self {
            roles: snapshot
                .roles
                .into_iter()
                .map(RoleStatus::from_contract)
                .collect(),
            recent_activity: snapshot
                .recent_activity
                .into_iter()
                .map(Activity::from_contract)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MindTextReply {
    ClaimAcceptance(ClaimAcceptance),
    ClaimRejection(ClaimRejection),
    ReleaseAcknowledgment(ReleaseAcknowledgment),
    RoleSnapshot(RoleSnapshot),
}

impl MindTextReply {
    pub fn from_reply(reply: contract::MindReply) -> Result<Self> {
        match reply {
            contract::MindReply::ClaimAcceptance(acceptance) => Ok(Self::ClaimAcceptance(
                ClaimAcceptance::from_contract(acceptance),
            )),
            contract::MindReply::ClaimRejection(rejection) => Ok(Self::ClaimRejection(
                ClaimRejection::from_contract(rejection),
            )),
            contract::MindReply::ReleaseAcknowledgment(acknowledgment) => Ok(
                Self::ReleaseAcknowledgment(ReleaseAcknowledgment::from_contract(acknowledgment)),
            ),
            contract::MindReply::RoleSnapshot(snapshot) => {
                Ok(Self::RoleSnapshot(RoleSnapshot::from_contract(snapshot)))
            }
            contract::MindReply::HandoffAcceptance(_) => Err(Error::UnsupportedTextReply {
                reply: "HandoffAcceptance",
            }),
            contract::MindReply::HandoffRejection(_) => Err(Error::UnsupportedTextReply {
                reply: "HandoffRejection",
            }),
            contract::MindReply::ActivityAcknowledgment(_) => Err(Error::UnsupportedTextReply {
                reply: "ActivityAcknowledgment",
            }),
            contract::MindReply::ActivityList(_) => Err(Error::UnsupportedTextReply {
                reply: "ActivityList",
            }),
            contract::MindReply::OpeningReceipt(_) => Err(Error::UnsupportedTextReply {
                reply: "OpeningReceipt",
            }),
            contract::MindReply::NoteReceipt(_) => Err(Error::UnsupportedTextReply {
                reply: "NoteReceipt",
            }),
            contract::MindReply::LinkReceipt(_) => Err(Error::UnsupportedTextReply {
                reply: "LinkReceipt",
            }),
            contract::MindReply::StatusReceipt(_) => Err(Error::UnsupportedTextReply {
                reply: "StatusReceipt",
            }),
            contract::MindReply::AliasReceipt(_) => Err(Error::UnsupportedTextReply {
                reply: "AliasReceipt",
            }),
            contract::MindReply::View(_) => Err(Error::UnsupportedTextReply { reply: "View" }),
            contract::MindReply::Rejection(_) => {
                Err(Error::UnsupportedTextReply { reply: "Rejection" })
            }
        }
    }

    pub fn to_nota(&self) -> Result<String> {
        let mut encoder = Encoder::new();
        self.encode(&mut encoder)?;
        Ok(encoder.into_string())
    }
}

impl NotaEncode for MindTextReply {
    fn encode(&self, encoder: &mut Encoder) -> nota_codec::Result<()> {
        match self {
            Self::ClaimAcceptance(acceptance) => acceptance.encode(encoder),
            Self::ClaimRejection(rejection) => rejection.encode(encoder),
            Self::ReleaseAcknowledgment(acknowledgment) => acknowledgment.encode(encoder),
            Self::RoleSnapshot(snapshot) => snapshot.encode(encoder),
        }
    }
}

struct MindTextEnd<'decoder, 'input> {
    decoder: &'decoder mut Decoder<'input>,
}

impl<'decoder, 'input> MindTextEnd<'decoder, 'input> {
    fn new(decoder: &'decoder mut Decoder<'input>) -> Self {
        Self { decoder }
    }

    fn expect(&mut self) -> nota_codec::Result<()> {
        if let Some(token) = self.decoder.peek_token()? {
            Err(nota_codec::Error::UnexpectedToken {
                expected: "end of input",
                got: token,
            })
        } else {
            Ok(())
        }
    }
}
