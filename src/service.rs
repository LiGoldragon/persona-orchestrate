use signal_persona_mind::MindReply;

use crate::actors::root::{Arguments as RootArguments, RootReply};
use crate::actors::{ActorManifest, ActorTrace, MindRootHandle};
use crate::{MindEnvelope, Result, StoreLocation};

pub struct MindRuntime {
    root: MindRootHandle,
}

#[derive(Debug)]
pub struct MindRuntimeReply {
    reply: Option<MindReply>,
    trace: ActorTrace,
}

impl MindRuntime {
    pub async fn start(store: StoreLocation) -> Result<Self> {
        let root = MindRootHandle::start(RootArguments::new(store)).await?;
        Ok(Self { root })
    }

    pub async fn submit(&self, envelope: MindEnvelope) -> Result<MindRuntimeReply> {
        let reply = self.root.submit(envelope).await?;
        Ok(MindRuntimeReply::from_root_reply(reply))
    }

    pub async fn manifest(&self) -> Result<ActorManifest> {
        self.root.manifest().await
    }

    pub async fn stop(self) -> Result<()> {
        self.root.stop().await
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
