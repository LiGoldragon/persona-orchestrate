use kameo::actor::{Actor, ActorRef};
use kameo::message::{Context, Message};

use crate::MindEnvelope;

use super::kernel::{ApplyActivity, KernelReply, ReadActivity, StoreKernel};
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

pub(super) struct Read {
    pub(super) envelope: MindEnvelope,
    pub(super) trace: ActorTrace,
}

pub(super) struct ActivityStore {
    kernel: ActorRef<StoreKernel>,
}

impl ActivityStore {
    fn new(arguments: Arguments) -> Self {
        Self {
            kernel: arguments.kernel,
        }
    }

    async fn apply(&self, envelope: MindEnvelope, mut trace: ActorTrace) -> PipelineReply {
        trace.record(ActorKind::ActivityStore, TraceAction::MessageReceived);
        trace.record(ActorKind::Clock, TraceAction::MessageReceived);
        trace.record(ActorKind::SemaWriter, TraceAction::WriteIntentSent);

        let reply = self
            .kernel
            .ask(ApplyActivity::new(envelope))
            .await
            .map_err(|error| crate::Error::ActorCall(error.to_string()))
            .map(KernelReply::into_reply)
            .unwrap_or_else(|error| Some(PersistenceRejection::reply(error)));

        trace.record(ActorKind::ActivityAppender, TraceAction::MessageReceived);
        trace.record(ActorKind::Commit, TraceAction::CommitCompleted);
        PipelineReply::new(reply, trace)
    }

    async fn read(&self, envelope: MindEnvelope, mut trace: ActorTrace) -> PipelineReply {
        trace.record(ActorKind::ActivityStore, TraceAction::MessageReceived);
        trace.record(ActorKind::SemaReader, TraceAction::MessageReceived);
        trace.record(ActorKind::RecentActivityView, TraceAction::MessageReceived);

        let reply = self
            .kernel
            .ask(ReadActivity::new(envelope))
            .await
            .map_err(|error| crate::Error::ActorCall(error.to_string()))
            .map(KernelReply::into_reply)
            .unwrap_or_else(|error| Some(PersistenceRejection::reply(error)));

        PipelineReply::new(reply, trace)
    }
}

impl Actor for ActivityStore {
    type Args = Arguments;
    type Error = std::convert::Infallible;

    async fn on_start(
        arguments: Self::Args,
        _actor_reference: ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        Ok(Self::new(arguments))
    }
}

impl Message<Apply> for ActivityStore {
    type Reply = PipelineReply;

    async fn handle(
        &mut self,
        message: Apply,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.apply(message.envelope, message.trace).await
    }
}

impl Message<Read> for ActivityStore {
    type Reply = PipelineReply;

    async fn handle(
        &mut self,
        message: Read,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.read(message.envelope, message.trace).await
    }
}
