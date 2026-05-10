use signal_persona_mind::MindReply;

use super::trace::ActorTrace;

pub(crate) struct PipelineReply {
    pub(crate) reply: Option<MindReply>,
    pub(crate) trace: ActorTrace,
}

impl PipelineReply {
    pub(crate) fn new(reply: Option<MindReply>, trace: ActorTrace) -> Self {
        Self { reply, trace }
    }
}
