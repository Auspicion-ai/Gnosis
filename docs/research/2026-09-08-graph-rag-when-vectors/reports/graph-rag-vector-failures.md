# Vector RAG Failure Modes — Relationship Data, Similarity vs Relevance, Explainability, Maturity

- **Topic:** Vector RAG failure modes (relationship data, similarity vs relevance, explainability, maturity) — and how a graph layer (Graph RAG) remedies them.
- **Date:** 2026-09-08
- **Tier:** MUST
- **Report slug:** `graph-rag-vector-failures`
- **Primary focus:** Astrographer, Zodiac. **Secondary:** Incanter, Familiar, Solomon, Mystery, Astral.
- **Status:** Web-grounded research report (documentation deliverable only — no spec/code/test changes).

---

## §1 What the technique is (web-grounded, cited)

The source talk frames the **normal RAG pipeline** as a linear path:
*Question / Retriever / Data source request → Response.* It then enumerates the
**failure modes of vector RAG** — the reasons a pure vector (embedding-similarity)
retriever is insufficient — and positions a **knowledge graph** as the remedy:

1. **Vectors don't capture relationship data.** A vector index stores embeddings,
   not the relationships between entities. A knowledge graph supplies the
   relational context that enables **follow-up questions** and carries **domain
   knowledge** that similarity search cannot express. Relationship data enables
   **filtering**, **multi-step query data**, and **multi-hop questions**.
