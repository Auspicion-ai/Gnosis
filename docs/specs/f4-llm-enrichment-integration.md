# F4-LLM — Automatic community / manual-enrichment pass via a harnessed LLM — FUTURE-INTEGRATION SPEC

- **Status:** **SPECULATIVE / FUTURE-INTEGRATION** (not a Gnosis-repo code unit; a
  suite-side integration surface). Drafted 2026-09-09 as the forward-looking
  complement to the F4 retrieval re-scope (`docs/specs/f4-community-summaries-review.md`,
  decision `F4-COMMUNITY-RETRIEVAL`).
- **Owner:** a later suite-integration unit (Astrographer shell, Familiar, or
  Emerald) that hosts a **harnessed LLM** (a text-generation provider with a
  controlled prompt/response contract). Gnosis itself has **no text-generation
  provider** (`EmbeddingProvider` is embed-only) and does **not** own this pass.
- **Revisit condition:** a suite tool with a harnessed LLM (Familiar's light-LLM
  candidate extraction, Astrographer's shell, or Emerald) lands a text-generation
  seam and wants to drive Gnosis's enrichment surfaces automatically.

---

## What this spec asks

When a suite tool that hosts a **harnessed LLM** (Familiar, Astrographer, Emerald)
is wired to Gnosis, it should be able to run an **automatic enrichment pass** that
drives Gnosis's existing manual-override surfaces — community declaration,
entity resolution, fact merging — using the LLM to *propose* enrichments that the
engine then applies through its **authoritative manual-override** paths. The LLM
proposes; Gnosis's manual-override invariants stay authoritative.

This is the **suite-side** half of the enrichment story that the F4/F6 re-scopes
explicitly parked: LLM-generated community summaries, entity-resolution
candidates, and relationship/property enrichment. It is **not** a Gnosis-repo
code unit — it is a contract for how a harnessed-LLM host drives Gnosis's surfaces.

## The core principle: LLM proposes, Gnosis's manual override stays authoritative

Gnosis's enrichment surfaces are all **manual-override-authoritative**:

- **`declareCommunity`** (§4.2.8) — the manual summary is authoritative and never
  overwritten by an automatic computation (§4.2.8.4).
- **`resolveEntities`** (§4.2.9.1) — each call is an **authoritative overwrite**
  (decision `RESOLVE-ENTITIES-AUTHORITATIVE-OVERWRITE`): the durable alias→canonical
  map re-stabilizes to an acyclic, flat alias graph; a later manual call supersedes
  an earlier one.
- **`mergeFacts`** (§4.2.9.1) — merges duplicate facts; the canonical value + union
  of citations; `ConflictError` on conflicting values with no resolution.
- **`updateCommunitySummary`** (§4.2.8.2) — the manual summary override.

The automatic LLM pass therefore **proposes** enrichments and applies them **as if
they were manual declarations** — through the same authoritative-override paths.
The engine cannot tell (and must not need to tell) whether a declaration came from
a human or an LLM host; both are "manual" in the §4.2.8.4 sense (neither is an
automatic computation *inside* the engine). This preserves the invariant by
construction: the LLM host is a *caller*, not an engine-internal automatic pass.

## The integration contract (what the harnessed-LLM host does)

A suite tool with a harnessed LLM drives Gnosis's surfaces over the existing
`RagStore` API (and, once the shell-integration unit lands, over the wire). The
pass has three phases:

### Phase 1 — Community enrichment (the F4 forward path)

1. The host reads the wiki's communities via `listCommunities` + `getCommunityContext`
   (the F4 retrieval accessor) to see what already exists.
2. The host's harnessed LLM **proposes** a community summary for a node set (or a
   new community declaration) from the member node values/snippets.
3. The host applies it via **`declareCommunity`** (new community) or
   **`updateCommunitySummary`** (refresh an existing manual summary) — the
   authoritative manual-override paths.
4. **Guardrail:** the LLM host must NOT call any engine-internal automatic pass
   (there is none); it is a caller. The engine's §4.2.8.4 invariant is preserved
   because the LLM host is a manual caller, not an engine computation.

### Phase 2 — Entity resolution (the graph-enrichments SHOULD)

1. The host's LLM proposes that two `fact`/entity nodes refer to the same
   real-world object (e.g. "license" vs "licence").
2. The host applies it via **`resolveEntities`** — the authoritative-overwrite
   path (decision `RESOLVE-ENTITIES-AUTHORITATIVE-OVERWRITE`): the chosen canonical
   becomes the root, aliases point directly to it, no residual alias, no cycle.
3. **Guardrail:** the LLM host must present a *complete* `entityIds` set + a
   `canonicalId`; the engine validates (empty → `ValidationError`, canonical not in
   set → `ValidationError`, unknown doc → `DocumentNotFound`). The host should
   re-read `entityAliasCanonical` after to confirm convergence.

### Phase 3 — Fact merging (dedup)

1. The host's LLM proposes that two `fact` keys are duplicates.
2. The host applies it via **`mergeFacts`** — the union-citations + canonical-value
   path. A merge with conflicting values and no resolution → `ConflictError`; the
   host must supply a resolution (a canonical value) or skip.

## The harnessed-LLM contract (what the host must provide)

