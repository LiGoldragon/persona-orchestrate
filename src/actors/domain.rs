use kameo::actor::{Actor, ActorRef};
use kameo::error::Infallible;
use kameo::message::{Context, Message};
use signal_persona_mind::MindRequest;

use crate::{MindEnvelope, Result as CrateResult};

use super::pipeline::PipelineReply;
use super::store;
use super::trace::{ActorKind, ActorTrace, TraceAction};

pub(super) struct DomainSupervisor {
    store: ActorRef<store::StoreSupervisor>,
}

#[derive(Clone)]
pub(super) struct Arguments {
    pub(super) store: ActorRef<store::StoreSupervisor>,
}

pub struct ApplyMemory {
    pub envelope: MindEnvelope,
    pub trace: ActorTrace,
}

impl DomainSupervisor {
    fn new(store: ActorRef<store::StoreSupervisor>) -> Self {
        Self { store }
    }

    async fn apply_memory(
        &self,
        envelope: MindEnvelope,
        mut trace: ActorTrace,
    ) -> CrateResult<PipelineReply> {
        trace.record(ActorKind::DomainSupervisor, TraceAction::MessageReceived);
        trace.record(
            ActorKind::MemoryGraphSupervisor,
            TraceAction::MessageReceived,
        );
        MemoryOperation::from_request(envelope.request()).record_into(&mut trace);

        self.store
            .ask(store::ApplyMemory { envelope, trace })
            .await
            .map_err(|error| crate::Error::ActorCall(error.to_string()))
    }
}

impl Actor for DomainSupervisor {
    type Args = Arguments;
    type Error = Infallible;

    async fn on_start(
        arguments: Self::Args,
        _actor_reference: ActorRef<Self>,
    ) -> std::result::Result<Self, Self::Error> {
        Ok(Self::new(arguments.store))
    }
}

impl Message<ApplyMemory> for DomainSupervisor {
    type Reply = PipelineReply;

    async fn handle(
        &mut self,
        message: ApplyMemory,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        match self.apply_memory(message.envelope, message.trace).await {
            Ok(reply) => reply,
            Err(_error) => PipelineReply::new(None, ActorTrace::new()),
        }
    }
}

struct MemoryOperation {
    actor: ActorKind,
}

impl MemoryOperation {
    fn from_request(request: &MindRequest) -> Self {
        let actor = match request {
            MindRequest::Opening(_) => ActorKind::ItemOpen,
            MindRequest::NoteSubmission(_) => ActorKind::NoteAdd,
            MindRequest::Link(_) => ActorKind::Link,
            MindRequest::StatusChange(_) => ActorKind::StatusChange,
            MindRequest::AliasAssignment(_) => ActorKind::AliasAdd,
            _ => ActorKind::ErrorShaper,
        };
        Self { actor }
    }

    fn record_into(&self, trace: &mut ActorTrace) {
        trace.record(self.actor, TraceAction::MessageReceived);
    }
}
