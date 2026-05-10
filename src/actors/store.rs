use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort};
use signal_persona_mind::MindRequest;

use crate::{MemoryState, MindEnvelope, StoreLocation};

use super::pipeline::PipelineReply;
use super::trace::{ActorKind, ActorTrace, TraceAction};

pub struct StoreSupervisor;

pub struct State {
    memory: MemoryState,
}

pub struct Arguments {
    pub store: StoreLocation,
}

pub enum Message {
    ApplyMemory {
        envelope: MindEnvelope,
        trace: ActorTrace,
        reply_port: RpcReplyPort<PipelineReply>,
    },
    ReadMemory {
        envelope: MindEnvelope,
        trace: ActorTrace,
        reply_port: RpcReplyPort<PipelineReply>,
    },
}

impl State {
    pub fn new(store: StoreLocation) -> Self {
        Self {
            memory: MemoryState::open(store),
        }
    }

    fn apply_memory(&self, envelope: MindEnvelope, mut trace: ActorTrace) -> PipelineReply {
        trace.record(
            ActorKind::StoreSupervisorActor,
            TraceAction::MessageReceived,
        );
        WriteTrace::from_request(envelope.request()).record_into(&mut trace);

        let reply = self.memory.dispatch_envelope(envelope);

        trace.record(ActorKind::EventAppendActor, TraceAction::MessageReceived);
        trace.record(ActorKind::CommitActor, TraceAction::CommitCompleted);
        PipelineReply::new(reply, trace)
    }

    fn read_memory(&self, envelope: MindEnvelope, mut trace: ActorTrace) -> PipelineReply {
        trace.record(
            ActorKind::StoreSupervisorActor,
            TraceAction::MessageReceived,
        );
        trace.record(ActorKind::SemaReadActor, TraceAction::MessageReceived);

        let reply = self.memory.dispatch_envelope(envelope);

        PipelineReply::new(reply, trace)
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
            trace.record(ActorKind::SemaReadActor, TraceAction::MessageReceived);
        }
        if self.mints_identity {
            trace.record(ActorKind::IdMintActor, TraceAction::MessageReceived);
        }
        trace.record(ActorKind::ClockActor, TraceAction::MessageReceived);
        trace.record(ActorKind::SemaWriterActor, TraceAction::WriteIntentSent);
    }
}

#[ractor::async_trait]
impl Actor for StoreSupervisor {
    type Msg = Message;
    type State = State;
    type Arguments = Arguments;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        arguments: Arguments,
    ) -> std::result::Result<Self::State, ActorProcessingErr> {
        Ok(State::new(arguments.store))
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Message,
        state: &mut State,
    ) -> std::result::Result<(), ActorProcessingErr> {
        match message {
            Message::ApplyMemory {
                envelope,
                trace,
                reply_port,
            } => {
                let _ = reply_port.send(state.apply_memory(envelope, trace));
            }
            Message::ReadMemory {
                envelope,
                trace,
                reply_port,
            } => {
                let _ = reply_port.send(state.read_memory(envelope, trace));
            }
        }
        Ok(())
    }
}
