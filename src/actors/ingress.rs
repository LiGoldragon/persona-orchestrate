use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort};

use crate::error::ActorReply;
use crate::{MindEnvelope, Result};

use super::dispatch;
use super::pipeline::PipelineReply;
use super::trace::{ActorKind, ActorTrace, TraceAction};

pub(super) struct IngressSupervisor;

pub struct State {
    dispatch: ActorRef<dispatch::Message>,
}

pub struct Arguments {
    pub dispatch: ActorRef<dispatch::Message>,
}

pub enum Message {
    Accept {
        envelope: MindEnvelope,
        trace: ActorTrace,
        reply_port: RpcReplyPort<PipelineReply>,
    },
}

impl State {
    pub fn new(dispatch: ActorRef<dispatch::Message>) -> Self {
        Self { dispatch }
    }

    async fn accept(&self, envelope: MindEnvelope, mut trace: ActorTrace) -> Result<PipelineReply> {
        trace.record(
            ActorKind::IngressSupervisorActor,
            TraceAction::MessageReceived,
        );
        trace.record(ActorKind::RequestSessionActor, TraceAction::MessageReceived);
        trace.record(ActorKind::NotaDecodeActor, TraceAction::MessageReceived);
        trace.record(ActorKind::CallerIdentityActor, TraceAction::MessageReceived);
        trace.record(ActorKind::EnvelopeActor, TraceAction::MessageReplied);

        let raw = self
            .dispatch
            .call(
                |reply_port| dispatch::Message::Route {
                    envelope,
                    trace,
                    reply_port,
                },
                None,
            )
            .await
            .map_err(|error| crate::Error::ActorCall(error.to_string()))?;
        ActorReply::new(raw, "dispatch route").into_result()
    }
}

#[ractor::async_trait]
impl Actor for IngressSupervisor {
    type Msg = Message;
    type State = State;
    type Arguments = Arguments;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        arguments: Arguments,
    ) -> std::result::Result<Self::State, ActorProcessingErr> {
        Ok(State::new(arguments.dispatch))
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Message,
        state: &mut State,
    ) -> std::result::Result<(), ActorProcessingErr> {
        match message {
            Message::Accept {
                envelope,
                trace,
                reply_port,
            } => {
                let reply = match state.accept(envelope, trace).await {
                    Ok(reply) => reply,
                    Err(_error) => PipelineReply::new(None, ActorTrace::new()),
                };
                let _ = reply_port.send(reply);
            }
        }
        Ok(())
    }
}
