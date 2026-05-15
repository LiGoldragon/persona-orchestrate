use nota_codec::{Decoder, Encoder, NotaDecode, NotaEncode, NotaEnum, NotaRecord};
use signal_persona_mind as contract;

use crate::Result;

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

#[derive(NotaEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKindText {
    Task,
    Defect,
    Question,
    Decision,
    Note,
    Handoff,
}

impl ItemKindText {
    fn from_contract(kind: contract::ItemKind) -> Self {
        match kind {
            contract::ItemKind::Task => Self::Task,
            contract::ItemKind::Defect => Self::Defect,
            contract::ItemKind::Question => Self::Question,
            contract::ItemKind::Decision => Self::Decision,
            contract::ItemKind::Note => Self::Note,
            contract::ItemKind::Handoff => Self::Handoff,
        }
    }

    fn into_contract(self) -> contract::ItemKind {
        match self {
            Self::Task => contract::ItemKind::Task,
            Self::Defect => contract::ItemKind::Defect,
            Self::Question => contract::ItemKind::Question,
            Self::Decision => contract::ItemKind::Decision,
            Self::Note => contract::ItemKind::Note,
            Self::Handoff => contract::ItemKind::Handoff,
        }
    }
}

#[derive(NotaEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemPriorityText {
    Critical,
    High,
    Normal,
    Low,
    Backlog,
}

impl ItemPriorityText {
    fn from_contract(priority: contract::ItemPriority) -> Self {
        match priority {
            contract::ItemPriority::Critical => Self::Critical,
            contract::ItemPriority::High => Self::High,
            contract::ItemPriority::Normal => Self::Normal,
            contract::ItemPriority::Low => Self::Low,
            contract::ItemPriority::Backlog => Self::Backlog,
        }
    }

    fn into_contract(self) -> contract::ItemPriority {
        match self {
            Self::Critical => contract::ItemPriority::Critical,
            Self::High => contract::ItemPriority::High,
            Self::Normal => contract::ItemPriority::Normal,
            Self::Low => contract::ItemPriority::Low,
            Self::Backlog => contract::ItemPriority::Backlog,
        }
    }
}

#[derive(NotaEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemStatusText {
    Open,
    InProgress,
    Blocked,
    Closed,
    Deferred,
}

impl ItemStatusText {
    fn from_contract(status: contract::ItemStatus) -> Self {
        match status {
            contract::ItemStatus::Open => Self::Open,
            contract::ItemStatus::InProgress => Self::InProgress,
            contract::ItemStatus::Blocked => Self::Blocked,
            contract::ItemStatus::Closed => Self::Closed,
            contract::ItemStatus::Deferred => Self::Deferred,
        }
    }

    fn into_contract(self) -> contract::ItemStatus {
        match self {
            Self::Open => contract::ItemStatus::Open,
            Self::InProgress => contract::ItemStatus::InProgress,
            Self::Blocked => contract::ItemStatus::Blocked,
            Self::Closed => contract::ItemStatus::Closed,
            Self::Deferred => contract::ItemStatus::Deferred,
        }
    }
}

#[derive(NotaEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeKindText {
    DependsOn,
    ParentOf,
    RelatesTo,
    Duplicates,
    Supersedes,
    Answers,
    References,
}

impl EdgeKindText {
    fn from_contract(kind: contract::EdgeKind) -> Self {
        match kind {
            contract::EdgeKind::DependsOn => Self::DependsOn,
            contract::EdgeKind::ParentOf => Self::ParentOf,
            contract::EdgeKind::RelatesTo => Self::RelatesTo,
            contract::EdgeKind::Duplicates => Self::Duplicates,
            contract::EdgeKind::Supersedes => Self::Supersedes,
            contract::EdgeKind::Answers => Self::Answers,
            contract::EdgeKind::References => Self::References,
        }
    }

