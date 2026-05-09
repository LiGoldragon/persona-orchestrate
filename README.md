# persona-mind

Central typed mind state for Persona agents.

This crate models role ownership, claimed scopes, handoff tasks, activity,
memory/work items, notes, dependencies, aliases, and ready-work views.

It is not Persona's runtime router or harness adapter. It owns `mind.redb`
through `persona-sema`; lock files and ready-work lists are projections of
typed mind state.
