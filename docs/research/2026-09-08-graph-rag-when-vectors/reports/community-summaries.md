# Community Summaries (GraphRAG-style aggregation)

- **Topic:** Community summaries — the GraphRAG technique where the knowledge graph is hierarchically clustered into communities and each community is summarized, enabling global/theme-level queries.
- **Date:** 2026-09-08
- **Tier:** SHOULD
- **Report slug:** `community-summaries`
- **Primary focus:** Astrographer, Zodiac. **Secondary:** Incanter, Familiar, Solomon, Mystery, Astral.
- **Status:** Web-grounded research report (documentation deliverable only — no spec/code/test changes). Companion to `graph-rag-astrographer.md` (tier MUST) and `graph-rag-suite-consumers.md` (tier SHOULD) in the same directory.

---

## §1 What the technique is (web-grounded, cited)

**Community summaries** are the aggregation half of Microsoft's GraphRAG. The
pipeline is: (1) build a knowledge graph from the corpus, (2) **detect
communities** — clusters of densely-connected nodes — using a hierarchical
community-detection algorithm (Leiden), (3) **summarize each community** with an
LLM into a short "community report," producing a hierarchy of summaries from
fine-grained (leaf) to coarse (root), and (4) at query time, answer **global /
theme-level questions** by reading the community summaries rather than the raw
graph. This is GraphRAG's *global search* path, contrasted with *local search*
(which retrieves specific entities/relationships near the query).

