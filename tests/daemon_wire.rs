use std::time::{SystemTime, UNIX_EPOCH};

use persona_mind::{MindClient, MindDaemon, MindDaemonEndpoint, MindFrameCodec, StoreLocation};
use signal_persona_mind::{
    ActorName, Frame, FrameBody, ItemKind, ItemPriority, MindReply, MindRequest, Opening, Query,
    QueryKind, QueryLimit, RoleClaim, RoleName, RoleObservation, ScopeReason, ScopeReference,
    TextBody, Title, WirePath,
};
use tokio::net::UnixStream;

struct SocketFixture {
    endpoint: MindDaemonEndpoint,
    store: StoreLocation,
}

impl SocketFixture {
    fn new(test_name: &str) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "persona-mind-{test_name}-{}-{stamp}",
            std::process::id()
        ));
        let socket = root.with_extension("sock");
        let store = root.with_extension("redb");
        Self {
            endpoint: MindDaemonEndpoint::new(socket),
            store: StoreLocation::new(store.to_string_lossy().to_string()),
        }
    }

    fn endpoint(&self) -> MindDaemonEndpoint {
        self.endpoint.clone()
    }

    fn store(&self) -> StoreLocation {
        self.store.clone()
    }

    fn request(&self) -> MindRequest {
        MindRequest::Opening(Opening {
            kind: ItemKind::Task,
            priority: ItemPriority::High,
            title: Title::new("Route one request through daemon wire"),
            body: TextBody::new("The daemon receives a Signal frame and replies with one."),
        })
    }
}

#[tokio::test]
async fn daemon_round_trip_uses_signal_frames_over_socket() {
    let fixture = SocketFixture::new("round-trip");
    let daemon = MindDaemon::new(fixture.endpoint(), fixture.store())
        .bind()
        .await
        .expect("daemon binds");
    let endpoint = daemon.endpoint().clone();
    let server = tokio::spawn(async move { daemon.serve_one().await });

    let client = MindClient::new(endpoint, ActorName::new("operator"));
    let client_reply = client
        .submit(fixture.request())
        .await
        .expect("client receives reply frame");
    let daemon_reply = server
        .await
        .expect("daemon task joins")
        .expect("daemon serves one request");

    assert_eq!(client_reply, daemon_reply);
    let MindReply::OpeningReceipt(receipt) = client_reply else {
        panic!("expected opening receipt");
    };
    assert_eq!(receipt.event.header.actor, ActorName::new("operator"));
}

#[tokio::test]
async fn daemon_uses_signal_auth_for_actor_identity() {
    let fixture = SocketFixture::new("auth-identity");
    let daemon = MindDaemon::new(fixture.endpoint(), fixture.store())
        .bind()
        .await
        .expect("daemon binds");
    let endpoint = daemon.endpoint().clone();
    let server = tokio::spawn(async move { daemon.serve_one().await });

    let client = MindClient::new(endpoint, ActorName::new("designer"));
    let client_reply = client
        .submit(fixture.request())
        .await
        .expect("client receives reply frame");
    server
        .await
        .expect("daemon task joins")
        .expect("daemon serves one request");

    let MindReply::OpeningReceipt(receipt) = client_reply else {
        panic!("expected opening receipt");
    };
    assert_eq!(receipt.event.header.actor, ActorName::new("designer"));
}

#[tokio::test]
async fn daemon_rejects_request_frames_without_auth() {
    let fixture = SocketFixture::new("missing-auth");
    let daemon = MindDaemon::new(fixture.endpoint(), fixture.store())
        .bind()
        .await
        .expect("daemon binds");
    let endpoint = daemon.endpoint().clone();
    let server = tokio::spawn(async move { daemon.serve_one().await });

    let codec = MindFrameCodec::default();
    let mut stream = UnixStream::connect(endpoint.as_path())
        .await
        .expect("client connects to daemon");
    let frame = Frame::new(FrameBody::Request(signal_core::Request::assert(
        fixture.request(),
    )));
    codec
        .write_frame(&mut stream, &frame)
        .await
        .expect("client writes unauthenticated frame");

    let error = server
        .await
        .expect("daemon task joins")
        .expect_err("daemon rejects missing signal auth");

    assert!(matches!(error, persona_mind::Error::MissingAuthProof));
}

