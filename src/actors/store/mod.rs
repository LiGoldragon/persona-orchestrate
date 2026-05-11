mod activity;
mod claims;
mod kernel;
mod memory;
mod persistence;
mod write_trace;

use kameo::actor::{Actor, ActorRef, Spawn};
use kameo::message::{Context, Message};

use crate::{MindEnvelope, StoreLocation};

use super::pipeline::PipelineReply;
use super::trace::{ActorTrace, TraceAction, TraceNode};
use activity::ActivityStore;
use claims::ClaimStore;
use kernel::{LoadMemoryGraph, StoreKernel};
use memory::MemoryStore;
use persistence::PersistenceRejection;

#[derive(Clone)]
pub(super) struct Arguments {
    pub(super) store: StoreLocation,
}

pub(super) struct StoreSupervisor {
    memory: ActorRef<MemoryStore>,
    claims: ActorRef<ClaimStore>,
    activity: ActorRef<ActivityStore>,
}

pub struct ApplyMemory {
    pub envelope: MindEnvelope,
    pub trace: ActorTrace,
}

pub struct ReadMemory {
    pub envelope: MindEnvelope,
    pub trace: ActorTrace,
}

pub struct ApplyClaim {
    pub envelope: MindEnvelope,
    pub trace: ActorTrace,
}

pub struct ApplyHandoff {
    pub envelope: MindEnvelope,
    pub trace: ActorTrace,
}

pub struct ReadClaims {
    pub envelope: MindEnvelope,
    pub trace: ActorTrace,
}

pub struct ApplyActivity {
    pub envelope: MindEnvelope,
    pub trace: ActorTrace,
}

pub struct ReadActivity {
    pub envelope: MindEnvelope,
    pub trace: ActorTrace,
}

impl StoreSupervisor {
    fn new(
        memory: ActorRef<MemoryStore>,
        claims: ActorRef<ClaimStore>,
        activity: ActorRef<ActivityStore>,
    ) -> Self {
        Self {
            memory,
            claims,
            activity,
        }
    }

    async fn apply_memory(
        &self,
        envelope: MindEnvelope,
        mut trace: ActorTrace,
    ) -> crate::Result<PipelineReply> {
        trace.record(TraceNode::STORE_SUPERVISOR, TraceAction::MessageReceived);
        self.memory
            .ask(memory::Apply { envelope, trace })
            .await
            .map_err(|error| crate::Error::ActorCall(error.to_string()))
    }

    async fn read_memory(
        &self,
        envelope: MindEnvelope,
        mut trace: ActorTrace,
    ) -> crate::Result<PipelineReply> {
        trace.record(TraceNode::STORE_SUPERVISOR, TraceAction::MessageReceived);
        self.memory
            .ask(memory::Read { envelope, trace })
            .await
            .map_err(|error| crate::Error::ActorCall(error.to_string()))
    }

    async fn apply_claim(
        &self,
        envelope: MindEnvelope,
        mut trace: ActorTrace,
    ) -> crate::Result<PipelineReply> {
        trace.record(TraceNode::STORE_SUPERVISOR, TraceAction::MessageReceived);
        self.claims
            .ask(claims::Apply { envelope, trace })
            .await
            .map_err(|error| crate::Error::ActorCall(error.to_string()))
    }

    async fn apply_handoff(
        &self,
        envelope: MindEnvelope,
        mut trace: ActorTrace,
    ) -> crate::Result<PipelineReply> {
        trace.record(TraceNode::STORE_SUPERVISOR, TraceAction::MessageReceived);
        self.claims
            .ask(claims::ApplyHandoffRequest { envelope, trace })
            .await
            .map_err(|error| crate::Error::ActorCall(error.to_string()))
    }

    async fn read_claims(
        &self,
        envelope: MindEnvelope,
        mut trace: ActorTrace,
    ) -> crate::Result<PipelineReply> {
        trace.record(TraceNode::STORE_SUPERVISOR, TraceAction::MessageReceived);
        self.claims
            .ask(claims::Read { envelope, trace })
            .await
            .map_err(|error| crate::Error::ActorCall(error.to_string()))
    }

