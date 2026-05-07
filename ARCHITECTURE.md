# Persona Orchestrate Architecture

`persona-orchestrate` will replace the ad hoc lock helper with typed state.

```mermaid
flowchart LR
  Agent[agent] --> Claim[ClaimCommand]
  Claim --> State[OrchestrationState]
  State --> Locks[role-visible lock views]
  State --> Tasks[handoff tasks]
  Tasks --> Router[persona-router]
```

The current primary workspace protocol remains the source of truth until this
crate is ready to take over.
