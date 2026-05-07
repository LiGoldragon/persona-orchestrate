# Persona Orchestrate Architecture

`persona-orchestrate` will replace the ad hoc lock helper with typed workspace
coordination state.

```mermaid
flowchart LR
  "agent" -->|"claim command"| "OrchestrationState"
  "OrchestrationState" -->|"role-visible views"| "lock projections"
  "OrchestrationState" -->|"handoff tasks"| "task projections"
  "OrchestrationState" -->|"workspace transition"| "persona-store"
```

The current primary workspace protocol remains the source of truth until this
crate is ready to take over.

This crate does not own harness delivery or the global Persona reducer. It
coordinates workspace claims and handoffs, then records typed transitions through
the store when that daemon exists.
