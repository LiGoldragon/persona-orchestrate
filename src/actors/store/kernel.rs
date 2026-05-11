use kameo::actor::{Actor, ActorRef};
use kameo::message::{Context, Message};
use signal_persona_mind::{MindReply, MindRequest};

use crate::{ActivityLedger, ClaimLedger, MemoryGraph, MindEnvelope, MindTables, StoreLocation};

use super::persistence::PersistenceRejection;

pub(super) struct StoreKernel {
    tables: MindTables,
}

pub(super) struct CommitMemoryGraph {
    graph: MemoryGraph,
}

pub(super) struct LoadMemoryGraph;

pub(super) struct ApplyClaim {
    envelope: MindEnvelope,
}

pub(super) struct ApplyHandoff {
    envelope: MindEnvelope,
}

pub(super) struct ReadClaims {
    envelope: MindEnvelope,
}

pub(super) struct ApplyActivity {
    envelope: MindEnvelope,
}

pub(super) struct ReadActivity {
    envelope: MindEnvelope,
}

#[derive(kameo::Reply)]
pub(super) struct KernelReply {
    reply: Option<MindReply>,
}

impl KernelReply {
    fn new(reply: Option<MindReply>) -> Self {
        Self { reply }
    }

    pub(super) fn into_reply(self) -> Option<MindReply> {
        self.reply
    }
}

#[derive(kameo::Reply)]
pub(super) struct KernelCommit {
    rejection: Option<MindReply>,
}

impl KernelCommit {
    fn accepted() -> Self {
        Self { rejection: None }
    }

    fn rejected(error: crate::Error) -> Self {
        Self {
            rejection: Some(PersistenceRejection::reply(error)),
        }
    }

    pub(super) fn into_rejection(self) -> Option<MindReply> {
        self.rejection
    }
}

impl CommitMemoryGraph {
    pub(super) fn new(graph: MemoryGraph) -> Self {
        Self { graph }
    }
}

impl ApplyClaim {
    pub(super) fn new(envelope: MindEnvelope) -> Self {
        Self { envelope }
    }
}

impl ApplyHandoff {
    pub(super) fn new(envelope: MindEnvelope) -> Self {
        Self { envelope }
    }
}

impl ReadClaims {
    pub(super) fn new(envelope: MindEnvelope) -> Self {
        Self { envelope }
    }
}

impl ApplyActivity {
    pub(super) fn new(envelope: MindEnvelope) -> Self {
        Self { envelope }
    }
}

impl ReadActivity {
    pub(super) fn new(envelope: MindEnvelope) -> Self {
        Self { envelope }
    }
}

impl StoreKernel {
    fn open(store: StoreLocation) -> crate::Result<Self> {
        Ok(Self {
            tables: MindTables::open(&store)?,
        })
    }

    fn commit_memory_graph(&self, graph: MemoryGraph) -> KernelCommit {
        self.tables
            .replace_memory_graph(&graph)
            .map(|()| KernelCommit::accepted())
            .unwrap_or_else(KernelCommit::rejected)
    }

    fn load_memory_graph(&self) -> Option<MemoryGraph> {
        self.tables.memory_graph().ok().flatten()
    }

    fn apply_claim(&self, envelope: MindEnvelope) -> KernelReply {
        let reply = match envelope.request().clone() {
            MindRequest::RoleClaim(claim) => Some(
                ClaimLedger::new(&self.tables)
                    .apply_claim(claim)
                    .unwrap_or_else(PersistenceRejection::reply),
            ),
            MindRequest::RoleRelease(release) => Some(
                ClaimLedger::new(&self.tables)
                    .apply_release(release)
                    .unwrap_or_else(PersistenceRejection::reply),
            ),
            _ => None,
        };

        KernelReply::new(reply)
    }

    fn apply_handoff(&self, envelope: MindEnvelope) -> KernelReply {
        let reply = match envelope.request().clone() {
            MindRequest::RoleHandoff(handoff) => Some(
                ClaimLedger::new(&self.tables)
                    .apply_handoff(handoff)
                    .unwrap_or_else(PersistenceRejection::reply),
            ),
            _ => None,
        };

        KernelReply::new(reply)
    }

    fn read_claims(&self, envelope: MindEnvelope) -> KernelReply {
        let reply = match envelope.request().clone() {
            MindRequest::RoleObservation(observation) => Some(
                ClaimLedger::new(&self.tables)
                    .observe(observation)
                    .unwrap_or_else(PersistenceRejection::reply),
            ),
            _ => None,
        };

        KernelReply::new(reply)
    }

    fn apply_activity(&self, envelope: MindEnvelope) -> KernelReply {
        let reply = match envelope.request().clone() {
            MindRequest::ActivitySubmission(submission) => Some(
                ActivityLedger::new(&self.tables)
                    .submit(submission)
                    .unwrap_or_else(PersistenceRejection::reply),
            ),
            _ => None,
        };

        KernelReply::new(reply)
    }

    fn read_activity(&self, envelope: MindEnvelope) -> KernelReply {
        let reply = match envelope.request().clone() {
            MindRequest::ActivityQuery(query) => Some(
                ActivityLedger::new(&self.tables)
                    .query(query)
                    .unwrap_or_else(PersistenceRejection::reply),
            ),
            _ => None,
        };

        KernelReply::new(reply)
    }
}

impl Actor for StoreKernel {
    type Args = StoreLocation;
    type Error = crate::Error;

    async fn on_start(
        store: Self::Args,
        _actor_reference: ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        Self::open(store)
    }
}

impl Message<CommitMemoryGraph> for StoreKernel {
    type Reply = KernelCommit;

    async fn handle(
        &mut self,
        message: CommitMemoryGraph,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.commit_memory_graph(message.graph)
    }
}

impl Message<LoadMemoryGraph> for StoreKernel {
    type Reply = Option<MemoryGraph>;

    async fn handle(
        &mut self,
        _message: LoadMemoryGraph,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.load_memory_graph()
    }
}

impl Message<ApplyClaim> for StoreKernel {
    type Reply = KernelReply;

    async fn handle(
        &mut self,
        message: ApplyClaim,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.apply_claim(message.envelope)
    }
}

impl Message<ApplyHandoff> for StoreKernel {
    type Reply = KernelReply;

    async fn handle(
        &mut self,
        message: ApplyHandoff,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.apply_handoff(message.envelope)
    }
}

impl Message<ReadClaims> for StoreKernel {
    type Reply = KernelReply;

    async fn handle(
        &mut self,
        message: ReadClaims,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.read_claims(message.envelope)
    }
}

impl Message<ApplyActivity> for StoreKernel {
    type Reply = KernelReply;

    async fn handle(
        &mut self,
        message: ApplyActivity,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.apply_activity(message.envelope)
    }
}

impl Message<ReadActivity> for StoreKernel {
    type Reply = KernelReply;

    async fn handle(
        &mut self,
        message: ReadActivity,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.read_activity(message.envelope)
    }
}
