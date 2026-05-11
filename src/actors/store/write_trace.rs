use signal_persona_mind::MindRequest;

use super::super::trace::{ActorKind, ActorTrace, TraceAction};

pub(super) struct WriteTrace {
    reads_existing_graph: bool,
    mints_identity: bool,
}

impl WriteTrace {
    pub(super) fn from_request(request: &MindRequest) -> Self {
        match request {
            MindRequest::Opening(_) => Self {
                reads_existing_graph: false,
                mints_identity: true,
            },
            MindRequest::NoteSubmission(_)
            | MindRequest::Link(_)
            | MindRequest::StatusChange(_)
            | MindRequest::AliasAssignment(_) => Self {
                reads_existing_graph: true,
                mints_identity: false,
            },
            _ => Self {
                reads_existing_graph: false,
                mints_identity: false,
            },
        }
    }

    pub(super) fn record_into(&self, trace: &mut ActorTrace) {
        if self.reads_existing_graph {
            trace.record(ActorKind::SemaReader, TraceAction::MessageReceived);
        }
        if self.mints_identity {
            trace.record(ActorKind::IdMint, TraceAction::MessageReceived);
        }
        trace.record(ActorKind::Clock, TraceAction::MessageReceived);
        trace.record(ActorKind::SemaWriter, TraceAction::WriteIntentSent);
    }
}
