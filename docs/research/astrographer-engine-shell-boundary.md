# Astrographer — Engine/Shell Use-Case Boundary (graph/vector vs interface/presentation)

- **Date:** 2026-09-08
- **Status:** Architectural analysis / reference. Not a behavior contract.
- **Purpose:** classify every use case that must be pinned for Astrographer by whether its core operation is a **technical graph/data function** (moves out of the Electron shell into a separate project running in a faster/multithread-capable language) or an **interface/presentation function** (stays in the Electron shell). This is the extraction boundary for the INCANTER-DISPOSITION / C22 RAG-engine migration.
- **Companion:** `docs/research/astrographer-use-cases-to-pin.md` (the full use-case catalogue with sources + statuses).

---

## 1. The boundary principle

The architectural rule for the extraction:

**The separate engine project owns every operation over the graph and the vectors — the data, the queries, the traversal, the indexing, the embeddings, and the retrieval math.** The Electron shell owns presentation, configuration, and the human/agent interaction surface (GUI, MCP server, IPC wiring, rendering, editing UI, provisioning).

### ENGINE (moves out) — technical graph/data functions
- Graph construction / storage / persistence (the node+edge store).
- Graph traversal / queries / adjacency / topological resolution / multi-hop.
- Lexical indexing (BM25) + exact-match retrieval.
- Vector embeddings, provider calls, the vector index, ANN/cosine scoring, the embedding cache.
- Retrieval: scoring, top-k selection, fusion (RRF), reranking, multi-query merge, compression, HyDE.
- Ingestion / chunking / parsing text→graph.
- Evaluation harness over retrieval outputs.
- The cross-link/embed consistency machinery **over the data** (the store-level enforcement).

### SHELL (stays) — interface/presentation functions
- The GUI (document editor, wiki browser, sidebar panes, settings, panels, audit panel).
- MCP server + IPC wiring (the tool surface that *proxies* engine calls).
- Rendering (Provident graph → HTML/markdown).
- Editing UI + the document authoring UX.
- Configuration / provisioning (engine connection state, provider config, credentials).
- The security-configuration carve-out (auth, TLS, credentials).
- Integration wiring (the HTTP/event transport to Astral/Zodiac/Solomon/Firmament/Augur).

### MIXED — both engine and shell components
Many use cases split across the boundary. Each is noted with **what moves** (engine) vs **what stays** (shell).

---

## 2. Use-case classification

### Part A — Preexisting design cases

| Use case | Classification | What moves vs stays |
| --- | --- | --- |
| **A1 Document store/wiki** (§4.1) | **MIXED** | The **store + persistence + revisioning** (nodes/edges) → ENGINE. The **wiki model API + GUI** (create/list/browse) → SHELL. Optimistic concurrency (`ConflictError`) is a store/data rule → ENGINE. |
| **A2 Cross-link/data-embed consistency** (§4.2) | **MIXED** | The consistency **invariant enforcement** (STALE/FRESH/BROKEN/RESOLVED, re-sync, the publish gate, `getConsistencyReport`) → ENGINE (it operates on the graph data). The **consistency panel GUI** → SHELL. |
| **A3 RAG query surface** (§4.3) | **MIXED** | The **RAG engine** (`ragQuery`/`ragStream`/`getEngineStatus`, scoring, the source union) → ENGINE. The **MCP tool + engine-status GUI** → SHELL (thin proxy). |
| **A4 Provident-graph push to Astral** (§4.4) | **MIXED** | The **graph→`provident-graph/1` serialization + content-hash idempotency** → ENGINE (a data export). The **HTTP push transport + push-status UI** → SHELL. |
| **A5 MCP surface** (§4.5) | **SHELL** | The whole MCP tool set + error mapping + D4 parity + security carve-out → SHELL (it proxies engine calls). |
| **A6 GUI surface** (§4.6) | **SHELL** | The whole GUI + parity mapping + fail-state presentation → SHELL. |
| **A7 Integrations** (§5) | **MIXED** | Zodiac query fan-out + Solomon cross-instance search + Incanter engine consumption → the **data/query half** is ENGINE; the **connection/event transport + wiring** (edges to Astral/Emerald/Firmament/Augur) → SHELL. |

### Part B — Research-derived feature requests

