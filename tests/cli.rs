use std::time::{SystemTime, UNIX_EPOCH};

use persona_mind::{MindCommand, MindDaemon, MindDaemonEndpoint, StoreLocation};
use signal_persona_mind::{MindRequest, RoleName};

struct CliFixture {
    endpoint: MindDaemonEndpoint,
    store: StoreLocation,
}

impl CliFixture {
    fn new(test_name: &str) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "persona-mind-cli-{test_name}-{}-{stamp}",
            std::process::id()
        ));
        Self {
            endpoint: MindDaemonEndpoint::new(root.with_extension("sock")),
            store: StoreLocation::new(root.with_extension("redb").to_string_lossy().to_string()),
        }
    }

    async fn bind(&self) -> persona_mind::transport::BoundMindDaemon {
        MindDaemon::new(self.endpoint.clone(), self.store.clone())
            .bind()
            .await
            .expect("daemon binds")
    }
}

#[test]
fn nota_role_claim_text_maps_to_signal_request() {
    let request = persona_mind::MindTextRequest::from_nota(
        "(RoleClaim Operator [(Path \"/git/github.com/LiGoldragon/persona-mind\")] \"claim via text\")",
    )
    .expect("text decodes")
    .into_request()
    .expect("request maps to signal");

    let MindRequest::RoleClaim(claim) = request else {
        panic!("expected role claim");
    };

    assert_eq!(claim.role, RoleName::Operator);
    assert_eq!(claim.scopes.len(), 1);
}

#[tokio::test]
async fn mind_cli_sends_nota_role_claim_to_daemon() {
    let fixture = CliFixture::new("claim");
    let daemon = fixture.bind().await;
    let endpoint = daemon.endpoint().clone();
    let server = tokio::spawn(async move { daemon.serve_one().await });

    let mut output = Vec::new();
    MindCommand::from_arguments([
        "--socket",
        endpoint.as_path().to_str().expect("socket path utf8"),
        "--actor",
        "operator",
        "(RoleClaim Operator [(Path \"/git/github.com/LiGoldragon/persona-mind\")] \"claim via cli\")",
    ])
    .run(&mut output)
    .await
    .expect("cli sends claim");

    server
        .await
        .expect("daemon task joins")
        .expect("daemon serves request");
    let text = String::from_utf8(output).expect("cli output utf8");

    assert_eq!(
        text,
        "(ClaimAcceptance Operator [(Path \"/git/github.com/LiGoldragon/persona-mind\")])\n"
    );
}

#[tokio::test]
async fn mind_cli_reads_role_observation_without_lock_files() {
    let fixture = CliFixture::new("observe");
    let daemon = fixture.bind().await;
    let endpoint = daemon.endpoint().clone();
    let server = tokio::spawn(async move { daemon.serve_count(2).await });

    let mut claim_output = Vec::new();
    MindCommand::from_arguments([
        "--socket",
        endpoint.as_path().to_str().expect("socket path utf8"),
        "--actor",
        "operator",
        "(RoleClaim Operator [(Path \"/git/github.com/LiGoldragon/persona\")] \"claim before observe\")",
    ])
    .run(&mut claim_output)
    .await
    .expect("cli sends claim");

    let mut observation_output = Vec::new();
    MindCommand::from_arguments([
        "--socket",
        endpoint.as_path().to_str().expect("socket path utf8"),
        "--actor",
        "operator",
        "(RoleObservation)",
    ])
    .run(&mut observation_output)
    .await
    .expect("cli reads observation");

    server
        .await
        .expect("daemon task joins")
        .expect("daemon serves requests");
    let text = String::from_utf8(observation_output).expect("cli output utf8");

    assert!(text.contains("(RoleStatus Operator [(ClaimEntry (Path \"/git/github.com/LiGoldragon/persona\") \"claim before observe\")]"));
}