The host's LLM must be **harnessed** — a controlled prompt/response contract, not a
free-form call. The spec pins the minimum:

- **A text-generation provider** with a defined request/response schema (the
  engine's `EmbeddingProvider` is embed-only and does NOT provide this; the host
  owns it).
- **A prompt template** per enrichment type (community-summary, entity-resolution,
  fact-merge) that takes the relevant Gnosis data (member node values, fact values,
  entity keys) and returns a **structured proposal** (JSON) the host can validate
  before applying.
- **A validation gate** in the host: the LLM's proposal must be schema-checked and
  the resulting Gnosis call must be legal (non-empty sets, canonical ∈ set, etc.)
  before it reaches the engine — the engine's `ValidationError`/`ConflictError`
  fail-states are the backstop, not the primary check.
- **A budget** (D2 cost/latency): the pass is ingestion-time and LLM-heavy; the
  host must budget token cost and latency. This is where the parked
  `EnrichmentBudgetExceeded` (FS-24) *would* become a real fail-state — but only in
  the host, not the engine (the engine keeps FS-24 RESERVED).

## What Gnosis provides (the surfaces the host drives)

- `listCommunities` / `getCommunityContext` (F4) / `getCommunity` — read the
  current community state.
- `declareCommunity` / `updateCommunitySummary` — apply community enrichments
  (authoritative manual override).
- `resolveEntities` / `entityAliasCanonical` — apply entity-resolution enrichments
  (authoritative overwrite).
- `mergeFacts` / `getFact` — apply fact-merge enrichments (union citations).
- `getQueryAuditLog` (§4.3.4) — the host can use the audit log to see what queries
  have been run (for enrichment targeting), though the F6 harness re-runs
  `rag_query` for quality measurement.

## What is explicitly OUT of scope (belted)

- **Not a Gnosis-repo code unit.** Gnosis does not add a text-generation provider,
  an automatic enrichment pass, or a `derived_summary` field. The engine's
  enrichment surfaces stay manual-override-authoritative; the LLM host is a caller.
- **No hierarchical community detection / global retrieval / full GraphRAG
  pipeline** — stays parked for D2 cost/latency (§7.5).
- **No `EnrichmentBudgetExceeded` (FS-24) in the engine** — stays RESERVED; the
  budget lives in the host.
- **No change to `re_derive_community` semantics** — stays "clear the STALE flag";
  the LLM host refreshes a summary via `updateCommunitySummary`, not re-derive.

## D1–D4 compliance

- **D1 (AGPL-3.0):** the enrichment algorithms (entity resolution, relationship
  enrichment, denoising) are well-known; the LLM host is a suite tool. No blocker.
- **D2 (local-first):** the harnessed LLM runs locally (Familiar's light-LLM,
  Astrographer's shell, Emerald); the pass is budgeted against ingestion latency
  and token cost. The community-summarization pass is the heaviest and least
  D2-friendly — the host must scope it (lazy, per-community, budgeted).
- **D3 (interconnection):** enrichment done once in the LLM host benefits the whole
  suite — Zodiac enriches its pre-graphs, Astrographer/Familiar consume enriched
  graphs. This is a clean D3 crossover.
- **D4 (MCP-GUI parity):** any enrichment feature must be reachable through both
  the GUI and the MCP surface. For Astrographer this means new MCP tools (e.g.
  `resolve_entities`, `enrich_graph`) alongside the existing set; for Zodiac, new
  tools alongside its §4.5 set. Enrichment is **not** a security-configuration
  feature, so it is **not** a D4 carve-out — it must be exposed on both surfaces.

## Revisit condition / how this unparks

This spec is **SPECULATIVE** until a suite tool with a harnessed LLM (Familiar,
Astrographer, or Emerald) lands a text-generation seam and wants to drive Gnosis's
enrichment surfaces. When that happens:

1. The host tool authors its own integration spec (its prompt templates, its
   validation gate, its budget) referencing this contract.
2. Gnosis's surfaces are already in place (the manual-override paths + the F4
   `getCommunityContext` retrieval accessor); no engine change is required.
3. If the host needs a wire surface, the shell-integration unit (the HTTP/native-IPC
   server + SSE client) must land first so the host can drive Gnosis over the wire.

## Cross-references

- `docs/specs/f4-community-summaries-review.md` + decision `F4-COMMUNITY-RETRIEVAL`
  (the F4 retrieval re-scope this spec complements).
- `docs/specs/6-f6-eval-review.md` + decision `F6-EVAL-RE-SCOPED` (the precedent:
  LLM-judge/answer-generation is suite-side, not Gnosis).
- `docs/research/2026-09-08-graph-rag-when-vectors/reports/graph-enrichments.md`
  (the enrichment SHOULD: entity resolution over `fact` nodes, relationship/property
  enrichment, community summaries parked).
- `docs/specs/gnosis.md` §4.2.8 (community model + manual-override invariant),
  §4.2.9 (entity resolution / dedup / graph enrichment), §4.4.3 (staleness),
  §7.5 (F4), §7.6 (F5).
- `docs/decisions.md` `RESOLVE-ENTITIES-AUTHORITATIVE-OVERWRITE`,
  `RESERVED-ERRVARIANTS-DISCIPLINE`.
