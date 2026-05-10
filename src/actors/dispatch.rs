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

pub(super) struct DispatchSupervisor {
    domain: ActorRef<domain::DomainSupervisor>,
    view: ActorRef<view::ViewSupervisor>,
    reply: ActorRef<reply::ReplySupervisor>,
}

#[derive(Clone)]
pub(super) struct Arguments {
    pub(super) domain: ActorRef<domain::DomainSupervisor>,
    pub(super) view: ActorRef<view::ViewSupervisor>,
    pub(super) reply: ActorRef<reply::ReplySupervisor>,
}

pub struct RouteEnvelope {
    pub envelope: MindEnvelope,
    pub trace: ActorTrace,
}

impl DispatchSupervisor {
    fn new(
        domain: ActorRef<domain::DomainSupervisor>,
        view: ActorRef<view::ViewSupervisor>,
        reply: ActorRef<reply::ReplySupervisor>,
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
        trace.record(ActorKind::DispatchSupervisor, TraceAction::MessageReceived);
        trace.record(ActorKind::RequestDispatcher, TraceAction::MessageReceived);

        let pipeline = match envelope.request() {
            MindRequest::Open(_)
            | MindRequest::AddNote(_)
            | MindRequest::Link(_)
            | MindRequest::ChangeStatus(_)
            | MindRequest::AddAlias(_) => {
                trace.record(ActorKind::MemoryFlow, TraceAction::MessageReceived);
                self.apply_memory(envelope, trace).await?
            }
            MindRequest::Query(_) => {
                trace.record(ActorKind::QueryFlow, TraceAction::MessageReceived);
                self.read_memory(envelope, trace).await?
            }
            MindRequest::RoleClaim(_) => self.unsupported(envelope, trace, ActorKind::ClaimFlow),
            MindRequest::RoleHandoff(_) => {
                self.unsupported(envelope, trace, ActorKind::HandoffFlow)
            }
            MindRequest::ActivitySubmission(_) | MindRequest::ActivityQuery(_) => {
                self.unsupported(envelope, trace, ActorKind::ActivityFlow)
            }
            MindRequest::RoleRelease(_) | MindRequest::RoleObservation(_) => {
                self.unsupported(envelope, trace, ActorKind::ClaimFlow)
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
        trace.record(ActorKind::ErrorShaper, TraceAction::MessageReplied);
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

impl Actor for DispatchSupervisor {
    type Args = Arguments;
    type Error = Infallible;

    async fn on_start(
        arguments: Self::Args,
        _actor_reference: ActorRef<Self>,
    ) -> std::result::Result<Self, Self::Error> {
        Ok(Self::new(arguments.domain, arguments.view, arguments.reply))
    }
}

impl Message<RouteEnvelope> for DispatchSupervisor {
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
