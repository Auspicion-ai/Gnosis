# Graph-Enhanced Vector Search (Hybrid Graph + Vector Retrieval)

- **Topic:** Graph-enhanced vector search — hybrid retrieval that fuses a
  knowledge-graph layer with dense vector (embedding) retrieval.
- **Date:** 2026-09-08
- **Tier:** MUST (Zodiac, already in spec) / SHOULD (Astrographer increment)
- **Report slug:** `graph-enhanced-vector-search`
- **Primary focus:** Astrographer, Zodiac. **Secondary:** Incanter, Familiar,
  Solomon, Mystery, Astral.
- **Status:** Web-grounded research report (documentation deliverable only — no
  spec/code/test changes). Companion to the sibling report
  `graph-rag-astrographer.md` in this directory, which covers the broader Graph
  RAG topic; this report narrows to the **graph-enhanced vector search**
  sub-technique.

---

## §1 What the technique is (web-grounded, cited)

**Graph-enhanced vector search** is the family of hybrid retrieval designs in
which a **knowledge graph** (nodes, edges, properties) is used to *enhance* a
**dense vector** (embedding) retrieval path — rather than replacing it. The
source talk frames it as one of the Graph RAG layer methods: *"use both vector
and graph layer methods."* The motivating failures of pure vector RAG are:

1. **Vectors don't capture relationship data.** A vector index stores
   embeddings, not the relationships between entities. A knowledge graph
   supplies the relational context that enables **follow-up questions** and
   carries **domain knowledge** that similarity search cannot express.
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
nodes and edges). Graph-enhanced vector search then layers the vector and graph
paths together. The concrete sub-techniques named in the source talk:

- **Graph-enhanced vector search** — retrieve by vector similarity, then expand
  or re-rank using the graph neighborhood (the hybrid vector + graph search).
