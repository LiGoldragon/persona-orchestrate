use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use kameo::actor::ActorRef;
use sema::{SchemaVersion, Table};
use sema_engine::{
    Assertion, Engine, EngineOpen, EngineRecord, QueryPlan, RecordKey, SinkError,
    SubscriptionDeliveryMode, SubscriptionEvent as EngineSubscriptionEvent, SubscriptionSink,
    TableDescriptor, TableName, TableReference,
};
use signal_persona_mind::{
    Activity, ActorName, RecordId, Relation, RelationId, RoleName, ScopeReason, ScopeReference,
    SubmitRelation, SubmitThought, SubscribeRelations, SubscribeThoughts, SubscriptionId, Thought,
    TimestampNanos,
};

use crate::actors::subscription::{
    PublishRelationDelta, PublishThoughtDelta, SubscriptionSupervisor,
};
use crate::{MemoryGraph, Result, StoreLocation};

const MIND_SCHEMA_VERSION: SchemaVersion = SchemaVersion::new(7);

const CLAIMS: Table<&'static str, StoredClaim> = Table::new("claims");
const ACTIVITIES: Table<u64, StoredActivity> = Table::new("activities");
const ACTIVITY_NEXT_SLOT: Table<&'static str, u64> = Table::new("activity_next_slot");
const MEMORY_GRAPH: Table<&'static str, MemoryGraph> = Table::new("memory_graph");
const THOUGHT_SUBSCRIPTIONS: Table<&'static str, StoredThoughtSubscription> =
    Table::new("thought_subscriptions");
const RELATION_SUBSCRIPTIONS: Table<&'static str, StoredRelationSubscription> =
    Table::new("relation_subscriptions");
const ACTIVITY_NEXT_SLOT_KEY: &str = "next";
const MEMORY_GRAPH_KEY: &str = "current";
const THOUGHTS: TableName = TableName::new("thoughts");
const RELATIONS: TableName = TableName::new("relations");

pub struct MindTables {
    engine: Engine,
    thoughts: TableReference<StoredThought>,
    relations: TableReference<StoredRelation>,
    subscription_publisher: GraphSubscriptionPublisher,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct StoredClaim {
    pub role: RoleName,
    pub scope: ScopeReference,
    pub reason: ScopeReason,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct StoredActivity {
    pub slot: u64,
    pub role: RoleName,
    pub scope: ScopeReference,
    pub reason: ScopeReason,
    pub stamped_at: TimestampNanos,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug, Clone, PartialEq, Eq)]
pub(crate) struct StoredThoughtSubscription {
    pub subscription: SubscriptionId,
    pub filter: signal_persona_mind::ThoughtFilter,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug, Clone, PartialEq, Eq)]
pub(crate) struct StoredRelationSubscription {
    pub subscription: SubscriptionId,
    pub filter: signal_persona_mind::RelationFilter,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug, Clone, PartialEq, Eq)]
pub(crate) struct StoredThought {
    record: Thought,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug, Clone, PartialEq, Eq)]
pub(crate) struct StoredRelation {
    record: Relation,
}

pub(crate) struct OpenedThoughtSubscription {
    record: StoredThoughtSubscription,
    initial: Vec<Thought>,
}

pub(crate) struct OpenedRelationSubscription {
    record: StoredRelationSubscription,
    initial: Vec<Relation>,
}

#[derive(Clone)]
pub(crate) enum GraphSubscriptionPublisher {
    Actor(ActorRef<SubscriptionSupervisor>),
    #[cfg(test)]
    Disabled,
}

impl StoredThought {
    fn new(record: Thought) -> Self {
        Self { record }
    }

    fn into_record(self) -> Thought {
        self.record
    }
}

impl StoredRelation {
    fn new(record: Relation) -> Self {
        Self { record }
    }

    fn into_record(self) -> Relation {
        self.record
    }
}

impl OpenedThoughtSubscription {
    fn new(record: StoredThoughtSubscription, initial: Vec<Thought>) -> Self {
        Self { record, initial }
    }

    pub(crate) fn record(&self) -> &StoredThoughtSubscription {
        &self.record
    }

    pub(crate) fn initial(&self) -> &[Thought] {
        &self.initial
    }
}

