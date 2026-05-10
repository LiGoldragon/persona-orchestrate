use kameo::actor::{Actor, ActorRef};
use kameo::error::Infallible;
use kameo::message::{Context, Message};
use signal_persona_mind::MindReply;

use super::pipeline::PipelineReply;
use super::trace::{ActorKind, ActorTrace, TraceAction};

pub(super) struct ReplySupervisorActor {
    shaped_reply_count: u64,
}

#[derive(Clone)]
pub(super) struct Arguments {
    pub shaped_reply_count: u64,
}

impl Default for Arguments {
    fn default() -> Self {
        Self {
            shaped_reply_count: 0,
        }
    }
}

pub struct ShapeReply {
    pub reply: Option<MindReply>,
    pub trace: ActorTrace,
}

impl ReplySupervisorActor {
    fn new(arguments: Arguments) -> Self {
        Self {
            shaped_reply_count: arguments.shaped_reply_count,
        }
    }

    fn shape(&mut self, reply: Option<MindReply>, mut trace: ActorTrace) -> PipelineReply {
        self.shaped_reply_count += 1;
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

impl Actor for ReplySupervisorActor {
    type Args = Arguments;
    type Error = Infallible;

    async fn on_start(
        arguments: Self::Args,
        _actor_reference: ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        Ok(Self::new(arguments))
    }
}

impl Message<ShapeReply> for ReplySupervisorActor {
    type Reply = PipelineReply;

    async fn handle(
        &mut self,
        message: ShapeReply,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.shape(message.reply, message.trace)
    }
}
