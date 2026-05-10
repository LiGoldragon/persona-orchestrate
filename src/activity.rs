use crate::{MindTables, Result, StoredActivity};
use signal_persona_mind::{
    ActivityAcknowledgment, ActivityFilter, ActivityList, ActivityQuery, ActivitySubmission,
    MindReply, ScopeReference,
};

pub struct ActivityLedger<'tables> {
    tables: &'tables MindTables,
}

impl<'tables> ActivityLedger<'tables> {
    pub fn new(tables: &'tables MindTables) -> Self {
        Self { tables }
    }

    pub fn submit(&self, submission: ActivitySubmission) -> Result<MindReply> {
        let activity =
            self.tables
                .append_activity(submission.role, submission.scope, submission.reason)?;

        Ok(MindReply::ActivityAcknowledgment(ActivityAcknowledgment {
            slot: activity.slot,
        }))
    }

    pub fn query(&self, query: ActivityQuery) -> Result<MindReply> {
        let limit = query.limit as usize;
        let mut records = self.tables.activity_records()?;
        records.sort_by_key(|activity| activity.slot);
        records.reverse();

        let records = records
            .into_iter()
            .filter(|activity| ActivityPredicate::new(&query.filters).matches(activity))
            .take(limit)
            .map(StoredActivity::into_activity)
            .collect();

        Ok(MindReply::ActivityList(ActivityList { records }))
    }
}

struct ActivityPredicate<'filters> {
    filters: &'filters [ActivityFilter],
}

impl<'filters> ActivityPredicate<'filters> {
    fn new(filters: &'filters [ActivityFilter]) -> Self {
        Self { filters }
    }

    fn matches(&self, activity: &StoredActivity) -> bool {
        self.filters
            .iter()
            .all(|filter| ActivityFilterMatch::new(filter).matches(activity))
    }
}

struct ActivityFilterMatch<'filter> {
    filter: &'filter ActivityFilter,
}

impl<'filter> ActivityFilterMatch<'filter> {
    fn new(filter: &'filter ActivityFilter) -> Self {
        Self { filter }
    }

    fn matches(&self, activity: &StoredActivity) -> bool {
        match self.filter {
            ActivityFilter::RoleFilter(role) => activity.role == *role,
            ActivityFilter::PathPrefix(prefix) => match &activity.scope {
                ScopeReference::Path(path) => path.as_str().starts_with(prefix.as_str()),
                ScopeReference::Task(_) => false,
            },
            ActivityFilter::TaskToken(token) => match &activity.scope {
                ScopeReference::Path(_) => false,
                ScopeReference::Task(activity_token) => *activity_token == *token,
            },
        }
    }
}
