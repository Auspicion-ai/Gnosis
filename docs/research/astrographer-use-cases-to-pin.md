# Astrographer — Use Cases to Pin (Preexisting Design Cases + Research-Derived Feature Requests)

- **Date:** 2026-09-08
- **Status:** Analysis / reference (a synthesis of the use cases that must be pinned as contract elements for the Astrographer project). Not a behavior contract.
- **Purpose:** enumerate every use case that needs to be pinned for Astrographer, drawn from (a) the preexisting design cases in the Auspicion Suite canonical contract `docs/specs/astrographer.md` and (b) the feature requests developed from the research notes (Pass 8 Advanced-RAG, Pass 12 docs-as-product, Pass 13 graph-RAG) plus the Astrographer project's own parked/speculative items.
- **"Pinned"** = captured as a concrete contract element (a method/API signature, return shape, throw pattern, happy-path state, or fail-state) that a TestWriter can derive states from.

---

## Part A — Preexisting design cases (the core contract baseline, already pinned in `docs/specs/astrographer.md`)

These are the design cases that define Astrographer as a **document store + graph RAG hybrid**. They are already pinned in the canonical contract; they are the baseline every research-derived feature request builds on.

### A1. Document store / wiki model (§4.1)
- **Document lifecycle:** create / get / update / delete / publish / unpublish / archive a Document (Provident graph), with the `DRAFT → PUBLISHED → ARCHIVED` state machine.
- **Wiki model:** named collections of Documents; cross-wiki references flagged.
- **Optimistic concurrency:** `updateDocument` requires a base `revision`; a stale base → `ConflictError` (HTTP 409 / MCP error).
- **Store operations:** `createDocument`/`getDocument`/`updateDocument`/`deleteDocument`/`publishDocument`/`unpublishDocument`/`archiveDocument`/`listDocuments`/`createWiki`/`getWiki`/`listWikis`.

### A2. Cross-link / data-embed consistency (§4.2) — the defining feature
- **Node kinds:** `content` / `fact` / `reference`.
- **Reference modes:** `link` (live resolution) / `embed` (snapshot).
- **Consistency invariant:** a `fact`'s canonical `value` is the single source of truth; every `embed` snapshot and `link` must stay consistent.
- **Consistency enforcement:** `STALE`/`FRESH` embeds, `BROKEN`/`RESOLVED` links, re-sync, the publish gate (`UnresolvedReference` on `BROKEN`/`STALE`), `getConsistencyReport(wikiId)`.
- **Reference integrity on delete:** `DocumentInUse` if another document references the target.

### A3. RAG query surface (§4.3)
- **Engine connection state:** `CONNECTED`/`CONNECTING`/`UNAVAILABLE`/`DISCONNECTED`; the engine is optional (D2).
- **RAG operations:** `ragQuery`/`ragStream`/`getEngineStatus`, with the `source: 'local'|'incanter'|'zodiac'` union and the `engine: 'incanter'` framing.
- **RAG fail-states:** `EngineUnavailable` vs `EngineError`.

### A4. Provident-graph push to Astral (§4.4)
- **Push format:** `provident-graph/1` canonical schema with idempotency `(unitId, revision, contentHash)` and an honored `state` field.
- **Push operations:** `pushToAstral`/`getPushStatus`; auto-publish; `PushRejected`/`AstralUnavailable`/`SchemaMismatch`.

### A5. MCP surface (§4.5) — D4 parity
- **MCP tool set:** the 17-tool provisional set (document/wiki/consistency/RAG/push/engine tools).
- **MCP error mapping** and the **D4 parity rule** (every non-security feature reachable through both GUI and MCP).
- **Security-configuration carve-out:** engine credentials, Astral push credentials, TLS/secret management, Firmament bridge auth are GUI-only.

### A6. GUI surface (§4.6) — parity
- **GUI feature set** and the **parity mapping** (each GUI screen ↔ an MCP tool).
- **GUI fail-state presentation** (distinct `EngineUnavailable` vs `EngineError`, `PushRejected` vs `AstralUnavailable`, `ConflictError`, `UnresolvedReference`).

