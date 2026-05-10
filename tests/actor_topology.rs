use persona_mind::actors::{ActorKind, ActorManifest, ActorResidency, TraceAction};
use persona_mind::{
    ActorRef, MindEnvelope, MindRoot, MindRootArguments, MindRootReply, StoreLocation,
    SubmitEnvelope,
};
use signal_persona_mind::{
    ActorName, ItemKind, ItemPriority, MindReply, MindRequest, Opening, Query, QueryKind,
    QueryLimit, RoleClaim, RoleName, RoleObservation, RoleRelease, ScopeReason, ScopeReference,
    TextBody, Title, WirePath,
};

struct ActorFixture {
    root: ActorRef<MindRoot>,
    actor: ActorName,
}

impl ActorFixture {
    async fn new() -> Self {
        Self {
            root: MindRoot::start(MindRootArguments::new(StoreLocation::new("mind.redb")))
                .await
                .expect("mind root starts"),
            actor: ActorName::new("operator-assistant"),
        }
    }

    fn envelope(&self, request: MindRequest) -> MindEnvelope {
        MindEnvelope::new(self.actor.clone(), request)
    }

    async fn submit(&self, request: MindRequest) -> MindRootReply {
        self.root
            .ask(SubmitEnvelope {
                envelope: self.envelope(request),
            })
            .await
            .expect("actor request succeeds")
    }

    async fn stop(self) {
        MindRoot::stop(self.root).await.expect("mind root stops");
    }
}

struct ClaimFixture {
    role: RoleName,
    reason: ScopeReason,
}

impl ClaimFixture {
    fn operator() -> Self {
        Self {
            role: RoleName::Operator,
            reason: ScopeReason::from_text("testing persona mind claim flow").expect("reason"),
        }
    }

    fn path(&self, path: &str) -> ScopeReference {
        ScopeReference::Path(WirePath::from_absolute_path(path).expect("absolute path"))
    }

    fn claim(&self, path: &str) -> MindRequest {
        MindRequest::RoleClaim(RoleClaim {
            role: self.role,
            scopes: vec![self.path(path)],
            reason: self.reason.clone(),
        })
    }
}

#[test]
fn topology_manifest_names_required_actor_planes() {
    let manifest = ActorManifest::persona_mind_phase_one();

    for actor in [
        ActorKind::MindRoot,
        ActorKind::Config,
        ActorKind::IngressPhase,
        ActorKind::DispatchPhase,
        ActorKind::DomainPhase,
        ActorKind::StoreSupervisor,
        ActorKind::ViewPhase,
        ActorKind::SubscriptionSupervisor,
        ActorKind::ReplySupervisor,
        ActorKind::SemaWriter,
        ActorKind::SemaReader,
        ActorKind::IdMint,
        ActorKind::ReadyWorkView,
        ActorKind::NotaReplyEncoder,
    ] {
        assert!(manifest.contains(actor), "missing {}", actor.label());
    }

    assert_eq!(manifest.actor_count_for(ActorResidency::Root), 1);
    assert!(manifest.actor_count_for(ActorResidency::LongLived) >= 8);
    assert!(manifest.contains_edge(ActorKind::MindRoot, ActorKind::StoreSupervisor));
    assert!(manifest.contains_edge(ActorKind::ReplySupervisor, ActorKind::NotaReplyEncoder));
}

#[tokio::test]
async fn open_item_runs_through_kameo_write_path() {
    let fixture = ActorFixture::new().await;
    let response = fixture
        .submit(MindRequest::Opening(Opening {
            kind: ItemKind::Task,
            priority: ItemPriority::High,
            title: Title::new("Implement Kameo-backed mind"),
            body: TextBody::new("Phase one actor path"),
        }))
        .await;

    let MindReply::OpeningReceipt(receipt) = response.reply().expect("reply exists") else {
        panic!("expected opened reply");
    };

    assert_eq!(
        receipt.event.header.actor,
        ActorName::new("operator-assistant")
    );
    assert!(response.trace().contains_ordered(&[
        ActorKind::MindRoot,
        ActorKind::IngressPhase,
        ActorKind::DispatchPhase,
        ActorKind::MemoryFlow,
        ActorKind::DomainPhase,
        ActorKind::ItemOpen,
        ActorKind::StoreSupervisor,
        ActorKind::SemaWriter,
        ActorKind::Commit,
        ActorKind::ReplySupervisor,
        ActorKind::MindRoot,
    ]));
    assert!(
        response
            .trace()
            .contains_action(ActorKind::SemaWriter, TraceAction::WriteIntentSent)
    );
    assert!(
        response
            .trace()
            .contains_action(ActorKind::Commit, TraceAction::CommitCompleted)
    );

    fixture.stop().await;
}

