# persona-mind skill

Work here when the change concerns Persona's central typed state: roles,
claims, handoff tasks, activity, memory/work items, notes, dependencies,
aliases, ready-work views, compatibility lock state, or the `mind` CLI.

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
- Lock files are compatibility artifacts while the workspace migrates; they are
  not durable truth and should not be regenerated as the long-term interface.
- Runtime actors use direct `kameo`; do not add a second actor abstraction as a
  prerequisite for mind work.
