//! In-memory mind graph used by the first typed tests.
//!
//! Production storage moves through `persona-sema`; this module keeps
//! the graph reducer honest while durable tables land.

use std::cell::RefCell;

use crate::MindEnvelope;
use signal_persona_mind::{
    ActorName, AliasAddedEvent, AliasAssignment, DisplayId, Edge, EdgeAddedEvent, EdgeKind,
    EdgeTarget, Event, EventHeader, EventSeq, ExternalAlias, Item, ItemOpenedEvent, ItemReference,
    Kind, Link, LinkTarget, MindReply, MindRequest, Note, NoteAddedEvent, NoteSubmission, Opening,
    OpeningReceipt, OperationId, Query, QueryKind, QueryLimit, Rejection, RejectionReason,
    StableItemId, Status, StatusChange, StatusChangedEvent, View,
};

pub struct MemoryState {
    store: StoreLocation,
    graph: RefCell<Graph>,
}

impl MemoryState {
    pub fn open(store: StoreLocation) -> Self {
        Self {
            store,
            graph: RefCell::new(Graph::new(ActorName::new("persona-mind"))),
        }
    }

    pub fn store(&self) -> &StoreLocation {
        &self.store
    }

    pub fn dispatch(&self, request: MindRequest) -> Option<MindReply> {
        self.graph.borrow_mut().dispatch(request)
    }

    pub fn dispatch_envelope(&self, envelope: MindEnvelope) -> Option<MindReply> {
        self.graph.borrow_mut().dispatch_envelope(envelope)
    }
}

struct Graph {
    default_actor: ActorName,
    next_item: u64,
    next_event: u64,
    next_operation: u64,
    items: Vec<Item>,
    edges: Vec<Edge>,
    notes: Vec<Note>,
    events: Vec<Event>,
}

impl Graph {
    fn new(default_actor: ActorName) -> Self {
        Self {
            default_actor,
            next_item: 0,
            next_event: 0,
            next_operation: 0,
            items: Vec::new(),
            edges: Vec::new(),
            notes: Vec::new(),
            events: Vec::new(),
        }
    }

    fn dispatch(&mut self, request: MindRequest) -> Option<MindReply> {
        self.dispatch_envelope(MindEnvelope::new(self.default_actor.clone(), request))
    }

    fn dispatch_envelope(&mut self, envelope: MindEnvelope) -> Option<MindReply> {
        let MindEnvelope { actor, request } = envelope;
        match request {
            MindRequest::Open(opening) => Some(self.open(opening, &actor)),
            MindRequest::AddNote(note) => Some(self.add_note(note, &actor)),
            MindRequest::Link(link) => Some(self.link(link, &actor)),
            MindRequest::ChangeStatus(change) => Some(self.change_status(change, &actor)),
            MindRequest::AddAlias(alias) => Some(self.add_alias(alias, &actor)),
            MindRequest::Query(query) => Some(self.query(query)),
            MindRequest::RoleClaim(_)
            | MindRequest::RoleRelease(_)
            | MindRequest::RoleHandoff(_)
            | MindRequest::RoleObservation(_)
            | MindRequest::ActivitySubmission(_)
            | MindRequest::ActivityQuery(_) => None,
        }
    }

    fn open(&mut self, opening: Opening, actor: &ActorName) -> MindReply {
        self.next_item += 1;
        let header = self.next_header(actor);
        let item = Item {
            id: StableItemId::new(format!("item-{:016x}", self.next_item)),
            display_id: DisplayIdMint::new(self.next_item).into_display_id(),
            aliases: Vec::new(),
            kind: opening.kind,
            status: Status::Open,
            priority: opening.priority,
            title: opening.title,
            body: opening.body,
        };
        let event = ItemOpenedEvent { header, item };

        self.items.push(event.item.clone());
        self.events.push(Event::ItemOpened(event.clone()));

        MindReply::Opened(OpeningReceipt { event })
    }

    fn add_note(&mut self, submission: NoteSubmission, actor: &ActorName) -> MindReply {
        let Some(item) = self.resolve_item(&submission.item) else {
            return Self::rejected(RejectionReason::UnknownItem);
        };
        let header = self.next_header(actor);
        let note = Note {
            event: header.event,
            item,
            author: actor.clone(),
            body: submission.body,
        };
        let event = NoteAddedEvent { header, note };

        self.notes.push(event.note.clone());
        self.events.push(Event::NoteAdded(event.clone()));

        MindReply::NoteAdded(signal_persona_mind::NoteReceipt { event })
    }