impl OpenedRelationSubscription {
    fn new(record: StoredRelationSubscription, initial: Vec<Relation>) -> Self {
        Self { record, initial }
    }

    pub(crate) fn record(&self) -> &StoredRelationSubscription {
        &self.record
    }

    pub(crate) fn initial(&self) -> &[Relation] {
        &self.initial
    }
}

impl EngineRecord for StoredThought {
    fn record_key(&self) -> RecordKey {
        RecordKey::new(self.record.id.as_str())
    }
}

impl EngineRecord for StoredRelation {
    fn record_key(&self) -> RecordKey {
        RecordKey::new(self.record.id.as_str())
    }
}

impl StoredActivity {
    fn new(
        slot: u64,
        role: RoleName,
        scope: ScopeReference,
        reason: ScopeReason,
        stamped_at: TimestampNanos,
    ) -> Self {
        Self {
            slot,
            role,
            scope,
            reason,
            stamped_at,
        }
    }

    pub fn into_activity(self) -> Activity {
        Activity {
            role: self.role,
            scope: self.scope,
            reason: self.reason,
            stamped_at: self.stamped_at,
        }
    }
}

impl StoredClaim {
    pub fn new(role: RoleName, scope: ScopeReference, reason: ScopeReason) -> Self {
        Self {
            role,
            scope,
            reason,
        }
    }

    pub fn key(&self) -> String {
        ClaimKey::new(self.role, &self.scope).into_string()
    }
}

impl MindTables {
    pub(crate) fn open(
        store: &StoreLocation,
        subscription_publisher: GraphSubscriptionPublisher,
    ) -> Result<Self> {
        let mut engine = Engine::open(EngineOpen::new(store.as_path(), MIND_SCHEMA_VERSION))?;
        engine.storage_kernel().write(|transaction| {
            CLAIMS.ensure(transaction)?;
            ACTIVITIES.ensure(transaction)?;
            ACTIVITY_NEXT_SLOT.ensure(transaction)?;
            MEMORY_GRAPH.ensure(transaction)?;
            THOUGHT_SUBSCRIPTIONS.ensure(transaction)?;
            RELATION_SUBSCRIPTIONS.ensure(transaction)?;
            Ok(())
        })?;
        let thoughts = engine.register_table(TableDescriptor::new(THOUGHTS))?;
        let relations = engine.register_table(TableDescriptor::new(RELATIONS))?;
        Ok(Self {
            engine,
            thoughts,
            relations,
            subscription_publisher,
        })
    }

    pub fn claim_records(&self) -> Result<Vec<StoredClaim>> {
        Ok(self.engine.storage_kernel().read(|transaction| {
            Ok(CLAIMS
                .iter(transaction)?
                .into_iter()
                .map(|(_key, claim)| claim)
                .collect())
        })?)
    }

    pub fn replace_claims(
        &self,
        remove_keys: &[String],
        insert_claims: &[StoredClaim],
    ) -> Result<()> {
        self.engine.storage_kernel().write(|transaction| {
            for key in remove_keys {
                CLAIMS.remove(transaction, key.as_str())?;
            }
            for claim in insert_claims {
                let key = claim.key();
                CLAIMS.insert(transaction, key.as_str(), claim)?;
            }
            Ok(())
        })?;
        Ok(())
    }

    pub fn append_activity(
        &self,
        role: RoleName,
        scope: ScopeReference,
        reason: ScopeReason,
    ) -> Result<StoredActivity> {
        let slot = self.next_activity_slot()?;
        let stamped_at = StoreClock::system().timestamp()?;
        let activity = StoredActivity::new(slot.value(), role, scope, reason, stamped_at);
        self.engine.storage_kernel().write(|transaction| {
            ACTIVITIES.insert(transaction, slot.value(), &activity)?;
            ACTIVITY_NEXT_SLOT.insert(transaction, ACTIVITY_NEXT_SLOT_KEY, &slot.next_value())?;
            Ok(())
        })?;
        Ok(activity)
    }

    pub fn activity_records(&self) -> Result<Vec<StoredActivity>> {
        Ok(self.engine.storage_kernel().read(|transaction| {
            Ok(ACTIVITIES
                .iter(transaction)?
                .into_iter()
                .map(|(_slot, activity)| activity)
                .collect())
        })?)
    }

