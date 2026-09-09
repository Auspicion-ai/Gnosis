# Dynamic Graph Query Generation (LLM-generated graph queries)

- **Topic:** Dynamic graph query generation — an LLM translates a natural-language
  question into a structured graph query (Cypher / SPARQL / a custom graph query
  language) at query time, which is then executed against a knowledge graph to
  retrieve the relevant subgraph.
- **Date:** 2026-09-08
- **Tier:** SHOULD
- **Report slug:** `dynamic-graph-query-generation`
- **Primary focus:** Astrographer, Zodiac. **Secondary:** Incanter, Familiar,
  Solomon, Mystery, Astral.
- **Status:** Web-grounded research report (documentation deliverable only — no
  spec/code/test changes). Companion to `graph-rag-astrographer.md` in this
  directory, which frames the broad Graph RAG family and parks this technique
  until the retrieval foundation ships; this report is the dedicated deep-dive.

---

## §1 What the technique is (web-grounded, cited)

**Dynamic graph query generation** is the Graph RAG retrieval path in which an
LLM converts a user's natural-language question into a **structured graph query**
— most commonly **Cypher** (Neo4j, Kuzu, FalkorDB), **SPARQL** (RDF knowledge
graphs), or a custom graph query language — and that query is **executed against
a knowledge graph** to return the relevant subgraph, which is then fed to the
generator. It is one of the five Graph RAG techniques named in the source talk
("Dynamic graph query generation"), alongside graph-enhanced vector search,
parent-child retrievers, community summaries, and graph enrichments.

The defining property is **dynamism**: the query is generated **per question at
inference time**, not precomputed. This contrasts with the two other dominant
Graph RAG retrieval styles:

