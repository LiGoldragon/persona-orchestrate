use kameo::actor::{Actor, ActorRef};
use kameo::error::Infallible;
use kameo::message::{Context, Message};
use signal_persona_mind::MindRequest;

use crate::{MemoryState, MindEnvelope, StoreLocation};

use super::pipeline::PipelineReply;
use super::trace::{ActorKind, ActorTrace, TraceAction};

pub(super) struct StoreSupervisor {
    memory: MemoryState,
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

impl StoreSupervisor {
    fn new(store: StoreLocation) -> Self {
        Self {
            memory: MemoryState::open(store),
        }
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
}

impl Actor for StoreSupervisor {
    type Args = Arguments;
    type Error = Infallible;

    async fn on_start(
        arguments: Self::Args,
        _actor_reference: ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        Ok(Self::new(arguments.store))
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

struct WriteTrace {
    reads_existing_graph: bool,
    mints_identity: bool,
}

impl WriteTrace {
    fn from_request(request: &MindRequest) -> Self {
        match request {
            MindRequest::Open(_) => Self {
                reads_existing_graph: false,
                mints_identity: true,
            },
            MindRequest::AddNote(_)
            | MindRequest::Link(_)
            | MindRequest::ChangeStatus(_)
            | MindRequest::AddAlias(_) => Self {
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
