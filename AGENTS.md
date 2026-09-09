# Gnosis — Agent Configuration

Context management and process guidelines for agents working in this
repository. **Gnosis** is the **production graph/vector engine** of the Auspicion
Suite — the fast/multithread-capable (Rust) backend that owns the document
store, the knowledge graph, fact/citation tracking, consistency enforcement, and
the RAG/agent-memory retrieval stack. It is the production replacement for the
archived **Incanter** prototype.

## Relationship to the suite

- Gnosis is a **pure backend**: it has **no MCP surface and no GUI surface**. It
  is consumed by the **Astrographer** Electron shell, which proxies engine calls
  over the `RagStore` + query/stream/engine-status API. Astrographer is Gnosis's
  front-end (D4 parity applies at the shell, not at Gnosis).
- Gnosis is a **separate project** running in a **faster/multithread-capable
  language** (Rust, per decision INCANTER-DISPOSITION / GNOSIS-ENGINE).
- The **canonical behavior contract** is `docs/specs/gnosis.md` (copied from the
  Auspicion Suite top-level architecture repo). This repo must honor that
  contract; a TestWriter derives every state and fail-state from it.

## The four design constraints (the suite's constitution)

These are the non-negotiable constraints every Auspicion Suite tool honors, and
Gnosis is subject to all four:

1. **Open-source first (AGPL-3.0).** Gnosis is provided freely under AGPL-3.0
   (D1).
2. **"If it can be local, it should be local."** Gnosis runs locally with no
   cloud dependency for core function; the only non-local runtime dependency is
   an optional local embedding/LLM provider (e.g. Ollama) for the
   embedding/rerank/compression/HyDE legs (D2).
3. **Interconnection as the selling point.** Gnosis both gains and provides
   utility when wired to Zodiac (a RAG data source), Solomon (cross-instance
   search), Astral (via the shell's push), and Familiar (knowledge retrieval +
   optional memory store) (D3).
4. **MCP-GUI feature parity.** Gnosis is a headless backend — D4 parity applies
   at the Astrographer shell, which provides the MCP + GUI surfaces that proxy
   Gnosis's API. Gnosis itself has no MCP/GUI surface (D4, applied at the shell).

## Key design principles (from `docs/specs/gnosis.md`)

- **Manual override for graph operations.** Non-vector operations in the graph
  space always have a manual access/override option (§4.2.8) — an agent/user can
  directly declare a community, a fact, a reference state, a segment, or a triple.
- **Single source of truth.** Triples, vector fields, communities, and all
  derived data (indexes, summaries) are owned by the graph (node/edge
  structures); any lookup index is a derived, rebuildable artifact, never a
  second source of truth (§4.2.7.2).
- **Multiple vector fields.** A node/chunk can carry multiple vector fields
  (full + binary for fast first-pass ANN) (§4.5.3a).

## Context budget rules

1. **75% threshold**: past 75% of available context, stop starting new work and
   switch to preparing handover documents so a fresh sub-agent can continue.
2. **50% task threshold**: a task estimated to take >50% of context is delegated
   to sub-agents, never done inline.
   **RCA-5**: a multi-unit deliverable must be delegated PER UNIT, not inlined as
   one pass.
3. **Handover must include a documentation-staleness review**: before any
   handover is reported complete, reconcile the active trackers
   (`docs/next-steps.md`, `docs/pending.md`, `docs/defects.md`,
   `docs/decisions.md`, `docs/HANDOFF.md`) and the relevant `docs/specs/*.md`
   against the ACTUAL repo/build state. Fix stale entries in the same pass.

## Process gates (per the Auspicion Suite AGENTS.md, adapted)

1. **Proposal-review gate** — a proposal that changes the Gnosis contract or the
   top-level architecture goes through validity ∥ critique → architecture review
   → change-analysis first, then lands as `docs/specs/<proposal>-review.md`
   before any code. Only a passing review PLUS the user's go-ahead may proceed.
2. **Spec gate** — a code unit is only delegable once its contract exists and the
   reviewer loop returned empty.
3. **TDD, always (red → green)** — a TestWriter writes tests from the spec ALONE
   (reports the failing red set), then the Implementer lands the least code to
   green.
4. **Adversarial gate** — role_adversarial_reviewer after each unit's green;
   host findings fixed here, package/foundation findings → defects.md/HANDOFF.md.
5. **Blind-greens + live-scenario + doc-review** after the greens.
6. **Trio gate** — `cargo test` (and any lint/build) must be green before
   reporting complete.

## Roles

The Auspicion Suite role set applies (SpecWriter, TestWriter, Implementer,
Adversarial reviewer, Blind-test writer, Proofreader, Documentation reviewer,
Live-scenario runner), all read-only unless they own the artifact they write.

## Reporting

When a pass is complete, report which gates passed, the red/green counts, the
adversarial findings, the blind-greens result, the doc-review pass, and the trio
result. Flag any regression before reporting completion.
