use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort};
use signal_persona_mind::MindRequest;

use crate::error::ActorReply;
use crate::{MindEnvelope, Result};

use super::domain;
use super::pipeline::PipelineReply;
use super::reply;
use super::trace::{ActorKind, ActorTrace, TraceAction};
use super::view;

pub struct DispatchSupervisor;

pub struct State {
    domain: ActorRef<domain::Message>,
    view: ActorRef<view::Message>,
    reply: ActorRef<reply::Message>,
}

pub struct Arguments {
    pub domain: ActorRef<domain::Message>,
    pub view: ActorRef<view::Message>,
    pub reply: ActorRef<reply::Message>,
}

pub enum Message {
    Route {
        envelope: MindEnvelope,
        trace: ActorTrace,
        reply_port: RpcReplyPort<PipelineReply>,
    },
}

impl State {
    pub fn new(
        domain: ActorRef<domain::Message>,
        view: ActorRef<view::Message>,
        reply: ActorRef<reply::Message>,
    ) -> Self {
        Self {
            domain,
            view,
            reply,
        }
    }

    async fn route(&self, envelope: MindEnvelope, mut trace: ActorTrace) -> Result<PipelineReply> {
        trace.record(
            ActorKind::DispatchSupervisorActor,
            TraceAction::MessageReceived,
        );
        trace.record(
            ActorKind::RequestDispatchActor,
            TraceAction::MessageReceived,
        );

        let pipeline = match envelope.request() {
            MindRequest::Open(_)
            | MindRequest::AddNote(_)
            | MindRequest::Link(_)
            | MindRequest::ChangeStatus(_)
            | MindRequest::AddAlias(_) => {
                trace.record(ActorKind::MemoryFlowActor, TraceAction::MessageReceived);
                self.apply_memory(envelope, trace).await?
            }
            MindRequest::Query(_) => {
                trace.record(ActorKind::QueryFlowActor, TraceAction::MessageReceived);
                self.read_memory(envelope, trace).await?
            }
            MindRequest::RoleClaim(_) => {
                self.unsupported(envelope, trace, ActorKind::ClaimFlowActor)
            }
            MindRequest::RoleHandoff(_) => {
                self.unsupported(envelope, trace, ActorKind::HandoffFlowActor)
            }
            MindRequest::ActivitySubmission(_) | MindRequest::ActivityQuery(_) => {
                self.unsupported(envelope, trace, ActorKind::ActivityFlowActor)
            }
            MindRequest::RoleRelease(_) | MindRequest::RoleObservation(_) => {
                self.unsupported(envelope, trace, ActorKind::ClaimFlowActor)
            }
        };

        self.shape_reply(pipeline).await
    }

    async fn apply_memory(
        &self,
        envelope: MindEnvelope,
        trace: ActorTrace,
    ) -> Result<PipelineReply> {
        let raw = self
            .domain
            .call(
                |reply_port| domain::Message::ApplyMemory {
                    envelope,
                    trace,
                    reply_port,
                },
                None,
            )
            .await
            .map_err(|error| crate::Error::ActorCall(error.to_string()))?;
        ActorReply::new(raw, "domain apply memory").into_result()
    }

    async fn read_memory(
        &self,
        envelope: MindEnvelope,
        trace: ActorTrace,
    ) -> Result<PipelineReply> {
        let raw = self
            .view
            .call(
                |reply_port| view::Message::ReadMemory {
                    envelope,
                    trace,
                    reply_port,
                },
                None,
            )
            .await
            .map_err(|error| crate::Error::ActorCall(error.to_string()))?;
        ActorReply::new(raw, "view read memory").into_result()
    }

    fn unsupported(
        &self,
        _envelope: MindEnvelope,
        mut trace: ActorTrace,
        actor: ActorKind,
    ) -> PipelineReply {
        trace.record(actor, TraceAction::MessageReceived);
        trace.record(ActorKind::ErrorShapeActor, TraceAction::MessageReplied);
        PipelineReply::new(None, trace)
    }

    async fn shape_reply(&self, pipeline: PipelineReply) -> Result<PipelineReply> {
        let raw = self
            .reply
            .call(
                |reply_port| reply::Message::Shape {
                    reply: pipeline.reply,
                    trace: pipeline.trace,
                    reply_port,
                },
                None,
            )
            .await
            .map_err(|error| crate::Error::ActorCall(error.to_string()))?;
        ActorReply::new(raw, "reply shape").into_result()
    }
}

#[ractor::async_trait]
impl Actor for DispatchSupervisor {
    type Msg = Message;
    type State = State;
    type Arguments = Arguments;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        arguments: Arguments,
    ) -> std::result::Result<Self::State, ActorProcessingErr> {
        Ok(State::new(
            arguments.domain,
            arguments.view,
            arguments.reply,
        ))
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Message,
        state: &mut State,
    ) -> std::result::Result<(), ActorProcessingErr> {
        match message {
            Message::Route {
                envelope,
                trace,
                reply_port,
            } => {
                let reply = match state.route(envelope, trace).await {
                    Ok(reply) => reply,
                    Err(_error) => PipelineReply::new(None, ActorTrace::new()),
                };
                let _ = reply_port.send(reply);
            }
        }
        Ok(())
    }
}
