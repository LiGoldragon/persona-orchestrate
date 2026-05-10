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

#[test]
fn nota_activity_submission_text_maps_to_signal_request() {
    let request = persona_mind::MindTextRequest::from_nota(
        "(ActivitySubmission Operator (Path \"/git/github.com/LiGoldragon/persona-mind\") \"activity via text\")",
    )
    .expect("text decodes")
    .into_request()
    .expect("request maps to signal");

    let MindRequest::ActivitySubmission(submission) = request else {
        panic!("expected activity submission");
    };

    assert_eq!(submission.role, RoleName::Operator);
}

#[test]
fn nota_role_handoff_text_maps_to_signal_request() {
    let request = persona_mind::MindTextRequest::from_nota(
        "(RoleHandoff Operator Designer [(Path \"/git/github.com/LiGoldragon/persona-mind\")] \"handoff via text\")",
    )
    .expect("text decodes")
    .into_request()
    .expect("request maps to signal");

    let MindRequest::RoleHandoff(handoff) = request else {
        panic!("expected role handoff");
    };

    assert_eq!(handoff.from, RoleName::Operator);
    assert_eq!(handoff.to, RoleName::Designer);
    assert_eq!(handoff.scopes.len(), 1);
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
async fn mind_cli_sends_activity_submission_and_query_to_daemon() {
    let fixture = CliFixture::new("activity");
    let daemon = fixture.bind().await;
    let endpoint = daemon.endpoint().clone();
    let server = tokio::spawn(async move { daemon.serve_count(2).await });

    let mut submission_output = Vec::new();
    MindCommand::from_arguments([
        "--socket",
        endpoint.as_path().to_str().expect("socket path utf8"),
        "--actor",
        "operator",
        "(ActivitySubmission Operator (Path \"/git/github.com/LiGoldragon/persona-mind\") \"activity via cli\")",
    ])
    .run(&mut submission_output)
    .await
    .expect("cli sends activity");

    let mut query_output = Vec::new();
    MindCommand::from_arguments([
        "--socket",
        endpoint.as_path().to_str().expect("socket path utf8"),
        "--actor",
        "operator",
        "(ActivityQuery 5 [])",
    ])
    .run(&mut query_output)
    .await
    .expect("cli queries activity");

    server
        .await
        .expect("daemon task joins")
        .expect("daemon serves activity requests");

    assert_eq!(
        String::from_utf8(submission_output).expect("cli output utf8"),
        "(ActivityAcknowledgment 0)\n"
    );
    let text = String::from_utf8(query_output).expect("cli output utf8");
    assert!(text.contains("(ActivityList [(Activity Operator (Path \"/git/github.com/LiGoldragon/persona-mind\") \"activity via cli\""));
}

#[tokio::test]
async fn mind_cli_sends_role_handoff_to_daemon() {
    let fixture = CliFixture::new("handoff");
    let daemon = fixture.bind().await;
    let endpoint = daemon.endpoint().clone();
    let server = tokio::spawn(async move { daemon.serve_count(3).await });

    let mut claim_output = Vec::new();
    MindCommand::from_arguments([
        "--socket",
        endpoint.as_path().to_str().expect("socket path utf8"),
        "--actor",
        "operator",
        "(RoleClaim Operator [(Path \"/git/github.com/LiGoldragon/persona-mind\")] \"claim before handoff\")",
    ])
    .run(&mut claim_output)
    .await
    .expect("cli sends claim");

    let mut handoff_output = Vec::new();
    MindCommand::from_arguments([
        "--socket",
        endpoint.as_path().to_str().expect("socket path utf8"),
        "--actor",
        "operator",
        "(RoleHandoff Operator Designer [(Path \"/git/github.com/LiGoldragon/persona-mind\")] \"handoff via cli\")",
    ])
    .run(&mut handoff_output)
    .await
    .expect("cli sends handoff");

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

    assert_eq!(
        String::from_utf8(handoff_output).expect("cli output utf8"),
        "(HandoffAcceptance Operator Designer [(Path \"/git/github.com/LiGoldragon/persona-mind\")])\n"
    );
    let text = String::from_utf8(observation_output).expect("cli output utf8");
    assert!(text.contains("(RoleStatus Operator [])"));
    assert!(text.contains("(RoleStatus Designer [(ClaimEntry (Path \"/git/github.com/LiGoldragon/persona-mind\") \"handoff via cli\")]"));
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