    fn link(&mut self, link: Link, actor: &ActorName) -> MindReply {
        let Some(source) = self.resolve_item(&link.source) else {
            return Self::rejected(RejectionReason::UnknownItem);
        };
        let Some(target) = self.resolve_link_target(link.target) else {
            return Self::rejected(RejectionReason::UnknownItem);
        };
        let header = self.next_header(actor);
        let edge = Edge {
            event: header.event,
            source,
            kind: link.kind,
            target,
            body: link.body,
        };
        let event = EdgeAddedEvent { header, edge };

        self.edges.push(event.edge.clone());
        self.events.push(Event::EdgeAdded(event.clone()));

        MindReply::Linked(signal_persona_mind::LinkReceipt { event })
    }

    fn change_status(&mut self, change: StatusChange, actor: &ActorName) -> MindReply {
        let Some(position) = self.resolve_item_position(&change.item) else {
            return Self::rejected(RejectionReason::UnknownItem);
        };
        self.items[position].status = change.status;
        let item = self.items[position].id.clone();
        let header = self.next_header(actor);
        let event = StatusChangedEvent {
            header,
            item,
            status: change.status,
            body: change.body,
        };

        self.events.push(Event::StatusChanged(event.clone()));

        MindReply::StatusChanged(signal_persona_mind::StatusReceipt { event })
    }

    fn add_alias(&mut self, assignment: AliasAssignment, actor: &ActorName) -> MindReply {
        if self.alias_exists(&assignment.alias) {
            return Self::rejected(RejectionReason::DuplicateAlias);
        }
        let Some(position) = self.resolve_item_position(&assignment.item) else {
            return Self::rejected(RejectionReason::UnknownItem);
        };

        let item = self.items[position].id.clone();
        self.items[position].aliases.push(assignment.alias.clone());
        let header = self.next_header(actor);
        let event = AliasAddedEvent {
            header,
            item,
            alias: assignment.alias,
        };

        self.events.push(Event::AliasAdded(event.clone()));

        MindReply::AliasAdded(signal_persona_mind::AliasReceipt { event })
    }

    fn query(&self, query: Query) -> MindReply {
        let limit = query.limit.into_usize();
        let reply = match query.kind {
            QueryKind::Ready => self.ready_view(limit),
            QueryKind::Blocked => self.blocked_view(limit),
            QueryKind::Open => self.status_view(Status::Open, limit),
            QueryKind::RecentEvents => self.recent_events_view(limit),
            QueryKind::ByItem(reference) => {
                let Some(item) = self.resolve_item(&reference) else {
                    return Self::rejected(RejectionReason::UnknownItem);
                };
                self.item_view(&item, limit)
            }
            QueryKind::ByKind(kind) => self.kind_view(kind, limit),
            QueryKind::ByStatus(status) => self.status_view(status, limit),
            QueryKind::ByAlias(alias) => {
                let Some(item) = self.resolve_item(&ItemReference::Alias(alias)) else {
                    return Self::rejected(RejectionReason::UnknownItem);
                };
                self.item_view(&item, limit)
            }
        };

        MindReply::View(reply)
    }

    fn ready_view(&self, limit: usize) -> View {
        self.view_for_items(
            self.items
                .iter()
                .filter(|item| item.status == Status::Open && self.item_is_ready(&item.id))
                .take(limit)
                .cloned()
                .collect(),
        )
    }

    fn blocked_view(&self, limit: usize) -> View {
        self.view_for_items(
            self.items
                .iter()
                .filter(|item| item.status == Status::Open && !self.item_is_ready(&item.id))
                .take(limit)
                .cloned()
                .collect(),
        )
    }

    fn kind_view(&self, kind: Kind, limit: usize) -> View {
        self.view_for_items(
            self.items
                .iter()
                .filter(|item| item.kind == kind)
                .take(limit)
                .cloned()
                .collect(),
        )
    }

    fn status_view(&self, status: Status, limit: usize) -> View {
        self.view_for_items(
            self.items
                .iter()
                .filter(|item| item.status == status)
                .take(limit)
                .cloned()
                .collect(),
        )
    }

    fn recent_events_view(&self, limit: usize) -> View {
        let mut events = self
            .events
            .iter()
            .rev()
            .take(limit)
            .cloned()
            .collect::<Vec<_>>();
        events.reverse();

        View {
            items: Vec::new(),
            edges: Vec::new(),
            notes: Vec::new(),
            events,
        }
    }