- **Dynamic graph query generation** — generate a graph query (Cypher-style or a
  sub-task DAG) from the natural-language question at query time.
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
  canonical GraphRAG paper; builds a knowledge graph, computes hierarchical
  community summaries, and does local (entity-level) + global (community-summary)
  retrieval. The GraphRAG project's own reference documents name
  **graph-enhanced vector search** as a first-class retriever
  ([graphrag.com](https://graphrag.com/reference/graphrag/graph-enhanced-vector-search/)),
  alongside the **parent-child retriever**
  ([graphrag.com](https://graphrag.com/reference/graphrag/parent-child-retriever/))
  and the **global community summary retriever**
  ([graphrag.com](https://graphrag.com/reference/graphrag/global-community-summary-retriever/)).
- **TigerVector** ([SIGMOD 2025](https://www.cs.purdue.edu/homes/csjgwang/pubs/SIGMOD25_TigerVector.pdf),
  [arXiv:2501.11216](https://arxiv.org/html/2501.11216)) — native vector search
  *inside* a graph database, supporting hybrid graph + vector queries in one
  engine. This is the "maturity" point: graph databases gaining first-class
  vector indexes.
- **Neo4j** — hybrid search combining a vector index with graph traversal
  ([neo4j.com](https://neo4j.com/developer/genai-ecosystem/hybrid-search/)) and a
  GraphRAG field guide that catalogs graph-enhanced vector search among the RAG
  patterns ([neo4j.com](https://neo4j.com/blog/developer/graphrag-field-guide-rag-patterns/)).
- **GNN-RAG** ([ACL 2025 Findings](https://doi.org/10.18653/v1/2025.findings-acl.856)) —
  graph neural retrieval over a knowledge graph to select the most relevant
  subgraph for LLM reasoning.
- **GRAG** ([arXiv:2405.16506](http://arxiv.org/pdf/2405.16506)) — graph
  retrieval-augmented generation that retrieves subgraphs and checks their
  faithfulness before generation.
- **Deep GraphRAG** ([arXiv:2601.11144](https://arxiv.org/abs/2601.11144v3)) —
  hierarchical retrieval with adaptive integration of graph and vector paths.
- **GraphER** ([arXiv:2603.24925](https://arxiv.org/html/2603.24925v3)) —
  graph-based enrichment and reranking of retrieval candidates.
- **Use Graph When It Needs** ([arXiv:2602.03578](https://doi.org/10.48550/arxiv.2602.03578)) —
  adaptively deciding when to invoke the graph path vs. the vector path.
- **High-Throughput Vector Similarity Search in Knowledge Graphs**
  ([arXiv:2304.01926](https://arxiv.org/pdf/2304.01926)) — vector similarity
  search over knowledge-graph nodes, the substrate for graph-enhanced retrieval.
- **NaviX** ([VLDB 2026](https://www.vldb.org/pvldb/vol18/p4438-sehgal.pdf)) —
  a native vector index design for graph DBMSs with predicate-agnostic search.
- **GEVS** ([github.com/organicdesign/GEVS](https://github.com/organicdesign/GEVS)) —
  a reference "graph-enhanced vector search" implementation.
- **Semantic Compression and Graph-Augmented Retrieval**
  ([alphaXiv](https://www.alphaxiv.org/abs/2507.19715)) — graph-augmented
  retrieval for enhanced vector search.
- **Graph-Augmented RAG Patterns in Azure HorizonDB**
  ([Microsoft Learn](https://learn.microsoft.com/en-us/azure/horizondb/ai/graph-rag)) —
  production graph+vector RAG patterns.

**Origin / status.** Graph-enhanced vector search is not a single paper; it is a
mature, widely-deployed retrieval pattern. The canonical reference is Microsoft
GraphRAG ([docs](https://microsoft.github.io/graphrag/),
[repo](https://github.com/microsoft/GraphRAG),
[research blog](https://www.microsoft.com/en-us/research/blog/graphrag-unlocking-llm-discovery-on-narrative-private-data/)),
which established the graph+vector hybrid as a production RAG design. The
technique is open-source (AGPL/MIT-compatible across the cited projects), runs
locally, and is the standard answer to the "vectors don't capture relationships"
problem.

---

## §2 How it applies to Astrographer

Astrographer is a **Graphical RAG engine + document store/wiki** where documents
are **Provident graphs** (nodes + edges) and retrieval is graph-aware
(`docs/specs/astrographer.md` §2). Its RAG surface (§4.3) consumes the Incanter
engine over HTTP and returns results tagged `source: 'local'|'incanter'|'zodiac'`
(§4.3.2). Graph-enhanced vector search maps onto it concretely:

- **The reference graph is already a knowledge graph (§4.2).** Astrographer's
  data model has `content`, `fact`, and `reference` node kinds (§4.2.1), with
  `link`/`embed` reference modes (§4.2.2). A `fact` is the single source of
  truth; `reference`/`embed` nodes point at it. This is exactly the
  "nodes / edges / properties" knowledge-graph model the source talk describes.
  Graph-enhanced vector search would use these edges to **enrich vector
  results**: when a vector search returns a `fact` node, the graph supplies the
  relationship context (which documents reference/embed it) — the "relationship
  data for follow-up questions" the source talk says vectors lack.

- **It directly answers "similarity ≠ relevance" (§4.2.3).** The consistency
  invariant makes the graph authoritative: a `fact`'s canonical `value` is the
  single source of truth, and every `embed`/`link` must stay consistent. A
  graph-enhanced retriever can ground a vector hit in the canonical fact value
  and its reference neighborhood, so a semantically-similar-but-wrong node is
  not surfaced as the answer. This is the "sourcing and tracing" explainability
  the source talk calls for, and it reuses the provenance the consistency model
  already tracks.

- **It is a natural extension of the existing RAG surface (§4.3).** The
  `ragQuery`/`ragStream` operations (§4.3.2) already accept `topK`/`filters`.
  Graph-enhanced retrieval is a retrieval-mode change (a graph-enrichment pass
  over the vector candidates), not a new engine. It is **not** a
  security-configuration feature, so under D4 it must be exposed on both the GUI
  (§4.6) and the MCP surface (`rag_query`/`rag_stream`, §4.5.1).

- **Index consistency is the key pitfall (§4.2.3).** When a `fact`'s `value`
  changes and embeds go `STALE`, the vector embeddings over that fact must be
  re-synced in the same pass, or graph-enhanced retrieval returns stale or
  contradictory content. This mirrors the hybrid-search research's index
  consistency pitfall (`docs/research/hybrid-search-research-notes.md` §5.3).

**Status in the spec:** **PROPOSED spec change.** `astrographer.md` §4.3 does
not currently name graph-enhanced vector search as a distinct technique. Incanter
already fuses graph-tension + vector (§4.8 of `incanter.md`), but that is a
different axis (a token-transition graph, not the semantic reference graph).
Graph-enhanced vector search over the reference graph is a complementary,
low-cost enhancement to the RAG surface. It is not parked — it is a natural
extension of the existing graph-aware design.

---

## §3 How it applies to Zodiac

Zodiac is a **backend/remote RAG companion** that **pre-graphs** and
**vector-embeds** crawled data so it can reply to Astrographer queries with
pre-graphed, vector-embedded data (`docs/specs/zodiac.md` §2). Its query-reply
surface (§4.3) is the suite's **primary in-spec home** for graph-enhanced vector
search:

- **The `hybrid` mode is literally the technique (§4.3.1, §4.3.2).** The query
  shape carries `mode` (`graph` | `vector` | `hybrid`) (§4.3.1). The result
  contract pins: *"Hybrid mode: returns a merged result combining graph and
  vector matches"* (§4.3.2). This is graph-enhanced vector search as specified —
  the pre-graphed data (dependency package metadata + relationships) is the
  graph layer, the vector-embedded items are the vector layer, and hybrid merges
  them.

- **The relationship data is exactly what Zodiac pre-graphs (§4.2).** The
  source talk's "relationship data" (dependency packages and their dependencies)
  is what Zodiac crawls and pre-graphs. Graph-enhanced retrieval lets a query
  about a package's dependencies return the relationship structure, not just
  similar text — the multi-hop / multi-step query data the source talk names.

- **The `stale` flag is a graph/vector consistency concern (§4.2, §4.3.2).** The
  RAG store has `EMPTY`/`READY`/`STALE`/`REFRESHING` states (§4.2). When a
  refresh crawl updates the graph, the vector embeddings must be re-synced in
  the same pass, or hybrid retrieval returns stale items (flagged `stale: true`,
  §4.3.2). This is the same index-consistency pitfall as Astrographer's.

- **D4 parity applies (§4.5).** The `zodiac_query` MCP tool accepts `mode`
  (§4.5). Graph-enhanced retrieval is a retrieval-mode change, not a
  security-configuration feature, so it must be reachable on both the GUI and
  MCP. The crawler-auth carve-out (§4.5) is unrelated.

**Status in the spec:** **ALREADY IN SPEC.** Zodiac's `hybrid` mode (§4.3.2) is
the suite's committed implementation of graph-enhanced vector search. The
remaining work is to pin the merge semantics (how graph and vector matches are
combined and ranked) and the re-sync of embeddings on graph refresh — both
design decisions to finalize at implementation.

---

## §4 How it applies to the other suite consumers

- **Incanter** (`docs/specs/incanter.md`): Incanter already fuses **dense vector
  similarity** with **graph-tension proximity** via weighted combination
  (`vector_weight = 0.6`, `graph_weight = 0.4`) in `POST /v1/query` (§4.8). This
  is a graph+vector hybrid, but on a **token-transition graph**, not a semantic
  reference graph. Graph-enhanced vector search over a semantic graph is a
  complementary axis. Because Incanter is a prototype (decision
  INCANTER-DISPOSITION, `docs/decisions.md`), graph-enhanced vector search is a
  design consideration for the **production** (non-JS) engine, not a near-term
  Incanter change. **Recommendation:** no direct Incanter work; carry the
  graph-enhanced design into the production engine.

- **Familiar** (`docs/specs/familiar.md`): Familiar's research domain (F4) routes
  **through Astrographer's RAG surface** (§4.3.3), which may return Zodiac
  results. Graph-enhanced vector search benefits Familiar **indirectly**: when it
  queries Astrographer's `rag_query`, graph-enhanced retrieval returns
  relationship context (which documents reference a fact), improving the
  assistant's answers. Familiar's own memory store is a **facts table** (§4.1.3),
  not a graph; the parked agent-memory vector-search layer (`docs/pending.md` D6)
  is a vector, not a graph, addition. **Recommendation:** no direct graph-enhanced
  work in Familiar; it inherits the technique through Astrographer/Zodiac.

- **Solomon** (`docs/specs/solomon.md`): Solomon is cross-instance search over
  `zodiac|astral|astrographer` instances (§4.2, §4.5.1). Its result aggregation
  (§4.2.2) is flat (grouped by peer, ordered by the peer's own relevance).
  Graph-enhanced vector search could improve cross-instance ranking by using each
  instance's graph structure, but Solomon's Phase-1 contract deliberately keeps
  cross-peer ordering unpinned (§7.4). **Recommendation:** NICE-TO-HAVE / PARKED —
  defer graph-aware cross-instance ranking until Phase-1 search semantics
  finalize.

- **Mystery** (`docs/specs/mystery.md` — **absent**; `docs/pending.md` #12):
  Mystery is a message board that crosslinks information references by post topic
  (`docs/architecture-overview.md` §4.3). Its topic→reference resolution
  (`docs/pending.md` #12) is a relationship-mapping problem that a knowledge
  graph (Astrographer's reference graph) could serve. **Recommendation:** when
  the Mystery contract is written, model its crosslinks as graph edges over
  Astrographer's reference graph. PARKED until the spec exists.

- **Astral** (`docs/specs/astral.md`): Astral is the wiki/publishing host (§2).
  It receives Provident graphs (§4.2) and serves them (§4.1). Graph-enhanced
  vector search applies to Astral only as a **serving** concern: the hosted pages
  are graphs, and the docs-as-product reachability surface (GAP-4/GAP-5,
  `docs/defects.md`) is about agent-facing markdown, not graph retrieval.
  **Recommendation:** no direct graph-enhanced work; Astral serves the graphs
  Astrographer produces.

---

## §5 Recommendation for the suite

Grounded in D1–D4:

- **MUST — Zodiac's `hybrid` mode is the suite's committed graph-enhanced vector
  search (§4.3.2).** It is already in the spec. The remaining work is to pin the
  merge semantics (how graph and vector matches are combined/ranked) and the
  re-sync of embeddings on graph refresh (§4.2). This is the primary home of the
  technique and should be finalized at implementation. **Already in spec.**

- **SHOULD — Graph-enhanced vector search over Astrographer's reference graph
  (§4.3).** Enrich vector results with the `reference`→`fact` relationship
  context (§4.2), so a vector hit on a `fact` also returns the documents that
  reference/embed it (relationship data for follow-up questions) and grounds the
  answer in the canonical fact value (explainability). This directly answers the
  source talk's "vectors don't capture relationship data" and "similarity ≠
  relevance" concerns, and is low-cost because the graph already exists as
  Provident graphs. **Proposed spec change** to `astrographer.md` §4.3. It is
  D2-compliant (runs locally over the Provident graph) and D4-compliant (a
  retrieval-mode change, exposed on both GUI §4.6 and MCP §4.5.1).

- **SHOULD — Index consistency on graph/vector re-sync.** When a `fact` value
  changes (Astrographer §4.2.3) or a Zodiac refresh crawl updates the graph
  (Zodiac §4.2), the vector embeddings must be re-synced in the same pass, or
  graph-enhanced retrieval returns stale/contradictory results. This is the
  dominant pitfall of the technique and must be pinned in both contracts.
  **Proposed spec change.**

- **NICE-TO-HAVE — Parent-child retrievers in Astrographer.** Return a node plus
  its containing document/section for context (`astrographer.md` §4.1.1). Low
  cost; improves answer context. **Proposed spec change.**

- **PARKED — Dynamic graph query generation and community summaries.** Generating
  a graph query / sub-task DAG at query time, and precomputing hierarchical
  community summaries (Microsoft GraphRAG global search), are the expensive
  sub-techniques. They conflict with D2 local-first cost/latency (multiple LLM
  calls per query; a pre-built global graph). The DAG-RAG research already
  DISCARDs pre-built global graph construction
  (`docs/research/dag-rag-research-notes.md` §4). Land them only after the
  MUST/SHOULD retrieval foundation ships.

**Overall recommendation tier: SHOULD** (with Zodiac's `hybrid` mode being
**MUST / already in spec**). Graph-enhanced vector search is not a foreign
technique for the suite — Zodiac already specifies it (§4.3.2) and Astrographer's
data model is already a knowledge graph (§4.2). The concrete SHOULD work is the
graph-enrichment pass over Astrographer's reference graph plus index-consistency
pinning; the expensive global techniques stay PARKED for D2 cost reasons. This is
consistent with the sibling report's MUST for the broader Graph RAG (whose MUST
core is topological `reference`→`fact` resolution, a distinct mechanism).

---

## §6 Source URL list

1. https://graphrag.com/reference/graphrag/graph-enhanced-vector-search/ — GraphRAG, *Graph-Enhanced Vector Search*
2. https://graphrag.com/concepts/intro-to-graphrag/ — GraphRAG, *Intro to GraphRAG*
3. https://graphrag.com/reference/graphrag/parent-child-retriever/ — GraphRAG, *Parent-Child Retriever*
4. https://graphrag.com/reference/graphrag/global-community-summary-retriever/ — GraphRAG, *Global Community Summary Retriever*
5. https://arxiv.org/pdf/2404.16130 — Microsoft, *From Local to Global: A GraphRAG Approach to Query-Focused Summarization*
6. https://microsoft.github.io/graphrag/ — Microsoft GraphRAG documentation
7. https://github.com/microsoft/GraphRAG — Microsoft GraphRAG repository
8. https://www.microsoft.com/en-us/research/blog/graphrag-unlocking-llm-discovery-on-narrative-private-data/ — Microsoft Research, *GraphRAG: Unlocking LLM discovery on narrative private data*
9. https://learn.microsoft.com/en-us/azure/horizondb/ai/graph-rag — Microsoft Learn, *Graph-Augmented RAG Patterns in Azure HorizonDB*
10. https://www.cs.purdue.edu/homes/csjgwang/pubs/SIGMOD25_TigerVector.pdf — *TigerVector: Supporting Vector Search in Graph Databases for Advanced RAGs* (SIGMOD 2025)
11. https://arxiv.org/html/2501.11216 — *TigerVector* (arXiv)
12. https://neo4j.com/developer/genai-ecosystem/hybrid-search/ — Neo4j, *Hybrid Search*
13. https://neo4j.com/blog/developer/graphrag-field-guide-rag-patterns/ — Neo4j, *GraphRAG Field Guide*
14. https://doi.org/10.18653/v1/2025.findings-acl.856 — *GNN-RAG: Graph Neural Retrieval for Efficient LLM Reasoning on Knowledge Graphs* (ACL 2025 Findings)
15. http://arxiv.org/pdf/2405.16506 — *GRAG: Graph Retrieval-Augmented Generation*
16. https://arxiv.org/abs/2601.11144v3 — *Deep GraphRAG: A Balanced Approach to Hierarchical Retrieval and Adaptive Integration*
17. https://arxiv.org/html/2603.24925v3 — *GraphER: An Efficient Graph-Based Enrichment and Reranking Method for RAG*
18. https://doi.org/10.48550/arxiv.2602.03578 — *Use Graph When It Needs: Efficiently and Adaptively Integrating RAG with Graphs*
19. https://arxiv.org/pdf/2304.01926 — *High-Throughput Vector Similarity Search in Knowledge Graphs*
20. https://www.vldb.org/pvldb/vol18/p4438-sehgal.pdf — *NaviX: A Native Vector Index Design for Graph DBMSs* (VLDB 2026)
21. https://github.com/organicdesign/GEVS — *GEVS: Graph-Enhanced Vector Search* (reference implementation)
22. https://www.alphaxiv.org/abs/2507.19715 — *Semantic Compression and Graph-Augmented Retrieval for Enhanced Vector Search*
