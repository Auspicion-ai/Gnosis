# Reranking (Cross-Encoders) — Research Notes

**Topic:** Cross-encoder reranking for retrieval-augmented generation (RAG)
**Date:** 2025 (research compiled from primary + current sources)
**Status:** Research / reference notes (not a behavior contract)
**Scope:** Astrographer (Graphical RAG engine + document store/wiki) feasibility

---

## 1. Original paper & primary documentation

| Item | Source |
| --- | --- |
| **Foundational paper** | [Sentence-BERT: Sentence Embeddings using Siamese BERT-Networks](https://arxiv.org/pdf/1908.10084) — Reimers & Gurevych (EMNLP 2019). Introduces the bi-encoder / cross-encoder distinction and the retrieve-and-rerank pattern. |
| **Sentence Transformers rerankers** | [Rerankers — Sentence Transformers docs](https://www.sbert.net/examples/cross_encoder/training/rerankers/README.html) |
| **Retrieve & Re-Rank walkthrough** | [Retrieve & Re-Rank — Sentence Transformers docs](https://sbert.net/examples/sentence_transformer/applications/retrieve_rerank/README.html) |
| **Cohere Rerank API** | [Rerank reference](https://docs.cohere.com/reference/rerank.mdx) · [Reranking quickstart](https://docs.cohere.com/docs/reranking-quickstart.mdx) |
| **LangChain integration** | [CrossEncoderReranker document transformer](https://docs.langchain.com/oss/python/integrations/document_transformers/cross_encoder_reranker) |
| **Local reranker model (BAAI)** | [BAAI/bge-reranker-v2-m3 · Hugging Face](https://huggingface.co/BAAI/bge-reranker-v2-m3) · [Superlinked model card](https://superlinked.com/models/baai-bge-reranker-v2-m3) |

**Core idea in one line:** a cheap **bi-encoder** (embedding) retriever pulls a
candidate set, then a more expensive **cross-encoder** re-scores each candidate
*jointly with the query* to produce a much more accurate final ranking.

---

## 2. Exact technical mechanism

### 2.1 Two-stage (retrieve → rerank) pipeline
1. **First stage — bi-encoder retrieval.** The query and every document are
   embedded independently into a shared vector space; candidates are ranked by
   cosine similarity. This is fast (embeddings are precomputed at ingestion)
   and scales to large corpora, but the query and document never "see" each
   other during scoring.
2. **Second stage — cross-encoder reranking.** The query and each candidate
   are concatenated into a **single input** and passed through a transformer
   that produces a **joint relevance score** for that (query, document) pair.
   The top-k candidates from stage one are re-sorted by this joint score.

### 2.2 Bi-encoder vs cross-encoder (the core trade-off)
- **Bi-encoder:** query and document are encoded **separately** (Siamese
  architecture). Embeddings can be precomputed and cached, so retrieval is
  cheap and fast. But the model never sees the query and document together, so
  it cannot model fine-grained interactions — this is the **precision
  ceiling**.
- **Cross-encoder:** query and document are fed **together** into the model
  (concatenated with a `[SEP]` token), so the model can attend across both
  and capture deep lexical/semantic interaction. This is far more accurate but
  **cannot precompute** — every (query, candidate) pair must be scored at
  query time, so it is O(top-k) forward passes per query.

See [Cross-Encoder vs Bi-Encoder: Why Your Retriever and Your Reranker Can't Be the Same Model](https://dreaming.press/posts/cross-encoder-vs-bi-encoder.html) and [Cross-encoder: joint query-document scoring for rerankers](https://zeroentropy.dev/concepts/cross-encoder/).

### 2.3 Why the two stages are complementary
The bi-encoder's job is **recall** (don't miss relevant docs); the
cross-encoder's job is **precision** (rank the truly relevant docs on top).
Because the cross-encoder only sees a small top-k set, its cost stays bounded
while its accuracy lifts the final ranking. This is the standard
retrieve-and-rerank pattern from the Sentence-BERT paper.

---

## 3. Problem solved

1. **Bi-encoder precision ceiling.** Embedding similarity is a coarse proxy for
   relevance. Two documents can be close in embedding space for reasons
   (topic, style, shared entities) that do not mean the document answers the
   query. The cross-encoder's joint scoring resolves this ambiguity.
2. **Retrieval noise.** A top-k set from a bi-encoder typically contains
   several irrelevant or weakly-relevant hits. Reranking pushes the genuinely
   relevant items to the top and demotes the noise, so the LLM sees a cleaner
   context window.
3. **Recall/precision decoupling.** A single retriever must trade off recall
   (retrieve broadly, more noise) against precision (retrieve narrowly, risk
   missing). Two-stage retrieval lets the first stage maximize recall and the
   second stage restore precision — you get both instead of a compromise.
4. **Context-window efficiency.** By surfacing the most relevant items first,
   reranking lets you feed the LLM a smaller, higher-quality context, which
   improves answer quality and reduces token cost.

---

## 4. Key benchmarks & performance gains

- **Sentence-BERT (foundational):** the retrieve-and-rerank pattern
  (bi-encoder recall + cross-encoder rerank) is the canonical way to get
  near-cross-encoder accuracy at bi-encoder cost; the paper establishes the
  accuracy gap between the two architectures.
- **bge-reranker-v2-m3 (BAAI):** a strong open, local reranker. Multilingual
  (100+ languages), supports long contexts (up to 32K tokens), and is
  lightweight enough to run on commodity hardware. It is a common default for
  local-first reranking. See [Hugging Face](https://huggingface.co/BAAI/bge-reranker-v2-m3).
- **Scaling laws for cross-encoder reranking:** recent work
  ([Scaling Laws for Cross-Encoder Reranking](https://arxiv.org/html/2603.04816v2))
  shows reranker quality scales with model size and data, confirming that
  bigger cross-encoders rerank better — but also that small rerankers already
  capture most of the gain over a bi-encoder baseline.
- **Evaluation tooling:** [retrieval-ranking-eval](https://github.com/mukund1985/retrieval-ranking-eval)
  provides a reference harness for measuring reranker lift (nDCG/MRR-style
  ranking metrics) over a bi-encoder baseline.

> **Caveat:** the *size* of the reranker lift depends on the first stage. A
> reranker cannot recover a document the first stage never retrieved — the
> reranker's ceiling is bounded by first-stage recall (see §9.5).

---

## 5. Auspicion feasibility (D2 — local-first)

**Verdict: feasible, and a good fit for the D2 constraint.**

- **Local models exist and are strong.** `bge-reranker-v2-m3` (BAAI) and the
  `ms-marco` family (e.g. `cross-encoder/ms-marco-MiniLM-L-6-v2`) are
  open-weights cross-encoders that run on CPU or a modest GPU with no cloud
  dependency. They satisfy D2 (local-first) and D1 (open-source, AGPL-3.0
  compatible — these are permissive-licensed weights).
- **Compute/latency on a small top-k set is bounded.** The cross-encoder does
  one forward pass per candidate. On a small top-k (e.g. 10–50), a
  MiniLM-class reranker scores the whole set in tens of milliseconds on CPU
  and single-digit milliseconds on GPU. This is the key to feasibility: the
  cross-encoder is only ever run over the *small* candidate set, never the
  whole corpus.
- **The cost is query-time, not ingestion-time.** Because cross-encoders
  cannot precompute embeddings, the rerank pass adds latency to every query.
  For a local-first wiki/engine this is acceptable if the top-k is kept small
  and the model is small; it is the standard trade-off of the two-stage
  pattern.
- **No cloud dependency.** Everything runs in-process or via a local model
  server. This preserves the D2 "if it can be local, it should be local"
  constraint and the D4 MCP-GUI parity requirement (the same rerank path is
  exposed to both the GUI and MCP).

---

## 6. Astrographer synergy (graphical RAG/wiki)

Astrographer stores documents as **Provident graphs** (nodes + edges) and
consumes the **Incanter** Rust Graph-RAG engine over HTTP/REST + SSE
(`ragQuery`/`ragStream`). Reranking maps onto this architecture in several
ways:

1. **Scoring graph nodes/chunks by joint relevance.** The first stage
   (Incanter graph-aware retrieval) returns a candidate set of nodes/chunks.
   A cross-encoder can re-score those candidates jointly with the query,
   lifting precision on top of the graph's structural recall. This is the
   natural two-stage pairing: graph retrieval for recall, cross-encoder for
   precision.
2. **Surfacing the single most relevant `fact` node.** Astrographer's
   cross-link/data-embed consistency model treats a `fact` node as the single
   source of truth. Reranking is well suited to pick the *one* most relevant
   `fact` node (or the top few) to surface, rather than returning a noisy
   ranked list of references. This directly supports the "single source of
   truth" model.
3. **Capping the hybrid first stage.** Astrographer's RAG query sources are
   `local` | `incanter` | `zodiac`. A reranker can cap the hybrid first stage:
   retrieve a generous candidate set from the hybrid source, then rerank down
   to a small, high-precision context. This keeps the LLM context clean and
   reduces token cost.
4. **Graph-aware reranking is an active research area.** Recent work
   ([GraphER: An Efficient Graph-Based Enrichment and Reranking Method for RAG](https://arxiv.org/html/2603.24925v3),
   [CAGE: Coherence-Aware Graph Encoding for RAG](https://arxiv.org/abs/2609.04647))
   enriches or encodes graph structure into the reranking step. This is a
   promising direction for a *graphical* RAG engine, though it is newer and
   less mature than plain cross-encoder reranking.
5. **MCP-GUI parity (D4).** Because reranking is a pure query-time scoring
   step, it can be exposed identically through both the GUI and the MCP
   endpoint, satisfying D4 without extra surface area.

---

## 7. Integration path — ingestion-time or query-time?

**Recommendation: query-time reranking.**

- **Cross-encoders cannot be precomputed at ingestion.** Their whole value is
  joint query–document scoring, which requires the query at scoring time.
  There is no meaningful ingestion-time cross-encoder pass (you could
  precompute bi-encoder embeddings at ingestion, which Astrographer/Incanter
  already does for the first stage).
- **Query-time placement:** after Incanter's graph-aware first stage returns a
  candidate set, run the cross-encoder over that top-k and re-rank before
  feeding the LLM. This is a thin, well-scoped step in the `ragQuery`/`ragStream`
  path.
- **Ingestion-time role is limited to the first stage.** Keep the bi-encoder
  embedding/indexing at ingestion (already the case); add the cross-encoder
  only at query time. This keeps ingestion fast and puts the extra latency
  exactly where it buys accuracy.

---

## 8. Recommendation

**SHOULD HAVE** — with a clear rationale and a bounded scope.

- **Rationale:** the two-stage retrieve-and-rerank pattern is the single
  highest-leverage, lowest-risk accuracy improvement for a RAG engine. It
  directly addresses the bi-encoder precision ceiling and retrieval noise
  (§3), runs fully local on open models (§5), and maps cleanly onto
  Astrographer's graph retrieval + `fact`-node model (§6). It is a well
  understood, mature technique with strong open-source tooling.
- **Why not MUST HAVE:** Astrographer's first stage is already *graph-aware*,
  which provides structural recall that a flat-text RAG lacks. The marginal
  lift of a cross-encoder over a good graph retriever is real but smaller than
  over a naive vector retriever. It is a precision polish on top of an already
  strong retrieval layer, so it is not a prerequisite for the engine to work.
- **Why not NICE TO HAVE:** the accuracy and context-efficiency gains are
  substantial and cheap to obtain, and the local-first constraint is fully
  satisfied. It is too valuable to defer indefinitely.
- **Scope for a first cut:** a small local reranker (e.g. `bge-reranker-v2-m3`
  or `ms-marco-MiniLM-L-6-v2`) over a capped top-k (10–50) at query time, in
  the `ragQuery`/`ragStream` path, exposed identically to GUI and MCP (D4).
  Graph-aware reranking (GraphER/CAGE-style) is a later, speculative
  enhancement — park it.

---

## 9. Common implementation pitfalls

1. **Latency.** The cross-encoder adds a forward pass per candidate at query
   time. On a large top-k or a large model this can dominate query latency.
   Mitigate by keeping top-k small and the model small; measure on your
   hardware. See [Rerankers and the Latency Budget](https://dev.to/gabrielanhaia/rerankers-and-the-latency-budget-when-cross-encoders-are-worth-it-3efc)
   and [Cross-Encoder Rerankers in Production RAG](https://www.llms.blog/posts/cross-encoder-rerankers-in-production-rag-architecture-score-calibration-latency-budgets-and-model-trade-offs).

2. **Score calibration.** Cross-encoder scores are **not** calibrated
   probabilities. A fixed cutoff like 0.5 is meaningless — scores are
   model- and domain-dependent. Use relative ranking (take top-k) rather than
   an absolute threshold, or calibrate on your own data. See
   [Cross-Encoder Reranker Score Calibration: Why 0.5 Cutoffs Fail](https://dev.to/ji_ai/cross-encoder-reranker-score-calibration-why-05-cutoffs-fail-1k9b)
   and [How to Choose a Reranker's Top-K and Score Threshold](https://dreaming.press/posts/how-to-choose-reranker-top-k-and-score-threshold.html).

3. **Domain sensitivity.** A reranker trained on one domain (e.g. MS MARCO
   web search) may rank poorly on a specialized corpus (e.g. legal, medical,
   or a niche wiki). Evaluate on your own data before trusting it; consider
   fine-tuning a small reranker on domain pairs if the gap is large.

4. **Bounded by first-stage recall.** A reranker can only re-rank what the
   first stage retrieved. If the bi-encoder/graph retriever misses the relevant
   node entirely, the reranker cannot recover it. The reranker's ceiling is the
   first stage's recall — so the first stage must retrieve generously (a
   larger top-k) for the reranker to add value. See
   [How to Evaluate a Reranker for RAG: The Number That Caps It Isn't the Reranker's](https://dreaming.press/posts/how-to-evaluate-a-reranker.html).

5. **The "reranker gap."** Many RAG pipelines skip reranking entirely,
   leaving accuracy on the table. Adding a reranker is often the cheapest
   large accuracy win available. See [The Reranker Gap: Why Most RAG Pipelines Skip the Most Important Layer](https://tianpan.co/blog/2026/04/20/reranker-gap-rag-pipelines).

6. **Don't rerank the whole corpus.** Running a cross-encoder over everything
   defeats the purpose and is prohibitively slow. Always cap the first stage
   to a small candidate set first.

7. **Model/version drift.** Reranker weights and behavior change across
   releases; pin the model version and re-validate ranking quality after any
   upgrade.

---

## 10. Source URL list

**Foundational / primary**
- https://arxiv.org/pdf/1908.10084 (Sentence-BERT, Reimers & Gurevych)
- https://www.sbert.net/examples/cross_encoder/training/rerankers/README.html
- https://sbert.net/examples/sentence_transformer/applications/retrieve_rerank/README.html

**Mechanism / general**
- https://dreaming.press/posts/cross-encoder-vs-bi-encoder.html
- https://zeroentropy.dev/concepts/cross-encoder/
- https://towardsdatascience.com/advanced-rag-retrieval-cross-encoders-reranking/
- https://dataaspirant.com/blog/reranking/
- https://docs.langchain.com/oss/python/integrations/document_transformers/cross_encoder_reranker

**Local models**
- https://huggingface.co/BAAI/bge-reranker-v2-m3
- https://superlinked.com/models/baai-bge-reranker-v2-m3

**Benchmarks / evaluation**
- https://arxiv.org/html/2603.04816v2 (Scaling Laws for Cross-Encoder Reranking)
- https://github.com/mukund1985/retrieval-ranking-eval

**Graph-RAG synergy**
- https://arxiv.org/html/2603.24925v3 (GraphER)
- https://arxiv.org/abs/2609.04647 (CAGE)

**Pitfalls / production**
- https://tianpan.co/blog/2026/04/20/reranker-gap-rag-pipelines
- https://dev.to/ji_ai/cross-encoder-reranker-score-calibration-why-05-cutoffs-fail-1k9b
- https://dev.to/gabrielanhaia/rerankers-and-the-latency-budget-when-cross-encoders-are-worth-it-3efc
- https://www.llms.blog/posts/cross-encoder-rerankers-in-production-rag-architecture-score-calibration-latency-budgets-and-model-trade-offs
- https://dreaming.press/posts/how-to-choose-reranker-top-k-and-score-threshold.html
- https://dreaming.press/posts/how-to-evaluate-a-reranker.html

**Cohere (managed reference)**
- https://docs.cohere.com/reference/rerank.mdx
- https://docs.cohere.com/docs/reranking-quickstart.mdx
