# persona-orchestrate

Typed workspace orchestration state for Persona agents.

This crate models role ownership, claimed scopes, handoff tasks, and the typed
replacement for primary workspace lock files.

It is not Persona's runtime router, harness, or message database. Each
state-bearing Persona component owns its own Sema database through
`persona-sema`; this crate owns only collaborative workspace coordination
state.