#[tokio::test]
async fn client_cannot_reply_without_daemon_signal_frame() {
    let fixture = SocketFixture::new("no-daemon");
    let client = MindClient::new(fixture.endpoint(), ActorName::new("operator"));
    let error = client
        .submit(fixture.request())
        .await
        .expect_err("missing daemon cannot produce reply");

    assert!(matches!(error, persona_mind::Error::Io(_)));
}

#[tokio::test]
async fn mind_store_survives_process_restart() {
    let fixture = SocketFixture::new("store-restart");

    {
        let daemon = MindDaemon::new(fixture.endpoint(), fixture.store())
            .bind()
            .await
            .expect("first daemon binds");
        let endpoint = daemon.endpoint().clone();
        let server = tokio::spawn(async move { daemon.serve_one().await });

        let client = MindClient::new(endpoint, ActorName::new("operator"));
        client
            .submit(MindRequest::RoleClaim(RoleClaim {
                role: RoleName::Operator,
                scopes: vec![ScopeReference::Path(
                    WirePath::from_absolute_path("/git/github.com/LiGoldragon/persona-mind")
                        .expect("absolute path"),
                )],
                reason: ScopeReason::from_text("durable role claim").expect("scope reason"),
            }))
            .await
            .expect("claim committed");

        server
            .await
            .expect("first daemon joins")
            .expect("first daemon serves claim");
    }

    let daemon = MindDaemon::new(fixture.endpoint(), fixture.store())
        .bind()
        .await
        .expect("second daemon binds");
    let endpoint = daemon.endpoint().clone();
    let server = tokio::spawn(async move { daemon.serve_one().await });

    let client = MindClient::new(endpoint, ActorName::new("operator"));
    let reply = client
        .submit(MindRequest::RoleObservation(RoleObservation))
        .await
        .expect("observation reads durable store");

    server
        .await
        .expect("second daemon joins")
        .expect("second daemon serves observation");

    let MindReply::RoleSnapshot(snapshot) = reply else {
        panic!("expected role snapshot");
    };
    let operator = snapshot
        .roles
        .iter()
        .find(|status| status.role == RoleName::Operator)
        .expect("operator status exists");

    assert_eq!(operator.claims.len(), 1);
    assert_eq!(
        operator.claims[0].scope,
        ScopeReference::Path(
            WirePath::from_absolute_path("/git/github.com/LiGoldragon/persona-mind")
                .expect("absolute path")
        )
    );
    assert_eq!(
        operator.claims[0].reason,
        ScopeReason::from_text("durable role claim").expect("scope reason")
    );
}

#[tokio::test]
async fn mind_memory_graph_survives_process_restart() {
    let fixture = SocketFixture::new("memory-restart");

    {
        let daemon = MindDaemon::new(fixture.endpoint(), fixture.store())
            .bind()
            .await
            .expect("first daemon binds");
        let endpoint = daemon.endpoint().clone();
        let server = tokio::spawn(async move { daemon.serve_one().await });

        let client = MindClient::new(endpoint, ActorName::new("operator"));
        client
            .submit(MindRequest::Opening(Opening {
                kind: ItemKind::Task,
                priority: ItemPriority::High,
                title: Title::new("Durable mind memory"),
                body: TextBody::new("The work graph survives daemon restart."),
            }))
            .await
            .expect("opening committed");

        server
            .await
            .expect("first daemon joins")
            .expect("first daemon serves opening");
    }

    let daemon = MindDaemon::new(fixture.endpoint(), fixture.store())
        .bind()
        .await
        .expect("second daemon binds");
    let endpoint = daemon.endpoint().clone();
    let server = tokio::spawn(async move { daemon.serve_one().await });

    let client = MindClient::new(endpoint, ActorName::new("operator"));
    let reply = client
        .submit(MindRequest::Query(Query {
            kind: QueryKind::Open,
            limit: QueryLimit::new(10),
        }))
        .await
        .expect("query reads durable graph");

    server
        .await
        .expect("second daemon joins")
        .expect("second daemon serves query");

    let MindReply::View(view) = reply else {
        panic!("expected view reply");
    };

    assert_eq!(view.items.len(), 1);
    assert_eq!(view.items[0].title, Title::new("Durable mind memory"));
}
