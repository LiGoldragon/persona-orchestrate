use kameo::actor::{Actor, ActorRef, Spawn};
use kameo::error::Infallible;
use kameo::message::{Context, Message};
use signal_persona_mind::MindReply;

use crate::{Error, MindEnvelope, Result, StoreLocation};

use super::manifest::ActorManifest;
use super::trace::{ActorKind, ActorTrace, TraceAction};
use super::{config, dispatch, domain, ingress, reply, store, subscription, view};

pub(crate) struct MindRoot {
    ingress: ActorRef<ingress::IngressSupervisor>,
    manifest: ActorManifest,
}

pub struct Arguments {
    pub store: StoreLocation,
}

impl Arguments {
    pub fn new(store: StoreLocation) -> Self {
        Self { store }
    }
}

pub struct SubmitEnvelope {
    pub envelope: MindEnvelope,
}

pub(crate) struct ReadManifest {
    probe: ManifestProbe,
}

impl ReadManifest {
    pub(crate) fn expecting_at_least(minimum_actors: usize) -> Self {
        Self {
            probe: ManifestProbe { minimum_actors },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ManifestProbe {
    minimum_actors: usize,
}

#[derive(Debug, kameo::Reply)]
pub struct RootReply {
    reply: Option<MindReply>,
    trace: ActorTrace,
}

impl RootReply {
    pub fn new(reply: Option<MindReply>, trace: ActorTrace) -> Self {
        Self { reply, trace }
    }

    pub fn reply(&self) -> Option<&MindReply> {
        self.reply.as_ref()
    }

    pub fn trace(&self) -> &ActorTrace {
        &self.trace
    }
}

impl MindRoot {
    fn new(ingress: ActorRef<ingress::IngressSupervisor>, manifest: ActorManifest) -> Self {
        Self { ingress, manifest }
    }

    pub(crate) async fn start(arguments: Arguments) -> Result<ActorRef<Self>> {
        let actor_reference = Self::spawn(arguments);
        actor_reference.wait_for_startup().await;
        Ok(actor_reference)
    }

    pub(crate) async fn stop(actor_reference: ActorRef<Self>) -> Result<()> {
        actor_reference
            .stop_gracefully()
            .await
            .map_err(|error| Error::ActorCall(error.to_string()))?;
        actor_reference.wait_for_shutdown().await;
        Ok(())
    }

    async fn submit(&self, envelope: MindEnvelope) -> Result<RootReply> {
        let mut trace = ActorTrace::new();
        trace.record(ActorKind::MindRoot, TraceAction::MessageReceived);

        let mut pipeline = self
            .ingress
            .ask(ingress::AcceptEnvelope { envelope, trace })
            .await
            .map_err(|error| Error::ActorCall(error.to_string()))?;
        pipeline
            .trace
            .record(ActorKind::MindRoot, TraceAction::MessageReplied);

        Ok(RootReply::new(pipeline.reply, pipeline.trace))
    }
}

impl Actor for MindRoot {
    type Args = Arguments;
    type Error = Infallible;

    async fn on_start(
        arguments: Self::Args,
        actor_reference: ActorRef<Self>,
    ) -> std::result::Result<Self, Self::Error> {
        let manifest = ActorManifest::persona_mind_phase_one();

        let _config = config::Config::supervise(
            &actor_reference,
            config::Arguments {
                store: arguments.store.clone(),
            },
        )
        .spawn()
        .await;

        let store = store::StoreSupervisor::supervise(
            &actor_reference,
            store::Arguments {
                store: arguments.store.clone(),
            },
        )
        .spawn()
        .await;

        let _subscription = subscription::SubscriptionSupervisor::supervise(
            &actor_reference,
            subscription::Arguments::default(),
        )
        .spawn()
        .await;

        let reply =
            reply::ReplySupervisor::supervise(&actor_reference, reply::Arguments::default())
                .spawn()
                .await;

        let view = view::ViewSupervisor::supervise(
            &actor_reference,
            view::Arguments {
                store: store.clone(),
            },
        )
        .spawn()
        .await;

        let domain = domain::DomainSupervisor::supervise(
            &actor_reference,
            domain::Arguments {
                store: store.clone(),
            },
        )
        .spawn()
        .await;

        let dispatch = dispatch::DispatchSupervisor::supervise(
            &actor_reference,
            dispatch::Arguments {
                domain,
                view,
                reply,
            },
        )
        .spawn()
        .await;

        let ingress = ingress::IngressSupervisor::supervise(
            &actor_reference,
            ingress::Arguments { dispatch },
        )
        .spawn()
        .await;

        Ok(Self::new(ingress, manifest))
    }
}

impl Message<SubmitEnvelope> for MindRoot {
    type Reply = RootReply;

    async fn handle(
        &mut self,
        message: SubmitEnvelope,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        match self.submit(message.envelope).await {
            Ok(reply) => reply,
            Err(_error) => {
                let mut trace = ActorTrace::new();
                trace.record(ActorKind::MindRoot, TraceAction::MessageReceived);
                trace.record(ActorKind::ErrorShaper, TraceAction::MessageReplied);
                RootReply::new(None, trace)
            }
        }
    }
}

impl Message<ReadManifest> for MindRoot {
    type Reply = ActorManifest;

    async fn handle(
        &mut self,
        message: ReadManifest,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let _satisfied = self.manifest.actors().len() >= message.probe.minimum_actors;
        self.manifest.clone()
    }
}
