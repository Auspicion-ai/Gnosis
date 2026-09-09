# Agent Memory (3-Layer Memory, Candidate Facts, Per-Exchange Chunking) — Research Notes

**Topic:** Agent-memory architecture — 3-layer memory (profile summary / facts table / vector search), candidate-fact extraction + deterministic validation, per-exchange chunking
**Date:** 2026-09-07
**Status:** Research / reference notes (not a behavior contract)
**Scope:** General technique + Auspicion Suite application analysis, with **primary focus on Familiar's memory store** (`docs/specs/familiar.md` §4.1.3) and its integrations
**Source:** `/media/ryanr/Boot/Tech Talks/Commit Your Code/Building Better Agent Memory.md` (local notes, 24 lines)

---

## 1. Source notes

| Item | Source |
| --- | --- |
| **Source notes (local file)** | `/media/ryanr/Boot/Tech Talks/Commit Your Code/Building Better Agent Memory.md` |
| **Mem0** | "Supposed to extract information from chat logs. Severe knowledge compression." |
| **Vectorization issue** | Embedding a long vector creates a "blurry image" (no precise relation); the agent takes too long to parse a passed response transcript. **Per-exchange chunking** creates many more precise vectors. |
| **3-layer memory** | **Profile Summary** (executive summary about what the agent knows about the user, drafted from the facts table) · **Facts table** (tracks every individual fact about the user) · **Vector Search** (every turn, embed what was said, run cosine distance, find the closest chunks under a similarity threshold, inject into the prompt). |
| **Candidate Fact** | Use a light LLM to extract candidate facts; deterministic fact validation using TypeScript. |
| **RecallMEM** | `npx recallmem` (a tool). |

**Origin / status.** The source is a terse set of personal notes (a talk summary), not a peer-reviewed paper. The techniques are a well-known agent-memory pattern (Mem0-style memory, layered memory stores, embedding-based retrieval). The notes are treated as **design guidance**, not a canonical spec; each technique is evaluated for Auspicion fit below.

---

## 1a. Web-grounded research (bulk-research run, 2026-09-08)

The source notes were run through the `@agent-harness/bulk-research` workflow
(P0→P7), producing **12 web-grounded research reports** under
`docs/research/2026-09-08-agent-memory/reports/` (each `.md` + `.json`, each
citing 5–16 URLs). The reports corroborate and deepen the source techniques and
add several findings that refine the recommendation tiers below. Key findings:

| Report | Key finding relevant to the suite |
| --- | --- |
| `mem0-chat-extraction` | Mem0's two-phase pipeline (LLM extract → reconcile) is the reference pattern; it is model/vector-store agnostic and local-first. **Licensing:** Mem0 core is **Apache 2.0, not AGPL** — a D1 licensing decision if the suite vendors it. |
| `mem0-knowledge-compression` | Mem0's default `infer=True` is **lossy by design** (drops exact phrasing, numbers, constraints, decision reasoning, cross-turn deps, implicit prefs). Use `infer=False` or store raw in metadata when verbatim matters. Phase-1 embedding defect (#5148) on long/multi-topic conversations. |
| `vectorization-long-vector` | Single long-vector embedding is a known failure mode ("embedding collapse", "second-order collapse", "semantic shift"). **Chunk-then-embed** is the standard mitigation. |
| `vectorization-relation-loss` | Single long-vector cannot capture precise fact-to-fact / exchange-to-exchange relationships (entity-swap paradox, bag-of-words behavior). Needs finer chunking, multi-vector/late-interaction, or **graph-structured indexing**. |
| `vectorization-chunking` | **Per-exchange chunking** produces many precise vectors and preserves the question→answer relationship. Recover cross-exchange context at retrieval (neighbor return, hybrid fusion, aggregation). Late chunking as an enhancement. |
| `vectorization-token-cost` | Full-transcript passing is **quadratic cost + growing latency**. Use active-window + vectorized retrieval + masking/summarization. Familiar (highest turn volume) is most exposed. |
| `memory-facts-table` | Facts table = **discrete, individually addressable records** (`fact_id`, content, provenance, confidence, status). CRUD by id, never rewrite whole memory. "Facts as first-class objects." |
| `memory-profile-summary` | Profile summary is **DERIVED from the facts table** (a projection, single source of truth, no drift). Regeneration triggers (event-driven or lazy). Keep in the always-injected context layer. |
| `memory-vector-search` | Third layer: embed each turn, cosine distance, inject above threshold. **Threshold does not transfer across embedding models** — must be calibrated per model. Local embedding store (D2), NOT delegated to Astrographer/Incanter (MUST-2). Parked until the facts-table + profile-summary core ships. |
| `candidate-fact-extraction` | Two-stage pipeline: **light LLM proposes** candidate facts (structured output), **deterministic TS validation confirms/rejects**. "LLM proposes, deterministic code disposes." |
| `candidate-fact-validation` | Deterministic validation gate: structural (schema), semantic/grounding (provenance), deterministic post-processing (dedup, referential integrity, cross-field consistency). **Fail-closed**, machine-actionable rejection reasons. |
| `recallmem-cli` | A CLI (`npx recallmem`) as the query/manage gateway over facts-table + vector-search, with hybrid retrieval. MCP-GUI parity (D4). |

