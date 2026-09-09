# Graph RAG — "When Vectors aren't enough" (Nyah Macklin) — Research Notes

**Topic:** Graph RAG — why vector-only retrieval fails (relationship data, similarity ≠ relevance, explainability, maturity), the knowledge-graph model (noun nodes / relationship edges / properties), and the graph-RAG technique family (graph-enhanced vector search, dynamic graph query generation, parent-child retrievers, community summaries, graph enrichments)
**Date:** 2026-09-08
**Status:** Research / reference notes (not a behavior contract)
**Scope:** General technique + Auspicion Suite application analysis, with **primary focus on Astrographer and Zodiac** (`docs/specs/astrographer.md`, `docs/specs/zodiac.md`), and secondary coverage of the other consumers (Incanter, Familiar, Solomon, Mystery, Astral)
**Source:** `/media/ryanr/Boot/Tech Talks/Commit Your Code/When Vectors aren't enough.md` (local notes, 36 lines)

---

## 1. Source notes

| Item | Source |
| --- | --- |
| **Source notes (local file)** | `/media/ryanr/Boot/Tech Talks/Commit Your Code/When Vectors aren't enough.md` |
| **Speaker / origin** | Nyah Macklin — "When Vectors aren't enough" (Commit Your Code talk; terse talk summary) |
| **Normal RAG pipeline** | "Question / Retriever / Data source request → Response" |
| **Vector RAG failure — relationship data** | "Vectors don't capture relationship data. Knowledge graph can supply context for relationships that allow for follow-up questions. Supplies domain knowledge." |
| **Vector RAG failure — similarity ≠ relevance** | "Vector similarity != relevance." |
| **Vector RAG failure — explainability** | "Lack explainability. Governance questions are more easily answered by sourcing and tracing." |
| **Vector RAG failure — maturity** | "Maturity not on par with existing database systems." |
| **Relationship data** | "Filtering · Multi-Step query data · Multi-Hop Questions" |
| **Similarity ≠ relevance (worked example)** | "Ex. Person with comment access to documentation will frequently appear in docs and thus might appear to a vector search as if they are part of doc team regardless of whether they really are." |
| **Knowledge-graph model** | "Nodes — 'Noun'-focused. Edges — Relationship focused. Properties — Can be attached to nodes and edges." |
| **Graph RAG definition** | "RAG where retrieval path also includes a knowledge graph. Use both vector and graph layer methods." |
| **Graph RAG techniques** | "Graph-enhanced vector search · Dynamic graph query generation · Parent-child retrievers · Community summaries · graph enrichments" |

**Origin / status.** The source is a terse set of personal notes (a talk summary), not a peer-reviewed paper. The techniques are a well-known graph-RAG pattern (knowledge-graph-augmented retrieval, GraphRAG-style community summarization, hybrid graph+vector retrieval). The notes are treated as **design guidance**, not a canonical spec; each technique is evaluated for Auspicion fit below. The prior `graph-rag-deep-dive` report was never produced — `docs/research/2026-09-06-graph-rag-deep-dive/` contains only a failed ingest-log (`rag-ingester: command not found`); this note is the first consolidated graph-RAG research note for the suite.

---

## 1a. Web-grounded research (bulk-research run, 2026-09-08)

The source notes were run through the bulk-research workflow (P0→P7),
producing **12 web-grounded research reports** under
`docs/research/2026-09-08-graph-rag-when-vectors/reports/` (each `.md` + `.json`,
each citing 16–37 URLs). The reports corroborate and deepen the source
techniques and refine the recommendation tiers below. Key findings:

