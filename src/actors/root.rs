use std::sync::atomic::{AtomicU64, Ordering};

use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort};
use signal_persona_mind::MindReply;

use crate::error::ActorReply;
use crate::{Error, MindEnvelope, Result, StoreLocation};

use super::manifest::ActorManifest;
use super::trace::{ActorKind, ActorTrace, TraceAction};
use super::{config, dispatch, domain, ingress, reply, store, subscription, view};

struct MindRoot;

pub struct State {
    ingress: ActorRef<ingress::Message>,
    manifest: ActorManifest,
}

pub struct Arguments {
    pub store: StoreLocation,
    actor_prefix: String,
}

impl Arguments {
    pub fn new(store: StoreLocation) -> Self {
        static NEXT_ACTOR_PREFIX: AtomicU64 = AtomicU64::new(0);
        let value = NEXT_ACTOR_PREFIX.fetch_add(1, Ordering::Relaxed);
        Self {
            store,
            actor_prefix: format!("mind-{value}"),
        }
    }

    fn root_name(&self) -> String {
        format!("{}-root", self.actor_prefix)
    }

    fn child_name(&self, child: &str) -> String {
        format!("{}-{child}", self.actor_prefix)
    }
}

pub enum Message {
    Submit {
        envelope: MindEnvelope,
        reply_port: RpcReplyPort<RootReply>,
    },
    Manifest {
        reply_port: RpcReplyPort<ActorManifest>,
    },
}

#[derive(Debug)]
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

impl State {
    pub fn new(ingress: ActorRef<ingress::Message>, manifest: ActorManifest) -> Self {
        Self { ingress, manifest }
    }

    async fn submit(&self, envelope: MindEnvelope) -> Result<RootReply> {
        let mut trace = ActorTrace::new();
        trace.record(ActorKind::MindRootActor, TraceAction::MessageReceived);

        let raw = self
            .ingress
            .call(
                |reply_port| ingress::Message::Accept {
                    envelope,
                    trace,
                    reply_port,
                },
                None,
            )
            .await
            .map_err(|error| Error::ActorCall(error.to_string()))?;
        let mut pipeline = ActorReply::new(raw, "ingress accept").into_result()?;
        pipeline
            .trace
            .record(ActorKind::MindRootActor, TraceAction::MessageReplied);

        Ok(RootReply::new(pipeline.reply, pipeline.trace))
    }
}

pub struct MindRootHandle {
    actor_ref: ActorRef<Message>,
    join_handle: tokio::task::JoinHandle<()>,
}

impl MindRootHandle {
    pub async fn start(arguments: Arguments) -> Result<Self> {
        let root_name = arguments.root_name();
        let (actor_ref, join_handle) = Actor::spawn(Some(root_name), MindRoot, arguments)
            .await
            .map_err(|error| Error::ActorSpawn(error.to_string()))?;

        Ok(Self {
            actor_ref,
            join_handle,
        })
    }

    pub async fn submit(&self, envelope: MindEnvelope) -> Result<RootReply> {
        let raw = self
            .actor_ref
            .call(
                |reply_port| Message::Submit {
                    envelope,
                    reply_port,
                },
                None,
            )
            .await
            .map_err(|error| Error::ActorCall(error.to_string()))?;
        ActorReply::new(raw, "mind root submit").into_result()
    }

    pub async fn manifest(&self) -> Result<ActorManifest> {
        let raw = self
            .actor_ref
            .call(|reply_port| Message::Manifest { reply_port }, None)
            .await
            .map_err(|error| Error::ActorCall(error.to_string()))?;
        ActorReply::new(raw, "mind root manifest").into_result()
    }

    pub async fn stop(self) -> Result<()> {
        self.actor_ref.stop(None);
        self.join_handle
            .await
            .map_err(|error| Error::ActorJoin(error.to_string()))
    }
}

#[ractor::async_trait]
impl Actor for MindRoot {
    type Msg = Message;
    type State = State;
    type Arguments = Arguments;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        arguments: Arguments,
    ) -> std::result::Result<Self::State, ActorProcessingErr> {
        let manifest = ActorManifest::persona_mind_phase_one();

        let (_config, _) = Actor::spawn_linked(
            Some(arguments.child_name("config")),
            config::Config,
            config::Arguments {
                store: arguments.store.clone(),
            },
            myself.get_cell(),
        )
        .await?;

        let (store, _) = Actor::spawn_linked(
            Some(arguments.child_name("store-supervisor")),
            store::StoreSupervisor,
            store::Arguments {
                store: arguments.store.clone(),
            },
            myself.get_cell(),
        )
        .await?;

        let (_subscription, _) = Actor::spawn_linked(
            Some(arguments.child_name("subscription-supervisor")),
            subscription::SubscriptionSupervisor,
            subscription::Arguments,
            myself.get_cell(),
        )
        .await?;

        let (reply, _) = Actor::spawn_linked(
            Some(arguments.child_name("reply-supervisor")),
            reply::ReplySupervisor,
            reply::Arguments,
            myself.get_cell(),
        )
        .await?;

        let (view, _) = Actor::spawn_linked(
            Some(arguments.child_name("view-supervisor")),
            view::ViewSupervisor,
            view::Arguments {
                store: store.clone(),
            },
            myself.get_cell(),
        )
        .await?;

        let (domain, _) = Actor::spawn_linked(
            Some(arguments.child_name("domain-supervisor")),
            domain::DomainSupervisor,
            domain::Arguments { store },
            myself.get_cell(),
        )
        .await?;

        let (dispatch, _) = Actor::spawn_linked(
            Some(arguments.child_name("dispatch-supervisor")),
            dispatch::DispatchSupervisor,
            dispatch::Arguments {
                domain,
                view,
                reply,
            },
            myself.get_cell(),
        )
        .await?;

        let (ingress, _) = Actor::spawn_linked(
            Some(arguments.child_name("ingress-supervisor")),
            ingress::IngressSupervisor,
            ingress::Arguments { dispatch },
            myself.get_cell(),
        )
        .await?;

        Ok(State::new(ingress, manifest))
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Message,
        state: &mut State,
    ) -> std::result::Result<(), ActorProcessingErr> {
        match message {
            Message::Submit {
                envelope,
                reply_port,
            } => {
                let reply = match state.submit(envelope).await {
                    Ok(reply) => reply,
                    Err(_error) => {
                        let mut trace = ActorTrace::new();
                        trace.record(ActorKind::MindRootActor, TraceAction::MessageReceived);
                        trace.record(ActorKind::ErrorShapeActor, TraceAction::MessageReplied);
                        RootReply::new(None, trace)
                    }
                };
                let _ = reply_port.send(reply);
            }
            Message::Manifest { reply_port } => {
                let _ = reply_port.send(state.manifest.clone());
            }
        }
        Ok(())
    }
}
