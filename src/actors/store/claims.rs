use kameo::actor::{Actor, ActorRef};
use kameo::message::{Context, Message};

use crate::MindEnvelope;

use super::kernel::{ApplyClaim, ApplyHandoff, KernelReply, ReadClaims, StoreKernel};
use super::persistence::PersistenceRejection;
use super::{ActorKind, ActorTrace, PipelineReply, TraceAction};

#[derive(Clone)]
pub(super) struct Arguments {
    pub(super) kernel: ActorRef<StoreKernel>,
}

pub(super) struct Apply {
    pub(super) envelope: MindEnvelope,
    pub(super) trace: ActorTrace,
}

pub(super) struct ApplyHandoffRequest {
    pub(super) envelope: MindEnvelope,
    pub(super) trace: ActorTrace,
}

pub(super) struct Read {
    pub(super) envelope: MindEnvelope,
    pub(super) trace: ActorTrace,
}

pub(super) struct ClaimStore {
    kernel: ActorRef<StoreKernel>,
}

impl ClaimStore {
    fn new(arguments: Arguments) -> Self {
        Self {
            kernel: arguments.kernel,
        }
    }

    async fn apply_claim(&self, envelope: MindEnvelope, mut trace: ActorTrace) -> PipelineReply {
        trace.record(ActorKind::ClaimStore, TraceAction::MessageReceived);
        trace.record(ActorKind::SemaReader, TraceAction::MessageReceived);
        trace.record(ActorKind::SemaWriter, TraceAction::WriteIntentSent);

        let reply = self
            .kernel
            .ask(ApplyClaim::new(envelope))
            .await
            .map_err(|error| crate::Error::ActorCall(error.to_string()))
            .map(KernelReply::into_reply)
            .unwrap_or_else(|error| Some(PersistenceRejection::reply(error)));

        trace.record(ActorKind::EventAppender, TraceAction::MessageReceived);
        trace.record(ActorKind::Commit, TraceAction::CommitCompleted);
        PipelineReply::new(reply, trace)
    }

    async fn apply_handoff(&self, envelope: MindEnvelope, mut trace: ActorTrace) -> PipelineReply {
        trace.record(ActorKind::ClaimStore, TraceAction::MessageReceived);
        trace.record(ActorKind::SemaReader, TraceAction::MessageReceived);
        trace.record(ActorKind::SemaWriter, TraceAction::WriteIntentSent);

        let reply = self
            .kernel
            .ask(ApplyHandoff::new(envelope))
            .await
            .map_err(|error| crate::Error::ActorCall(error.to_string()))
            .map(KernelReply::into_reply)
            .unwrap_or_else(|error| Some(PersistenceRejection::reply(error)));

        trace.record(ActorKind::EventAppender, TraceAction::MessageReceived);
        trace.record(ActorKind::Commit, TraceAction::CommitCompleted);
        PipelineReply::new(reply, trace)
    }

    async fn read_claims(&self, envelope: MindEnvelope, mut trace: ActorTrace) -> PipelineReply {
        trace.record(ActorKind::ClaimStore, TraceAction::MessageReceived);
        trace.record(ActorKind::SemaReader, TraceAction::MessageReceived);

        let reply = self
            .kernel
            .ask(ReadClaims::new(envelope))
            .await
            .map_err(|error| crate::Error::ActorCall(error.to_string()))
            .map(KernelReply::into_reply)
            .unwrap_or_else(|error| Some(PersistenceRejection::reply(error)));

        PipelineReply::new(reply, trace)
    }
}

impl Actor for ClaimStore {
    type Args = Arguments;
    type Error = std::convert::Infallible;

    async fn on_start(
        arguments: Self::Args,
        _actor_reference: ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        Ok(Self::new(arguments))
    }
}

impl Message<Apply> for ClaimStore {
    type Reply = PipelineReply;

    async fn handle(
        &mut self,
        message: Apply,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.apply_claim(message.envelope, message.trace).await
    }
}

impl Message<ApplyHandoffRequest> for ClaimStore {
    type Reply = PipelineReply;

    async fn handle(
        &mut self,
        message: ApplyHandoffRequest,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.apply_handoff(message.envelope, message.trace).await
    }
}

impl Message<Read> for ClaimStore {
    type Reply = PipelineReply;

    async fn handle(
        &mut self,
        message: Read,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.read_claims(message.envelope, message.trace).await
    }
}
