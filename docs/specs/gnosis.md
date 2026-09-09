# Gnosis — Per-Component Behavior Contract

- **Status:** DRAFT — per-component contract, 2026-09-08.
- **Tool lifecycle:** PLANNED (production engine) — see `docs/architecture-overview.md` §4.2 (registry), §5.2 (Layer 2), §6 (cross-cutting principles).
- **Companion docs:** `docs/architecture-overview.md` (component registry §4, layering §5, cross-cutting principles §6, open questions §8); `docs/integration-matrix.md` (directed edges §3.2, MCP-GUI parity §5, flows §6); `docs/decisions.md` (ACTIVE decisions, esp. D1–D4, GNOSIS-ENGINE, INCANTER-DISPOSITION, MCP-SURFACE-DEFERRED, D4-CARVEOUT-SCOPE, GRAPH-PUSH-FORMAT, D2-CLARIFICATION); `docs/pending.md` (open question #2, parked items D3/D4/D5/D8/D9); `docs/research/astrographer-engine-shell-boundary.md` (the engine work-package Gnosis owns); `docs/research/astrographer-use-cases-to-pin.md` (the use-case catalogue); `docs/specs/astrographer.md` (the shell contract Gnosis's engine-side contract honors); `docs/specs/zodiac.md` (the remote RAG companion whose query modes Gnosis's must be consistent with); `docs/specs/incanter.md` (the archived prototype Gnosis supersedes).
- **Format:** compile-horizon-review (status / what the proposal asks / feasibility verdict / behavior contract / integration contract / fail-states / gaps + costs-benefits). This is a documentation deliverable only — no code, no tests.

---

## 1. Status

**DRAFT — per-component contract, 2026-09-08.**

This is the compile-horizon-review behavior contract for **Gnosis**, the
**production graph/vector engine** of the Auspicion Suite. It is a documentation
deliverable only — no code, no tests. It is derived from the reviewed proposal
(the engine/shell boundary analysis `docs/research/astrographer-engine-shell-boundary.md`),
the use-case catalogue (`docs/research/astrographer-use-cases-to-pin.md`), the
canonical Astrographer contract (`docs/specs/astrographer.md`), the archived
Incanter prototype contract (`docs/specs/incanter.md`), the Zodiac companion
contract (`docs/specs/zodiac.md`), and the ACTIVE decisions (`docs/decisions.md`).
It is the contract the Gnosis project's own repo must honor, and the source a
TestWriter derives every state and fail-state from.

**Component lifecycle.** Gnosis is **PLANNED (production engine)**. It is a
**separate project** running in a **faster/multithread-capable language** (per
decision INCANTER-DISPOSITION's "production engine pulled out of JS"; the
production engine direction is unchanged — the prototype is now archived). It
replaces **Incanter** as the RAG engine Astrographer consumes.

**Supersession (decision GNOSIS-ENGINE).** Gnosis **supersedes Incanter**.
Incanter is **ARCHIVED as a prototype lexical system** — its disposition changes
from "prototype for RAG design" to "archived prototype"; its spec
(`docs/specs/incanter.md`) is **archived, not deleted**. Gnosis owns the
**engine work-package** from the engine/shell boundary analysis: the document
store, the knowledge graph, fact/citation tracking, consistency enforcement, and
the RAG/agent-memory retrieval stack (all query modes/surfaces). The Electron
shell (Astrographer) keeps the interface/presentation surface and **proxies
engine calls over the `RagStore` + query/stream/engine-status API**.

**Open-question flags in this contract** (each is a resolution target before the
corresponding integration can be finalized; see §7):
- **F1 — MCP surface — NOT APPLICABLE to Gnosis.** Gnosis has **no MCP surface
  and no GUI surface** — it is a pure backend to Astrographer and uses
  Astrographer as its front-end (§4.6.2). The suite-wide MCP surface
  (`docs/pending.md` #2) is a **shell-side** concern (the Astrographer contract
  §4.5/§4.6), not a Gnosis one.
- **F2 — engine transport/API reconciliation**: the exact IPC/process transport
  between the shell and the Gnosis engine (the `RagStore` + query/stream/engine-
  status seam) is not yet pinned to a concrete wire format. The contract here
  pins the **logical API**; the transport is a Gnosis-repo decision (§7.2).
- **F3 — adaptive RAG routing to Zodiac** — **PARKED** (`docs/pending.md` D3):
  the query-complexity router + adequacy qualifier is parked; Gnosis notes it as
  a parked retrieval-stack layer (§4.5.6, §7.4).
- **F4 — community summaries** — **PARKED** (`docs/pending.md` D8): the
  wiki-level aggregation surface is parked; Gnosis notes it as a parked
  retrieval surface (§4.5.7, §7.5).
- **F5 — LLM-generated dynamic graph query** — **PARKED** (`docs/pending.md`
  D9): the Text2Cypher-style NL→structured-graph-query is parked; Gnosis notes
  it as a parked query surface (§4.5.8, §7.6).

---

## 2. What the proposal asks

**Purpose.** Gnosis is the **production graph/vector engine** of the Auspicion
Suite. It is the engine that **Astrographer** (the Electron shell) consumes for
all graph/data and RAG/retrieval work. It replaces Incanter as the RAG engine
and generalizes the engine/shell boundary: Gnosis owns every operation over the
graph and the vectors — the data, the queries, the traversal, the indexing, the
embeddings, and the retrieval math. The shell owns presentation, configuration,
and the human/agent interaction surface (GUI, MCP server, IPC wiring, rendering,
editing UI, provisioning).

**Scope.** Gnosis is a **separate project** running in a **faster/multithread-
capable language** (per INCANTER-DISPOSITION's "production engine pulled out of
JS"). It is a **Layer 2 local-first application** (`docs/architecture-overview.md`
§5.2). It is **PLANNED**. Gnosis owns the **engine work-package** from the
engine/shell boundary analysis (`docs/research/astrographer-engine-shell-boundary.md`
§3):

- **Graph data:** the doc-store persistence (nodes/edges) + revisioning +
  optimistic concurrency; graph traversal + adjacency; topological
  `reference`→`fact` resolution; multi-hop traversal + relationship filtering +
  parent-context expansion; the cross-link/embed consistency invariant +
  `getConsistencyReport`; graph-path extraction, subgraph computation/
  serialization, line→node map, staleness detection, shared-node detection,
  graph mutation, change detection; markdown→graph parsing + graph-structure
  chunking.
- **Vector / lexical data:** embeddings + provider calls + the vector index +
  cosine/ANN scoring + the embedding cache; the lexical BM25 index + exact
  fact-key retrieval; retrieval (top-k selection, RRF fusion, cross-encoder
  reranking, multi-query merge, compression, HyDE); RAG object versioning from
  the journal; the remote RAG store.
- **The engine's own API surface** (the query/stream/engine-status contract the
  shell proxies): `ragQuery`/`ragStream`/`getEngineStatus`, the provenance
  trace/citations computation, the query audit-log recording.

**Design goals (non-negotiable):**
1. **Graphical RAG.** Documents are stored as Provident graphs; retrieval is
   graph-aware (nodes + edges), not flat text.
2. **Document store / wiki.** Gnosis is the persistence home of documentation;
   documents are the unit of authoring and the unit pushed to Astral.
3. **Cross-link / data-embed consistency.** A fact is defined once and
   referenced or embedded elsewhere; the engine enforces that references stay
   consistent (no stale duplicated facts).
4. **Local-first (D2).** Fully self-hosted, no cloud dependency for core
   function. Interconnection is additive. The engine runs locally; the only
   non-local runtime dependency is an optional local embedding/LLM provider
   (e.g. Ollama) for the embedding/rerank/compression/HyDE legs.
5. **No MCP/GUI surface (D4 applies at the shell).** Gnosis is a **pure backend
   to Astrographer** and uses Astrographer as its front-end — it has **no MCP
   surface and no GUI surface** (§4.6.2). The D4 MCP-GUI parity requirement
   applies at the **Astrographer shell**, which provides the MCP + GUI surfaces
   that proxy Gnosis's API. Gnosis is not itself a user-facing app.