| Report | Key finding relevant to the suite |
| --- | --- |
| `graph-rag-vector-failures` | The four vector-RAG failure modes (relationship data, similarity≠relevance, explainability, maturity) are the **justification for the suite's graph-first architecture**, not a new technique to add. Astrographer's Provident graph (`astrographer.md` §4.2) and Zodiac's pre-graphing (`zodiac.md` §4.2) already capture relationship data at the substrate. |
| `knowledge-graph-model` | The noun-node / relationship-edge / property model **is the labeled property graph (LPG) model**. Astrographer's Provident graph already IS the model (`fact`/`reference`/`content` nodes §4.2.1, `link`/`embed` edges §4.2.2, properties §4.1.1); Zodiac's pre-graphing (§4.2) is the suite's clearest entity/relation extraction. |
| `graph-enhanced-vector-search` | Graph-enhanced vector search is a mature, production-stable hybrid family (Microsoft GraphRAG, TigerVector, Neo4j, GNN-RAG, GRAG). Zodiac's `hybrid` mode (`zodiac.md` §4.3.1/§4.3.2) and Incanter's graph-tension + vector hybrid (`incanter.md` §4.8) are already the in-spec expression. |
| `dynamic-graph-query-generation` | An LLM writes a structured graph query (Cypher/SPARQL) per question and executes it against the KG — the generated query doubles as the **explainability trace**. Astrographer's flat top-K RAG surface (§4.3.2) and Zodiac's static `mode: graph` (§4.3.2) do not yet express it. |
| `parent-child-retrievers` | Decouples the indexing unit (small child chunks for precise retrieval) from the context unit (the larger parent returned to the generator). Astrographer's document→node granularity (§4.1.1) and Incanter's dynamic chunking (`incanter.md` §4.6) are the pattern; a parent-context return is the missing piece. |
| `community-summaries` | Community summarization (GraphRAG global search) answers global/theme questions but is the **most expensive** part of GraphRAG (LLM pass over every community at index time). LightRAG skips it and reports matching/beating GraphRAG at ~1–2 orders lower cost. A **scoped, lazy, local** wiki-level/per-target summary is the D2-compatible SHOULD; the full hierarchical pipeline stays PARKED. |
| `graph-enrichments` | Entity resolution, relationship/property enrichment, and denoising supply the relationship context vector similarity lacks. Astrographer's fact/reference/embed graph (§4.2) is already property-rich; the highest-value enrichment is entity resolution. |
| `multi-hop-questions` | Relationship data (filtering, multi-step, multi-hop) is the suite's **native** RAG value proposition. Astrographer's `ragQuery` already has an undefined `filters?` parameter (§4.3.2) — the low-risk, additive home for relationship filtering. |
| `explainability-governance` | The provenance substrate is already in the specs (Astrographer consistency report §4.2.3, Zodiac `sourceCrawlId` §4.3.2, Familiar facts provenance §4.1.3, Solomon `peerId` §4.2.2, Astral `contentHash` §4.1). The concrete MUST work is making provenance **answer-level**: add `citations` + `trace` fields to RAG results. |
| `graph-rag-astrographer` | Graph RAG is a retrieval behavior over the existing substrate, not a foreign addition. The graph-enhanced vector search axis is already in the suite (Incanter §4.8 + Zodiac §4.3); the MUST work is completing graph-aware retrieval over the reference graph. |
| `graph-rag-zodiac` | Graph RAG is the **core** of Zodiac's design, not an enhancement: pre-graphing + vector-embedding (§4.2) and graph/vector/hybrid modes (§4.3.2) are exactly the knowledge-graph-from-web-crawl and graph-enhanced-vector-search patterns. |
| `graph-rag-suite-consumers` | The suite is already graph-native (Astrographer, Zodiac, Incanter are all graphical RAG engines). Graph RAG is a completion/exposure task, not a foreign adoption. |

**Net refinement to the recommendation tiers (§4):** the web-grounded research
confirms the MUST/SHOULD/PARKED split and adds precision:
- **MUST** — keep the graph-first substrate (Astrographer §4.2 consistency
  invariant + publish gate, `getConsistencyReport`; Zodiac §4.2 pipeline +
  `mode: hybrid` + RAG store states + `sourceCrawlId`). The four failure modes
  are the *justification* for the architecture, not new work
  (`graph-rag-vector-failures`, `knowledge-graph-model`, `graph-rag-astrographer`,
  `graph-rag-zodiac`).
- **SHOULD** — make provenance **answer-level** (`citations` + `trace` on RAG
  results, Astrographer §4.3.2 + Zodiac §4.3.2) (`explainability-governance`);
  add a **parent-context return** to Astrographer's RAG surface
  (`parent-child-retrievers`); add a **multi-hop / dynamic graph query mode**
  (Incanter-repo change surfaced through `ragQuery`, GAP-7) and a **query-driven
  graph traversal mode** in Zodiac (GAP-8) (`dynamic-graph-query-generation`,
  `multi-hop-questions`); use the undefined `filters?` param (§4.3.2) for
  relationship filtering (`multi-hop-questions`).
- **NICE TO HAVE / PARKED** — a **scoped, lazy, local community-summary surface**
  (wiki-level/per-target) is the D2-compatible SHOULD; the full hierarchical
  GraphRAG pipeline stays PARKED for cost/latency (`community-summaries`, GAP-9);
  Mystery topic→reference model (`knowledge-graph-model`).
- **DISCARD** — none.

