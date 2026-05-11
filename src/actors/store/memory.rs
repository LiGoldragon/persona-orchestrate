use kameo::actor::{Actor, ActorRef};
use kameo::message::{Context, Message};

use crate::{MemoryGraph, MemoryState, MindEnvelope, StoreLocation};

use super::kernel::{CommitMemoryGraph, KernelCommit, StoreKernel};
use super::persistence::PersistenceRejection;
use super::write_trace::WriteTrace;
use super::{ActorKind, ActorTrace, PipelineReply, TraceAction};

#[derive(Clone)]
pub(super) struct Arguments {
    pub(super) store: StoreLocation,
    pub(super) graph: Option<MemoryGraph>,
    pub(super) kernel: ActorRef<StoreKernel>,
}

pub(super) struct Apply {
    pub(super) envelope: MindEnvelope,
    pub(super) trace: ActorTrace,
}

pub(super) struct Read {
    pub(super) envelope: MindEnvelope,
    pub(super) trace: ActorTrace,
}

pub(super) struct MemoryStore {
    memory: MemoryState,
    kernel: ActorRef<StoreKernel>,
}

impl MemoryStore {
    fn open(arguments: Arguments) -> Self {
        Self {
            memory: MemoryState::open_with_graph(arguments.store, arguments.graph),
            kernel: arguments.kernel,
        }
    }

    async fn apply(&mut self, envelope: MindEnvelope, mut trace: ActorTrace) -> PipelineReply {
        trace.record(ActorKind::MemoryStore, TraceAction::MessageReceived);
        WriteTrace::from_request(envelope.request()).record_into(&mut trace);

        let stage = self.memory.stage_envelope(envelope);
        let reply = stage.reply();

        if let Some(graph) = stage.graph() {
            let commit = self
                .kernel
                .ask(CommitMemoryGraph::new(graph.clone()))
                .await
                .map_err(|error| crate::Error::ActorCall(error.to_string()))
                .map(KernelCommit::into_rejection)
                .unwrap_or_else(|error| Some(PersistenceRejection::reply(error)));
            if commit.is_some() {
                trace.record(ActorKind::EventAppender, TraceAction::MessageReceived);
                trace.record(ActorKind::Commit, TraceAction::CommitCompleted);
                return PipelineReply::new(commit, trace);
            }
        }

        if let Some(graph) = stage.into_graph() {
            self.memory.replace_graph(graph);
        }

        trace.record(ActorKind::EventAppender, TraceAction::MessageReceived);
        trace.record(ActorKind::Commit, TraceAction::CommitCompleted);
        PipelineReply::new(reply, trace)
    }

    fn read(&mut self, envelope: MindEnvelope, mut trace: ActorTrace) -> PipelineReply {
        trace.record(ActorKind::MemoryStore, TraceAction::MessageReceived);
        trace.record(ActorKind::SemaReader, TraceAction::MessageReceived);

        let reply = self.memory.dispatch_envelope(envelope);

        PipelineReply::new(reply, trace)
    }
}

impl Actor for MemoryStore {
    type Args = Arguments;
    type Error = std::convert::Infallible;

    async fn on_start(
        arguments: Self::Args,
        _actor_reference: ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        Ok(Self::open(arguments))
    }
}

impl Message<Apply> for MemoryStore {
    type Reply = PipelineReply;

    async fn handle(
        &mut self,
        message: Apply,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.apply(message.envelope, message.trace).await
    }
}

impl Message<Read> for MemoryStore {
    type Reply = PipelineReply;

    async fn handle(
        &mut self,
        message: Read,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.read(message.envelope, message.trace)
    }
}
