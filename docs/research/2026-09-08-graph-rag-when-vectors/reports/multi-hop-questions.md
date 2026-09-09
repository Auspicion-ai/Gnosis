# Relationship Data: Filtering, Multi-Step Query Data, Multi-Hop Questions

- **Topic:** Relationship data in Graph RAG — filtering by relationship/edge type, multi-step query data, and multi-hop question answering over a knowledge graph
- **Date:** 2026-09-08
- **Tier:** MUST
- **Report slug:** `multi-hop-questions`
- **Primary consumers:** Astrographer, Zodiac (secondary: Incanter, Familiar, Solomon, Mystery, Astral)
- **Status:** Web-grounded research report (not a behavior contract; no spec/code/test changes)

---

## §1 What the technique is (web-grounded, cited)

The source talk (Nyah Macklin) frames the core Graph RAG value proposition as
**relationship data**: vectors do not capture relationships, so a pure vector
RAG fails on three related problems that a knowledge graph solves:

1. **Filtering** — retrieving by *relationship/edge type* rather than by
   semantic similarity. The talk's example: a person with comment access to
   documentation appears frequently in docs and so a vector search may rank
   them as if they were part of the doc team, regardless of whether they
   really are. A graph lets you filter on the actual relationship ("has
   `comment_access` edge to doc X"), not on co-occurrence frequency.
2. **Multi-step query data** — data that requires traversing more than one
   edge to answer (e.g. "which packages depend on X transitively?").
3. **Multi-hop questions** — questions that chain multiple facts/relationships
   (e.g. "which documents reference fact X and are published?").

The web literature confirms and formalizes this. The survey
*Graph-based Approaches and Functionalities in Retrieval-Augmented Generation*
([arXiv 2504.10499](https://arxiv.org/pdf/2504.10499)) and *Retrieval-Augmented
Generation with Graphs (GraphRAG)* ([arXiv 2501.00309](http://arxiv.org/pdf/2501.00309))
both position the graph as the mechanism that supplies **relational context**
that flat vector retrieval lacks. The *Retrieval–Reasoning Processes for
Multi-hop Question Answering* four-axis framework
([arXiv 2601.00536](https://arxiv.org/html/2601.00536)) and the *Advancements
in Complex Knowledge Graph Question Answering* survey
([MDPI Electronics 12(21):4395](https://www.mdpi.com/2079-9292/12/21/4395))
treat multi-hop QA as a distinct, harder retrieval problem that requires
**chaining** evidence across relationships rather than a single similarity
lookup.

Concretely, the technique decomposes into three retrievable capabilities:

- **Relationship filtering.** Restrict retrieval to nodes reachable via a
  specific edge/relationship type (or a specific target). *Empowering GraphRAG
  with Knowledge Filtering and Integration* ([arXiv 2503.13804](https://arxiv.org/html/2503.13804v1))
  and the SurrealDB *Knowledge Graph RAG: two query patterns for smarter AI
  agents* ([SurrealDB blog](https://surrealdb.com/blog/knowledge-graph-rag-two-query-patterns-for-smarter-ai-agents))
  describe filtering the graph by relationship before/while retrieving.
- **Multi-step / multi-hop traversal.** Walk the graph across multiple edges to
  assemble the evidence chain. *StepChain GraphRAG: Reasoning Over Knowledge
  Graphs for Multi-Hop Question Answering* ([arXiv 2510.02827](https://arxiv.org/pdf/2510.02827)),
  *S-Path-RAG: Semantic-Aware Shortest-Path Retrieval*
  ([arXiv 2603.23512](https://arxiv.org/html/2603.23512)), and *Query-Aware
  Spreading Activation for Multi-Hop Retrieval over Knowledge Graphs*
  ([arXiv 2606.30133](https://arxiv.org/html/2606.30133v1)) are representative
  multi-hop retrieval methods. The Neural Base's *Multi-hop reasoning with
  graphs* ([course](https://theneuralbase.com/graphrag/learn/advanced/multi-hop-reasoning-with-graphs/))
  and Enison's *Implementation Guide for Answering Complex Queries with
  Knowledge Graph × RAG* ([Enison](https://enison.ai/en/blog/knowledge-graph-rag-implementation))
  give practical pipelines.
- **Graph-native retrieval primitives.** The GraphRAG reference library exposes
  a **Parent-Child Retriever** ([graphrag.com](https://graphrag.com/reference/graphrag/parent-child-retriever/))
  and a **Global Community Summary Retriever**
  ([graphrag.com](https://graphrag.com/reference/graphrag/global-community-summary-retriever/)),
  and the original *From Local to Global* GraphRAG paper
  ([arXiv 2404.16130](https://arxiv.org/pdf/2404.16130)) establishes
  community-summary retrieval for broad questions. *GraphSearch: An Agentic
  Deep Searching Workflow for Graph RAG* ([arXiv 2509.22009](https://arxiv.org/html/2509.22009))
  and *TigerVector: Supporting Vector Search in Graph Databases*
  ([arXiv 2501.11216](https://arxiv.org/html/2501.11216)) show how to combine
  graph traversal with vector search (graph-enhanced vector search / dynamic
  graph query generation).

**Why it matters for the suite.** The Auspicion Suite's RAG surface is already
graph-aware by design: Astrographer stores documents as Provident graphs with a
`reference` → `fact` relationship structure, and Zodiac pre-graphs crawled data
into nodes + edges. The relationship-data capabilities above are therefore not
an exotic add-on — they are the natural, native expression of the suite's
existing graph substrate. The prior research notes already point here:
`dag-rag-research-notes.md` §4 recommends **topological resolution of
`reference` → `fact` dependencies** as a MUST HAVE, and
`multi-query-retrieval-research-notes.md` §6 shows how graph-region-scoped
query variants exploit the graph's relationship structure.

---

## §2 How it applies to Astrographer

Astrographer is the suite's **Graphical RAG engine + document store/wiki**
(`docs/specs/astrographer.md`). Its documents are Provident graphs, and its
defining feature is the cross-link/data-embed consistency model — which is
precisely a **relationship-data** structure.

**Relationship data already exists in the contract.**
- **§4.2.1 Node kinds** — a Provident graph node is `content`, `fact`, or
  `reference`. The `reference` node points at a `fact` (or another node) in the
  same or another document. This is the suite's native relationship data: a
  `reference` → `fact` edge is a typed relationship.
- **§4.2.2 Reference modes** — `link` (navigational, live-resolved) and `embed`
  (snapshot + pointer). Both are relationship-typed edges.
- **§4.2.3 Consistency enforcement** — the invariant that a `fact`'s canonical
  `value` is the single source of truth, with `embed`/`link` kept consistent.
  The reference graph is a **directed dependency structure**: resolving a
  `reference` requires resolving its target `fact` first. This is exactly the
  dependency-ordered (topological) structure that multi-hop resolution needs
  (see `dag-rag-research-notes.md` §3).
- **§4.1.2 Cross-wiki references** — a reference may target a node in another
  Wiki, flagged `crossWiki: true`. Relationship data spans wikis.

**Filtering — the `filters?` parameter is the natural home.**
- **§4.3.2 `ragQuery`** signature is `(query, {wikiId?, topK?, filters?}) →
  RagResult`. The `filters?` parameter exists in the contract but is **not
  defined**. This is the concrete, low-risk place to add **relationship-aware
  filtering**: filter by node kind (`content`/`fact`/`reference`), by
  relationship/edge type (`link`/`embed`), by reference target, and by wiki.
  This directly implements the source talk's "filtering" capability and the
  "person with comment access" example (filter on the actual relationship, not
  on co-occurrence frequency).
- **§4.5.1 MCP tools** `rag_query`/`rag_stream` mirror `ragQuery`/`ragStream`
  and must expose the same `filters` surface (D4 parity).

**Multi-hop — the reference graph is a multi-hop substrate.**
- A multi-hop question over Astrographer's wiki ("which published documents
  reference fact X?", "which facts does document Y transitively depend on?")
  is answered by **traversing the reference graph** across multiple
  `reference` → `fact` edges. This is the MUST-HAVE topological resolution
  already recommended in `dag-rag-research-notes.md` §4.
- **§4.2.3** gives the enforcement point: `publishDocument`/`pushToAstral`
  fail with `UnresolvedReference` on `BROKEN`/`STALE`. A multi-hop resolver
  must respect the same consistency invariant — it should resolve through
  `FRESH`/`RESOLVED` references and surface `BROKEN`/`STALE` ones rather than
  silently traversing stale embeds.

**Status in the spec.** Relationship filtering and multi-hop traversal are
**NOT yet in the spec** — they are **proposed spec changes** to §4.3.2 (define
the `filters` shape) and to the RAG surface (add a multi-hop/traversal mode).
The underlying relationship data (reference graph, §4.2) is already pinned.
The `filters?` parameter is already present but undefined, so defining it is a
low-risk, additive change.

---

## §3 How it applies to Zodiac

Zodiac is the suite's **backend/remote RAG companion** that pre-graphs and
vector-embeds web-crawled data (dependency packages, current web data)
(`docs/specs/zodiac.md`). Its entire value is supplying **non-local
relationship data** that a purely local Astrographer cannot obtain.

**Relationship data already exists in the contract.**
- **§4.2 RAG engine (pre-graphing + vector-embedding)** — raw material is
  parsed into a **graph (nodes and edges)** representing the crawled
  information, e.g. a dependency package's metadata, its dependencies, and
  their relationships. This is the suite's richest source of *external*
  relationship data.
- **§4.3.1 Query shape** — `mode` is `graph` | `vector` | `hybrid`. The
  `graph` mode is the relationship-data surface.
- **§4.3.2 Query result** — **Graph mode** returns the pre-graphed subgraph for
  the target (nodes and edges); **hybrid mode** merges graph and vector matches.

**Multi-hop — transitive dependency traversal.**
- The canonical Zodiac use case is dependency packages. A multi-hop question
  ("which packages depend on X transitively?", "what is the full dependency
  closure of package Y?") is answered by **traversing the pre-graphed
  dependency edges** across multiple hops. This is the natural multi-step
  query-data capability for Zodiac's graph mode.
- **§4.3.2** currently returns a subgraph for a single target; a multi-hop
  mode would extend this to return the transitive closure / reachable subgraph
  along specified relationship types.

**Filtering — relationship-typed traversal.**
- A query should be able to filter by relationship type (e.g. "only direct
  `depends_on` edges, not `dev_depends_on`", "only `dependency` source type,
  not `web`"). This is a proposed extension to the **§4.3.1 query shape** (add
  a relationship/edge-type filter) and to the **§4.5 `zodiac_query`** MCP tool.

**Status in the spec.** Zodiac's `graph` mode already returns relationship
data (§4.3.2), but **multi-hop traversal and relationship filtering are NOT yet
in the spec** — they are **proposed spec changes** to §4.3.1/§4.3.2 and the
`zodiac_query` MCP tool. The pre-graphed relationship substrate (§4.2) is
already pinned.

---

## §4 How it applies to the other suite consumers

**Incanter** (`docs/specs/incanter.md`). Incanter is the prototype Rust
Graph-RAG engine Astrographer consumes over HTTP/REST + SSE (edge #14). Its
graph (§4.4) is a **token-transition directed multigraph** (co-occurrence
weights), not a semantic entity/relationship graph. Multi-hop over Incanter's
graph is therefore **limited**: the graph captures token adjacency, not
`reference` → `fact` or dependency relationships. The relationship-data
capabilities belong at the **Astrographer/Zodiac layer** (which own the
semantic relationship graphs), not inside Incanter's token graph. Incanter's
`POST /v1/query` (§4.8) already accepts `filter_doc_ids`; relationship filtering
is a handoff candidate to Incanter only if a lexical/relationship leg is added
(see `hybrid-search-research-notes.md` §9, GAP-2). **Disposition: relationship
data is out of Incanter's token-graph scope; the semantic relationship layer
lives in Astrographer/Zodiac.**

**Familiar** (`docs/specs/familiar.md`). Familiar is the agent harness that
orchestrates the suite. Multi-hop questions surface through its orchestration:
- **§4.3.1 F1 (knowledge)** — Familiar calls Astrographer's `rag_query`
  (§4.3.2). A multi-hop question is routed to Astrographer's RAG surface, which
  (with the §2 changes) can answer it via reference-graph traversal.
- **§4.3.3 F4 (research)** — research routes through Astrographer's RAG
  surface, which may return Zodiac results (`source: 'zodiac'`). Multi-hop
  over Zodiac's dependency graph (§3) is reachable through this path.
- **§4.1.3 memory store** — the facts table is a flat set of facts, not a
  relationship graph; multi-hop over assistant memory is out of scope for the
  facts-table core (the parked vector-search layer, pending D6, is the future
  home). **Disposition: Familiar is a consumer of relationship-aware RAG, not
  a producer; it inherits the §2/§3 capabilities through its delegation edges.**

**Solomon** (`docs/specs/solomon.md`). Solomon is the cross-instance search
layer. Its search (§4.2) fans a query out to peers and aggregates results
(§4.2.2). **Cross-instance multi-hop** — chaining relationship data across
instances (e.g. a fact in one Astrographer instance that references a fact in
another) — is a harder, **NICE-TO-HAVE** capability. The Phase-1 aggregation
contract (§4.2.2) groups by source peer and leaves cross-peer ordering
unpinned (§7.4); relationship-aware cross-instance traversal would build on
that. **Disposition: NICE-TO-HAVE / PARKED — cross-instance multi-hop is a
Phase-2 concern; the local multi-hop foundation (§2/§3) lands first.**

**Astral** (`docs/specs/astral.md`). Astral hosts the published wiki (Provident
graphs pushed via `provident-graph/1`, §4.2). It is a **hosting/serving** layer,
not a RAG engine; multi-hop questions over the published wiki are answered by
the RAG surface (Astrographer/Zodiac), not by Astral itself. Astral's role is
to serve the relationship-rich Provident graphs (§4.1) so the RAG surface can
traverse them. **Disposition: Astral is a passive relationship-data carrier;
no change needed beyond serving the graph faithfully.**

**Mystery** (`docs/specs/mystery.md` — **absent**). Mystery is a PLANNED
message board that crosslinks information references from other apps by post
topic (registry §4.3; pending #12). Its core design is **relationship data**:
a post topic → information reference (Astrographer documentation, Astral
pages) mapping. Multi-hop ("link the relevant web-app documentation during a
debug thread") is the natural expression of Mystery's topic→reference
crosslink. Because the spec does not exist, this is **PARKED** — the
relationship-data design should be captured in the Mystery contract when it is
written (pending #12). **Disposition: PARKED until `docs/specs/mystery.md`
exists; the topic→reference relationship model is the design target.**

---

## §5 Recommendation for the suite

Grounded in D1–D4 (open-source AGPL-3.0, local-first, interconnection,
MCP-GUI parity) and the prior research (`dag-rag-research-notes.md` §4,
`multi-query-retrieval-research-notes.md` §6).

| # | Recommendation | Tier | Where | Rationale |
| --- | --- | --- | --- | --- |
| 1 | **Define relationship-aware filtering on Astrographer's `ragQuery` `filters` parameter** — filter by node kind (`content`/`fact`/`reference`), relationship/edge type (`link`/`embed`), reference target, and wiki. | **MUST** | `astrographer.md` §4.3.2, §4.5.1 | The `filters?` param already exists but is undefined. This is the direct implementation of the source talk's "filtering" capability and the "person with comment access" example. Low-risk, additive, D4-parity (expose through `rag_query`/`rag_stream`). |
| 2 | **Add topological (multi-hop) resolution of the `reference` → `fact` graph in Astrographer** — resolve references in dependency order, respecting the §4.2.3 consistency invariant. | **MUST** | `astrographer.md` §4.2.3, §4.3 | Already recommended as MUST HAVE in `dag-rag-research-notes.md` §4. The reference graph is a directed dependency structure; multi-hop questions over the wiki are answered by traversing it. Runs fully local (D2), open algorithms (D1). |
| 3 | **Add multi-hop (transitive) traversal to Zodiac's `graph` mode** — return the reachable subgraph along specified relationship types (e.g. transitive dependency closure). | **SHOULD** | `zodiac.md` §4.3.1, §4.3.2, §4.5 | Zodiac's pre-graphed dependency data is the suite's richest external relationship source. Multi-hop over it is the natural dependency-package use case. Remote by design (D2-CLARIFICATION) but additive. |
| 4 | **Add relationship-type filtering to Zodiac's query shape** — filter traversal by edge type (e.g. `depends_on` vs `dev_depends_on`, `dependency` vs `web`). | **SHOULD** | `zodiac.md` §4.3.1, §4.5 | Complements #3; gives Zodiac the same relationship-filtering capability as #1. |
| 5 | **Expose relationship filtering + multi-hop through both GUI and MCP (D4)** — `rag_query`/`rag_stream` and `zodiac_query` must carry the new params; the GUI RAG panel must surface them. | **MUST (parity)** | `astrographer.md` §4.5.1, `zodiac.md` §4.5 | D4 requires every non-security feature on both surfaces. Relationship filtering is not a security-configuration carve-out. |
| 6 | **Cross-instance multi-hop via Solomon** — chain relationship data across instances. | **NICE-TO-HAVE / PARKED** | `solomon.md` §4.2.2, §7.4 | Harder; builds on the local multi-hop foundation (#2/#3) and the unpinned cross-peer aggregation. Land after the local foundation ships. |
| 7 | **Capture Mystery's topic→reference relationship model in its contract.** | **PARKED** | `docs/specs/mystery.md` (absent) | Mystery's core design is relationship data (pending #12). Park until the spec exists. |

**Net tier: MUST.** Relationship data — filtering, multi-step query data, and
multi-hop questions — is the suite's native RAG value proposition, not an
optional enhancement. The relationship substrate (Astrographer's reference
graph, Zodiac's pre-graphed dependency data) is already pinned in the
contracts; the MUST items (#1, #2, #5) are low-risk, additive spec changes that
make that substrate queryable. The SHOULD items (#3, #4) extend the same
capability to Zodiac's external relationship data. The NICE-TO-HAVE/PARKED
items (#6, #7) are deferred until the local foundation lands.

**D1–D4 grounding.**
- **D1 (AGPL-3.0):** all techniques are open algorithms/papers (StepChain,
  S-Path-RAG, spreading activation, GraphRAG reference retrievers); no
  licensing blocker.
- **D2 (local-first):** multi-hop over Astrographer's local reference graph
  runs fully local; no cloud dependency. Zodiac is remote by design
  (D2-CLARIFICATION) but additive — its unavailability degrades to local-only
  results (`astrographer.md` §5.7).
- **D3 (interconnection):** relationship data is the interconnection enabler —
  Zodiac's external dependency graph + Astrographer's reference graph + (later)
  Solomon cross-instance traversal make multi-hop questions answerable across
  the suite.
- **D4 (MCP-GUI parity):** relationship filtering and multi-hop must be exposed
  through both GUI and MCP (`rag_query`/`rag_stream`, `zodiac_query`); neither
  is a security-configuration carve-out.

---

## §6 Source URL list

**Surveys / foundations**
- https://arxiv.org/pdf/2504.10499 — *Graph-based Approaches and Functionalities in Retrieval-Augmented Generation: A Comprehensive Survey*
- http://arxiv.org/pdf/2501.00309 — *Retrieval-Augmented Generation with Graphs (GraphRAG)*
- https://arxiv.org/html/2601.00536 — *Retrieval–Reasoning Processes for Multi-hop Question Answering: A Four-Axis Design Framework and Empirical Trends*
- https://www.mdpi.com/2079-9292/12/21/4395 — *Advancements in Complex Knowledge Graph Question Answering: A Survey*
- https://arxiv.org/pdf/2506.00054v1 — *Retrieval-Augmented Generation: A Comprehensive Survey of Architectures, Enhancements, and Robustness Frontiers*

**Multi-hop retrieval methods**
- https://arxiv.org/pdf/2510.02827 — *StepChain GraphRAG: Reasoning Over Knowledge Graphs for Multi-Hop Question Answering*
- https://arxiv.org/html/2603.23512 — *S-Path-RAG: Semantic-Aware Shortest-Path Retrieval Augmented Generation for Multi-Hop Knowledge Graph Question Answering*
- https://arxiv.org/html/2606.30133v1 — *Query-Aware Spreading Activation for Multi-Hop Retrieval over Knowledge Graphs*
- https://theneuralbase.com/graphrag/learn/advanced/multi-hop-reasoning-with-graphs/ — *Multi-hop reasoning with graphs* (GraphRAG Advanced Course)
- https://enison.ai/en/blog/knowledge-graph-rag-implementation — *Implementation Guide for Answering Complex Queries with Knowledge Graph × RAG*

**Relationship filtering / graph-aware retrieval**
- https://arxiv.org/html/2503.13804v1 — *Empowering GraphRAG with Knowledge Filtering and Integration*
- https://surrealdb.com/blog/knowledge-graph-rag-two-query-patterns-for-smarter-ai-agents — *Knowledge Graph RAG: two query patterns for smarter AI agents* (SurrealDB)
- https://aclanthology.org/2025.findings-emnlp.329.pdf — *KERAG: Knowledge-Enhanced Retrieval-Augmented Generation for Advanced Question Answering*

**Graph-native retrieval primitives / graph+vector**
- https://graphrag.com/reference/graphrag/parent-child-retriever/ — *Parent-Child Retriever* (GraphRAG reference)
- https://graphrag.com/reference/graphrag/global-community-summary-retriever/ — *Global Community Summary Retriever* (GraphRAG reference)
- https://arxiv.org/pdf/2404.16130 — *From Local to Global: A GraphRAG Approach to Query-Focused Summarization*
- https://arxiv.org/html/2509.22009 — *GraphSearch: An Agentic Deep Searching Workflow for Graph Retrieval-Augmented Generation*
- https://arxiv.org/html/2501.11216 — *TigerVector: Supporting Vector Search in Graph Databases for Advanced RAGs*
