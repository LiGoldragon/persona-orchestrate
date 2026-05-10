use kameo::actor::{Actor, ActorRef};
use kameo::error::Infallible;
use kameo::message::{Context, Message};

use super::trace::{ActorKind, ActorTrace, TraceAction};

pub(super) struct SubscriptionSupervisorActor {
    post_commit_count: u64,
}

#[derive(Clone, Default)]
pub(super) struct Arguments {
    pub post_commit_count: u64,
}

#[allow(dead_code)]
pub struct PublishPostCommit {
    pub trace: ActorTrace,
}

impl SubscriptionSupervisorActor {
    fn new(arguments: Arguments) -> Self {
        Self {
            post_commit_count: arguments.post_commit_count,
        }
    }
}

impl Actor for SubscriptionSupervisorActor {
    type Args = Arguments;
    type Error = Infallible;

    async fn on_start(
        arguments: Self::Args,
        _actor_reference: ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        Ok(Self::new(arguments))
    }
}

impl Message<PublishPostCommit> for SubscriptionSupervisorActor {
    type Reply = ActorTrace;

    async fn handle(
        &mut self,
        message: PublishPostCommit,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.post_commit_count += 1;
        let mut trace = message.trace;
        trace.record(
            ActorKind::SubscriptionSupervisorActor,
            TraceAction::MessageReceived,
        );
        trace.record(ActorKind::CommitBusActor, TraceAction::MessageReceived);
        trace
    }
}