    pub(crate) fn memory_graph(&self) -> Result<Option<MemoryGraph>> {
        Ok(self
            .engine
            .storage_kernel()
            .read(|transaction| MEMORY_GRAPH.get(transaction, MEMORY_GRAPH_KEY))?)
    }

    pub(crate) fn replace_memory_graph(&self, graph: &MemoryGraph) -> Result<()> {
        self.engine.storage_kernel().write(|transaction| {
            MEMORY_GRAPH.insert(transaction, MEMORY_GRAPH_KEY, graph)?;
            Ok(())
        })?;
        Ok(())
    }

    pub(crate) fn append_thought(
        &self,
        actor: ActorName,
        submission: SubmitThought,
    ) -> Result<Thought> {
        let actual = submission.body.kind();
        if submission.kind != actual {
            return Err(crate::Error::MindGraphThoughtKindMismatch {
                declared: submission.kind,
                actual,
            });
        }

        let id = RecordId::new(GraphIdMint::new(&self.engine).record_id_string()?);
        let thought = Thought {
            id,
            kind: submission.kind,
            body: submission.body,
            author: actor,
            occurred_at: StoreClock::system().timestamp()?,
        };

        self.engine.assert(Assertion::new(
            self.thoughts,
            StoredThought::new(thought.clone()),
        ))?;
        Ok(thought)
    }

    pub(crate) fn append_relation(
        &self,
        actor: ActorName,
        submission: SubmitRelation,
    ) -> Result<Relation> {
        let source = self.read_thought(&submission.source)?;
        let target = self.read_thought(&submission.target)?;
        submission
            .kind
            .validate_endpoints(&source, &target)
            .map_err(|mismatch| crate::Error::MindGraphRelationKindMismatch { mismatch })?;

        let id = RelationId::new(GraphIdMint::new(&self.engine).record_id_string()?);
        let relation = Relation {
            id,
            kind: submission.kind,
            source: submission.source,
            target: submission.target,
            author: actor,
            occurred_at: StoreClock::system().timestamp()?,
            note: submission.note,
        };

        self.engine.assert(Assertion::new(
            self.relations,
            StoredRelation::new(relation.clone()),
        ))?;
        Ok(relation)
    }

    pub(crate) fn thought_records(&self) -> Result<Vec<Thought>> {
        Ok(self
            .engine
            .match_records(QueryPlan::all(self.thoughts))?
            .records()
            .iter()
            .cloned()
            .map(StoredThought::into_record)
            .collect())
    }

    pub(crate) fn relation_records(&self) -> Result<Vec<Relation>> {
        Ok(self
            .engine
            .match_records(QueryPlan::all(self.relations))?
            .records()
            .iter()
            .cloned()
            .map(StoredRelation::into_record)
            .collect())
    }

    pub(crate) fn append_thought_subscription(
        &self,
        subscription: SubscribeThoughts,
    ) -> Result<OpenedThoughtSubscription> {
        let filter = subscription.filter;
        let receipt = self.engine.subscribe(
            QueryPlan::all(self.thoughts),
            ThoughtSubscriptionSink::new(
                THOUGHTS,
                filter.clone(),
                self.subscription_publisher.clone(),
            ),
        )?;
        let initial = receipt
            .initial()
            .snapshot()
            .records()
            .iter()
            .cloned()
            .map(StoredThought::into_record)
            .collect();
        let record = StoredThoughtSubscription {
            subscription: Self::subscription_id_from_engine(receipt.handle().id()),
            filter,
        };
        self.engine.storage_kernel().write(|transaction| {
            THOUGHT_SUBSCRIPTIONS.insert(transaction, record.subscription.as_str(), &record)?;
            Ok(())
        })?;
        Ok(OpenedThoughtSubscription::new(record, initial))
    }

