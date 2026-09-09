# Hybrid Search (BM25 + Dense Vector + Fusion) — Research Notes

**Topic:** Hybrid Search for RAG — lexical (BM25/keyword) + dense-vector retrieval fused via Reciprocal Rank Fusion or weighted combination
**Date:** 2026-09-06
**Status:** Research / reference notes (not a behavior contract)
**Scope:** General technique + Astrographer-specific feasibility/synergy analysis (Auspicion Suite)

---

## 1. Original paper & primary documentation

| Item | Source |
| --- | --- |
| **RRF paper (the canonical fusion method)** | [Reciprocal rank fusion outperforms condorcet and individual rank learning methods](https://doi.org/10.1145/1571941.1572114) — Cormack, Clarke, Büttcher (SIGIR 2009). Author page: [cormack.uwaterloo.ca/cormacksigir09-rrf](https://cormack.uwaterloo.ca/cormacksigir09-rrf) |
| **Fusion-functions analysis** | [An Analysis of Fusion Functions for Hybrid Retrieval](https://arxiv.org/html/2210.11934v2) (arXiv 2210.11934) · [Pinecone research page](https://www.pinecone.io/research/an-analysis-of-fusion-functions-for-hybrid-retrieval/) |
| **Hybrid-search trade-offs (experimental)** | [Balancing the Blend: An Experimental Analysis of Trade-offs in Hybrid Search](https://www.vldb.org/pvldb/vol19/p1715-gao.pdf) (VLDB 2026) |
| **HybridRAG (graph + vector)** | [HybridRAG: Integrating Knowledge Graphs and Vector Retrieval Augmented Generation for Efficient Information Extraction](https://arxiv.org/html/2408.04948) |
| **HYBGRAG (textual + relational KBs)** | [HYBGRAG: Hybrid Retrieval-Augmented Generation on Textual and Relational Knowledge Bases](https://aclanthology.org/2025.acl-long.43.pdf) (ACL 2025) |
| **Elasticsearch RRF reference** | [Elasticsearch Reciprocal Rank Fusion API](https://www.elastic.co/docs/reference/elasticsearch/rest-apis/reciprocal-rank-fusion) |
| **Neo4j hybrid search (graph-native)** | [Neo4j — Hybrid Search](https://neo4j.com/developer/genai-ecosystem/hybrid-search/) |

**Origin / status.** Hybrid search is not a single paper; it is a mature, widely-deployed retrieval pattern. The two canonical pillars are (a) **lexical retrieval** (BM25, Robertson/Sparck Jones lineage) and (b) **dense vector retrieval** (embedding similarity). The standard fusion method is **Reciprocal Rank Fusion (RRF)**, introduced by Cormack et al. (SIGIR 2009) and since adopted by Elasticsearch, Weaviate, Qdrant, and most RAG frameworks. The technique is production-stable and well-documented.

---

## 2. Exact technical mechanism

### 2.1 The two retrieval legs

1. **Lexical / sparse leg (BM25).** A classic inverted-index keyword scorer. BM25 scores a document against a query by term frequency, inverse document frequency, and field-length normalization. It is **exact-match**: it only matches terms that literally appear in the query. It is cheap, deterministic, and excellent at IDs, technical terms, code, and rare proper nouns.
2. **Dense / vector leg.** Each chunk is embedded into a high-dimensional vector; the query is embedded with the same model; retrieval returns the nearest neighbors by cosine similarity (or another metric). It is **semantic**: it matches meaning even when the query and the document share no literal terms. It requires an embedding model and a vector index (HNSW/IVF or a vector store).

### 2.2 Fusion — combining the two ranked lists

The two legs each return a ranked list of candidates. Fusion merges them into one final ranking. Two dominant approaches:

- **Reciprocal Rank Fusion (RRF).** For each candidate, sum a score derived from its **rank position** in each list:
  `RRF(d) = Σ_k 1 / (k + rank_k(d))`
  where `k` is a constant (commonly `k = 60`). RRF is **rank-based, not score-based** — it does not require the two legs' scores to be on a comparable scale, which is its main advantage. It is parameter-light (one constant `k`) and robust.
- **Weighted combination (score fusion).** Linearly combine the two legs' **raw scores**:
  `score(d) = w_v · vector_sim(d) + w_l · lexical_sim(d)`, with `w_v + w_l = 1`.
  This requires the two scores to be **normalized to a common scale** (e.g. min-max or z-score) before combining, otherwise the higher-magnitude leg dominates. It exposes a tunable `w_v`/`w_l` trade-off but is more sensitive to score calibration than RRF.

### 2.3 Optional third stage: reranking

Many production hybrid pipelines add a **cross-encoder reranker** on top of the fused candidate set: retrieve a broad top-K via hybrid, then rerank the top ~50–100 with a cross-encoder for precision. This is an additive stage, not part of the core hybrid mechanism.

### 2.4 Where hybrid fits in a RAG pipeline

`query → [BM25 index] + [vector index] → fuse (RRF or weighted) → [optional rerank] → top-K context → LLM generation`.

---

## 3. Problem solved

- **Dense-only misses exact matches.** Embeddings are lossy and fuzzy. A query for an exact ID, a version string, a technical symbol, a code identifier, or a rare proper noun often fails dense retrieval because the embedding of the query and the embedding of the exact token are not close enough. BM25 nails these because it is exact-match.
- **Keyword-only misses semantics.** BM25 cannot match a query that uses synonyms, paraphrases, or different wording than the document. Dense retrieval captures meaning.
- **Hybrid solves both.** By fusing the two, hybrid search recovers exact-match recall that dense-only drops, while keeping the semantic recall that keyword-only lacks. The result is higher **recall** (fewer missed relevant chunks) and, with a reranker, higher **precision** (fewer irrelevant chunks in the top-K). This directly improves RAG answer quality because the LLM only sees the retrieved context.

---

## 4. Key benchmarks & performance gains

- **RRF beats individual rank-learning methods.** The original Cormack et al. (SIGIR 2009) paper showed RRF outperforming Condorcet fusion and individual rank-learning methods on TREC data, with no training and a single constant `k`.
- **Fusion functions analysis (arXiv 2210.11934).** A systematic comparison of fusion functions for hybrid retrieval; RRF and related rank-based methods are consistently strong and robust across datasets, and are favored over naive score combination because they avoid cross-scale calibration problems.
- **Hybrid > single-leg recall.** Independent evaluations (e.g. [Evaluating Lexical, Dense, and Hybrid Retrieval Pipelines for RAG](https://doi.org/10.1109/punecon67554.2025.11379254), [From BM25 to Corrective RAG: Benchmarking Retrieval Strategies for Text-and-Table Documents](https://doi.org/10.48550/arxiv.2604.01733)) consistently report hybrid retrieval improving recall over either leg alone, at modest extra compute.
- **Graph + vector hybrid (HybridRAG).** [HybridRAG](https://arxiv.org/html/2408.04948) shows that combining knowledge-graph retrieval with vector retrieval improves answer accuracy and faithfulness on information-extraction QA versus either alone — directly relevant to Astrographer's graph-aware RAG.
- **Caveat.** Exact numbers vary by dataset, embedding model, and fusion method. The authoritative figures are in the cited papers; treat third-party blog numbers as indicative, not canonical.

---

## 5. Common implementation pitfalls

1. **Fusion weight / method tuning.** Weighted combination requires the two legs' scores to be on a comparable scale; if not normalized, the higher-magnitude leg silently dominates. RRF sidesteps this (rank-based) but its `k` constant still needs a sane value (60 is the common default). Tune on *your* corpus, not a benchmark.
2. **RRF vs weighted combination.** RRF is the safer default (no score calibration, one constant, robust). Weighted combination is preferable only when you have a clear reason to bias toward one leg (e.g. a domain where lexical precision matters more) and you can calibrate scores. See [Hybrid Retrieval Fusion: RRF vs Weighted vs Learned](https://dev.to/gabrielanhaia/hybrid-retrieval-fusion-rrf-vs-weighted-vs-learned-when-each-wins-26i1).
3. **Index consistency.** The BM25 index and the vector index must stay in sync with the source documents. If a document is edited/deleted, both indexes must be updated atomically or near-atomically, or retrieval returns stale/ghost chunks. See [Index synchronization strategy](https://theneuralbase.com/hybrid-search/learn/intermediate/index-synchronization-strategy/) and [Consistency guarantees in hybrid search](https://theneuralbase.com/hybrid-search/learn/advanced/consistency-guarantees/).
4. **Embedding-model drift.** Changing the embedding model invalidates the vector index (old vectors are in a different space). Re-embed or version the index. Mixing models across the index and the query breaks dense retrieval.
5. **BM25 tokenization mismatch.** BM25's tokenizer must match how the corpus is indexed; a query tokenized differently than the index silently misses matches. For code/technical terms, ensure the tokenizer keeps symbols/IDs intact.
6. **Score normalization is not free.** If you choose weighted combination, min-max/z-score normalization is itself a tuning surface and can distort rankings on skewed score distributions.
7. **Reranker cost.** Adding a cross-encoder reranker improves precision but adds latency and (if remote) cost. For local-first (D2), a local reranker is feasible but must be budgeted.
8. **"Hybrid" is overloaded.** In graph-RAG contexts, "hybrid" can mean (a) lexical+dense fusion, (b) local+global graph retrieval (LightRAG's `hybrid` mode), or (c) graph+vector retrieval (HybridRAG). Clarify which you mean. Astrographer/Incanter's "hybrid" is graph-tension + vector, which is a distinct axis from lexical+dense.

---

## 6. Auspicion feasibility (D2 — local-first)

**Verdict: HIGHLY FEASIBLE locally.** Hybrid search is one of the most local-friendly RAG techniques:

- **BM25 is cheap.** It is a classic inverted-index scorer with no model inference. It runs on CPU, in-process, with negligible compute and memory. It is the cheapest retrieval leg available.
- **Dense leg is the only model cost.** The vector leg needs an embedding model. Auspicion already runs local embeddings via **Ollama** (Incanter's default `http://127.0.0.1:11434`, model `embeddinggemma:latest`), so the dense leg is already local and D2-compliant.
- **No cloud dependency.** Both legs run entirely on localhost/intranet. This satisfies D2 (local-first) and D1 (AGPL-3.0 — the technique is a well-known algorithm, not a proprietary service).
- **Compute overhead is modest.** BM25 adds near-zero cost; the fusion step (RRF or weighted) is a trivial O(n) merge over the two candidate lists. The dominant cost remains the embedding inference, which is already required for the dense leg.
- **Storage.** BM25 needs an inverted index (a few hundred KB to a few MB per corpus); the vector index is the larger footprint but is already present for the dense leg. No new heavy dependency.

**Net:** hybrid search adds a cheap lexical leg and a trivial fusion step on top of infrastructure Auspicion already runs locally. It is fully D2/D1-compliant.

---

## 7. Astrographer synergy (graphical RAG / wiki)

Astrographer is a **graphical RAG engine + document store/wiki** where documents are Provident graphs and retrieval is graph-aware. Hybrid search maps onto it in several concrete ways:

### 7.1 Incanter already implements a hybrid — but a different axis
Incanter's `POST /v1/query` already fuses **dense vector similarity** with **graph-tension proximity** via **weighted combination** (`vector_weight = 0.6`, `graph_weight = 0.4`, sum 1.0). This is a *graph+vector* hybrid (the HybridRAG axis), **not** a *lexical+dense* hybrid. The two are complementary:
- Incanter's existing hybrid captures **semantic + graph-structural** relevance.
- A **lexical (BM25/keyword) leg** would add **exact-match** recall for IDs, technical terms, and fact keys that neither the vector nor the graph-tension leg reliably catches.

So the Astrographer-relevant question is not "should we add hybrid?" (Incanter already has one) but "should we add a **lexical leg** to Incanter's existing graph+vector hybrid?" — i.e. a **three-way** fusion (BM25 + vector + graph-tension).

### 7.2 Keyword search on fact keys / tags / titles
Astrographer's model has natural lexical targets that dense retrieval handles poorly:
- **`factKey`** — a stable string, unique within a Wiki, the single source of truth for a fact. Users will query by exact fact key ("the `factKey` for the license"). This is a textbook exact-match case where BM25 shines and embeddings are unreliable.
- **`tags`** (string array on each document) and **`title`** — short, exact, identifier-like fields. BM25 over these is cheap and precise.
- **`reference`/`link`/`embed`** targets — resolving "where is this fact referenced?" is a graph-structural query, but finding the *fact by its key* is lexical.

A lexical index over `factKey` + `title` + `tags` (+ node text) would give Astrographer a fast, exact "find the fact/document by name" path that complements Incanter's semantic/graph retrieval.

### 7.3 The fact/reference consistency model
Astrographer's core invariant is that a `fact`'s canonical `value` is the single source of truth, referenced/embedded elsewhere. Hybrid search supports this:
- **Exact-match retrieval of fact keys** lets a user/agent locate the canonical fact node precisely, so references resolve to the right source of truth (not a semantically-similar-but-wrong node).
- **Index consistency** (§5 pitfall 3) is especially important here: when a `fact`'s `value` changes and embeds go `STALE`, the lexical index must reflect the updated fact text, or retrieval returns stale fact content. The BM25 index must be updated in the same pass as the fact update.

### 7.4 RAG query sources (`local` | `incanter` | `zodiac`)
Astrographer's `ragQuery`/`ragStream` return results tagged `source: 'local'|'incanter'|'zodiac'`. A lexical leg could live at any of these layers:
- **`local`** — a local BM25 index over Astrographer's own document store (fact keys, titles, tags, node text) is the natural home for a lexical leg. It runs entirely in Astrographer (TypeScript/Electron), needs no engine, and works even when Incanter is `UNAVAILABLE` (D2 — the engine is optional).
- **`incanter`** — Incanter could add a lexical leg to its `POST /v1/query` (three-way fusion). This is an Incanter-repo change, not an Astrographer change.
- **`zodiac`** — remote companion; hybrid there is out of scope for local-first.

### 7.5 MCP-GUI parity (D4)
A lexical/hybrid search feature must be reachable through both the GUI and the MCP surface (`rag_query`/`rag_stream`). Because hybrid search is a retrieval-mode change (not a security-configuration feature), it is **not** a D4 carve-out — it must be exposed on both surfaces. The `ragQuery` signature already accepts `topK`/`filters`; a hybrid mode would be a query-mode parameter or a default behavior.

---

## 8. Integration path — ingestion-time or query-time?

**Recommendation: primarily query-time, with a small ingestion-time index build.**

- **Query-time (the fusion).** RRF/weighted fusion is purely query-time: run both legs, merge the lists. No re-indexing needed to change the fusion method. This is where the bulk of the work lives and it is cheap to iterate on.
- **Ingestion-time (the BM25 index).** The lexical leg needs an inverted index built at ingestion time (over `factKey`, `title`, `tags`, node text). This is a one-time build per document, updated on document edit/delete. It is the only ingestion-time cost, and it is small (BM25 indexing is fast).
- **Incanter's existing hybrid is already query-time** (`POST /v1/query` fuses at query time). Adding a lexical leg to Incanter would follow the same pattern: build the BM25 index at ingestion, fuse at query time.

**Net:** build the BM25 index at ingestion (cheap, incremental), do the fusion at query time (flexible, tunable). This matches Incanter's existing architecture and keeps the fusion method swappable without re-indexing.

---

## 9. Recommendation

**SHOULD HAVE** — with a clear path to MUST HAVE for the fact-key use case.

**Rationale:**
- **MUST HAVE (narrow):** exact-match retrieval of **`factKey`** is core to Astrographer's single-source-of-truth model. A user/agent must be able to find a fact by its exact key reliably. Dense-only retrieval cannot guarantee this. A lexical index over fact keys is effectively required for the consistency model to be usable.
- **SHOULD HAVE (broad):** a full hybrid (BM25 + vector + graph-tension) materially improves recall for technical terms, IDs, and tags, at near-zero marginal compute (BM25 is cheap, fusion is trivial). It is fully D2/D1-compliant and reuses the existing local Ollama embedding leg.
- **Why not MUST HAVE (broad):** Incanter already provides a graph+vector hybrid. The incremental value of adding a lexical leg is real but not blocking for the core wiki/authoring features; it is an enhancement to retrieval quality, not a prerequisite for the document store or the consistency invariant.
- **Why not NICE TO HAVE / DISCARD:** the cost is low (a local BM25 index + a query-time fusion step) and the benefit (exact-match recall on fact keys, titles, tags) is directly aligned with Astrographer's defining feature. Discarding it would leave a real gap in fact-key retrieval.

**Suggested disposition:**
1. **Local lexical leg in Astrographer (`local` source)** — SHOULD HAVE. A BM25 index over `factKey` + `title` + `tags` + node text, fused with Incanter's results at query time (or surfaced as a `local`-source result). This is the highest-value, lowest-cost increment and works even when Incanter is down.
2. **Three-way fusion in Incanter** — NICE TO HAVE / follow-up. Adding a lexical leg to Incanter's `POST /v1/query` (BM25 + vector + graph-tension) is an Incanter-repo change; park it as a follow-up unless fact-key recall proves insufficient with the local leg.
3. **Fusion method** — use **RRF** (rank-based, no score calibration, one constant) as the default; consider weighted combination only if a clear lexical-vs-semantic bias is needed and scores can be calibrated.

---

## 10. Source URL list

**Primary / papers**
- https://doi.org/10.1145/1571941.1572114 (RRF, Cormack et al., SIGIR 2009)
- https://cormack.uwaterloo.ca/cormacksigir09-rrf
- https://arxiv.org/html/2210.11934v2 (An Analysis of Fusion Functions for Hybrid Retrieval)
- https://www.pinecone.io/research/an-analysis-of-fusion-functions-for-hybrid-retrieval/
- https://www.vldb.org/pvldb/vol19/p1715-gao.pdf (Balancing the Blend, VLDB 2026)
- https://arxiv.org/html/2408.04948 (HybridRAG)
- https://aclanthology.org/2025.acl-long.43.pdf (HYBGRAG, ACL 2025)

**Reference / vendor docs**
- https://www.elastic.co/docs/reference/elasticsearch/rest-apis/reciprocal-rank-fusion
- https://neo4j.com/developer/genai-ecosystem/hybrid-search/

**Guides / deep-dives**
- https://changegamer.ai/resources/hybrid-search-for-rag
- https://www.digitalapplied.com/blog/hybrid-search-bm25-vector-reranking-reference-2026
- https://glaforge.dev/posts/2026/02/10/advanced-rag-understanding-reciprocal-rank-fusion-in-hybrid-search/
- https://topreviewed.ai/blog/hybrid-search-rag-in-production-bm25-dense-vectors-rrf-with-measured-results
- https://mbrenndoerfer.com/writing/hybrid-search-bm25-dense-retrieval-fusion
- https://denser.ai/blog/hybrid-search-for-rag/
- https://www.devopsness.com/blog/hybrid-search-bm25-embeddings-rag
- https://airbyte.com/agentic-data/what-is-hybrid-search
- https://yuhi-sa.github.io/en/posts/20260720_rrf/1/ (RRF deep-dive with math)

**Local / D2 feasibility**
- https://www.santanulabs.com/blog/local-rag-system (OpenSearch Hybrid Search + Ollama)
- https://dev.to/zhangzeyu/stop-paying-for-embedding-apis-local-hybrid-search-with-hippo-b62
- https://baeseokjae.github.io/posts/vector-database-rag-alternative-2026/

**Fusion / consistency pitfalls**
- https://dev.to/gabrielanhaia/hybrid-retrieval-fusion-rrf-vs-weighted-vs-learned-when-each-wins-26i1
- https://theneuralbase.com/hybrid-search/learn/intermediate/index-synchronization-strategy/
- https://theneuralbase.com/hybrid-search/learn/advanced/consistency-guarantees/
- https://theneuralbase.com/hybrid-search/learn/advanced/index-synchronization-at-scale/

**Benchmarks / evaluations**
- https://doi.org/10.1109/punecon67554.2025.11379254 (Lexical, Dense, and Hybrid Retrieval Pipelines for RAG)
- https://doi.org/10.48550/arxiv.2604.01733 (From BM25 to Corrective RAG: Benchmarking Retrieval Strategies)
