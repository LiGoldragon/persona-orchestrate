use kameo::actor::{Actor, ActorRef};
use kameo::error::Infallible;
use kameo::message::{Context, Message};
use signal_persona_mind::{MindRequest, QueryKind};

use crate::{MindEnvelope, Result as CrateResult};

use super::pipeline::PipelineReply;
use super::store;
use super::trace::{ActorKind, ActorTrace, TraceAction};

pub(super) struct ViewSupervisorActor {
    store: ActorRef<store::StoreSupervisorActor>,
}

#[derive(Clone)]
pub(super) struct Arguments {
    pub(super) store: ActorRef<store::StoreSupervisorActor>,
}

pub struct ReadMemory {
    pub envelope: MindEnvelope,
    pub trace: ActorTrace,
}

impl ViewSupervisorActor {
    fn new(store: ActorRef<store::StoreSupervisorActor>) -> Self {
        Self { store }
    }

    async fn read_memory(
        &self,
        envelope: MindEnvelope,
        mut trace: ActorTrace,
    ) -> CrateResult<PipelineReply> {
        trace.record(ActorKind::ViewSupervisorActor, TraceAction::MessageReceived);
        trace.record(
            ActorKind::QuerySupervisorActor,
            TraceAction::MessageReceived,
        );
        trace.record(ActorKind::QueryPlanActor, TraceAction::MessageReceived);
        QueryOperation::from_request(envelope.request()).record_into(&mut trace);
        trace.record(ActorKind::GraphTraversalActor, TraceAction::MessageReceived);

        let mut reply = self
            .store
            .ask(store::ReadMemory { envelope, trace })
            .await
            .map_err(|error| crate::Error::ActorCall(error.to_string()))?;
        reply.trace.record(
            ActorKind::QueryResultShapeActor,
            TraceAction::MessageReceived,
        );
        Ok(reply)
    }
}

impl Actor for ViewSupervisorActor {
    type Args = Arguments;
    type Error = Infallible;

    async fn on_start(
        arguments: Self::Args,
        _actor_reference: ActorRef<Self>,
    ) -> std::result::Result<Self, Self::Error> {
        Ok(Self::new(arguments.store))
    }
}

impl Message<ReadMemory> for ViewSupervisorActor {
    type Reply = PipelineReply;

    async fn handle(
        &mut self,
        message: ReadMemory,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        match self.read_memory(message.envelope, message.trace).await {
            Ok(reply) => reply,
            Err(_error) => PipelineReply::new(None, ActorTrace::new()),
        }
    }
}

struct QueryOperation {
    actor: ActorKind,
}

impl QueryOperation {
    fn from_request(request: &MindRequest) -> Self {
        let actor = match request {
            MindRequest::Query(query) => match &query.kind {
                QueryKind::Ready => ActorKind::ReadyWorkViewActor,
                QueryKind::Blocked => ActorKind::BlockedWorkViewActor,
                QueryKind::RecentEvents => ActorKind::RecentActivityViewActor,
                QueryKind::Open
                | QueryKind::ByItem(_)
                | QueryKind::ByKind(_)
                | QueryKind::ByStatus(_)
                | QueryKind::ByAlias(_) => ActorKind::GraphTraversalActor,
            },
            _ => ActorKind::ErrorShapeActor,
        };
        Self { actor }
    }

    fn record_into(&self, trace: &mut ActorTrace) {
        trace.record(self.actor, TraceAction::MessageReceived);
    }
}
