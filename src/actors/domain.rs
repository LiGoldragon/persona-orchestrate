use kameo::actor::{Actor, ActorRef};
use kameo::error::Infallible;
use kameo::message::{Context, Message};
use signal_persona_mind::MindRequest;

use crate::{MindEnvelope, Result as CrateResult};

use super::pipeline::PipelineReply;
use super::store;
use super::trace::{ActorKind, ActorTrace, TraceAction};

pub(super) struct DomainSupervisorActor {
    store: ActorRef<store::StoreSupervisorActor>,
}

#[derive(Clone)]
pub(super) struct Arguments {
    pub(super) store: ActorRef<store::StoreSupervisorActor>,
}

pub struct ApplyMemory {
    pub envelope: MindEnvelope,
    pub trace: ActorTrace,
}

impl DomainSupervisorActor {
    fn new(store: ActorRef<store::StoreSupervisorActor>) -> Self {
        Self { store }
    }

    async fn apply_memory(
        &self,
        envelope: MindEnvelope,
        mut trace: ActorTrace,
    ) -> CrateResult<PipelineReply> {
        trace.record(
            ActorKind::DomainSupervisorActor,
            TraceAction::MessageReceived,
        );
        trace.record(
            ActorKind::MemoryGraphSupervisorActor,
            TraceAction::MessageReceived,
        );
        MemoryOperation::from_request(envelope.request()).record_into(&mut trace);

        self.store
            .ask(store::ApplyMemory { envelope, trace })
            .await
            .map_err(|error| crate::Error::ActorCall(error.to_string()))
    }
}

impl Actor for DomainSupervisorActor {
    type Args = Arguments;
    type Error = Infallible;

    async fn on_start(
        arguments: Self::Args,
        _actor_reference: ActorRef<Self>,
    ) -> std::result::Result<Self, Self::Error> {
        Ok(Self::new(arguments.store))
    }
}

impl Message<ApplyMemory> for DomainSupervisorActor {
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
            MindRequest::Open(_) => ActorKind::ItemOpenActor,
            MindRequest::AddNote(_) => ActorKind::NoteAddActor,
            MindRequest::Link(_) => ActorKind::LinkActor,
            MindRequest::ChangeStatus(_) => ActorKind::StatusChangeActor,
            MindRequest::AddAlias(_) => ActorKind::AliasAddActor,
            _ => ActorKind::ErrorShapeActor,
        };
        Self { actor }
    }

    fn record_into(&self, trace: &mut ActorTrace) {
        trace.record(self.actor, TraceAction::MessageReceived);
    }
}