    fn into_contract(self) -> contract::EdgeKind {
        match self {
            Self::DependsOn => contract::EdgeKind::DependsOn,
            Self::ParentOf => contract::EdgeKind::ParentOf,
            Self::RelatesTo => contract::EdgeKind::RelatesTo,
            Self::Duplicates => contract::EdgeKind::Duplicates,
            Self::Supersedes => contract::EdgeKind::Supersedes,
            Self::Answers => contract::EdgeKind::Answers,
            Self::References => contract::EdgeKind::References,
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

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RoleHandoff {
    pub from: RoleNameText,
    pub to: RoleNameText,
    pub scopes: Vec<ScopeReferenceText>,
    pub reason: String,
}

impl RoleHandoff {
    fn into_contract(self) -> Result<contract::MindRequest> {
        let scopes = self
            .scopes
            .into_iter()
            .map(ScopeReferenceText::into_contract)
            .collect::<Result<Vec<_>>>()?;
        Ok(contract::MindRequest::RoleHandoff(contract::RoleHandoff {
            from: self.from.into_contract(),
            to: self.to.into_contract(),
            scopes,
            reason: contract::ScopeReason::from_text(self.reason)?,
        }))
    }
}

#[derive(NotaRecord, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoleObservation {}

impl RoleObservation {
    fn into_contract(self) -> contract::MindRequest {
        contract::MindRequest::RoleObservation(contract::RoleObservation)
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ActivitySubmission {
    pub role: RoleNameText,
    pub scope: ScopeReferenceText,
    pub reason: String,
}

impl ActivitySubmission {
    fn into_contract(self) -> Result<contract::MindRequest> {
        Ok(contract::MindRequest::ActivitySubmission(
            contract::ActivitySubmission {
                role: self.role.into_contract(),
                scope: self.scope.into_contract()?,
                reason: contract::ScopeReason::from_text(self.reason)?,
            },
        ))
    }
}

#[derive(NotaRecord, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoleFilter {
    pub role: RoleNameText,
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct PathPrefix {
    pub path: String,
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct TaskFilter {
    pub token: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivityFilterText {
    RoleFilter(RoleFilter),
    PathPrefix(PathPrefix),
    TaskFilter(TaskFilter),
}

impl ActivityFilterText {
    fn into_contract(self) -> Result<contract::ActivityFilter> {
        match self {
            Self::RoleFilter(filter) => Ok(contract::ActivityFilter::RoleFilter(
                filter.role.into_contract(),
            )),
            Self::PathPrefix(prefix) => Ok(contract::ActivityFilter::PathPrefix(
                contract::WirePath::from_absolute_path(prefix.path)?,
            )),
            Self::TaskFilter(filter) => Ok(contract::ActivityFilter::TaskToken(
                contract::TaskToken::from_wire_token(filter.token)?,
            )),
        }
    }
}

impl NotaEncode for ActivityFilterText {
    fn encode(&self, encoder: &mut Encoder) -> nota_codec::Result<()> {
        match self {
            Self::RoleFilter(filter) => filter.encode(encoder),
            Self::PathPrefix(prefix) => prefix.encode(encoder),
            Self::TaskFilter(filter) => filter.encode(encoder),
        }
    }
}

impl NotaDecode for ActivityFilterText {
    fn decode(decoder: &mut Decoder<'_>) -> nota_codec::Result<Self> {
        match decoder.peek_record_head()?.as_str() {
            "RoleFilter" => Ok(Self::RoleFilter(RoleFilter::decode(decoder)?)),
            "PathPrefix" => Ok(Self::PathPrefix(PathPrefix::decode(decoder)?)),
            "TaskFilter" => Ok(Self::TaskFilter(TaskFilter::decode(decoder)?)),
            other => Err(nota_codec::Error::UnknownKindForVerb {
                verb: "ActivityFilter",
                got: other.to_string(),
            }),
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ActivityQuery {
    pub limit: u32,
    pub filters: Vec<ActivityFilterText>,
}

impl ActivityQuery {
    fn into_contract(self) -> Result<contract::MindRequest> {
        let filters = self
            .filters
            .into_iter()
            .map(ActivityFilterText::into_contract)
            .collect::<Result<Vec<_>>>()?;
        Ok(contract::MindRequest::ActivityQuery(
            contract::ActivityQuery {
                limit: self.limit,
                filters,
            },
        ))
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Opening {
    pub kind: ItemKindText,
    pub priority: ItemPriorityText,
    pub title: String,
    pub body: String,
}

impl Opening {
    fn into_contract(self) -> contract::MindRequest {
        contract::MindRequest::Opening(contract::Opening {
            kind: self.kind.into_contract(),
            priority: self.priority.into_contract(),
            title: contract::Title::new(self.title),
            body: contract::TextBody::new(self.body),
        })
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Stable {
    pub id: String,
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Display {
    pub id: String,
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Alias {
    pub alias: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemReferenceText {
    Stable(Stable),
    Display(Display),
    Alias(Alias),
}

impl ItemReferenceText {
    fn into_contract(self) -> contract::ItemReference {
        match self {
            Self::Stable(stable) => {
                contract::ItemReference::Stable(contract::StableItemId::new(stable.id))
            }
            Self::Display(display) => {
                contract::ItemReference::Display(contract::DisplayId::new(display.id))
            }
            Self::Alias(alias) => {
                contract::ItemReference::Alias(contract::ExternalAlias::new(alias.alias))
            }
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ItemReferenceTarget {
    pub item: ItemReferenceText,
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub path: String,
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct GitCommit {
    pub hash: String,
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct BeadsTask {
    pub token: String,
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct File {
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkTargetText {
    ItemReferenceTarget(ItemReferenceTarget),
    Report(Report),
    GitCommit(GitCommit),
    BeadsTask(BeadsTask),
    File(File),
}

impl LinkTargetText {
    fn into_contract(self) -> contract::LinkTarget {
        match self {
            Self::ItemReferenceTarget(target) => {
                contract::LinkTarget::Item(target.item.into_contract())
            }
            Self::Report(report) => contract::LinkTarget::External(
                contract::ExternalReference::Report(contract::ReportPath::new(report.path)),
            ),
            Self::GitCommit(commit) => contract::LinkTarget::External(
                contract::ExternalReference::GitCommit(contract::CommitHash::new(commit.hash)),
            ),
            Self::BeadsTask(task) => contract::LinkTarget::External(
                contract::ExternalReference::BeadsTask(contract::BeadsToken::new(task.token)),
            ),
            Self::File(file) => contract::LinkTarget::External(contract::ExternalReference::File(
                contract::ReferencePath::new(file.path),
            )),
        }
    }
}

impl NotaEncode for LinkTargetText {
    fn encode(&self, encoder: &mut Encoder) -> nota_codec::Result<()> {
        match self {
            Self::ItemReferenceTarget(target) => target.encode(encoder),
            Self::Report(report) => report.encode(encoder),
            Self::GitCommit(commit) => commit.encode(encoder),
            Self::BeadsTask(task) => task.encode(encoder),
            Self::File(file) => file.encode(encoder),
        }
    }
}

impl NotaDecode for LinkTargetText {
    fn decode(decoder: &mut Decoder<'_>) -> nota_codec::Result<Self> {
        match decoder.peek_record_head()?.as_str() {
            "ItemReferenceTarget" => Ok(Self::ItemReferenceTarget(ItemReferenceTarget::decode(
                decoder,
            )?)),
            "Report" => Ok(Self::Report(Report::decode(decoder)?)),
            "GitCommit" => Ok(Self::GitCommit(GitCommit::decode(decoder)?)),
            "BeadsTask" => Ok(Self::BeadsTask(BeadsTask::decode(decoder)?)),
            "File" => Ok(Self::File(File::decode(decoder)?)),
            other => Err(nota_codec::Error::UnknownKindForVerb {
                verb: "LinkTarget",
                got: other.to_string(),
            }),
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct NoteSubmission {
    pub item: ItemReferenceText,
    pub body: String,
}

impl NoteSubmission {
    fn into_contract(self) -> contract::MindRequest {
        contract::MindRequest::NoteSubmission(contract::NoteSubmission {
            item: self.item.into_contract(),
            body: contract::TextBody::new(self.body),
        })
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub source: ItemReferenceText,
    pub kind: EdgeKindText,
    pub target: LinkTargetText,
    pub body: Option<String>,
}

impl Link {
    fn into_contract(self) -> contract::MindRequest {
        contract::MindRequest::Link(contract::Link {
            source: self.source.into_contract(),
            kind: self.kind.into_contract(),
            target: self.target.into_contract(),
            body: self.body.map(contract::TextBody::new),
        })
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct StatusChange {
    pub item: ItemReferenceText,
    pub status: ItemStatusText,
    pub body: Option<String>,
}

impl StatusChange {
    fn into_contract(self) -> contract::MindRequest {
        contract::MindRequest::StatusChange(contract::StatusChange {
            item: self.item.into_contract(),
            status: self.status.into_contract(),
            body: self.body.map(contract::TextBody::new),
        })
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct AliasAssignment {
    pub item: ItemReferenceText,
    pub alias: String,
}

impl AliasAssignment {
    fn into_contract(self) -> contract::MindRequest {
        contract::MindRequest::AliasAssignment(contract::AliasAssignment {
            item: self.item.into_contract(),
            alias: contract::ExternalAlias::new(self.alias),
        })
    }
}

impl NotaEncode for ItemReferenceText {
    fn encode(&self, encoder: &mut Encoder) -> nota_codec::Result<()> {
        match self {
            Self::Stable(stable) => stable.encode(encoder),
            Self::Display(display) => display.encode(encoder),
            Self::Alias(alias) => alias.encode(encoder),
        }
    }
}

impl NotaDecode for ItemReferenceText {
    fn decode(decoder: &mut Decoder<'_>) -> nota_codec::Result<Self> {
        match decoder.peek_record_head()?.as_str() {
            "Stable" => Ok(Self::Stable(Stable::decode(decoder)?)),
            "Display" => Ok(Self::Display(Display::decode(decoder)?)),
            "Alias" => Ok(Self::Alias(Alias::decode(decoder)?)),
            other => Err(nota_codec::Error::UnknownKindForVerb {
                verb: "ItemReference",
                got: other.to_string(),
            }),
        }
    }
}

#[derive(NotaRecord, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ready {}

#[derive(NotaRecord, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Blocked {}

#[derive(NotaRecord, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Open {}

#[derive(NotaRecord, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecentEvents {}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ByItem {
    pub item: ItemReferenceText,
}

#[derive(NotaRecord, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByKind {
    pub kind: ItemKindText,
}

#[derive(NotaRecord, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByStatus {
    pub status: ItemStatusText,
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ByAlias {
    pub alias: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryKindText {
    Ready(Ready),
    Blocked(Blocked),
    Open(Open),
    RecentEvents(RecentEvents),
    ByItem(ByItem),
    ByKind(ByKind),
    ByStatus(ByStatus),
    ByAlias(ByAlias),
}

impl QueryKindText {
    fn into_contract(self) -> contract::QueryKind {
        match self {
            Self::Ready(_) => contract::QueryKind::Ready,
            Self::Blocked(_) => contract::QueryKind::Blocked,
            Self::Open(_) => contract::QueryKind::Open,
            Self::RecentEvents(_) => contract::QueryKind::RecentEvents,
            Self::ByItem(query) => contract::QueryKind::ByItem(query.item.into_contract()),
            Self::ByKind(query) => contract::QueryKind::ByKind(query.kind.into_contract()),
            Self::ByStatus(query) => contract::QueryKind::ByStatus(query.status.into_contract()),
            Self::ByAlias(query) => {
                contract::QueryKind::ByAlias(contract::ExternalAlias::new(query.alias))
            }
        }
    }
}

impl NotaEncode for QueryKindText {
    fn encode(&self, encoder: &mut Encoder) -> nota_codec::Result<()> {
        match self {
            Self::Ready(query) => query.encode(encoder),
            Self::Blocked(query) => query.encode(encoder),
            Self::Open(query) => query.encode(encoder),
            Self::RecentEvents(query) => query.encode(encoder),
            Self::ByItem(query) => query.encode(encoder),
            Self::ByKind(query) => query.encode(encoder),
            Self::ByStatus(query) => query.encode(encoder),
            Self::ByAlias(query) => query.encode(encoder),
        }
    }
}

impl NotaDecode for QueryKindText {
    fn decode(decoder: &mut Decoder<'_>) -> nota_codec::Result<Self> {
        match decoder.peek_record_head()?.as_str() {
            "Ready" => Ok(Self::Ready(Ready::decode(decoder)?)),
            "Blocked" => Ok(Self::Blocked(Blocked::decode(decoder)?)),
            "Open" => Ok(Self::Open(Open::decode(decoder)?)),
            "RecentEvents" => Ok(Self::RecentEvents(RecentEvents::decode(decoder)?)),
            "ByItem" => Ok(Self::ByItem(ByItem::decode(decoder)?)),
            "ByKind" => Ok(Self::ByKind(ByKind::decode(decoder)?)),
            "ByStatus" => Ok(Self::ByStatus(ByStatus::decode(decoder)?)),
            "ByAlias" => Ok(Self::ByAlias(ByAlias::decode(decoder)?)),
            other => Err(nota_codec::Error::UnknownKindForVerb {
                verb: "QueryKind",
                got: other.to_string(),
            }),
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Query {
    pub kind: QueryKindText,
    pub limit: u16,
}

impl Query {
    fn into_contract(self) -> contract::MindRequest {
        contract::MindRequest::Query(contract::Query {
            kind: self.kind.into_contract(),
            limit: contract::QueryLimit::new(self.limit),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MindTextRequest {
    RoleClaim(RoleClaim),
    RoleRelease(RoleRelease),
    RoleHandoff(RoleHandoff),
    RoleObservation(RoleObservation),
    ActivitySubmission(ActivitySubmission),
    ActivityQuery(ActivityQuery),
    Opening(Opening),
    NoteSubmission(NoteSubmission),
    Link(Link),
    StatusChange(StatusChange),
    AliasAssignment(AliasAssignment),
    Query(Query),
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
            Self::RoleHandoff(handoff) => handoff.into_contract(),
            Self::RoleObservation(observation) => Ok(observation.into_contract()),
            Self::ActivitySubmission(submission) => submission.into_contract(),
            Self::ActivityQuery(query) => query.into_contract(),
            Self::Opening(opening) => Ok(opening.into_contract()),
            Self::NoteSubmission(submission) => Ok(submission.into_contract()),
            Self::Link(link) => Ok(link.into_contract()),
            Self::StatusChange(change) => Ok(change.into_contract()),
            Self::AliasAssignment(assignment) => Ok(assignment.into_contract()),
            Self::Query(query) => Ok(query.into_contract()),
        }
    }
}

impl NotaEncode for MindTextRequest {
    fn encode(&self, encoder: &mut Encoder) -> nota_codec::Result<()> {
        match self {
            Self::RoleClaim(claim) => claim.encode(encoder),
            Self::RoleRelease(release) => release.encode(encoder),
            Self::RoleHandoff(handoff) => handoff.encode(encoder),
            Self::RoleObservation(observation) => observation.encode(encoder),
            Self::ActivitySubmission(submission) => submission.encode(encoder),
            Self::ActivityQuery(query) => query.encode(encoder),
            Self::Opening(opening) => opening.encode(encoder),
            Self::NoteSubmission(submission) => submission.encode(encoder),
            Self::Link(link) => link.encode(encoder),
            Self::StatusChange(change) => change.encode(encoder),
            Self::AliasAssignment(assignment) => assignment.encode(encoder),
            Self::Query(query) => query.encode(encoder),
        }
    }
}

impl NotaDecode for MindTextRequest {
    fn decode(decoder: &mut Decoder<'_>) -> nota_codec::Result<Self> {
        match decoder.peek_record_head()?.as_str() {
            "RoleClaim" => Ok(Self::RoleClaim(RoleClaim::decode(decoder)?)),
            "RoleRelease" => Ok(Self::RoleRelease(RoleRelease::decode(decoder)?)),
            "RoleHandoff" => Ok(Self::RoleHandoff(RoleHandoff::decode(decoder)?)),
            "RoleObservation" => Ok(Self::RoleObservation(RoleObservation::decode(decoder)?)),
            "ActivitySubmission" => Ok(Self::ActivitySubmission(ActivitySubmission::decode(
                decoder,
            )?)),
            "ActivityQuery" => Ok(Self::ActivityQuery(ActivityQuery::decode(decoder)?)),
            "Opening" => Ok(Self::Opening(Opening::decode(decoder)?)),
            "NoteSubmission" => Ok(Self::NoteSubmission(NoteSubmission::decode(decoder)?)),
            "Link" => Ok(Self::Link(Link::decode(decoder)?)),
            "StatusChange" => Ok(Self::StatusChange(StatusChange::decode(decoder)?)),
            "AliasAssignment" => Ok(Self::AliasAssignment(AliasAssignment::decode(decoder)?)),
            "Query" => Ok(Self::Query(Query::decode(decoder)?)),
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
pub struct HandoffAcceptance {
    pub from: RoleNameText,
    pub to: RoleNameText,
    pub scopes: Vec<ScopeReferenceText>,
}

impl HandoffAcceptance {
    fn from_contract(acceptance: contract::HandoffAcceptance) -> Self {
        Self {
            from: RoleNameText::from_contract(acceptance.from),
            to: RoleNameText::from_contract(acceptance.to),
            scopes: acceptance
                .scopes
                .into_iter()
                .map(ScopeReferenceText::from_contract)
                .collect(),
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct SourceRoleDoesNotHold {}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct TargetRoleConflict {
    pub conflicts: Vec<ScopeConflict>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandoffRejectionReason {
    SourceRoleDoesNotHold(SourceRoleDoesNotHold),
    TargetRoleConflict(TargetRoleConflict),
}

impl HandoffRejectionReason {
    fn from_contract(reason: contract::HandoffRejectionReason) -> Self {
        match reason {
            contract::HandoffRejectionReason::SourceRoleDoesNotHold => {
                Self::SourceRoleDoesNotHold(SourceRoleDoesNotHold {})
            }
            contract::HandoffRejectionReason::TargetRoleConflict(conflicts) => {
                Self::TargetRoleConflict(TargetRoleConflict {
                    conflicts: conflicts
                        .into_iter()
                        .map(ScopeConflict::from_contract)
                        .collect(),
                })
            }
        }
    }
}

impl NotaEncode for HandoffRejectionReason {
    fn encode(&self, encoder: &mut Encoder) -> nota_codec::Result<()> {
        match self {
            Self::SourceRoleDoesNotHold(reason) => reason.encode(encoder),
            Self::TargetRoleConflict(reason) => reason.encode(encoder),
        }
    }
}

impl NotaDecode for HandoffRejectionReason {
    fn decode(decoder: &mut Decoder<'_>) -> nota_codec::Result<Self> {
        match decoder.peek_record_head()?.as_str() {
            "SourceRoleDoesNotHold" => Ok(Self::SourceRoleDoesNotHold(
                SourceRoleDoesNotHold::decode(decoder)?,
            )),
            "TargetRoleConflict" => Ok(Self::TargetRoleConflict(TargetRoleConflict::decode(
                decoder,
            )?)),
            other => Err(nota_codec::Error::UnknownKindForVerb {
                verb: "HandoffRejectionReason",
                got: other.to_string(),
            }),
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct HandoffRejection {
    pub from: RoleNameText,
    pub to: RoleNameText,
    pub reason: HandoffRejectionReason,
}

impl HandoffRejection {
    fn from_contract(rejection: contract::HandoffRejection) -> Self {
        Self {
            from: RoleNameText::from_contract(rejection.from),
            to: RoleNameText::from_contract(rejection.to),
            reason: HandoffRejectionReason::from_contract(rejection.reason),
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

#[derive(NotaRecord, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActivityAcknowledgment {
    pub slot: u64,
}

impl ActivityAcknowledgment {
    fn from_contract(acknowledgment: contract::ActivityAcknowledgment) -> Self {
        Self {
            slot: acknowledgment.slot,
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ActivityList {
    pub records: Vec<Activity>,
}

impl ActivityList {
    fn from_contract(list: contract::ActivityList) -> Self {
        Self {
            records: list
                .records
                .into_iter()
                .map(Activity::from_contract)
                .collect(),
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub id: String,
    pub display_id: String,
    pub aliases: Vec<String>,
    pub kind: ItemKindText,
    pub status: ItemStatusText,
    pub priority: ItemPriorityText,
    pub title: String,
    pub body: String,
}

impl Item {
    fn from_contract(item: contract::Item) -> Self {
        Self {
            id: item.id.as_str().to_string(),
            display_id: item.display_id.as_str().to_string(),
            aliases: item
                .aliases
                .into_iter()
                .map(|alias| alias.as_str().to_string())
                .collect(),
            kind: ItemKindText::from_contract(item.kind),
            status: ItemStatusText::from_contract(item.status),
            priority: ItemPriorityText::from_contract(item.priority),
            title: item.title.as_str().to_string(),
            body: item.body.as_str().to_string(),
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub event: u64,
    pub item: String,
    pub author: String,
    pub body: String,
}

impl Note {
    fn from_contract(note: contract::Note) -> Self {
        Self {
            event: note.event.into_u64(),
            item: note.item.as_str().to_string(),
            author: note.author.as_str().to_string(),
            body: note.body.as_str().to_string(),
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ItemTarget {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeTargetText {
    ItemTarget(ItemTarget),
    Report(Report),
    GitCommit(GitCommit),
    BeadsTask(BeadsTask),
    File(File),
}

impl EdgeTargetText {
    fn from_contract(target: contract::EdgeTarget) -> Self {
        match target {
            contract::EdgeTarget::Item(id) => Self::ItemTarget(ItemTarget {
                id: id.as_str().to_string(),
            }),
            contract::EdgeTarget::External(external) => match external {
                contract::ExternalReference::Report(path) => Self::Report(Report {
                    path: path.as_str().to_string(),
                }),
                contract::ExternalReference::GitCommit(hash) => Self::GitCommit(GitCommit {
                    hash: hash.as_str().to_string(),
                }),
                contract::ExternalReference::BeadsTask(token) => Self::BeadsTask(BeadsTask {
                    token: token.as_str().to_string(),
                }),
                contract::ExternalReference::File(path) => Self::File(File {
                    path: path.as_str().to_string(),
                }),
            },
        }
    }
}

impl NotaEncode for EdgeTargetText {
    fn encode(&self, encoder: &mut Encoder) -> nota_codec::Result<()> {
        match self {
            Self::ItemTarget(target) => target.encode(encoder),
            Self::Report(report) => report.encode(encoder),
            Self::GitCommit(commit) => commit.encode(encoder),
            Self::BeadsTask(task) => task.encode(encoder),
            Self::File(file) => file.encode(encoder),
        }
    }
}

impl NotaDecode for EdgeTargetText {
    fn decode(decoder: &mut Decoder<'_>) -> nota_codec::Result<Self> {
        match decoder.peek_record_head()?.as_str() {
            "ItemTarget" => Ok(Self::ItemTarget(ItemTarget::decode(decoder)?)),
            "Report" => Ok(Self::Report(Report::decode(decoder)?)),
            "GitCommit" => Ok(Self::GitCommit(GitCommit::decode(decoder)?)),
            "BeadsTask" => Ok(Self::BeadsTask(BeadsTask::decode(decoder)?)),
            "File" => Ok(Self::File(File::decode(decoder)?)),
            other => Err(nota_codec::Error::UnknownKindForVerb {
                verb: "EdgeTarget",
                got: other.to_string(),
            }),
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pub event: u64,
    pub source: String,
    pub kind: EdgeKindText,
    pub target: EdgeTargetText,
    pub body: Option<String>,
}

impl Edge {
    fn from_contract(edge: contract::Edge) -> Self {
        Self {
            event: edge.event.into_u64(),
            source: edge.source.as_str().to_string(),
            kind: EdgeKindText::from_contract(edge.kind),
            target: EdgeTargetText::from_contract(edge.target),
            body: edge.body.map(|body| body.as_str().to_string()),
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct EventHeader {
    pub event: u64,
    pub operation: String,
    pub actor: String,
}

impl EventHeader {
    fn from_contract(header: contract::EventHeader) -> Self {
        Self {
            event: header.event.into_u64(),
            operation: header.operation.as_str().to_string(),
            actor: header.actor.as_str().to_string(),
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ItemOpenedEvent {
    pub header: EventHeader,
    pub item: Item,
}

impl ItemOpenedEvent {
    fn from_contract(event: contract::ItemOpenedEvent) -> Self {
        Self {
            header: EventHeader::from_contract(event.header),
            item: Item::from_contract(event.item),
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct NoteAddedEvent {
    pub header: EventHeader,
    pub note: Note,
}

impl NoteAddedEvent {
    fn from_contract(event: contract::NoteAddedEvent) -> Self {
        Self {
            header: EventHeader::from_contract(event.header),
            note: Note::from_contract(event.note),
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct EdgeAddedEvent {
    pub header: EventHeader,
    pub edge: Edge,
}

impl EdgeAddedEvent {
    fn from_contract(event: contract::EdgeAddedEvent) -> Self {
        Self {
            header: EventHeader::from_contract(event.header),
            edge: Edge::from_contract(event.edge),
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct StatusChangedEvent {
    pub header: EventHeader,
    pub item: String,
    pub status: ItemStatusText,
    pub body: Option<String>,
}

impl StatusChangedEvent {
    fn from_contract(event: contract::StatusChangedEvent) -> Self {
        Self {
            header: EventHeader::from_contract(event.header),
            item: event.item.as_str().to_string(),
            status: ItemStatusText::from_contract(event.status),
            body: event.body.map(|body| body.as_str().to_string()),
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct AliasAddedEvent {
    pub header: EventHeader,
    pub item: String,
    pub alias: String,
}

impl AliasAddedEvent {
    fn from_contract(event: contract::AliasAddedEvent) -> Self {
        Self {
            header: EventHeader::from_contract(event.header),
            item: event.item.as_str().to_string(),
            alias: event.alias.as_str().to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    ItemOpened(ItemOpenedEvent),
    NoteAdded(NoteAddedEvent),
    EdgeAdded(EdgeAddedEvent),
    StatusChanged(StatusChangedEvent),
    AliasAdded(AliasAddedEvent),
}

impl Event {
    fn from_contract(event: contract::Event) -> Self {
        match event {
            contract::Event::ItemOpened(event) => {
                Self::ItemOpened(ItemOpenedEvent::from_contract(event))
            }
            contract::Event::NoteAdded(event) => {
                Self::NoteAdded(NoteAddedEvent::from_contract(event))
            }
            contract::Event::EdgeAdded(event) => {
                Self::EdgeAdded(EdgeAddedEvent::from_contract(event))
            }
            contract::Event::StatusChanged(event) => {
                Self::StatusChanged(StatusChangedEvent::from_contract(event))
            }
            contract::Event::AliasAdded(event) => {
                Self::AliasAdded(AliasAddedEvent::from_contract(event))
            }
        }
    }
}

impl NotaEncode for Event {
    fn encode(&self, encoder: &mut Encoder) -> nota_codec::Result<()> {
        match self {
            Self::ItemOpened(event) => event.encode(encoder),
            Self::NoteAdded(event) => event.encode(encoder),
            Self::EdgeAdded(event) => event.encode(encoder),
            Self::StatusChanged(event) => event.encode(encoder),
            Self::AliasAdded(event) => event.encode(encoder),
        }
    }
}

impl NotaDecode for Event {
    fn decode(decoder: &mut Decoder<'_>) -> nota_codec::Result<Self> {
        match decoder.peek_record_head()?.as_str() {
            "ItemOpenedEvent" => Ok(Self::ItemOpened(ItemOpenedEvent::decode(decoder)?)),
            "NoteAddedEvent" => Ok(Self::NoteAdded(NoteAddedEvent::decode(decoder)?)),
            "EdgeAddedEvent" => Ok(Self::EdgeAdded(EdgeAddedEvent::decode(decoder)?)),
            "StatusChangedEvent" => Ok(Self::StatusChanged(StatusChangedEvent::decode(decoder)?)),
            "AliasAddedEvent" => Ok(Self::AliasAdded(AliasAddedEvent::decode(decoder)?)),
            other => Err(nota_codec::Error::UnknownKindForVerb {
                verb: "Event",
                got: other.to_string(),
            }),
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct OpeningReceipt {
    pub event: ItemOpenedEvent,
}

impl OpeningReceipt {
    fn from_contract(receipt: contract::OpeningReceipt) -> Self {
        Self {
            event: ItemOpenedEvent::from_contract(receipt.event),
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct NoteReceipt {
    pub event: NoteAddedEvent,
}

impl NoteReceipt {
    fn from_contract(receipt: contract::NoteReceipt) -> Self {
        Self {
            event: NoteAddedEvent::from_contract(receipt.event),
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct LinkReceipt {
    pub event: EdgeAddedEvent,
}

impl LinkReceipt {
    fn from_contract(receipt: contract::LinkReceipt) -> Self {
        Self {
            event: EdgeAddedEvent::from_contract(receipt.event),
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct StatusReceipt {
    pub event: StatusChangedEvent,
}

impl StatusReceipt {
    fn from_contract(receipt: contract::StatusReceipt) -> Self {
        Self {
            event: StatusChangedEvent::from_contract(receipt.event),
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct AliasReceipt {
    pub event: AliasAddedEvent,
}

impl AliasReceipt {
    fn from_contract(receipt: contract::AliasReceipt) -> Self {
        Self {
            event: AliasAddedEvent::from_contract(receipt.event),
        }
    }
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct View {
    pub items: Vec<Item>,
    pub edges: Vec<Edge>,
    pub notes: Vec<Note>,
    pub events: Vec<Event>,
}

impl View {
    fn from_contract(view: contract::View) -> Self {
        Self {
            items: view.items.into_iter().map(Item::from_contract).collect(),
            edges: view.edges.into_iter().map(Edge::from_contract).collect(),
            notes: view.notes.into_iter().map(Note::from_contract).collect(),
            events: view.events.into_iter().map(Event::from_contract).collect(),
        }
    }
}

#[derive(NotaEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectionReasonText {
    UnknownItem,
    DuplicateAlias,
    InvalidEdge,
    PersistenceRejected,
    UnsupportedQuery,
    CollisionUnresolved,
}

impl RejectionReasonText {
    fn from_contract(reason: contract::RejectionReason) -> Self {
        match reason {
            contract::RejectionReason::UnknownItem => Self::UnknownItem,
            contract::RejectionReason::DuplicateAlias => Self::DuplicateAlias,
            contract::RejectionReason::InvalidEdge => Self::InvalidEdge,
            contract::RejectionReason::PersistenceRejected => Self::PersistenceRejected,
            contract::RejectionReason::UnsupportedQuery => Self::UnsupportedQuery,
            contract::RejectionReason::CollisionUnresolved => Self::CollisionUnresolved,
        }
    }
}

#[derive(NotaRecord, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rejection {
    pub reason: RejectionReasonText,
}

impl Rejection {
    fn from_contract(rejection: contract::Rejection) -> Self {
        Self {
            reason: RejectionReasonText::from_contract(rejection.reason),
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
    HandoffAcceptance(HandoffAcceptance),
    HandoffRejection(HandoffRejection),
    RoleSnapshot(RoleSnapshot),
    ActivityAcknowledgment(ActivityAcknowledgment),
    ActivityList(ActivityList),
    OpeningReceipt(OpeningReceipt),
    NoteReceipt(NoteReceipt),
    LinkReceipt(LinkReceipt),
    StatusReceipt(StatusReceipt),
    AliasReceipt(AliasReceipt),
    View(View),
    Rejection(Rejection),
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
            contract::MindReply::HandoffAcceptance(acceptance) => Ok(Self::HandoffAcceptance(
                HandoffAcceptance::from_contract(acceptance),
            )),
            contract::MindReply::HandoffRejection(rejection) => Ok(Self::HandoffRejection(
                HandoffRejection::from_contract(rejection),
            )),
            contract::MindReply::RoleSnapshot(snapshot) => {
                Ok(Self::RoleSnapshot(RoleSnapshot::from_contract(snapshot)))
            }
            contract::MindReply::ActivityAcknowledgment(acknowledgment) => Ok(
                Self::ActivityAcknowledgment(ActivityAcknowledgment::from_contract(acknowledgment)),
            ),
            contract::MindReply::ActivityList(list) => {
                Ok(Self::ActivityList(ActivityList::from_contract(list)))
            }
            contract::MindReply::OpeningReceipt(receipt) => {
                Ok(Self::OpeningReceipt(OpeningReceipt::from_contract(receipt)))
            }
            contract::MindReply::NoteReceipt(receipt) => {
                Ok(Self::NoteReceipt(NoteReceipt::from_contract(receipt)))
            }
            contract::MindReply::LinkReceipt(receipt) => {
                Ok(Self::LinkReceipt(LinkReceipt::from_contract(receipt)))
            }
            contract::MindReply::StatusReceipt(receipt) => {
                Ok(Self::StatusReceipt(StatusReceipt::from_contract(receipt)))
            }
            contract::MindReply::AliasReceipt(receipt) => {
                Ok(Self::AliasReceipt(AliasReceipt::from_contract(receipt)))
            }
            contract::MindReply::View(view) => Ok(Self::View(View::from_contract(view))),
            contract::MindReply::Rejection(rejection) => {
                Ok(Self::Rejection(Rejection::from_contract(rejection)))
            }
            contract::MindReply::ThoughtCommitted(_)
            | contract::MindReply::RelationCommitted(_)
            | contract::MindReply::ThoughtList(_)
            | contract::MindReply::RelationList(_)
            | contract::MindReply::SubscriptionAccepted(_)
            | contract::MindReply::AdjudicationReceipt(_)
            | contract::MindReply::ChannelReceipt(_)
            | contract::MindReply::AdjudicationDenyReceipt(_)
            | contract::MindReply::ChannelListView(_)
            | contract::MindReply::MindRequestUnimplemented(_) => Err(
                crate::Error::UnexpectedFrame("mind reply has no MindTextReply projection"),
            ),
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
            Self::HandoffAcceptance(acceptance) => acceptance.encode(encoder),
            Self::HandoffRejection(rejection) => rejection.encode(encoder),
            Self::RoleSnapshot(snapshot) => snapshot.encode(encoder),
            Self::ActivityAcknowledgment(acknowledgment) => acknowledgment.encode(encoder),
            Self::ActivityList(list) => list.encode(encoder),
            Self::OpeningReceipt(receipt) => receipt.encode(encoder),
            Self::NoteReceipt(receipt) => receipt.encode(encoder),
            Self::LinkReceipt(receipt) => receipt.encode(encoder),
            Self::StatusReceipt(receipt) => receipt.encode(encoder),
            Self::AliasReceipt(receipt) => receipt.encode(encoder),
            Self::View(view) => view.encode(encoder),
            Self::Rejection(rejection) => rejection.encode(encoder),
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
