use kameo::actor::{Actor, ActorRef};
use kameo::error::Infallible;
use kameo::message::{Context, Message};
use signal_persona_mind::MindRequest;

use crate::{MindEnvelope, Result as CrateResult};

use super::domain;
use super::pipeline::PipelineReply;
use super::reply;
use super::trace::{ActorKind, ActorTrace, TraceAction};
use super::view;

pub(super) struct DispatchSupervisorActor {
    domain: ActorRef<domain::DomainSupervisorActor>,
    view: ActorRef<view::ViewSupervisorActor>,
    reply: ActorRef<reply::ReplySupervisorActor>,
}

#[derive(Clone)]
pub(super) struct Arguments {
    pub(super) domain: ActorRef<domain::DomainSupervisorActor>,
    pub(super) view: ActorRef<view::ViewSupervisorActor>,
    pub(super) reply: ActorRef<reply::ReplySupervisorActor>,
}

pub struct RouteEnvelope {
    pub envelope: MindEnvelope,
    pub trace: ActorTrace,
}

impl DispatchSupervisorActor {
    fn new(
        domain: ActorRef<domain::DomainSupervisorActor>,
        view: ActorRef<view::ViewSupervisorActor>,
        reply: ActorRef<reply::ReplySupervisorActor>,
    ) -> Self {
        Self {
            domain,
            view,
            reply,
        }
    }

    async fn route(
        &self,
        envelope: MindEnvelope,
        mut trace: ActorTrace,
    ) -> CrateResult<PipelineReply> {
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
    ) -> CrateResult<PipelineReply> {
        self.domain
            .ask(domain::ApplyMemory { envelope, trace })
            .await
            .map_err(|error| crate::Error::ActorCall(error.to_string()))
    }

    async fn read_memory(
        &self,
        envelope: MindEnvelope,
        trace: ActorTrace,
    ) -> CrateResult<PipelineReply> {
        self.view
            .ask(view::ReadMemory { envelope, trace })
            .await
            .map_err(|error| crate::Error::ActorCall(error.to_string()))
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

    async fn shape_reply(&self, pipeline: PipelineReply) -> CrateResult<PipelineReply> {
        self.reply
            .ask(reply::ShapeReply {
                reply: pipeline.reply,
                trace: pipeline.trace,
            })
            .await
            .map_err(|error| crate::Error::ActorCall(error.to_string()))
    }
}

impl Actor for DispatchSupervisorActor {
    type Args = Arguments;
    type Error = Infallible;

    async fn on_start(
        arguments: Self::Args,
        _actor_reference: ActorRef<Self>,
    ) -> std::result::Result<Self, Self::Error> {
        Ok(Self::new(arguments.domain, arguments.view, arguments.reply))
    }
}

impl Message<RouteEnvelope> for DispatchSupervisorActor {
    type Reply = PipelineReply;

    async fn handle(
        &mut self,
        message: RouteEnvelope,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        match self.route(message.envelope, message.trace).await {
            Ok(reply) => reply,
            Err(_error) => PipelineReply::new(None, ActorTrace::new()),
        }
    }
}
