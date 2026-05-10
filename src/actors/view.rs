use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort};
use signal_persona_mind::{MindRequest, QueryKind};

use crate::error::ActorReply;
use crate::{MindEnvelope, Result};

use super::pipeline::PipelineReply;
use super::store;
use super::trace::{ActorKind, ActorTrace, TraceAction};

pub struct ViewSupervisor;

pub struct State {
    store: ActorRef<store::Message>,
}

pub struct Arguments {
    pub store: ActorRef<store::Message>,
}

pub enum Message {
    ReadMemory {
        envelope: MindEnvelope,
        trace: ActorTrace,
        reply_port: RpcReplyPort<PipelineReply>,
    },
}

impl State {
    pub fn new(store: ActorRef<store::Message>) -> Self {
        Self { store }
    }

    async fn read_memory(
        &self,
        envelope: MindEnvelope,
        mut trace: ActorTrace,
    ) -> Result<PipelineReply> {
        trace.record(ActorKind::ViewSupervisorActor, TraceAction::MessageReceived);
        trace.record(
            ActorKind::QuerySupervisorActor,
            TraceAction::MessageReceived,
        );
        trace.record(ActorKind::QueryPlanActor, TraceAction::MessageReceived);
        QueryOperation::from_request(envelope.request()).record_into(&mut trace);
        trace.record(ActorKind::GraphTraversalActor, TraceAction::MessageReceived);

        let raw = self
            .store
            .call(
                |reply_port| store::Message::ReadMemory {
                    envelope,
                    trace,
                    reply_port,
                },
                None,
            )
            .await
            .map_err(|error| crate::Error::ActorCall(error.to_string()))?;
        let mut reply = ActorReply::new(raw, "store read memory").into_result()?;
        reply.trace.record(
            ActorKind::QueryResultShapeActor,
            TraceAction::MessageReceived,
        );
        Ok(reply)
    }
}

struct QueryOperation {
    actor: ActorKind,
}

impl QueryOperation {
    fn from_request(request: &MindRequest) -> Self {
        let actor = match request {
            MindRequest::Query(query) => match &query.kind {
                QueryKind::Ready => ActorKind::ReadyWorkViewActor,
                QueryKind::Blocked => ActorKind::BlockedWorkViewActor,
                QueryKind::RecentEvents => ActorKind::RecentActivityViewActor,
                QueryKind::Open
                | QueryKind::ByItem(_)
                | QueryKind::ByKind(_)
                | QueryKind::ByStatus(_)
                | QueryKind::ByAlias(_) => ActorKind::GraphTraversalActor,
            },
            _ => ActorKind::ErrorShapeActor,
        };
        Self { actor }
    }

    fn record_into(&self, trace: &mut ActorTrace) {
        trace.record(self.actor, TraceAction::MessageReceived);
    }
}

#[ractor::async_trait]
impl Actor for ViewSupervisor {
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
            Message::ReadMemory {
                envelope,
                trace,
                reply_port,
            } => {
                let reply = match state.read_memory(envelope, trace).await {
                    Ok(reply) => reply,
                    Err(_error) => PipelineReply::new(None, ActorTrace::new()),
                };
                let _ = reply_port.send(reply);
            }
        }
        Ok(())
    }
}