| Use case | Classification | What moves vs stays |
| --- | --- | --- |
| **B1.1 Topological `reference`→`fact` resolution** | **ENGINE** | Purely graph traversal → ENGINE. |
| **B1.2 Inference-time sub-task DAG** | **ENGINE** | Multi-hop query decomposition → ENGINE (drives engine query/stream). |
| **B1.3 Question-specific evidence subgraph** | **ENGINE** | Graph subgraph retrieval → ENGINE. |
| **B1.4 Local lexical (BM25) leg** | **ENGINE** | Lexical index + exact-match retrieval → ENGINE. |
| **B1.5 Cross-encoder reranking** | **ENGINE** | Joint query-doc scoring (a data operation) → ENGINE. |
| **B1.6 Multi-query retrieval** | **ENGINE** | Query fan-out + merge → ENGINE (engine-side capability). |
| **B1.7 Contextual compression** | **ENGINE** | Post-retrieval ML transform (filter/extract) → ENGINE (query-time data op). |
| **B1.8 HyDE (opt-in)** | **ENGINE** | Hypothetical-doc embedding + routing → ENGINE (LLM embedding op). |
| **B1.9 Adaptive RAG routing to Zodiac** | **ENGINE** | Query-complexity router + adequacy qualifier → ENGINE (LLM decision op). The Zodiac *connection* stays SHELL. |
| **B1.10 DeepEval RAG evaluation harness** | **ENGINE** | Offline evaluation of retrieval faithfulness/relevancy/precision-recall → ENGINE (or a standalone harness co-located with the engine). |
| **B1.11 Graph-structure / per-fact-node chunking** | **ENGINE** | Ingestion-time chunking → ENGINE. |
| **B2.1 Agent-facing reachability surface** (llms.txt/llms-full.txt + markdown-copyable + MCP resources) | **SHELL** | The doc-serving surface + markdown rendering (Provident adapter) + MCP resources → SHELL. (The *content* it serves is engine-produced documents; the serving/presentation is shell.) |
| **B2.2 Doc feedback / analytics loop** | **SHELL** | Feedback instrumentation + analytics surfacing + defect routing → SHELL. |
| **B3.1 Result-level provenance (`citations` + `trace`)** | **MIXED** | The **trace/citations computation** (walking the graph) → ENGINE. The **presence on the shell's MCP/IPC result** → SHELL. |
| **B3.2 Query audit log** | **MIXED** | The **audit-log recording at the engine API** → ENGINE. The **MCP `get_query_audit_log` + GUI audit panel** → SHELL. |
| **B3.3 Multi-hop traversal (`mode: 'graph'`)** | **ENGINE** | Deterministic graph walk → ENGINE (GAP-7). |
| **B3.4 Relationship filtering (`filters?`)** | **ENGINE** | The node/edge filtering over the graph → ENGINE. |
| **B3.5 Parent-context return (`expand: 'parent'`)** | **ENGINE** | The parent lookup over the store → ENGINE. |
| **B3.6 Community summaries** | **ENGINE** | Wiki-level graph aggregation + LLM summarization → ENGINE (D8/GAP-9). |
| **B3.7 LLM-generated dynamic graph query** | **ENGINE** | Text2Cypher-style NL→structured-graph-query + execution → ENGINE (D9). |

### Part C — Astrographer project's own parked/speculative items

| Use case | Classification | What moves vs stays |
| --- | --- | --- |
| **C1 Markdown-vs-source diffing** | **MIXED** | The **diff computation** (parsing source vs markdown) → ENGINE. The **export/mirror trigger + UI** → SHELL. |
| **C2 Remote embedding provider** | **ENGINE** | The provider call + embedding → ENGINE (already behind the `Embedder` seam). |
| **C3 API access to a remote DB (remote RAG store)** | **ENGINE** | The remote store behind the `RagStore` interface → ENGINE. |
| **C4 RAG object versioning by journal** | **ENGINE** | Point-in-time reconstruction from the journal → ENGINE. |
| **C5 RBAC to selective versions** | **SHELL** | The authorization gate at the access boundary → SHELL (a policy/interface concern; it gates access to engine data). |
| **C6 Per-leaf citation from markdown** | **SHELL** | The markdown adapter + node-identity rendering → SHELL/foundation (the underlying line→node *map* is ENGINE). |
| **C7 Markdown parsing to storage** | **ENGINE** | Text→graph ingestion/parsing → ENGINE. |
| **C8 Shared-node edit notification + save prompt** | **MIXED** | The **shared-node ownership detection** → ENGINE (graph query). The **notification + save-prompt UI** → SHELL. |
| **C9 Edge/node staleness flagging** | **MIXED** | The **staleness detection** (which connected nodes/edges to flag) → ENGINE (graph). The **review-aid surfacing** → SHELL. |
| **C10 Document tabs (multi-doc render)** | **SHELL** | Multi-document UI → SHELL. (The per-doc subgraph is engine-produced.) |
| **C11 Manual document segmentation** | **MIXED** | The **graph mutation** (override node/edge boundaries) → ENGINE. The **selection/mark UI** → SHELL. |
| **C12 Binary-condense first-pass vector search** | **ENGINE** | ANN / binary-quantization scoring over the vector index → ENGINE. |
| **C13 Crosslink hover-preview pane** | **SHELL** | The display pane + hover UX → SHELL. (The linked-node render is engine-data-driven.) |
| **C14 Click-drag moveable pane layout** | **SHELL** | Pane-layout UX + placement → SHELL. |
| **C15 Scoped snapshot for rendering** | **MIXED** | The **subgraph computation/serialization** → ENGINE. The **renderer consumption** → SHELL. |
| **C16 Doc-nav select MCP-dispatchable** | **SHELL** | The MCP dispatch target + UI parity → SHELL. |
| **C17 Editing-mode config** | **SHELL** | The textarea-vs-contenteditable toggle + settings → SHELL. |
| **C18 Lock document elements from editing** | **MIXED** | The **lock state** (stored on the node) → ENGINE data. The **edit-path enforcement + read-only UI** → SHELL. |
| **C19 Toggle whether MCP can lock/unlock** | **SHELL** | The authorization gate → SHELL. |
| **C20 Automatic markdown re-export** | **MIXED** | The **change detection** → ENGINE. The **export rendering** → SHELL. |
| **C21 Optional HTML export** | **SHELL** | HTML rendering/export → SHELL. |
| **C22 Migrate RAG engine to a faster/multithread-capable language** | **ENGINE (driver)** | THIS IS THE EXTRACTION: the whole graph/vector engine moves to a separate project. |
| **C23 JSON export of a document path through the graph** | **MIXED** | The **graph-path extraction** → ENGINE. The **JSON export/transport** → SHELL. |
| **C24 GUI audit panel** | **SHELL** | The D4-parity UI surface for the query audit log → SHELL. |

