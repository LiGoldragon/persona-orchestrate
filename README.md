# persona-mind

Central typed mind state for Persona agents.

This crate models role ownership, claimed scopes, handoff tasks, activity,
memory/work items, notes, dependencies, aliases, and ready-work views.

It is not Persona's runtime router or harness adapter. The current runtime is
actor-backed and in-process; durable `mind.redb` storage through
`persona-sema` is the storage target. Lock files are compatibility debris, not
durable truth for this crate.
