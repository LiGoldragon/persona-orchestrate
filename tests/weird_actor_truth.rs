use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use persona_mind::actors::{ActorKind, ActorManifest, ActorResidency};
use persona_mind::{MindEnvelope, MindRuntime, StoreLocation};
use signal_persona_mind::{
    ActorName, Body, Kind, MindReply, MindRequest, Opening, Priority, Query, QueryKind, QueryLimit,
    RoleClaim, RoleName, ScopeReason, ScopeReference, Title, WirePath,
};

struct SourceTree {
    root: PathBuf,
}

struct SourceFile {
    path: PathBuf,
    text: String,
}

struct RustFileCollector {
    pending_paths: Vec<PathBuf>,
    files: Vec<SourceFile>,
}

struct GuardedFileCollector {
    pending_paths: Vec<PathBuf>,
    files: Vec<SourceFile>,
}

struct ForbiddenFragment {
    text: &'static str,
    reason: &'static str,
}

struct SourceViolation {
    path: PathBuf,
    line_number: usize,
    reason: String,
}

struct ActorRuntimeFixture {
    runtime: MindRuntime,
    actor: ActorName,
}

impl SourceTree {
    fn new() -> Self {
        Self {
            root: PathBuf::from(env!("CARGO_MANIFEST_DIR")),
        }
    }

    fn source_files(&self) -> Vec<SourceFile> {
        RustFileCollector::new(self.root.join("src")).into_files()
    }

    fn guarded_files(&self) -> Vec<SourceFile> {
        GuardedFileCollector::new(self.root.clone()).into_files()
    }
}

impl RustFileCollector {
    fn new(root: PathBuf) -> Self {
        Self {
            pending_paths: vec![root],
            files: Vec::new(),
        }
    }

    fn into_files(mut self) -> Vec<SourceFile> {
        while let Some(path) = self.pending_paths.pop() {
            self.visit_path(path);
        }
        self.files
    }

    fn visit_path(&mut self, path: PathBuf) {
        if path.is_dir() {
            let mut child_paths = fs::read_dir(&path)
                .expect("source directory is readable")
                .map(|entry| entry.expect("source entry is readable").path())
                .collect::<Vec<_>>();
            child_paths.sort();
            child_paths.reverse();
            self.pending_paths.extend(child_paths);
            return;
        }

        if path.extension().is_some_and(|extension| extension == "rs") {
            let text = fs::read_to_string(&path).expect("source file is readable");
            self.files.push(SourceFile { path, text });
        }
    }
}

impl GuardedFileCollector {
    fn new(root: PathBuf) -> Self {
        Self {
            pending_paths: vec![
                root.join("src"),
                root.join("tests"),
                root.join("Cargo.toml"),
                root.join("Cargo.lock"),
                root.join("flake.nix"),
                root.join("README.md"),
                root.join("ARCHITECTURE.md"),
            ],
            files: Vec::new(),
        }
    }

    fn into_files(mut self) -> Vec<SourceFile> {
        while let Some(path) = self.pending_paths.pop() {
            self.visit_path(path);
        }
        self.files
    }

    fn visit_path(&mut self, path: PathBuf) {
        if path.is_dir() {
            let mut child_paths = fs::read_dir(&path)
                .expect("guarded directory is readable")
                .map(|entry| entry.expect("guarded entry is readable").path())
                .collect::<Vec<_>>();
            child_paths.sort();
            child_paths.reverse();
            self.pending_paths.extend(child_paths);
            return;
        }

        if !path.is_file() {
            return;
        }

        let text = fs::read_to_string(&path).expect("guarded file is readable");
        self.files.push(SourceFile { path, text });
    }
}

impl SourceFile {
    fn relative_name(&self) -> String {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        self.path
            .strip_prefix(root)
            .expect("source file lives under manifest directory")
            .display()
            .to_string()
    }

    fn is_root_actor_source(&self) -> bool {
        self.relative_name() == "src/actors/root.rs"
    }

    fn is_this_guard_source(&self) -> bool {
        self.relative_name() == "tests/weird_actor_truth.rs"
    }

    fn violations_for(&self, fragment: &ForbiddenFragment) -> Vec<SourceViolation> {
        self.text
            .lines()
            .enumerate()
            .filter(|(_index, line)| line.contains(fragment.text))
            .map(|(index, _line)| SourceViolation {
                path: self.path.clone(),
                line_number: index + 1,
                reason: fragment.reason.to_string(),
            })
            .collect()
    }

    fn public_zst_actor_violations(&self) -> Vec<SourceViolation> {
        self.text
            .lines()
            .enumerate()
            .filter(|(_index, line)| {
                let trimmed = line.trim();
                trimmed.starts_with("pub struct ") && trimmed.ends_with(';')
            })
            .map(|(index, line)| SourceViolation {
                path: self.path.clone(),
                line_number: index + 1,
                reason: format!("public ZST actor marker: {line}"),
            })
            .collect()
    }
}

