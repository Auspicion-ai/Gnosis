# Emerald — graph editing round-trip

**Edge:** Emerald ↔ Gnosis (`docs/specs/gnosis.md` §5.5).

## Nature

Emerald (the web development app) edits/builds on Gnosis-backed documentation
graphs, and Gnosis's graphs are editable in Emerald.

## Mechanism

The graph-push format is the canonical `provident-graph/1` contract (decision
GRAPH-PUSH-FORMAT, §4.4 of the shell spec). Emerald edits return as graph
updates; optimistic concurrency (`ConflictError` on a stale `revision`) guards
concurrent edits.

## Value

Emerald is a design/editing surface for Gnosis-backed documentation graphs;
Gnosis's graphs are editable in Emerald without a second graph-push producer
(D3).

## Fail-states

`ConflictError` on concurrent edits (§4.1.4); `UnresolvedReference` if an edit
introduces a `BROKEN`/`STALE` reference; `SchemaMismatch` if Emerald uses an
incompatible graph schema.