    fn item_view(&self, item: &StableItemId, limit: usize) -> View {
        self.view_for_items(
            self.items
                .iter()
                .filter(|candidate| &candidate.id == item)
                .take(limit)
                .cloned()
                .collect(),
        )
    }

    fn view_for_items(&self, items: Vec<Item>) -> View {
        let item_ids = items
            .iter()
            .map(|item| item.id.clone())
            .collect::<Vec<StableItemId>>();

        View {
            edges: self.edges_for_items(&item_ids),
            notes: self.notes_for_items(&item_ids),
            events: self.events.clone(),
            items,
        }
    }

    fn edges_for_items(&self, item_ids: &[StableItemId]) -> Vec<Edge> {
        self.edges
            .iter()
            .filter(|edge| {
                item_ids.contains(&edge.source)
                    || match &edge.target {
                        EdgeTarget::Item(target) => item_ids.contains(target),
                        EdgeTarget::External(_) => false,
                    }
            })
            .cloned()
            .collect()
    }

    fn notes_for_items(&self, item_ids: &[StableItemId]) -> Vec<Note> {
        self.notes
            .iter()
            .filter(|note| item_ids.contains(&note.item))
            .cloned()
            .collect()
    }

    fn item_is_ready(&self, item: &StableItemId) -> bool {
        self.edges.iter().all(|edge| {
            if edge.kind != EdgeKind::DependsOn || &edge.source != item {
                return true;
            }

            match &edge.target {
                EdgeTarget::Item(target) => self
                    .item_by_id(target)
                    .map(|target_item| target_item.status == Status::Closed)
                    .unwrap_or(false),
                EdgeTarget::External(_) => false,
            }
        })
    }

    fn resolve_link_target(&self, target: LinkTarget) -> Option<EdgeTarget> {
        match target {
            LinkTarget::Item(reference) => self.resolve_item(&reference).map(EdgeTarget::Item),
            LinkTarget::External(external) => Some(EdgeTarget::External(external)),
        }
    }

    fn resolve_item(&self, reference: &ItemReference) -> Option<StableItemId> {
        match reference {
            ItemReference::Stable(id) => self.item_by_id(id).map(|item| item.id.clone()),
            ItemReference::Display(display_id) => self
                .items
                .iter()
                .find(|item| &item.display_id == display_id)
                .map(|item| item.id.clone()),
            ItemReference::Alias(alias) => self
                .items
                .iter()
                .find(|item| item.aliases.iter().any(|candidate| candidate == alias))
                .map(|item| item.id.clone()),
        }
    }

    fn resolve_item_position(&self, reference: &ItemReference) -> Option<usize> {
        self.resolve_item(reference)
            .and_then(|id| self.items.iter().position(|item| item.id == id))
    }

    fn item_by_id(&self, id: &StableItemId) -> Option<&Item> {
        self.items.iter().find(|item| &item.id == id)
    }

    fn alias_exists(&self, alias: &ExternalAlias) -> bool {
        self.items
            .iter()
            .any(|item| item.aliases.iter().any(|candidate| candidate == alias))
    }

    fn next_header(&mut self, actor: &ActorName) -> EventHeader {
        self.next_event += 1;
        self.next_operation += 1;
        EventHeader {
            event: EventSeq::new(self.next_event),
            operation: OperationId::new(format!("op-{:016x}", self.next_operation)),
            actor: actor.clone(),
        }
    }

    fn rejected(reason: RejectionReason) -> MindReply {
        MindReply::Rejected(Rejection { reason })
    }
}

struct DisplayIdMint {
    value: u64,
}

impl DisplayIdMint {
    fn new(value: u64) -> Self {
        Self { value }
    }

    fn into_display_id(self) -> DisplayId {
        let alphabet = b"0123456789abcdefghjkmnpqrstvwxyz";
        let mut value = self.value;
        let mut text = [b'0'; 5];

        for slot in text.iter_mut().rev() {
            let index = (value % 32) as usize;
            *slot = alphabet[index];
            value /= 32;
        }

        DisplayId::new(String::from_utf8(text.to_vec()).expect("display ids are ascii"))
    }
}

trait QueryLimitExt {
    fn into_usize(self) -> usize;
}

impl QueryLimitExt for QueryLimit {
    fn into_usize(self) -> usize {
        usize::from(self.into_u16())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, kameo::Reply)]
pub struct StoreLocation {
    path: String,
}

impl StoreLocation {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }

    pub fn as_str(&self) -> &str {
        &self.path
    }
}
