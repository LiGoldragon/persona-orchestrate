#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActorKind {
    MindRoot,
    Config,
    IngressPhase,
    RequestSession,
    NotaDecoder,
    CallerIdentityResolver,
    EnvelopeBuilder,
    DispatchPhase,
    RequestDispatcher,
    ClaimFlow,
    HandoffFlow,
    ActivityFlow,
    MemoryFlow,
    QueryFlow,
    DomainPhase,
    ClaimSupervisor,
    MemoryGraphSupervisor,
    QuerySupervisor,
    ItemOpen,
    NoteAdd,
    Link,
    StatusChange,
    AliasAdd,
    QueryPlanner,
    GraphTraversal,
    QueryResultShaper,
    StoreSupervisor,
    StoreKernel,
    MemoryStore,
    ClaimStore,
    ActivityStore,
    SemaWriter,
    SemaReader,
    IdMint,
    Clock,
    EventAppender,
    ActivityAppender,
    Commit,
    ViewPhase,
    RoleSnapshotView,
    ReadyWorkView,
    BlockedWorkView,
    RecentActivityView,
    SubscriptionSupervisor,
    CommitBus,
    Subscriber,
    ReplySupervisor,
    NotaReplyEncoder,
    ErrorShaper,
}

impl ActorKind {
    pub fn label(self) -> &'static str {
        match self {
            ActorKind::MindRoot => "MindRoot",
            ActorKind::Config => "Config",
            ActorKind::IngressPhase => "IngressPhase",
            ActorKind::RequestSession => "RequestSession",
            ActorKind::NotaDecoder => "NotaDecoder",
            ActorKind::CallerIdentityResolver => "CallerIdentityResolver",
            ActorKind::EnvelopeBuilder => "EnvelopeBuilder",
            ActorKind::DispatchPhase => "DispatchPhase",
            ActorKind::RequestDispatcher => "RequestDispatcher",
            ActorKind::ClaimFlow => "ClaimFlow",
            ActorKind::HandoffFlow => "HandoffFlow",
            ActorKind::ActivityFlow => "ActivityFlow",
            ActorKind::MemoryFlow => "MemoryFlow",
            ActorKind::QueryFlow => "QueryFlow",
            ActorKind::DomainPhase => "DomainPhase",
            ActorKind::ClaimSupervisor => "ClaimSupervisor",
            ActorKind::MemoryGraphSupervisor => "MemoryGraphSupervisor",
            ActorKind::QuerySupervisor => "QuerySupervisor",
            ActorKind::ItemOpen => "ItemOpen",
            ActorKind::NoteAdd => "NoteAdd",
            ActorKind::Link => "Link",
            ActorKind::StatusChange => "StatusChange",
            ActorKind::AliasAdd => "AliasAdd",
            ActorKind::QueryPlanner => "QueryPlanner",
            ActorKind::GraphTraversal => "GraphTraversal",
            ActorKind::QueryResultShaper => "QueryResultShaper",
            ActorKind::StoreSupervisor => "StoreSupervisor",
            ActorKind::StoreKernel => "StoreKernel",
            ActorKind::MemoryStore => "MemoryStore",
            ActorKind::ClaimStore => "ClaimStore",
            ActorKind::ActivityStore => "ActivityStore",
            ActorKind::SemaWriter => "SemaWriter",
            ActorKind::SemaReader => "SemaReader",
            ActorKind::IdMint => "IdMint",
            ActorKind::Clock => "Clock",
            ActorKind::EventAppender => "EventAppender",
            ActorKind::ActivityAppender => "ActivityAppender",
            ActorKind::Commit => "Commit",
            ActorKind::ViewPhase => "ViewPhase",
            ActorKind::RoleSnapshotView => "RoleSnapshotView",
            ActorKind::ReadyWorkView => "ReadyWorkView",
            ActorKind::BlockedWorkView => "BlockedWorkView",
            ActorKind::RecentActivityView => "RecentActivityView",
            ActorKind::SubscriptionSupervisor => "SubscriptionSupervisor",
            ActorKind::CommitBus => "CommitBus",
            ActorKind::Subscriber => "Subscriber",
            ActorKind::ReplySupervisor => "ReplySupervisor",
            ActorKind::NotaReplyEncoder => "NotaReplyEncoder",
            ActorKind::ErrorShaper => "ErrorShaper",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TraceAction {
    ActorStarted,
    MessageReceived,
    MessageReplied,
    ChildSpawned,
    WriteIntentSent,
    CommitCompleted,
    ViewRefreshed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TraceEvent {
    actor: ActorKind,
    action: TraceAction,
}

impl TraceEvent {
    pub fn new(actor: ActorKind, action: TraceAction) -> Self {
        Self { actor, action }
    }

    pub fn actor(&self) -> ActorKind {
        self.actor
    }

    pub fn action(&self) -> TraceAction {
        self.action
    }
}

#[derive(Debug, Clone, PartialEq, Eq, kameo::Reply)]
pub struct ActorTrace {
    events: Vec<TraceEvent>,
}

impl ActorTrace {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn events(&self) -> &[TraceEvent] {
        &self.events
    }

    pub fn record(&mut self, actor: ActorKind, action: TraceAction) {
        self.events.push(TraceEvent::new(actor, action));
    }

    pub fn contains(&self, actor: ActorKind) -> bool {
        self.events.iter().any(|event| event.actor == actor)
    }

    pub fn contains_action(&self, actor: ActorKind, action: TraceAction) -> bool {
        self.events
            .iter()
            .any(|event| event.actor == actor && event.action == action)
    }

    pub fn contains_ordered(&self, actors: &[ActorKind]) -> bool {
        let mut remaining = actors.iter();
        let Some(mut expected) = remaining.next() else {
            return true;
        };

        for event in &self.events {
            if event.actor == *expected {
                match remaining.next() {
                    Some(next) => expected = next,
                    None => return true,
                }
            }
        }

        false
    }
}

impl Default for ActorTrace {
    fn default() -> Self {
        Self::new()
    }
}
