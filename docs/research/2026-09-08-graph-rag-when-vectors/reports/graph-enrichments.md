# Graph Enrichments — Research Report

**Topic:** Graph enrichments (enriching a knowledge graph — entity resolution,
relationship enrichment, node/edge properties, community detection — to improve
graph-aware RAG retrieval)
**Date:** 2026-09-08
**Tier:** SHOULD
**Report slug:** graph-enrichments
**Primary focus:** Astrographer, Zodiac · **Secondary:** Incanter, Familiar,
Solomon, Mystery, Astral

---

## §1 What the technique is (web-grounded, cited)

"Graph enrichments" is the family of techniques that **improve the quality and
density of a knowledge graph before or during retrieval**, so that graph-aware
RAG returns more relevant, more complete, and more trustworthy context. It is
one of the retrieval-path components of Graph RAG — the class of RAG where the
retrieval path includes a knowledge graph alongside (or instead of) a vector
store. The core idea: a graph is only as useful as the entities, relationships,
and properties it encodes; enrichment makes that encoding richer and more
accurate.

The concrete enrichment operations, grounded in the literature:

1. **Entity resolution / deduplication.** Merging duplicate entities that refer
   to the same real-world object (e.g. "Apple Inc." vs "Apple") so the graph
   does not fragment a single concept across many nodes. This is the single most
   cited enrichment step. TypeGraph's *Entity Resolution in RAG Pipelines*
   ([typegraph.ai](https://typegraph.ai/blog/entity-resolution-rag-knowledge-graph))
   and OpenDataScience's *Entity Resolved Knowledge Graphs* ([opendatascience.com](https://opendatascience.com/entity-resolved-knowledge-graphs-the-foundation-for-effective-graphrag/))
   both frame entity resolution as the foundation for effective GraphRAG — a
   graph with unresolved duplicates produces noisy, split retrieval. The Neural
   Base's GraphRAG course covers entity resolution for newly ingested content
   ([theneuralbase.com](https://theneuralbase.com/graphrag/learn/intermediate/entity-resolution-for-new-content/)).
2. **Relationship enrichment.** Adding edges between entities that are related
   but not explicitly co-mentioned, so multi-hop traversal can reach them.
   KARMA ([arXiv 2502.06472](https://arxiv.org/html/2502.06472)) uses multi-agent
   LLMs to **automatically enrich a knowledge graph** with new relationships and
   entities; *LLM-Based Automatic Knowledge Graph Enrichment*
   ([springerprofessional.de](https://www.springerprofessional.de/large-language-model-based-automatic-knowledge-graph-enrichment/52617996))
   surveys the same automatic-enrichment pattern.
3. **Node/edge property enrichment.** Attaching metadata (types, attributes,
   confidence, provenance) to nodes and edges so retrieval can filter and rank
   on structure. *Graphs RAG at Scale* ([arXiv 2603.22340](https://arxiv.org/abs/2603.22340v1))
   argues for **labeled property graphs** (nodes/edges with typed properties) as
   the substrate for complex, unknown search spaces — richer properties make the
   graph more queryable.
4. **Community detection / summarization.** Grouping related nodes into
   communities and summarizing them so broad, theme-level queries can be
   answered. This is the signature of Microsoft's GraphRAG
   ([arXiv 2404.16130](https://arxiv.org/pdf/2404.16130),
   [microsoft.github.io/graphrag](https://microsoft.github.io/graphrag/),
   [github.com/microsoft/graphrag](https://github.com/microsoft/graphrag)): it
   builds a graph, detects hierarchical communities, and generates community
   summaries for global (theme-level) retrieval.
5. **Graph denoising / pruning.** Removing spurious or low-confidence nodes and
   edges so retrieval is not polluted. *Less is More: Denoising Knowledge Graphs
   for RAG* ([arXiv 2510.14271](https://www.alphaxiv.org/abs/2510.14271)) shows
   that pruning a noisy graph can *improve* retrieval — enrichment is not only
   additive, it is also subtractive.
6. **Graph-enhanced vector search.** Using the enriched graph to augment or
   re-rank vector retrieval — e.g. expanding a vector hit to its graph
   neighborhood, or using graph structure to re-rank. GraphRAG's
   *Graph-Enhanced Vector Search* reference ([graphrag.com](https://graphrag.com/reference/graphrag/graph-enhanced-vector-search/))
   and GraphER ([arXiv 2603.24925](https://arxiv.org/html/2603.24925v3)) — a
   graph-based **enrichment and reranking** method — are the canonical examples.

**Why it matters for RAG.** The source talk's core thesis is that **vector
similarity ≠ relevance** and that **vectors don't capture relationship data**.
Graph enrichments are the mechanism that supplies the relationship context a
vector store cannot: by resolving entities, adding relationships, and attaching
properties, the graph becomes a source of *relational* relevance that
complements (and corrects) vector similarity. It also improves
**explainability** — a richer graph with provenance lets a governance question
be answered by tracing the source, rather than by opaque vector similarity.

**Cost model.** Enrichment is **LLM-heavy** (entity resolution, relationship
extraction, and community summarization are all LLM passes) and is primarily an
**ingestion-time / index-time** cost, not a query-time cost. This is the key
D2 (local-first) tension: enrichment must run on local LLMs and be budgeted
against ingestion latency and token cost. The query-time benefit is cheaper
retrieval (a cleaner graph needs less post-processing), but the enrichment
itself is paid up front.

---

## §2 How it applies to Astrographer

Astrographer is the **Graphical RAG engine + document store/wiki** where
documents are **Provident graphs** (nodes + edges) and retrieval is graph-aware
(`docs/specs/astrographer.md` §2). Its graph model is already richer than a
flat text store, and graph enrichments map onto it in several concrete ways.

**The existing graph structure (§4.2).** Astrographer's nodes are one of
`content`, `fact`, or `reference` (§4.2.1); references are `link` (live pointer)
or `embed` (snapshot) (§4.2.2); and the core invariant is that a `fact`'s
canonical `value` is the single source of truth, with every `embed` re-synced
and every `link` resolved (§4.2.3). This is a **property-rich graph already** —
the `fact`/`reference`/`embed` distinction and the `FRESH`/`STALE`/`RESOLVED`/
`BROKEN` states are node/edge properties in the labeled-property-graph sense
([arXiv 2603.22340](https://arxiv.org/abs/2603.22340v1)).

**Where graph enrichment adds value:**

- **Entity resolution over `fact` nodes (§4.2.1).** A `factKey` is unique within
  a Wiki, but nothing prevents two *different* `fact` nodes in the same Wiki
  (or across wikis) from referring to the same real-world entity under different
  keys (e.g. "license" vs "licence", or "Apache Iceberg" vs "Iceberg"). Entity
  resolution ([typegraph.ai](https://typegraph.ai/blog/entity-resolution-rag-knowledge-graph),
  [opendatascience.com](https://opendatascience.com/entity-resolved-knowledge-graphs-the-foundation-for-effective-graphrag/))
  would merge or alias these so a query about the entity surfaces the canonical
  `fact` node, not a fragmented set. This directly strengthens the
  single-source-of-truth invariant (§4.2.3) — the consistency model is only as
  good as the fact graph's entity resolution.
- **Relationship enrichment between `fact` nodes (§4.2).** The wiki's reference
  graph already encodes `reference → fact` edges. Enrichment would add
  *semantic* edges between related facts (e.g. a dependency-package fact and its
  license fact) so multi-hop queries can traverse them. This is the
  relationship-context that the source talk says vectors lack, and it is exactly
  the automatic-enrichment pattern of KARMA
  ([arXiv 2502.06472](https://arxiv.org/html/2502.06472)).
- **Graph denoising (§4.2.3).** The `STALE`/`BROKEN` states are a form of
  denoising — a `STALE` embed or `BROKEN` link is a low-quality graph element
  that should not be retrieved as authoritative. *Less is More*
  ([arXiv 2510.14271](https://www.alphaxiv.org/abs/2510.14271)) supports the
  principle that pruning low-quality graph elements improves retrieval; the
  consistency report (`getConsistencyReport`, §4.2.3) is the natural surface to
  expose which nodes are denoised.
- **Graph-enhanced vector search (§4.3).** Astrographer's `ragQuery`/`ragStream`
  (§4.3.2) return results tagged `source: 'local'|'incanter'|'zodiac'`. A
  graph-enrichment layer could expand a vector hit to its graph neighborhood
  (the `fact` it references, the documents that `link`/`embed` it) — the
  graph-enhanced-vector-search pattern
  ([graphrag.com](https://graphrag.com/reference/graphrag/graph-enhanced-vector-search/),
  [arXiv 2603.24925](https://arxiv.org/html/2603.24925v3)). This is a
  **query-time** enrichment that composes with the existing RAG surface without
  changing the engine boundary.

**Status in the spec.** Graph enrichments are **NOT currently in the
Astrographer contract**. The graph model (§4.2) provides the *substrate* (fact/
reference/embed nodes, consistency states) but the contract does not pin any
enrichment operation (entity resolution, relationship enrichment, community
detection). It is a **proposed spec change** — a new §4.x subsection (e.g.
§4.7 "Graph enrichment") would pin the enrichment operations, their MCP/GUI
surfaces (D4), and their fail-states. It is not parked; it is a forward-looking
enhancement that builds on the existing graph model.

---

## §3 How it applies to Zodiac

Zodiac is the **backend/remote companion to Astrographer** — a graphical RAG
engine integrated with web crawlers that **pre-graphs and vector-embeds**
crawled data (`docs/specs/zodiac.md` §2). It is the suite's primary
graph-construction surface, and therefore the place where graph enrichment is
**most valuable**.

**The existing pre-graphing pipeline (§4.2).** Zodiac parses crawled raw
material into a **graph** (nodes and edges) representing e.g. a dependency
package's metadata, its dependencies, and their relationships (§4.2
"Pre-graphing"), then vector-embeds the raw material and/or graph nodes (§4.2
"Vector-embedding"). The RAG store has `EMPTY`/`READY`/`STALE`/`REFRESHING`
states (§4.2). The query-reply surface supports `graph`/`vector`/`hybrid` modes
(§4.3.2).

**Where graph enrichment adds value:**

- **Entity resolution across crawled sources (§4.2).** Zodiac crawls the web
  for dependency packages and current data. The same package/entity will be
  crawled from multiple sources under different names or versions. Entity
  resolution ([typegraph.ai](https://typegraph.ai/blog/entity-resolution-rag-knowledge-graph))
  is the natural enrichment step to merge these into a single canonical node —
  otherwise the pre-graphed store fragments a package across many nodes and
  `graph`-mode queries (§4.3.2) return split results. This is the highest-value
  enrichment for Zodiac.
- **Relationship enrichment of the pre-graph (§4.2).** The pre-graph already
  captures dependency relationships. Enrichment would add *derived* edges (e.g.
  "package A is used by package B", "package A and B share a license") so
  multi-hop queries over the crawled graph are possible. This is the
  automatic-enrichment pattern of KARMA
  ([arXiv 2502.06472](https://arxiv.org/html/2502.06472)) and the
  relationship-context the source talk says vectors lack.
- **Property enrichment (§4.2).** Attaching typed properties (version, license,
  maintainer, source URL, crawl provenance) to pre-graph nodes/edges makes the
  `graph`-mode query (§4.3.2) filterable and rankable on structure — the
  labeled-property-graph argument of *Graphs RAG at Scale*
  ([arXiv 2603.22340](https://arxiv.org/abs/2603.22340v1)).
- **Graph denoising (§4.2, §6).** Zodiac's fail-states include crawler-blocked
  and vector-embedding-failure (§6.1, §6.4). A denoising pass
  ([arXiv 2510.14271](https://www.alphaxiv.org/abs/2510.14271)) would prune
  low-confidence or stale pre-graph nodes so `graph`-mode retrieval is not
  polluted by unreliable crawled data.

**Status in the spec.** Graph enrichments are **NOT currently in the Zodiac
contract**. The pre-graphing pipeline (§4.2) is the *construction* step, but no
enrichment operation (entity resolution, relationship/property enrichment,
denoising) is pinned. It is a **proposed spec change** — a new §4.x subsection
(e.g. §4.6 "Graph enrichment") pinning the enrichment operations, their
MCP/GUI surfaces (D4), and their fail-states. Because Zodiac is the
graph-construction surface, this is the natural home for the suite's enrichment
logic.

---

## §4 How it applies to the other suite consumers

- **Incanter** (`docs/specs/incanter.md`). Incanter is the prototype Rust
  Graph-RAG engine Astrographer consumes over HTTP/REST + SSE (§4.2). Its graph
  model is a **token-transition multigraph** (§4.4) — a *different* graph from
  Astrographer's semantic fact graph. Graph enrichment applies to Incanter
  mainly as **graph-enhanced vector search** (§4.8 hybrid query): enriching the
  graph-tension + vector fusion with entity/relationship context. However,
  Incanter's graph is a token-co-occurrence structure, not a semantic entity
  graph, so entity resolution and relationship enrichment are **less natural**
  here than in Astrographer/Zodiac. The higher-value enrichment lives in the
  semantic graphs (Astrographer facts, Zodiac pre-graphs), not in Incanter's
  token graph. **Recommendation:** leave Incanter's token graph as-is; do not
  add semantic enrichment to it.
- **Familiar** (`docs/specs/familiar.md`). Familiar's memory store is a
  **facts table** of individually addressable fact records (§4.1.3). Graph
  enrichment applies to the **facts table** as entity resolution / dedup: the
  candidate-fact pipeline (§4.1.3.1) already has a deterministic dedup/conflict
  stage, and a graph-enrichment pass could merge facts that refer to the same
  entity. Familiar's knowledge domain delegates to Astrographer (F1, §4.3.1) and
  research routes through Astrographer's RAG surface (F4, §4.3.3), so Familiar
  *consumes* enriched graphs rather than building them. **Recommendation:**
  NICE-TO-HAVE — a light entity-resolution pass over the facts table, reusing
  the existing deterministic validation gate (§4.1.3.1).
- **Solomon** (`docs/specs/solomon.md`). Solomon is the cross-instance search
  layer (§4.2). It fans queries out to peers and aggregates results (§4.2.2).
  Graph enrichment applies to Solomon as **cross-instance entity resolution**:
  the same entity may exist in different instances' graphs under different
  keys, and Solomon's aggregation (§4.2.2) could merge results by resolved
  entity identity rather than by raw node id. This is a **query-time**
  enrichment that improves cross-instance result quality. **Recommendation:**
  NICE-TO-HAVE — cross-instance entity resolution in the aggregation step;
  park until the base search contract (§4.2) is implemented.
- **Mystery** (`docs/specs/mystery.md` — **absent**). Mystery is a PLANNED
  message-board app that crosslinks information references by post topic
  (`docs/architecture-overview.md` §4.3). Its crosslink resolution is an OPEN
  item (`docs/pending.md` #12). Graph enrichment would apply to Mystery's
  topic→reference resolution (mapping a post topic to the relevant Astrographer
  documentation / Astral pages) — a form of relationship enrichment over the
  suite's reference graph. **Recommendation:** PARKED — no Mystery contract
  exists; revisit when `docs/specs/mystery.md` is written (consistent with the
  F3 deferral in `docs/specs/familiar.md` §5.6).
- **Astral** (`docs/specs/astral.md`). Astral is the webhost that receives
  Provident-graph pushes (§4.2) and serves them (§4.1). It is a **host**, not a
  graph-construction or retrieval surface, so graph enrichment does not apply to
  Astral's core function. The only relevance is that Astral *serves* enriched
  graphs (the pushed `provident-graph/1` payload, §4.2) — enrichment happens
  upstream in Astrographer/Zodiac. **Recommendation:** DISCARD for Astral's own
  contract; it is a passive consumer of enriched graphs.

---

## §5 Recommendation for the suite

**Recommendation: SHOULD HAVE** — a scoped, local-first first pass, with the
heaviest enrichment (community summaries) parked.

**Rationale, grounded in D1–D4:**

- **D2 (local-first) is the deciding constraint.** Enrichment is LLM-heavy and
  ingestion-time. The suite already runs local LLMs (Incanter's local Ollama,
  `incanter.md` §4.7; Familiar's light-LLM candidate extraction, `familiar.md`
  §4.1.3.1), so entity resolution and relationship enrichment **can** run locally
  — but they must be budgeted against ingestion latency and token cost. The
  **community-summarization** pass (GraphRAG's signature,
  [arXiv 2404.16130](https://arxiv.org/pdf/2404.16130)) is the heaviest and
  least D2-friendly; **park it** (consistent with the DAG-RAG research note's
  DISCARD of pre-built global graph construction, `docs/research/dag-rag-research-notes.md` §4).
- **D1 (AGPL-3.0).** Entity resolution, relationship enrichment, and denoising
  are well-known algorithms with no licensing constraint; they can be
  implemented in the suite's own repos. No blocker.
- **D3 (interconnection).** Enrichment is a natural **shared** capability:
  Zodiac enriches its pre-graphs (the graph-construction surface), Astrographer
  enriches its fact graph and consumes Zodiac's enriched data (edge #1, §5.7),
  and Familiar/Solomon consume enriched graphs. This is a clean D3 crossover —
  enrichment done once in Zodiac/Astrographer benefits the whole suite.
- **D4 (MCP-GUI parity).** Any enrichment feature must be reachable through both
  the GUI and the MCP surface. For Astrographer this means new MCP tools (e.g.
  `resolve_entities`, `enrich_graph`) alongside the existing §4.5.1 set; for
  Zodiac, new tools alongside the §4.5 set. Enrichment is **not** a
  security-configuration feature, so it is **not** a D4 carve-out — it must be
  exposed on both surfaces.

**Why SHOULD, not MUST:** the suite's core value — Astrographer's
document-store/wiki + cross-link/data-embed consistency (§4.2), Zodiac's
pre-graphing + vector-embedding (§4.2), the RAG surface (§4.3) — works **without**
graph enrichment. Enrichment is an *enhancement* to retrieval quality and
relationship context, not a prerequisite for the core build. It should not block
the Astrographer/Zodiac MVP.

**Why SHOULD, not NICE-TO-HAVE/DISCARD:** the source talk's core thesis is that
**vectors don't capture relationship data** and **vector similarity ≠
relevance**. Graph enrichment is the mechanism that supplies the relationship
context a vector store cannot, and it directly improves the explainability the
source talk calls out for governance questions. Discarding it would leave the
suite's graph-aware RAG without the very enrichment that makes graph-aware
retrieval more than vector retrieval. The cost is bounded (local LLMs, scoped
passes), so it is more than a nice-to-have.

**Suggested disposition:**

1. **SHOULD HAVE — entity resolution over `fact` nodes (Astrographer, §4.2.1).**
   Merge/alias `fact` nodes that refer to the same real-world entity, so the
   single-source-of-truth invariant (§4.2.3) is not fragmented. Highest-value,
   lowest-cost increment; aligns with the fact-key model.
2. **SHOULD HAVE — entity resolution + relationship/property enrichment in
   Zodiac's pre-graphing pipeline (§4.2).** Resolve crawled entities across
   sources and add derived edges/properties so `graph`-mode queries (§4.3.2) are
   complete. This is the suite's graph-construction surface and the natural home
   for enrichment.
3. **NICE-TO-HAVE — graph-enhanced vector search in Astrographer's RAG surface
   (§4.3.2).** Expand a vector hit to its graph neighborhood (the `fact` it
   references, the documents that `link`/`embed` it). Query-time, composes with
   the existing `ragQuery`/`ragStream` without changing the engine boundary.
4. **NICE-TO-HAVE — cross-instance entity resolution in Solomon's aggregation
   (§4.2.2).** Merge results by resolved entity identity across peers.
5. **PARKED — community summaries (GraphRAG-style).** Heaviest, least D2-friendly;
   park until the base graph + enrichment passes ship.
6. **DISCARD — semantic enrichment of Incanter's token-transition graph (§4.4).**
   Incanter's graph is a token-co-occurrence structure, not a semantic entity
   graph; enrichment belongs in Astrographer/Zodiac, not Incanter.

**Spec-change note.** This is a documentation deliverable; it does **not** modify
any `docs/specs/*.md` contract. The proposed spec changes (new §4.x enrichment
subsections in `astrographer.md` and `zodiac.md`) are recommendations for a
future spec-gate pass, not edits made here.

---

## §6 Source URL list

**Entity resolution / dedup**
- https://typegraph.ai/blog/entity-resolution-rag-knowledge-graph
- https://opendatascience.com/entity-resolved-knowledge-graphs-the-foundation-for-effective-graphrag/
- https://theneuralbase.com/graphrag/learn/intermediate/entity-resolution-for-new-content/

**Automatic graph enrichment (relationships/entities)**
- https://arxiv.org/html/2502.06472 (KARMA)
- https://www.springerprofessional.de/large-language-model-based-automatic-knowledge-graph-enrichment/52617996

**Property graphs / labeled property graphs**
- https://arxiv.org/abs/2603.22340v1 (Graphs RAG at Scale)

**Community detection / summarization (GraphRAG)**
- https://arxiv.org/pdf/2404.16130 (GraphRAG paper)
- https://www.microsoft.com/en-us/research/publication/from-local-to-global-a-graph-rag-approach-to-query-focused-summarization/
- https://microsoft.github.io/graphrag/
- https://github.com/microsoft/graphrag

**Graph denoising / pruning**
- https://www.alphaxiv.org/abs/2510.14271 (Less is More: Denoising KGs for RAG)

**Graph-enhanced vector search / enrichment + reranking**
- https://graphrag.com/reference/graphrag/graph-enhanced-vector-search/
- https://arxiv.org/html/2603.24925v3 (GraphER)

**Graph construction / reorganization (context)**
- https://arxiv.org/pdf/2504.09823v1 (RAKG)
- https://aclanthology.org/2025.findings-emnlp.290.pdf (ReGraphRAG)
- https://arxiv.org/pdf/2504.11544 (NodeRAG)
- https://enison.ai/en/blog/knowledge-graph-rag-implementation
