use persona_mind::actors::{ActorKind, ActorManifest, ActorResidency, TraceAction};
use persona_mind::{MindEnvelope, MindRuntime, StoreLocation};
use signal_persona_mind::{
    ActorName, Body, Kind, MindReply, MindRequest, Opening, Priority, Query, QueryKind, QueryLimit,
    Title,
};

struct ActorFixture {
    runtime: MindRuntime,
    actor: ActorName,
}

impl ActorFixture {
    async fn new() -> Self {
        Self {
            runtime: MindRuntime::start(StoreLocation::new("mind.redb"))
                .await
                .expect("actor runtime starts"),
            actor: ActorName::new("operator-assistant"),
        }
    }

    fn envelope(&self, request: MindRequest) -> MindEnvelope {
        MindEnvelope::new(self.actor.clone(), request)
    }

    async fn submit(&self, request: MindRequest) -> persona_mind::MindRuntimeReply {
        self.runtime
            .submit(self.envelope(request))
            .await
            .expect("actor request succeeds")
    }

    async fn stop(self) {
        self.runtime.stop().await.expect("actor runtime stops");
    }
}

#[test]
fn topology_manifest_names_required_actor_planes() {
    let manifest = ActorManifest::persona_mind_phase_one();

    for actor in [
        ActorKind::MindRootActor,
        ActorKind::ConfigActor,
        ActorKind::IngressSupervisorActor,
        ActorKind::DispatchSupervisorActor,
        ActorKind::DomainSupervisorActor,
        ActorKind::StoreSupervisorActor,
        ActorKind::ViewSupervisorActor,
        ActorKind::SubscriptionSupervisorActor,
        ActorKind::ReplySupervisorActor,
        ActorKind::SemaWriterActor,
        ActorKind::SemaReadActor,
        ActorKind::IdMintActor,
        ActorKind::ReadyWorkViewActor,
        ActorKind::NotaReplyEncodeActor,
    ] {
        assert!(manifest.contains(actor), "missing {}", actor.label());
    }

    assert_eq!(manifest.actor_count_for(ActorResidency::Root), 1);
    assert!(manifest.actor_count_for(ActorResidency::LongLived) >= 8);
    assert!(manifest.contains_edge(ActorKind::MindRootActor, ActorKind::StoreSupervisorActor));
    assert!(manifest.contains_edge(
        ActorKind::ReplySupervisorActor,
        ActorKind::NotaReplyEncodeActor
    ));
}

#[tokio::test]
async fn open_item_runs_through_actor_backed_write_path() {
    let fixture = ActorFixture::new().await;
    let response = fixture
        .submit(MindRequest::Open(Opening {
            kind: Kind::Task,
            priority: Priority::High,
            title: Title::new("Implement actor-backed mind"),
            body: Body::new("Phase one actor path"),
        }))
        .await;

    let MindReply::Opened(receipt) = response.reply().expect("reply exists") else {
        panic!("expected opened reply");
    };

    assert_eq!(
        receipt.event.header.actor,
        ActorName::new("operator-assistant")
    );
    assert!(response.trace().contains_ordered(&[
        ActorKind::MindRootActor,
        ActorKind::IngressSupervisorActor,
        ActorKind::DispatchSupervisorActor,
        ActorKind::MemoryFlowActor,
        ActorKind::DomainSupervisorActor,
        ActorKind::ItemOpenActor,
        ActorKind::StoreSupervisorActor,
        ActorKind::SemaWriterActor,
        ActorKind::CommitActor,
        ActorKind::ReplySupervisorActor,
        ActorKind::MindRootActor,
    ]));
    assert!(
        response
            .trace()
            .contains_action(ActorKind::SemaWriterActor, TraceAction::WriteIntentSent)
    );
    assert!(
        response
            .trace()
            .contains_action(ActorKind::CommitActor, TraceAction::CommitCompleted)
    );

    fixture.stop().await;
}

#[tokio::test]
async fn query_path_uses_read_actor_without_writer() {
    let fixture = ActorFixture::new().await;
    let _opened = fixture
        .submit(MindRequest::Open(Opening {
            kind: Kind::Task,
            priority: Priority::Normal,
            title: Title::new("Query actor path"),
            body: Body::new("Read path witness"),
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
        ActorKind::MindRootActor,
        ActorKind::IngressSupervisorActor,
        ActorKind::DispatchSupervisorActor,
        ActorKind::QueryFlowActor,
        ActorKind::ViewSupervisorActor,
        ActorKind::ReadyWorkViewActor,
        ActorKind::StoreSupervisorActor,
        ActorKind::SemaReadActor,
        ActorKind::QueryResultShapeActor,
        ActorKind::ReplySupervisorActor,
    ]));
    assert!(response.trace().contains(ActorKind::SemaReadActor));
    assert!(!response.trace().contains(ActorKind::SemaWriterActor));

    fixture.stop().await;
}
