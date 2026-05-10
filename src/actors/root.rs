use kameo::actor::{Actor, ActorRef, Spawn};
use kameo::error::Infallible;
use kameo::message::{Context, Message};
use signal_persona_mind::MindReply;

use crate::{Error, MindEnvelope, Result, StoreLocation};

use super::manifest::ActorManifest;
use super::trace::{ActorKind, ActorTrace, TraceAction};
use super::{config, dispatch, domain, ingress, reply, store, subscription, view};

struct MindRootActor {
    ingress: ActorRef<ingress::IngressSupervisorActor>,
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

struct ReadManifest;

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

impl MindRootActor {
    fn new(ingress: ActorRef<ingress::IngressSupervisorActor>, manifest: ActorManifest) -> Self {
        Self { ingress, manifest }
    }

    async fn submit(&self, envelope: MindEnvelope) -> Result<RootReply> {
        let mut trace = ActorTrace::new();
        trace.record(ActorKind::MindRootActor, TraceAction::MessageReceived);

        let mut pipeline = self
            .ingress
            .ask(ingress::AcceptEnvelope { envelope, trace })
            .await
            .map_err(|error| Error::ActorCall(error.to_string()))?;
        pipeline
            .trace
            .record(ActorKind::MindRootActor, TraceAction::MessageReplied);

        Ok(RootReply::new(pipeline.reply, pipeline.trace))
    }
}

pub struct MindRootHandle {
    actor_reference: ActorRef<MindRootActor>,
}

impl MindRootHandle {
    pub async fn start(arguments: Arguments) -> Result<Self> {
        let actor_reference = MindRootActor::spawn(arguments);
        actor_reference.wait_for_startup().await;

        Ok(Self { actor_reference })
    }

    pub async fn submit(&self, envelope: MindEnvelope) -> Result<RootReply> {
        self.actor_reference
            .ask(SubmitEnvelope { envelope })
            .await
            .map_err(|error| Error::ActorCall(error.to_string()))
    }

    pub async fn manifest(&self) -> Result<ActorManifest> {
        self.actor_reference
            .ask(ReadManifest)
            .await
            .map_err(|error| Error::ActorCall(error.to_string()))
    }

    pub async fn stop(self) -> Result<()> {
        self.actor_reference
            .stop_gracefully()
            .await
            .map_err(|error| Error::ActorCall(error.to_string()))?;
        self.actor_reference.wait_for_shutdown().await;
        Ok(())
    }
}

impl Actor for MindRootActor {
    type Args = Arguments;
    type Error = Infallible;

    async fn on_start(
        arguments: Self::Args,
        actor_reference: ActorRef<Self>,
    ) -> std::result::Result<Self, Self::Error> {
        let manifest = ActorManifest::persona_mind_phase_one();

        let _config = config::ConfigActor::supervise(
            &actor_reference,
            config::Arguments {
                store: arguments.store.clone(),
            },
        )
        .spawn()
        .await;

        let store = store::StoreSupervisorActor::supervise(
            &actor_reference,
            store::Arguments {
                store: arguments.store.clone(),
            },
        )
        .spawn()
        .await;

        let _subscription = subscription::SubscriptionSupervisorActor::supervise(
            &actor_reference,
            subscription::Arguments::default(),
        )
        .spawn()
        .await;

        let reply =
            reply::ReplySupervisorActor::supervise(&actor_reference, reply::Arguments::default())
                .spawn()
                .await;

        let view = view::ViewSupervisorActor::supervise(
            &actor_reference,
            view::Arguments {
                store: store.clone(),
            },
        )
        .spawn()
        .await;

        let domain = domain::DomainSupervisorActor::supervise(
            &actor_reference,
            domain::Arguments {
                store: store.clone(),
            },
        )
        .spawn()
        .await;

        let dispatch = dispatch::DispatchSupervisorActor::supervise(
            &actor_reference,
            dispatch::Arguments {
                domain,
                view,
                reply,
            },
        )
        .spawn()
        .await;

        let ingress = ingress::IngressSupervisorActor::supervise(
            &actor_reference,
            ingress::Arguments { dispatch },
        )
        .spawn()
        .await;

        Ok(Self::new(ingress, manifest))
    }
}

impl Message<SubmitEnvelope> for MindRootActor {
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
                trace.record(ActorKind::MindRootActor, TraceAction::MessageReceived);
                trace.record(ActorKind::ErrorShapeActor, TraceAction::MessageReplied);
                RootReply::new(None, trace)
            }
        }
    }
}

impl Message<ReadManifest> for MindRootActor {
    type Reply = ActorManifest;

    async fn handle(
        &mut self,
        _message: ReadManifest,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.manifest.clone()
    }
}