    async fn apply_activity(
        &self,
        envelope: MindEnvelope,
        mut trace: ActorTrace,
    ) -> crate::Result<PipelineReply> {
        trace.record(TraceNode::STORE_SUPERVISOR, TraceAction::MessageReceived);
        self.activity
            .ask(activity::Apply { envelope, trace })
            .await
            .map_err(|error| crate::Error::ActorCall(error.to_string()))
    }

    async fn read_activity(
        &self,
        envelope: MindEnvelope,
        mut trace: ActorTrace,
    ) -> crate::Result<PipelineReply> {
        trace.record(TraceNode::STORE_SUPERVISOR, TraceAction::MessageReceived);
        self.activity
            .ask(activity::Read { envelope, trace })
            .await
            .map_err(|error| crate::Error::ActorCall(error.to_string()))
    }
}

impl Actor for StoreSupervisor {
    type Args = Arguments;
    type Error = crate::Error;

    async fn on_start(
        arguments: Self::Args,
        actor_reference: ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        let kernel = StoreKernel::supervise(&actor_reference, arguments.store.clone())
            .spawn()
            .await;
        let graph = kernel
            .ask(LoadMemoryGraph)
            .await
            .map_err(|error| crate::Error::ActorCall(error.to_string()))?;
        let memory = MemoryStore::supervise(
            &actor_reference,
            memory::Arguments {
                store: arguments.store,
                graph,
                kernel: kernel.clone(),
            },
        )
        .spawn()
        .await;
        let claims = ClaimStore::supervise(
            &actor_reference,
            claims::Arguments {
                kernel: kernel.clone(),
            },
        )
        .spawn()
        .await;
        let activity = ActivityStore::supervise(&actor_reference, activity::Arguments { kernel })
            .spawn()
            .await;

        Ok(Self::new(memory, claims, activity))
    }
}

impl Message<ApplyMemory> for StoreSupervisor {
    type Reply = PipelineReply;

    async fn handle(
        &mut self,
        message: ApplyMemory,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.apply_memory(message.envelope, message.trace)
            .await
            .unwrap_or_else(PersistenceRejection::pipeline)
    }
}

impl Message<ReadMemory> for StoreSupervisor {
    type Reply = PipelineReply;

    async fn handle(
        &mut self,
        message: ReadMemory,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.read_memory(message.envelope, message.trace)
            .await
            .unwrap_or_else(PersistenceRejection::pipeline)
    }
}

impl Message<ApplyClaim> for StoreSupervisor {
    type Reply = PipelineReply;

    async fn handle(
        &mut self,
        message: ApplyClaim,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.apply_claim(message.envelope, message.trace)
            .await
            .unwrap_or_else(PersistenceRejection::pipeline)
    }
}

impl Message<ApplyHandoff> for StoreSupervisor {
    type Reply = PipelineReply;

    async fn handle(
        &mut self,
        message: ApplyHandoff,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.apply_handoff(message.envelope, message.trace)
            .await
            .unwrap_or_else(PersistenceRejection::pipeline)
    }
}

impl Message<ReadClaims> for StoreSupervisor {
    type Reply = PipelineReply;

    async fn handle(
        &mut self,
        message: ReadClaims,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.read_claims(message.envelope, message.trace)
            .await
            .unwrap_or_else(PersistenceRejection::pipeline)
    }
}

impl Message<ApplyActivity> for StoreSupervisor {
    type Reply = PipelineReply;

    async fn handle(
        &mut self,
        message: ApplyActivity,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.apply_activity(message.envelope, message.trace)
            .await
            .unwrap_or_else(PersistenceRejection::pipeline)
    }
}

impl Message<ReadActivity> for StoreSupervisor {
    type Reply = PipelineReply;

    async fn handle(
        &mut self,
        message: ReadActivity,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.read_activity(message.envelope, message.trace)
            .await
            .unwrap_or_else(PersistenceRejection::pipeline)
    }
}
