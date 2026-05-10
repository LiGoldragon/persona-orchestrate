use kameo::actor::{Actor, ActorRef};
use kameo::error::Infallible;
use kameo::message::{Context, Message};

use crate::{MindEnvelope, Result as CrateResult};

use super::dispatch;
use super::pipeline::PipelineReply;
use super::trace::{ActorKind, ActorTrace, TraceAction};

pub(super) struct IngressSupervisorActor {
    dispatch: ActorRef<dispatch::DispatchSupervisorActor>,
}

#[derive(Clone)]
pub(super) struct Arguments {
    pub(super) dispatch: ActorRef<dispatch::DispatchSupervisorActor>,
}

pub struct AcceptEnvelope {
    pub envelope: MindEnvelope,
    pub trace: ActorTrace,
}

impl IngressSupervisorActor {
    fn new(dispatch: ActorRef<dispatch::DispatchSupervisorActor>) -> Self {
        Self { dispatch }
    }

    async fn accept(
        &self,
        envelope: MindEnvelope,
        mut trace: ActorTrace,
    ) -> CrateResult<PipelineReply> {
        trace.record(
            ActorKind::IngressSupervisorActor,
            TraceAction::MessageReceived,
        );
        trace.record(ActorKind::RequestSessionActor, TraceAction::MessageReceived);
        trace.record(ActorKind::NotaDecodeActor, TraceAction::MessageReceived);
        trace.record(ActorKind::CallerIdentityActor, TraceAction::MessageReceived);
        trace.record(ActorKind::EnvelopeActor, TraceAction::MessageReplied);

        self.dispatch
            .ask(dispatch::RouteEnvelope { envelope, trace })
            .await
            .map_err(|error| crate::Error::ActorCall(error.to_string()))
    }
}

impl Actor for IngressSupervisorActor {
    type Args = Arguments;
    type Error = Infallible;

    async fn on_start(
        arguments: Self::Args,
        _actor_reference: ActorRef<Self>,
    ) -> std::result::Result<Self, Self::Error> {
        Ok(Self::new(arguments.dispatch))
    }
}

impl Message<AcceptEnvelope> for IngressSupervisorActor {
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