2. **Vector similarity ≠ relevance.** High cosine similarity to a query is not the
   same as being the relevant fact. This is a well-documented failure: cosine
   similarity is a geometric measure, not a relevance measure, and it degrades on
   exact-match, rare-term, and relationship-dependent queries
   ([Why Cosine Similarity Fails in RAG](https://dev.to/mossforge/why-cosine-similarity-fails-in-rag-and-what-to-use-instead-pb5);
   [Your Vector Database Is Not a Search Engine](https://dev.to/gabrielanhaia/your-vector-database-is-not-a-search-engine-heres-why-thats-killing-your-rag-2db5);
   [Why Vector Search Alone Isn't Enough: Hybrid Retrieval for RAG](https://www.infoq.com/articles/vector-search-hybrid-retrieval-rag/)).
3. **Lack of explainability.** Governance questions are more easily answered by
   **sourcing and tracing** — walking the retrieval back to the originating fact —
   than by a black-box similarity score. RAG governance (source authority, access
   control, freshness, auditability) and provenance-native answer traces are
   active research areas ([RAG Governance](https://thomasthelliez.com/blog/rag-governance-source-authority-access-control-auditability/);
   [ProvenAI: Provenance-Native Traces of Evidence](https://doi.org/10.48550/arxiv.2606.26449);
   [Towards Transparent RAG](https://arxiv.org/html/2505.13258);
   [TrustGraph Explainability](https://docs.trustgraph.ai/overview/explainability.html)).
4. **Maturity not on par with existing database systems.** Vector stores are
   younger than established database systems; graph databases bring mature query,
   constraint, and transaction semantics. Enterprise guidance repeatedly warns
   against building production AI on a raw vector store alone
   ([Vector vs graph databases](https://vercel.com/i/vector-vs-graph-databases);
   [Graph Database vs Vector Database: What Enterprise AI Needs](https://www.tigergraph.com/blog/vector-database-vs-graph-database-for-ai/);
   [The Architectural Trap That Breaks Production AI Systems](https://pub.towardsai.net/vector-db-vs-graph-db-the-architectural-trap-that-breaks-production-ai-systems-d34c48142dd7)).

The **knowledge graph** model is: **nodes** (noun-focused entities), **edges**
(relationship-focused links), and **properties** (attributes attachable to both
nodes and edges). **Graph RAG** is then *"RAG where the retrieval path also
includes a knowledge graph — use both vector and graph layer methods."* The
techniques layered on top include **graph-enhanced vector search**, **dynamic
graph query generation**, **parent-child retrievers**, **community summaries**,
and **graph enrichments** ([Graph RAG vs Vector RAG](https://airbyte.com/agentic-data/graph-rag-vs-vector-rag);
[How to Implement Graph RAG Using Knowledge Graphs and Vector Databases](https://towardsdatascience.com/how-to-implement-graph-rag-using-knowledge-graphs-and-vector-databases-60bb69a22759/);
[Microsoft GraphRAG — From Local to Global](https://arxiv.org/pdf/2404.16130)).

**Key cited sources:**
- *Beyond Vector Similarity: A Structural Analysis of Graph-Augmented Retrieval
  for Industrial Knowledge Graphs* ([arXiv:2606.06003](https://arxiv.org/html/2606.06003)) — shows graph structure (paths, hops, shared entities) is a complementary relevance signal to vector similarity.
- *When Vector Search Fails: Why Knowledge Graphs Handle Queries Embeddings Can't*
  ([tianpan.co](https://tianpan.co/blog/2026/04/20/knowledge-graphs-vs-vector-search-retrieval)) — concrete failure cases where embeddings miss relationship/multi-hop queries.
- *Why basic RAG fails at multi-hop reasoning (and how GraphRAG fixes it)*
  ([The New Stack](https://thenewstack.io/graphrag-multi-hop-reasoning-python/)) and *GraphRAG in Production: When Vector Search Fails at Multi-Hop Reasoning* ([tianpan.co](https://tianpan.co/blog/2026-04-12-graphrag-production-when-vector-search-fails-multi-hop-reasoning)) — the relationship/multi-hop failure mode.
- *Mitigating Lost-in-Retrieval Problems in Retrieval Augmented Multi-Hop QA*
  ([ACL 2025](https://aclanthology.org/2025.acl-long.1089.pdf)) — retrieval quality bounds multi-hop answer quality.
- *Policy-Governed RAG* ([arXiv:2510.19877](https://doi.org/10.48550/arxiv.2510.19877)) and *ContextNest: Verifiable Context Governance for Autonomous AI Agents* ([arXiv:2607.02116](https://arxiv.org/html/2607.02116v2)) — governance/explainability of RAG context.
- *TigerVector: Supporting Vector Search in Graph Databases for Advanced RAGs*
  ([SIGMOD 2025](https://www.cs.purdue.edu/homes/csjgwang/pubs/SIGMOD25_TigerVector.pdf)) — the maturity point: vector search as a capability *inside* a graph database, not a separate immature store.

---

## §2 How it applies to Astrographer

Astrographer is a **graphical RAG engine + document store/wiki** over **Provident
graphs** (`docs/specs/astrographer.md` §2, §4.1.1). Its data model is, in effect,
a knowledge graph with properties, so the four vector-RAG failure modes map onto
it directly:

- **Relationship data** (`astrographer.md` §4.2). The cross-link/data-embed
  mechanism defines `fact` (single source of truth), `reference`, `link`, and
  `embed` nodes/edges (§4.2.1, §4.2.2). This is exactly the "relationship data"
  the source talk says vectors miss. **Filtering by relationship** (e.g. "all
  documents that embed fact X") and **multi-step/multi-hop queries** over the
  `reference`→`fact` graph are natural graph operations. The DAG-RAG research
  already maps this: resolving a `reference` requires resolving its target `fact`
  first — a dependency-ordered topological walk (`docs/research/dag-rag-research-notes.md`
  §3, §4). **Already in the spec** as the graph substrate; the retrieval behavior
  over it is a proposed spec change (§4.3).

- **Similarity ≠ relevance** (`astrographer.md` §4.3). `ragQuery`/`ragStream`
  (§4.3.2) consume Incanter's hybrid graph-tension + vector query
  (`incanter.md` §4.8) and Zodiac's `graph`/`vector`/`hybrid` modes
  (`zodiac.md` §4.3.2). The `source` union `local|incanter|zodiac` (§4.3.2) is the
  vector+graph fusion seam. The structural relevance signal (graph-tension
  proximity) is the counter to pure cosine similarity. The **lexical (BM25) leg**
  for exact-match on `factKey`/`title`/`tags` is the missing relevance leg —
  recorded as GAP-2 / parked D5 (`docs/defects.md`, `docs/pending.md` D5). **In
  spec** (hybrid axis); the lexical leg is a proposed spec change / SHOULD-HAVE.

- **Explainability** (`astrographer.md` §4.2.3, §4.3.2). The consistency model
  already tracks every `link`/`embed` to its target and exposes
  `getConsistencyReport(wikiId)` (§4.1.3, §4.5.1 `get_consistency_report`). The
  `ragQuery` result already returns `{documentId, nodeId, score, snippet, source}`
  (§4.3.2). Adding a **trace path** (the edges walked to reach the answer) reuses
  this provenance and directly answers the source talk's "governance questions
  are more easily answered by sourcing and tracing." **Proposed spec change** to
  §4.3.

- **Maturity** (`astrographer.md` §4.1.4, §4.4). Astrographer's graph is a
  Provident graph (Layer 1, `docs/architecture-overview.md` §5.1) — a first-class,
  versioned graph model with optimistic concurrency (§4.1.4) and a canonical push
  format (§4.4, decision GRAPH-PUSH-FORMAT). This is a mature substrate relative
  to a raw vector store, and it is the reason the graph layer is the natural
  primary retrieval structure here. **Already in spec.**

**Net for Astrographer:** the graph substrate already mitigates the relationship
and maturity failure modes; the concrete MUST/SHOULD work is (a) graph-enhanced
retrieval over the reference graph (topological resolution, per DAG-RAG), (b) an
**explainability/trace surface** on RAG results, and (c) the lexical leg for the
similarity-vs-relevance gap (GAP-2).

---

## §3 How it applies to Zodiac

Zodiac is a **graphical RAG engine integrated with web crawlers** that replies to
Astrographer with **pre-graphed and vector-embedded data** (`docs/specs/zodiac.md`
§2, §4.2). It is the suite's clearest Graph RAG engine and the one most exposed
to the vector-RAG failure modes:

- **Relationship data** (`zodiac.md` §4.2). **Pre-graphing** parses crawled raw
  material into a graph (nodes + edges) — e.g. a dependency package's metadata,
  its dependencies, and their relationships. This is a knowledge graph built from
  crawled data, so relationship data is captured at ingestion. **Already in spec.**

- **Similarity ≠ relevance** (`zodiac.md` §4.3.2). The query surface exposes
  `graph` (returns the pre-graphed subgraph), `vector` (top-`limit` by
  similarity), and `hybrid` (merged graph + vector) modes. The `hybrid` mode is
  **graph-enhanced vector search** in its purest form — the source talk's "use
  both vector and graph layer methods." **Already in spec.**

- **Explainability** (`zodiac.md` §4.3.2). Results carry `sourceCrawlId` and
  `stale`, giving provenance back to the crawl. A trace path from a result to its
  source crawl is a natural extension. **Proposed spec change** (SHOULD).

- **Maturity** (`zodiac.md` §4.2). The RAG store has a mature state model
  (`EMPTY`/`READY`/`STALE`/`REFRESHING`) and a defined embedding-failure path
  (§6.4). As a remote/backend managed-service component (D2-CLARIFICATION,
  MANAGED-SERVICE-PHASING), Zodiac is where the suite can afford a heavier
  graph+vector stack. **Already in spec.**

**Net for Zodiac:** the graph+vector hybrid is already the engine's design
(`zodiac.md` §4.2, §4.3.2). The concrete addition is explainability/trace
(provenance to `sourceCrawlId`); the relationship and maturity failure modes are
already addressed by pre-graphing and the store-state model.

---

## §4 How it applies to the other suite consumers

- **Incanter** (`docs/specs/incanter.md`): Incanter is the prototype Graph-RAG
  engine Astrographer consumes over HTTP (`incanter.md` §2, §5). Its hybrid query
  (`incanter.md` §4.8) already fuses **graph-tension + vector**
  (`vector_weight`/`graph_weight`), so the similarity-vs-relevance failure mode is
  already mitigated by a structural relevance signal. The graph-tension model
  (`incanter.md` §4.6) is a graph-based retrieval signal, so relationship data is
  native. The missing leg is lexical (BM25) — GAP-2 / parked D5
  (`docs/defects.md`, `docs/pending.md` D5). **Recommendation:** the graph-enhanced
  axis is already in spec; the concrete gap is the lexical leg, not a new Graph
  RAG technique.

- **Familiar** (`docs/specs/familiar.md`): Familiar's knowledge domain delegates
  to Astrographer's RAG surface (`familiar.md` §4.3.1) and its research domain
  routes through Astrographer's `rag_query` (which may return Zodiac results,
  `familiar.md` §4.3.3). Familiar's own memory store is a **facts table**
  (`familiar.md` §4.1.3) — individually addressable fact records, not a graph.
  The vector-RAG failure modes apply to Familiar **indirectly**, through the
  Astrographer/Zodiac surfaces it consumes. **Recommendation:** no direct work;
  Familiar inherits the graph layer and its provenance through Astrographer/Zodiac.

- **Solomon** (`docs/specs/solomon.md`): Solomon is cross-instance search over
  `zodiac|astral|astrographer` instances (`solomon.md` §4.2, §4.5.1). Its result
  aggregation (`solomon.md` §4.2.2) is flat (grouped by peer, ordered by the
  peer's own relevance). The similarity-vs-relevance and explainability failure
  modes apply to cross-instance ranking, but Solomon's Phase-1 contract
  deliberately keeps cross-peer ordering unpinned (`solomon.md` §7.4).
  **Recommendation:** NICE-TO-HAVE / PARKED — defer graph-aware cross-instance
  ranking and provenance until Phase-1 search semantics finalize.

- **Mystery** (`docs/specs/mystery.md` — **absent**; `docs/pending.md` #12):
  Mystery is a message board that crosslinks information references by post topic
  (`docs/architecture-overview.md` §4.3). Its topic→reference resolution
  (`docs/pending.md` #12) is a **relationship-mapping** problem — exactly the
  failure mode vectors miss. **Recommendation:** when the Mystery contract is
  written, model its crosslinks as graph edges over Astrographer's reference
  graph. PARKED until the spec exists.

- **Astral** (`docs/specs/astral.md`): Astral is the wiki/publishing host
  (`astral.md` §2). It receives Provident graphs (`astral.md` §4.2) and serves
  them (`astral.md` §4.1). The vector-RAG failure modes apply to Astral only as a
  **serving** concern: the hosted pages are graphs, and the docs-as-product
  reachability surface (GAP-4/GAP-5, `docs/defects.md`) is about agent-facing
  markdown, not graph retrieval. **Recommendation:** no direct work; Astral serves
  the graphs Astrographer produces.

---

## §5 Recommendation for the suite

Grounded in D1–D4:

- **MUST — Keep the graph layer as the primary retrieval structure (relationship
  data + maturity).** Astrographer's data model is already a knowledge graph
  (`astrographer.md` §4.2) and Zodiac pre-graphs crawled data (`zodiac.md` §4.2).
  This is the correct answer to the "vectors don't capture relationship data" and
  "maturity" failure modes, and it is D2-compliant (runs locally over the
  Provident graph) and D1-compliant (no proprietary vector service). **Already in
  spec** — preserve it.

- **MUST — Graph-enhanced retrieval over Astrographer's reference graph
  (topological resolution of `reference`→`fact`).** Resolving `reference`→`fact`
  in dependency order is the core, low-risk win for relationship-aware, multi-hop
  answers. Consistent with the DAG-RAG research's MUST-HAVE
  (`docs/research/dag-rag-research-notes.md` §4) and with D2/D4. **Proposed spec
  change** to `astrographer.md` §4.3.

- **SHOULD — Explainability / trace surface on RAG results.** Extend the
  `ragQuery` result (`astrographer.md` §4.3.2) with a **trace path** (the edges
  walked to reach the answer), reusing the provenance already tracked by the
  consistency model (`astrographer.md` §4.2.3). This directly answers the source
  talk's governance/sourcing point and is cheap because the graph already records
  every reference. **Proposed spec change** to `astrographer.md` §4.3.

- **SHOULD — Zodiac explainability (provenance to `sourceCrawlId`).** Zodiac
  results already carry `sourceCrawlId` and `stale` (`zodiac.md` §4.3.2); expose a
  trace path from a result to its source crawl. **Proposed spec change** to
  `zodiac.md` §4.3.

- **SHOULD — Lexical (BM25) leg for the similarity-vs-relevance gap.** The
  exact-match retrieval of `factKey`/`title`/`tags` is the concrete counter to
  "vector similarity ≠ relevance." This is GAP-2 / parked D5 (`docs/defects.md`,
  `docs/pending.md` D5) and the hybrid-search research's SHOULD-HAVE
  (`docs/research/hybrid-search-research-notes.md` §9). **Proposed spec change** /
  handoff to Incanter.

- **PARKED — Community summaries (Microsoft GraphRAG global search) and dynamic
  graph query generation.** Precomputing hierarchical community summaries over
  the wiki reference graph is expensive and conflicts with D2 local-first
  cost/latency; the DAG-RAG research already DISCARDs pre-built global graph
  construction (`docs/research/dag-rag-research-notes.md` §4). Dynamic graph
  query generation is the most complex phase (multiple LLM calls per query).
  Revisit only if a global/theme question use case surfaces.

**Overall recommendation tier: MUST.** The vector-RAG failure modes are the
*justification* for the suite's graph-first architecture, not a new technique to
add. Astrographer and Zodiac are already graphical RAG engines over
knowledge-graph-shaped data, which addresses the relationship-data and maturity
failure modes. The concrete MUST/SHOULD work is graph-enhanced retrieval over the
reference graph, an explainability/trace surface on RAG results, and the lexical
leg for the similarity-vs-relevance gap; the expensive global techniques stay
PARKED for D2 cost reasons.

---

## §6 Source URL list

1. https://airbyte.com/agentic-data/graph-rag-vs-vector-rag — *Graph RAG vs Vector RAG: Choosing Your Retrieval Strategy*
2. https://arxiv.org/html/2606.06003 — *Beyond Vector Similarity: A Structural Analysis of Graph-Augmented Retrieval for Industrial Knowledge Graphs*
3. https://towardsdatascience.com/how-to-implement-graph-rag-using-knowledge-graphs-and-vector-databases-60bb69a22759/ — *How to Implement Graph RAG Using Knowledge Graphs and Vector Databases*
4. https://www.cs.purdue.edu/homes/csjgwang/pubs/SIGMOD25_TigerVector.pdf — *TigerVector: Supporting Vector Search in Graph Databases for Advanced RAGs* (SIGMOD 2025)
5. https://tianpan.co/blog/2026/04/20/knowledge-graphs-vs-vector-search-retrieval — *When Vector Search Fails: Why Knowledge Graphs Handle Queries Embeddings Can't*
6. https://doi.org/10.48550/arxiv.2510.19877 — *Policy-Governed RAG*
7. https://thomasthelliez.com/blog/rag-governance-source-authority-access-control-auditability/ — *RAG Governance: Source Authority, Access Control, Freshness, and Auditability*
8. https://github.com/dakshtrehan/ragcompliance — *ragcompliance* (RAG compliance tooling)
9. https://docs.trustgraph.ai/overview/explainability.html — TrustGraph, *Explainability*
10. https://arxiv.org/html/2607.02116v2 — *ContextNest: Verifiable Context Governance for Autonomous AI Agents*
11. https://vercel.com/i/vector-vs-graph-databases — *Vector vs graph databases: How to choose for retrieval problems*
12. https://tianpan.co/blog/2026/04/19/graphrag-vs-vector-rag-architecture-decision — *GraphRAG vs. Vector RAG: The Architecture Decision Teams Make Too Late*
13. https://www.tigergraph.com/blog/vector-database-vs-graph-database-for-ai/ — *Graph Database vs Vector Database: What Enterprise AI Needs*
14. https://pub.towardsai.net/vector-db-vs-graph-db-the-architectural-trap-that-breaks-production-ai-systems-d34c48142dd7 — *Vector DB vs Graph DB: The Architectural Trap That Breaks Production AI Systems*
15. https://techbytes.app/posts/vector-vs-graph-databases-rag-2026/ — *Vector vs Graph Databases for RAG [Deep Dive] 2026*
16. https://typegraph.ai/blog/multi-hop-reasoning-knowledge-graph-rag — *Multi-Hop Reasoning Over Knowledge Graphs in RAG*
17. https://thenewstack.io/graphrag-multi-hop-reasoning-python/ — *Why basic RAG fails at multi-hop reasoning (and how GraphRAG fixes it)*
18. https://tianpan.co/blog/2026-04-12-graphrag-production-when-vector-search-fails-multi-hop-reasoning — *GraphRAG in Production: When Vector Search Fails at Multi-Hop Reasoning*
19. https://enison.ai/en/blog/knowledge-graph-rag-implementation — *Implementation Guide for Answering Complex Queries with Knowledge Graph × RAG*
20. https://aclanthology.org/2025.acl-long.1089.pdf — *Mitigating Lost-in-Retrieval Problems in Retrieval Augmented Multi-Hop Question Answering* (ACL 2025)
21. https://arxiv.org/pdf/2404.16130 — Microsoft, *From Local to Global: A GraphRAG Approach to Query-Focused Summarization*
22. https://microsoft.github.io/graphrag/ — Microsoft GraphRAG documentation
23. https://github.com/microsoft/graphrag — Microsoft GraphRAG repository
24. https://www.infoq.com/articles/vector-search-hybrid-retrieval-rag/ — *Why Vector Search Alone Isn't Enough: Hybrid Retrieval for RAG*
25. https://blog.acur.ai/the-insanity-of-relying-on-vector-embeddings-why-rag-fails/ — *The Insanity of Relying on Vector Embeddings: Why RAG Fails*
26. https://dev.to/gabrielanhaia/your-vector-database-is-not-a-search-engine-heres-why-thats-killing-your-rag-2db5 — *Your Vector Database Is Not a Search Engine*
27. https://dev.to/mossforge/why-cosine-similarity-fails-in-rag-and-what-to-use-instead-pb5 — *Why Cosine Similarity Fails in RAG*
28. https://docs.bswen.com/blog/2026-03-26-why-rag-returns-irrelevant-results/ — *Why Does RAG Return Irrelevant Results?*
29. https://doi.org/10.48550/arxiv.2606.26449 — *ProvenAI: Provenance-Native Traces of Evidence in Generated Answers*
30. https://arxiv.org/html/2505.13258 — *Towards Transparent RAG: Fostering Evidence Traceability in LLM Generation via Reinforcement Learning*
31. https://doi.org/10.5281/zenodo.20542423 — *Structured Attribution Traces for Explainable Agentic RAG*
32. https://aclanthology.org/2024.emnlp-main.347.pdf — *Model Internals-based Answer Attribution for Trustworthy RAG* (EMNLP 2024)
33. https://www.arxiv.org/pdf/2604.06211 — *Illocutionary Explanation Planning for Source-Faithful Explanations in RALMs*
34. https://www.mdpi.com/2073-431X/14/9/382 — *GraphTrace: A Modular Retrieval Framework Combining Knowledge Graphs and LLMs for Multi-Hop QA*
35. https://arxiv.org/pdf/2510.02827 — *StepChain GraphRAG: Reasoning Over Knowledge Graphs for Multi-Hop QA*
36. https://neo4j.com/blog/developer/rag-tutorial/ — Neo4j, *RAG tutorial*
37. https://theaidatabaseblog.com/learn/graphrag/ — *GraphRAG: Combining Knowledge Graphs and Vectors*
