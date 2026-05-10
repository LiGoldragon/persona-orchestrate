use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort};
use signal_persona_mind::MindReply;

use super::pipeline::PipelineReply;
use super::trace::{ActorKind, ActorTrace, TraceAction};

pub struct ReplySupervisor;

pub struct State;

pub struct Arguments;

pub enum Message {
    Shape {
        reply: Option<MindReply>,
        trace: ActorTrace,
        reply_port: RpcReplyPort<PipelineReply>,
    },
}

impl State {
    fn shape(&self, reply: Option<MindReply>, mut trace: ActorTrace) -> PipelineReply {
        trace.record(
            ActorKind::ReplySupervisorActor,
            TraceAction::MessageReceived,
        );
        match reply {
            Some(reply) => {
                trace.record(ActorKind::NotaReplyEncodeActor, TraceAction::MessageReplied);
                PipelineReply::new(Some(reply), trace)
            }
            None => {
                trace.record(ActorKind::ErrorShapeActor, TraceAction::MessageReplied);
                PipelineReply::new(None, trace)
            }
        }
    }
}

#[ractor::async_trait]
impl Actor for ReplySupervisor {
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
        state: &mut State,
    ) -> std::result::Result<(), ActorProcessingErr> {
        match message {
            Message::Shape {
                reply,
                trace,
                reply_port,
            } => {
                let _ = reply_port.send(state.shape(reply, trace));
            }
        }
        Ok(())
    }
}
