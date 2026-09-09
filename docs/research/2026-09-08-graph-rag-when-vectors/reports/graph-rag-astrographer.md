# Graph RAG — Application to Astrographer (graphical RAG engine + document store/wiki)

- **Topic:** Graph RAG — RAG whose retrieval path also includes a knowledge graph (nodes, edges, properties), used alongside the vector layer.
- **Date:** 2026-09-08
- **Tier:** MUST
- **Report slug:** `graph-rag-astrographer`
- **Primary focus:** Astrographer, Zodiac. **Secondary:** Incanter, Familiar, Solomon, Mystery, Astral.
- **Status:** Web-grounded research report (documentation deliverable only — no spec/code/test changes).

---

## §1 What the technique is (web-grounded, cited)

**Graph RAG** is a family of retrieval-augmented-generation designs in which the
retrieval path includes a **knowledge graph** (nodes, edges, properties) in
addition to (or instead of) a flat vector index. The source talk frames it as:
*"RAG where retrieval path also includes a knowledge graph — use both vector and
graph layer methods."* The motivating failures of pure vector RAG are:

1. **Vectors don't capture relationship data.** A vector index stores
   embeddings, not the relationships between entities. A knowledge graph supplies
   the relational context that enables **follow-up questions** and carries
   **domain knowledge** that similarity search cannot express.
2. **Vector similarity ≠ relevance.** High cosine similarity to a query is not
   the same as being the relevant fact; graph structure (paths, hops, shared
   entities) is a complementary relevance signal.
3. **Lack of explainability.** Governance questions are more easily answered by
   **sourcing and tracing** — walking the graph back to the originating fact —
   than by a black-box similarity score.
4. **Maturity.** Vector stores are younger than established database systems;
   graph databases bring mature query, constraint, and transaction semantics.

The **knowledge graph** model is: **nodes** (noun-focused entities), **edges**
(relationship-focused links), and **properties** (attributes attachable to both
nodes and edges). Graph RAG then layers retrieval techniques on top:

- **Graph-enhanced vector search** — retrieve by vector similarity, then expand
  or re-rank using the graph neighborhood (e.g. hybrid vector + graph search).
- **Dynamic graph query generation** — generate a graph query (Cypher-style or a
  sub-task DAG) from the natural-language question at query time, rather than
  relying on a pre-built static graph.
- **Parent-child retrievers** — retrieve at a coarse granularity (a document or
  section) and return the finer-grained child chunks, preserving context.
