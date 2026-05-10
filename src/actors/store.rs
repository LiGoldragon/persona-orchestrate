use kameo::actor::{Actor, ActorRef};
use kameo::message::{Context, Message};
use signal_persona_mind::MindRequest;

use crate::{ActivityLedger, ClaimLedger, MemoryState, MindEnvelope, MindTables, StoreLocation};

use super::pipeline::PipelineReply;
use super::trace::{ActorKind, ActorTrace, TraceAction};

pub(super) struct StoreSupervisor {
    memory: MemoryState,
    tables: MindTables,
}

#[derive(Clone)]
pub(super) struct Arguments {
    pub(super) store: StoreLocation,
}

pub struct ApplyMemory {
    pub envelope: MindEnvelope,
    pub trace: ActorTrace,
}

pub struct ReadMemory {
    pub envelope: MindEnvelope,
    pub trace: ActorTrace,
}

pub struct ApplyClaim {
    pub envelope: MindEnvelope,
    pub trace: ActorTrace,
}

pub struct ApplyHandoff {
    pub envelope: MindEnvelope,
    pub trace: ActorTrace,
}

pub struct ReadClaims {
    pub envelope: MindEnvelope,
    pub trace: ActorTrace,
}

pub struct ApplyActivity {
    pub envelope: MindEnvelope,
    pub trace: ActorTrace,
}

pub struct ReadActivity {
    pub envelope: MindEnvelope,
    pub trace: ActorTrace,
}

impl StoreSupervisor {
    fn new(store: StoreLocation) -> crate::Result<Self> {
        Ok(Self {
            memory: MemoryState::open(store.clone()),
            tables: MindTables::open(&store)?,
        })
    }

    fn apply_memory(&self, envelope: MindEnvelope, mut trace: ActorTrace) -> PipelineReply {
        trace.record(ActorKind::StoreSupervisor, TraceAction::MessageReceived);
        WriteTrace::from_request(envelope.request()).record_into(&mut trace);

        let reply = self.memory.dispatch_envelope(envelope);

        trace.record(ActorKind::EventAppender, TraceAction::MessageReceived);
        trace.record(ActorKind::Commit, TraceAction::CommitCompleted);
        PipelineReply::new(reply, trace)
    }

    fn read_memory(&self, envelope: MindEnvelope, mut trace: ActorTrace) -> PipelineReply {
        trace.record(ActorKind::StoreSupervisor, TraceAction::MessageReceived);
        trace.record(ActorKind::SemaReader, TraceAction::MessageReceived);

        let reply = self.memory.dispatch_envelope(envelope);

        PipelineReply::new(reply, trace)
    }

    fn apply_claim(&mut self, envelope: MindEnvelope, mut trace: ActorTrace) -> PipelineReply {
        trace.record(ActorKind::StoreSupervisor, TraceAction::MessageReceived);
        trace.record(ActorKind::SemaReader, TraceAction::MessageReceived);
        trace.record(ActorKind::SemaWriter, TraceAction::WriteIntentSent);

        let reply = match envelope.request().clone() {
            MindRequest::RoleClaim(claim) => Some(
                ClaimLedger::new(&self.tables)
                    .apply_claim(claim)
                    .unwrap_or_else(Self::persistence_rejection),
            ),
            MindRequest::RoleRelease(release) => Some(
                ClaimLedger::new(&self.tables)
                    .apply_release(release)
                    .unwrap_or_else(Self::persistence_rejection),
            ),
            _ => None,
        };

        trace.record(ActorKind::EventAppender, TraceAction::MessageReceived);
        trace.record(ActorKind::Commit, TraceAction::CommitCompleted);
        PipelineReply::new(reply, trace)
    }

    fn apply_handoff(&mut self, envelope: MindEnvelope, mut trace: ActorTrace) -> PipelineReply {
        trace.record(ActorKind::StoreSupervisor, TraceAction::MessageReceived);
        trace.record(ActorKind::SemaReader, TraceAction::MessageReceived);
        trace.record(ActorKind::SemaWriter, TraceAction::WriteIntentSent);

        let reply = match envelope.request().clone() {
            MindRequest::RoleHandoff(handoff) => Some(
                ClaimLedger::new(&self.tables)
                    .apply_handoff(handoff)
                    .unwrap_or_else(Self::persistence_rejection),
            ),
            _ => None,
        };

        trace.record(ActorKind::EventAppender, TraceAction::MessageReceived);
        trace.record(ActorKind::Commit, TraceAction::CommitCompleted);
        PipelineReply::new(reply, trace)
    }

    fn read_claims(&self, envelope: MindEnvelope, mut trace: ActorTrace) -> PipelineReply {
        trace.record(ActorKind::StoreSupervisor, TraceAction::MessageReceived);
        trace.record(ActorKind::SemaReader, TraceAction::MessageReceived);

        let reply = match envelope.request().clone() {
            MindRequest::RoleObservation(observation) => Some(
                ClaimLedger::new(&self.tables)
                    .observe(observation)
                    .unwrap_or_else(Self::persistence_rejection),
            ),
            _ => None,
        };

        PipelineReply::new(reply, trace)
    }