**Net refinement to the recommendation tiers (§5):** the web-grounded research
confirms the MUST/SHOULD/PARKED split and adds precision:
- **Facts table** should be **individually addressable records** (not a flat
  key→value store) with provenance + confidence + status — "facts as first-class
  objects" (`memory-facts-table`).
- **Candidate-fact pipeline** = light LLM proposes (structured output) +
  **deterministic TS validation** (schema + grounding + dedup, fail-closed)
  (`candidate-fact-extraction`, `candidate-fact-validation`).
- **Profile summary** is a **derived projection** of the facts table (single
  source of truth, no drift), regenerated on fact change (`memory-profile-summary`).
- **Vector search** uses **per-exchange chunking** + a **local embedding store**
  (D2), with **threshold calibration per model**; it is the parked layer
  (`vectorization-chunking`, `memory-vector-search`).
- **Mem0 is DISCARDED as the extraction engine** (lossy compression, Apache-2.0
  licensing) in favor of the candidate-fact + deterministic-validation pattern;
  if Mem0 is ever used, `infer=False` or raw-in-metadata is required for
  verbatim-critical facts (`mem0-knowledge-compression`).
- **RecallMEM is DISCARDED** as an external dependency; the suite's own memory
  surface (Familiar's MCP/GUI) is the query/manage gateway (`recallmem-cli`).

---

## 2. Summary of the source techniques

1. **Mem0** — extract information from chat logs into a memory store. The notes flag **severe knowledge compression** as a downside: extracting from chat logs loses precision. This is a caution, not a recommendation.
2. **Vectorization issue + per-exchange chunking** — embedding a long vector (a whole conversation, a long transcript) produces a "blurry image": the vector averages over many distinct facts and loses precise relations, and the agent is slow to parse a passed response transcript. **Per-exchange chunking** (embed each user↔assistant exchange as its own vector) yields many more precise vectors. This is the core retrieval-quality insight.
3. **3-layer memory**:
   - **Profile Summary** — an executive summary of what the agent knows about the user, **drafted from the facts table** (a derived artifact, not a source of truth).
   - **Facts table** — tracks every individual fact about the user (the source of truth for discrete facts).
   - **Vector Search** — every turn, embed what was said, run cosine distance, find the closest chunks under a similarity threshold, inject into the prompt (semantic recall over the fact/note corpus).
4. **Candidate Fact** — a **light LLM** extracts candidate facts from a turn; **deterministic validation** (TypeScript) confirms them before they are committed to the facts table. This separates the fuzzy extraction step (LLM) from the deterministic commit step (code), so only validated facts persist.
5. **RecallMEM** — `npx recallmem`, a third-party CLI tool. Not evaluated for suite fit (external dependency, license unverified).

---

## 3. Per-tool application analysis

### 3.1 Familiar's memory store (§4.1.3) — PRIMARY application

**Current contract.** `docs/specs/familiar.md` §4.1.3 pins a **small local store of assistant-scoped facts/notes** — a simple key→value store: `MemoryEntry = { key, value, updatedAt }` with `storeMemory` / `getMemory` / `listMemory` / `deleteMemory`. It is **distinct from knowledge memory**, which delegates to Astrographer's document store (MUST-2, decision FAMILIAR-ASSISTANT-CORE-SCOPE). The memory store holds small, non-document entries (user preferences, short notes).

**How the 3-layer model maps onto it.**

- **Facts table = the key→value store, upgraded.** The key→value store *is* a facts table: each `key` is a fact key, each `value` a fact value. The upgrade is to add the **candidate-fact pipeline** from the notes: a light LLM extracts candidate facts from conversation turns (§4.1.1), and **deterministic TypeScript validation** confirms them before they are committed via `storeMemory`. This is the "smart" part of the memory store and the highest-value increment. The facts table **complements, not replaces**, the key→value store — it is the same backing store, given a structured fact model (factKey, value, source, validated flag, updatedAt).
- **Profile summary = a derived artifact.** An executive summary drafted from the facts table. It is **not a source of truth** — it is regenerated on demand (or on fact-table change) from the facts table. Recommended home: a **reserved key in the memory store** (e.g. `_profile_summary`) or a small derived store, kept D2-local. It is the artifact that steers orchestration (§3.3).
- **Vector search = the parked/speculative layer.** Every turn, embed what was said, run cosine distance, find the closest chunks under a similarity threshold, inject into the prompt. This requires a **local embedding store** (D2). It is the heaviest lift and is parked until the facts-table + profile-summary core ships.

**Does vector search over memory delegate to Astrographer/Incanter?** **No.** The memory store is **distinct from knowledge memory** (MUST-2, decision FAMILIAR-ASSISTANT-CORE-SCOPE). Delegating memory vector search to Astrographer's document store or Incanter would conflate the two and violate MUST-2. If vector search is pursued, it must be a **local embedding store inside Familiar** (mirroring Incanter's local-Ollama approach, `incanter.md` §4.7), not a delegation to the knowledge-memory tools. This is a deliberate D2 boundary.

