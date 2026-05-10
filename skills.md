# persona-mind skill

Work here when the change concerns Persona's central typed state: roles,
claims, handoff tasks, activity, memory/work items, notes, dependencies,
aliases, ready-work views, or the `mind` CLI.

Rules for work here:

- Never model BEADS as exclusively locked. Any agent may write BEADS while it
  remains the transitional task substrate.
- Keep runtime message delivery in `persona-router`.
- Keep harness lifecycle in `persona-harness`.
- This component owns **its own** mind Sema layer over the `sema` kernel and
  writes one `mind.redb`. The mind state actor sequences writes through that
  database; no shared cross-component DB.
- Memory/work mutations append typed events; item state and ready-work lists are
  projections.
- Lock files are outside the implementation target. They are temporary
  workspace coordination artifacts and should not be regenerated or projected
  by `persona-mind`.
- Runtime actors use direct `kameo`; do not add a second actor abstraction as a
  prerequisite for mind work.