The canonical reference is Microsoft's *From Local to Global: A GraphRAG
Approach to Query-Focused Summarization*
([arXiv:2404.16130](https://arxiv.org/pdf/2404.16130)), which introduced
community detection + hierarchical summarization and showed it answers
"global sensemaking" questions (e.g. "what are the main themes of this corpus?")
that vector-only retrieval cannot. The official docs pin the two query paths —
[global search](https://github.com/microsoft/graphrag/blob/main/docs/query/global_search.md)
(community summaries) vs [local search](https://mintlify.wiki/microsoft/graphrag/concepts/retrieval-methods) —
and the [community-detection](https://microsoft-graphrag.mintlify.app/concepts/community-detection)
concept page describes the Leiden-based hierarchical clustering. Microsoft later
added [dynamic community selection](https://www.microsoft.com/en-us/research/blog/graphrag-improving-global-search-via-dynamic-community-selection/)
to make global search cheaper by selecting only the communities relevant to the
query at query time.

**The cost caveat is central.** Community summarization is the most expensive
part of GraphRAG: it is an LLM pass over every community at every hierarchy
level, and it is paid at **index time** (before any query). Microsoft's own
[GraphRAG costs explainer](https://techcommunity.microsoft.com/blog/azure-ai-foundry-blog/graphrag-costs-explained-what-you-need-to-know/4207978)
and independent engineering analyses
([indexing cost cliff](https://www.bestaiweb.ai/indexing-cost-token-blowup-and-the-hard-engineering-limits-of-graphrag-at-scale/),
[12-hour benchmark](https://bestin-it.com/graphrag-benchmark-sec-filings/))
document the token blow-up. The strongest counter-evidence is **LightRAG**, which
**deliberately skips the community-summarization pass** and instead uses the
graph structure + dual-level (low/high) retrieval directly; the
[LightRAG paper](https://aclanthology.org/2025.findings-emnlp.568.pdf) and
[repo](https://github.com/HKUDS/LightRAG/) report matching or beating GraphRAG on
retrieval accuracy at roughly **1–2 orders of magnitude lower token cost**
(up to ~100×), precisely because it avoids community summarization. A Neo4j
engineering deep-dive ([Under the Covers With LightRAG](https://collabnix.com/neo4j/2025/05/20/under-the-covers-with-lightrag-extraction-and-retrieval/))
confirms the mechanism. The systematic evaluation
([RAG vs. GraphRAG](https://arxiv.org/abs/2502.11371v3)) likewise finds
GraphRAG's community-summary strength is on **global, relationship-heavy,
multi-hop questions**, while plain RAG wins on cost and simple fact lookup.

**When it is worth it.** The [when-to-use-each-retrieval-path](https://theneuralbase.com/graphrag/learn/intermediate/when-to-use-each-retrieval-path/)
guidance and the evaluation literature converge: community summaries pay off when
the corpus is large, the questions are **theme/global** ("summarize the
architecture of this codebase", "what are the recurring topics?"), and the
corpus changes slowly (so the index-time cost amortizes). They are poor value
for small corpora, fast-changing corpora, or point-fact questions. Local-first
deployments can run the pipeline fully self-hosted
([graphrag-local-ollama](https://github.com/theaisingularity/graphrag-local-ollama),
[local multi-tenant GraphRAG](https://joeywang.github.io/posts/rag-hub/),
[LlamaIndex GraphRAG cookbook](https://developers.llamaindex.ai/python/examples/cookbooks/graphrag_v1/)),
but the LLM cost is paid locally and is still real.

---

## §2 How it applies to Astrographer

Astrographer is a **graphical RAG engine + document store/wiki** over **Provident
graphs** (`docs/specs/astrographer.md` §2, §4.1.1). Its graph is a **reference
graph**, not a dense entity graph:

- **Nodes** (`astrographer.md` §4.2.1): `content`, `fact`, and `reference` node
  kinds. The `fact` node is the single source of truth.
- **Edges** (`astrographer.md` §4.2.2): `link` and `embed` reference modes —
  relationship edges from a referencing document to a target `fact`/node.
- **Consistency** (`astrographer.md` §4.2.3): the invariant that a `fact`'s
  canonical `value` is the single source of truth, enforced by the publish gate
  (`UnresolvedReference` on `BROKEN`/`STALE`) and surfaced by
  `getConsistencyReport(wikiId)` (§4.1.3, MCP `get_consistency_report` §4.5.1).

**Mapping the technique to contract elements:**

| Community-summary element | Astrographer contract element | Status |
| --- | --- | --- |
| **Community detection** | The wiki's reference graph (§4.2) — documents cluster by shared `fact`/`reference` targets. A "community" here is a **document cluster** (documents that embed/reference the same facts), not a dense entity cluster. | **Proposed spec change** (a wiki-level aggregation surface). |
| **Community summarization** | No contract element exists. `getConsistencyReport` (§4.2.3) is **per-reference**, not a summary; `ragQuery`/`ragStream` (§4.3.2) return flat top-K results, not aggregated themes. | **Proposed spec change** (see §5). |
| **Global / theme query** | The RAG surface (§4.3.2) has no global/theme mode — it is local top-K retrieval. A "summarize this wiki" query has no path. | **Proposed spec change** (see §5). |
| **Explainability / provenance** | Already strong: every `link`/`embed` is tracked to its target (§4.2.3); a community summary could carry the set of `documentId`/`factKey` it was derived from, giving the "sourcing and tracing" the source talk demands. | **Already in spec** (the substrate); extend to summaries. |

**Concrete fit.** Astrographer's defining value is **documentation consistency**
(§4.2). A community-summary surface is a natural, low-cost expression of the
technique here: **a wiki-level / document-cluster summary** derived from the
Provident reference graph — e.g. "these N documents all embed fact X, so they
form a cluster about X" — which answers theme-level questions ("what is this wiki
about?") that the flat `ragQuery` cannot. Because the graph already records every
reference (§4.2.3), the cluster membership is deterministic and cheap to compute
locally (D2); the only LLM cost is the summary text itself, and it can be
**lazy/on-demand** (generated on a theme query) rather than a full index-time
pass. This is the D2-compatible, scoped form of the technique.

**What is NOT a good fit.** The full Microsoft GraphRAG pipeline — Leiden
hierarchical community detection over a dense entity graph + an LLM summary pass
over every community at every level — is **cost-prohibitive and mis-shaped** for
Astrographer: the reference graph is sparse (documents ↔ facts), not a dense
entity graph, and the index-time LLM cost conflicts with D2 local-first
cost/latency. The prior graph-RAG research already marked community summaries
**PARKED** for Astrographer on exactly these grounds
(`docs/research/2026-09-08-graph-rag-when-vectors/reports/graph-rag-astrographer.md`
§2, §5). This report refines that: the **scoped wiki-level summary** is SHOULD;
the **full hierarchical pipeline** stays PARKED.

---

## §3 How it applies to Zodiac

Zodiac is the **backend/remote RAG companion** that pre-graphs + vector-embeds
crawled data (`docs/specs/zodiac.md` §2, §4.2) and replies to Astrographer
queries with `mode: graph|vector|hybrid` (§4.3.1, §4.3.2). It is the suite's
clearest Graph RAG engine.

**Mapping the technique to contract elements:**

| Community-summary element | Zodiac contract element | Status |
| --- | --- | --- |
| **Community detection** | The pre-graphed data (§4.2) — e.g. a dependency package's metadata, its dependencies, and their relationships — is a real entity graph, closer to GraphRAG's shape than Astrographer's reference graph. Communities = clusters of related crawled entities. | **Proposed spec change** (a per-target or per-corpus summary). |
| **Community summarization** | No contract element. `zodiac_query` (§4.3.2) returns graph/vector items, not aggregated summaries of a crawled corpus. | **Proposed spec change** (see §5). |
| **Global / theme query** | `mode: graph` returns the pre-graphed subgraph for a target (§4.3.2) — a **static target subgraph**, not a corpus-level theme summary. | **Proposed spec change** (see §5). |
| **Explainability / provenance** | Already strong: every embedded item carries `sourceCrawlId` (§4.2, §4.3.2); a community summary could carry the set of `sourceCrawlId`s it was derived from. | **Already in spec** (the substrate); extend to summaries. |

**Concrete fit.** Zodiac's pre-graphed data is the one place in the suite where
the graph is a **dense entity graph** (dependency packages and their
relationships), so community detection is more meaningful there than in
Astrographer's reference graph. A **per-target community summary** — "summarize
what this dependency package and its dependency graph do" — is a natural global
answer for a `mode: graph` query. However, Zodiac is **remote/backend by design**
(D2-CLARIFICATION, `zodiac.md` §4.4) and its value is non-local data; the
index-time LLM cost of full hierarchical summarization is a real managed-service
cost. The scoped, D2-compatible expression is a **lazy per-target summary**
generated on a theme query, not a full index-time community-summarization pass.

**What is NOT a good fit.** Full GraphRAG-style hierarchical community
summarization over the entire crawled corpus is cost-prohibitive for a remote
service whose data refreshes (RAG store states `STALE`/`REFRESHING`, `zodiac.md`
§4.2) — every refresh would invalidate the summaries. The prior research marked
community summaries **PARKED** for Zodiac on cost grounds
(`graph-rag-astrographer.md` §3). This report refines that: the **scoped
per-target summary** is SHOULD; the **full hierarchical pipeline** stays PARKED.

---

## §4 How it applies to the other suite consumers

- **Incanter** (`docs/specs/incanter.md`): Incanter is the prototype Graph-RAG
  engine Astrographer consumes over HTTP (`incanter.md` §2, §5). Its document
  graph (`incanter.md` §4.4) is a **per-document token-transition multigraph**,
  not a corpus-level community graph; its tension-space model (`incanter.md`
  §4.6) is per-document. Community summaries are **not applicable** to Incanter's
  per-document structure — there is no corpus-level community graph to summarize.
  **PARKED / not applicable.**

- **Familiar** (`docs/specs/familiar.md`): Familiar is a **consumer** of graph
  RAG, not an engine. Its knowledge/research domains route through Astrographer's
  RAG surface (`familiar.md` §4.3.1, §4.3.3). A community-summary surface in
  Astrographer would surface to Familiar as a **global/theme answer** to a
  research question ("summarize what the wiki says about X") — a natural
  assistant capability. Its own memory store is a **facts table** (`familiar.md`
  §4.1.3), not a graph; the derived **profile summary** (§4.1.3.2) is a
  single-entity summary, not a community summary. **No direct work in Familiar**;
  it inherits the surface through Astrographer.

- **Solomon** (`docs/specs/solomon.md`): Solomon is cross-instance search over
  `zodiac|astral|astrographer` instances (`solomon.md` §4.2, §4.5.1). Its result
  aggregation (`solomon.md` §4.2.2) is flat (grouped by peer, ordered by the
  peer's own relevance). A **cross-instance community summary** — "summarize
  what all linked instances say about topic X" — would be a global aggregation
  over the peers' own summaries, but Phase-1 deliberately keeps cross-peer
  ordering unpinned (`solomon.md` §7.4). **NICE-TO-HAVE / PARKED** until Phase-1
  search semantics finalize.

- **Mystery** (`docs/specs/mystery.md` — **absent**; `docs/pending.md` #12):
  Mystery is a message board that crosslinks information references by post topic
  (`docs/architecture-overview.md` §4.3). Its **topic→reference resolution**
  (`docs/pending.md` #12) is a relationship-mapping problem; a **topic-level
  summary** of the crosslinked references (e.g. "here's what the linked docs say
  about this debug topic") is a community-summary-style aggregation over the
  topic's reference cluster. **Proposed when the Mystery contract exists** —
  PARKED until then.

- **Astral** (`docs/specs/astral.md`): Astral is the wiki/publishing host
  (`astral.md` §2). It receives Provident graphs (`astral.md` §4.2) and serves
  them (`astral.md` §4.1). The most natural community-summary surface in the
  suite is **Astral-side**: a **wiki-level index/summary of PUBLISHED units** —
  the docs-as-product research already recommends `llms.txt`/`llms-full.txt` +
  markdown-copyable pages on Astral (GAP-5, `docs/defects.md`). A community
  summary is a higher-level aggregation on top of that: a theme-level overview of
  the published wiki. **Proposed spec change** (GAP-5 + a summary layer).

---

## §5 Recommendation for the suite

Grounded in D1–D4. The full Microsoft GraphRAG community-summarization pipeline
is **cost-prohibitive and mis-shaped** for the suite (D2 local-first cost/latency;
the suite's primary graph is a sparse reference graph, not a dense entity graph;
LightRAG's evidence that the pass is optional). But a **scoped, lazy, local
community-summary surface** is a genuine SHOULD that answers the source talk's
"global/theme questions" gap without the index-time cost cliff.

- **SHOULD — Astrographer: a wiki-level / document-cluster summary surface.**
  Add a global/theme query mode to the RAG surface (`astrographer.md` §4.3.2)
  that clusters documents by shared `fact`/`reference` targets (§4.2) and returns
  a **lazy, on-demand summary** of a cluster (e.g. "these documents all embed
  fact X"). Cluster membership is deterministic from the reference graph (§4.2.3)
  and cheap to compute locally (D2); the only LLM cost is the summary text, paid
  on a theme query, not at index time. Expose it on both GUI and MCP (D4) — a
  `rag_query` mode or a new `get_wiki_summary` MCP tool. **Proposed spec change**
  to `astrographer.md` §4.3/§4.5.

- **SHOULD — Zodiac: a lazy per-target community summary.** Add a summary mode
  to `zodiac_query` (`zodiac.md` §4.3.2) that summarizes a pre-graphed target's
  community (e.g. a dependency package + its dependency graph) on a theme query,
  carrying the `sourceCrawlId`s it was derived from (§4.2). Lazy (not index-time)
  so it survives the `STALE`/`REFRESHING` store states (§4.2). **Proposed spec
  change** to `zodiac.md` §4.3.

- **SHOULD — Astral: a wiki-level summary of PUBLISHED units.** Layer a
  theme-level overview of the published wiki on top of the docs-as-product
  reachability surface (GAP-5, `docs/defects.md`). This is the suite's most
  natural "global" surface — a human or agent reading the published wiki gets a
  summary of what it covers. **Proposed spec change** (GAP-5 + summary layer).

- **NICE-TO-HAVE / PARKED — Solomon cross-instance community summary.** A global
  aggregation over peers' summaries is valuable but gated on Phase-1 search
  semantics (`solomon.md` §7.4). **PARKED.**

- **PARKED — the full Microsoft GraphRAG hierarchical community-summarization
  pipeline** (Leiden detection + an LLM summary pass over every community at
  every level). Cost-prohibitive for D2 local-first; mis-shaped for the sparse
  reference graph; LightRAG's evidence shows it is optional. Revisit only if a
  large, slow-changing, dense-entity corpus and a global-question use case
  surface.

- **PARKED — Mystery topic-level summary.** When the Mystery contract exists
  (`docs/pending.md` #12), model its topic→reference clusters and add a
  topic-level summary. PARKED until the spec exists.

**Overall recommendation tier: SHOULD.** Community summaries are not a MUST for
the suite — the core graph-RAG value is already in the reference graph and the
consistency invariant (`astrographer.md` §4.2.3), and LightRAG demonstrates the
summarization pass is optional. But a **scoped, lazy, local** community-summary
surface (Astrographer wiki-level, Zodiac per-target, Astral published-wiki) is a
genuine SHOULD that closes the source talk's "global/theme questions" gap at
acceptable D2 cost, is additive (D3), and is exposable on both GUI and MCP (D4).
The full hierarchical pipeline stays PARKED.

---

## §6 Source URL list

1. https://arxiv.org/pdf/2404.16130 — Microsoft, *From Local to Global: A GraphRAG Approach to Query-Focused Summarization* (canonical; community detection + hierarchical summarization)
2. https://github.com/microsoft/graphrag/blob/main/docs/query/global_search.md — Microsoft GraphRAG, *global search* (community summaries)
3. https://microsoft-graphrag.mintlify.app/concepts/community-detection — Microsoft GraphRAG, *community detection* (Leiden hierarchical clustering)
4. https://mintlify.wiki/microsoft/graphrag/concepts/retrieval-methods — Microsoft GraphRAG, *retrieval methods* (local vs global)
5. https://www.microsoft.com/en-us/research/blog/graphrag-improving-global-search-via-dynamic-community-selection/ — Microsoft Research, *GraphRAG: Improving global search via dynamic community selection*
6. https://aclanthology.org/2025.findings-emnlp.568.pdf — *LightRAG: Simple and Fast Retrieval-Augmented Generation* (skips community summarization; ~1–2 orders of magnitude lower cost)
7. https://github.com/HKUDS/LightRAG/ — LightRAG repository (dual-level retrieval without community summaries)
8. https://collabnix.com/neo4j/2025/05/20/under-the-covers-with-lightrag-extraction-and-retrieval/ — Neo4j, *Under the Covers With LightRAG* (engineering deep-dive)
9. https://techcommunity.microsoft.com/blog/azure-ai-foundry-blog/graphrag-costs-explained-what-you-need-to-know/4207978 — Microsoft, *GraphRAG Costs Explained*
10. https://www.bestaiweb.ai/indexing-cost-token-blowup-and-the-hard-engineering-limits-of-graphrag-at-scale/ — *GraphRAG Indexing: Why the $33K Cost Cliff Hits*
11. https://bestin-it.com/graphrag-benchmark-sec-filings/ — *GraphRAG benchmark: what 12 hours of graph building actually buys*
12. https://arxiv.org/abs/2502.11371v3 — *RAG vs. GraphRAG: A Systematic Evaluation and Key Insights*
13. https://theneuralbase.com/graphrag/learn/intermediate/when-to-use-each-retrieval-path/ — *When to use each retrieval path* (global vs local)
14. https://github.com/theaisingularity/graphrag-local-ollama — *GraphRAG local with Ollama* (local-first deployment)
15. https://joeywang.github.io/posts/rag-hub/ — *Escaping the Cloud Token Trap: Building a Multi-Tenant Graph RAG System Locally*
16. https://developers.llamaindex.ai/python/examples/cookbooks/graphrag_v1/ — LlamaIndex, *GraphRAG Implementation* (community summaries cookbook)

**Suite context cited throughout:** `docs/specs/astrographer.md` (§4.1.1, §4.2, §4.2.3, §4.3.2, §4.5.1), `docs/specs/zodiac.md` (§2, §4.2, §4.3.1, §4.3.2, §4.4), `docs/specs/incanter.md` (§4.4, §4.6), `docs/specs/familiar.md` (§4.1.3, §4.1.3.2, §4.3.1, §4.3.3), `docs/specs/solomon.md` (§4.2.2, §7.4), `docs/specs/astral.md` (§2, §4.1, §4.2), `docs/architecture-overview.md` (§4.3, §5), `docs/decisions.md` (D1–D4, D2-CLARIFICATION), `docs/pending.md` (#12), `docs/defects.md` (GAP-5), `docs/research/lightrag-research-notes.md`, `docs/research/2026-09-08-graph-rag-when-vectors/reports/graph-rag-astrographer.md`, `graph-rag-suite-consumers.md`.