**Minimal D2-compliant upgrade path (recommended):**
1. **Facts table** — keep the key→value store as the backing store; add a structured fact model + the candidate-fact extraction + deterministic-validation pipeline. **MUST HAVE.**
2. **Profile summary** — a derived artifact regenerated from the facts table; reserved key or small derived store. **SHOULD HAVE.**
3. **Vector search** — a local embedding store + per-exchange chunking over conversation turns; **PARKED** until 1–2 ship. **NICE TO HAVE / PARKED.**

**Spec change vs deferred.** Items 1–2 are a **proposed spec change** to `docs/specs/familiar.md` §4.1.3 (a separate spec-gate unit — this research note does NOT rewrite the spec). Item 3 (vector search) and the Augur/Horoscope/Solomon applications (§3.4–§3.7) are **deferred/parked** and tracked in `docs/pending.md`.

### 3.2 Familiar's conversation model (§4.1.1)

The notes' **per-exchange chunking** maps directly onto the conversation model. Instead of embedding a whole conversation (or a long transcript) as one "blurry" vector, Familiar should chunk **per exchange** (a user message + its assistant reply) and embed each exchange as its own precise vector. This is the retrieval-quality fix for the vector-search layer (§3.1 item 3): precise per-exchange vectors, not one averaged long vector.

This is a **conversation-model enhancement** (the conversation would need a per-exchange chunk/embedding index). It is a **spec change** to §4.1.1 and is **deferred** until the vector-search layer is pursued. The conversation model itself (§4.1.1) is unchanged for the facts-table + profile-summary core.

### 3.3 Familiar's orchestration (§4.2)

The **profile summary** is the natural steering input to the orchestration loop. In §4.2 step 2 (Plan), Familiar maps a request to a domain delegation edge (organization→F2, knowledge→F1, research→F1 RAG, coordination→F6, cross-instance→F7). The profile summary (e.g. "user prefers Astrographer for knowledge, keeps tasks in Horoscope") can **bias the edge mapping** and the delegation context. This is low-cost (inject the derived summary into the planning context) and high-value (memory informs delegation, the D3 harness role).

This is a **spec change** to §4.2 (the Plan step consults the profile summary) and is **SHOULD HAVE** (it ships with the profile summary, §3.1 item 2).