    fn apply_activity(&self, envelope: MindEnvelope, mut trace: ActorTrace) -> PipelineReply {
        trace.record(ActorKind::StoreSupervisor, TraceAction::MessageReceived);
        trace.record(ActorKind::Clock, TraceAction::MessageReceived);
        trace.record(ActorKind::SemaWriter, TraceAction::WriteIntentSent);

        let reply = match envelope.request().clone() {
            MindRequest::ActivitySubmission(submission) => Some(
                ActivityLedger::new(&self.tables)
                    .submit(submission)
                    .unwrap_or_else(Self::persistence_rejection),
            ),
            _ => None,
        };

        trace.record(ActorKind::ActivityAppender, TraceAction::MessageReceived);
        trace.record(ActorKind::Commit, TraceAction::CommitCompleted);
        PipelineReply::new(reply, trace)
    }

    fn read_activity(&self, envelope: MindEnvelope, mut trace: ActorTrace) -> PipelineReply {
        trace.record(ActorKind::StoreSupervisor, TraceAction::MessageReceived);
        trace.record(ActorKind::SemaReader, TraceAction::MessageReceived);
        trace.record(ActorKind::RecentActivityView, TraceAction::MessageReceived);

        let reply = match envelope.request().clone() {
            MindRequest::ActivityQuery(query) => Some(
                ActivityLedger::new(&self.tables)
                    .query(query)
                    .unwrap_or_else(Self::persistence_rejection),
            ),
            _ => None,
        };

        PipelineReply::new(reply, trace)
    }

    fn persistence_rejection(_error: crate::Error) -> signal_persona_mind::MindReply {
        signal_persona_mind::MindReply::Rejection(signal_persona_mind::Rejection {
            reason: signal_persona_mind::RejectionReason::PersistenceRejected,
        })
    }
}

impl Actor for StoreSupervisor {
    type Args = Arguments;
    type Error = crate::Error;

    async fn on_start(
        arguments: Self::Args,
        _actor_reference: ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        Self::new(arguments.store)
    }
}

impl Message<ApplyMemory> for StoreSupervisor {
    type Reply = PipelineReply;

    async fn handle(
        &mut self,
        message: ApplyMemory,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.apply_memory(message.envelope, message.trace)
    }
}

impl Message<ReadMemory> for StoreSupervisor {
    type Reply = PipelineReply;

    async fn handle(
        &mut self,
        message: ReadMemory,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.read_memory(message.envelope, message.trace)
    }
}

impl Message<ApplyClaim> for StoreSupervisor {
    type Reply = PipelineReply;

    async fn handle(
        &mut self,
        message: ApplyClaim,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.apply_claim(message.envelope, message.trace)
    }
}

impl Message<ApplyHandoff> for StoreSupervisor {
    type Reply = PipelineReply;

    async fn handle(
        &mut self,
        message: ApplyHandoff,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.apply_handoff(message.envelope, message.trace)
    }
}

impl Message<ReadClaims> for StoreSupervisor {
    type Reply = PipelineReply;

    async fn handle(
        &mut self,
        message: ReadClaims,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.read_claims(message.envelope, message.trace)
    }
}

impl Message<ApplyActivity> for StoreSupervisor {
    type Reply = PipelineReply;

    async fn handle(
        &mut self,
        message: ApplyActivity,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.apply_activity(message.envelope, message.trace)
    }
}

impl Message<ReadActivity> for StoreSupervisor {
    type Reply = PipelineReply;

    async fn handle(
        &mut self,
        message: ReadActivity,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.read_activity(message.envelope, message.trace)
    }
}

struct WriteTrace {
    reads_existing_graph: bool,
    mints_identity: bool,
}

impl WriteTrace {
    fn from_request(request: &MindRequest) -> Self {
        match request {
            MindRequest::Opening(_) => Self {
                reads_existing_graph: false,
                mints_identity: true,
            },
            MindRequest::NoteSubmission(_)
            | MindRequest::Link(_)
            | MindRequest::StatusChange(_)
            | MindRequest::AliasAssignment(_) => Self {
                reads_existing_graph: true,
                mints_identity: false,
            },
            _ => Self {
                reads_existing_graph: false,
                mints_identity: false,
            },
        }
    }

    fn record_into(&self, trace: &mut ActorTrace) {
        if self.reads_existing_graph {
            trace.record(ActorKind::SemaReader, TraceAction::MessageReceived);
        }
        if self.mints_identity {
            trace.record(ActorKind::IdMint, TraceAction::MessageReceived);
        }
        trace.record(ActorKind::Clock, TraceAction::MessageReceived);
        trace.record(ActorKind::SemaWriter, TraceAction::WriteIntentSent);
    }
}