### A7. Integrations (§5)
- **Zodiac → Astrographer (edge #1):** non-local RAG data; degrade-to-local on Zodiac unreachable.
- **Astrographer → Astral (edge #2):** wiki publishing.
- **Solomon ↔ Astrographer (edge #9):** cross-instance search.
- **Incanter → Astrographer (edge #14):** the prototype RAG engine over HTTP.
- **Emerald ↔ Astrographer (edges #15/#21):** graph editing round-trip.
- **Firmament → Astrographer (edge #18):** secure remote bridge.
- **Augur fan-out (edges #5/#6):** `document.published`/`document.updated`/`document.archived` events.

---

## Part B — Research-derived feature requests (need to be pinned as new contract elements / units)

These are the use cases developed from the research notes. Each is a candidate to be pinned as a new contract element in `docs/specs/astrographer.md` (or a new unit in the Astrographer project). Status reflects whether it is already landed, needs pinning, or is parked.

### B1. Pass 8 — Advanced RAG (from the Advanced-RAG research notes)

| # | Use case | Source | Status |
| --- | --- | --- | --- |
| B1.1 | **Topological `reference`→`fact` resolution** — resolve references in dependency order for multi-step answers (the DAG-RAG MUST HAVE). | `dag-rag-research-notes.md` §4 | **PARTIALLY LANDED** — the deterministic multi-hop walk (Unit X A2) realizes the topological resolution; the full dependency-ordered multi-step answer surface still needs pinning. |
| B1.2 | **Inference-time sub-task DAG** — decompose multi-hop queries into a sub-problem DAG, topologically sort, drive `ragQuery`/`ragStream` per node (the DAG-RAG SHOULD HAVE). | `dag-rag-research-notes.md` §4 | **NEEDS PINNING** (SHOULD HAVE). |
| B1.3 | **Question-specific evidence subgraph** — retrieve linked textual subgraphs over the reference graph and build a per-query knowledge graph (DAGR-style, NICE TO HAVE). | `dag-rag-research-notes.md` §4 | **PARKED** (overlaps B1.2). |
| B1.4 | **Local lexical (BM25) leg** — a BM25 index over `factKey`/`title`/`tags`/node text, fused at query time, surfaced as a `local`-source result (exact fact-key recall). | `hybrid-search-research-notes.md` §9 | **NEEDS PINNING** (SHOULD HAVE; GAP-2). |
| B1.5 | **Cross-encoder reranking** — two-stage retrieve-and-rerank with a small local reranker (e.g. `bge-reranker-v2-m3`). | `reranking-research-notes.md` §8 | **NEEDS PINNING** (SHOULD HAVE). |
| B1.6 | **Multi-query retrieval** — fan out N query variants, merge by stable node/document id. | `multi-query-retrieval-research-notes.md` §8 | **NEEDS PINNING** (SHOULD HAVE; GAP-1 — Incanter-side engine capability). |
| B1.7 | **Contextual compression** — `filter` mode on `rag_query`/`rag_stream` with a small local compressor; `extract` mode (fact-value extraction) as Phase 2. | `contextual-compression-research-notes.md` §8 | **NEEDS PINNING** (SHOULD HAVE). |
| B1.8 | **HyDE (opt-in query mode)** — generate a hypothetical fact/graph snippet via the local LLM, embed it, route into `local|incanter|zodiac`. | `hyde-research-notes.md` §6.4 | **NEEDS PINNING** (SHOULD HAVE, opt-in). |
| B1.9 | **Adaptive RAG routing to Zodiac** — a query-complexity router deciding between local graph RAG, local-only, and Zodiac, plus an answer-adequacy qualifier. | `adaptive-rag-research-notes.md` | **PARKED** (`docs/pending.md` D3). |
| B1.10 | **DeepEval RAG evaluation harness** — offline evaluation gate on faithfulness/relevancy/contextual precision-recall. | `rag-pipeline-evaluation-research-notes.md` §7 | **NEEDS PINNING** (SHOULD HAVE). |
| B1.11 | **Graph-structure / per-fact-node chunking** — chunk along graph structure, not flat text, preserving the single-source-of-truth invariant. | `rag-pipeline-evaluation-research-notes.md` §7 | **NEEDS PINNING** (MUST HAVE; Incanter's dynamic chunking is the vehicle). |

### B2. Pass 12 — Docs-as-product (from the docs-as-product research)

| # | Use case | Source | Status |
| --- | --- | --- | --- |
| B2.1 | **Agent-facing reachability surface** — `llms.txt`/`llms-full.txt` on the Astral-published wiki, markdown-copyable pages, and an MCP *resources* (read-docs) surface alongside the MCP *tools* (act) surface. | `docs-as-product-research-notes.md` §3.1 | **NEEDS PINNING** (MUST; GAP-4). |
| B2.2 | **Doc feedback / analytics loop** — no-result searches as a self-reporting documentation gap; feedback instrumentation routed into the defect catalogue. | `docs-as-product-research-notes.md` §3.1 | **NEEDS PINNING** (SHOULD). |

### B3. Pass 13 — Graph RAG (from the "When Vectors aren't enough" research)

| # | Use case | Source | Status |
| --- | --- | --- | --- |
| B3.1 | **Result-level provenance (`citations` + `trace`)** — the grounding set + the retrieval path walked on every RAG result. | `graph-rag-when-vectors-arent-enough-research-notes.md` §4 | **LANDED** (Unit X A1). |
| B3.2 | **Query audit log** — `getQueryAuditLog` + MCP `get_query_audit_log` + GUI audit panel (D4 parity). | `explainability-governance.md` | **PARTIALLY LANDED** — the log + MCP tool landed (Unit X A1); the GUI audit panel is parked (`docs/pending.md`). |
| B3.3 | **Multi-hop traversal (`mode: 'graph'`)** — deterministic hop-limited walk over the `reference`→`fact` graph (GAP-7, Astrographer-side). | `graph-rag-when-vectors-arent-enough-research-notes.md` §5 | **LANDED** (Unit X A2). |
| B3.4 | **Relationship filtering (`filters?`)** — the pinned `filters?` shape (node kind / edge type / target / state). | `multi-hop-questions.md` | **LANDED** (Unit X A2). |
| B3.5 | **Parent-context return (`expand: 'parent'`)** — return the parent Document/node-cluster for a retrieved child, capped + stale-propagating. | `parent-child-retrievers.md` | **LANDED** (Unit X A2). |
| B3.6 | **Community summaries (wiki-level aggregation)** — a scoped, lazy, local wiki-level summary surface. | `community-summaries.md` | **PARKED** (`docs/pending.md` D8; GAP-9). |
| B3.7 | **LLM-generated dynamic graph query** — Text2Cypher-style natural-language→structured-graph-query. | `dynamic-graph-query-generation.md` | **PARKED** (`docs/pending.md` D9). |

---

## Part C — Astrographer project's own parked/speculative use cases (from the Astrographer `docs/pending.md`)

These are the Astrographer project's own deferred/speculative items that may need pinning when their revisit conditions are met.

| # | Use case | Status |
| --- | --- | --- |
| C1 | **Markdown-vs-source diffing to detect changes** | SPECULATIVE (markdown is export-only). |
| C2 | **Remote embedding provider** | IMPLEMENTED (Unit F). |
| C3 | **API access to a remote DB (remote RAG store)** — a `createRemoteRagStore` behind the `RagStore` interface. | SPECULATIVE. |
| C4 | **RAG object versioning by journal timestamps** — point-in-time reconstruction. | SPECULATIVE. |
| C5 | **RBAC to selective versions** | SPECULATIVE (builds on C4). |
| C6 | **Per-leaf citation from markdown** | SHELVED (ENG-GAP-1; markdown is export-only). |
| C7 | **Markdown parsing to storage via text-match diffing** | IMPLEMENTED (Unit T, initial-ingestion framing). |
| C8 | **Shared-node edit notification + save prompt** — notify when a node is shared across N documents; prompt update-all vs fork. | SPECULATIVE. |
| C9 | **Edge/node staleness flagging (bidirectional)** — a review aid flagging connected nodes/edges when content changes. | SPECULATIVE. |
| C10 | **Document tabs (multi-document simultaneous render)** | SPECULATIVE. |
| C11 | **Manual document segmentation** — override default chunking. | SPECULATIVE. |
| C12 | **Binary-condense first-pass vector search** — ANN optimization over the vector index. | SPECULATIVE. |
| C13 | **Crosslink hover-preview pane** | SPECULATIVE. |
| C14 | **Click-drag moveable pane layout** | SPECULATIVE. |
| C15 | **Scoped snapshot for rendering** — a main-side traversal or a scoped `rag-doc-subgraph` IPC. | SPECULATIVE. |
| C16 | **Doc-nav select MCP-dispatchable** — MCP/UI parity for the doc-nav select. | SPECULATIVE. |
| C17 | **Editing-mode config (textarea vs rich-text contenteditable)** | IMPLEMENTED (5-unit slice). |
| C18 | **Lock document elements from editing** | SPECULATIVE. |
| C19 | **Toggle whether MCP can lock/unlock elements, or only human** | SPECULATIVE (builds on C18). |
| C20 | **Automatic (re-)export of markdown on content change** | SPECULATIVE. |
| C21 | **Optional HTML export instead of markdown** | SPECULATIVE. |
| C22 | **Migrate RAG engine to a faster/multithreading-capable language (Rust/Go/Odin)** | SPECULATIVE (MAJOR). |
| C23 | **JSON export of a document path through the RAG graph with adjacent nodes out to N steps** | SPECULATIVE. |
| C24 | **GUI audit panel (the `rag-query-audit` IPC + the D4-parity audit panel)** | PARKED from Unit X. |

---

## Summary — the use cases that most need pinning now

The highest-priority use cases to pin (beyond the already-landed Unit X A1/A2 and the preexisting core contract) are the **Pass 8 Advanced-RAG SHOULD-HAVE stack** and the **Pass 12 docs-as-product MUST**:

1. **B1.4 — Local lexical (BM25) leg** (exact fact-key recall; GAP-2).
2. **B1.5 — Cross-encoder reranking** (two-stage retrieve-and-rerank).
3. **B1.6 — Multi-query retrieval** (GAP-1, Incanter-side).
4. **B1.7 — Contextual compression** (`filter`/`extract` modes).
5. **B1.8 — HyDE (opt-in query mode)**.
6. **B1.10 — DeepEval RAG evaluation harness**.
7. **B1.11 — Graph-structure / per-fact-node chunking**.
8. **B2.1 — Agent-facing reachability surface** (`llms.txt`/`llms-full.txt` + markdown-copyable pages + MCP resources; GAP-4).
9. **B1.2 — Inference-time sub-task DAG** (the DAG-RAG SHOULD HAVE).
10. **B3.2 — GUI audit panel** (the D4-parity UI surface for the query audit log).

Parked (revisit when their conditions are met): B1.3, B1.9, B3.6, B3.7, and the Part C speculative items.
