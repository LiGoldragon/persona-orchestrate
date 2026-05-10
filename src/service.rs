use kameo::actor::ActorRef;
use signal_persona_mind::MindReply;

use crate::actors::root::{
    Arguments as RootArguments, MindRoot, ReadManifest, RootReply, SubmitEnvelope,
};
use crate::actors::{ActorManifest, ActorTrace};
use crate::{MindEnvelope, Result, StoreLocation};

pub struct MindRuntime {
    root: ActorRef<MindRoot>,
}

#[derive(Debug)]
pub struct MindRuntimeReply {
    reply: Option<MindReply>,
    trace: ActorTrace,
}

impl MindRuntime {
    pub async fn start(store: StoreLocation) -> Result<Self> {
        let root = MindRoot::start(RootArguments::new(store)).await?;
        Ok(Self { root })
    }

    pub async fn submit(&self, envelope: MindEnvelope) -> Result<MindRuntimeReply> {
        let reply = self
            .root
            .ask(SubmitEnvelope { envelope })
            .await
            .map_err(|error| crate::Error::ActorCall(error.to_string()))?;
        Ok(MindRuntimeReply::from_root_reply(reply))
    }

    pub async fn manifest(&self) -> Result<ActorManifest> {
        self.root
            .ask(ReadManifest::expecting_at_least(1))
            .await
            .map_err(|error| crate::Error::ActorCall(error.to_string()))
    }

    pub async fn stop(self) -> Result<()> {
        MindRoot::stop(self.root).await
    }
}

impl MindRuntimeReply {
    fn from_root_reply(reply: RootReply) -> Self {
        Self {
            reply: reply.reply().cloned(),
            trace: reply.trace().clone(),
        }
    }

    pub fn reply(&self) -> Option<&MindReply> {
        self.reply.as_ref()
    }

    pub fn trace(&self) -> &ActorTrace {
        &self.trace
    }
}