    pub(crate) fn append_relation_subscription(
        &self,
        subscription: SubscribeRelations,
    ) -> Result<OpenedRelationSubscription> {
        let filter = subscription.filter;
        let receipt = self.engine.subscribe(
            QueryPlan::all(self.relations),
            RelationSubscriptionSink::new(
                RELATIONS,
                filter.clone(),
                self.subscription_publisher.clone(),
            ),
        )?;
        let initial = receipt
            .initial()
            .snapshot()
            .records()
            .iter()
            .cloned()
            .map(StoredRelation::into_record)
            .collect();
        let record = StoredRelationSubscription {
            subscription: Self::subscription_id_from_engine(receipt.handle().id()),
            filter,
        };
        self.engine.storage_kernel().write(|transaction| {
            RELATION_SUBSCRIPTIONS.insert(transaction, record.subscription.as_str(), &record)?;
            Ok(())
        })?;
        Ok(OpenedRelationSubscription::new(record, initial))
    }

    fn next_activity_slot(&self) -> Result<ActivitySlot> {
        let stored = self
            .engine
            .storage_kernel()
            .read(|transaction| ACTIVITY_NEXT_SLOT.get(transaction, ACTIVITY_NEXT_SLOT_KEY))?;
        match stored {
            Some(next_slot) => Ok(ActivitySlot::new(next_slot)),
            None => Ok(ActivitySlot::after_records(&self.activity_records()?)),
        }
    }

    fn read_thought(&self, record: &RecordId) -> Result<Thought> {
        self.engine
            .match_records(QueryPlan::key(
                self.thoughts,
                RecordKey::new(record.as_str()),
            ))?
            .records()
            .first()
            .cloned()
            .map(StoredThought::into_record)
            .ok_or_else(|| crate::Error::MindGraphMissingRecord {
                record: record.as_str().to_string(),
            })
    }

    fn subscription_id_from_engine(engine_id: sema_engine::SubscriptionId) -> SubscriptionId {
        SubscriptionId::new(CompactGraphId::new(engine_id.value().saturating_sub(1)).into_string())
    }
}

impl GraphSubscriptionPublisher {
    pub(crate) fn actor(actor: ActorRef<SubscriptionSupervisor>) -> Self {
        Self::Actor(actor)
    }

    #[cfg(test)]
    fn disabled() -> Self {
        Self::Disabled
    }

    fn publish_thought(
        &self,
        subscription: SubscriptionId,
        filter: signal_persona_mind::ThoughtFilter,
        thought: Thought,
    ) -> std::result::Result<(), SinkError> {
        match self {
            Self::Actor(actor) => actor
                .tell(PublishThoughtDelta::new(subscription, filter, thought))
                .try_send()
                .map_err(|error| SinkError::new(error.to_string())),
            #[cfg(test)]
            Self::Disabled => Ok(()),
        }
    }

    fn publish_relation(
        &self,
        subscription: SubscriptionId,
        filter: signal_persona_mind::RelationFilter,
        relation: Relation,
    ) -> std::result::Result<(), SinkError> {
        match self {
            Self::Actor(actor) => actor
                .tell(PublishRelationDelta::new(subscription, filter, relation))
                .try_send()
                .map_err(|error| SinkError::new(error.to_string())),
            #[cfg(test)]
            Self::Disabled => Ok(()),
        }
    }
}

struct ActivitySlot {
    value: u64,
}

struct GraphIdMint<'engine> {
    engine: &'engine Engine,
}

struct ThoughtSubscriptionSink {
    table: TableName,
    filter: signal_persona_mind::ThoughtFilter,
    publisher: GraphSubscriptionPublisher,
}

struct RelationSubscriptionSink {
    table: TableName,
    filter: signal_persona_mind::RelationFilter,
    publisher: GraphSubscriptionPublisher,
}

impl<'engine> GraphIdMint<'engine> {
    fn new(engine: &'engine Engine) -> Self {
        Self { engine }
    }

    fn record_id_string(&self) -> Result<String> {
        let next_snapshot = self.engine.latest_snapshot()?.next();
        Ok(CompactGraphId::new(next_snapshot.value().saturating_sub(1)).into_string())
    }
}

struct CompactGraphId {
    value: u64,
}

impl ThoughtSubscriptionSink {
    fn new(
        table: TableName,
        filter: signal_persona_mind::ThoughtFilter,
        publisher: GraphSubscriptionPublisher,
    ) -> Arc<Self> {
        Arc::new(Self {
            table,
            filter,
            publisher,
        })
    }

