# Graph RAG — Application to Zodiac (remote graphical RAG companion + web crawlers)

- **Topic:** Graph RAG — retrieval paths that include a knowledge graph alongside vector search (graph-enhanced vector search, dynamic graph query generation, parent-child retrievers, community summaries, graph enrichments)
- **Date:** 2026-09-08
- **Tier:** MUST
- **Report slug:** `graph-rag-zodiac`
- **Primary focus:** Astrographer and Zodiac; secondary coverage of Incanter, Familiar, Solomon, Mystery, Astral.
- **Status:** Web-grounded research report (not a behavior contract). Does not modify any `docs/specs/*.md`, code, test, or tracker.

---

## §1 What the technique is (web-grounded, cited)

Graph RAG is the family of retrieval-augmented-generation designs in which the
**retrieval path includes a knowledge graph** in addition to (or instead of) a
flat vector index. The motivating failures of pure vector RAG are well
documented: **vectors do not capture relationship data** (a knowledge graph
supplies the context for relationships that enable follow-up and multi-hop
questions), **vector similarity is not the same as relevance** (a token that
appears frequently in a corpus can be embedded close to a query without being
the answer), and **vector RAG lacks explainability** (governance questions are
more easily answered by sourcing and tracing through an explicit graph). See
[Similarity Isn't Accuracy in GenAI: Vector RAG vs GraphRAG (DZone)](https://dzone.com/articles/when-similarity-isnt-accuracy-genai-vector-rag-vs-graphrag)
and [Knowledge Graph RAG: Structured Retrieval for AI Agents (Redis)](https://redis.io/en/blog/knowledge-graph-rag-structured-retrieval-ai-agents/).

The canonical reference is Microsoft's **GraphRAG** — *From Local to Global: A
GraphRAG Approach to Query-Focused Summarization* — which builds a knowledge
graph from a corpus, detects **communities** of related entities, and computes
**hierarchical community summaries** so that both local (entity/relation) and
global (theme/summary) queries can be answered
([arXiv 2404.16130](https://arxiv.org/pdf/2404.16130),
[Microsoft Research blog](https://www.microsoft.com/en-us/research/blog/graphrag-new-tool-for-complex-data-discovery-now-on-github/),
[GraphRAG docs](https://github.com/microsoft/graphrag/blob/main/docs/index.md)).
The community-summarization pass is the expensive part; **LightRAG** showed a
lighter alternative that skips community summaries and instead performs
**dual-level retrieval** (low-level entity/relation + high-level theme) over a
deduplicated graph, at roughly 1–2 orders of magnitude lower token cost
([LightRAG arXiv 2410.05779](https://arxiv.org/html/2410.05779v1),
[LightRAG ACL Anthology](https://aclanthology.org/2025.findings-emnlp.568/),
[Neo4j: Under the covers with LightRAG](https://neo4j.com/blog/developer/under-the-covers-with-lightrag-retrieval/)).

The concrete retrieval techniques in the Graph RAG family, each with a
web-grounded reference:

1. **Graph-enhanced vector search** — retrieve a candidate set by vector
   similarity, then expand/refine it by walking the graph (neighbors, paths,
   subgraphs) so the final context is relationship-aware
   ([GraphRAG reference: Graph-Enhanced Vector Search](https://graphrag.com/reference/graphrag/graph-enhanced-vector-search/),
   [Neo4j GraphRAG field guide](https://neo4j.com/blog/developer/graphrag-field-guide-rag-patterns/)).
2. **Dynamic graph query generation** — translate a natural-language query into
   a graph query (e.g. Cypher/GQL) at query time, so relationship and multi-hop
   questions are answered by an explicit graph traversal rather than by
   embedding similarity
   ([GraphRAG reference: Dynamic Cypher Generation](https://graphrag.com/reference/graphrag/dynamic-cypher-generation/),
   [Multi-Agent GraphRAG: A Text-to-Cypher Framework](https://arxiv.org/pdf/2511.08274)).
3. **Parent-child retrievers** — store a document as a hierarchy (parent
   document → child chunks) and retrieve at the child granularity while
   returning the parent context, preserving both precision and context
   ([GraphRAG reference: Parent-Child Retriever](https://graphrag.com/reference/graphrag/parent-child-retriever/)).
4. **Community summaries** — group related entities into communities and
   precompute a summary per community so global/thematic queries are answered
   from aggregated context (Microsoft GraphRAG, above).
5. **Graph enrichments** — entity resolution, deduplication, and external-KG
   augmentation that improve graph quality before retrieval
   ([GraphER: Graph-Based Enrichment and Reranking](https://arxiv.org/html/2603.24925v3),
   [KG-Infused RAG](https://arxiv.org/html/2506.09542)).

Two additional results are directly relevant to a **web-crawling** RAG engine:
- **Knowledge-graph construction from web crawls** is an established pipeline —
  crawl → extract entities/relations → build a graph → retrieve
  ([Knowledge Graph RAG: Agentic Crawling and Graph Construction](https://arxiv.org/html/2604.14220),
  [GraphRAG + Web Scraping (KnowledgeSDK)](https://knowledgesdk.com/blog/graphrag-web-scraping),
  [Build a Knowledge Graph from Any Website (KnowledgeSDK)](https://knowledgesdk.com/blog/knowledge-graph-from-website)).
- **Dependency-graph RAG** is a named pattern for software ecosystems — modeling
  packages and their dependency relationships as a graph so queries about
  dependencies, transitive deps, and version relationships are answered
  structurally ([DepsRAG](https://github.com/Mohannadcse/DepsRAG)). This is the
  exact domain Zodiac targets (dependency packages).

**Local-first feasibility (D2).** Graph RAG is fully implementable on local
hardware: LightRAG is explicitly lightweight, and community projects run
GraphRAG against a local Ollama embedding endpoint
([GraphRAG with Ollama](https://github.com/eastsea17/GraphRAG_with_Ollama),
[local-graphrag-mcp](https://github.com/nonatofabio/local-graphrag-mcp)). The
techniques are open algorithms with no proprietary service dependency, so they
are AGPL-3.0-compatible (D1).

---

## §2 How it applies to Astrographer (reference `astrographer.md` §4.x)

Astrographer is a **Graphical RAG engine + document store/wiki** whose documents
are already **Provident graphs** (nodes + edges) with a **`fact`-node
single-source-of-truth** model (`astrographer.md` §4.1, §4.2). Graph RAG is
therefore not a foreign technique to add — it is the substrate Astrographer is
built on. The incremental value of the Graph RAG family is in how Astrographer
**consumes relationship data**, especially from Zodiac.

- **§4.2 cross-link/data-embed consistency** — the `fact`/`reference`/`embed`
  model is a knowledge-graph-like structure. **Graph-enhanced vector search**
  (§1.1) maps directly: retrieve a candidate `fact` node by vector similarity,
  then expand along `reference`/`embed` edges to surface the documents that
  depend on it. This reinforces the single-source-of-truth invariant and is a
  **proposed spec change** (not yet in the contract — the contract pins the
  consistency mechanism but not a graph-expansion retrieval step).
- **§4.3 RAG query surface** — `ragQuery`/`ragStream` return results tagged
  `source: 'local'|'incanter'|'zodiac'` (§4.3.2). When Zodiac is reachable, its
  results are **pre-graphed** data (§5.7). Graph RAG techniques (community
  summaries, graph-enhanced expansion) apply to how Astrographer **merges and
  presents** Zodiac's relationship-aware results alongside Incanter's
  graph-tension results. This is a **proposed spec change** to the merge/ranking
  semantics of §4.3.2.
- **§4.3.3 fail-state semantics** — `EngineUnavailable`/`EngineError` already
  cover Incanter; Zodiac unreachable degrades to local-only (§5.7, FS-15). A
  graph-expansion step must inherit the same degradation (a graph-expansion
  failure must not break the local result path). This is a **proposed spec
  change** (a new fail-state or a degradation rule), not yet pinned.
- **§4.5 MCP surface / §4.6 GUI** — any graph-expansion or graph-query mode must
  be reachable through both `rag_query`/`rag_stream` and the GUI RAG panel (D4).
  This is a **proposed spec change** (a query-mode parameter), consistent with
  the existing `topK`/`filters` parameters.

**Status summary for Astrographer:** the graph substrate is **already in the
spec** (§4.1/§4.2); the Graph RAG *retrieval* techniques (graph-enhanced
expansion, community summaries over the wiki, dynamic graph query over the
reference graph) are **proposed spec changes** — none are currently pinned in
`astrographer.md`.

---

## §3 How it applies to Zodiac (reference `zodiac.md` §4.x)

Zodiac is the **primary** consumer of Graph RAG in the suite. Its entire value
proposition is to be a **graphical RAG engine** that **pre-graphs** and
**vector-embeds** web-crawled data (`zodiac.md` §2, §4.2). Graph RAG is not an
optional enhancement for Zodiac — it is the core of the design.

- **§4.2 RAG engine (pre-graphing + vector-embedding)** — this is the
  knowledge-graph-construction-from-web-crawl pipeline (§1). The "pre-graphing"
  step parses crawled raw material into **nodes and edges** representing a
  dependency package's metadata, its dependencies, and their relationships
  (§4.2). This is exactly the **dependency-graph RAG** pattern
  ([DepsRAG](https://github.com/Mohannadcse/DepsRAG)) and the
  **agentic-crawling → graph-construction** pipeline
  ([arXiv 2604.14220](https://arxiv.org/html/2604.14220)). **Already in the
  spec** — §4.2 pins pre-graphing as a first-class pipeline stage.
- **§4.3.2 query modes (`graph` | `vector` | `hybrid`)** — the `hybrid` mode
  "returns a merged result combining graph and vector matches." This is
  **graph-enhanced vector search** (§1.1) and the **HybridRAG** pattern
  ([HybridRAG arXiv 2408.04948](https://arxiv.org/html/2408.04948)). **Already
  in the spec** — the hybrid mode is pinned; the *merge algorithm* (how graph
  and vector matches are combined/ranked) is **not** pinned and is a **proposed
  spec change** (RRF-style rank fusion or weighted combination, per the
  hybrid-search research).
- **§4.3.2 `graph` mode** — returns the pre-graphed subgraph for a target. For
  relationship and multi-hop questions ("what depends on package X?", "which
  packages share dependency Y?"), **dynamic graph query generation** (§1.2)
  would translate the natural-language query into a graph traversal
  ([Dynamic Cypher Generation](https://graphrag.com/reference/graphrag/dynamic-cypher-generation/),
  [Multi-Agent GraphRAG](https://arxiv.org/pdf/2511.08274)). **Proposed spec
  change** — the contract pins a subgraph return but not a natural-language→
  graph-query translation step.
- **§4.2 RAG store states / §4.1 crawl model** — **parent-child retrievers**
  (§1.3) map onto the crawl → raw-material → embedded-items hierarchy: a crawled
  page/package is the parent, its embedded chunks are the children
  ([Parent-Child Retriever](https://graphrag.com/reference/graphrag/parent-child-retriever/)).
  This gives Zodiac both precise child-level retrieval and parent context.
  **Proposed spec change** — the contract pins embedding per item but not a
  parent-child hierarchy.
- **§4.2 pre-graphing** — **community summaries** (§1.4) would let Zodiac
  precompute summaries of related dependency clusters (e.g. "the logging
  ecosystem"), enabling global/thematic queries over crawled data
  ([Microsoft GraphRAG](https://arxiv.org/pdf/2404.16130)). **Proposed spec
  change** — not in the contract; the cost/benefit must be weighed (community
  summarization is the expensive pass LightRAG avoids).
- **§4.2 pre-graphing** — **graph enrichments** (§1.5): entity resolution and
  deduplication of crawled entities (e.g. the same package referenced under
  different names/versions) improve graph quality before retrieval
  ([GraphER](https://arxiv.org/html/2603.24925v3),
  [KG-Infused RAG](https://arxiv.org/html/2506.09542)). **Proposed spec change**
  — the contract does not pin a dedup/enrichment stage.
- **§4.3.2 result shape / §4.3.3 `zodiac.query.reply`** — **explainability**
  (§1) is a first-class Graph RAG benefit: because Zodiac's results are
  pre-graphed, each result can carry a **traceable path** (source crawl id →
  node → edge) that answers governance/sourcing questions
  ([TrustGraph explainability](https://docs.trustgraph.ai/overview/explainability.html),
  [XGRAG](https://doi.org/10.48550/arxiv.2604.24623)). The result shape already
  carries `sourceCrawlId` (§4.3.2); a **proposed spec change** would add a
  trace/path field for explainability.
- **§4.5 MCP surface** — the provisional `zodiac_query` tool already accepts
  `mode: "graph"|"vector"|"hybrid"` (§4.5). Any new graph-query capability
  (dynamic graph query, community summary query) must be reachable through both
  `zodiac_query` and the GUI (D4). **Proposed spec change** — extend the `mode`
  enum or add a query parameter.

**Status summary for Zodiac:** the **core** (pre-graphing + vector-embedding +
hybrid mode) is **already in the spec** (§4.2, §4.3.2). The **advanced Graph RAG
techniques** (dynamic graph query generation, parent-child retrievers,
community summaries, graph enrichments, explainability traces) are **proposed
spec changes** — none are currently pinned in `zodiac.md`.

---

## §4 How it applies to the other suite consumers

- **Incanter** (`incanter.md`) — Incanter is already a **graph-RAG engine**
  (document graph + spring-tension dynamic chunking + hybrid graph-tension +
  vector query, §4.2/§4.8). Graph RAG techniques apply as **enhancements**:
  graph-enhanced vector search and dynamic graph query generation over the
  document graph would strengthen Incanter's `POST /v1/query` (§4.8). These are
  **proposed spec changes** to Incanter's contract (and, per AGENTS.md, handoff
  candidates to the Incanter repo — this repo never patches a tool's source).
- **Familiar** (`familiar.md`) — Familiar's **research** domain routes through
  Astrographer's RAG surface, which may return Zodiac results (§4.3.3, F4).
  Graph RAG gives Familiar **relationship-aware** research answers (e.g. "what
  depends on this package?") that flat vector RAG cannot. No contract change to
  Familiar itself — the value arrives through Astrographer/Zodiac. **Already
  covered** by the F4 routing decision.
- **Solomon** (`solomon.md`) — Solomon links and searches across Zodiac
  instances (§5.1, edge #7). Because Zodiac's data is **pre-graphed**, Solomon's
  cross-instance search can be **relationship-aware** (search returns not just
  matching items but their graph neighborhoods). This is a **proposed spec
  change** to Solomon's aggregation semantics (§4.2.2) — the Phase-1 contract
  pins grouping by source peer but not graph-aware result expansion.
- **Mystery** (`mystery.md` — **spec absent**, PLANNED per
  `architecture-overview.md` §4.3) — Mystery crosslinks information references
  by post topic. A topic→reference resolution is a graph-like structure; Graph
  RAG (graph-enhanced retrieval over the crosslink graph) is a natural fit once
  the Mystery contract exists. **Parked** — no spec to map to yet.
- **Astral** (`astral.md`) — Astral hosts wiki pages pushed as Provident graphs
  (§4.2). Graph RAG could enhance **search over hosted pages** (graph-aware
  retrieval over the hosted-unit graph) and is relevant to Solomon's
  cross-instance search over Astral (§5.4, edge #8). **Proposed spec change** —
  the Astral contract pins hosting/management but not a graph-aware search
  surface.

---

## §5 Recommendation for the suite

Grounded in the four design constraints (D1 open-source AGPL-3.0, D2
local-first, D3 interconnection, D4 MCP-GUI parity).

**Overall: MUST** — Graph RAG is the core of Zodiac's design and a first-class
capability of Astrographer. The techniques are open algorithms (D1-compatible),
fully local-first implementable (D2-compatible — LightRAG and local-Ollama
GraphRAG are proof), and directly serve the interconnection value (D3 — Zodiac's
relationship data is more valuable to Astrographer/Familiar/Solomon than flat
vector data). Every graph-query capability must be reachable through both GUI
and MCP (D4).

Per-technique tiering:

| Technique | Tier | Where it lands | Status |
| --- | --- | --- | --- |
| **Pre-graphing / KG construction from web crawl** | **MUST** | Zodiac §4.2 | Already in spec |
| **Graph-enhanced vector search (hybrid mode)** | **MUST** | Zodiac §4.3.2; Astrographer §4.3 | Already in spec (merge algorithm unpinned) |
| **Parent-child retrievers** | **SHOULD** | Zodiac §4.2 (crawl → chunk hierarchy) | Proposed spec change |
| **Dynamic graph query generation** | **SHOULD** | Zodiac §4.3.2 `graph` mode | Proposed spec change |
| **Graph enrichments (entity resolution/dedup)** | **SHOULD** | Zodiac §4.2 pre-graphing | Proposed spec change |
| **Explainability traces (source path)** | **SHOULD** | Zodiac §4.3.2 result shape | Proposed spec change |
| **Community summaries** | **NICE-TO-HAVE** | Zodiac §4.2 (dependency-ecosystem summaries) | Proposed spec change; weigh cost (LightRAG avoids it) |
| **Graph-aware cross-instance search** | **SHOULD** | Solomon §4.2.2 (edge #7/#8) | Proposed spec change |
| **Graph-aware search over hosted pages** | **NICE-TO-HAVE** | Astral §4.3 | Proposed spec change |

**Rationale for MUST (not SHOULD):** Zodiac's contract already *requires* a
graphical RAG engine that pre-graphs and vector-embeds crawled data (§4.2) and
answers queries in `graph`/`vector`/`hybrid` modes (§4.3.2). Graph RAG is not an
optional enhancement to Zodiac — it is the mechanism by which Zodiac delivers
its stated value. The MUST tier is therefore a confirmation that the existing
spec is on the right track, plus a directive to pin the unpinned merge/ranking
and graph-query semantics. The advanced techniques (dynamic graph query,
parent-child, enrichments, explainability) are SHOULD — high value, low
licensing/local-first risk, but not prerequisites for Zodiac's core function.

**Why not DISCARD:** the relationship data, multi-hop capability, and
explainability that Graph RAG provides are precisely the failures of pure vector
RAG that the source talk names (vectors don't capture relationships; similarity
≠ relevance; lack of explainability). Discarding Graph RAG would leave Zodiac
as a flat vector store, which contradicts its own contract.

**D4 note:** every graph-query capability (hybrid mode, dynamic graph query,
community summary query, explainability trace) must be exposed through both the
GUI and the MCP surface (`zodiac_query`, `rag_query`/`rag_stream`). A graph
feature reachable through only one surface is a D4 violation and a review
finding.

---

## §6 Source URL list

**Graph RAG foundations**
- https://arxiv.org/pdf/2404.16130 (Microsoft GraphRAG — From Local to Global)
- https://www.microsoft.com/en-us/research/blog/graphrag-new-tool-for-complex-data-discovery-now-on-github/
- https://github.com/microsoft/graphrag/blob/main/docs/index.md
- https://arxiv.org/html/2410.05779v1 (LightRAG)
- https://aclanthology.org/2025.findings-emnlp.568/ (LightRAG, ACL Anthology)
- https://neo4j.com/blog/developer/under-the-covers-with-lightrag-retrieval/
- https://arxiv.org/html/2408.04948 (HybridRAG)
- https://arxiv.org/pdf/2507.03226 (Towards Practical GraphRAG)

**Retrieval techniques**
- https://graphrag.com/reference/graphrag/graph-enhanced-vector-search/
- https://graphrag.com/reference/graphrag/parent-child-retriever/
- https://graphrag.com/reference/graphrag/dynamic-cypher-generation/
- https://arxiv.org/pdf/2511.08274 (Multi-Agent GraphRAG: Text-to-Cypher)
- https://neo4j.com/blog/developer/graphrag-field-guide-rag-patterns/
- https://arxiv.org/html/2603.24925v3 (GraphER — graph enrichment + reranking)
- https://arxiv.org/html/2506.09542 (KG-Infused RAG)

**Web-crawl → knowledge graph / dependency RAG**
- https://arxiv.org/html/2604.14220 (Knowledge Graph RAG: Agentic Crawling)
- https://knowledgesdk.com/blog/graphrag-web-scraping
- https://knowledgesdk.com/blog/knowledge-graph-from-website
- https://github.com/Mohannadcse/DepsRAG (dependency-graph RAG)

**Explainability / similarity-vs-relevance**
- https://docs.trustgraph.ai/overview/explainability.html
- https://doi.org/10.48550/arxiv.2604.24623 (XGRAG)
- https://dzone.com/articles/when-similarity-isnt-accuracy-genai-vector-rag-vs-graphrag
- https://redis.io/en/blog/knowledge-graph-rag-structured-retrieval-ai-agents/

**Local-first (D2) feasibility**
- https://github.com/eastsea17/GraphRAG_with_Ollama
- https://github.com/nonatofabio/local-graphrag-mcp
