# persona-mind — architecture

*Kameo-backed central state machine for Persona coordination and memory.*

> Status: Phase 1 is `kameo`-backed and in-process. The runtime starts a
> `kameo` tree, routes typed `MindEnvelope` requests through named
> supervisor actors, and proves the path with manifest/trace tests. Durable
> `persona-sema` tables are the next storage substrate; current state is still
> held by in-memory mind reducers behind the mind-owned state/write path.

---

## 0 · TL;DR

`persona-mind` owns Persona's workspace coordination truth: role claims,
handoffs, activity, work/memory items, notes, dependencies, aliases, events,
and ready-work views. It consumes `signal-persona-mind`; it does not own
router delivery, harness lifecycle, terminal adapters, or the sema database
library.

All public operations enter as a typed `MindEnvelope { actor, request }`. The
actor field is infrastructure context supplied before persistence; request
payloads do not mint sender identity. The current runtime is short-lived and
in-process for tests and the future `mind` CLI, but it already uses the same
actor path a long-lived host can reuse.

```mermaid
flowchart LR
    caller["agent or CLI"] --> envelope["MindEnvelope"]
    envelope --> root["MindRoot"]
    root --> ingress["IngressSupervisor"]
    ingress --> dispatch["DispatchSupervisor"]
    dispatch --> domain["DomainSupervisor"]
    dispatch --> views["ViewSupervisor"]
    domain --> store["StoreSupervisor (mind state/write plane)"]
    views --> store
    store --> reducer["memory reducers"]
    store -. "next storage substrate" .-> sema["persona-sema"]
    sema -.-> db[("mind.redb")]
    dispatch --> reply["ReplySupervisor"]
```

## 1 · Component Surface

The crate exposes:

- `MindEnvelope` — caller identity plus one `MindRequest`.
- `MindRuntime` — in-process `kameo` facade used by tests and future CLI
  entry.
- `MindRuntime` — domain wrapper over the root `ActorRef`; `MindRoot` is the
  only bare Kameo spawn site.
- `actors::ActorManifest` — topology witness naming root, long-lived
  supervisors, and trace-phase actors.
- `actors::ActorTrace` — per-request witness proving which actor planes ran.
- `MemoryState` — current in-memory memory/work reducer owned by
  the mind state/write actor path.
- `ClaimState` — current in-memory claim reducer used by existing claim tests.

The `mind` binary is still a scaffold. It must become a one-NOTA-record input
to one-NOTA-record reply surface over the same `MindRuntime` path; it must not
grow a second command language.

## 2 · Runtime Topology

Phase 1 starts these supervised `kameo` actors:

```mermaid
flowchart TB
    root["MindRoot"]
    root --> config["Config"]
    root --> ingress["IngressSupervisor"]
    root --> dispatch["DispatchSupervisor"]
    root --> domain["DomainSupervisor"]
    root --> store["StoreSupervisor (mind state/write plane)"]
    root --> views["ViewSupervisor"]
    root --> subscriptions["SubscriptionSupervisor"]
    root --> reply["ReplySupervisor"]

    ingress --> request_session["RequestSession trace phase"]
    ingress --> identity["CallerIdentityResolver trace phase"]
    ingress --> envelope["EnvelopeBuilder trace phase"]

    dispatch --> memory_flow["MemoryFlow trace phase"]
    dispatch --> query_flow["QueryFlow trace phase"]

    domain --> memory_graph["MemoryGraphSupervisor trace phase"]
    memory_graph --> item_open["ItemOpen trace phase"]
    memory_graph --> note_add["NoteAdd trace phase"]
    memory_graph --> link["Link trace phase"]
    memory_graph --> status["StatusChange trace phase"]
    memory_graph --> alias["AliasAdd trace phase"]

    store --> read["SemaReader trace phase"]
    store --> writer["SemaWriter trace phase"]
    store --> id["IdMint trace phase"]
    store --> clock["Clock trace phase"]
    store --> append["EventAppender trace phase"]
    store --> commit["Commit trace phase"]

    views --> ready["ReadyWorkView trace phase"]
    views --> blocked["BlockedWorkView trace phase"]
    views --> recent["RecentActivityView trace phase"]

    reply --> encode["NotaReplyEncoder trace phase"]
    reply --> error["ErrorShaper trace phase"]
```

The long-lived supervisors are real actors today. The smaller operation planes
are trace-phase actors in Phase 1: they are explicit manifest entries and test
witnesses, and their boundaries are the names that future fine-grained Kameo
actors must preserve as persistence lands.

### 2.1 · Kameo Boundary

`persona-mind` uses `kameo` directly. No second actor abstraction is required
before persistence work proceeds. Long-lived actor structs carry state directly;
message types are per-verb records implemented through Kameo's `Message<T>`
trait.

The local `actors::manifest` and `actors::trace` modules are persona-mind
architecture witnesses. Keep them local until multiple real runtime crates
duplicate the same concrete API.