    fn ensure_table(&self, table: &TableName) -> std::result::Result<(), SinkError> {
        if self.table == *table {
            return Ok(());
        }

        Err(SinkError::new(format!(
            "subscription sink for {} received {}",
            self.table.as_str(),
            table.as_str()
        )))
    }
}

impl RelationSubscriptionSink {
    fn new(
        table: TableName,
        filter: signal_persona_mind::RelationFilter,
        publisher: GraphSubscriptionPublisher,
    ) -> Arc<Self> {
        Arc::new(Self {
            table,
            filter,
            publisher,
        })
    }

    fn ensure_table(&self, table: &TableName) -> std::result::Result<(), SinkError> {
        if self.table == *table {
            return Ok(());
        }

        Err(SinkError::new(format!(
            "subscription sink for {} received {}",
            self.table.as_str(),
            table.as_str()
        )))
    }
}

impl SubscriptionSink<StoredThought> for ThoughtSubscriptionSink {
    fn delivery_mode(&self) -> SubscriptionDeliveryMode {
        SubscriptionDeliveryMode::Inline
    }

    fn deliver(
        &self,
        event: EngineSubscriptionEvent<StoredThought>,
    ) -> std::result::Result<(), SinkError> {
        match event {
            EngineSubscriptionEvent::InitialSnapshot(snapshot) => {
                self.ensure_table(snapshot.handle().table())
            }
            EngineSubscriptionEvent::Delta(delta) => {
                self.ensure_table(delta.table())?;
                self.publisher.publish_thought(
                    MindTables::subscription_id_from_engine(delta.handle().id()),
                    self.filter.clone(),
                    delta.record().clone().into_record(),
                )
            }
        }
    }
}

impl SubscriptionSink<StoredRelation> for RelationSubscriptionSink {
    fn delivery_mode(&self) -> SubscriptionDeliveryMode {
        SubscriptionDeliveryMode::Inline
    }

    fn deliver(
        &self,
        event: EngineSubscriptionEvent<StoredRelation>,
    ) -> std::result::Result<(), SinkError> {
        match event {
            EngineSubscriptionEvent::InitialSnapshot(snapshot) => {
                self.ensure_table(snapshot.handle().table())
            }
            EngineSubscriptionEvent::Delta(delta) => {
                self.ensure_table(delta.table())?;
                self.publisher.publish_relation(
                    MindTables::subscription_id_from_engine(delta.handle().id()),
                    self.filter.clone(),
                    delta.record().clone().into_record(),
                )
            }
        }
    }
}

impl CompactGraphId {
    fn new(value: u64) -> Self {
        Self { value }
    }

    fn into_string(self) -> String {
        let alphabet = b"abcdefghijklmnopqrstuvwxyz";
        let mut value = self.value;
        let mut bytes = Vec::new();
        loop {
            bytes.push(alphabet[(value % 26) as usize]);
            value /= 26;
            if value == 0 {
                break;
            }
        }
        while bytes.len() < 3 {
            bytes.push(alphabet[0]);
        }
        bytes.reverse();
        String::from_utf8(bytes).expect("compact graph id is ascii")
    }
}

impl ActivitySlot {
    fn new(value: u64) -> Self {
        Self { value }
    }

    fn after_records(records: &[StoredActivity]) -> Self {
        let value = records
            .iter()
            .map(|activity| activity.slot)
            .max()
            .map_or(0, |slot| slot + 1);
        Self { value }
    }

    fn value(&self) -> u64 {
        self.value
    }

    fn next_value(&self) -> u64 {
        self.value + 1
    }
}

struct StoreClock {
    epoch: SystemTime,
}

impl StoreClock {
    fn system() -> Self {
        Self { epoch: UNIX_EPOCH }
    }

    fn timestamp(&self) -> Result<TimestampNanos> {
        let nanos = SystemTime::now()
            .duration_since(self.epoch)?
            .as_nanos()
            .min(u64::MAX as u128) as u64;
        Ok(TimestampNanos::new(nanos))
    }
}

struct ClaimKey {
    role: RoleName,
    scope: String,
}