---

## 2. Summary of the source techniques

1. **Vector RAG failure modes.** The talk enumerates four reasons vector-only RAG is insufficient:
   - **Relationship data** — vectors don't capture relationships; a knowledge graph supplies relationship context that enables follow-up questions and supplies domain knowledge.
   - **Similarity ≠ relevance** — embedding similarity is not the same as relevance (worked example: a person with comment access appears frequently in docs and thus looks like a doc-team member to a vector search, regardless of whether they are).
   - **Explainability** — vector retrieval lacks explainability; governance questions are more easily answered by sourcing and tracing.
   - **Maturity** — vector systems' maturity is not on par with existing database systems.
2. **Relationship data.** The value of relationship data is realized through **filtering**, **multi-step query data**, and **multi-hop questions**.
3. **The knowledge-graph model.** A knowledge graph is **noun-focused nodes**, **relationship-focused edges**, and **properties** that can be attached to both nodes and edges.
4. **Graph RAG.** RAG where the retrieval path also includes a knowledge graph — using **both vector and graph layer methods**.
5. **Graph RAG techniques.** The technique family: **graph-enhanced vector search**, **dynamic graph query generation**, **parent-child retrievers**, **community summaries**, and **graph enrichments**.

---

## 3. Per-tool application analysis

### 3.1 Astrographer (PRIMARY) — the Provident graph IS the knowledge graph

**Current contract.** `docs/specs/astrographer.md` pins Astrographer as a Graphical RAG engine + document store/wiki. Documents are Provident graphs (nodes + edges, §4.1.1); node kinds are `content` / `fact` / `reference` (§4.2.1); reference modes are `link` / `embed` (§4.2.2); the consistency invariant (§4.2.3) enforces that a `fact`'s canonical `value` is the single source of truth; the RAG surface consumes Incanter over HTTP (§4.3) with a `source: 'local'|'incanter'|'zodiac'` union (§4.3.2); the MCP surface is §4.5; fail-states are §6.

**How the talk maps onto it.** Astrographer is the suite's most direct realization of the talk — its Provident graph **is** the knowledge graph the talk describes:

- **The Provident graph = the knowledge-graph model.** `fact` nodes are the "noun-focused" nodes (a fact has a `factKey` and a canonical `value`, §4.2.1); edges carry relationships (§4.1.1); `reference`/`link`/`embed` nodes are the "relationship-focused" edges (§4.2.2); properties attach to nodes and edges (document metadata `title`/`tags`/`author`, §4.1.1). **Already in the spec.**
- **The `reference`→`fact` topological resolution = the relationship data.** A `link` resolves live to its target; an `embed` snapshots the target's value (§4.2.2). This is the "relationship context that allows for follow-up questions" — a follow-up question can traverse from a referencing document to the canonical fact. **Already in the spec** (§4.2.2, §4.2.5 cross-wiki).
- **The consistency invariant (§4.2.3) = the "similarity ≠ relevance" fix.** This is the talk's core insight made mechanical: a `fact` is resolved by **exact reference** (a `link`/`embed` points at a specific `factKey`), not by embedding similarity. A vector search might surface a semantically-similar-but-wrong node; Astrographer's reference resolution surfaces the exact canonical fact. The publish gate (`UnresolvedReference` on `BROKEN`/`STALE`, §4.1.3) enforces that published docs are consistent. **Already in the spec** — this is the strongest mapping in the suite.
- **Explainability = the consistency report + provenance.** `getConsistencyReport(wikiId)` (§4.2.3) returns every `{documentId, nodeId, kind, state, target}` reference — a machine-readable "what is consistent / what is not" surface. This is the talk's "governance questions answered by sourcing and tracing" made concrete: an agent can trace a fact to its canonical source and see every reference to it. **Already in the spec** (§4.2.3, MCP `get_consistency_report` §4.5.1).

**Graph RAG technique mapping (each → a contract element, with disposition):**

