use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort};

use super::trace::{ActorKind, ActorTrace, TraceAction};

pub struct SubscriptionSupervisor;

pub struct State;

pub struct Arguments;

#[allow(dead_code)]
pub enum Message {
    PostCommit {
        trace: ActorTrace,
        reply_port: RpcReplyPort<ActorTrace>,
    },
}

#[ractor::async_trait]
impl Actor for SubscriptionSupervisor {
    type Msg = Message;
    type State = State;
    type Arguments = Arguments;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        _arguments: Arguments,
    ) -> std::result::Result<Self::State, ActorProcessingErr> {
        Ok(State)
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Message,
        _state: &mut State,
    ) -> std::result::Result<(), ActorProcessingErr> {
        match message {
            Message::PostCommit {
                mut trace,
                reply_port,
            } => {
                trace.record(
                    ActorKind::SubscriptionSupervisorActor,
                    TraceAction::MessageReceived,
                );
                trace.record(ActorKind::CommitBusActor, TraceAction::MessageReceived);
                let _ = reply_port.send(trace);
            }
        }
        Ok(())
    }
}
