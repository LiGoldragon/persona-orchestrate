use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort};
use signal_persona_mind::MindRequest;

use crate::error::ActorReply;
use crate::{MindEnvelope, Result};

use super::pipeline::PipelineReply;
use super::store;
use super::trace::{ActorKind, ActorTrace, TraceAction};

pub struct DomainSupervisor;

pub struct State {
    store: ActorRef<store::Message>,
}

pub struct Arguments {
    pub store: ActorRef<store::Message>,
}

pub enum Message {
    ApplyMemory {
        envelope: MindEnvelope,
        trace: ActorTrace,
        reply_port: RpcReplyPort<PipelineReply>,
    },
}

impl State {
    pub fn new(store: ActorRef<store::Message>) -> Self {
        Self { store }
    }

    async fn apply_memory(
        &self,
        envelope: MindEnvelope,
        mut trace: ActorTrace,
    ) -> Result<PipelineReply> {
        trace.record(
            ActorKind::DomainSupervisorActor,
            TraceAction::MessageReceived,
        );
        trace.record(
            ActorKind::MemoryGraphSupervisorActor,
            TraceAction::MessageReceived,
        );
        MemoryOperation::from_request(envelope.request()).record_into(&mut trace);

        let raw = self
            .store
            .call(
                |reply_port| store::Message::ApplyMemory {
                    envelope,
                    trace,
                    reply_port,
                },
                None,
            )
            .await
            .map_err(|error| crate::Error::ActorCall(error.to_string()))?;
        ActorReply::new(raw, "store apply memory").into_result()
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

#[ractor::async_trait]
impl Actor for DomainSupervisor {
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
                let reply = match state.apply_memory(envelope, trace).await {
                    Ok(reply) => reply,
                    Err(_error) => PipelineReply::new(None, ActorTrace::new()),
                };
                let _ = reply_port.send(reply);
            }
        }
        Ok(())
    }
}
