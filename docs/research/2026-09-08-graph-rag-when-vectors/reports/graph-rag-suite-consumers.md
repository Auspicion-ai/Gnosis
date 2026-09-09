# Graph RAG — Other Suite Consumers (Incanter, Familiar, Solomon, Mystery, Astral)

- **Topic:** Graph RAG — RAG whose retrieval path also includes a knowledge graph (nodes, edges, properties), used alongside the vector layer — as it applies to the suite consumers beyond the two flagship engines.
- **Date:** 2026-09-08
- **Tier:** SHOULD
- **Report slug:** `graph-rag-suite-consumers`
- **Primary focus:** Astrographer, Zodiac. **Secondary (the distinguishing content):** Incanter, Familiar, Solomon, Mystery, Astral.
- **Status:** Web-grounded research report (documentation deliverable only — no spec/code/test changes). Companion to `graph-rag-astrographer.md` (tier MUST) in the same directory; this report covers the cross-cutting consumer picture and the non-flagship consumers in depth.

---

## §1 What the technique is (web-grounded, cited)

**Graph RAG** is a family of retrieval-augmented-generation designs in which the
retrieval path includes a **knowledge graph** (nodes, edges, properties) in
addition to (or instead of) a flat vector index. The source talk frames it as:
*"RAG where retrieval path also includes a knowledge graph — use both vector and
graph layer methods."* The motivating failures of pure vector RAG are:

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
- *Graph Retrieval-Augmented Generation: A Survey*
  ([arXiv:2408.08921](https://arxiv.org/html/2408.08921v2)) — a systematic
  catalog of the Graph RAG family and its techniques.
- *Graph-based Approaches and Functionalities in RAG: A Comprehensive Survey*
  ([arXiv:2504.10499](https://arxiv.org/pdf/2504.10499)) — the graph-based RAG
  taxonomy (graph construction, retrieval, generation).
- *In-depth Analysis of Graph-based RAG in a Unified Framework*
  ([VLDB 2025](https://www.vldb.org/pvldb/vol18/p5623-zhou.pdf)) — a unified
  framework analysis of graph-based RAG.
- Neo4j's *What is GraphRAG?* ([neo4j.com](https://neo4j.com/blog/genai/what-is-graphrag/))
  and *GraphRAG Field Guide* ([medium.com/neo4j](https://medium.com/neo4j/graphrag-field-guide-navigating-the-world-of-advanced-rag-patterns-123d847a2837)) —
  a taxonomy of RAG patterns including graph-enhanced vector search, parent-child
  retrievers, and community summaries.
- Cognee's *What Is GraphRAG?* ([cognee.ai](https://www.cognee.ai/what-is-graphrag)) —
  a practitioner framing of GraphRAG with knowledge graphs.
- *Graph RAG in the Wild: Insights and Best Practices*
  ([Semantic Web Journal](https://www.semantic-web-journal.net/system/files/swj4027.pdf)) —
  production insights and best practices for Graph RAG.
- *Dissecting GraphRAG: A Modular Analysis of Knowledge Structuring for Factoid
  QA* ([TACL 2026](https://aclanthology.org/2026.tacl-1.29.pdf)) — a modular
  analysis of what GraphRAG's graph structuring actually buys.
- *HYBGRAG: Hybrid Retrieval-Augmented Generation on Textual and Relational
  Knowledge Bases* ([ACL 2025](https://aclanthology.org/2025.acl-long.43.pdf)) —
  hybrid retrieval over textual + relational (graph) knowledge bases.
- *HopRAG: Multi-Hop Reasoning for Logic-Aware RAG*
  ([arXiv:2502.12442](https://arxiv.org/pdf/2502.12442)) — multi-hop reasoning
  over a knowledge graph.
- *Youtu-GraphRAG: Vertically Unified Agents for Graph Retrieval-Augmented
  Complex Reasoning* ([arXiv:2508.19855](https://arxiv.org/html/2508.19855v1)) —
  agentic graph RAG for complex reasoning.
- *GraphSearch: An Agentic Deep Searching Workflow for Graph RAG*
  ([arXiv:2509.22009](https://arxiv.org/html/2509.22009)) — agentic, iterative
  graph search.
- *GraphRAG local with Ollama* ([github.com/TheAiSingularity/graphrag-local-ollama](https://github.com/TheAiSingularity/graphrag-local-ollama/)) —
  a fully local, self-hosted GraphRAG deployment (relevant to D2).
- *graphrag-rs* ([github.com/automataia/graphrag-rs](https://github.com/automataia/graphrag-rs)) —
  a Rust GraphRAG implementation (relevant to the suite's non-JS engine direction).
- *Flexible GraphRAG* ([stevereiner.github.io/flexible-graphrag](https://stevereiner.github.io/flexible-graphrag/)) —
  a configurable, local-first GraphRAG framework.

---

## §2 How it applies to Astrographer

Astrographer is already a **graphical RAG engine + document store/wiki** over
**Provident graphs** (`docs/specs/astrographer.md` §2, §4.1.1). Its data model is,
in effect, a knowledge graph with properties:

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

**Relationship data / multi-hop.** The `reference`→`fact` graph (§4.2) is exactly
the "relationship data" the source talk says vectors miss. Filtering by
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
are PARKED (cost, D2, overlap with parked DAG-RAG items). See the companion
report `graph-rag-astrographer.md` for the full Astrographer analysis.

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
Community summaries stay PARKED. See the companion report `graph-rag-astrographer.md`
for the full Zodiac analysis.

---

## §4 How it applies to the other suite consumers

This is the distinguishing content of this report: the non-flagship consumers
(Incanter, Familiar, Solomon, Mystery, Astral) and how Graph RAG touches each.

### 4.1 Incanter (`docs/specs/incanter.md`)

Incanter is the **prototype Graph-RAG engine** Astrographer consumes over HTTP
(`incanter.md` §2, §5; decision INCANTER-DISPOSITION). It is already graph-native:

- **Document graph** (`incanter.md` §4.4): each document is a directed topological
  multigraph of token transitions — nodes (tokens) + edges (co-occurrence
  weights). This is a graph substrate, not a flat bag-of-words.
- **Graph-tension retrieval** (`incanter.md` §4.6, §4.8): the hybrid query fuses
  **graph-tension proximity + dense vector similarity** (`vector_weight`/`graph_weight`,
  §4.8). This is **graph-enhanced vector search** — the graph layer is native.
- **Multi-hop / relationship data** (`incanter.md` §4.4): the graph's edges carry
  co-occurrence relationships; graph traversal is the relationship-aware signal.

**Graph RAG techniques:**
- **Graph-enhanced vector search** — already the engine's design (`incanter.md`
  §4.8). **Already in spec.**
- **Dynamic graph query generation** — Incanter's `POST /v1/query` (§4.8) is a
  single hybrid call; generating query variants scoped to graph regions is the
  multi-query direction (GAP-1, `docs/defects.md`). **Proposed spec change /
  handoff to Incanter.**
- **Community summaries** — not in Incanter's contract; the tension-space model
  (§4.6) is a per-document structure, not a corpus-level community graph.
  **PARKED.**
- **Graph enrichments** — Incanter's graph is built from ingested documents
  (§4.3); enrichment from external data is out of scope (that is Zodiac's role).
  **Not applicable.**

**Net for Incanter:** the graph-enhanced axis is already in spec. The concrete
gaps are the **lexical (BM25) leg** (GAP-2, parked D5) and the **multi-query
fan-out** (GAP-1) — both are engine-boundary handoffs, not new Graph RAG
techniques. Incanter is the suite's proof that the graph layer is native to the
RAG engine, and its Rust implementation aligns with the local-first, non-JS
production-engine direction (D2, INCANTER-DISPOSITION).

### 4.2 Familiar (`docs/specs/familiar.md`)

Familiar is the **"smart assistant" agent harness** (PLANNED). It is a **consumer
of Graph RAG, not a Graph RAG engine**:

- **Knowledge domain** (`familiar.md` §4.3.1, F1): delegates to Astrographer's
  RAG/document store via MCP (`rag_query`, `create_document`, etc.). Graph RAG
  reaches Familiar through Astrographer's `ragQuery` surface.
- **Research domain** (`familiar.md` §4.3.3, F4): routes **through Astrographer's
  RAG surface**, which may return Zodiac results (`source: 'zodiac'`). No direct
  Familiar→Zodiac edge. Graph RAG reaches Familiar through the Astrographer
  surface.
- **Memory store** (`familiar.md` §4.1.3): a **facts table** — individually
  addressable fact records, **not a graph**. The parked agent-memory vector-search
  layer (`docs/pending.md` D6) is a vector, not a graph, addition.

**Graph RAG techniques:**
- **Graph-enhanced vector search** — inherited indirectly through Astrographer's
  `ragQuery` (`familiar.md` §4.3.1). **Already in spec (via Astrographer).**
- **Relationship data / multi-hop** — Familiar's orchestration planning (§4.2)
  could use the graph structure of Astrographer's reference graph to answer
  multi-hop knowledge questions, but this is downstream of Astrographer's RAG
  surface. **No direct work in Familiar.**
- **Community summaries / dynamic graph query generation** — not applicable to
  Familiar's facts-table memory. **PARKED.**

**Net for Familiar:** no direct Graph RAG work. Familiar inherits the graph layer
through the Astrographer/Zodiac surfaces it consumes (F1, F4). Its own memory is a
facts table, and the parked D6 vector-search layer is orthogonal to graph RAG.

### 4.3 Solomon (`docs/specs/solomon.md`)

Solomon is the **decentralized peer-network search** layer (IN DEVELOPMENT). It
links and searches across `zodiac|astral|astrographer` instances (`solomon.md`
§4.2, §4.5.1). It is a **search fan-out/aggregation layer over Graph RAG
instances**, not a Graph RAG engine itself:

- **Cross-instance search** (`solomon.md` §4.2): fans a query out to reachable
  peers and aggregates results. The instances it searches (Zodiac, Astral,
  Astrographer) are graph RAG engines or graph hosts.
- **Result aggregation** (`solomon.md` §4.2.2): flat — grouped by source peer,
  ordered by the peer's own relevance ranking. Cross-peer ordering is deliberately
  unpinned in Phase-1 (`solomon.md` §7.4).

**Graph RAG techniques:**
- **Graph-enhanced vector search** — Solomon's `solomon_search` (§4.5.1) returns
  `{peerId, instanceType, itemId, snippet}`; the graph-aware ranking happens
  inside each peer, not in Solomon. **Already in spec (delegated to peers).**
- **Relationship data / multi-hop** — a cross-instance query could traverse
  relationships across instances (e.g. a fact in one Astrographer instance
  referenced in another). This is a **graph-aware cross-instance ranking**
  enhancement. **NICE-TO-HAVE / PARKED** — defer until Phase-1 search semantics
  finalize (`solomon.md` §7.4).
- **Community summaries / dynamic graph query generation** — not applicable to
  Solomon's fan-out model. **PARKED.**

**Net for Solomon:** Graph RAG applies to Solomon only as a **cross-instance
ranking** enhancement over the graph RAG instances it searches. Phase-1 keeps
cross-peer ordering unpinned, so this is PARKED until the search semantics
finalize. The `instanceTypes` enum (`zodiac|astral|astrographer`, §4.2/§4.5.1)
already reflects that the searchable instances are graph RAG engines.

### 4.4 Mystery (`docs/specs/mystery.md` — **absent**; `docs/pending.md` #12)

Mystery is a **message board application that crosslinks information references
from other apps by post topic** (`docs/architecture-overview.md` §4.3). Its spec
does not exist yet. Graph RAG is directly relevant to its defining feature:

- **Topic→reference resolution** (`docs/pending.md` #12): how Mystery maps a post
  topic to an information reference (Astrographer documentation, Astral pages).
  This is a **relationship-mapping problem** — exactly what a knowledge graph
  (Astrographer's reference graph, `astrographer.md` §4.2) serves.
- **Crosslink model** (`docs/architecture-overview.md` §4.3): Mystery crosslinks
  references by post topic (e.g. linking relevant web-app documentation during a
  debug thread). Modeling these crosslinks as **graph edges over Astrographer's
  reference graph** is the natural Graph RAG application.

**Graph RAG techniques:**
- **Graph-enhanced vector search** — Mystery's crosslink resolution could use
  Astrographer's reference graph to find related documentation by relationship,
  not just by topic similarity. **Proposed when the spec exists.**
- **Relationship data / multi-hop** — a debug thread linking documentation across
  topics is a multi-hop relationship query. **Proposed when the spec exists.**

**Net for Mystery:** when the Mystery contract is written, model its crosslinks
as graph edges over Astrographer's reference graph. **PARKED until the spec
exists** (F3 in Familiar is likewise deferred on the absent Mystery spec,
`docs/decisions.md` FAMILIAR-DEFERRED-EDGES).

### 4.5 Astral (`docs/specs/astral.md`)

Astral is the **locally-runnable webhost / wiki-publishing host** (PLANNED). It
receives Provident graphs (`astral.md` §4.2) and serves them (`astral.md` §4.1).
Graph RAG applies to Astral only as a **serving** concern:

- **Hosted units are graphs** (`astral.md` §4.1): each hosted unit is a Provident
  graph rendered by Provident-SSR. Astral serves the graphs Astrographer
  produces.
- **No retrieval engine** (`astral.md` §2): Astral is a host, not a RAG engine.
  It has no query surface of its own; retrieval happens in Astrographer/Zodiac/
  Incanter.

**Graph RAG techniques:**
- **Graph-enhanced vector search** — not applicable; Astral has no retrieval
  surface. **Not applicable.**
- **Graph enrichments** — Astral's hosted content is the graph; enrichment happens
  upstream (Zodiac crawls, Astrographer authors). **Not applicable.**
- **Agent-facing reachability** — the relevant Astral concern is the docs-as-product
  reachability surface (GAP-5, `docs/defects.md`): `llms.txt`/`llms-full.txt` +
  markdown-copyable pages so agents can read the hosted graphs. This is a serving
  concern, not graph retrieval. **Proposed spec change (GAP-5).**

**Net for Astral:** no direct Graph RAG work. Astral serves the graphs
Astrographer produces; its relevant enhancement is the agent-facing reachability
surface (GAP-5), which is a docs-as-product concern, not a graph-retrieval one.

---

## §5 Recommendation for the suite

Grounded in D1–D4. The suite is already **graph-native**: Astrographer, Zodiac,
and Incanter are all graphical RAG engines over knowledge-graph-shaped data. The
recommendation is therefore not "adopt Graph RAG" but "complete and expose the
graph layer the suite already has, and keep the expensive global techniques
parked."

- **MUST — Graph-enhanced retrieval over Astrographer's reference graph
  (topological resolution of `reference`→`fact`).** Astrographer's data model is
  already a knowledge graph (`astrographer.md` §4.2). Resolving `reference`→`fact`
  in dependency order is the core, low-risk win and directly enables
  relationship-aware, multi-hop answers. Consistent with the DAG-RAG research's
  MUST-HAVE (`docs/research/dag-rag-research-notes.md` §4), D2 (runs locally over
  the Provident graph), and D4 (exposable as MCP + GUI). **Proposed spec change**
  to `astrographer.md` §4.3.

- **SHOULD — Explainability / trace surface on RAG results (Astrographer +
  Zodiac).** Extend the `ragQuery` result (`astrographer.md` §4.3.2) with a
  **trace path** (the edges walked to reach the answer), reusing the provenance
  already tracked by the consistency model (`astrographer.md` §4.2.3); extend
  Zodiac's results (`zodiac.md` §4.3.2) with a trace path to `sourceCrawlId`. This
  directly answers the source talk's governance/sourcing point and is cheap
  because the graph already records every reference. **Proposed spec change.**

- **SHOULD — Complete the engine-boundary gaps in Incanter (lexical leg + multi-query
  fan-out).** Incanter's graph-enhanced hybrid (`incanter.md` §4.8) is already in
  spec; the concrete gaps are the **BM25 lexical leg** (GAP-2, parked D5) and the
  **multi-query fan-out** (GAP-1). Both are Incanter-repo handoffs
  (`docs/defects.md`), not new Graph RAG techniques. **Handoff to Incanter.**

- **NICE-TO-HAVE — Parent-child retrievers in Astrographer.** Return a node plus
  its containing document/section for context (`astrographer.md` §4.1.1). Low
  cost; improves answer context. **Proposed spec change.**

- **NICE-TO-HAVE / PARKED — Graph-aware cross-instance ranking in Solomon.** A
  cross-instance query could traverse relationships across the graph RAG instances
  it searches (`solomon.md` §4.2.2). Defer until Phase-1 search semantics finalize
  (`solomon.md` §7.4). **PARKED.**

- **PARKED — Community summaries (Microsoft GraphRAG global search).** Precomputing
  hierarchical community summaries over the wiki reference graph is expensive and
  conflicts with D2 local-first cost/latency; the DAG-RAG research already
  DISCARDs pre-built global graph construction (`docs/research/dag-rag-research-notes.md`
  §4). Revisit only if a global/theme question use case surfaces.

- **PARKED — Dynamic graph query generation.** Generating a sub-task DAG or graph
  query at query time overlaps with the parked DAG-RAG SHOULD-HAVE and is the
  most complex phase (multiple LLM calls per query). Land it after the
  MUST/SHOULD retrieval foundation ships.

- **PARKED — Mystery crosslink graph model.** When the Mystery contract is
  written, model its crosslinks as graph edges over Astrographer's reference
  graph (`docs/pending.md` #12). PARKED until the spec exists.

**Overall recommendation tier: SHOULD.** The graph RAG substrate is already
present across the suite (Astrographer, Zodiac, Incanter are graph-native), so
the MUST work is completing graph-enhanced retrieval over the reference graph.
The SHOULD work is the explainability/trace surface and the Incanter engine
gaps. The expensive global techniques (community summaries, dynamic graph query
generation) and the deferred consumers (Solomon ranking, Mystery crosslinks)
stay PARKED for D2 cost reasons and spec-absence reasons. This is consistent with
D1 (Graph RAG is a well-known algorithm with open-source, local implementations —
`graphrag-local-ollama`, `graphrag-rs`), D2 (the suite's graph RAG already runs
locally via Incanter + Ollama), D3 (the graph layer is the interconnection
substrate — Astrographer/Zodiac/Incanter share graph data, Solomon searches
across them), and D4 (any new graph RAG feature must be exposed on both GUI and
MCP).

---

## §6 Source URL list

1. https://arxiv.org/pdf/2404.16130 — Microsoft, *From Local to Global: A GraphRAG Approach to Query-Focused Summarization*
2. https://arxiv.org/html/2408.08921v2 — *Graph Retrieval-Augmented Generation: A Survey*
3. https://arxiv.org/pdf/2504.10499 — *Graph-based Approaches and Functionalities in RAG: A Comprehensive Survey*
4. https://www.vldb.org/pvldb/vol18/p5623-zhou.pdf — *In-depth Analysis of Graph-based RAG in a Unified Framework* (VLDB 2025)
5. https://neo4j.com/blog/genai/what-is-graphrag/ — Neo4j, *What is GraphRAG?*
6. https://www.cognee.ai/what-is-graphrag — Cognee, *What Is GraphRAG?*
7. https://microsoft.github.io/graphrag/ — Microsoft GraphRAG official docs
8. https://github.com/microsoft/graphrag — Microsoft GraphRAG repository
9. https://github.com/microsoft/graphrag/blob/main/docs/query/global_search.md — Microsoft GraphRAG, *global search* (community summaries)
10. https://medium.com/neo4j/graphrag-field-guide-navigating-the-world-of-advanced-rag-patterns-123d847a2837 — Neo4j, *GraphRAG Field Guide*
11. https://arxiv.org/html/2508.19855v1 — *Youtu-GraphRAG: Vertically Unified Agents for Graph Retrieval-Augmented Complex Reasoning*
12. https://arxiv.org/html/2509.22009 — *GraphSearch: An Agentic Deep Searching Workflow for Graph RAG*
13. https://aclanthology.org/2025.acl-long.43.pdf — *HYBGRAG: Hybrid Retrieval-Augmented Generation on Textual and Relational Knowledge Bases* (ACL 2025)
14. https://arxiv.org/pdf/2502.12442 — *HopRAG: Multi-Hop Reasoning for Logic-Aware RAG*
15. https://github.com/TheAiSingularity/graphrag-local-ollama/ — *GraphRAG local with Ollama* (local-first deployment)
16. https://www.semantic-web-journal.net/system/files/swj4027.pdf — *Graph RAG in the Wild: Insights and Best Practices* (Semantic Web Journal)
17. https://aclanthology.org/2026.tacl-1.29.pdf — *Dissecting GraphRAG: A Modular Analysis of Knowledge Structuring for Factoid QA* (TACL 2026)
18. https://github.com/automataia/graphrag-rs — *graphrag-rs* (Rust GraphRAG)
19. https://stevereiner.github.io/flexible-graphrag/ — *Flexible GraphRAG* (configurable, local-first)