- **Community summaries** — precompute hierarchical summaries over graph
  communities (Microsoft GraphRAG's *global search*) for broad, theme-level
  questions.
- **Graph enrichments** — augment the graph with additional nodes/edges/properties
  (e.g. from external or crawled data) to improve retrieval.

**Key cited sources:**
- Microsoft's *From Local to Global: A GraphRAG Approach to Query-Focused
  Summarization* ([arXiv:2404.16130](https://arxiv.org/pdf/2404.16130)) — the
  canonical GraphRAG paper; community summaries + local/global query.
- *RAG vs. GraphRAG: A Systematic Evaluation and Key Insights*
  ([arXiv:2502.11371](https://arxiv.org/abs/2502.11371v3)) — a systematic
  comparison showing GraphRAG's strengths (multi-hop, relationship-heavy,
  global questions) and where plain RAG wins (cost, simple fact lookup).
- Neo4j's *GraphRAG Field Guide* ([neo4j.com](https://neo4j.com/blog/developer/graphrag-field-guide-rag-patterns/)) — a taxonomy of RAG patterns including graph-enhanced vector search, parent-child retrievers, and community summaries.
- *Graph-Enhanced RAG: Powerful Architectural Patterns Beyond Vector Search in
  Production* ([progressiverobot.com](https://www.progressiverobot.com/2026/05/17/graph-enhanced-rag/)) — production patterns for combining vector + graph layers.
- *Knowledge Graph RAG Hybrid: When It Helps and How to Build It*
  ([osfoundry.io](https://osfoundry.io/articles/knowledge-graph-rag-hybrid-when-and-how)) — when the graph layer adds value vs. when it is overhead.
- *Graph RAG Without a Graph Database* ([Milvus blog](https://milvus.io/blog/vector-graph-rag-without-graph-database.md)) — graph RAG over a vector store without a dedicated graph DB (relevant to D2 local-first).
- *Hybrid retrieval in RAG: vector + graph search* ([learnwithparam.com](https://www.learnwithparam.com/blog/hybrid-retrieval-rag-vector-graph-search)) — concrete hybrid vector+graph retrieval.
- *GraphRAG: Improving global search via dynamic community selection* ([Microsoft Research](https://www.microsoft.com/en-us/research/blog/graphrag-improving-global-search-via-dynamic-community-selection/)) — dynamic (query-time) graph selection.
- *StepChain GraphRAG: Reasoning Over Knowledge Graphs for Multi-Hop QA* ([arXiv:2510.02827](https://arxiv.org/pdf/2510.02827)) — multi-hop reasoning over a knowledge graph.
- *Knowledge Graph-extended RAG for Question Answering* ([arXiv:2504.08893](https://arxiv.org/html/2504.08893v1)) — extending RAG with a knowledge graph.
- *GraphER: An Efficient Graph-Based Enrichment and Reranking Method for RAG* ([arXiv:2603.24925](https://arxiv.org/html/2603.24925v3)) — graph enrichment + reranking.
- *PAGE-RAG: Provenance-Aware Graph Evidence Promotion for Fixed-Budget Multi-hop RAG* ([arXiv:2608.29753](https://arxiv.org/abs/2608.29753)) — provenance/traceability in multi-hop graph RAG.
- *HyCE-RAG: Hypergraph Chain-of-Evidence RAG for Explainable Multi-hop QA* ([arXiv:2607.22597](https://arxiv.org/abs/2607.22597)) — explainable multi-hop retrieval.
- TrustGraph *Explainability* ([docs.trustgraph.ai](https://docs.trustgraph.ai/overview/explainability.html)) and *query-time explainability* ([github.com/trustgraph-ai/trustgraph](https://github.com/trustgraph-ai/trustgraph/blob/4e3bd85a/docs/tech-specs/query-time-explainability.md)) — traceable, explainable graph RAG.
- *Evaluating GraphRAG for Traceable and Interpretable Question Answering* ([DOI:10.1109/acdsa67686.2026.11468256](https://doi.org/10.1109/acdsa67686.2026.11468256)) — traceability/interpretability evaluation.
- *Graph data models for RAG applications* ([neo4j.com](https://neo4j.com/blog/developer/graph-data-models-rag-applications/)) — nodes/edges/properties modeling for RAG.
- *Knowledge Graphs Meet RAG: A Practical Integration Guide* ([bigdataboutique.com](https://bigdataboutique.com/blog/knowledge-graphs-meet-rag-practical-integration-guide)) — practical KG+RAG integration.
- *Stop graphing everything: When GraphRAG actually beats vector RAG* ([VentureBeat](https://venturebeat.com/orchestration/stop-graphing-everything-when-graphrag-actually-beats-vector-rag)) — when NOT to use a graph layer (cost/overhead).
- *GraphRAG vs Vector RAG: Choosing Your Retrieval Strategy* ([airbyte.com](https://airbyte.com/agentic-data/graph-rag-vs-vector-rag)) and *GraphRAG vs Vector RAG: Which Wins for Enterprise AI?* ([tigergraph.com](https://www.tigergraph.com/blog/graphrag-vs-vector-rag/)) — decision guidance.

---

## §2 How it applies to Astrographer

Astrographer is already a **graphical RAG engine + document store/wiki** over
**Provident graphs** (`docs/specs/astrographer.md` §2, §4.1.1). Its data model
is, in effect, a knowledge graph with properties:

- **Nodes** (`astrographer.md` §4.2.1): `content`, `fact`, and `reference` node
  kinds. The `fact` node is the single source of truth (noun-focused, like a KG
  node).
- **Edges** (`astrographer.md` §4.2.2): `link` and `embed` reference modes —
  relationship-focused edges from a referencing document to a target `fact`/node.
- **Properties** (`astrographer.md` §4.1.1, §4.2.1): document metadata
  (`title`, `tags`, `author`, `revision`, `state`) and fact `factKey`/`value` —
  properties attached to nodes; the `mode` (`link`/`embed`) and `crossWiki`
  flag are properties on edges.

So the **core Graph RAG substrate is already in the spec** — the technique is not
a foreign addition but a set of retrieval behaviors to run over the existing
graph. Mapping the source-talk techniques to concrete contract elements:

| Graph RAG technique | Astrographer contract element | Status |
| --- | --- | --- |
| **Graph-enhanced vector search** | `ragQuery`/`ragStream` (`astrographer.md` §4.3.2) consume Incanter's hybrid graph-tension + vector query (`incanter.md` §4.8) and Zodiac's `graph`/`vector`/`hybrid` modes (`zodiac.md` §4.3.2). The `source` union `local\|incanter\|zodiac` (§4.3.2) is the vector+graph fusion seam. | **Already in spec** (the hybrid axis); extend with graph-neighborhood expansion/re-rank. |
| **Dynamic graph query generation** | Multi-hop questions over the `reference`→`fact` graph (§4.2). The DAG-RAG research already recommends **topological resolution of `reference`→`fact` dependencies** as a MUST HAVE (`docs/research/dag-rag-research-notes.md` §4). | **Proposed spec change** (aligns with the parked DAG-RAG SHOULD-HAVE). |
| **Parent-child retrievers** | A `Document` is a graph (§4.1.1); nodes are the children. Retrieval could return a node and its containing document/section for context. | **Proposed spec change** (NICE-TO-HAVE). |
| **Community summaries** | The wiki's reference graph (§4.2) could be summarized per community for global/theme questions. | **PARKED** (cost; D2; overlaps with the parked DAG-RAG global-graph DISCARD). |
| **Graph enrichments** | Zodiac pre-graphs + vector-embeds crawled data (`zodiac.md` §4.2) and feeds it as a second RAG source (`astrographer.md` §5.7). | **Already in spec** (edge #1). |

**Explainability / traceability (governance).** This is the strongest Graph RAG
fit for Astrographer. The consistency model (`astrographer.md` §4.2.3) already
tracks every `link`/`embed` to its target and exposes
`getConsistencyReport(wikiId)` (§4.1.3, §4.5.1 `get_consistency_report`). A RAG
result could carry the same provenance — walking the graph back to the source
`fact`/`documentId`/`nodeId` — which directly answers the source talk's
"governance questions are more easily answered by sourcing and tracing." The
`ragQuery` result shape (`astrographer.md` §4.3.2) already returns
`{documentId, nodeId, score, snippet, source}`; adding a **trace path** (the
edges walked to reach the answer) is a small, high-value extension.

**Relationship data / multi-hop.** The `reference`→`fact` graph (§4.2) is
exactly the "relationship data" the source talk says vectors miss. Filtering by
relationship (e.g. "all documents that embed fact X") and multi-step/multi-hop
queries are natural graph operations over this structure. The DAG-RAG research
(`docs/research/dag-rag-research-notes.md` §3) already maps this to Astrographer:
resolving a `reference` requires resolving its target `fact` first — a
dependency-ordered topological walk.

**Maturity.** The source talk notes vector stores are less mature than
established DB systems. Astrographer's graph is a Provident graph (Layer 1,
`docs/architecture-overview.md` §5.1) — a first-class, versioned graph model with
optimistic concurrency (`astrographer.md` §4.1.4) and a canonical push format
(`astrographer.md` §4.4, decision GRAPH-PUSH-FORMAT). This is a mature substrate
relative to a raw vector store, and it is the reason the graph layer is the
natural primary retrieval structure here.

**Net for Astrographer:** the graph substrate is already in the spec; the
concrete MUST/SHOULD work is (a) **graph-enhanced retrieval over the reference
graph** (topological resolution, per DAG-RAG) and (b) an **explainability/trace
surface** on RAG results. Community summaries and dynamic graph query generation
are PARKED (cost, D2, overlap with parked DAG-RAG items).

---

## §3 How it applies to Zodiac

Zodiac is a **graphical RAG engine integrated with web crawlers** that replies to
Astrographer with **pre-graphed and vector-embedded data** (`docs/specs/zodiac.md`
§2, §4.2). It is the suite's clearest Graph RAG engine:

- **Pre-graphing** (`zodiac.md` §4.2): crawled raw material is parsed into a
  graph (nodes + edges) — e.g. a dependency package's metadata, its dependencies,
  and their relationships. This is a knowledge graph built from crawled data.
- **Vector-embedding** (`zodiac.md` §4.2): the raw material and/or graph nodes
  are embedded into a vector store, each item carrying an `embedding id` and a
  `sourceCrawlId`.
- **Query modes** (`zodiac.md` §4.3.2): `graph` (returns the pre-graphed
  subgraph), `vector` (top-`limit` by similarity), and `hybrid` (merged graph +
  vector). This is **graph-enhanced vector search** in its purest form — the
  source talk's "use both vector and graph layer methods."
- **RAG store states** (`zodiac.md` §4.2): `EMPTY`/`READY`/`STALE`/`REFRESHING`
  — a mature store-state model (the "maturity" point).

**Mapping the techniques:**
- **Graph-enhanced vector search** — already the core of Zodiac's `hybrid` mode
  (`zodiac.md` §4.3.2). **Already in spec.**
- **Graph enrichments** — Zodiac's crawlers add non-local data (dependency
  packages, current web data) to the graph (`zodiac.md` §2, §4.1). This is graph
  enrichment by definition. **Already in spec.**
- **Dynamic graph query generation** — a query in `graph`/`hybrid` mode
  (`zodiac.md` §4.3.2) could generate a subgraph query from the natural-language
  `query` at query time. **Proposed spec change** (SHOULD/NICE-TO-HAVE).
- **Community summaries** — Zodiac could precompute summaries over the
  pre-graphed communities for global questions. **PARKED** (cost; D2-CLARIFICATION
  allows remote-only features but the cost/benefit is not yet justified).
- **Explainability** — Zodiac results carry `sourceCrawlId` and `stale`
  (`zodiac.md` §4.3.2), giving provenance back to the crawl. A trace path from a
  result to its source crawl is a natural extension. **Proposed spec change**
  (SHOULD).

**Net for Zodiac:** the graph+vector hybrid is already the engine's design
(`zodiac.md` §4.2, §4.3.2). The concrete additions are explainability/trace
(provenance to `sourceCrawlId`) and, optionally, dynamic graph query generation.
Community summaries stay PARKED.

---

## §4 How it applies to the other suite consumers

- **Incanter** (`docs/specs/incanter.md`): Incanter is the prototype Graph-RAG
  engine Astrographer consumes over HTTP (`incanter.md` §2, §5). Its hybrid query
  (`incanter.md` §4.8) already fuses **graph-tension + vector**
  (`vector_weight`/`graph_weight`). Graph RAG's graph-enhanced vector search is
  therefore already the engine's design. The missing leg is lexical (BM25) —
  recorded as GAP-2 / parked D5 (`docs/defects.md`, `docs/pending.md` D5). The
  graph-tension model (`incanter.md` §4.6) is a graph-based retrieval signal, so
  the graph layer is native here. **Recommendation:** the graph-enhanced axis is
  already in spec; the concrete gap is the lexical leg (GAP-2), not a new Graph
  RAG technique.

- **Familiar** (`docs/specs/familiar.md`): Familiar's knowledge domain delegates
  to Astrographer's RAG surface (`familiar.md` §4.3.1, §4.3.3) and its research
  domain routes through Astrographer's `rag_query` (which may return Zodiac
  results, `familiar.md` §4.3.3). Familiar's own memory store is a **facts table**
  (`familiar.md` §4.1.3) — individually addressable fact records, not a graph.
  Graph RAG applies to Familiar **indirectly**, through the Astrographer/Zodiac
  surfaces it consumes. The parked agent-memory vector-search layer
  (`docs/pending.md` D6) is a vector, not a graph, addition. **Recommendation:**
  no direct Graph RAG work in Familiar; it inherits the graph layer through
  Astrographer/Zodiac.

- **Solomon** (`docs/specs/solomon.md`): Solomon is cross-instance search over
  `zodiac|astral|astrographer` instances (`solomon.md` §4.2, §4.5.1). Its result
  aggregation (`solomon.md` §4.2.2) is flat (grouped by peer, ordered by the
  peer's own relevance). Graph RAG's relationship-aware retrieval could improve
  cross-instance ranking, but Solomon's Phase-1 contract deliberately keeps
  cross-peer ordering unpinned (`solomon.md` §7.4). **Recommendation:**
  NICE-TO-HAVE / PARKED — defer graph-aware cross-instance ranking until Phase-1
  search semantics finalize.

- **Mystery** (`docs/specs/mystery.md` — **absent**; `docs/pending.md` #12):
  Mystery is a message board that crosslinks information references by post topic
  (`docs/architecture-overview.md` §4.3). Its topic→reference resolution
  (`docs/pending.md` #12) is a relationship-mapping problem that a knowledge
  graph (Astrographer's reference graph) could serve. **Recommendation:** when the
  Mystery contract is written, model its crosslinks as graph edges over
  Astrographer's reference graph. PARKED until the spec exists.

- **Astral** (`docs/specs/astral.md`): Astral is the wiki/publishing host
  (`astral.md` §2). It receives Provident graphs (`astral.md` §4.2) and serves
  them (`astral.md` §4.1). Graph RAG applies to Astral only as a **serving**
  concern: the hosted pages are graphs, and the docs-as-product reachability
  surface (GAP-4/GAP-5, `docs/defects.md`) is about agent-facing markdown, not
  graph retrieval. **Recommendation:** no direct Graph RAG work; Astral serves the
  graphs Astrographer produces.

---

## §5 Recommendation for the suite

Grounded in D1–D4:

- **MUST — Graph-enhanced retrieval over Astrographer's reference graph
  (topological resolution of `reference`→`fact`).** Astrographer's data model is
  already a knowledge graph (`astrographer.md` §4.2). Resolving `reference`→`fact`
  in dependency order is the core, low-risk win and directly enables
  relationship-aware, multi-hop answers. This is consistent with the DAG-RAG
  research's MUST-HAVE (`docs/research/dag-rag-research-notes.md` §4) and with
  D2 (runs locally over the Provident graph) and D4 (exposable as MCP + GUI).
  **Proposed spec change** to `astrographer.md` §4.3.

- **SHOULD — Explainability / trace surface on RAG results.** Extend the
  `ragQuery` result (`astrographer.md` §4.3.2) with a **trace path** (the edges
  walked to reach the answer), reusing the provenance already tracked by the
  consistency model (`astrographer.md` §4.2.3). This directly answers the source
  talk's governance/sourcing point and is cheap because the graph already
  records every reference. **Proposed spec change** to `astrographer.md` §4.3.

- **SHOULD — Zodiac explainability (provenance to `sourceCrawlId`).** Zodiac
  results already carry `sourceCrawlId` and `stale` (`zodiac.md` §4.3.2); expose a
  trace path from a result to its source crawl. **Proposed spec change** to
  `zodiac.md` §4.3.

- **NICE-TO-HAVE — Parent-child retrievers in Astrographer.** Return a node plus
  its containing document/section for context (`astrographer.md` §4.1.1). Low
  cost; improves answer context. **Proposed spec change.**

- **PARKED — Community summaries (Microsoft GraphRAG global search).** Precomputing
  hierarchical community summaries over the wiki reference graph is expensive and
  conflicts with D2 local-first cost/latency; the DAG-RAG research already
  DISCARDs pre-built global graph construction (`docs/research/dag-rag-research-notes.md`
  §4). Revisit only if a global/theme question use case surfaces.

- **PARKED — Dynamic graph query generation.** Generating a sub-task DAG or graph
  query at query time overlaps with the parked DAG-RAG SHOULD-HAVE
  (`docs/pending.md` D3/D4 context) and is the most complex phase (multiple LLM
  calls per query). Land it after the MUST/SHOULD retrieval foundation ships.

**Overall recommendation tier: MUST.** Graph RAG is not a foreign technique for
the suite — Astrographer and Zodiac are already graphical RAG engines over
knowledge-graph-shaped data. The concrete MUST/SHOULD work is graph-enhanced
retrieval over the reference graph plus an explainability/trace surface; the
expensive global techniques (community summaries, dynamic graph query
generation) stay PARKED for D2 cost reasons.

---

## §6 Source URL list

1. https://arxiv.org/pdf/2404.16130 — Microsoft, *From Local to Global: A GraphRAG Approach to Query-Focused Summarization*
2. https://arxiv.org/abs/2502.11371v3 — *RAG vs. GraphRAG: A Systematic Evaluation and Key Insights*
3. https://neo4j.com/blog/developer/graphrag-field-guide-rag-patterns/ — Neo4j, *GraphRAG Field Guide*
4. https://www.progressiverobot.com/2026/05/17/graph-enhanced-rag/ — *Graph-Enhanced RAG: Powerful Architectural Patterns Beyond Vector Search in Production*
5. https://osfoundry.io/articles/knowledge-graph-rag-hybrid-when-and-how — *Knowledge Graph RAG Hybrid: When It Helps and How to Build It*
6. https://milvus.io/blog/vector-graph-rag-without-graph-database.md — *Graph RAG Without a Graph Database*
7. https://www.learnwithparam.com/blog/hybrid-retrieval-rag-vector-graph-search — *Hybrid retrieval in RAG: vector + graph search*
8. https://www.microsoft.com/en-us/research/blog/graphrag-improving-global-search-via-dynamic-community-selection/ — *GraphRAG: Improving global search via dynamic community selection*
9. https://arxiv.org/pdf/2510.02827 — *StepChain GraphRAG: Reasoning Over Knowledge Graphs for Multi-Hop QA*
10. https://arxiv.org/html/2504.08893v1 — *Knowledge Graph-extended RAG for Question Answering*
11. https://arxiv.org/html/2603.24925v3 — *GraphER: An Efficient Graph-Based Enrichment and Reranking Method for RAG*
12. https://arxiv.org/abs/2608.29753 — *PAGE-RAG: Provenance-Aware Graph Evidence Promotion for Fixed-Budget Multi-hop RAG*
13. https://arxiv.org/abs/2607.22597 — *HyCE-RAG: Hypergraph Chain-of-Evidence RAG for Explainable Multi-hop QA*
14. https://docs.trustgraph.ai/overview/explainability.html — TrustGraph, *Explainability*
15. https://github.com/trustgraph-ai/trustgraph/blob/4e3bd85a/docs/tech-specs/query-time-explainability.md — TrustGraph, *query-time explainability*
16. https://doi.org/10.1109/acdsa67686.2026.11468256 — *Evaluating GraphRAG for Traceable and Interpretable Question Answering*
17. https://neo4j.com/blog/developer/graph-data-models-rag-applications/ — Neo4j, *Graph data models for RAG applications*
18. https://bigdataboutique.com/blog/knowledge-graphs-meet-rag-practical-integration-guide — *Knowledge Graphs Meet RAG: A Practical Integration Guide*
19. https://venturebeat.com/orchestration/stop-graphing-everything-when-graphrag-actually-beats-vector-rag — *Stop graphing everything: When GraphRAG actually beats vector RAG*
20. https://airbyte.com/agentic-data/graph-rag-vs-vector-rag — *Graph RAG vs Vector RAG: Choosing Your Retrieval Strategy*
21. https://www.tigergraph.com/blog/graphrag-vs-vector-rag/ — *GraphRAG vs Vector RAG: Which Wins for Enterprise AI?*