| Talk technique | Astrographer contract element | Disposition |
| --- | --- | --- |
| **Graph-enhanced vector search** | The hybrid `local`/`incanter`/`zodiac` sources in `ragQuery`/`ragStream` (§4.3.2) — graph-aware retrieval over the Provident graph plus Incanter's graph-tension + vector hybrid (`incanter.md` §4.8) and Zodiac's pre-graphed data. | **Already in the spec** (the `source` union is provisional pending Zodiac, §4.3.2 note). |
| **Parent-child retrievers** | Document→node chunking: a Document is a Provident graph of nodes (§4.1.1); retrieval returns `{documentId, nodeId, snippet}` (§4.3.2) — retrieve the node, return the parent document context. Incanter's dynamic chunking (`incanter.md` §4.6) is the child-chunk boundary. | **Partially in the spec** (node-level results); a parent-context return (the full document or a node-cluster) is a **proposed spec change** to §4.3.2. |
| **Community summaries** | Wiki-level / document-cluster aggregation. No contract element exists — `getConsistencyReport` is per-reference, not a summary. | **Proposed spec change** (see §5, GAP-9). |
| **Graph enrichments** | The cross-link/embed mechanism (§4.2) — a `reference`/`embed` enriches a document with a canonical fact's value; cross-wiki references (§4.2.5) enrich across wikis. | **Already in the spec** (§4.2). |
| **Dynamic graph query generation** | Multi-hop / query-generated graph traversal. The RAG surface (§4.3.2) returns flat top-K results; it does not traverse edges or generate a multi-hop path. | **Proposed spec change** (see §5, GAP-7). |

**Net.** Astrographer already implements the talk's two strongest ideas — the knowledge-graph model (Provident graph) and the "similarity ≠ relevance" fix (exact-reference consistency). The gaps are the graph-RAG *technique* layer: multi-hop traversal, parent-context return, and community summaries are not yet contract elements.

### 3.2 Zodiac (PRIMARY) — the pre-graphing + vector-embedding pipeline IS graph RAG

**Current contract.** `docs/specs/zodiac.md` pins Zodiac as the backend/remote RAG companion: web crawlers (§4.1), a pre-graphing + vector-embedding pipeline (§4.2), a query-reply surface with `mode: graph|vector|hybrid` (§4.3), RAG store states `EMPTY`/`READY`/`STALE`/`REFRESHING` (§4.2), and fail-states (§6).

**How the talk maps onto it.** Zodiac is the suite's second direct realization of the talk — its pipeline is graph RAG by construction:

- **The pre-graphing + vector-embedding pipeline (§4.2) = graph RAG.** Zodiac parses crawled raw material into a graph (nodes + edges) AND embeds it into a vector store. This is the talk's "use both vector and graph layer methods" — the pre-graphed form is the graph layer, the vector store is the vector layer. **Already in the spec** (§4.2).
- **`mode: graph|vector|hybrid` (§4.3.1) = graph-enhanced vector search.** The `hybrid` mode returns a merged result combining graph and vector matches (§4.3.2) — the talk's graph-enhanced vector search. **Already in the spec** (§4.3.1, §4.3.2).
- **The RAG store states (`EMPTY`/`READY`/`STALE`/`REFRESHING`, §4.2) = the maturity/freshness concern.** The talk's "maturity not on par with existing database systems" is addressed by making freshness a first-class contract state: `STALE` data is flagged `stale: true` (§4.3.2, §6.11), `REFRESHING` keeps prior data queryable, and a refresh re-trigger (§4.1.2) updates the store. This is the "bad data becomes bad actions" mitigation made mechanical (consistent with `docs/research/docs-as-product-research-notes.md` §3.3). **Already in the spec** (§4.2, §4.3.2).
- **Explainability = `sourceCrawlId` provenance.** Every embedded item carries a reference to its source crawl id (§4.2); every query result carries `sourceCrawlId` (§4.3.2). This is the talk's "sourcing and tracing" — an agent can trace a result back to the crawl that produced it. **Already in the spec** (§4.2, §4.3.2).

**Graph RAG technique mapping (each → a contract element, with disposition):**

