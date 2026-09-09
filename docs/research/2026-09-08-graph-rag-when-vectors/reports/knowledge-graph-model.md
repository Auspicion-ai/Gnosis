# Knowledge Graph Model — noun-focused nodes, relationship-focused edges, properties on both

- **Topic:** The knowledge-graph model — **nodes** (noun-focused entities), **edges** (relationship-focused links), and **properties** (attributes attachable to both nodes and edges) — as the substrate for graph RAG.
- **Date:** 2026-09-08
- **Tier:** MUST
- **Report slug:** `knowledge-graph-model`
- **Primary focus:** Astrographer, Zodiac. **Secondary:** Incanter, Familiar, Solomon, Mystery, Astral.
- **Status:** Web-grounded research report (documentation deliverable only — no spec/code/test/tracker changes).

---

## §1 What the technique is (web-grounded, cited)

The source talk defines the knowledge-graph model in three parts:

> **Nodes** — "Noun"-focused. **Edges** — Relationship focused. **Properties** — Can be attached to nodes and edges.

This is the **labeled property graph (LPG)** model, the standard data model of
graph databases and of knowledge-graph-augmented RAG. In an LPG, a graph is a
set of **nodes** (entities, typically nouns), a set of **relationships/edges**
(typed links between nodes, typically verbs/relations), and **properties**
(key–value attributes that can be attached to either a node or a relationship)
([Neo4j, *Graph database concepts*](https://neo4j.com/docs/getting-started/appendix/graphdb-concepts/);
[Neo4j, *What is graph data modeling?*](https://neo4j.com/docs/getting-started/data-modeling/)).
The same three-part model is what a **property-graph index** exposes to a RAG
retriever — nodes, relationships, and properties are the retrieval units
([LlamaIndex, *Property Graph Index*](https://deepwiki.com/run-llama/llama_index/10.1-property-graph-index)).

The model is the foundation of graph RAG: retrieval paths that include a
knowledge graph alongside (or instead of) a flat vector index. The motivating
failures of pure vector RAG are exactly the ones the talk names — vectors do
not capture **relationship data** (a graph supplies the relational context that
enables follow-up and multi-hop questions), **vector similarity ≠ relevance**
(a frequently-appearing token can be embedded close to a query without being
the answer), and vector RAG **lacks explainability** (governance questions are
more easily answered by sourcing and tracing through an explicit graph)
([Meilisearch, *Knowledge graph vs. vector database for RAG*](https://www.meilisearch.com/blog/knowledge-graph-vs-vector-database-for-rag);
[Tian Pan, *GraphRAG vs. Vector RAG: When Knowledge Graphs Beat Embeddings*](https://tianpan.co/blog/2026-04-17-graphrag-vs-vector-rag-knowledge-graphs);
[Neo4j, *What is GraphRAG?*](https://neo4j.com/blog/genai/what-is-graphrag/)).

Two things make the model concrete for a RAG system:

1. **How the graph is built (entity/relation extraction).** A knowledge graph
   is constructed from unstructured text by extracting **entities** (the noun
   nodes) and **relations** (the edges between them), then attaching the
   extracted attributes as **properties**. This is the standard LLM-driven
   KG-construction pipeline ([Neo4j, *How to convert unstructured text to knowledge graphs using LLMs*](https://neo4j.com/blog/developer/unstructured-text-to-knowledge-graph/);
   [Neo4j GraphRAG Python, *Knowledge Graph Builder*](https://neo4j.com/docs/neo4j-graphrag-python/current/user_guide_kg_builder.html);
   [RAKG: Document-level Retrieval Augmented Knowledge Graph Construction](https://arxiv.org/pdf/2504.09823v1);
   [RAGA: Reading-And-Graph-building-Agent for Autonomous KG Construction and RAG](https://arxiv.org/html/2605.17072);
   [Efficient Knowledge Graph Construction and Retrieval from Unstructured Text for Large-Scale RAG Systems](https://arxiv.org/html/2507.03226v2)).
2. **How the graph is queried (graph-aware retrieval).** Once the graph exists,
   retrieval can walk nodes/edges/properties — filtering by relationship,
   answering multi-step/multi-hop questions, and tracing a result back to its
   source. The labeled-property-graph model is the substrate for these
   relationship-aware, multi-hop retrievals ([Graphs RAG at Scale: Labeled Property Graphs and RDF](https://arxiv.org/abs/2603.22340v1);
   [StepChain GraphRAG: Reasoning Over Knowledge Graphs for Multi-Hop QA](https://arxiv.org/pdf/2510.02827);
   [HopRAG: Multi-Hop Reasoning for Logic-Aware RAG](https://arxiv.org/pdf/2502.12442);
   [Enison, *Implementation Guide for Answering Complex Queries with Knowledge Graph × RAG*](https://enison.ai/en/blog/knowledge-graph-rag-implementation);
   [LazyLLM, *Advanced RAG: Knowledge-Graph-Based RAG*](https://docs.lazyllm.ai/en/v0.7.6/Tutorial/19/)).

**Provenance is a property.** The talk's "governance questions are more easily
answered by sourcing and tracing" is realized by attaching **provenance
properties** to nodes/edges — the source document, the crawl, the timestamp —
so a result can be traced back to its origin. This mirrors the W3C **PROV data
model**, where provenance is a graph of entities, activities, and agents with
attributes ([PROV-DM, *prov.readthedocs.io*](https://prov.readthedocs.io/en/3.1.0/explanation/prov-dm.html)).

**Net for the technique:** the knowledge-graph model is not a single algorithm
but the **data model** that graph RAG runs over. Its three parts — noun nodes,
relationship edges, properties on both — are the retrieval units, and the
concrete value is relationship-aware, multi-hop, traceable retrieval that pure
vector similarity cannot express.

---

## §2 How it applies to Astrographer

Astrographer is a **Graphical RAG engine + document store/wiki** whose documents
are **Provident graphs** (`docs/specs/astrographer.md` §2, §4.1.1). The
Provident graph **is** the knowledge-graph model the talk describes — the
three-part model is already the substrate, not a foreign addition:

- **Noun-focused nodes** (`astrographer.md` §4.2.1): a Provident graph node is
  one of `content` (authored text), `fact` (a single source-of-truth fact with a
  `factKey` and a canonical `value`), or `reference` (a pointer to a `fact`/node).
  The `fact` node is the noun-focused entity — the canonical, addressable fact.
- **Relationship-focused edges** (`astrographer.md` §4.2.2): a `reference` node
  carries a `target` and a `mode` — `link` (navigational, resolved live) or
  `embed` (a snapshot of the target's value). These are the relationship-focused
  edges from a referencing document to a target `fact`/node.
- **Properties on nodes and edges** (`astrographer.md` §4.1.1, §4.2.1, §4.2.2,
  §4.2.5): document metadata (`title`, `tags`, `author`, `revision`, `state`) and
  fact `factKey`/`value` are **node properties**; the `mode` (`link`/`embed`) and
  the `crossWiki` flag are **edge properties**.

**Status: already in the spec.** The knowledge-graph model is Astrographer's
native data model. The concrete work is not building the model but **retrieving
over it**:

- **Relationship data / filtering / multi-step / multi-hop** (`astrographer.md`
  §4.2): the `reference`→`fact` graph is exactly the "relationship data" the
  talk says vectors miss. Filtering by relationship ("all documents that embed
  fact X") and multi-step/multi-hop queries are natural graph operations over
  this structure. The DAG-RAG research already maps this to Astrographer:
  resolving a `reference` requires resolving its target `fact` first — a
  dependency-ordered topological walk (`docs/research/dag-rag-research-notes.md`
  §3, §4). **Proposed spec change** to `astrographer.md` §4.3 (a graph-aware
  retrieval mode over the reference graph).
- **Explainability / provenance as a property** (`astrographer.md` §4.2.3,
  §4.1.3): the consistency model already tracks every `link`/`embed` to its
  target and exposes `getConsistencyReport(wikiId)` (MCP `get_consistency_report`,
  §4.5.1). A RAG result could carry the same **trace path** (the edges walked to
  reach the answer) — provenance as a property on the result. The `ragQuery`
  result shape (`astrographer.md` §4.3.2) already returns
  `{documentId, nodeId, score, snippet, source}`; adding a trace path is a small,
  high-value extension. **Proposed spec change** to `astrographer.md` §4.3.
- **Maturity** (`astrographer.md` §4.1.4, §4.4): the Provident graph is a
  first-class, versioned graph model with optimistic concurrency and a canonical
  push format (`provident-graph/1`, decision GRAPH-PUSH-FORMAT) — a mature
  substrate relative to a raw vector store, which is the talk's "maturity" point.

**Net for Astrographer:** the knowledge-graph model is already the spec's data
model. The MUST/SHOULD work is graph-aware retrieval over the reference graph
(topological `reference`→`fact` resolution) plus a provenance/trace surface on
RAG results — both reusing the model that already exists.

---

## §3 How it applies to Zodiac

Zodiac is a **graphical RAG engine integrated with web crawlers** that replies to
Astrographer with **pre-graphed and vector-embedded data**
(`docs/specs/zodiac.md` §2, §4.2). Its **pre-graphing** step is the suite's
clearest realization of the knowledge-graph model built from unstructured data:

- **Pre-graphing** (`zodiac.md` §4.2): crawled raw material is parsed into a
  **graph (nodes and edges)** — e.g. a dependency package's metadata, its
  dependencies, and their relationships. This is **entity/relation extraction**
  from unstructured web data: the package and its dependencies are the noun
  nodes, the dependency relationships are the edges, and the metadata are the
  properties. This is the standard LLM-driven KG-construction pipeline
  ([Neo4j, *How to convert unstructured text to knowledge graphs using LLMs*](https://neo4j.com/blog/developer/unstructured-text-to-knowledge-graph/);
  [RAKG](https://arxiv.org/pdf/2504.09823v1);
  [RAGA](https://arxiv.org/html/2605.17072)).
- **Vector-embedding** (`zodiac.md` §4.2): the raw material and/or graph nodes
  are embedded into a vector store, each item carrying an `embedding id` and a
  `sourceCrawlId`. This is the "use both vector and graph layer methods" axis.
- **Query modes** (`zodiac.md` §4.3.1, §4.3.2): `graph` (returns the pre-graphed
  subgraph), `vector` (top-`limit` by similarity), and `hybrid` (merged graph +
  vector). The `graph`/`hybrid` modes are **graph-aware retrieval over the
  knowledge-graph model**.
- **Provenance as a property** (`zodiac.md` §4.2, §4.3.2): every embedded item
  and every query result carries `sourceCrawlId` (and `stale`), so a result can
  be traced back to the crawl that produced it — the talk's "sourcing and
  tracing" made concrete, and the PROV-style provenance-as-property pattern
  ([PROV-DM](https://prov.readthedocs.io/en/3.1.0/explanation/prov-dm.html)).
- **RAG store states** (`zodiac.md` §4.2): `EMPTY`/`READY`/`STALE`/`REFRESHING`
  — a mature store-state model (the talk's "maturity" point).

**Status: already in the spec.** Zodiac's pre-graphing + vector-embedding
pipeline is the knowledge-graph model built from crawled data, and its
`graph`/`hybrid` modes are graph-aware retrieval over it. The concrete additions
are explainability/trace (a trace path from a result to its source crawl) and,
optionally, dynamic graph query generation over the pre-graphed model.
**Proposed spec change** to `zodiac.md` §4.3 (SHOULD).

**Net for Zodiac:** the knowledge-graph model is the engine's design — it
pre-graphs crawled data into noun nodes, relationship edges, and properties, and
queries them in `graph`/`hybrid` mode. The concrete work is provenance/trace and
(optionally) dynamic graph query generation.

---

## §4 How it applies to the other suite consumers

- **Incanter** (`docs/specs/incanter.md`): Incanter models each document as a
  **directed topological multigraph of sequential token transitions**
  (`incanter.md` §4.4, INC-SPEC-002) — a graph, but at **token granularity**, not
  the noun-entity granularity of the knowledge-graph model. The knowledge-graph
  model (noun nodes / relationship edges / properties) is the **semantic** layer
  that Astrographer and Zodiac build; Incanter's graph is the **lexical/topology**
  layer underneath. **Recommendation:** no change — Incanter's token-transition
  graph is a different (complementary) graph model; the noun-node/relationship-
  edge model lives in Astrographer/Zodiac.

- **Familiar** (`docs/specs/familiar.md`): Familiar's memory store is a **facts
  table** (`familiar.md` §4.1.3) — individually addressable fact records, each
  with `content`, `provenance` (`{conversation_id, message_id}`), `confidence`,
  and `status`. This is a **flat facts table, not a graph**, but each fact
  already carries **provenance as a property** (`familiar.md` §4.1.3) — the
  talk's "sourcing and tracing" applied to assistant memory. The knowledge-graph
  model could inform a future relationship layer over facts, but the current
  contract is deliberately flat (facts as first-class objects, not a graph).
  **Recommendation:** no direct change; Familiar inherits the graph layer through
  the Astrographer/Zodiac surfaces it consumes (`familiar.md` §4.3.1, §4.3.3).

- **Solomon** (`docs/specs/solomon.md`): Solomon is cross-instance search over
  `zodiac|astral|astrographer` instances (`solomon.md` §4.2, §4.5.1). Its result
  aggregation (`solomon.md` §4.2.2) is flat — grouped by peer, ordered by the
  peer's own relevance. The knowledge-graph model applies at the **cross-instance
  level** (each instance's graph is a node in a larger graph), but Solomon's
  Phase-1 contract deliberately keeps cross-peer ordering unpinned
  (`solomon.md` §7.4). **Recommendation:** PARKED — defer graph-aware
  cross-instance ranking until Phase-1 search semantics finalize.

- **Mystery** (`docs/specs/mystery.md` — **absent**; `docs/pending.md` #12):
  Mystery is a message board that crosslinks information references by post
  topic (`docs/architecture-overview.md` §4.3). Its **topic→reference
  resolution** (`docs/pending.md` #12) is a relationship-mapping problem that
  the knowledge-graph model serves directly: a post **topic** is a noun node,
  and the crosslinked information references (Astrographer documentation, Astral
  pages) are **relationship edges** from the topic to the reference. **Recommendation:**
  when the Mystery contract is written, model its crosslinks as graph edges over
  Astrographer's reference graph. PARKED until the spec exists.

- **Astral** (`docs/specs/astral.md`): Astral is the wiki/publishing host
  (`astral.md` §2). It receives Provident graphs (`astral.md` §4.2) and serves
  them (`astral.md` §4.1). The knowledge-graph model applies to Astral only as a
  **serving** concern — the hosted pages are graphs, and the docs-as-product
  reachability surface (GAP-4/GAP-5, `docs/defects.md`) is about agent-facing
  markdown, not graph retrieval. **Recommendation:** no direct change; Astral
  serves the graphs Astrographer produces.

---

## §5 Recommendation for the suite

Grounded in the four design constraints (D1 open-source AGPL-3.0, D2 local-first,
D3 interconnection, D4 MCP-GUI parity):

- **MUST — Treat the knowledge-graph model as the suite's native substrate and
  keep it non-negotiable.** Astrographer's Provident graph (`astrographer.md`
  §4.2) and Zodiac's pre-graphing pipeline (`zodiac.md` §4.2) already implement
  the noun-node / relationship-edge / property model. This is the highest-value
  graph-RAG element and it is **already in the spec** — no change. It satisfies
  D2 (runs locally over the Provident graph), D3 (the graph is the interconnection
  seam between Astrographer, Zodiac, Astral, Emerald), and D4 (exposable as MCP +
  GUI). **Already in the spec.**

- **MUST — Graph-aware retrieval over Astrographer's reference graph
  (topological `reference`→`fact` resolution).** The model exists; the retrieval
  over it does not. Resolving `reference`→`fact` in dependency order is the core,
  low-risk win and directly enables relationship-aware, multi-hop answers. This
  is consistent with the DAG-RAG research's MUST-HAVE
  (`docs/research/dag-rag-research-notes.md` §4) and with D2/D4. **Proposed spec
  change** to `astrographer.md` §4.3.

- **SHOULD — Provenance / trace as a property on RAG results.** Extend the
  `ragQuery` result (`astrographer.md` §4.3.2) with a **trace path** (the edges
  walked to reach the answer), reusing the provenance the consistency model
  already tracks (`astrographer.md` §4.2.3). This directly answers the talk's
  governance/sourcing point and is cheap because the graph already records every
  reference. **Proposed spec change** to `astrographer.md` §4.3.

- **SHOULD — Zodiac provenance (trace to `sourceCrawlId`).** Zodiac results
  already carry `sourceCrawlId` and `stale` (`zodiac.md` §4.3.2); expose a trace
  path from a result to its source crawl. **Proposed spec change** to
  `zodiac.md` §4.3.

- **NICE-TO-HAVE — Dynamic graph query generation over the pre-graphed model.**
  Generating a graph query at query time (Zodiac `graph`/`hybrid` mode,
  `zodiac.md` §4.3.2) is the most complex phase (multiple LLM calls per query).
  Land it after the MUST/SHOULD retrieval foundation ships. **Proposed spec
  change** (NICE-TO-HAVE).

- **PARKED — Cross-instance graph-aware ranking (Solomon) and Mystery
  topic→reference edges.** Solomon's Phase-1 aggregation is deliberately flat
  (`solomon.md` §7.4); Mystery's contract does not exist yet (`docs/pending.md`
  #12). Both are PARKED until their search/crosslink semantics finalize.

- **DISCARD — None.** The knowledge-graph model conflicts with no D1–D4
  constraint. The "maturity not on par with existing database systems" concern
  is a caution, not a discard — it argues for the suite's local-first, well-
  specified Provident graph (with optimistic concurrency and a canonical push
  format) over ad-hoc vector systems.

**Overall recommendation tier: MUST.** The knowledge-graph model is not a foreign
technique for the suite — Astrographer and Zodiac are already built on it. The
concrete MUST/SHOULD work is graph-aware retrieval over the existing model plus a
provenance/trace surface; the expensive global techniques (dynamic graph query
generation, cross-instance graph ranking) stay PARKED for D2 cost reasons.

---

## §6 Source URL list

1. https://neo4j.com/docs/getting-started/appendix/graphdb-concepts/ — Neo4j, *Graph database concepts* (nodes, relationships, properties)
2. https://neo4j.com/docs/getting-started/data-modeling/ — Neo4j, *What is graph data modeling?*
3. https://deepwiki.com/run-llama/llama_index/10.1-property-graph-index — LlamaIndex, *Property Graph Index*
4. https://arxiv.org/abs/2603.22340v1 — *Graphs RAG at Scale: Labeled Property Graphs and RDF*
5. https://arxiv.org/html/2605.17072 — *RAGA: Reading-And-Graph-building-Agent for Autonomous KG Construction and RAG*
6. https://arxiv.org/pdf/2504.09823v1 — *RAKG: Document-level Retrieval Augmented Knowledge Graph Construction*
7. https://arxiv.org/html/2507.03226v2 — *Efficient Knowledge Graph Construction and Retrieval from Unstructured Text for Large-Scale RAG Systems*
8. https://neo4j.com/blog/developer/unstructured-text-to-knowledge-graph/ — Neo4j, *How to convert unstructured text to knowledge graphs using LLMs*
9. https://neo4j.com/docs/neo4j-graphrag-python/current/user_guide_kg_builder.html — Neo4j GraphRAG Python, *Knowledge Graph Builder*
10. https://enison.ai/en/blog/knowledge-graph-rag-implementation — Enison, *Implementation Guide for Answering Complex Queries with Knowledge Graph × RAG*
11. https://docs.lazyllm.ai/en/v0.7.6/Tutorial/19/ — LazyLLM, *Advanced RAG: Knowledge-Graph-Based RAG*
12. https://prov.readthedocs.io/en/3.1.0/explanation/prov-dm.html — W3C PROV data model (provenance as a graph)
13. https://arxiv.org/pdf/2510.02827 — *StepChain GraphRAG: Reasoning Over Knowledge Graphs for Multi-Hop QA*
14. https://arxiv.org/pdf/2502.12442 — *HopRAG: Multi-Hop Reasoning for Logic-Aware RAG*
15. https://www.meilisearch.com/blog/knowledge-graph-vs-vector-database-for-rag — Meilisearch, *Knowledge graph vs. vector database for RAG*
16. https://tianpan.co/blog/2026-04-17-graphrag-vs-vector-rag-knowledge-graphs — Tian Pan, *GraphRAG vs. Vector RAG: When Knowledge Graphs Beat Embeddings*

**Suite context cited throughout:** `docs/specs/astrographer.md` (§2, §4.1.1, §4.2.1, §4.2.2, §4.2.3, §4.2.5, §4.3, §4.4, §4.5.1), `docs/specs/zodiac.md` (§2, §4.2, §4.3.1, §4.3.2), `docs/specs/incanter.md` (§4.4), `docs/specs/familiar.md` (§4.1.3, §4.3.1, §4.3.3), `docs/specs/solomon.md` (§4.2, §4.5.1, §7.4), `docs/specs/astral.md` (§2, §4.1, §4.2), `docs/architecture-overview.md` (§4.2, §4.3, §5, §6), `docs/decisions.md` (D1–D4, GRAPH-PUSH-FORMAT), `docs/pending.md` (#12), `docs/defects.md` (GAP-4/GAP-5), `docs/research/dag-rag-research-notes.md` (§3, §4).