---

## 3. The resulting engine work-package (everything classified ENGINE/MIXED-engine-half)

The following operations form the **engine work-package** that moves to the separate fast/multithreaded language project:

**Graph data**
- The doc-store persistence (nodes/edges) + revisioning + optimistic concurrency.
- Graph traversal + adjacency (`edgesFrom`/`edgesTo`/`edgesByKind`/`edgesForDocument`).
- Topological `reference`→`fact` resolution (B1.1).
- Multi-hop traversal (GAP-7, B3.3) + relationship filtering (B3.4) + parent-context expansion (B3.5).
- Inference-time sub-task DAG (B1.2) + evidence subgraph (B1.3).
- LLM-generated dynamic graph query (B3.7) + community summaries (B3.6).
- The cross-link/embed consistency invariant + `getConsistencyReport`.
- Graph-path extraction (C23), subgraph computation/serialization (C15), line→node map, staleness detection (C9), shared-node detection (C8), graph mutation (C11), change detection (C20).
- Markdown→graph parsing (C7) + graph-structure chunking (B1.11).

**Vector / lexical data**
- Embeddings + provider calls (C2) + the vector index + cosine/ANN scoring (C12) + the embedding cache.
- Lexical BM25 index + exact fact-key retrieval (B1.4).
- Retrieval: top-k selection, RRF fusion, cross-encoder reranking (B1.5), multi-query merge (B1.6), compression (B1.7), HyDE (B1.8).
- Adaptive RAG routing + qualifier (B1.9).
- DeepEval evaluation harness (B1.10).
- RAG object versioning from the journal (C4).
- Remote RAG store (C3).

**The engine's own API surface** (the query/stream/engine-status contract the shell proxies): `ragQuery`/`ragStream`/`getEngineStatus`, the provenance trace/citations computation (B3.1), the query audit-log recording (B3.2).

## 4. The resulting shell work-package (everything classified SHELL/MIXED-shell-half)

- GUI (document editor, wiki browser, sidebar panes, settings, panels, the audit panel C24, the doc feedback/analytics surface B2.2).
- MCP server + IPC wiring (proxying the engine) + the MCP resources surface (B2.1).
- Rendering (Provident graph → HTML/markdown), including markdown-copyable pages + HTML export (C21) + llms.txt/llms-full.txt (B2.1).
- Editing UI + document authoring UX + editing-mode config (C17) + element locks (C18's UI) + MCP-lock authorization (C19) + document tabs (C10) + manual-segmentation UI (C11).
- Crosslink hover-preview (C13) + pane layout (C14) + doc-nav MCP parity (C16).
- Configuration/provisioning (engine connection state, provider config) + the security-configuration carve-out (credentials, TLS, Firmament auth).
- Integration wiring/transport (HTTP push to Astral A4, Zodiac/Solomon/Firmament/Augur connections, event surfaces).
- The graph→`provident-graph/1` transport (the serialization is engine; the push is shell).

---

## 5. Boundary notes / open questions

1. **The document-store API is the seam.** The shell talks to the engine through the `RagStore` interface (persistence) + the query/stream/engine-status API. The current `RagStore` interface (`docs/specs/unit-a-rag-store.md` §5.4) + the retrieval API (`src/main/retrieval.ts`) are the natural IPC/process boundary. C3 (remote RAG store) already sketches `createRemoteRagStore` behind that interface — the migration generalizes it to `createEngineRagStore`.
2. **Consistency enforcement placement.** The cross-link/embed consistency invariant (§4.2.3) operates on the graph data, so it moves ENGINE. The publish gate + `getConsistencyReport` are engine-side data checks; only their UI is shell.
3. **Providence/audit.** The trace/citations computation and the audit-log *recording* are engine-side (they reflect engine work); the MCP tool + GUI panel surfaces are shell. The audit log should be owned by the engine (it records engine queries) and exposed via the engine API.
4. **Presentation of engine data.** Items like the graph path (C23), the subgraph (C15), and the parent-context (B3.5) compute *in the engine* but are *rendered in the shell*. The renderer only receives serialized subgraphs.
5. **The C22 driver.** This boundary IS the C22 migration scope. The engine work-package (Part 3) is what the separate project implements in a faster/multithread-capable language; the shell work-package (Part 4) stays in TypeScript/Electron.