| Talk technique | Zodiac contract element | Disposition |
| --- | --- | --- |
| **Graph-enhanced vector search** | `mode: graph|vector|hybrid` (§4.3.1); `hybrid` merges graph + vector matches (§4.3.2). | **Already in the spec** (§4.3). |
| **Dynamic graph query generation** | `mode: graph` returns the pre-graphed subgraph for the target (§4.3.2) — a **static** subgraph dump, not a query-generated traversal or multi-hop path. | **Proposed spec change** (see §5, GAP-8). |
| **Parent-child retrievers** | The pre-graphing step (§4.2) parses raw material into nodes/edges; a query could retrieve a node and return its parent crawl/document context. | **Partially in the spec** (node-level results); parent-context return is a **proposed spec change**. |
| **Community summaries** | No contract element — Zodiac returns graph/vector items, not aggregated summaries of a crawled corpus. | **Proposed spec change** (parked; see §5). |
| **Graph enrichments** | The pre-graphing step (§4.2) enriches raw material into a graph (e.g. a dependency package's metadata, dependencies, and their relationships). | **Already in the spec** (§4.2). |

**Net.** Zodiac already implements the graph-RAG pipeline (pre-graphing + vector-embedding) and the graph-enhanced vector search (`mode: hybrid`). The gaps are dynamic graph query generation (its `mode: graph` is static) and community summaries.

### 3.3 Incanter — the prototype engine boundary

**Current contract.** `docs/specs/incanter.md` pins Incanter as the prototype Rust Graph-RAG engine: document-graph construction, keyword scoring/selection, spring-tension dynamic chunking (§4.6), and a hybrid graph-tension + vector query (`POST /v1/query`, §4.8).

**How the talk maps onto it.** Incanter is the engine Astrographer consumes over HTTP (edge #14, `astrographer.md` §5.3). The talk's graph-RAG techniques split across the engine boundary:

- **Graph-enhanced vector search = Incanter's existing hybrid.** `POST /v1/query` fuses dense vector similarity with graph-tension proximity (`vector_weight = 0.6`, `graph_weight = 0.4`, §4.8). This is the graph+vector hybrid axis. **Already in the spec** (§4.8).
- **Parent-child retrievers = the engine's dynamic chunking.** Incanter's spring-tension dynamic chunking (§4.6) produces semantic chunk boundaries; the parent-child pattern (retrieve a child chunk, return the parent document) is a natural engine-side capability. **Partially in the spec** (chunking exists; parent-context return is not pinned).
- **Dynamic graph query generation / multi-hop = an engine-boundary question.** The talk's "multi-hop questions" and "dynamic graph query generation" belong in the engine (Incanter) or the Astrographer client. Per the existing handoffs, multi-query fan-out belongs **inside Incanter** (GAP-1) and three-way lexical fusion belongs **inside Incanter** (GAP-2). A multi-hop graph-traversal mode would follow the same boundary rule: it is an Incanter-side query-time capability, not an Astrographer HTTP loop. **Proposed spec change** (see §5, GAP-7 — the traversal mode is an Incanter-repo change surfaced through Astrographer's `ragQuery`).

**Net.** Incanter already implements the graph+vector hybrid and dynamic chunking. The talk's multi-hop/dynamic-query-generation techniques are engine-boundary additions that follow the GAP-1/GAP-2 precedent (Incanter-repo changes, surfaced through Astrographer's single-call RAG surface).

### 3.4 Familiar — retrieval quality for the assistant

**Current contract.** `docs/specs/familiar.md` pins Familiar as the smart-assistant harness: knowledge retrieval delegates to Astrographer (F1, §4.3.1), research routes through Astrographer's RAG surface (F4, §4.3.3), and cross-instance search uses Solomon (F7, §4.3.5). Its memory store is a facts table (§4.1.3).

**How the talk maps onto it.** Familiar is a *consumer* of graph RAG, not a producer:

- **Graph RAG improves retrieval quality for the assistant.** Familiar's knowledge/research edges (F1, F4) retrieve through Astrographer's graph-aware RAG surface (`astrographer.md` §4.3). The talk's "relationship context that allows for follow-up questions" maps directly: a user's follow-up question can traverse the Provident graph (reference→fact) rather than re-embedding. **Already in the spec** (F1/F4 route through Astrographer).
- **The "similarity ≠ relevance" caution for assistant memory.** Familiar's memory store is a facts table (§4.1.3) with a candidate-fact pipeline (§4.1.3.1) and a derived profile summary (§4.1.3.2). The talk's caution applies: a vector search over memory could surface a semantically-similar-but-wrong fact. The facts table's exact `fact_id` addressing and the deterministic validation gate (§4.1.3.1) are the "exact reference, not embedding similarity" fix — consistent with the agent-memory research (`docs/research/agent-memory-research-notes.md` §3.1). **Already in the spec** (§4.1.3).
- **Explainability = provenance.** Familiar's facts carry `provenance` (`{conversation_id, message_id}`, §4.1.3) and the candidate pipeline is fail-closed with machine-actionable rejection reasons (§4.1.3.1). This is the talk's "sourcing and tracing" applied to assistant memory. **Already in the spec** (§4.1.3).

**Net.** Familiar is a consumer; the talk's techniques are realized through Astrographer (retrieval) and its own facts-table model (exact-reference memory). No new Familiar contract change is required by this research.

### 3.5 Solomon — cross-instance search + explainability

