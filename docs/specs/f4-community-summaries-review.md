# F4 — Community summaries — PROPOSAL-REVIEW record

- **Unit:** §7.5 F4 — Community summaries (SHOULD-HAVE; the full hierarchical GraphRAG pipeline is PARKED for D2 cost/latency).
- **Gate:** proposal-review gate (validity ∥ critique → architecture → change-analysis).
- **Date:** 2026-09-09.
- **Verdict:** **PASS — RE-SCOPED TO A RETRIEVAL SURFACE** (contingent on the user's go-ahead). The **derivation** surface is **invalid** (vacuous + redundant); the **retrieval** surface is a valid, delegable Gnosis-repo unit.
- **Status:** proposal approved by review (re-scoped to retrieval); code/contract delegation awaits the user's go-ahead.

## What F4 is (as stated) and why it was re-scoped

§7.5 frames F4 as a "scoped, lazy, local community-summary surface" (the D2-compatible
SHOULD), distinct from the parked full hierarchical GraphRAG pipeline. The
proposal-review found the **derivation** reading of this **invalid** on two facts:

1. **Vacuous:** `declareCommunity` REQUIRES a non-empty manual summary
   (`src/store/mod.rs:3159`), so every community already carries an authoritative
   manual summary — there is never a "no summary to fill in" state. A deterministic
   derivation over a community that already has a manual summary produces a second
   text with **no consumer**.
2. **Redundant:** a deterministic concatenation of member node values/snippets
   (mirroring `get_profile_summary`) is a **dump, not a summary**, and duplicates what
   `rag_query` graph mode already returns. It adds no retrieval capability and does
   not deliver §7.5's "answers global/theme questions" benefit (which needs the
   parked hierarchical + LLM + global pipeline).

## The re-scoped F4 (the approved deliverable)

A **retrieval surface** — the non-vacuous, non-redundant half of the §7.5 SHOULD:

- **`getCommunityContext(communityId) → CommunityContext`** — a new read-only
  `RagStore` trait method returning `{community_id, wiki_id, summary, members,
  state}` (the authoritative manual summary + the member node set + the current
  `CommunityState`), reusing stored data. Fail-state: **`CommunityNotFound` only**
  (a `communityId`-keyed accessor derives the wiki from the community record, so
  `WikiNotFound` cannot fire — matching `get_community`'s existing FS-25).
- **`src/store/mod.rs`** — `CommunityContext` struct + `get_community_context` impl
  (read `communities` + `community_states`, join). Re-exported from `src/lib.rs`.
- **`tests/community_context_integration.rs`** + **`tests/props_community_context.rs`**
  — the TestWriter red set + PBT layer.
- **`docs/specs/4-2-graph-property-register.md`** (extend) — PBT rows authored by the
  spec-writer.

**Semantics:** a pre-joined read of `get_community` + `community_state`; no derivation,
no generation, no mutation; side-effect-free (no journal append, no state change);
deterministic (equal store state → equal `CommunityContext`).

**Guardrails (preserved by construction):**
- **§4.2.8.4** — the manual summary is the single source of truth and is never
  overwritten; the retrieval surface only *reads* it, so the invariant holds.
- **§4.4.3** staleness/publish gate — unchanged; `re_derive_community` keeps its
  current "clear the STALE flag" semantics (automatic regeneration stays parked).
- **`get_community`** stays side-effect-free.
- **FS-24 `EnrichmentBudgetExceeded`** stays **RESERVED** (no budget, no enrichment
  pass, no fabrication — per RESERVED-ERRVARIANTS-DISCIPLINE).
- **Zero new runtime deps**; **PBT-GATE-MANDATORY applies**.

**Contract change:** a new `RagStore` trait method is a contract change (this gate
adjudicates it). It does **NOT** extend the F2 wire contract (`engine-wire-contract.md`
covers only the §4.6.1 retrieval trio + health + audit); the wire codec for the
community surface is owned by the later shell-integration unit.

## Corrected unpark criterion

F4's §7.5 **global/theme benefit remains parked** (needs the hierarchical + LLM +
global pipeline). Only the **retrieval half** is unparked: a `getCommunityContext`
accessor that returns the pre-joined community unit (manual summary + members +
state) that `rag_query` graph mode does not provide. The marginal value is modest (it
fuses two existing reads) but the unit is cheap, well-bounded, and honest.

## Suite-side / parked (NOT Gnosis)

- LLM-generated community summaries (needs a **text-generation provider** — a NEW
  seam; `EmbeddingProvider` is embed-only, so this is not "already a dep").
- Hierarchical community detection; global/theme retrieval (the §7.5 benefit); the
  full GraphRAG pipeline.
- Any `derived_summary` field or `deriveCommunitySummary`/propose-only API (vacuous +
  redundant — refuted).
- Any change to `re_derive_community` semantics.
- The wire codec for the community surface (deferred to the shell-integration unit).
- FS-24 `EnrichmentBudgetExceeded` (stays RESERVED).

## Residual risks / open items (told to the user before approval)

- **Marginal value:** `getCommunityContext` fuses two existing reads; it is a
  convenience accessor, not a new capability, and does not deliver §7.5's
  global/theme benefit.
- **Contract change:** a new `RagStore` trait method (this gate adjudicates it); the
  wire codec is deferred to the shell-integration unit.
- **Fail-state:** only `CommunityNotFound` (the architecture's `WikiNotFound` is
  dropped — cannot fire on a `communityId`-keyed accessor).

## Next steps (after user go-ahead, gate order)

1. **SpecWriter** authors the F4 contract (compile-horizon format) + the PBT register
   rows (extend `docs/specs/4-2-graph-property-register.md`);
2. reviewer loop → empty;
3. **TestWriter** writes `community_context_integration.rs` + `props_community_context.rs`
   from the spec alone → red;
4. **Implementer** lands `get_community_context` → green;
5. adversarial → blind-greens → trio green.
