# persona-mind skill

Work here when the change concerns Persona's central typed state: roles,
claims, handoff tasks, activity, memory/work items, notes, dependencies,
aliases, ready-work projections, lock projections, or the `mind` CLI.

Rules for work here:

- Never model BEADS as exclusively locked. Any agent may write BEADS while it
  remains the transitional task substrate.
- Keep runtime message delivery in `persona-router`.
- Keep harness lifecycle in `persona-harness`.
- This component owns **its own** `persona-sema`-backed redb file (e.g.
  `mind.redb`). The mind state actor sequences writes through
  that database; no shared cross-component DB.
- Memory/work mutations append typed events; item state and ready-work lists are
  projections.
- Lock files are projections for human and cross-harness visibility,
  regenerated from the typed records on commit.