impl ClaimKey {
    fn new(role: RoleName, scope: &ScopeReference) -> Self {
        Self {
            role,
            scope: ScopeKey::new(scope).into_string(),
        }
    }

    fn into_string(self) -> String {
        format!("{}|{}", self.role.as_wire_token(), self.scope)
    }
}

struct ScopeKey {
    value: String,
}

impl ScopeKey {
    fn new(scope: &ScopeReference) -> Self {
        let value = match scope {
            ScopeReference::Path(path) => format!("path:{}", path.as_str()),
            ScopeReference::Task(task) => format!("task:{}", task.as_str()),
        };
        Self { value }
    }

    fn into_string(self) -> String {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use signal_core::SemaVerb;
    use signal_persona_mind::{
        ByThoughtKind, GoalBody, GoalScope, TextBody, ThoughtBody, ThoughtFilter, ThoughtKind,
        WorkspaceGoal,
    };

    #[test]
    fn thought_subscription_is_durable_table_data() {
        let store = StoreLocation::new(unique_store_path("thought-subscription-durable"));
        let tables =
            MindTables::open(&store, GraphSubscriptionPublisher::disabled()).expect("tables open");
        let opened = tables
            .append_thought_subscription(SubscribeThoughts {
                filter: ThoughtFilter::ByKind(ByThoughtKind {
                    kinds: vec![ThoughtKind::Goal],
                }),
            })
            .expect("subscription appends");
        let stored = opened.record().clone();
        drop(tables);

        let reopened = MindTables::open(&store, GraphSubscriptionPublisher::disabled())
            .expect("tables reopen");
        let persisted = reopened
            .engine
            .storage_kernel()
            .read(|transaction| {
                THOUGHT_SUBSCRIPTIONS.get(transaction, stored.subscription.as_str())
            })
            .expect("subscription lookup")
            .expect("subscription stored");

        assert_eq!(persisted, stored);
        assert_eq!(persisted.subscription.as_str().len(), 3);
    }

    #[test]
    fn typed_subscription_registration_uses_sema_engine_catalog() {
        let store = StoreLocation::new(unique_store_path("subscription-engine-catalog"));
        let tables =
            MindTables::open(&store, GraphSubscriptionPublisher::disabled()).expect("tables open");
        let opened = tables
            .append_thought_subscription(SubscribeThoughts {
                filter: ThoughtFilter::ByKind(ByThoughtKind {
                    kinds: vec![ThoughtKind::Goal],
                }),
            })
            .expect("subscription appends");

        let registrations = tables
            .engine
            .subscription_registrations()
            .expect("subscription registrations read");

        assert_eq!(opened.record().subscription.as_str(), "aaa");
        assert_eq!(registrations.len(), 1);
        assert_eq!(registrations[0].table_name(), "thoughts");
        assert_eq!(registrations[0].id().value(), 1);
    }

    #[test]
    fn typed_thought_append_uses_sema_engine_operation_log() {
        let store = StoreLocation::new(unique_store_path("thought-operation-log"));
        let tables =
            MindTables::open(&store, GraphSubscriptionPublisher::disabled()).expect("tables open");
        let thought = tables
            .append_thought(
                ActorName::new("operator"),
                SubmitThought {
                    kind: ThoughtKind::Goal,
                    body: ThoughtBody::Goal(GoalBody {
                        description: TextBody::new("prove engine path"),
                        scope: GoalScope::Workspace(WorkspaceGoal {
                            workspace: TextBody::new("primary"),
                        }),
                    }),
                },
            )
            .expect("thought appends");

        let log = tables.engine.operation_log().expect("operation log reads");
        let records = tables.thought_records().expect("thoughts read");

        assert_eq!(thought.id.as_str(), "aaa");
        assert_eq!(records, vec![thought.clone()]);
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].verb(), SemaVerb::Assert);
        assert_eq!(log[0].table_name(), "thoughts");
        assert_eq!(
            log[0].key().map(RecordKey::as_str),
            Some(thought.id.as_str())
        );
    }

    fn unique_store_path(name: &str) -> String {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir()
            .join(format!(
                "persona-mind-{name}-{}-{stamp}.redb",
                std::process::id()
            ))
            .to_string_lossy()
            .to_string()
    }
}
