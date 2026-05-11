use kameo::actor::{Actor, ActorRef};
use kameo::error::Infallible;
use kameo::message::{Context, Message};

use crate::{MindEnvelope, Result as CrateResult};

use super::dispatch;
use super::pipeline::PipelineReply;
use super::trace::{ActorTrace, TraceAction, TraceNode};

pub(super) struct IngressPhase {
    dispatch: ActorRef<dispatch::DispatchPhase>,
}

#[derive(Clone)]
pub(super) struct Arguments {
    pub(super) dispatch: ActorRef<dispatch::DispatchPhase>,
}

pub struct AcceptEnvelope {
    pub envelope: MindEnvelope,
    pub trace: ActorTrace,
}

impl IngressPhase {
    fn new(dispatch: ActorRef<dispatch::DispatchPhase>) -> Self {
        Self { dispatch }
    }

    async fn accept(
        &self,
        envelope: MindEnvelope,
        mut trace: ActorTrace,
    ) -> CrateResult<PipelineReply> {
        trace.record(TraceNode::INGRESS_PHASE, TraceAction::MessageReceived);
        trace.record(TraceNode::REQUEST_SESSION, TraceAction::MessageReceived);
        trace.record(TraceNode::NOTA_DECODER, TraceAction::MessageReceived);
        trace.record(
            TraceNode::CALLER_IDENTITY_RESOLVER,
            TraceAction::MessageReceived,
        );
        trace.record(TraceNode::ENVELOPE_BUILDER, TraceAction::MessageReplied);

        self.dispatch
            .ask(dispatch::RouteEnvelope { envelope, trace })
            .await
            .map_err(|error| crate::Error::ActorCall(error.to_string()))
    }
}

impl Actor for IngressPhase {
    type Args = Arguments;
    type Error = Infallible;

    async fn on_start(
        arguments: Self::Args,
        _actor_reference: ActorRef<Self>,
    ) -> std::result::Result<Self, Self::Error> {
        Ok(Self::new(arguments.dispatch))
    }
}

impl Message<AcceptEnvelope> for IngressPhase {
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