- **Community summaries** (Microsoft GraphRAG's *global search*): the graph is
  pre-partitioned into communities and each is summarized offline; a query
  selects the relevant community summaries. This is a **precomputed** path.
- **Graph-enhanced vector search**: retrieve by vector similarity, then expand
  or re-rank using the graph neighborhood. The graph is used as a **post-filter /
  expansion** on vector results.

Dynamic graph query generation instead treats the graph as a **first-class query
target**: the LLM writes a query that *traverses* the graph to answer the
question directly. The canonical, production-mature implementation is
**Text2Cypher** — the Neo4j GraphRAG Python package ships a `Text2CypherRetriever`
that takes a user question, grounds it against the graph schema, generates a
Cypher query, executes it, and returns the matched subgraph
([Neo4j GraphRAG RAG user guide](https://neo4j.com/docs/neo4j-graphrag-python/current/user_guide_rag.html),
[Text2CypherRetriever source](https://neo4j.com/docs/neo4j-graphrag-python/current/_modules/neo4j_graphrag/retrievers/text2cypher.html),
[Effortless RAG with Text2CypherRetriever](https://neo4j.com/blog/developer/effortless-rag-text2cypherretriever/)).
The same pattern is exposed by LangChain's `GraphCypherQAChain`
([reference](https://reference.langchain.com/python/langchain-neo4j/chains/graph_qa/cypher/GraphCypherQAChain),
[Neo4j Cypher integration](https://docs.langchain.com/oss/python/integrations/graphs/neo4j_cypher))
and LlamaIndex's `KnowledgeGraphQueryEngine` over a property-graph index
([LlamaIndex KG query engine](https://developers.llamaindex.ai/python/examples/query_engine/knowledge_graph_query_engine/),
[LlamaIndex property-graph index](https://developers.llamaindex.ai/python/framework/module_guides/indexing/lpg_index_guide/)).

**The pipeline (per user question):**
1. **Schema grounding.** The LLM is given the graph schema (node labels,
   relationship types, properties) so it can generate a *valid* query. This is
   the single most important input — without the schema, the LLM invents labels
   and the query fails or returns nothing.
2. **Query generation.** The LLM produces a structured query (Cypher/SPARQL)
   expressing the question as a graph traversal.
3. **Query validation / execution.** The query is executed against the graph DB.
   Invalid or empty-result queries are caught and **corrected** — either by
   re-prompting the LLM with the error, or by a reflection/self-correction loop.
4. **Result post-processing.** The matched subgraph (nodes + edges) is returned
   as the retrieval context for generation.

**Why it matters (the source talk's motivation).** The source talk's core
complaint about vector RAG is that *"vectors don't capture relationship data"*
and *"vector similarity ≠ relevance."* Dynamic graph query generation attacks
both directly: a graph query expresses **relationship data** (e.g. "all documents
that embed fact X") and **multi-hop questions** ("which published documents
reference a fact that is itself referenced by document Y?") as explicit
traversals, and it returns **exactly** the relevant subgraph rather than a
similarity-ranked approximation. It also serves the **explainability** goal: the
generated query *is* the trace — the answer can be traced back through the edges
the query walked.

**Key cited sources:**
- *Text2Cypher: Bridging Natural Language and Graph Databases*
  ([arXiv:2412.10064](https://arxiv.org/html/2412.10064),
  [ACL GenAIK 2025](https://aclanthology.org/2025.genaik-1.11.pdf)) — the
  canonical Text2Cypher paper; schema grounding + query generation + validation.
- Neo4j's `Text2CypherRetriever`
  ([user guide](https://neo4j.com/docs/neo4j-graphrag-python/current/user_guide_rag.html),
  [source](https://neo4j.com/docs/neo4j-graphrag-python/current/_modules/neo4j_graphrag/retrievers/text2cypher.html),
  [blog](https://neo4j.com/blog/developer/effortless-rag-text2cypherretriever/)) —
  the production-mature implementation.
- LangChain `GraphCypherQAChain`
  ([reference](https://reference.langchain.com/python/langchain-neo4j/chains/graph_qa/cypher/GraphCypherQAChain),
  [Neo4j integration](https://docs.langchain.com/oss/python/integrations/graphs/neo4j_cypher)).
- LlamaIndex `KnowledgeGraphQueryEngine`
  ([docs](https://developers.llamaindex.ai/python/examples/query_engine/knowledge_graph_query_engine/),
  [property-graph index](https://developers.llamaindex.ai/python/framework/module_guides/indexing/lpg_index_guide/)).
- *SPARQL-LLM: Real-Time SPARQL Query Generation from Natural Language Questions*
  ([arXiv:2512.14277](https://arxiv.org/html/2512.14277)) and IBM's
  `agentic-text2sparql` ([GitHub](https://github.com/IBM/agentic-text2sparql)) —
  the SPARQL (RDF) variant.
- *Multi-Agent GraphRAG: A Text-to-Cypher Framework for Labeled Property Graphs*
  ([arXiv:2511.08274](https://arxiv.org/pdf/2511.08274)) — multi-agent
  decomposition of the text-to-Cypher task.
- *RAS: Reflection-Augmented Scaling with In-Context Learning for Executable
  Cypher Query Generation* ([arXiv:2605.22937](https://arxiv.org/html/2605.22937v1))
  — the query-validation / self-correction loop.
- FalkorDB's GraphRAG-SDK `cypher_generation.py`
  ([GitHub](https://github.com/FalkorDB/GraphRAG-SDK/blob/main/graphrag_sdk/src/graphrag_sdk/retrieval/strategies/cypher_generation.py))
  — an open-source (AGPL-compatible) reference implementation of the strategy.
- Microsoft's *From Local to Global: A GraphRAG Approach to Query-Focused
  Summarization* ([arXiv:2404.16130](https://arxiv.org/pdf/2404.16130)) — the
  canonical GraphRAG paper; positions dynamic query generation against the
  precomputed community-summary path.
- Neo4j's *What is GraphRAG?* ([neo4j.com](https://neo4j.com/blog/genai/what-is-graphrag/))
  and the GraphRAG `Text2Cypher` reference
  ([graphrag.com](https://graphrag.com/reference/graphrag/text2cypher/)) — the
  technique's place in the Graph RAG taxonomy.

---

## §2 How it applies to Astrographer

Astrographer is a **Graphical RAG engine + document store/wiki** over **Provident
graphs** (`docs/specs/astrographer.md` §2, §4.1.1). Its data model is already a
knowledge graph with nodes, edges, and properties:

- **Nodes** (`astrographer.md` §4.2.1): `content`, `fact`, and `reference` node
  kinds. The `fact` node is the single source of truth (noun-focused, like a KG
  node).
- **Edges** (`astrographer.md` §4.2.2): `link` and `embed` reference modes —
  relationship-focused edges from a referencing document to a target
  `fact`/node.
- **Properties** (`astrographer.md` §4.1.1, §4.2.1): document metadata (`title`,
  `tags`, `author`, `revision`, `state`) and fact `factKey`/`value`; the `mode`
  (`link`/`embed`) and `crossWiki` flag are edge properties.

Dynamic graph query generation maps onto this substrate in concrete ways:

- **The `reference`→`fact` graph is a natural query target.** A question like
  "which documents embed fact X?" or "which published documents reference a fact
  that is itself referenced by document Y?" is a **graph traversal** over the
  `reference`/`embed` edges (`astrographer.md` §4.2). An LLM-generated graph
  query could express this directly, returning the exact subgraph — the
  relationship-aware, multi-hop retrieval the source talk says vectors miss.
- **The RAG surface (`astrographer.md` §4.3.2).** `ragQuery`/`ragStream` currently
  take a string `query` plus `topK`/`filters` and return results tagged
  `source: 'local'|'incanter'|'zodiac'`. A **graph-query mode** would let the LLM
  generate a structured query over the Provident graph (or over Incanter's graph
  model) instead of a similarity search. This is a **proposed spec change** to
  §4.3.2 — the `query` field would need a mode/format parameter, and the result
  shape would need to carry the matched subgraph (nodes + edges) alongside the
  existing `{documentId, nodeId, score, snippet, source}`.
- **Explainability / traceability.** The generated query *is* the trace. Because
  the consistency model already tracks every `link`/`embed` to its target and
  exposes `getConsistencyReport(wikiId)` (`astrographer.md` §4.1.3, §4.5.1
  `get_consistency_report`), a graph-query result can carry the exact edges
  walked — directly serving the source talk's "governance questions are more
  easily answered by sourcing and tracing."
- **D4 parity.** A graph-query mode must be reachable through both the GUI RAG
  panel (`astrographer.md` §4.6.2) and the MCP `rag_query`/`rag_stream` tools
  (`astrographer.md` §4.5.1). It is a retrieval-mode change, not a
  security-configuration feature, so it is **not** a D4 carve-out.

**Status in the spec:** **not present.** `astrographer.md` §4.3 pins a string
`query` with no graph-query mode. Dynamic graph query generation is a **proposed
spec change** to §4.3.2 (and the MCP/GUI parity surfaces §4.5.1/§4.6.2). It
aligns with the parked DAG-RAG SHOULD-HAVE (topological resolution of
`reference`→`fact`, `docs/research/dag-rag-research-notes.md` §4) and with the
sibling report's PARKED note (`graph-rag-astrographer.md` §5): it is the most
complex phase (multiple LLM calls per query) and should land **after** the
MUST/SHOULD retrieval foundation (reranking, multi-query, hybrid, compression)
ships.

**Dependency / feasibility caveat.** Dynamic graph query generation requires a
**graph query executor** (a Cypher/SPARQL engine) over the Provident graph. The
suite's Provident graphs are an in-memory/SSR graph model (Layer 1,
`docs/architecture-overview.md` §5.1), not a graph database with a query
language. So the technique is feasible but **gated on a graph-query substrate**
that does not yet exist in the suite. This is the dominant cost/benefit
consideration (see §5).

---

## §3 How it applies to Zodiac

Zodiac is a **graphical RAG engine integrated with web crawlers** that replies to
Astrographer with **pre-graphed and vector-embedded data**
(`docs/specs/zodiac.md` §2, §4.2). It is the suite's clearest Graph RAG engine:

- **Pre-graphing** (`zodiac.md` §4.2): crawled raw material is parsed into a
  graph (nodes + edges) — e.g. a dependency package's metadata, its dependencies,
  and their relationships. This is a knowledge graph built from crawled data.
- **Query modes** (`zodiac.md` §4.3.2): `graph` (returns the pre-graphed
  subgraph), `vector` (top-`limit` by similarity), and `hybrid` (merged graph +
  vector).

Dynamic graph query generation maps onto Zodiac in a specific way:

- **The `graph` mode (`zodiac.md` §4.3.2) is currently target-scoped.** It
  returns the *whole* pre-graphed subgraph for a `target` (a dependency package or
  web target). Dynamic graph query generation would let the LLM generate a
  **subgraph query** from the natural-language `query` to retrieve only the
  relevant portion of the pre-graphed data — e.g. "what are the transitive
  dependencies of package X that are also used by package Y?" — rather than
  returning the entire pre-graphed neighborhood. This is a **proposed spec change**
  to `zodiac.md` §4.3.2.
- **Query shape (`zodiac.md` §4.3.1).** The query carries `mode:
  'graph'|'vector'|'hybrid'`. A graph-query mode (LLM-generated) is a natural
  extension of the `graph` mode. The `target` field already scopes the graph; a
  generated query would add the traversal.
- **D4 parity.** A graph-query mode must be reachable through both the GUI and the
  MCP `zodiac_query` tool (`zodiac.md` §4.5). It is a retrieval-mode change, not a
  security-configuration feature, so it is **not** a D4 carve-out.
- **Remote/backend (D2-CLARIFICATION).** Zodiac is remote/backend by design
  (`zodiac.md` §4.4). The LLM that generates the graph query could run locally
  (Astrographer-side) or remotely (Zodiac-side); either is consistent with
  D2-CLARIFICATION, which permits remote-only features where the design case
  requires them.

**Status in the spec:** **not present.** `zodiac.md` §4.3.2 pins `graph`/`vector`/
`hybrid` modes but no LLM-generated graph-query mode. Dynamic graph query
generation is a **proposed spec change** to `zodiac.md` §4.3.2/§4.3.1. It is
lower priority than Astrographer's because Zodiac's `graph` mode already returns
a pre-graphed subgraph (the relationship data is already surfaced); the generated
query would add *precision* (retrieve only the relevant subgraph) rather than
*capability*.

---

## §4 How it applies to the other suite consumers

- **Incanter** (`docs/specs/incanter.md`): Incanter is the prototype Graph-RAG
  engine Astrographer consumes over HTTP (`incanter.md` §2, §5). Its hybrid query
  (`incanter.md` §4.8) fuses graph-tension + vector. Dynamic graph query
  generation is a natural **engine-side** capability: Incanter's `POST /v1/query`
  could accept an LLM-generated graph query (or a query-mode parameter) and
  execute it over its document graph. This mirrors GAP-1 (multi-query belongs
  inside Incanter, `docs/defects.md` GAP-1) — the engine boundary is the correct
  home for query-time graph retrieval. **Recommendation:** a **handoff candidate**
  to Incanter (recorded per AGENTS.md — this repo never patches a tool's source);
  Astrographer's `ragQuery` would pass the graph-query mode through unchanged.

- **Familiar** (`docs/specs/familiar.md`): Familiar's knowledge domain delegates
  to Astrographer's RAG surface (`familiar.md` §4.3.1) and its research domain
  routes through Astrographer's `rag_query` (`familiar.md` §4.3.3). Dynamic graph
  query generation applies to Familiar **indirectly** — Familiar's natural-language
  questions would be answered by Astrographer's graph-query mode. Familiar itself
  does not generate graph queries; it delegates. **Recommendation:** no direct
  work; Familiar inherits the capability through Astrographer.

- **Solomon** (`docs/specs/solomon.md`): Solomon is cross-instance search over
  `zodiac|astral|astrographer` instances (`solomon.md` §4.2, §4.5.1). Its query
  shape is a string `query` with `instanceTypes`/`limit`/`timeout`; its result
  aggregation is flat (grouped by peer, `solomon.md` §4.2.2). Dynamic graph query
  generation could be a **per-instance retrieval mode** (each instance answers
  with a graph query), but Solomon's Phase-1 contract deliberately keeps
  cross-peer ordering unpinned (`solomon.md` §7.4). **Recommendation:**
  NICE-TO-HAVE / PARKED — defer until Phase-1 search semantics finalize.

- **Mystery** (`docs/specs/mystery.md` — **absent**; `docs/pending.md` #12):
  Mystery is a message board that crosslinks information references by post topic
  (`docs/architecture-overview.md` §4.3). Its topic→reference resolution
  (`docs/pending.md` #12) is a relationship-mapping problem that a graph query
  over Astrographer's reference graph could serve. **Recommendation:** when the
  Mystery contract is written, consider a graph-query path for topic→reference
  resolution. PARKED until the spec exists.

- **Astral** (`docs/specs/astral.md`): Astral is the wiki/publishing host
  (`astral.md` §2). It receives Provident graphs (`astral.md` §4.2) and serves
  them (`astral.md` §4.1). Dynamic graph query generation applies to Astral only
  as a **serving** concern — the hosted pages are graphs, but Astral does not
  perform retrieval. **Recommendation:** no direct work; Astral serves the graphs
  Astrographer produces.

---

## §5 Recommendation for the suite

Grounded in D1–D4:

- **SHOULD — Dynamic graph query generation as a graph-query mode on
  Astrographer's RAG surface (`astrographer.md` §4.3.2), gated on a graph-query
  substrate.** The technique is a strong, graph-native fit: Astrographer's data
  model is already a knowledge graph (`astrographer.md` §4.2), and an
  LLM-generated graph query directly expresses the relationship-aware, multi-hop
  retrieval the source talk says vectors miss, with the generated query doubling
  as the explainability trace. It is fully D2-compatible (the LLM and the graph
  are local) and D4-compatible (a retrieval-mode change, not a carve-out). **Why
  SHOULD, not MUST:** it is an *enhancement* to an existing retrieval surface, not
  a prerequisite for the document-store/wiki core; and it is **gated on a graph
  query executor** over Provident graphs, which the suite does not yet have. It
  should land **after** the MUST/SHOULD retrieval foundation (reranking,
  multi-query, hybrid, compression — `docs/pending.md` D3/D4/D5) ships, consistent
  with the sibling report's PARKED-until-foundation note
  (`graph-rag-astrographer.md` §5).

- **SHOULD — Zodiac graph-query mode (`zodiac.md` §4.3.2), lower priority.**
  Zodiac's `graph` mode already returns a pre-graphed subgraph; a generated query
  adds precision (retrieve only the relevant subgraph). **Proposed spec change**;
  land after Astrographer's graph-query mode.

- **NICE-TO-HAVE — Incanter engine-side graph-query capability.** The engine
  boundary is the correct home for query-time graph retrieval (mirrors GAP-1,
  `docs/defects.md`). **Handoff candidate** to Incanter; not a this-repo change.

- **PARKED — Solomon graph-aware cross-instance ranking.** Defer until Phase-1
  search semantics finalize (`solomon.md` §7.4).

- **PARKED — Mystery topic→reference graph query.** Defer until the Mystery
  contract exists (`docs/pending.md` #12).

**Overall recommendation tier: SHOULD.** Dynamic graph query generation is a
high-value, graph-native retrieval technique that directly addresses the source
talk's "vectors don't capture relationship data" and "vector similarity ≠
relevance" failures, and it doubles as an explainability mechanism. It is not a
MUST because it is an enhancement gated on a graph-query substrate the suite
does not yet have, and it is not NICE-TO-HAVE because the graph-native value is
too high to defer indefinitely. The concrete path: (1) ship the MUST/SHOULD
retrieval foundation; (2) add a graph-query substrate over Provident graphs (or
delegate to Incanter); (3) add the graph-query mode to `astrographer.md` §4.3.2
and `zodiac.md` §4.3.2 with D4 parity.

---

## §6 Source URL list

1. https://arxiv.org/html/2412.10064 — *Text2Cypher: Bridging Natural Language and Graph Databases*
2. https://aclanthology.org/2025.genaik-1.11.pdf — *Text2Cypher* (ACL GenAIK 2025)
3. https://neo4j.com/docs/neo4j-graphrag-python/current/user_guide_rag.html — Neo4j GraphRAG, *User Guide: RAG*
4. https://neo4j.com/docs/neo4j-graphrag-python/current/_modules/neo4j_graphrag/retrievers/text2cypher.html — Neo4j GraphRAG, *Text2CypherRetriever source*
5. https://neo4j.com/blog/developer/effortless-rag-text2cypherretriever/ — Neo4j, *Effortless RAG with Text2CypherRetriever*
6. https://graphrag.com/reference/graphrag/text2cypher/ — GraphRAG, *Text2Cypher reference*
7. https://neo4j.com/blog/genai/what-is-graphrag/ — Neo4j, *What is GraphRAG?*
8. https://reference.langchain.com/python/langchain-neo4j/chains/graph_qa/cypher/GraphCypherQAChain — LangChain, *GraphCypherQAChain*
9. https://docs.langchain.com/oss/python/integrations/graphs/neo4j_cypher — LangChain, *Neo4j Cypher integration*
10. https://developers.llamaindex.ai/python/examples/query_engine/knowledge_graph_query_engine/ — LlamaIndex, *Knowledge Graph Query Engine*
11. https://developers.llamaindex.ai/python/framework/module_guides/indexing/lpg_index_guide/ — LlamaIndex, *Property Graph Index*
12. https://arxiv.org/html/2512.14277 — *SPARQL-LLM: Real-Time SPARQL Query Generation from Natural Language Questions*
13. https://github.com/IBM/agentic-text2sparql — IBM, *agentic-text2sparql*
14. https://arxiv.org/pdf/2511.08274 — *Multi-Agent GraphRAG: A Text-to-Cypher Framework for Labeled Property Graphs*
15. https://arxiv.org/html/2605.22937v1 — *RAS: Reflection-Augmented Scaling with In-Context Learning for Executable Cypher Query Generation*
16. https://github.com/FalkorDB/GraphRAG-SDK/blob/main/graphrag_sdk/src/graphrag_sdk/retrieval/strategies/cypher_generation.py — FalkorDB GraphRAG-SDK, *cypher_generation.py*
17. https://arxiv.org/pdf/2404.16130 — Microsoft, *From Local to Global: A GraphRAG Approach to Query-Focused Summarization*
