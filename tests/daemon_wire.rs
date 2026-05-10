use std::time::{SystemTime, UNIX_EPOCH};

use persona_mind::{MindClient, MindDaemon, MindDaemonEndpoint, StoreLocation};
use signal_persona_mind::{
    ActorName, ItemKind, ItemPriority, MindReply, MindRequest, Opening, TextBody, Title,
};

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
    let daemon = MindDaemon::new(
        fixture.endpoint(),
        fixture.store(),
        ActorName::new("operator"),
    )
    .bind()
    .await
    .expect("daemon binds");
    let endpoint = daemon.endpoint().clone();
    let server = tokio::spawn(async move { daemon.serve_one().await });

    let client = MindClient::new(endpoint);
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
async fn client_cannot_reply_without_daemon_signal_frame() {
    let fixture = SocketFixture::new("no-daemon");
    let client = MindClient::new(fixture.endpoint());
    let error = client
        .submit(fixture.request())
        .await
        .expect_err("missing daemon cannot produce reply");

    assert!(matches!(error, persona_mind::Error::Io(_)));
}
