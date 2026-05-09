# persona-orchestrate — architecture

*Typed workspace coordination for agents working in the Persona ecosystem.*

`persona-orchestrate` models the workspace coordination protocol as Rust state:
roles, claims, role-visible lock projections, and handoff tasks. It is the
typed successor to `~/primary/tools/orchestrate`.

---

## 0 · TL;DR

This repo coordinates agents and workspace scopes. It is not the Persona
runtime router, harness delivery engine, or main database owner.

```mermaid
flowchart LR
    "agent" -->|"claim command"| "OrchestrationState"
    "OrchestrationState" -->|"lock projection"| "role lock files"
    "OrchestrationState" -->|"handoff task"| "task projection"
    "OrchestrationState" -->|"orchestrate-owned state"| "persona-sema"
```

## 1 · Component Surface

`persona-orchestrate` exposes:

- role records;
- claim scope records;
- overlap checks;
- lock-file projections;
- handoff task records;
- an `orchestrate` CLI surface for isolated development.

## 2 · State and Ownership

This component owns workspace coordination state. While primary still uses
plain lock files, this repo models the typed replacement. BEADS remains a
transitional shared task substrate and is never modeled as an exclusive lock.

## 3 · Boundaries

This repo owns:

- agent role and claim state;
- claim/release command surfaces;
- workspace handoff tasks;
- projections compatible with the current orchestration protocol.

This repo does not own:

- runtime Persona delivery (`persona-router`);
- harness lifecycle (`persona-harness`);
- typed table mechanics (`persona-sema`);
- BEADS internals or BEADS exclusivity.

## 4 · Invariants

- Every agent knows its role before claiming.
- Claims prevent overlapping file ownership; BEADS is never claimed.
- Lock files are projections, not the source of durable typed truth.
- Open task checks are coordination visibility, not locking.

## Code Map

```text
src/role.rs   role records
src/claim.rs  claim scope records
src/main.rs   orchestrate CLI scaffold
tests/        orchestration smoke tests
```

## See Also

- `~/primary/protocols/orchestration.md`
- `../persona-sema/ARCHITECTURE.md`
- `../persona/ARCHITECTURE.md`