Kameo actors are data-bearing runtime nouns. Domain behavior belongs on those
actor structs, reducers owned by those actor structs, or domain wrappers.

## 3 · Request Paths

Memory mutations run through ingress, dispatch, domain, the mind state/write
plane, and reply:

```mermaid
sequenceDiagram
    participant Root as "MindRoot"
    participant Ingress as "IngressSupervisor"
    participant Dispatch as "DispatchSupervisor"
    participant Domain as "DomainSupervisor"
    participant Store as "StoreSupervisor"
    participant Reply as "ReplySupervisor"

    Root->>Ingress: MindEnvelope
    Ingress->>Dispatch: identity-checked envelope
    Dispatch->>Domain: memory mutation
    Domain->>Store: write intent
    Store-->>Domain: MindReply
    Domain-->>Dispatch: reply plus ActorTrace
    Dispatch->>Reply: shape reply
    Reply-->>Root: typed reply plus ActorTrace
```

Queries run through `ViewSupervisor` and `SemaReader` trace phases.
The query trace must not include `SemaWriter`; `tests/actor_topology.rs`
asserts that ready-work query is read-only by actor path, not by convention.

## 4 · State and Ownership

`StoreSupervisor` is the current internal name for the mind-owned
state/write plane. It is not a shared store component and does not imply a
`persona-store` repo. It owns `MemoryState`, which owns a private graph
reducer. The reducer appends typed `Event` values for memory/work mutations and
derives item, edge, note, alias, ready, blocked, and recent-event views from
that state.

`MemoryState::dispatch` remains as a reducer test facade. `MindRuntime` users
call `MindRuntime::submit`, which wraps the same reducer behind the actor
path. `MemoryState::dispatch_envelope` is the bridge that carries envelope
actor identity into event headers and note authorship.

The durable target is one workspace-local `mind.redb`, opened through
`persona-sema`. Only the mind state/write actor plane is allowed to commit
writes.
Queries use read snapshots and return typed views; they do not repair state
while answering.

## 5 · Boundaries

This repo owns:

- role claim and claim-overlap state;
- handoff and activity semantics as they land;
- work/memory graph reducers;
- actor topology, manifest, and traces for mind operations;
- the future `mind` CLI surface.

This repo does not own:

- `signal-persona-mind` contract records;
- `persona-router` delivery;
- `persona-harness` lifecycle;
- terminal or WezTerm state;
- `persona-sema` or `sema` internals;
- BEADS as a live backend.

## 6 · Invariants

- Every public operation enters as one `MindEnvelope`.
- Mind-supplied identity, sequence, time, and operation context are
  infrastructure concerns; request payloads carry content, not authority.
- The root actor is the only bare Kameo spawn site; children are supervised
  from `MindRoot`.
- No shared `Arc<Mutex<T>>` state exists between actors.
- Queries must not send write intents.
- Every memory/work mutation appends a typed event.
- BEADS is transitional import/history only, never an exclusive lock model.
- Lock files are not durable truth for this crate.
- Production code uses direct `kameo`; no second actor abstraction is a
  dependency or prerequisite.

## Code Map

```text
src/lib.rs                 crate surface
src/error.rs               typed Error enum and actor call errors
src/envelope.rs            MindEnvelope actor identity wrapper
src/service.rs             MindRuntime in-process actor facade
src/actors/mod.rs          actor module exports
src/actors/root.rs         MindRoot
src/actors/ingress.rs      ingress supervisor and envelope preparation trace
src/actors/dispatch.rs     request classification and flow selection
src/actors/domain.rs       memory mutation domain path
src/actors/store.rs        current state owner; reducer-backed read/write path
src/actors/view.rs         query/read-view path
src/actors/reply.rs        typed reply shaping path
src/actors/config.rs       store-path configuration actor
src/actors/subscription.rs post-commit push actor placeholder
src/actors/manifest.rs     actor topology manifest
src/actors/trace.rs        actor trace witness types
src/claim.rs               claim-scope reducer
src/memory.rs              memory/work graph reducer
src/role.rs                local role value
src/main.rs                scaffold CLI
tests/actor_topology.rs    manifest and actor-path truth tests
tests/weird_actor_truth.rs static actor-discipline and weird runtime tests
tests/memory.rs            memory/work reducer tests
tests/smoke.rs             claim reducer tests
```

## See Also

- `~/primary/reports/operator/101-persona-mind-full-architecture-proposal.md`
  — full actor-heavy target architecture.
- `~/primary/reports/operator-assistant/99-kameo-adoption-and-code-quality-audit.md`
  — Kameo adoption and code-quality audit for Persona migration.
- `~/primary/reports/designer/100-persona-mind-architecture-proposal.md`
  — concrete ID, envelope, database-path, and table-key decisions.
- `../signal-persona-mind/ARCHITECTURE.md` — request/reply contract.
- `../persona-sema/ARCHITECTURE.md` — typed table layer.
