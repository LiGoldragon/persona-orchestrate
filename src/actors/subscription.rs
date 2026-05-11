use kameo::actor::{Actor, ActorRef};
use kameo::error::Infallible;
use kameo::message::{Context, Message};

use super::trace::{ActorTrace, TraceAction, TraceNode};

pub(super) struct SubscriptionSupervisor {
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

impl SubscriptionSupervisor {
    fn new(arguments: Arguments) -> Self {
        Self {
            post_commit_count: arguments.post_commit_count,
        }
    }
}

impl Actor for SubscriptionSupervisor {
    type Args = Arguments;
    type Error = Infallible;

    async fn on_start(
        arguments: Self::Args,
        _actor_reference: ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        Ok(Self::new(arguments))
    }
}

impl Message<PublishPostCommit> for SubscriptionSupervisor {
    type Reply = ActorTrace;

    async fn handle(
        &mut self,
        message: PublishPostCommit,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.post_commit_count += 1;
        let mut trace = message.trace;
        trace.record(
            TraceNode::SUBSCRIPTION_SUPERVISOR,
            TraceAction::MessageReceived,
        );
        trace.record(TraceNode::COMMIT_BUS, TraceAction::MessageReceived);
        trace
    }
}
