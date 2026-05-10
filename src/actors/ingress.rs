use kameo::actor::{Actor, ActorRef};
use kameo::error::Infallible;
use kameo::message::{Context, Message};

use crate::{MindEnvelope, Result as CrateResult};

use super::dispatch;
use super::pipeline::PipelineReply;
use super::trace::{ActorKind, ActorTrace, TraceAction};

pub(super) struct IngressSupervisor {
    dispatch: ActorRef<dispatch::DispatchSupervisor>,
}

#[derive(Clone)]
pub(super) struct Arguments {
    pub(super) dispatch: ActorRef<dispatch::DispatchSupervisor>,
}

pub struct AcceptEnvelope {
    pub envelope: MindEnvelope,
    pub trace: ActorTrace,
}

impl IngressSupervisor {
    fn new(dispatch: ActorRef<dispatch::DispatchSupervisor>) -> Self {
        Self { dispatch }
    }

    async fn accept(
        &self,
        envelope: MindEnvelope,
        mut trace: ActorTrace,
    ) -> CrateResult<PipelineReply> {
        trace.record(ActorKind::IngressSupervisor, TraceAction::MessageReceived);
        trace.record(ActorKind::RequestSession, TraceAction::MessageReceived);
        trace.record(ActorKind::NotaDecoder, TraceAction::MessageReceived);
        trace.record(
            ActorKind::CallerIdentityResolver,
            TraceAction::MessageReceived,
        );
        trace.record(ActorKind::EnvelopeBuilder, TraceAction::MessageReplied);

        self.dispatch
            .ask(dispatch::RouteEnvelope { envelope, trace })
            .await
            .map_err(|error| crate::Error::ActorCall(error.to_string()))
    }
}

impl Actor for IngressSupervisor {
    type Args = Arguments;
    type Error = Infallible;

    async fn on_start(
        arguments: Self::Args,
        _actor_reference: ActorRef<Self>,
    ) -> std::result::Result<Self, Self::Error> {
        Ok(Self::new(arguments.dispatch))
    }
}

impl Message<AcceptEnvelope> for IngressSupervisor {
    type Reply = PipelineReply;

    async fn handle(
        &mut self,
        message: AcceptEnvelope,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        match self.accept(message.envelope, message.trace).await {
            Ok(reply) => reply,
            Err(_error) => PipelineReply::new(None, ActorTrace::new()),
        }
    }
}
