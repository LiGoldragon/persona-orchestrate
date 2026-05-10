use persona_mind::actors::{ActorKind, ActorManifest, ActorResidency, TraceAction};
use persona_mind::{MindEnvelope, MindRuntime, StoreLocation};
use signal_persona_mind::{
    ActorName, ItemKind, ItemPriority, MindReply, MindRequest, Opening, Query, QueryKind,
    QueryLimit, TextBody, Title,
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
                .expect("kameo runtime starts"),
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
        self.runtime.stop().await.expect("kameo runtime stops");
    }
}

#[test]
fn topology_manifest_names_required_actor_planes() {
    let manifest = ActorManifest::persona_mind_phase_one();

    for actor in [
        ActorKind::MindRoot,
        ActorKind::Config,
        ActorKind::IngressSupervisor,
        ActorKind::DispatchSupervisor,
        ActorKind::DomainSupervisor,
        ActorKind::StoreSupervisor,
        ActorKind::ViewSupervisor,
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
        ActorKind::IngressSupervisor,
        ActorKind::DispatchSupervisor,
        ActorKind::MemoryFlow,
        ActorKind::DomainSupervisor,
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
        ActorKind::IngressSupervisor,
        ActorKind::DispatchSupervisor,
        ActorKind::QueryFlow,
        ActorKind::ViewSupervisor,
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