### 3.4 Astrographer / Incanter (knowledge memory)

The notes' vectorization + chunking guidance maps onto Astrographer's RAG, but **no new Incanter/Astrographer change is required by this research**:

- **Per-fact-node chunking** (the notes' per-exchange chunking, applied to facts) is **already recommended** in Pass 8 research (`docs/research/hybrid-search-research-notes.md` §7, `graph-rag-deep-dive/`): graph-structure chunking per-fact-node, fact-key exact-match retrieval. The notes corroborate this: precise per-unit vectors beat one long "blurry" vector.
- **The 3-layer memory model maps conceptually onto Astrographer's document store:** the **facts table** = Astrographer's `fact` nodes (single source of truth, `astrographer.md` §4.2.1); the **profile summary** = a derived document; **vector search** = Astrographer's RAG surface (`astrographer.md` §4.3). This is a **conceptual mapping only** — Astrographer is knowledge memory, distinct from Familiar's assistant memory store (MUST-2). No Astrographer/Incanter contract change is needed for Familiar's memory store.

### 3.5 Horoscope (task memory)

The facts table / profile summary could inform Horoscope's task grading (§4.3.1). **But Familiar delegates execution to Horoscope** (MUST-1, decision FAMILIAR-AGENT-EXECUTION-OWNERSHIP), so this is a **Horoscope-side application** — and the cleanest path is **Familiar injecting the relevant profile-summary context into the delegation request** to Horoscope, **not** a Horoscope contract change. This keeps the memory in Familiar (its owner) and the execution in Horoscope (its owner), consistent with the delegation boundary.

This is **deferred/parked** (a cross-tool enhancement, not a core requirement) and is tracked in `docs/pending.md`. No Horoscope source change is required for the recommended path.

### 3.6 Solomon (cross-instance assistant memory, F7)

F7 pins "assistant memory" as a searchable instance type (MUST-5, decision FAMILIAR-SOLOMON-CONTRACT-CHANGE; `solomon.md` §4.2/§4.5.1/FS-6). The 3-layer model **clarifies what "assistant memory" means as a searchable type**:

- **Facts table** — the primary searchable unit: discrete key→value facts are the natural `solomon_search` result items (`{ peerId, instanceType: 'familiar', itemId, snippet }`).
- **Profile summary** — a secondary derived document, searchable as a summary.
- **Vector store** — the embedding index; searchable by similarity, but it is the parked layer.

So "assistant memory" as a searchable instance type = **the facts table (primary) + the profile summary (derived)**. This is a **refinement to the already-pinned Solomon contract change** (F7), not a new change. It is a handoff note to the Solomon contract (see `docs/HANDOFF.md` Round 3).

### 3.7 Augur (memory events, F6)

Memory updates should propagate as Augur events for cross-tool coordination. A **`familiar.fact.changed`** (or `familiar.memory.updated`) event on a `familiar.*` channel would let other tools react to memory changes (e.g. Horoscope re-grading a task, Solomon re-indexing the searchable facts). This is consistent with F6 (Familiar→Augur coordination edge, `familiar.md` §4.3.4).

The event schema is **provisional and owned by the Augur contract** (MUST-8, `familiar.md` §4.3.4/§7.3). So this is **deferred/parked** until the Augur schema finalizes. It is a **spec change** to the provisional `familiar.*` event set, tracked in `docs/pending.md`.

---

## 4. D1–D4 compliance check

| Recommendation | D1 (AGPL-3.0) | D2 (local-first) | D3 (interconnection) | D4 (MCP-GUI parity) |
| --- | --- | --- | --- | --- |
| **Facts table + candidate-fact extraction + deterministic validation** (§3.1) | ✅ — a local algorithm + a light local LLM; no proprietary service | ✅ — runs fully local; the light LLM is a local model (mirrors Incanter's local Ollama) | ✅ — the facts table is the searchable surface for F7 (Solomon) and the source for F6 (Augur) events | ✅ — `storeMemory`/`getMemory`/`listMemory`/`deleteMemory` already have GUI + MCP forms (`familiar.md` §4.4/§4.5); the fact model is a data-shape change, not a new surface |
| **Profile summary** (§3.1, §3.3) | ✅ — derived locally | ✅ — regenerated on demand from the local facts table | ✅ — steers orchestration (D3 harness role) and is a searchable derived document for F7 | ✅ — a derived artifact read via the existing memory surface; no new carve-out |
| **Vector search over memory** (§3.1, §3.2) | ✅ — local embedding store | ✅ — local embedding store inside Familiar (NOT delegated to Astrographer/Incanter, preserving MUST-2) | ✅ — per-exchange vectors feed F7 search | ✅ — a retrieval-mode change, not a security feature; must be GUI + MCP |
| **Augur memory events** (§3.7) | ✅ | ✅ — local event transport (Augur is local, `augur.md` §4.4) | ✅ — the F6 coordination edge | ✅ — publish/subscribe via Augur's MCP tools |
| **Memory-informed Horoscope grading** (§3.5) | ✅ | ✅ — Familiar injects local context into the delegation | ✅ — the D3 harness role | ✅ — delegation is via Horoscope's existing surface |
| **DISCARD: Mem0** | — | ❌ — the notes flag severe knowledge compression; not recommended | — | — |
| **DISCARD: RecallMEM** | ❌ — external tool, license unverified | ❌ — external dependency, not local-first | — | — |

**Net:** every recommended increment is D1/D2/D3/D4-compliant. The only D2-sensitive decision is **vector search over memory must be a local embedding store inside Familiar**, not a delegation to Astrographer/Incanter — this preserves the memory-store ≠ document-store distinction (MUST-2).

---

## 5. Recommendation tier

### MUST HAVE (Familiar memory store — proposed spec change to §4.1.3)
- **Facts table = the key→value store, upgraded with candidate-fact extraction + deterministic validation.** Keep the key→value store as the backing store; add a structured fact model (factKey, value, source, validated flag, updatedAt) and the pipeline: a light LLM extracts candidate facts from conversation turns (§4.1.1), deterministic TypeScript validation confirms them, then `storeMemory` commits only validated facts. This is the core value of the memory store and is fully D2-compliant.

### SHOULD HAVE (Familiar memory store + orchestration — proposed spec change to §4.1.3/§4.2)
- **Profile summary** — a derived executive summary regenerated from the facts table (reserved key or small derived store). Injected into the orchestration Plan step (§4.2) to steer domain-edge selection. Low cost, high value.

### NICE TO HAVE / PARKED (deferred — tracked in `docs/pending.md`)
- **Vector search over memory** — a local embedding store inside Familiar + per-exchange chunking over conversation turns (§4.1.1). Park until the facts-table + profile-summary core ships. When pursued, it must be a local embedding store (NOT delegated to Astrographer/Incanter, preserving MUST-2).
- **Augur memory events** — `familiar.fact.changed` on a `familiar.*` channel for cross-tool coordination. Park until the Augur event schema finalizes (MUST-8).
- **Memory-informed Horoscope grading** — Familiar injects the profile summary into the delegation request to Horoscope. Park as a cross-tool enhancement; no Horoscope source change required for the recommended path.

### DISCARD
- **Mem0** — the notes themselves flag severe knowledge compression; not recommended for the suite.
- **RecallMEM** (`npx recallmem`) — an external third-party tool; license unverified and not local-first (D2/D1). Not recommended.

---

## 6. Source

- `/media/ryanr/Boot/Tech Talks/Commit Your Code/Building Better Agent Memory.md` (local source notes, 24 lines — the sole source for this research note).
- Suite context cited throughout: `docs/specs/familiar.md` (§4.1.1, §4.1.3, §4.2, §4.3.4, §4.3.5, §4.4, §4.5), `docs/specs/astrographer.md` (§4.1, §4.2, §4.3), `docs/specs/incanter.md` (§4.3, §4.6, §4.7, §4.8), `docs/specs/horoscope.md` (§4.2, §4.3), `docs/specs/solomon.md` (§4.2, §4.5.1, FS-6), `docs/specs/augur.md` (§4.1, §4.2, §4.4, §4.5), `docs/decisions.md` (D1–D4, FAMILIAR-*), `docs/pending.md` (#2, #13, S1), `docs/research/hybrid-search-research-notes.md` (§7), `docs/research/lightrag-research-notes.md`.