6. **Open-source first (D1, AGPL-3.0).** The engine itself is never paywalled.
7. **Interconnection (D3).** Gnosis both gains and provides utility when wired
   to Zodiac (a second RAG data source), Solomon (cross-instance search), and
   Astral (via the shell's push).
8. **Manual override for graph operations.** **Non-vector operations in the
   graph space always have a manual access/override option.** For every graph
   operation that has an automatic/computed form (community detection, fact
   extraction, consistency derivation, chunking/segmentation, triple inference),
   an agent or user can **directly declare the result** — e.g. declare a given
   set as a community and provide a summary, rather than only relying on the
   automatic computation. The manual declaration is stored in the authoritative
   graph (a single source of truth) and the automatic computation is an
   alternative, never a replacement for the manual override (§4.2.8).

**What this contract does NOT cover.** Gnosis's own source, tests, or
implementation. The shell-side behavior (GUI, MCP server, IPC wiring, rendering,
editing UI, provisioning, the security-configuration carve-out's UI) is owned by
the Astrographer contract (`docs/specs/astrographer.md`). The remote RAG
companion's reply-side contract is owned by the Zodiac contract
(`docs/specs/zodiac.md`). The cross-instance search surface is owned by the
Solomon contract (`docs/specs/solomon.md`). The archived prototype's behavior is
owned by the archived Incanter spec (`docs/specs/incanter.md`). This contract
pins the **engine-side** behavior the shell proxies and the suite relies on.

---

## 3. Feasibility verdict

**Verdict: FEASIBLE-WITH-GAPS.**

**Reasoning.**
- **Feasible core.** The document-store/wiki + cross-link/data-embed model is a
  well-understood authoring problem, and the Provident graph substrate
  (Provident-SSR, Provident-Editable, Provident-Electron — all EXISTING, Layer 1)
  already provides the graph model and the Electron shell. The engine work-
  package (document store, knowledge graph, fact/citation tracking, consistency
  enforcement, RAG/agent-memory retrieval) is implementable in a
  faster/multithread-capable language (Rust per INCANTER-DISPOSITION).
- **Feasible boundary.** The engine/shell boundary is clean: the shell talks to
  the engine through the `RagStore` interface (persistence) + the
  query/stream/engine-status API. This is the natural IPC/process boundary
  (C3's `createRemoteRagStore` generalizes to `createEngineRagStore`). The
  engine's internals are opaque to the shell; a future engine swap is isolated
  to this seam.
- **Feasible RAG stack.** The retrieval stack (lexical BM25 leg, cross-encoder
  reranking, multi-query merge, contextual compression, HyDE) is fully
  D2-compliant: BM25 is a cheap in-process inverted index; the dense leg uses a
  local embedding provider (Ollama); the reranker/compressor/HyDE use local
  models. The research notes confirm local feasibility for each technique.
- **Gaps that block finalization (not feasibility):**
  - **F1 — MCP surface — NOT APPLICABLE to Gnosis.** Gnosis has no MCP/GUI
    surface (it is a pure backend to Astrographer, §4.6.2); the suite-wide MCP
    surface (`docs/pending.md` #2) is a shell-side concern, not a Gnosis one.
  - **F2 — engine transport/API reconciliation.** The exact IPC/process transport
    between the shell and the engine is not yet pinned to a concrete wire
    format. The logical API is pinned here; the transport is a Gnosis-repo
    decision (§7.2).
  - **F3/F4/F5 — parked retrieval layers** (adaptive RAG routing D3, community
    summaries D8, LLM-generated dynamic graph query D9). These are parked, not
    blocking; the MUST/SHOULD retrieval foundation ships first.
- **Not-yet-feasible aspects:** none. The engine is PLANNED and its dependencies
  (Provident foundation, local embedding/LLM providers) are EXISTING or
  locally-runnable; the remaining blockers are contract-pinning, not technical
  infeasibility.

---

## 4. Behavior contract

This section is the concrete contract a TestWriter derives states from. Every
method/API, its signature, return shape, and throw pattern; every valid/happy-
path state; every documented fail-state. Section numbers are stable references
for the TestWriter and the proofreader.

### 4.1 Document store (the persistence layer)

**4.1.1 The document unit.** A **Document** is the atomic unit of authoring and
the unit pushed to Astral. A Document is a Provident graph (nodes + edges) with
a stable identity.

- **Document identity:** `documentId` — a stable, globally-unique string
  (UUID v4). Immutable once created. Never reused.
- **Document revision:** `revision` — a monotonically increasing non-negative
  integer, incremented on every committed change. `revision = 0` is the initial
  state of a newly created document.
- **Document wiki:** `wikiId` — the UUID v4 of the Wiki the document belongs to.
  A Document belongs to exactly one Wiki.
- **Document content:** `graph` — the Provident graph (the document's body).
  Nodes carry content; edges carry relationships (see §4.2).
- **Document metadata:** `title`, `createdAt` (ISO-8601 UTC), `updatedAt`
  (ISO-8601 UTC), `tags` (string array), `author` (string).
- **Document state machine:** `DRAFT` → `PUBLISHED` → `ARCHIVED`. A document
  may transition DRAFT→PUBLISHED, PUBLISHED→DRAFT (unpublish), PUBLISHED→
  ARCHIVED, DRAFT→ARCHIVED. ARCHIVED is terminal (no further transitions).
  Publishing is the precondition for the Astral push (§4.4 of the Astrographer
  contract; the push transport is shell-side, the publish gate is engine-side,
  §4.4 of this contract).

**4.1.2 The wiki model.** A **Wiki** is a named collection of Documents with a
`wikiId` (UUID v4) and a `name`. A Document belongs to exactly one Wiki.
Cross-wiki references are permitted (a Document in Wiki A may reference a
Document in Wiki B) but are flagged as cross-wiki in the reference graph
(`crosslink` edges, §4.2.2).

**4.1.3 Store operations (the document-store API).** These are the core
operations a TestWriter derives states from. Each is exposed to the shell, which
provides the GUI + MCP surfaces that proxy the engine (§4.6.2).

| Operation | Signature | Return shape | Throw / fail-state |
| --- | --- | --- | --- |
| `createDocument` | `(wikiId, {title, tags?, author?}) → Document` | New `Document` at `revision = 0`, state `DRAFT` | `WikiNotFound` if `wikiId` unknown; `ValidationError` if `title` empty or > 200 chars |
| `getDocument` | `(documentId) → Document` | The `Document` at its current revision | `DocumentNotFound` if `documentId` unknown |
| `updateDocument` | `(documentId, {graph, title?, tags?}) → Document` | The `Document` at `revision + 1` | `DocumentNotFound`; `ValidationError` if `graph` is not a valid Provident graph; `ConflictError` if the caller's base revision is stale (optimistic concurrency, §4.1.4) |
| `deleteDocument` | `(documentId) → void` | `void` | `DocumentNotFound`; `DocumentInUse` if any other document references it (§4.2.4) |
| `publishDocument` | `(documentId) → Document` | The `Document` with state `PUBLISHED` | `DocumentNotFound`; `UnresolvedReference` if any reference in the graph is unresolved (§4.4.3) |
| `unpublishDocument` | `(documentId) → Document` | The `Document` with state `DRAFT` | `DocumentNotFound`; `InvalidState` if state is not `PUBLISHED` |
| `archiveDocument` | `(documentId) → Document` | The `Document` with state `ARCHIVED` | `DocumentNotFound`; `InvalidState` if state is `ARCHIVED` |
| `listDocuments` | `(wikiId, {state?, tag?, page?, pageSize?}) → {items, total, page, pageSize}` | Paginated list of `Document` summaries | `WikiNotFound`; `ValidationError` if `page < 1` or `pageSize < 1` or `pageSize > 100` |
| `createWiki` | `({name}) → Wiki` | New `Wiki` | `ValidationError` if `name` empty or > 100 chars |
| `getWiki` | `(wikiId) → Wiki` | The `Wiki` | `WikiNotFound` |
| `listWikis` | `() → Wiki[]` | All `Wiki`s | — |

**4.1.4 Optimistic concurrency.** `updateDocument` requires the caller to
supply the base `revision` it read. If the stored `revision` differs from the
caller's base, the update is rejected with `ConflictError` (HTTP 409 / MCP
error). The caller must re-read and re-apply. This is the consistency guard for
concurrent edits (including concurrent Emerald edits, §5.5 of the Astrographer
contract). The engine enforces this at the store level (a data rule, per the
engine/shell boundary analysis §2 A1).

**4.1.5 The `RagStore` interface (the seam the shell proxies).** The shell
talks to the engine through the `RagStore` interface (persistence) + the
query/stream/engine-status API (§4.6.1). The `RagStore` interface is the
**natural IPC/process boundary** (per the engine/shell boundary analysis §5.1).
It exposes the store operations (§4.1.3), the graph adjacency methods (§4.2.5),
the consistency report (§4.4.4), and the query audit log (§4.3.4). The
migration generalizes C3's `createRemoteRagStore` to `createEngineRagStore`
behind this interface. The `RagStore` interface is the **persistence seam**; the
query/stream/engine-status API is the **retrieval seam** (§4.6.1).

### 4.2 Knowledge graph (the graph model)

**4.2.1 Node kinds.** A Provident graph node is one of:
- **`content`** — a normal authored node (text, heading, list, etc.).
- **`fact`** — a single source-of-truth fact node. A fact has a `factKey`
  (stable string, unique within the Wiki) and a `value` (the canonical value).
- **`reference`** — a node that points at a `fact` (or another node) in the
  same or another document. A reference carries a `target` (the target
  `documentId` + node id) and a `mode` (see §4.2.2).

**4.2.2 Edge kinds.** A Provident graph edge is one of:
- **`link`** — a navigational reference edge. The referencing document stores
  only the target pointer; the displayed value is resolved live from thVe target
  at render time. No copy of the value is stored.
- **`embed`** — a shared data embed edge. The referencing document stores a
  **snapshot** of the target's value plus the target pointer. The snapshot is
  the "shared data embed" the brief describes. Consistency is enforced by
  re-syncing snapshots when the target changes (§4.4).
- **`crosslink`** — a cross-wiki reference edge. A reference whose target is in
  another Wiki. Resolved the same way as `link`/`embed` but flagged
  `crossWiki: true` in the consistency report (§4.4.4).
- **`doc-head`** — the document's entry edge: from the document root to its
  first node. Every document has exactly one `doc-head` edge.
- **`next-section`** — a sequential ordering edge: from one section node to the
  next within a document. Defines the document's linear reading order.
- **`doc-end`** — the document's terminal edge: to the document's end sentinel.
  Every document has exactly one `doc-end` edge.
- **`doc-child`** — a containment edge: from a parent node to a child node
  (hierarchy/containment within a document).

**4.2.3 Properties.** Properties may be attached to both nodes and edges (the
labeled-property-graph model, per `docs/research/graph-rag-when-vectors-arent-
enough-research-notes.md` §1a `knowledge-graph-model`). Node properties include
`kind`, `text`/`value`, `factKey` (on `fact` nodes), `target` + `mode` (on
`reference` nodes), and document metadata (`title`, `tags`, `author`). Edge
properties include `kind`, `state` (for `link`/`embed`/`crosslink` edges:
`FRESH`/`RESOLVED`/`STALE`/`BROKEN`, §4.4.2), and `crossWiki` (boolean).

**4.2.4 The `reference`→`fact` graph.** The `reference`→`fact` graph is the
directed structure formed by `link`/`embed`/`crosslink` edges from `reference`
nodes to their target `fact` (or other) nodes. Resolving a `reference` node's
value requires resolving its target `fact` first — a **dependency-ordered**
resolution that is exactly a topological walk over the reference graph
(`docs/research/dag-rag-research-notes.md` §3). A `fact` node is the single
source of truth; a DAG evaluation order lets the engine resolve facts in
dependency order and reuse each resolved fact across multiple dependents.

**4.2.5 Adjacency methods.** These are the graph traversal primitives the engine
exposes (per the engine/shell boundary analysis §3 "Graph data"). Each is
exposed through the `RagStore` interface (§4.1.5) and proxied by the shell.

| Method | Signature | Return shape | Throw / fail-state |
| --- | --- | --- | --- |
| `edgesFrom` | `(documentId, nodeId) → Edge[]` | All edges whose source is `(documentId, nodeId)` | `DocumentNotFound` if `documentId` unknown; `ValidationError` if `nodeId` is not a node in the document |
| `edgesTo` | `(documentId, nodeId) → Edge[]` | All edges whose target is `(documentId, nodeId)` | `DocumentNotFound`; `ValidationError` if `nodeId` is not a node in the document |
| `edgesByKind` | `(documentId, nodeId, kind) → Edge[]` | All edges from `(documentId, nodeId)` of the given `kind` | `DocumentNotFound`; `ValidationError` if `nodeId` invalid or `kind` not a valid edge kind (§4.2.2) |
| `edgesForDocument` | `(documentId) → Edge[]` | All edges in the document | `DocumentNotFound` |
| `docHeadForDocument` | `(documentId) → Node` | The document's head node (the target of its `doc-head` edge) | `DocumentNotFound`; `InvalidState` if the document has no `doc-head` edge (malformed graph) |

**4.2.6 Topological `reference`→`fact` resolution.** The engine resolves
`reference` nodes in **topological order** over the `reference`→`fact` graph
(B1.1, `docs/research/astrographer-use-cases-to-pin.md` §Part B). A `reference`
is resolved only after its target `fact` is resolved; each resolved `fact` is
reused across all its dependents (no redundant re-resolution). This is the
foundation of the multi-hop traversal (§4.5.2) and the consistency invariant
(§4.4). Fail-states: `CycleDetected` if the graph contains a `reference`→`fact`
cycle (a node already on the current path; DAG-RAG pitfall 3,
`docs/research/dag-rag-research-notes.md` §5); `HopLimitExceeded` if a bounded
resolution exceeds its hop cap.

**4.2.7 The subject-relation model (the knowledge-graph core).** The knowledge
graph's core functionality is the **subject-relation-object (triple) model** —
the semantic structure that makes the graph queryable as a knowledge graph, not
just a document tree. A triple is:

> **`(subject, relation, object)`** — a directed semantic statement where
> `subject` and `object` are nodes and `relation` is a typed edge between them.

**Example:** `(subject: 'Astrographer', relation: 'implements as backend',
object: 'Gnosis')` — a semantic edge asserting that Astrographer implements
Gnosis as its backend.

**4.2.7.1 Triple components.**
- **`subject`** — the source node of the triple (a `content`/`fact`/`reference`
  node, or a named entity node). The "noun" the statement is about.
- **`relation`** — a typed, directed edge from the subject to the object. A
  relation has a **`relationType`** (a stable string, e.g. `'implements as
  backend'`, `'depends_on'`, `'references'`, `'part_of'`, `'causes'`). The
  relation is the "verb" of the statement.
- **`object`** — the target node of the triple. The "noun" the statement points
  to.

**4.2.7.2 Triple storage.** A triple is stored as a **typed edge** on the
knowledge graph: the `subject` is the edge's source node, the `object` is the
edge's target node, and the `relation` is the edge's `relationType` property.
The existing edge kinds (§4.2.2) are the structural edges (document flow,
containment, reference); the **relation edges** are the semantic edges that
carry a `relationType`. A relation edge is a first-class edge kind
(`relation`), distinct from the structural kinds.

**Ownership (pinned).** The triple is **owned by the knowledge graph** as a
`relation` edge — a node/edge structure, NOT an external index. The subject and
object are nodes; the relation is an edge with a `relationType` property. This
keeps the triple a single source of truth (no second store to keep in sync) and
composable with the rest of the graph (adjacency, consistency, multi-hop
traversal).

**Derived index (not a second owner).** The engine MAY maintain a **derived
triple index** (subject/relation/object lookup) to make `queryTriples`/
`getTriples` efficient. This is a **derived, rebuildable index** over the
authoritative edge store — the same pattern as the lexical BM25 index (§4.5.3)
and the vector index (§4.5.3). It is a query-time optimization, never the owner
of the data; it can be rebuilt from the edge store at any time and is not a
second source of truth.

**4.2.7.3 Triple operations.** The engine exposes the triple query surface:

| Operation | Signature | Return shape | Throw / fail-state |
| --- | --- | --- | --- |
| `addTriple` | `(subject: {documentId, nodeId}, relation: string, object: {documentId, nodeId}, {wikiId}) → Triple` | `{subject, relation, object, relationType, createdAt}` | `DocumentNotFound` if `subject`/`object` document unknown; `ValidationError` if `relation` empty or `subject`/`object` node invalid |
| `getTriples` | `(node: {documentId, nodeId}, {direction?: 'out'|'in'|'both', relationType?, wikiId}) → Triple[]` | All triples where `node` is the subject (out), object (in), or either (both), optionally filtered by `relationType` | `DocumentNotFound`; `ValidationError` if `node` invalid or `relationType` empty |
| `queryTriples` | `(subject?, relation?, object?, {wikiId, limit?}) → Triple[]` | All triples matching the given subject/relation/object pattern (any component may be omitted as a wildcard) | `WikiNotFound`; `ValidationError` if `limit < 1` or `limit > 100` |

**4.2.7.4 The triple model and the reference graph.** The subject-relation model
composes with the `reference`→`fact` graph (§4.2.4): a `fact` node can be the
subject or object of a triple, and a `reference` node's resolution can traverse
relation edges. The triple model is the semantic layer on top of the structural
graph — it is what makes the knowledge graph answer relationship-aware and
multi-hop questions (the talk's "relationship data", `docs/research/graph-rag-
when-vectors-arent-enough-research-notes.md` §2). The multi-hop traversal
(§4.5.2) can walk relation edges as well as `reference`→`fact` edges.

**4.2.7.5 Triple fail-states.** `addTriple` with a `relation` that is not a
valid `relationType` (empty or malformed) → `ValidationError`. `getTriples`/
`queryTriples` with an invalid node or a malformed pattern → `ValidationError`.
A triple whose subject/object node is deleted → the triple is removed (cascade
on node delete, consistent with the reference-integrity rule §4.4.5).

**4.2.8 Manual override for graph operations (pinned requirement).** **Non-vector
operations in the graph space always have a manual access/override option.** For
every graph operation that has an automatic/computed form, an agent or user can
**directly declare the result** — the manual declaration is a first-class,
authoritative input, never a fallback that the automatic computation can
overwrite.

**4.2.8.1 The rule.** For each non-vector graph operation with an automatic form,
the engine exposes a **manual override** that lets an agent/user declare the
result directly:

| Graph operation | Automatic form | Manual override |
| --- | --- | --- |
| **Community detection / summaries** | Automatic community detection + summary generation (PARKED, F4 §4.5.5) | **`declareCommunity`** — an agent/user directly declares a given set of nodes/edges as a community and provides a summary (§4.2.8.2). |
| **Fact extraction** | Automatic fact extraction from documents (§4.3.2) | **`createFact`** — an agent/user directly declares a fact + its citations (§4.3.1). |
| **Consistency derivation** | Automatic staleness propagation (§4.4.3) | **`setReferenceState`** — an agent/user directly marks a reference `FRESH`/`STALE`/`RESOLVED`/`BROKEN` (a review-aid override, §4.4.2). |
| **Chunking / segmentation** | Automatic graph-structure chunking (§4.5.3) | **`declareSegment`** — an agent/user directly declares a node/edge set as a document segment (manual segmentation, C11). |
| **Triple inference** | (future) automatic triple inference | **`addTriple`** — an agent/user directly declares a triple (§4.2.7.3). |

**4.2.8.2 The community example (pinned).** It MUST be possible for an agent or
user to **directly declare a given set as a community and provide a summary**,
independent of any automatic community detection:

- **`declareCommunity`** — `(nodeIds: [{documentId, nodeId}], {summary, wikiId}) → Community`. The agent/user declares the given node set as a community and provides the summary. The declaration is stored in the authoritative graph (a `community` node/edge structure, §4.2.8.3).
- **`getCommunity`** — `(communityId) → Community` — returns the declared community + its summary.
- **`listCommunities`** — `(wikiId) → Community[]` — lists all declared communities.
- **`updateCommunitySummary`** — `(communityId, summary) → Community` — updates a declared community's summary.

**4.2.8.3 Community storage.** A declared community is stored in the
authoritative graph as a **`community` node** (a first-class node kind) whose
members are the declared node set (via `member` edges) and whose `summary`
property is the provided summary. It is a node/edge structure, NOT an external
index — consistent with the triple ownership model (§4.2.7.2). A community's
summary is a **derived projection** of its members for consistency purposes
(§4.4.1a): when a member changes, the community is marked `STALE` until
re-derived — but the **manual declaration and the provided summary are
authoritative** and are never overwritten by an automatic computation.

**4.2.8.4 Manual override precedence.** The manual declaration is the **single
source of truth** for the declared result. An automatic computation (community
detection, fact extraction, consistency derivation, chunking) is an
**alternative** that proposes a result; it never overwrites a manual
declaration. Where both exist, the manual declaration wins. This is the
"manual access/override" guarantee: an agent/user can always directly declare
the graph structure they want, without depending on the automatic computation.

**4.2.8.5 Manual-override fail-states.** `declareCommunity` with an empty node
set or an empty `summary` → `ValidationError`. `getCommunity`/`listCommunities`/
`updateCommunitySummary` with an unknown `communityId` → `CommunityNotFound`.
`setReferenceState` with an invalid state → `ValidationError`.

**4.2.9 Entity resolution / dedup / graph enrichment.** The knowledge graph
supports **entity resolution** — merging or aliasing duplicate entities/relations
that refer to the same real-world object (LightRAG's graph operation D(·),
`docs/research/lightrag-research-notes.md` §2.2; the graph-enrichments report's
entity-resolution SHOULD, `docs/research/2026-09-08-graph-rag-when-vectors/reports/graph-enrichments.md`
§5). This directly strengthens the single-source-of-truth invariant (§4.4.1):
two `fact` nodes in the same Wiki (or across wikis) that refer to the same entity
under different keys ("license" vs "licence") can be merged or aliased.

**4.2.9.1 The entity-resolution operation.** The engine exposes:

| Operation | Signature | Return shape | Throw / fail-state |
| --- | --- | --- | --- |
| `resolveEntities` | `(entityIds: [{documentId, nodeId}], {canonicalId?, wikiId}) → ResolutionResult` | `{merged: [{from, to}], aliases: [{alias, canonical}], canonicalId}` | `DocumentNotFound` if an entity's document is unknown; `ValidationError` if `entityIds` empty or `canonicalId` invalid |
| `mergeFacts` | `(factKeys: string[], {canonicalKey, wikiId}) → Fact` | The merged `Fact` (the canonical fact with the merged value + the union of citations) | `DocumentNotFound`; `ValidationError` if `factKeys` empty or `canonicalKey` invalid; `ConflictError` if the facts have conflicting values and no resolution is provided |

**`resolveEntities`** merges or aliases the given entity nodes: a `canonicalId`
(if provided) becomes the canonical entity; the others become aliases pointing to
it. **`mergeFacts`** merges duplicate `fact` nodes: the canonical fact keeps the
canonical value (or a resolved value), and the union of the merged facts'
citations becomes the canonical fact's citations (§4.3.2). A merge with
conflicting values and no resolution → `ConflictError`.

**4.2.9.2 Manual override (per §4.2.8).** Entity resolution is a **non-vector
graph operation** and therefore has a **manual access/override option** (§4.2.8):
an agent/user can directly declare which entities are duplicates and which is
canonical via `resolveEntities`/`mergeFacts`. The manual declaration is
authoritative and never overwritten by an automatic dedup computation.

**4.2.9.3 Automatic dedup (optional, derived).** The engine MAY provide an
automatic dedup/enrichment pass (entity resolution over `fact` nodes) as a
**derived, rebuildable** operation — the same pattern as the lexical/vector/triple
indexes (§4.2.7.2). It proposes merges; it never overwrites a manual declaration
(§4.2.8.4). The automatic pass is a SHOULD-HAVE (per the graph-enrichments
report); the manual override is the pinned requirement.

**4.2.9.4 Entity-resolution fail-states.** `resolveEntities`/`mergeFacts` with
an empty entity/fact set or an invalid canonical id → `ValidationError`.
`mergeFacts` with conflicting values and no resolution → `ConflictError`. A
merged fact's citations must satisfy the minimum-citation invariant (§4.3.2) —
a merge that would leave the canonical fact with zero citations →
`ValidationError`.

### 4.3 Fact/citation tracking

**4.3.1 Fact nodes.** A `fact` node is the single source of truth for a fact. It
carries:
- **`factKey`** — a stable string, unique within the Wiki. The exact-match
  retrieval target (the lexical leg, §4.5.3).
- **`value`** — the canonical value of the fact.
- **`documentId`** / **`nodeId`** — the fact's location in the store.
- **`citations`** — the grounding set the fact was **extracted from** (§4.3.2).
  A fact MUST have **at least one citation**.
- **`updatedAt`** — ISO-8601 UTC timestamp of the last value change.

**4.3.2 Fact extraction + citations (the grounding set).** A fact is
**extracted from a document** (or documents) and **links the document(s) and
their most immediately relevant nodes as the citation(s)**. The citation set is
the grounding evidence for the fact — the source nodes the fact was derived
from.

- **Extraction:** a fact is created by extracting a canonical value from one or
  more source documents. The extraction records the **source document(s)** and
  the **most immediately relevant node(s)** within them (the nodes whose content
  the fact was derived from) as the fact's `citations`.
- **Citation shape:** `citations: [{documentId, nodeId}]` — the deduplicated
  grounding set of the source document(s) + their most immediately relevant
  nodes. Order is by first appearance; duplicates are removed.
- **Minimum citation invariant:** **a fact MUST have at least one citation.**
  `createFact`/`updateFact` with an empty `citations` set → `ValidationError`
  (`fact requires at least one citation`). A fact with no citation is not a valid
  fact — it has no grounding evidence.
- **The `RagResult.citations` field:** every `RagResult` carries a `citations`
  field — the deduplicated grounding set of the result set. For a fact returned
  in a result, the fact's own `citations` (§4.3.2) are the grounding set. In
  `mode: 'graph'` (§4.5.2) it is the deduped set of resolved target nodes the
  traversal reached. Order is by first appearance; duplicates are removed.

**4.3.2a Candidate-fact extraction + deterministic validation.** Fact extraction
uses the **"light LLM proposes, deterministic code disposes"** pattern
(`docs/research/agent-memory-research-notes.md` §3.1; the candidate-fact
pipeline). A candidate fact is **proposed** by a light LLM (or a manual
`createFact`), then **confirmed or rejected by a deterministic validation gate**
before it is committed to the store.

**4.3.2a.1 The two-stage pipeline.**
1. **Propose.** A light LLM (or an agent/user via `createFact`) proposes a
   candidate fact: `{factKey, value, citations}`.
2. **Validate (deterministic, fail-closed).** A deterministic gate confirms or
   rejects the candidate before commit. The gate checks:
   - **Schema conformance** — the candidate has a valid `factKey`, a non-empty
     `value`, and a non-empty `citations` set (§4.3.2 minimum-citation
     invariant).
   - **Grounding / provenance** — every citation `{documentId, nodeId}` resolves
     to a real node in the store (no dangling citations).
   - **Dedup / conflict** — the `factKey` is unique within the Wiki (§4.3.1); a
     duplicate key or a conflicting value is rejected (or routed to entity
     resolution, §4.2.9).
   - **Cross-field consistency** — the value is consistent with the cited nodes'
     content (a grounding check).

**4.3.2a.2 Fail-closed + machine-actionable rejection.** The gate is
**fail-closed**: a candidate that fails any check is **rejected** — it is NOT
committed. The rejection returns a **machine-actionable reason** (a structured
`{code, field, message}`), so an agent/user can correct and re-submit. A
rejected candidate never partially commits.

**4.3.2a.3 Manual override (per §4.2.8).** Fact creation is a **non-vector graph
operation** and therefore has a **manual access/override option** (§4.2.8): an
agent/user can directly declare a fact via `createFact`. The manual declaration
still passes the deterministic validation gate (schema/grounding/dedup) — the
gate is a data-integrity check, not a policy that a manual declaration bypasses.
The manual declaration is authoritative and never overwritten by an automatic
extraction (§4.2.8.4).

**4.3.2a.4 Candidate-fact fail-states.** A candidate that fails schema
conformance → `ValidationError` (with the machine-actionable reason). A candidate
with a dangling citation → `ValidationError` (`citation does not resolve`). A
candidate with a duplicate `factKey` → `ConflictError` (or routed to entity
resolution, §4.2.9).

**4.3.3 Provenance (the `trace`).** Every `RagResult` carries a **`trace`** —
the retrieval path walked. Per-mode:
- **flat mode:** `{mode: 'flat', engine: 'gnosis', topK, source}` — a descriptor
  of the retrieval call (`engine` = `'gnosis'`; `source` = the RAG source
  `'local'|'zodiac'`; `topK` = the requested top-K).
- **graph mode (§4.5.2):** `[{from: {documentId, nodeId}, to: {documentId,
  nodeId}, edge: 'link'|'embed'|'crosslink', state: 'FRESH'|'RESOLVED'|'STALE'|
  'BROKEN'}]` — the ordered `reference`→`fact` path walked, in traversal order.
- **vector mode (§4.5.1):** `{mode: 'vector', engine: 'gnosis', topK, source}`.
- **hybrid mode (§4.5.1):** `{mode: 'hybrid', engine: 'gnosis', legs:
  ['graph','vector','lexical'], topK, source}`.

**Fail-state `TraceUnavailable`:** if the engine produces a result without a
`trace`, `ragQuery`/`ragStream` fail with `TraceUnavailable` (§6 FS-10). A
result set without a trace is not a valid `RagResult`.

**4.3.4 The query audit log.** `getQueryAuditLog()` returns
`[{query, filters, mode, resultCount, timestamp, requester}]` — every
`ragQuery`/`ragStream` call with the `query` text, the `filters` used (or
`null`), the `mode` (`'flat'|'graph'|'vector'|'hybrid'`), the `resultCount`
returned, the `timestamp` (ISO-8601 UTC), and the `requester` (the GUI user id
or the MCP caller identity). The audit-log **recording** is engine-side (it
records engine queries, per the engine/shell boundary analysis §5.3); the MCP
`get_query_audit_log` tool + GUI audit panel are shell-side (D4 parity).

### 4.4 Consistency enforcement

**4.4.1 The consistency invariant.** The invariant:
> **A `fact`'s canonical `value` is the single source of truth. Every `embed`
> that snapshots it must reflect the current canonical value, and every `link`
> must resolve to a live, non-stale target.**

This is the engine-side enforcement of the Astrographer contract's §4.2.3
consistency model. The engine operates on the graph data; only the consistency
panel GUI is shell-side (per the engine/shell boundary analysis §2 A2).

**4.4.1a Staleness propagation (the full consistency surface).** Facts are **one
part** of the document consistency enforcement. Staleness propagates through
**all** of the following, and the engine must track each:

- **Facts.** A `fact`'s `value` is the single source of truth. When a fact's
  value changes, every dependent that references it becomes stale (§4.4.3).
- **Links and embeds.** A `link`/`embed`/`crosslink` edge that references a
  changed target becomes `STALE` (an `embed` whose snapshot differs) or
  `BROKEN` (a `link` whose target is missing/archived) (§4.4.2).
- **Graph-RAG communities.** A **community** (a cluster of related nodes/edges,
  e.g. a wiki-level or document-cluster aggregation, §4.5.5) that incorporates a
  changed fact/reference can also propagate staleness: when a member node or edge
  of a community changes, the community's derived summary/aggregation is marked
  `STALE` until re-derived. Community staleness is a **derived** state (the
  community summary is a projection of its members), so it is re-derived on
  member change or on demand.

The consistency report (§4.4.4) surfaces all three propagators: fact staleness,
reference (link/embed/crosslink) staleness, and community staleness.

**4.4.2 Reference states.** A `link`/`embed`/`crosslink` edge is in exactly one
of:
- **`FRESH`** — an `embed` whose snapshot equals the canonical value.
- **`STALE`** — an `embed` whose snapshot differs from the canonical value.
- **`RESOLVED`** — a `link` whose target exists and is not archived.
- **`BROKEN`** — a `link` whose target is missing or archived.

**4.4.3 Enforcement rules.**
- **On fact update:** when a `fact` node's `value` changes, the engine computes
  the set of all `embed` nodes (in any document of the Wiki, and across wikis)
  that snapshot that fact, and marks each as **`STALE`**. It also marks any
  **community** that incorporates the fact as **`STALE`** (its derived summary
  is out of date).
- **On reference update:** when a `link`/`embed`/`crosslink` edge's target
  changes (or is deleted/archived), the edge is marked `STALE` (embed) or
  `BROKEN` (link). A `BROKEN`/`STALE` reference is never silently traversed
  (§4.5.2).
- **On community member change:** when a node or edge that is a member of a
  community changes, the community's derived summary/aggregation is marked
  **`STALE`** until re-derived.
- **Re-sync:** a `STALE` embed is re-synced (snapshot updated to the canonical
  value) either automatically (on next render/publish) or on explicit user
  action. A `STALE` community is re-derived (its summary regenerated from its
  members). Re-sync bumps the referencing document's `revision`.
- **Publish gate:** `publishDocument` (§4.1.3) **fails with
  `UnresolvedReference`** if the document contains any `BROKEN` link, any
  `STALE` embed that has not been re-synced, or any `STALE` community that has
  not been re-derived. This is the enforcement point that guarantees published
  documentation is consistent.

**4.4.4 The consistency report.** `getConsistencyReport(wikiId)` returns the set
of `{documentId, nodeId, kind: 'link'|'embed'|'crosslink', state: 'BROKEN'|
'STALE'|'FRESH'|'RESOLVED', target, crossWiki}` for every reference in the wiki.
This is the "documentation consistency" surface the brief names and the
explainability surface (the talk's "sourcing and tracing" made mechanical,
`docs/research/graph-rag-when-vectors-arent-enough-research-notes.md` §3.1).

**4.4.5 Reference integrity on delete.** `deleteDocument` (§4.1.3) fails with
`DocumentInUse` if any other document contains a `link`/`embed`/`crosslink`
whose `target` is in the document being deleted. The user must first remove or
re-point those references. This prevents dangling references.

### 4.5 RAG/agent-memory retrieval

**4.5.1 Query modes.** The engine supports four query modes, consistent with the
Zodiac companion's `mode: graph|vector|hybrid` surface (`docs/specs/zodiac.md`
§4.3.1) and extending it with the base `flat` mode:

- **`flat`** — top-k retrieval. Returns the top-k results by combined score.
  This is the base mode; the engine computes scores from the configured legs.
- **`graph`** — multi-hop traversal over the `reference`→`fact` graph. A
  deterministic, hop-limited walk (§4.5.2).
- **`vector`** — embedding similarity. Dense vector retrieval only (the vector
  leg, §4.5.3).
- **`hybrid`** — RRF fusion of the graph + vector + lexical legs (§4.5.3).

**4.5.2 Multi-hop traversal (`mode: 'graph'`).** A deterministic, hop-limited
walk over the `reference`→`fact` graph (GAP-7, re-scoped to the engine — NOT an
Incanter change; Incanter is archived). This is a **local, deterministic walk**
over the Provident graph (D2): no graph database, no query language, no external
service.

- `maxHops` is 1–5, default 3. The walk resolves through `FRESH`/`RESOLVED`
  references and **surfaces** `BROKEN`/`STALE` in the trace (never silently
  traverses a stale embed) — honoring the §4.4.1 consistency invariant.
- A traversal that resolves no target returns an empty result (`results: []`,
  `citations: []`) with `blockedBy: [{documentId, nodeId, state}]` listing the
  `BROKEN`/`STALE` nodes that blocked the walk. This is a **valid state, not an
  error** (consistent with Zodiac's empty-result pattern, `docs/specs/zodiac.md`
  §6.3).
- **`filters?` shape (pinned):** `{nodeKind?: 'content'|'fact'|'reference',
  edgeType?: 'link'|'embed'|'crosslink', target?: {documentId, nodeId}, state?:
  'FRESH'|'RESOLVED'|'STALE'|'BROKEN'}`. `wikiId` is already a top-level param.
  Filters restrict which nodes/edges the walk (or flat/vector/hybrid retrieval)
  considers.
- **`expand: 'parent'`** — parent-context return: return the parent Document (or
  node-cluster) for a retrieved child. Capped by `maxParentContext` (default 5
  expanded hits): the top `maxParentContext` results by `score` carry a `parent`
  field; results beyond the cap are returned without one. **Stale-propagating:**
  a `STALE` embed's parent carries `stale: true` so the generator does not trust
  stale content.
- **Result item shape with `expand: 'parent':`** each result gains
  `parent?: {documentId, title, snippet, stale}` (present only for the capped
  expanded hits).
- **Fail-states:** `HopLimitExceeded` when the walk exceeds `maxHops` without
  resolving a target (§6 FS-11); `CycleDetected` when the walk detects a
  `reference`→`fact` cycle (a node already on the current path; DAG-RAG pitfall
  3, `docs/research/dag-rag-research-notes.md` §5) (§6 FS-12); `ValidationError`
  extended for invalid `mode`, `maxHops` out of range, or malformed `filters`
  (§6 FS-3).

**4.5.3 The retrieval stack.** The engine's retrieval stack (per the engine
work-package, `docs/research/astrographer-engine-shell-boundary.md` §3 "Vector /
lexical data"):

- **Lexical BM25 leg.** A BM25 index over `factKey`/`title`/`tags`/node text,
  built at ingestion time and updated on document edit/delete. Exact-match
  retrieval for fact keys, IDs, technical terms, and rare proper nouns (B1.4,
  `docs/research/hybrid-search-research-notes.md` §9). This is the engine-side
  home of the "local lexical leg" (the Incanter three-way fusion D5 is moot —
  Incanter is archived; the lexical leg is now Gnosis's own).
- **Vector leg.** Embeddings + provider calls + the vector index + cosine/ANN
  scoring + the embedding cache. The dense leg uses a local embedding provider
  (e.g. Ollama, default `http://127.0.0.1:11434`). One consistent embedding
  model across index and query (avoid model drift). A node/chunk can carry
  **multiple vector fields** (§4.5.3a) — e.g. a full-length high-fidelity vector
  and a compressed binary vector for fast first-pass ANN search in large stores.
- **Graph leg.** Graph-structural relevance over the `reference`→`fact` graph
  (the multi-hop traversal, §4.5.2).
- **RRF fusion.** The `hybrid` mode merges the graph + vector + lexical legs by
  **Reciprocal Rank Fusion**: `RRF(d) = Σ_{r ∈ R} 1 / (k + rank_r(d))` where `R`
  is the set of ranked lists, `rank_r(d)` is the 1-based rank of item `d` in
  list `r`, and `k` is a fixed constant (default `k = 60`). The final result set
  is the top-`topK` items by descending RRF score; ties are broken
  deterministically by `(documentId, nodeId)` ascending. This is the **exact
  merge rule** — a TestWriter can derive the merged ordering from the input
  lists (consistent with Zodiac's pinned RRF rule, `docs/specs/zodiac.md`
  §4.3.2.1).
- **Cross-encoder reranking.** A two-stage retrieve-and-rerank: the first stage
  retrieves a generous candidate set (capped, e.g. top 10–50), then a small
  local cross-encoder (e.g. `bge-reranker-v2-m3` or `ms-marco-MiniLM-L-6-v2`)
  re-scores each candidate jointly with the query (B1.5,
  `docs/research/reranking-research-notes.md` §8). Query-time only; the
  reranker's ceiling is bounded by first-stage recall.
- **Multi-query merge.** Query fan-out: the engine generates N query variants
  (default `n: 3`, opt-in/configurable), retrieves for each, and merges by
  stable `(documentId, nodeId)` identity (B1.6,
  `docs/research/multi-query-retrieval-research-notes.md` §8). The merge is
  exact (stable node identities), not fuzzy.
- **Contextual compression.** A post-retrieval, pre-generation stage. `filter`
  mode (binary keep/drop) is Phase 1; `extract` mode (fact-value extraction) is
  Phase 2; `graph` mode (sub-structure compression) is Phase 3 (B1.7,
  `docs/research/contextual-compression-research-notes.md` §8). The compressor
  is query-aware and reference-aware (a `STALE` embed's compressed output
  carries a staleness marker). On compressor failure, the engine **degrades
  gracefully to uncompressed context** (not a query failure).
- **HyDE (opt-in).** A `hyde` query-mode wrapper: generate a hypothetical
  fact/graph snippet via the local LLM, embed it, and route the vector into the
  chosen source (B1.8, `docs/research/hyde-research-notes.md` §6.4). Opt-in, not
  default, to protect latency. Configurable toggle + latency budget.
- **Adaptive RAG routing (PARKED, F3).** A query-complexity router deciding
  between local graph RAG, local-only, and Zodiac, plus an answer-adequacy
  qualifier (B1.9, `docs/pending.md` D3). **PARKED** — noted here as a parked
  retrieval-stack layer (§7.4).
- **Inference-time sub-task DAG (SHOULD HAVE).** A query-decomposition stage
  (LogicRAG-style, `docs/research/dag-rag-research-notes.md` §4): decompose a
  complex multi-hop query into a **sub-problem DAG**, topologically sort it, and
  drive per-node retrieval/answer in dependency order, reusing resolved
  sub-answers. This is **distinct** from the deterministic `graph` walk (§4.5.2,
  which walks the *pre-existing* `reference`→`fact` graph) and from multi-query
  fan-out (§4.5.3, which generates query variants). It builds a **query-derived**
  sub-problem DAG at inference time. **SHOULD HAVE** — a high-value enhancement
  for complex multi-hop wiki questions; not a prerequisite for the core
  retrieval foundation. Opt-in via a `subTaskDag?: {enabled}` param (§4.6.1);
  fail-state `SubTaskDagFailed` (§6).

**4.5.3a Multiple vector fields (coarse-to-fine).** A node/chunk can carry
**multiple vector fields** — distinct vector representations of the same content,
each with a different purpose. The canonical example: a **full-length
high-fidelity vector** (accurate cosine similarity) and a **compressed binary
vector** (fast first-pass ANN search) for large stores.

**4.5.3a.1 The vector-field model.** A node/chunk's vector representation is a
set of **named vector fields**, each with a `fieldType`:

| Field type | Purpose | Example |
| --- | --- | --- |
| **`full`** | The full-length, high-fidelity embedding (accurate cosine similarity). | A 768-dim dense vector from the embedding provider. |
| **`binary`** | A compressed binary vector for fast first-pass ANN search (Hamming distance / binary quantization). | A sign-binarized / binary-quantized code derived from the `full` vector. |
| **`other`** | Any additional named vector field (e.g. a per-model or per-domain embedding). | A second model's embedding for a specific retrieval surface. |

A node/chunk carries at least the `full` field; the `binary` (and any `other`)
fields are **optional, additive** — a node without them is fully valid and
retrievable via the `full` field alone.

**4.5.3a.2 The vector index.** The vector index stores **multiple vector fields
per node/chunk** — each field is a separate indexable representation. The index
is keyed by `(documentId, nodeId, fieldType)`. A field is derived from the
authoritative content (the node's text/value) and is a **derived, rebuildable
index** (the same pattern as the lexical BM25 index and the triple index,
§4.2.7.2) — never a second source of truth.

**4.5.3a.3 Coarse-to-fine retrieval (the binary first-pass).** The engine
supports a **coarse-to-fine / approximate-nearest-neighbor (ANN)** retrieval
path for large stores:

1. **First pass (coarse):** the query's `binary` code is compared (e.g. Hamming
   distance) against all nodes' `binary` codes to cheaply select a **candidate
   subset** (a narrowed pool, e.g. the top-N by binary distance).
2. **Second pass (fine):** the full-precision cosine similarity is computed
   **only over the candidate subset** using the `full` vectors, producing the
   accurate top-k.

This is a **performance optimization** for large stores: the binary pass is fast
and memory-light, the full pass is accurate but only on the candidates. The
candidate-subset size is configurable (a `binaryCandidatePool` cap, default
e.g. 10× `topK`). The coarse-to-fine path is **opt-in** (a `vectorSearch` mode
or a `binaryFirstPass` flag); the default `vector`/`hybrid` modes use the `full`
field directly.

**4.5.3a.4 Multiple-vector-field fail-states.** A node/chunk with a malformed
`binary` field (wrong bit-length, non-binary) → the field is skipped at index
build (never loaded), consistent with the boot-skip discipline. A `binaryFirstPass`
query when the `binary` index is not built → **degrades to the `full`-field
search** (not an error). A `binaryFirstPass` query with an invalid
`binaryCandidatePool` → `ValidationError`.

**4.5.4 The agent-memory use case.** Gnosis provides the agent-memory retrieval
surface over the knowledge graph (per the agent-memory research,
`docs/research/agent-memory-research-notes.md` §3.4 — the conceptual mapping of
the 3-layer model onto the document store). The engine exposes:

- **Facts table.** The `fact` nodes (§4.3.1) are individually addressable
  records — the "facts as first-class objects" pattern. An agent retrieves a
  fact by its exact `factKey` (exact-reference, not embedding similarity — the
  "similarity ≠ relevance" fix).
- **Profile summary.** A **derived document** regenerated from the facts table
  (a projection, single source of truth, no drift). Regenerated on fact change
  or on demand.

The agent-memory retrieval surface:

| Operation | Signature | Return shape | Throw / fail-state |
| --- | --- | --- | --- |
| `getFact` | `(factKey, {wikiId}) → Fact` | `{factKey, value, documentId, nodeId, updatedAt, citations}` | `DocumentNotFound` if the fact's document is unknown; `ValidationError` if `factKey` empty or `wikiId` unknown |
| `listFacts` | `(wikiId, {state?, page?, pageSize?}) → {items, total, page, pageSize}` | Paginated list of `Fact` nodes | `WikiNotFound`; `ValidationError` if `page < 1` or `pageSize < 1` or `pageSize > 100` |
| `getProfileSummary` | `(wikiId) → ProfileSummary` | `{wikiId, summary, regeneratedAt, factCount}` | `WikiNotFound` |

**4.5.5 Parked retrieval surfaces (noted, not in scope).** Community summaries
(F4, `docs/pending.md` D8) and LLM-generated dynamic graph query (F5,
`docs/pending.md` D9) are **PARKED** and noted here as parked retrieval surfaces
(§7.5, §7.6). They are NOT part of this contract's behavior surface.

**Note (manual override):** the **automatic** community-detection + summary
generation is PARKED (F4), but the **manual** community declaration is a
**pinned requirement** (§4.2.8): an agent/user can directly declare a given set
as a community and provide a summary via `declareCommunity`/`getCommunity`/
`listCommunities`/`updateCommunitySummary`, independent of the parked automatic
detection. The manual declaration is authoritative and never overwritten by an
automatic computation (§4.2.8.4).

### 4.6 Query modes/surfaces

**4.6.1 The full query surface (the retrieval seam).** The shell proxies the
engine's query/stream/engine-status API. These are the RAG surface operations
Gnosis exposes to the shell; the shell provides the MCP + GUI surfaces that
proxy them (§4.6.2).

| Operation | Signature | Return shape | Throw / fail-state |
| --- | --- | --- | --- |
| `ragQuery` | `(query, {wikiId?, topK?, filters?, mode?: 'flat'|'graph'|'vector'|'hybrid', maxHops?, expand?: 'none'|'parent', maxParentContext?, multiQuery?: {enabled, n}, compression?: 'none'|'filter'|'extract'|'graph', hyde?: boolean, binaryFirstPass?: boolean, binaryCandidatePool?: number, subTaskDag?: {enabled}}) → RagResult` | `{query, results: [{documentId, nodeId, score, snippet, source: 'local'|'zodiac', parent?, stale?}], engine: 'gnosis', citations: [{documentId, nodeId}], trace, blockedBy?}` | `EngineUnavailable` if the engine is not `READY`; `ValidationError` if `query` empty or `topK < 1` or `topK > 50`, or `mode` invalid, or `maxHops` out of range, or `filters` malformed, or `multiQuery.n` out of range, or `compression` invalid, or `hyde` not boolean, or `binaryFirstPass` not boolean, or `binaryCandidatePool` not a positive integer, or `subTaskDag` not a boolean-object; `EngineError` if the engine returns a malformed result; `TraceUnavailable` if the engine returns results without a `trace`; `HopLimitExceeded` if a graph traversal exceeds `maxHops`; `CycleDetected` if a graph traversal detects a `reference`→`fact` cycle; `EmbeddingUnavailable` if the embedding provider is unreachable and a vector/hybrid/hyde leg requires it; `VectorIndexUnavailable` if the vector index is not built; `LexicalIndexUnavailable` if the BM25 index is not built; `RerankerUnavailable` if the reranker model is unavailable; `CompressionFailed` if the compressor fails and cannot degrade; `HyDEGenerationFailed` if HyDE hypothetical-doc generation fails; `MultiQueryExpansionFailed` if query expansion fails; `SubTaskDagFailed` if the sub-task DAG decomposition fails |
| `ragStream` | `(query, {wikiId?, topK?, filters?, mode?, maxHops?, expand?, maxParentContext?, multiQuery?, compression?, hyde?}) → AsyncIterable<RagChunk>` | Stream of `RagChunk` (`{type: 'result'|'done'|'error', ...}`) over SSE; each `result` chunk carries the full `RagResult` (with `citations`/`trace`) | `EngineUnavailable` if not `READY`; `EngineError` on mid-stream failure (emitted as a `type: 'error'` chunk, then the stream closes); `TraceUnavailable`/`HopLimitExceeded`/`CycleDetected`/`EmbeddingUnavailable`/`VectorIndexUnavailable`/`LexicalIndexUnavailable`/`RerankerUnavailable`/`CompressionFailed`/`HyDEGenerationFailed`/`MultiQueryExpansionFailed` emitted as a `type: 'error'` chunk, then the stream closes |
| `getEngineStatus` | `() → {state, version, subsystems: {store, graph, lexical, vector, embedding, reranker}, lastError?}` | Engine health state | — |
| `getQueryAuditLog` | `() → QueryAuditEntry[]` | `[{query, filters, mode, resultCount, timestamp, requester}]` | — |

**Engine connection state.** The engine's health state is one of:
`READY` / `STARTING` / `DEGRADED` / `UNAVAILABLE`. `READY` = all subsystems
operational. `STARTING` = engine booting. `DEGRADED` = a non-core subsystem
(e.g. the embedding provider) is down; core store/graph/lexical function works.
`UNAVAILABLE` = the engine is not running or unreachable. The engine is
**optional** (D2): the shell's document store and cross-link features work fully
without the engine; only RAG query features require it. `getEngineStatus`
reports the per-subsystem state so the shell can surface `DEGRADED` distinctly.

**4.6.2 No MCP surface, no GUI surface (Gnosis is a pure backend).** Gnosis has
**no MCP surface and no GUI surface planned** — it has no UI. It is a **pure
backend to Astrographer** and **uses Astrographer as its front-end**. The D4
MCP-GUI parity requirement therefore applies at the **Astrographer shell**, not
at Gnosis:

- **Gnosis exposes its API (§4.6.1) to the shell** over the `RagStore` +
  query/stream/engine-status seam (§5.1). It does NOT expose MCP tools or a GUI.
- **The Astrographer shell provides the MCP + GUI surfaces** that proxy Gnosis's
  API (the document/wiki/consistency/RAG/facts/profile/engine-status tools and
  screens). D4 parity is a shell-side contract (`docs/specs/astrographer.md`
  §4.5/§4.6), not a Gnosis-side one.
- **The security-configuration carve-out (decision D4-CARVEOUT-SCOPE)** — engine
  credentials, Astral push credentials, TLS/secret management, Firmament bridge
  auth — is **GUI-only at the shell** and blocked from MCP access there. Gnosis
  itself holds no user-facing credentials; the shell owns the credential
  handling for the engine's outbound connections (embedding/LLM provider, Astral
  push, Firmament bridge).

This is a deliberate boundary: Gnosis is a headless engine; all human/agent
interaction is mediated by the Astrographer shell.

---

## 5. Integration contract

Each directed edge from `docs/integration-matrix.md` §3.2 that involves Gnosis,
with the mechanism and the value. Section numbers cross-reference the behavior
contract (§4).

### 5.1 Edge — Astrographer (shell) → Gnosis (engine) — the proxy seam

- **Mechanism:** the shell proxies engine calls over the `RagStore` interface
  (§4.1.5) + the query/stream/engine-status API (§4.6.1). This is the natural
  IPC/process boundary (per the engine/shell boundary analysis §5.1; C3's
  `createRemoteRagStore` generalizes to `createEngineRagStore`).
- **Value:** the shell gains the full engine work-package (document store,
  knowledge graph, fact/citation tracking, consistency enforcement, RAG/agent-
  memory retrieval) without owning the graph/vector data or math.
- **Contract refs:** §4.1.5 (`RagStore`), §4.6.1 (query/stream/engine-status),
  §4.6.2 (the shell provides the MCP + GUI proxy surfaces).
- **Fail-states:** §6 (`EngineUnavailable`, `EngineError`, and the retrieval
  fail-states). The engine is optional (D2) — the shell's document store and
  cross-link features work without it.

### 5.2 Edge — Gnosis → Astral (via the shell's push)

- **Mechanism:** the shell's `pushToAstral` (§4.4 of the Astrographer contract)
  pushes published documents to Astral as `provident-graph/1` (decision
  GRAPH-PUSH-FORMAT), idempotent via `contentHash`. The **serialization** is
  engine-side (a data export); the **HTTP push transport + push-status UI** is
  shell-side (per the engine/shell boundary analysis §2 A4).
- **Value:** Gnosis's published documents become Astral-hosted wiki pages
  automatically.
- **Contract refs:** §4.1.3 `publishDocument` (publish gate), §4.4.3
  (`UnresolvedReference` blocks publish), §4.6.2 (push credentials are GUI-only
  at the shell).
- **Fail-states:** §6 (`PushRejected`, `AstralUnavailable`, `SchemaMismatch`).

### 5.3 Edge — Zodiac → Gnosis (a second RAG data source)

- **Mechanism:** Zodiac (the remote RAG companion) replies to queries with
  pre-graphed, vector-embedded data crawled from the web. Gnosis's query modes
  are consistent with Zodiac's `mode: graph|vector|hybrid` surface
  (`docs/specs/zodiac.md` §4.3.1). Zodiac results are a second `source`
  (`'zodiac'`) alongside the engine's local results (`'local'`).
- **Value:** Gnosis gains access to non-local, current data without
  compromising its local-first core (D2).
- **Contract refs:** §4.5.1 (query modes), §4.6.1 (`source: 'local'|'zodiac'`).
- **Fail-states:** §6 (`ZodiacUnavailable` — a query degrades to local-only
  results, not an error; consistent with the Astrographer contract §5.7).

### 5.4 Edge — Solomon ↔ Gnosis (cross-instance search)

- **Mechanism:** Solomon links and searches across instances. Gnosis's data
  (documents, facts) is searchable via `solomon_search` with `instanceTypes`
  including `astrographer` (the shell's instance type). The engine's stable
  `(documentId, nodeId)` identities make cross-instance result merging exact.
- **Value:** cross-instance discovery of Gnosis's documentation that no single
  local instance can provide alone.
- **Contract refs:** `docs/decisions.md` SOLOMON-TRUST-PHASING; `docs/specs/
  solomon.md` §5.1.
- **Fail-states:** peer auth failure (GUI-only per `docs/integration-matrix.md`
  §5); peer unreachable; no peers joined (search returns empty, not an error).

### 5.5 Edge — Emerald ↔ Gnosis (graph editing round-trip)

- **Mechanism:** Emerald edits/builds on Gnosis's documentation graphs. The
  graph-push format is the canonical `provident-graph/1` contract (decision
  GRAPH-PUSH-FORMAT). Emerald edits must respect `revision` (optimistic
  concurrency, §4.1.4) and must not break the consistency invariant (§4.4).
- **Value:** Emerald (the web development app) can build on Gnosis's
  documentation graphs.
- **Contract refs:** §4.1.4 (optimistic concurrency), §4.4 (consistency).
- **Fail-states:** §6 (`ConflictError`, `UnresolvedReference`, `SchemaMismatch`).

### 5.6 Edge — Firmament → Gnosis (secure remote bridge)

- **Mechanism:** Firmament creates a secure bridge to the local instance
  (tunnel/address-proxy, FIRMAMENT-DATAFLOW; mTLS + optional end-to-end
  mirroring, FIRMAMENT-AUTH). The bridge credentials are GUI-only at the shell
  (§4.6.2).
- **Value:** a local Gnosis-backed instance gains a reachable remote presence
  without compromising D2 local-first.
- **Contract refs:** `docs/decisions.md` FIRMAMENT-AUTH, FIRMAMENT-DATAFLOW;
  §4.6.2 (bridge credentials GUI-only at the shell).
- **Fail-states:** bridge auth failure (GUI-only config); bridge unreachable;
  offline caching (optional) serves cached data when the local instance is
  offline.

### 5.7 Edge — Familiar → Gnosis (knowledge retrieval + document creation + optional memory store)

- **Mechanism:** Familiar's knowledge retrieval (F1) and research (F4) route
  through the shell's RAG surface, which proxies Gnosis. Familiar's knowledge
  memory delegates to the document store (decision FAMILIAR-ASSISTANT-CORE-
  SCOPE). The agent-memory retrieval surface (§4.5.4) serves Familiar's
  knowledge retrieval. **Optionally (decision FAMILIAR-MEMORY-STORE-INTEGRATION),**
  Familiar can use Gnosis (via Astrographer) as its **memory store**: the facts
  table maps to Gnosis's `fact` nodes (§4.3.1), the profile summary to Gnosis's
  derived profile summary (§4.5.4), the candidate-fact pipeline to Gnosis's
  candidate-fact + deterministic validation (§4.3.2a), and vector search over
  memory to Gnosis's RAG `vector` mode (§4.5.1). The local memory store remains
  the default; the Gnosis backing is opt-in.
- **Value:** the assistant gains graph-aware retrieval and document creation
  over Gnosis's knowledge graph, and (optionally) a scalable, graph-backed
  memory store.
- **Contract refs:** §4.5.4 (agent-memory surface), §4.6.1 (RAG surface),
  §4.3.1 (fact nodes), §4.3.2a (candidate-fact + deterministic validation).
- **Fail-states:** §6 (retrieval fail-states; the engine is optional, D2).

---

## 6. Fail-states (complete catalogue)

Every documented fail-state, with the section that defines it and the
observable behavior a TestWriter asserts.

| # | Fail-state | Defined in | Observable behavior |
| --- | --- | --- | --- |
| FS-1 | `DocumentNotFound` | §4.1.3, §4.2.5, §4.5.4 | `getDocument`/`updateDocument`/`deleteDocument`/`publishDocument`/`unpublishDocument`/`archiveDocument`/`edgesFrom`/`edgesTo`/`edgesByKind`/`edgesForDocument`/`docHeadForDocument`/`getFact` on unknown `documentId` |
| FS-2 | `WikiNotFound` | §4.1.3, §4.5.4 | `createDocument`/`listDocuments`/`getWiki`/`getConsistencyReport`/`listFacts`/`getProfileSummary` on unknown `wikiId` |
| FS-3 | `ValidationError` | §4.1.3, §4.2.5, §4.5.2, §4.5.4, §4.6.1 | empty/overlong `title`/`name`; invalid `graph`; `page < 1`; `pageSize < 1` or `> 100`; empty `query`; `topK < 1` or `> 50`; invalid `mode` (not `'flat'`/`'graph'`/`'vector'`/`'hybrid'`); `maxHops` out of range (not 1–5); malformed `filters` (invalid enum value, missing `target` fields, invalid `state`); `multiQuery.n` out of range; invalid `compression`; `hyde` not boolean; `binaryFirstPass` not boolean; `binaryCandidatePool` not a positive integer; invalid `nodeId` in an adjacency method |
| FS-4 | `ConflictError` | §4.1.4 | `updateDocument` with stale base `revision` (HTTP 409 / MCP error) |
| FS-5 | `DocumentInUse` | §4.4.5 | `deleteDocument` when another document references the target |
| FS-6 | `InvalidState` | §4.1.3, §4.2.5 | `unpublishDocument` on non-`PUBLISHED`; `archiveDocument` on `ARCHIVED`; `docHeadForDocument` on a document with no `doc-head` edge |
| FS-7 | `UnresolvedReference` | §4.4.3 | `publishDocument` when the document has `BROKEN` links or `STALE` embeds |
| FS-8 | `EngineUnavailable` | §4.6.1 | `ragQuery`/`ragStream` when the engine is not `READY` (state `UNAVAILABLE`/`STARTING`) |
| FS-9 | `EngineError` | §4.6.1 | `ragQuery`/`ragStream` when the engine returns a malformed result |
| FS-10 | `TraceUnavailable` | §4.3.3 | `ragQuery`/`ragStream` when the engine produces a result without a `trace` |
| FS-11 | `HopLimitExceeded` | §4.5.2 | `ragQuery`/`ragStream` in `mode: 'graph'` when the traversal exceeds `maxHops` without resolving a target |
| FS-12 | `CycleDetected` | §4.2.6, §4.5.2 | `ragQuery`/`ragStream` in `mode: 'graph'` (or a topological resolution) when the traversal detects a `reference`→`fact` cycle |
| FS-13 | `EmbeddingUnavailable` | §4.5.3, §4.6.1 | `ragQuery`/`ragStream` in `mode: 'vector'`/`'hybrid'` (or with `hyde: true`) when the embedding provider is unreachable |
| FS-14 | `VectorIndexUnavailable` | §4.5.3, §4.6.1 | `ragQuery`/`ragStream` in `mode: 'vector'`/`'hybrid'` when the vector index is not built |
| FS-15 | `LexicalIndexUnavailable` | §4.5.3, §4.6.1 | `ragQuery`/`ragStream` in `mode: 'hybrid'` when the BM25 index is not built |
| FS-16 | `RerankerUnavailable` | §4.5.3, §4.6.1 | `ragQuery`/`ragStream` when the reranker model is unavailable and reranking is requested |
| FS-17 | `CompressionFailed` | §4.5.3, §4.6.1 | `ragQuery`/`ragStream` when the compressor fails and cannot degrade to uncompressed context |
| FS-18 | `HyDEGenerationFailed` | §4.5.3, §4.6.1 | `ragQuery`/`ragStream` with `hyde: true` when hypothetical-doc generation fails |
| FS-19 | `MultiQueryExpansionFailed` | §4.5.3, §4.6.1 | `ragQuery`/`ragStream` with `multiQuery.enabled` when query expansion fails |
| FS-20 | `PushRejected` | §5.2 | `pushToAstral` when Astral is reachable but rejects (non-2xx, `SchemaMismatch`, auth failure) |
| FS-21 | `AstralUnavailable` | §5.2 | `pushToAstral` when Astral is unreachable |
| FS-22 | `SchemaMismatch` | §5.2 | Astral rejects a payload with unknown `schema` value |
| FS-23 | `ZodiacUnavailable` | §5.3 | a query that would use Zodiac when Zodiac is unreachable — **degrades to local-only results, not an error** |
| FS-24 | `EnrichmentBudgetExceeded` | §7.5 (parked F4) | the parked community-summary enrichment pass exceeds its D2 cost budget (noted for the parked surface; not part of the current behavior surface) |
| FS-25 | `CommunityNotFound` | §4.2.8 | `getCommunity`/`listCommunities`/`updateCommunitySummary` on an unknown `communityId` |
| FS-26 | `SubTaskDagFailed` | §4.5.3, §4.6.1 | `ragQuery`/`ragStream` with `subTaskDag.enabled` when the sub-task DAG decomposition fails |

**Fail-state completeness rule:** a TestWriter must be able to derive every
state and fail-state from this catalogue plus the operation tables in §4.1.3,
§4.2.5, §4.5.4, and §4.6.1. Any behavior not covered here or in those tables is
a spec gap and a review finding.

---

## 7. Gaps + costs-benefits

The open questions that block finalization, and the cost/benefit of resolving
each.

### 7.1 F1 — MCP surface — NOT APPLICABLE to Gnosis

- **Gap:** none for Gnosis. Gnosis has **no MCP surface and no GUI surface** — it
  is a pure backend to Astrographer and uses Astrographer as its front-end
  (§4.6.2). The suite-wide MCP surface (`docs/pending.md` #2) is a **shell-side**
  concern (the Astrographer contract §4.5/§4.6), not a Gnosis one.
- **Cost of resolving:** n/a for Gnosis — the shell owns the MCP/GUI surfaces
  that proxy Gnosis's API.
- **Benefit:** n/a — D4 parity is verified at the shell, not at the headless
  engine.

### 7.2 F2 — engine transport/API reconciliation

- **Gap:** the exact IPC/process transport between the shell and the engine
  (the `RagStore` + query/stream/engine-status seam) is not yet pinned to a
  concrete wire format. The logical API is pinned here (§4.1.5, §4.6.1); the
  transport (e.g. HTTP/REST + SSE, or a native IPC channel) is a Gnosis-repo
  decision.
- **Cost of resolving:** pinning the wire format and the SSE event schema for
  `ragStream`.
- **Benefit:** makes the proxy seam testable end-to-end; confirms the
  `EngineUnavailable`/`EngineError` split matches the real transport's failure
  modes.

### 7.3 F3 — D4 security-configuration carve-out scope — RESOLVED (decision
D4-CARVEOUT-SCOPE)

- **Status:** the D4 security-configuration carve-out (GUI-only) is the minimum
  set (Firmament auth, Augur broker creds, TLS/secret management) + per-tool
  additions. **Gnosis has no MCP surface and no GUI surface** (§4.6.2), so the
  carve-out is a **shell-side** concern: the Astrographer shell owns the
  credential handling for the engine's outbound connections (embedding/LLM
  provider, Astral push, Firmament bridge) and blocks them from MCP access
  there. This resolves `docs/pending.md` #10 at the shell.
- **Remaining work:** n/a for Gnosis — the carve-out is enforced at the shell
  (the Astrographer contract §4.5.4), not at the headless engine.
- **Benefit:** closes the one deliberate D4 exception at the shell; prevents
  accidental MCP exposure of credentials.

### 7.4 F3 — Adaptive RAG routing to Zodiac — PARKED (`docs/pending.md` D3)

- **Status:** the query-complexity router + adequacy qualifier (B1.9) is
  **PARKED**. It is noted as a parked retrieval-stack layer (§4.5.3). Land it
  only after the MUST/SHOULD retrieval foundation (reranking, multi-query,
  compression, lexical hybrid leg) ships.
- **Cost of resolving:** multiple LLM calls per query (the most complex phase).
- **Benefit:** the highest-leverage expression of D3 interconnection.

### 7.5 F4 — Community summaries — PARKED (`docs/pending.md` D8)

- **Status:** the wiki-level aggregation surface (B3.6, GAP-9) is **PARKED**.
  It is noted as a parked retrieval surface (§4.5.5). The `EnrichmentBudgetExceeded`
  fail-state (FS-24) is noted for the parked surface.
- **Cost of resolving:** a scoped, lazy, local community-summary surface is the
  D2-compatible SHOULD; the full hierarchical GraphRAG pipeline stays PARKED for
  D2 cost/latency.
- **Benefit:** answers global/theme questions over the wiki.

### 7.6 F5 — LLM-generated dynamic graph query — PARKED (`docs/pending.md` D9)

- **Status:** the Text2Cypher-style NL→structured-graph-query (B3.7) is
  **PARKED**. It is noted as a parked query surface (§4.5.5). Gated on a
  graph-query substrate over the Provident graph and multiple LLM calls per
  query (D2 cost/latency).
- **Cost of resolving:** a graph-query substrate + multiple LLM calls per query.
- **Benefit:** natural-language→structured-graph-query over the knowledge graph.

### 7.7 Supersession — Incanter disposition (decision GNOSIS-ENGINE)

- **Status:** Gnosis supersedes Incanter. Incanter is **ARCHIVED as a prototype
  lexical system**; its spec is archived, not deleted. The production-engine
  direction (non-JS, per INCANTER-DISPOSITION) is unchanged.
- **Remaining work:** confirm no suite reference still points at Incanter as the
  production engine; repoint the Astrographer contract's `engine: 'incanter'`
  framing to `engine: 'gnosis'` and the `source: 'incanter'` union member to
  `'local'` (the engine is now the local engine).
- **Benefit:** removes the prototype from the production path; makes Gnosis the
  single engine contract.

### 7.8 F6 — RAG evaluation harness (SHOULD HAVE, dev/QA gate)

- **Gap:** the spec has the query audit log (§4.3.4) and the fail-state catalogue
  (§6), but no **offline RAG-evaluation harness** — no faithfulness /
  answer-relevancy / contextual precision-recall gate (DeepEval-style, local-first)
  that objectively compares retrieval changes
  (`docs/research/rag-pipeline-evaluation-research-notes.md` §7 item 6). The
  research recommends it as SHOULD HAVE — "the objective feedback loop the other
  stages need."
- **Disposition:** this is a **dev/QA gate**, not a runtime behavior — it is
  recorded here as a gap rather than pinned as a runtime contract element. The
  query audit log (§4.3.4) supplies the raw query/result material the harness
  evaluates. **SHOULD HAVE** — land it as a dev/QA harness over `ragQuery`/
  `ragStream` outputs when the retrieval foundation ships.
- **Cost of resolving:** a local-first evaluation harness (DeepEval, Apache-2.0)
  over the audit-log material; offline, no runtime cost.
- **Benefit:** the objective feedback loop the retrieval stack needs to gate
  pipeline changes on faithfulness/relevancy/contextual precision-recall.

---

## 8. Cross-references

- `docs/architecture-overview.md` — §4.2 (component registry: Gnosis PLANNED
  production engine), §5.2 (Layer 2), §6 (cross-cutting principles), §8 (open
  questions).
- `docs/integration-matrix.md` — §3.2 (directed edges), §3.3 (required
  coverage), §5 (MCP-GUI parity + carve-out), §6 (flows).
- `docs/decisions.md` — D1–D4, GNOSIS-ENGINE, INCANTER-DISPOSITION,
  MCP-SURFACE-DEFERRED, D4-CARVEOUT-SCOPE, GRAPH-PUSH-FORMAT, D2-CLARIFICATION,
  SOLOMON-TRUST-PHASING, FIRMAMENT-AUTH, FIRMAMENT-DATAFLOW,
  FAMILIAR-ASSISTANT-CORE-SCOPE.
- `docs/pending.md` — #2 (MCP surface, deferred), D3 (adaptive RAG routing,
  parked), D4 (graph-aware reranking, parked), D5 (Incanter three-way lexical
  fusion — moot, Incanter archived), D8 (community summaries, parked), D9
  (LLM-generated dynamic graph query, parked).
- `docs/research/astrographer-engine-shell-boundary.md` — the engine work-package
  Gnosis owns (§3), the shell work-package (§4), the boundary notes (§5).
- `docs/research/astrographer-use-cases-to-pin.md` — Part B (research-derived
  features), Part C (parked/speculative items).
- `docs/specs/astrographer.md` — the shell contract Gnosis's engine-side contract
  honors (§4.2 consistency model, §4.3 RAG surface, §4.4 graph push, §4.5 MCP,
  §6 fail-states).
- `docs/specs/zodiac.md` — the remote RAG companion (§4.3 query modes, §4.2 RAG
  store states, §4.3.2 `sourceCrawlId` provenance).
- `docs/specs/incanter.md` — the archived prototype Gnosis supersedes (archived,
  not deleted).
- `docs/research/dag-rag-research-notes.md` — topological `reference`→`fact`
  resolution (§4 MUST HAVE, §5 pitfalls).
- `docs/research/hybrid-search-research-notes.md` — BM25 leg + RRF fusion (§9).
- `docs/research/reranking-research-notes.md` — cross-encoder reranking (§8).
- `docs/research/multi-query-retrieval-research-notes.md` — multi-query merge (§8).
- `docs/research/contextual-compression-research-notes.md` — filter/extract modes (§8).
- `docs/research/hyde-research-notes.md` — HyDE opt-in query mode (§6.4).
- `docs/research/adaptive-rag-research-notes.md` — adaptive routing (parked, D3).
- `docs/research/rag-pipeline-evaluation-research-notes.md` — DeepEval harness,
  graph-structure chunking (§7).
- `docs/research/graph-rag-when-vectors-arent-enough-research-notes.md` —
  provenance + multi-hop + community summaries (§4, §5 GAP-7/GAP-8/GAP-9).
- `docs/research/agent-memory-research-notes.md` — facts table + profile summary
  (§3.4, §5).
