use kameo::actor::{Actor, ActorRef, Spawn};
use kameo::error::Infallible;
use kameo::message::{Context, Message};
use signal_persona_mind::MindReply;

use crate::{Error, MindEnvelope, Result, StoreLocation};

use super::trace::{ActorTrace, TraceAction, TraceNode};
use super::{config, dispatch, domain, ingress, reply, store, subscription, view};

pub struct MindRoot {
    ingress: ActorRef<ingress::IngressPhase>,
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
    fn new(ingress: ActorRef<ingress::IngressPhase>) -> Self {
        Self { ingress }
    }

    pub async fn start(arguments: Arguments) -> Result<ActorRef<Self>> {
        let actor_reference = Self::spawn(arguments);
        actor_reference.wait_for_startup().await;
        Ok(actor_reference)
    }

    pub async fn stop(actor_reference: ActorRef<Self>) -> Result<()> {
        actor_reference
            .stop_gracefully()
            .await
            .map_err(|error| Error::ActorCall(error.to_string()))?;
        actor_reference.wait_for_shutdown().await;
        Ok(())
    }

    async fn submit(&self, envelope: MindEnvelope) -> Result<RootReply> {
        let mut trace = ActorTrace::new();
        trace.record(TraceNode::MIND_ROOT, TraceAction::MessageReceived);

        let mut pipeline = self
            .ingress
            .ask(ingress::AcceptEnvelope { envelope, trace })
            .await
            .map_err(|error| Error::ActorCall(error.to_string()))?;
        pipeline
            .trace
            .record(TraceNode::MIND_ROOT, TraceAction::MessageReplied);

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

        let view = view::ViewPhase::supervise(
            &actor_reference,
            view::Arguments {
                store: store.clone(),
            },
        )
        .spawn()
        .await;

        let domain = domain::DomainPhase::supervise(
            &actor_reference,
            domain::Arguments {
                store: store.clone(),
            },
        )
        .spawn()
        .await;

        let dispatch = dispatch::DispatchPhase::supervise(
            &actor_reference,
            dispatch::Arguments {
                domain,
                view,
                reply,
            },
        )
        .spawn()
        .await;

        let ingress =
            ingress::IngressPhase::supervise(&actor_reference, ingress::Arguments { dispatch })
                .spawn()
                .await;

        Ok(Self::new(ingress))
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
                trace.record(TraceNode::MIND_ROOT, TraceAction::MessageReceived);
                trace.record(TraceNode::ERROR_SHAPER, TraceAction::MessageReplied);
                RootReply::new(None, trace)
            }
        }
    }
}