**Current contract.** `docs/specs/solomon.md` pins Solomon as the decentralized peer-network search: query fan-out and result aggregation (§4.2), with results carrying `{peerId, instanceType, itemId, snippet}` (§4.2.2).

**How the talk maps onto it.** Solomon is a cross-instance search surface over the graph-RAG tools:

- **Graph-aware retrieval across instances.** Solomon fans a query out to peers and aggregates results (§4.2.1). The talk's "relationship data" applies at the cross-instance level: a result from an Astrographer instance is a node in a Provident graph; a result from a Zodiac instance is a pre-graphed item. **Already in the spec** (§4.2).
- **Explainability = sourcing/tracing.** The talk's "governance questions answered by sourcing and tracing" maps to Solomon's result provenance: every result carries its `peerId` and `instanceType` (§4.2.2), so a cross-instance result can be traced to its source peer and instance. **Already in the spec** (§4.2.2).
- **The "similarity ≠ relevance" caution.** Solomon's Phase-1 aggregation groups by source peer and orders within a peer by the peer's own relevance ranking (§4.2.2); cross-peer ordering is unpinned (§7.4). The talk's caution applies: a cross-instance result that is semantically similar but from the wrong instance/peer is a relevance failure. The `partial: true` flag and `unreachablePeers` list (§4.2.2) are the maturity/freshness concern made explicit. **Already in the spec** (§4.2.2, §7.4).

**Net.** Solomon already provides result provenance (peerId/instanceType) that satisfies the talk's explainability concern. The cross-peer ordering gap (§7.4) is the "similarity ≠ relevance" risk at the aggregation layer.

### 3.6 Mystery — crosslinking info references by post topic