impl SourceViolation {
    fn summary(&self) -> String {
        format!(
            "{}:{}: {}",
            self.path.display(),
            self.line_number,
            self.reason
        )
    }
}

impl ActorRuntimeFixture {
    async fn new(actor: ActorName) -> Self {
        Self {
            runtime: MindRuntime::start(StoreLocation::new("mind.redb"))
                .await
                .expect("ractor runtime starts"),
            actor,
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
        self.runtime.stop().await.expect("ractor runtime stops");
    }
}

#[test]
fn raw_actor_spawn_cannot_escape_mind_root() {
    let violations = SourceTree::new()
        .source_files()
        .into_iter()
        .filter(|file| !file.is_root_actor_source())
        .flat_map(|file| {
            file.violations_for(&ForbiddenFragment {
                text: "Actor::spawn",
                reason: "raw ractor spawn outside MindRootActor",
            })
        })
        .collect::<Vec<_>>();

    assert!(
        violations.is_empty(),
        "raw spawn violations:\n{}",
        violations
            .iter()
            .map(SourceViolation::summary)
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn actor_source_cannot_hide_shared_locks_or_polling_waits() {
    let forbidden_fragments = [
        ForbiddenFragment {
            text: "Arc<Mutex",
            reason: "shared mutex state between actors",
        },
        ForbiddenFragment {
            text: "Arc < Mutex",
            reason: "shared mutex state between actors",
        },
        ForbiddenFragment {
            text: "RwLock",
            reason: "shared read-write lock state between actors",
        },
        ForbiddenFragment {
            text: "std::thread::sleep",
            reason: "blocking sleep in actor source",
        },
        ForbiddenFragment {
            text: "tokio::time::sleep",
            reason: "timer sleep in actor source",
        },
        ForbiddenFragment {
            text: "tokio::time::interval",
            reason: "polling interval in actor source",
        },
    ];

    let violations = SourceTree::new()
        .source_files()
        .into_iter()
        .flat_map(|file| {
            forbidden_fragments
                .iter()
                .flat_map(|fragment| file.violations_for(fragment))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert!(
        violations.is_empty(),
        "actor source discipline violations:\n{}",
        violations
            .iter()
            .map(SourceViolation::summary)
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn actor_adapter_markers_cannot_be_public_zst_nouns() {
    let violations = SourceTree::new()
        .source_files()
        .into_iter()
        .filter(|file| file.relative_name().starts_with("src/actors/"))
        .flat_map(|file| file.public_zst_actor_violations())
        .collect::<Vec<_>>();

    assert!(
        violations.is_empty(),
        "public actor ZST violations:\n{}",
        violations
            .iter()
            .map(SourceViolation::summary)
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn ractor_is_the_only_actor_library_boundary() {
    let forbidden_fragments = [
        ForbiddenFragment {
            text: "persona-actor",
            reason: "invented actor abstraction name",
        },
        ForbiddenFragment {
            text: "workspace-actor",
            reason: "invented actor abstraction name",
        },
        ForbiddenFragment {
            text: "workspace_actor",
            reason: "invented actor abstraction namespace",
        },
        ForbiddenFragment {
            text: "PersonaActor",
            reason: "invented actor abstraction type",
        },
        ForbiddenFragment {
            text: "WorkspaceActor",
            reason: "invented actor abstraction type",
        },
        ForbiddenFragment {
            text: "actix =",
            reason: "non-ractor actor dependency",
        },
        ForbiddenFragment {
            text: "name = \"actix\"",
            reason: "non-ractor actor dependency",
        },
        ForbiddenFragment {
            text: "xtra =",
            reason: "non-ractor actor dependency",
        },
        ForbiddenFragment {
            text: "name = \"xtra\"",
            reason: "non-ractor actor dependency",
        },
        ForbiddenFragment {
            text: "bastion =",
            reason: "non-ractor actor dependency",
        },
        ForbiddenFragment {
            text: "name = \"bastion\"",
            reason: "non-ractor actor dependency",
        },
        ForbiddenFragment {
            text: "kameo =",
            reason: "non-ractor actor dependency",
        },
        ForbiddenFragment {
            text: "name = \"kameo\"",
            reason: "non-ractor actor dependency",
        },
        ForbiddenFragment {
            text: "coerce =",
            reason: "non-ractor actor dependency",
        },
        ForbiddenFragment {
            text: "name = \"coerce\"",
            reason: "non-ractor actor dependency",
        },
        ForbiddenFragment {
            text: "kompact =",
            reason: "non-ractor actor dependency",
        },
        ForbiddenFragment {
            text: "name = \"kompact\"",
            reason: "non-ractor actor dependency",
        },
        ForbiddenFragment {
            text: "stakker =",
            reason: "non-ractor actor dependency",
        },
        ForbiddenFragment {
            text: "name = \"stakker\"",
            reason: "non-ractor actor dependency",
        },
    ];

    let violations = SourceTree::new()
        .guarded_files()
        .into_iter()
        .filter(|file| !file.is_this_guard_source())
        .flat_map(|file| {
            forbidden_fragments
                .iter()
                .flat_map(|fragment| file.violations_for(fragment))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert!(
        violations.is_empty(),
        "non-ractor actor boundary violations:\n{}",
        violations
            .iter()
            .map(SourceViolation::summary)
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn trace_phase_actor_cannot_float_without_parent_edge() {
    let manifest = ActorManifest::persona_mind_phase_one();
    let missing_parents = manifest
        .actors()
        .iter()
        .filter(|actor| actor.residency() != ActorResidency::Root)
        .filter(|actor| {
            !manifest
                .edges()
                .iter()
                .any(|edge| edge.child() == actor.kind())
        })
        .map(|actor| actor.kind().label())
        .collect::<Vec<_>>();

    assert!(
        missing_parents.is_empty(),
        "manifest actors without parent edges: {}",
        missing_parents.join(", ")
    );
}

#[test]
fn actor_kind_labels_cannot_collide() {
    let manifest = ActorManifest::persona_mind_phase_one();
    let mut labels = HashSet::new();
    let duplicate_labels = manifest
        .actors()
        .iter()
        .map(|actor| actor.kind().label())
        .filter(|label| !labels.insert(*label))
        .collect::<Vec<_>>();

    assert!(
        duplicate_labels.is_empty(),
        "duplicate actor labels: {}",
        duplicate_labels.join(", ")
    );
}

#[tokio::test]
async fn unsupported_claim_cannot_use_success_reply_path_or_writer() {
    let fixture = ActorRuntimeFixture::new(ActorName::new("operator-assistant")).await;
    let response = fixture
        .submit(MindRequest::RoleClaim(RoleClaim {
            role: RoleName::Operator,
            scopes: vec![ScopeReference::Path(
                WirePath::from_absolute_path("/git/github.com/LiGoldragon/persona-mind")
                    .expect("test path is absolute"),
            )],
            reason: ScopeReason::from_text("unsupported claim witness")
                .expect("test reason is valid"),
        }))
        .await;

    assert!(response.reply().is_none());
    assert!(response.trace().contains(ActorKind::ClaimFlowActor));
    assert!(response.trace().contains(ActorKind::ErrorShapeActor));
    assert!(!response.trace().contains(ActorKind::NotaReplyEncodeActor));
    assert!(!response.trace().contains(ActorKind::SemaWriterActor));

    fixture.stop().await;
}

#[tokio::test]
async fn parallel_runtimes_cannot_share_registry_names_or_memory() {
    let first_runtime = ActorRuntimeFixture::new(ActorName::new("operator")).await;
    let second_runtime = ActorRuntimeFixture::new(ActorName::new("designer")).await;

    let first_reply = first_runtime
        .submit(MindRequest::Open(Opening {
            kind: Kind::Task,
            priority: Priority::High,
            title: Title::new("First runtime item"),
            body: Body::new("only the first runtime sees this"),
        }))
        .await;
    let second_reply = second_runtime
        .submit(MindRequest::Open(Opening {
            kind: Kind::Task,
            priority: Priority::Low,
            title: Title::new("Second runtime item"),
            body: Body::new("only the second runtime sees this"),
        }))
        .await;

    let MindReply::Opened(first_receipt) = first_reply.reply().expect("first reply exists") else {
        panic!("expected first opened reply");
    };
    let MindReply::Opened(second_receipt) = second_reply.reply().expect("second reply exists")
    else {
        panic!("expected second opened reply");
    };

    assert_eq!(first_receipt.event.header.actor, ActorName::new("operator"));
    assert_eq!(
        second_receipt.event.header.actor,
        ActorName::new("designer")
    );

    let first_view = first_runtime
        .submit(MindRequest::Query(Query {
            kind: QueryKind::Open,
            limit: QueryLimit::new(10),
        }))
        .await;
    let second_view = second_runtime
        .submit(MindRequest::Query(Query {
            kind: QueryKind::Open,
            limit: QueryLimit::new(10),
        }))
        .await;

    let MindReply::View(first_items) = first_view.reply().expect("first view exists") else {
        panic!("expected first view");
    };
    let MindReply::View(second_items) = second_view.reply().expect("second view exists") else {
        panic!("expected second view");
    };

    assert_eq!(first_items.items.len(), 1);
    assert_eq!(second_items.items.len(), 1);
    assert_eq!(first_items.items[0].title, Title::new("First runtime item"));
    assert_eq!(
        second_items.items[0].title,
        Title::new("Second runtime item")
    );

    first_runtime.stop().await;
    second_runtime.stop().await;
}
