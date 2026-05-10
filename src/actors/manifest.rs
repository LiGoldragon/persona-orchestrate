use super::trace::ActorKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActorResidency {
    Root,
    LongLived,
    TracePhase,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestEntry {
    kind: ActorKind,
    residency: ActorResidency,
}

impl ManifestEntry {
    pub fn new(kind: ActorKind, residency: ActorResidency) -> Self {
        Self { kind, residency }
    }

    pub fn kind(&self) -> ActorKind {
        self.kind
    }

    pub fn residency(&self) -> ActorResidency {
        self.residency
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManifestEdge {
    parent: ActorKind,
    child: ActorKind,
}

impl ManifestEdge {
    pub fn new(parent: ActorKind, child: ActorKind) -> Self {
        Self { parent, child }
    }

    pub fn parent(&self) -> ActorKind {
        self.parent
    }

    pub fn child(&self) -> ActorKind {
        self.child
    }
}

#[derive(Debug, Clone, PartialEq, Eq, kameo::Reply)]
pub struct ActorManifest {
    actors: Vec<ManifestEntry>,
    edges: Vec<ManifestEdge>,
}

impl ActorManifest {
    pub fn persona_mind_phase_one() -> Self {
        let root = ActorResidency::Root;
        let long_lived = ActorResidency::LongLived;
        let trace_phase = ActorResidency::TracePhase;

        let actors = vec![
            ManifestEntry::new(ActorKind::MindRoot, root),
            ManifestEntry::new(ActorKind::Config, long_lived),
            ManifestEntry::new(ActorKind::IngressPhase, long_lived),
            ManifestEntry::new(ActorKind::RequestSession, trace_phase),
            ManifestEntry::new(ActorKind::NotaDecoder, trace_phase),
            ManifestEntry::new(ActorKind::CallerIdentityResolver, trace_phase),
            ManifestEntry::new(ActorKind::EnvelopeBuilder, trace_phase),
            ManifestEntry::new(ActorKind::DispatchPhase, long_lived),
            ManifestEntry::new(ActorKind::RequestDispatcher, trace_phase),
            ManifestEntry::new(ActorKind::ClaimFlow, trace_phase),
            ManifestEntry::new(ActorKind::HandoffFlow, trace_phase),
            ManifestEntry::new(ActorKind::ActivityFlow, trace_phase),
            ManifestEntry::new(ActorKind::MemoryFlow, trace_phase),
            ManifestEntry::new(ActorKind::QueryFlow, trace_phase),
            ManifestEntry::new(ActorKind::DomainPhase, long_lived),
            ManifestEntry::new(ActorKind::ClaimSupervisor, trace_phase),
            ManifestEntry::new(ActorKind::MemoryGraphSupervisor, trace_phase),
            ManifestEntry::new(ActorKind::QuerySupervisor, trace_phase),
            ManifestEntry::new(ActorKind::ItemOpen, trace_phase),
            ManifestEntry::new(ActorKind::NoteAdd, trace_phase),
            ManifestEntry::new(ActorKind::Link, trace_phase),
            ManifestEntry::new(ActorKind::StatusChange, trace_phase),
            ManifestEntry::new(ActorKind::AliasAdd, trace_phase),
            ManifestEntry::new(ActorKind::QueryPlanner, trace_phase),
            ManifestEntry::new(ActorKind::GraphTraversal, trace_phase),
            ManifestEntry::new(ActorKind::QueryResultShaper, trace_phase),
            ManifestEntry::new(ActorKind::StoreSupervisor, long_lived),
            ManifestEntry::new(ActorKind::SemaWriter, trace_phase),
            ManifestEntry::new(ActorKind::SemaReader, trace_phase),
            ManifestEntry::new(ActorKind::IdMint, trace_phase),
            ManifestEntry::new(ActorKind::Clock, trace_phase),
            ManifestEntry::new(ActorKind::EventAppender, trace_phase),
            ManifestEntry::new(ActorKind::Commit, trace_phase),
            ManifestEntry::new(ActorKind::ViewPhase, long_lived),
            ManifestEntry::new(ActorKind::RoleSnapshotView, trace_phase),
            ManifestEntry::new(ActorKind::ReadyWorkView, trace_phase),
            ManifestEntry::new(ActorKind::BlockedWorkView, trace_phase),
            ManifestEntry::new(ActorKind::RecentActivityView, trace_phase),
            ManifestEntry::new(ActorKind::SubscriptionSupervisor, long_lived),
            ManifestEntry::new(ActorKind::CommitBus, trace_phase),
            ManifestEntry::new(ActorKind::Subscriber, trace_phase),
            ManifestEntry::new(ActorKind::ReplySupervisor, long_lived),
            ManifestEntry::new(ActorKind::NotaReplyEncoder, trace_phase),
            ManifestEntry::new(ActorKind::ErrorShaper, trace_phase),
        ];

        let edges = vec![
            ManifestEdge::new(ActorKind::MindRoot, ActorKind::Config),
            ManifestEdge::new(ActorKind::MindRoot, ActorKind::IngressPhase),
            ManifestEdge::new(ActorKind::MindRoot, ActorKind::DispatchPhase),
            ManifestEdge::new(ActorKind::MindRoot, ActorKind::DomainPhase),
            ManifestEdge::new(ActorKind::MindRoot, ActorKind::StoreSupervisor),
            ManifestEdge::new(ActorKind::MindRoot, ActorKind::ViewPhase),
            ManifestEdge::new(ActorKind::MindRoot, ActorKind::SubscriptionSupervisor),
            ManifestEdge::new(ActorKind::MindRoot, ActorKind::ReplySupervisor),
            ManifestEdge::new(ActorKind::IngressPhase, ActorKind::RequestSession),
            ManifestEdge::new(ActorKind::IngressPhase, ActorKind::NotaDecoder),
            ManifestEdge::new(ActorKind::IngressPhase, ActorKind::CallerIdentityResolver),
            ManifestEdge::new(ActorKind::IngressPhase, ActorKind::EnvelopeBuilder),
            ManifestEdge::new(ActorKind::DispatchPhase, ActorKind::RequestDispatcher),
            ManifestEdge::new(ActorKind::DispatchPhase, ActorKind::ClaimFlow),
            ManifestEdge::new(ActorKind::DispatchPhase, ActorKind::HandoffFlow),
            ManifestEdge::new(ActorKind::DispatchPhase, ActorKind::ActivityFlow),
            ManifestEdge::new(ActorKind::DispatchPhase, ActorKind::MemoryFlow),
            ManifestEdge::new(ActorKind::DispatchPhase, ActorKind::QueryFlow),
            ManifestEdge::new(ActorKind::DomainPhase, ActorKind::ClaimSupervisor),
            ManifestEdge::new(ActorKind::DomainPhase, ActorKind::MemoryGraphSupervisor),
            ManifestEdge::new(ActorKind::DomainPhase, ActorKind::QuerySupervisor),
            ManifestEdge::new(ActorKind::MemoryGraphSupervisor, ActorKind::ItemOpen),
            ManifestEdge::new(ActorKind::MemoryGraphSupervisor, ActorKind::NoteAdd),
            ManifestEdge::new(ActorKind::MemoryGraphSupervisor, ActorKind::Link),
            ManifestEdge::new(ActorKind::MemoryGraphSupervisor, ActorKind::StatusChange),
            ManifestEdge::new(ActorKind::MemoryGraphSupervisor, ActorKind::AliasAdd),
            ManifestEdge::new(ActorKind::QuerySupervisor, ActorKind::QueryPlanner),
            ManifestEdge::new(ActorKind::QuerySupervisor, ActorKind::GraphTraversal),
            ManifestEdge::new(ActorKind::QuerySupervisor, ActorKind::QueryResultShaper),
            ManifestEdge::new(ActorKind::StoreSupervisor, ActorKind::SemaWriter),
            ManifestEdge::new(ActorKind::StoreSupervisor, ActorKind::SemaReader),
            ManifestEdge::new(ActorKind::StoreSupervisor, ActorKind::IdMint),
            ManifestEdge::new(ActorKind::StoreSupervisor, ActorKind::Clock),
            ManifestEdge::new(ActorKind::StoreSupervisor, ActorKind::EventAppender),
            ManifestEdge::new(ActorKind::StoreSupervisor, ActorKind::Commit),
            ManifestEdge::new(ActorKind::ViewPhase, ActorKind::RoleSnapshotView),
            ManifestEdge::new(ActorKind::ViewPhase, ActorKind::ReadyWorkView),
            ManifestEdge::new(ActorKind::ViewPhase, ActorKind::BlockedWorkView),
            ManifestEdge::new(ActorKind::ViewPhase, ActorKind::RecentActivityView),
            ManifestEdge::new(ActorKind::SubscriptionSupervisor, ActorKind::CommitBus),
            ManifestEdge::new(ActorKind::SubscriptionSupervisor, ActorKind::Subscriber),
            ManifestEdge::new(ActorKind::ReplySupervisor, ActorKind::NotaReplyEncoder),
            ManifestEdge::new(ActorKind::ReplySupervisor, ActorKind::ErrorShaper),
        ];

        Self { actors, edges }
    }

    pub fn actors(&self) -> &[ManifestEntry] {
        &self.actors
    }

    pub fn edges(&self) -> &[ManifestEdge] {
        &self.edges
    }

    pub fn contains(&self, actor: ActorKind) -> bool {
        self.actors.iter().any(|entry| entry.kind == actor)
    }

    pub fn contains_edge(&self, parent: ActorKind, child: ActorKind) -> bool {
        self.edges
            .iter()
            .any(|edge| edge.parent == parent && edge.child == child)
    }

    pub fn actor_count_for(&self, residency: ActorResidency) -> usize {
        self.actors
            .iter()
            .filter(|actor| actor.residency == residency)
            .count()
    }
}
