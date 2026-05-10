#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActorKind {
    MindRootActor,
    ConfigActor,
    IngressSupervisorActor,
    RequestSessionActor,
    NotaDecodeActor,
    CallerIdentityActor,
    EnvelopeActor,
    DispatchSupervisorActor,
    RequestDispatchActor,
    ClaimFlowActor,
    HandoffFlowActor,
    ActivityFlowActor,
    MemoryFlowActor,
    QueryFlowActor,
    DomainSupervisorActor,
    ClaimSupervisorActor,
    MemoryGraphSupervisorActor,
    QuerySupervisorActor,
    ItemOpenActor,
    NoteAddActor,
    LinkActor,
    StatusChangeActor,
    AliasAddActor,
    QueryPlanActor,
    GraphTraversalActor,
    QueryResultShapeActor,
    StoreSupervisorActor,
    SemaWriterActor,
    SemaReadActor,
    IdMintActor,
    ClockActor,
    EventAppendActor,
    CommitActor,
    ViewSupervisorActor,
    RoleSnapshotViewActor,
    ReadyWorkViewActor,
    BlockedWorkViewActor,
    RecentActivityViewActor,
    SubscriptionSupervisorActor,
    CommitBusActor,
    SubscriberActor,
    ReplySupervisorActor,
    NotaReplyEncodeActor,
    ErrorShapeActor,
}

impl ActorKind {
    pub fn label(self) -> &'static str {
        match self {
            ActorKind::MindRootActor => "MindRootActor",
            ActorKind::ConfigActor => "ConfigActor",
            ActorKind::IngressSupervisorActor => "IngressSupervisorActor",
            ActorKind::RequestSessionActor => "RequestSessionActor",
            ActorKind::NotaDecodeActor => "NotaDecodeActor",
            ActorKind::CallerIdentityActor => "CallerIdentityActor",
            ActorKind::EnvelopeActor => "EnvelopeActor",
            ActorKind::DispatchSupervisorActor => "DispatchSupervisorActor",
            ActorKind::RequestDispatchActor => "RequestDispatchActor",
            ActorKind::ClaimFlowActor => "ClaimFlowActor",
            ActorKind::HandoffFlowActor => "HandoffFlowActor",
            ActorKind::ActivityFlowActor => "ActivityFlowActor",
            ActorKind::MemoryFlowActor => "MemoryFlowActor",
            ActorKind::QueryFlowActor => "QueryFlowActor",
            ActorKind::DomainSupervisorActor => "DomainSupervisorActor",
            ActorKind::ClaimSupervisorActor => "ClaimSupervisorActor",
            ActorKind::MemoryGraphSupervisorActor => "MemoryGraphSupervisorActor",
            ActorKind::QuerySupervisorActor => "QuerySupervisorActor",
            ActorKind::ItemOpenActor => "ItemOpenActor",
            ActorKind::NoteAddActor => "NoteAddActor",
            ActorKind::LinkActor => "LinkActor",
            ActorKind::StatusChangeActor => "StatusChangeActor",
            ActorKind::AliasAddActor => "AliasAddActor",
            ActorKind::QueryPlanActor => "QueryPlanActor",
            ActorKind::GraphTraversalActor => "GraphTraversalActor",
            ActorKind::QueryResultShapeActor => "QueryResultShapeActor",
            ActorKind::StoreSupervisorActor => "StoreSupervisorActor",
            ActorKind::SemaWriterActor => "SemaWriterActor",
            ActorKind::SemaReadActor => "SemaReadActor",
            ActorKind::IdMintActor => "IdMintActor",
            ActorKind::ClockActor => "ClockActor",
            ActorKind::EventAppendActor => "EventAppendActor",
            ActorKind::CommitActor => "CommitActor",
            ActorKind::ViewSupervisorActor => "ViewSupervisorActor",
            ActorKind::RoleSnapshotViewActor => "RoleSnapshotViewActor",
            ActorKind::ReadyWorkViewActor => "ReadyWorkViewActor",
            ActorKind::BlockedWorkViewActor => "BlockedWorkViewActor",
            ActorKind::RecentActivityViewActor => "RecentActivityViewActor",
            ActorKind::SubscriptionSupervisorActor => "SubscriptionSupervisorActor",
            ActorKind::CommitBusActor => "CommitBusActor",
            ActorKind::SubscriberActor => "SubscriberActor",
            ActorKind::ReplySupervisorActor => "ReplySupervisorActor",
            ActorKind::NotaReplyEncodeActor => "NotaReplyEncodeActor",
            ActorKind::ErrorShapeActor => "ErrorShapeActor",
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