**Current contract.** `docs/specs/mystery.md` does **not exist** (confirmed — the file is absent). Mystery is a PLANNED message board that crosslinks information references from other apps by post topic (`docs/architecture-overview.md` §4.3; `docs/pending.md` #12 — "Mystery crosslink resolution" is an OPEN item).

**How the talk maps onto it.** Mystery is the suite's clearest *relationship-data* consumer:

- **The knowledge-graph relationship model (topic→reference edges).** The talk's "noun-focused nodes / relationship-focused edges" maps directly to Mystery's core: a post topic is a node, and the crosslinked information references (Astrographer documentation, Astral pages) are relationship edges from the topic to the reference. `docs/pending.md` #12 pins this as the "topic→reference resolution and the reference embed format" — the talk's relationship-data model is the natural design for it. **Proposed spec change** (the Mystery contract does not exist yet; this research informs it).
- **Multi-hop questions.** A debug thread that links relevant web-app documentation is a multi-hop traversal: post topic → reference → canonical fact. The talk's "multi-hop questions" is the retrieval pattern Mystery would enable. **Proposed spec change** (informs the future Mystery contract).

**Net.** Mystery is not yet specified; the talk's knowledge-graph relationship model (topic→reference edges) is the natural design basis for `docs/pending.md` #12. This is a design-guidance input to the future Mystery contract, not a change to an existing contract.

### 3.7 Astral — the published wiki + community-summary-style aggregation

**Current contract.** `docs/specs/astral.md` pins Astral as the locally-runnable webhost: it receives Provident-graph pushes from Astrographer (edge #2, §4.2) and serves PUBLISHED units at `GET /<slug>` (§4.1).

**How the talk maps onto it.** Astral is the published-wiki surface of the graph-RAG tools:

- **Community-summary-style aggregation of published docs.** The talk's "community summaries" (aggregating a graph into higher-level summaries) maps to Astral's role as the wiki host: a wiki-level summary or index of PUBLISHED units is a community-summary-style aggregation. The docs-as-product research already recommends `llms.txt`/`llms-full.txt` + markdown-copyable pages on Astral (`docs/research/docs-as-product-research-notes.md` §3.2; GAP-5). A community-summary surface would be a higher-level aggregation on top of that. **Proposed spec change** (see §5, GAP-9 — the community-summary surface is an Astrographer/Astral concern).
- **Explainability = provenance.** Astral's hosted units carry `source tool` and `content hash` metadata (§4.1), so a published page can be traced to its pushing tool and content revision. **Already in the spec** (§4.1).

**Net.** Astral is a consumer/host; the talk's community-summary technique is the one additive surface (a wiki-level aggregation of published docs), which is a proposed spec change.

---

## 4. Recommendation tiers

Grounded in the four design constraints (D1 open-source AGPL-3.0, D2 local-first, D3 interconnection, D4 MCP-GUI parity).

### MUST HAVE
- **Astrographer — keep the §4.2 consistency invariant + publish gate non-negotiable.** The exact-reference resolution of `fact`/`reference`/`embed` (§4.2.1–§4.2.3) is the suite's realization of the talk's "similarity ≠ relevance" fix. It is already in the spec; this research confirms it is the highest-value graph-RAG element. **Already in the spec** — no change.
- **Astrographer — keep `getConsistencyReport` (§4.2.3) as the explainability surface.** The consistency report + provenance is the talk's "sourcing and tracing" made mechanical. **Already in the spec** — no change.
- **Zodiac — keep the pre-graphing + vector-embedding pipeline (§4.2) and `mode: graph|vector|hybrid` (§4.3).** This is graph RAG by construction and graph-enhanced vector search. **Already in the spec** — no change.
- **Zodiac — keep the RAG store states (`EMPTY`/`READY`/`STALE`/`REFRESHING`, §4.2) and `sourceCrawlId` provenance (§4.3.2).** These address the talk's maturity/freshness and explainability concerns. **Already in the spec** — no change.

### SHOULD HAVE
- **Astrographer — add a parent-context return to the RAG surface (§4.3.2).** Retrieve a node, return the parent document (or a node-cluster) as context — the talk's parent-child retriever pattern. Low cost (the document is already the unit, §4.1.1), high value for follow-up questions. **Proposed spec change** to `astrographer.md` §4.3.2.
- **Astrographer/Incanter — add a multi-hop / dynamic graph query mode.** The talk's "multi-hop questions" and "dynamic graph query generation" have no contract element. Per the GAP-1/GAP-2 precedent, the traversal mode is an **Incanter-repo change** surfaced through Astrographer's single-call `ragQuery` (§4.3.2). **Proposed spec change** (see §5, GAP-7).
- **Zodiac — add a query-driven graph traversal mode.** `mode: graph` (§4.3.2) is a static subgraph dump; a query-generated traversal/multi-hop path would realize the talk's "dynamic graph query generation" over the crawled graph. **Proposed spec change** (see §5, GAP-8).

### NICE TO HAVE / PARKED
- **Community summaries (wiki-level aggregation).** The talk's "community summaries" has no contract element in Astrographer or Astral. A wiki-level summary derived from the Provident graph (or a summary of PUBLISHED units on Astral) is valuable but gated on the reachability core (GAP-4/5/6) and tool implementation. **Parked** (see §5, GAP-9).
- **Mystery topic→reference relationship model.** The talk's knowledge-graph relationship model informs the future Mystery contract (`docs/pending.md` #12). **Parked** until the Mystery contract exists.

### DISCARD
- **None.** The talk contains no technique that conflicts with the suite's D1–D4 constraints. Every technique maps to an existing or proposed contract element. The "maturity not on par with existing database systems" concern is a caution, not a discard — it argues for the suite's local-first, well-specified RAG store states (Zodiac §4.2) over ad-hoc vector systems.

---

## 5. Handoffs / gaps

The following are **genuinely new** graph-RAG-specific gaps, distinct from the existing GAP-1..GAP-6 (which are NOT duplicated here). Each is a proposed spec change or parked item; the orchestrator owns the tracker updates.

- **GAP-7 — Astrographer RAG surface has no multi-hop / dynamic graph query mode.** The talk's "multi-hop questions" and "dynamic graph query generation" are not represented in `ragQuery`/`ragStream` (`astrographer.md` §4.3.2), which return flat top-K results. The Provident graph (nodes + edges) is stored but the RAG surface does not traverse edges. **Proposed fix shape (owned by Incanter, surfaced through Astrographer):** add a multi-hop/graph-traversal query mode to Incanter's `POST /v1/query` (returning a subgraph path rather than flat chunks), following the GAP-1/GAP-2 engine-boundary precedent; Astrographer's `ragQuery` passes the mode through unchanged. **Spec change** to `astrographer.md` §4.3.2 + an Incanter-repo handoff.
- **GAP-8 — Zodiac `mode: graph` is a static subgraph dump, not dynamic graph query generation.** The talk's "dynamic graph query generation" is not represented; `mode: graph` (`zodiac.md` §4.3.2) returns the pre-graphed subgraph for the target, with no query-generated traversal or multi-hop path over the crawled graph. **Proposed fix shape (owned by Zodiac):** add a query-driven graph traversal mode that generates a subgraph path from the query, not just a static target subgraph. **Spec change** to `zodiac.md` §4.3.
- **GAP-9 — No community-summary / wiki-level aggregation surface.** The talk's "community summaries" has no contract element in Astrographer or Astral. `getConsistencyReport` (`astrographer.md` §4.2.3) is per-reference; there is no wiki-level or document-cluster summary. **Proposed fix shape (owned by Astrographer/Astral):** add a community-summary-style aggregation surface (e.g. a wiki-level summary derived from the Provident graph, or a summary of PUBLISHED units on Astral). **Parked** — gated on the reachability core (GAP-4/5/6) and tool implementation.

**Note on disposition.** GAP-7 and GAP-8 are **spec changes** (they extend existing RAG surfaces). GAP-9 is a **parked item** (a new surface, gated on implementation). None of these duplicate GAP-1..GAP-6: GAP-1 is multi-query fan-out, GAP-2 is lexical fusion, GAP-3 is Solomon assistant-memory, GAP-4/5/6 are docs-as-product reachability — all distinct from the graph-RAG traversal/summary gaps above.

---

## 6. Source URL list

The web-grounded bulk-research run (`2026-09-08-graph-rag-when-vectors`) will supply the full cited URL list in its reports under `docs/research/2026-09-08-graph-rag-when-vectors/reports/`. Preliminary URLs retrieved this session (see §1a):

- https://graphrag.com/reference/graphrag/graph-enhanced-vector-search/ (graph-enhanced vector search)
- https://microsoft.github.io/graphrag/ (Microsoft GraphRAG)
- https://www.microsoft.com/en-us/research/publication/from-local-to-global-a-graph-rag-approach-to-query-focused-summarization/ (community summaries)
- https://github.com/langchain-ai/langchain/blob/85a5a04210452aec2eb3a06a02961c8fefd5c8b6/libs/langchain/langchain_classic/retrievers/parent_document_retriever.py (parent-child retrievers)
- https://www.shyankdev.us/blogs/advanced-rag-hierarchical-node-parsing-parent-child-retrievers-and-metadata-pre-filtering (parent-child retrievers)
- https://arxiv.org/html/2605.00845v1 (dynamic graph query generation)
- https://www.sciencedirect.com/science/article/abs/pii/S092523122601492X (DynKGRAG)
- https://arxiv.org/html/2412.18644 (DynaGRAG)
- https://www.infoq.com/articles/vector-search-hybrid-retrieval-rag/ (vector RAG failures)
- https://www.broadnet.ai/blog/knowledge-graphs-vs-vector-rag/ (vector RAG failures)
- https://tianpan.co/blog/2026/04/20/knowledge-graphs-vs-vector-search-retrieval (vector RAG failures)
- https://redis.io/en/blog/knowledge-graph-rag-structured-retrieval-ai-agents/ (vector RAG failures)
- https://enison.ai/en/blog/knowledge-graph-rag-implementation (knowledge-graph model)
- https://arxiv.org/abs/2603.22340v1 (knowledge-graph model)
- https://machinalearning.com/lessons/rag_retrieval/knowledge-graph-rag (knowledge-graph model)
- https://www.semantic-web-journal.net/system/files/swj4027.pdf (Graph RAG in the wild)
- https://arxiv.org/html/2606.06003 (graph-augmented retrieval structural analysis)
- https://www.tigergraph.com/blog/advanced-rag-techniques-naive-to-hybrid-graphrag/ (advanced graph RAG)
- https://www.cs.purdue.edu/homes/csjgwang/pubs/SIGMOD25_TigerVector.pdf (vector search in graph DBs)

**Suite context cited throughout:** `docs/specs/astrographer.md` (§4.1, §4.2, §4.3, §4.5, §5.3, §6), `docs/specs/zodiac.md` (§4.1, §4.2, §4.3, §4.5, §6), `docs/specs/incanter.md` (§4.6, §4.8, §5), `docs/specs/familiar.md` (§4.1.3, §4.3.1, §4.3.3, §4.3.5), `docs/specs/solomon.md` (§4.2, §7.4), `docs/specs/astral.md` (§4.1, §4.2), `docs/architecture-overview.md` (§4.2, §4.3, §5, §6), `docs/decisions.md` (D1–D4, INCANTER-DISPOSITION, GRAPH-PUSH-FORMAT, D2-CLARIFICATION), `docs/pending.md` (#2, #12, D3, D4, D5), `docs/defects.md` (GAP-1..GAP-6), `docs/HANDOFF.md` (Rounds 2–4), `docs/research/hybrid-search-research-notes.md`, `docs/research/agent-memory-research-notes.md`, `docs/research/docs-as-product-research-notes.md`.