#[tokio::test]
async fn query_path_uses_read_actor_without_writer() {
    let fixture = ActorFixture::new().await;
    let _opened = fixture
        .submit(MindRequest::Opening(Opening {
            kind: ItemKind::Task,
            priority: ItemPriority::Normal,
            title: Title::new("Query actor path"),
            body: TextBody::new("Read path witness"),
        }))
        .await;

    let response = fixture
        .submit(MindRequest::Query(Query {
            kind: QueryKind::Ready,
            limit: QueryLimit::new(10),
        }))
        .await;

    let MindReply::View(view) = response.reply().expect("reply exists") else {
        panic!("expected view reply");
    };

    assert_eq!(view.items.len(), 1);
    assert!(response.trace().contains_ordered(&[
        ActorKind::MindRoot,
        ActorKind::IngressPhase,
        ActorKind::DispatchPhase,
        ActorKind::QueryFlow,
        ActorKind::ViewPhase,
        ActorKind::ReadyWorkView,
        ActorKind::StoreSupervisor,
        ActorKind::SemaReader,
        ActorKind::QueryResultShaper,
        ActorKind::ReplySupervisor,
    ]));
    assert!(response.trace().contains(ActorKind::SemaReader));
    assert!(!response.trace().contains(ActorKind::SemaWriter));

    fixture.stop().await;
}

#[tokio::test]
async fn role_claim_reaches_claim_flow_and_commits() {
    let fixture = ActorFixture::new().await;
    let claim = ClaimFixture::operator();
    let response = fixture
        .submit(claim.claim("/git/github.com/LiGoldragon/persona-mind"))
        .await;

    let MindReply::ClaimAcceptance(acceptance) = response.reply().expect("reply exists") else {
        panic!("expected claim acceptance");
    };

    assert_eq!(acceptance.role, RoleName::Operator);
    assert_eq!(acceptance.scopes.len(), 1);
    assert!(response.trace().contains_ordered(&[
        ActorKind::MindRoot,
        ActorKind::IngressPhase,
        ActorKind::DispatchPhase,
        ActorKind::ClaimFlow,
        ActorKind::DomainPhase,
        ActorKind::ClaimSupervisor,
        ActorKind::StoreSupervisor,
        ActorKind::SemaWriter,
        ActorKind::Commit,
        ActorKind::ReplySupervisor,
    ]));

    fixture.stop().await;
}

#[tokio::test]
async fn conflicting_claim_returns_typed_rejection() {
    let fixture = ActorFixture::new().await;
    let claim = ClaimFixture::operator();
    let _accepted = fixture
        .submit(claim.claim("/git/github.com/LiGoldragon/persona"))
        .await;

    let response = fixture
        .submit(MindRequest::RoleClaim(RoleClaim {
            role: RoleName::Designer,
            scopes: vec![claim.path("/git/github.com/LiGoldragon/persona/src")],
            reason: ScopeReason::from_text("designer conflict probe").expect("reason"),
        }))
        .await;

    let MindReply::ClaimRejection(rejection) = response.reply().expect("reply exists") else {
        panic!("expected claim rejection");
    };

    assert_eq!(rejection.role, RoleName::Designer);
    assert_eq!(rejection.conflicts.len(), 1);
    assert_eq!(rejection.conflicts[0].held_by, RoleName::Operator);
    assert!(response.trace().contains(ActorKind::ClaimFlow));
    assert!(response.trace().contains(ActorKind::Commit));

    fixture.stop().await;
}

#[tokio::test]
async fn role_observation_reads_claims_without_writer() {
    let fixture = ActorFixture::new().await;
    let claim = ClaimFixture::operator();
    let _accepted = fixture
        .submit(claim.claim("/git/github.com/LiGoldragon/persona-mind"))
        .await;

    let response = fixture
        .submit(MindRequest::RoleObservation(RoleObservation))
        .await;

    let MindReply::RoleSnapshot(snapshot) = response.reply().expect("reply exists") else {
        panic!("expected role snapshot");
    };
    let operator = snapshot
        .roles
        .iter()
        .find(|status| status.role == RoleName::Operator)
        .expect("operator status exists");

    assert_eq!(operator.claims.len(), 1);
    assert!(response.trace().contains(ActorKind::RoleSnapshotView));
    assert!(response.trace().contains(ActorKind::SemaReader));
    assert!(!response.trace().contains(ActorKind::SemaWriter));

    fixture.stop().await;
}

#[tokio::test]
async fn role_release_removes_claims_from_observation() {
    let fixture = ActorFixture::new().await;
    let claim = ClaimFixture::operator();
    let _accepted = fixture
        .submit(claim.claim("/git/github.com/LiGoldragon/persona-mind"))
        .await;

    let released = fixture
        .submit(MindRequest::RoleRelease(RoleRelease {
            role: RoleName::Operator,
        }))
        .await;
    let MindReply::ReleaseAcknowledgment(acknowledgment) = released.reply().expect("reply exists")
    else {
        panic!("expected release acknowledgment");
    };
    assert_eq!(acknowledgment.released_scopes.len(), 1);

    let observed = fixture
        .submit(MindRequest::RoleObservation(RoleObservation))
        .await;
    let MindReply::RoleSnapshot(snapshot) = observed.reply().expect("reply exists") else {
        panic!("expected role snapshot");
    };
    let operator = snapshot
        .roles
        .iter()
        .find(|status| status.role == RoleName::Operator)
        .expect("operator status exists");

    assert!(operator.claims.is_empty());

    fixture.stop().await;
}
