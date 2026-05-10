use super::trace::ActorKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActorResidency {
    Root,
    LongLived,
    TracePhase,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestActor {
    kind: ActorKind,
    residency: ActorResidency,
}

impl ManifestActor {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorManifest {
    actors: Vec<ManifestActor>,
    edges: Vec<ManifestEdge>,
}

impl ActorManifest {
    pub fn persona_mind_phase_one() -> Self {
        let root = ActorResidency::Root;
        let long_lived = ActorResidency::LongLived;
        let trace_phase = ActorResidency::TracePhase;

        let actors = vec![
            ManifestActor::new(ActorKind::MindRootActor, root),
            ManifestActor::new(ActorKind::ConfigActor, long_lived),
            ManifestActor::new(ActorKind::IngressSupervisorActor, long_lived),
            ManifestActor::new(ActorKind::RequestSessionActor, trace_phase),
            ManifestActor::new(ActorKind::NotaDecodeActor, trace_phase),
            ManifestActor::new(ActorKind::CallerIdentityActor, trace_phase),
            ManifestActor::new(ActorKind::EnvelopeActor, trace_phase),
            ManifestActor::new(ActorKind::DispatchSupervisorActor, long_lived),
            ManifestActor::new(ActorKind::RequestDispatchActor, trace_phase),
            ManifestActor::new(ActorKind::ClaimFlowActor, trace_phase),
            ManifestActor::new(ActorKind::HandoffFlowActor, trace_phase),
            ManifestActor::new(ActorKind::ActivityFlowActor, trace_phase),
            ManifestActor::new(ActorKind::MemoryFlowActor, trace_phase),
            ManifestActor::new(ActorKind::QueryFlowActor, trace_phase),
            ManifestActor::new(ActorKind::DomainSupervisorActor, long_lived),
            ManifestActor::new(ActorKind::ClaimSupervisorActor, trace_phase),
            ManifestActor::new(ActorKind::MemoryGraphSupervisorActor, trace_phase),
            ManifestActor::new(ActorKind::QuerySupervisorActor, trace_phase),
            ManifestActor::new(ActorKind::ItemOpenActor, trace_phase),
            ManifestActor::new(ActorKind::NoteAddActor, trace_phase),
            ManifestActor::new(ActorKind::LinkActor, trace_phase),
            ManifestActor::new(ActorKind::StatusChangeActor, trace_phase),
            ManifestActor::new(ActorKind::AliasAddActor, trace_phase),
            ManifestActor::new(ActorKind::QueryPlanActor, trace_phase),
            ManifestActor::new(ActorKind::GraphTraversalActor, trace_phase),
            ManifestActor::new(ActorKind::QueryResultShapeActor, trace_phase),
            ManifestActor::new(ActorKind::StoreSupervisorActor, long_lived),
            ManifestActor::new(ActorKind::SemaWriterActor, trace_phase),
            ManifestActor::new(ActorKind::SemaReadActor, trace_phase),
            ManifestActor::new(ActorKind::IdMintActor, trace_phase),
            ManifestActor::new(ActorKind::ClockActor, trace_phase),
            ManifestActor::new(ActorKind::EventAppendActor, trace_phase),
            ManifestActor::new(ActorKind::CommitActor, trace_phase),
            ManifestActor::new(ActorKind::ViewSupervisorActor, long_lived),
            ManifestActor::new(ActorKind::RoleSnapshotViewActor, trace_phase),
            ManifestActor::new(ActorKind::ReadyWorkViewActor, trace_phase),
            ManifestActor::new(ActorKind::BlockedWorkViewActor, trace_phase),
            ManifestActor::new(ActorKind::RecentActivityViewActor, trace_phase),
            ManifestActor::new(ActorKind::SubscriptionSupervisorActor, long_lived),
            ManifestActor::new(ActorKind::CommitBusActor, trace_phase),
            ManifestActor::new(ActorKind::SubscriberActor, trace_phase),
            ManifestActor::new(ActorKind::ReplySupervisorActor, long_lived),
            ManifestActor::new(ActorKind::NotaReplyEncodeActor, trace_phase),
            ManifestActor::new(ActorKind::ErrorShapeActor, trace_phase),
        ];

        let edges = vec![
            ManifestEdge::new(ActorKind::MindRootActor, ActorKind::ConfigActor),
            ManifestEdge::new(ActorKind::MindRootActor, ActorKind::IngressSupervisorActor),
            ManifestEdge::new(ActorKind::MindRootActor, ActorKind::DispatchSupervisorActor),
            ManifestEdge::new(ActorKind::MindRootActor, ActorKind::DomainSupervisorActor),
            ManifestEdge::new(ActorKind::MindRootActor, ActorKind::StoreSupervisorActor),
            ManifestEdge::new(ActorKind::MindRootActor, ActorKind::ViewSupervisorActor),
            ManifestEdge::new(
                ActorKind::MindRootActor,
                ActorKind::SubscriptionSupervisorActor,
            ),
            ManifestEdge::new(ActorKind::MindRootActor, ActorKind::ReplySupervisorActor),
            ManifestEdge::new(
                ActorKind::IngressSupervisorActor,
                ActorKind::RequestSessionActor,
            ),
            ManifestEdge::new(
                ActorKind::IngressSupervisorActor,
                ActorKind::NotaDecodeActor,
            ),
            ManifestEdge::new(
                ActorKind::IngressSupervisorActor,
                ActorKind::CallerIdentityActor,
            ),
            ManifestEdge::new(ActorKind::IngressSupervisorActor, ActorKind::EnvelopeActor),
            ManifestEdge::new(
                ActorKind::DispatchSupervisorActor,
                ActorKind::RequestDispatchActor,
            ),
            ManifestEdge::new(
                ActorKind::DispatchSupervisorActor,
                ActorKind::ClaimFlowActor,
            ),
            ManifestEdge::new(
                ActorKind::DispatchSupervisorActor,
                ActorKind::HandoffFlowActor,
            ),
            ManifestEdge::new(
                ActorKind::DispatchSupervisorActor,
                ActorKind::ActivityFlowActor,
            ),
            ManifestEdge::new(
                ActorKind::DispatchSupervisorActor,
                ActorKind::MemoryFlowActor,
            ),
            ManifestEdge::new(
                ActorKind::DispatchSupervisorActor,
                ActorKind::QueryFlowActor,
            ),
            ManifestEdge::new(
                ActorKind::DomainSupervisorActor,
                ActorKind::ClaimSupervisorActor,
            ),
            ManifestEdge::new(
                ActorKind::DomainSupervisorActor,
                ActorKind::MemoryGraphSupervisorActor,
            ),
            ManifestEdge::new(
                ActorKind::DomainSupervisorActor,
                ActorKind::QuerySupervisorActor,
            ),
            ManifestEdge::new(
                ActorKind::MemoryGraphSupervisorActor,
                ActorKind::ItemOpenActor,
            ),
            ManifestEdge::new(
                ActorKind::MemoryGraphSupervisorActor,
                ActorKind::NoteAddActor,
            ),
            ManifestEdge::new(ActorKind::MemoryGraphSupervisorActor, ActorKind::LinkActor),
            ManifestEdge::new(
                ActorKind::MemoryGraphSupervisorActor,
                ActorKind::StatusChangeActor,
            ),
            ManifestEdge::new(
                ActorKind::MemoryGraphSupervisorActor,
                ActorKind::AliasAddActor,
            ),
            ManifestEdge::new(ActorKind::QuerySupervisorActor, ActorKind::QueryPlanActor),
            ManifestEdge::new(
                ActorKind::QuerySupervisorActor,
                ActorKind::GraphTraversalActor,
            ),
            ManifestEdge::new(
                ActorKind::QuerySupervisorActor,
                ActorKind::QueryResultShapeActor,
            ),
            ManifestEdge::new(ActorKind::StoreSupervisorActor, ActorKind::SemaWriterActor),
            ManifestEdge::new(ActorKind::StoreSupervisorActor, ActorKind::SemaReadActor),
            ManifestEdge::new(ActorKind::StoreSupervisorActor, ActorKind::IdMintActor),
            ManifestEdge::new(ActorKind::StoreSupervisorActor, ActorKind::ClockActor),
            ManifestEdge::new(ActorKind::StoreSupervisorActor, ActorKind::EventAppendActor),
            ManifestEdge::new(ActorKind::StoreSupervisorActor, ActorKind::CommitActor),
            ManifestEdge::new(
                ActorKind::ViewSupervisorActor,
                ActorKind::RoleSnapshotViewActor,
            ),
            ManifestEdge::new(
                ActorKind::ViewSupervisorActor,
                ActorKind::ReadyWorkViewActor,
            ),
            ManifestEdge::new(
                ActorKind::ViewSupervisorActor,
                ActorKind::BlockedWorkViewActor,
            ),
            ManifestEdge::new(
                ActorKind::ViewSupervisorActor,
                ActorKind::RecentActivityViewActor,
            ),
            ManifestEdge::new(
                ActorKind::SubscriptionSupervisorActor,
                ActorKind::CommitBusActor,
            ),
            ManifestEdge::new(
                ActorKind::SubscriptionSupervisorActor,
                ActorKind::SubscriberActor,
            ),
            ManifestEdge::new(
                ActorKind::ReplySupervisorActor,
                ActorKind::NotaReplyEncodeActor,
            ),
            ManifestEdge::new(ActorKind::ReplySupervisorActor, ActorKind::ErrorShapeActor),
        ];

        Self { actors, edges }
    }

    pub fn actors(&self) -> &[ManifestActor] {
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
