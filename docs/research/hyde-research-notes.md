# HyDE (Hypothetical Document Embeddings) — Research Notes

**Topic:** HyDE — zero-shot dense retrieval via LLM-generated hypothetical documents
**Date:** 2025-09-07
**Status:** Research / reference notes (not a behavior contract)
**Scope:** General technique + Astrographer-specific feasibility analysis (sections 3–6)

---

## 1. Original paper & primary documentation

| Item | Source |
| --- | --- |
| **Paper (arXiv)** | [Precise Zero-Shot Dense Retrieval without Relevance Labels](https://arxiv.org/abs/2212.10496) · [PDF](https://arxiv.org/pdf/2212.10496) · [HTML (ar5iv)](https://ar5iv.labs.arxiv.org/html/2212.10496) |
| **Paper (ACL Anthology, ACL 2023)** | [ACL Anthology page](https://aclanthology.org/2023.acl-long.99.pdf) |
| **Author copy (CMU)** | [HyDE.pdf](https://boston.lti.cs.cmu.edu/luyug/HyDE/HyDE.pdf) |
| **Authors / origin** | Luyu Gao, Xueguang Ma, Jimmy Lin, Jamie Callan (CMU). Published at **ACL 2023** (long paper). |

**Featured implementations / practical guides:**
- [What Is HyDE? How to Improve RAG with Hypothetical Documents (freeCodeCamp)](https://www.freecodecamp.org/news/what-is-hyde-how-to-improve-rag-with-hypothetical-documents/)
- [HyDE Retrieval | Haystack Documentation](https://docs.haystack.deepset.ai/docs/next/hypothetical-document-embeddings-hyde)
- [HyDE Retrieval | WUNDERLAND](https://docs.wunderland.sh/docs/guides/hyde-retrieval)
- [RAG-with-Hyde (community repo)](https://github.com/imanoop7/RAG-with-Hyde)

---

## 2. Exact technical mechanism

### Core idea
HyDE **pivots through a hypothetical document** to bridge the query–document gap. Instead of embedding the (short, ambiguous) user query directly and searching, HyDE first asks an **instruction-following LLM** to *write a document that answers the question*, then embeds **that generated document** and searches the corpus with the resulting vector.

### The two-step decomposition
HyDE decomposes dense retrieval into two tasks that are each easier than learning a relevance-scoring query encoder:

1. **Generative task (LLM).** Feed the query `q` to an instruction-following language model with an instruction such as *"write a paragraph that answers the question."* The model samples a **hypothetical document** `d̂`. This document is *not real* and may contain factual errors, but it is expected to capture the **relevance pattern** of a real relevant document. Relevance modeling is thereby **offloaded from a representation-learning model to an NLG model** that generalizes more easily.
2. **Document–document similarity task (contrastive encoder).** Encode the hypothetical document with an **unsupervised contrastive encoder** (e.g. **Contriever** for English, **mContriever** for multilingual) into a vector `v̂`. Search the corpus by inner-product similarity against the precomputed document vectors. The encoder's **dense bottleneck acts as a lossy compressor** that filters out the hallucinated/extra details, grounding the hypothetical vector to the actual corpus.

### Key properties
- **No model is trained.** Both the generative LLM and the contrastive encoder remain intact. The only supervision ever involved was the LLM's instruction-following alignment. HyDE is therefore **fully zero-shot** and needs **no relevance labels**.
- **Query–document similarity is never explicitly modeled or computed.** The retrieval task is recast as two NLU/NLG tasks; the encoder only ever compares *document-to-document* similarity (which unsupervised contrastive learning captures well).
- **Backbone models in the paper:** InstructGPT (`text-davinci-003`, temperature 0.7) for generation; Contriever / mContriever for encoding. Ablations also used a 52B Cohere model and an 11B FLAN-T5-xxl.
- **Standard MIPS index** — no special index or training data required.

---

## 3. Problem solved

1. **Query–document vocabulary gap / asymmetry.** Queries are short and use the user's vocabulary; documents are long and use the author's vocabulary. A direct query embedding may fail to place the query near the relevant document. HyDE converts the query into a *document-shaped* text, so the search becomes **answer-against-answer** rather than question-against-answer, which is a much better-conditioned similarity problem.
2. **Query ambiguity.** A hypothetical document forces the model to commit to one concrete interpretation of an ambiguous query, producing a more specific search vector.
3. **Zero-shot retrieval without relevance labels.** HyDE removes the need for relevance supervision entirely — the single biggest practical win. It works out-of-the-box across tasks (web search, QA, fact verification) and languages (Swahili, Korean, Japanese, Bengali).
4. **Cold-start retrieval.** The paper argues HyDE is most valuable at the *beginning* of a search system's life, before a supervised retriever can be trained on accumulated query logs.

---

## 4. Key benchmarks & performance gains

### Web search — TREC DL19/DL20 (Table 1)

| Method | DL19 map | DL19 ndcg@10 | DL19 recall@1k | DL20 map | DL20 ndcg@10 | DL20 recall@1k |
|---|---|---|---|---|---|---|
| BM25 | 30.1 | 50.6 | 75.0 | 28.6 | 48.0 | 78.6 |
| Contriever (unsupervised) | 24.0 | 44.5 | 74.6 | 24.0 | 42.1 | 75.4 |
| **HyDE** | **41.8** | **61.3** | **88.0** | **38.2** | **57.9** | **84.4** |
| DPR (supervised) | 36.5 | 62.2 | 76.9 | 41.8 | 65.3 | 81.4 |
| ANCE (supervised) | 37.1 | 64.5 | 75.5 | 40.8 | 64.6 | 77.6 |
| Contriever FT (supervised) | 41.7 | 62.1 | 83.6 | 43.6 | 63.2 | 85.8 |

- HyDE **beats BM25 by large margins** and roughly **matches the supervised Contriever FT** on DL19 (best recall@1k of all). On DL20 it is ~10% lower than Contriever FT on map/ndcg@10 but similar recall@1k.

### Low-resource BEIR tasks (Table 2, nDCG@10)

| Dataset | BM25 | Contriever | **HyDE** | Contriever FT |
|---|---|---|---|---|
| SciFact | 67.9 | 64.9 | **69.1** | 67.7 |
| Arguana | 39.7 | 37.9 | **46.6** | 44.6 |
| TREC-Covid | **59.5** | 27.3 | 59.3 | 59.6 |
| FiQA | 23.6 | 24.5 | 27.3 | 32.9 |
| DBPedia | 31.8 | 29.2 | **36.8** | 41.3 |
| TREC-NEWS | 39.5 | 34.8 | **44.0** | 42.8 |

- HyDE **improves Contriever across the board** (both nDCG and recall). It is only outperformed by BM25 on **TREC-Covid by a tiny 0.2 margin** — where the underlying Contriever underperforms by >50%. HyDE generally **beats the supervised DPR and ANCE** and is competitive with Contriever FT.

### Multilingual — Mr.Tydi (Table 3)
- HyDE improves mContriever on Swahili, Korean, Japanese, Bengali, and can outperform non-Contriever models fine-tuned on MS-MARCO, though it trails fine-tuned mContriever FT (low-resource languages are under-trained in both the encoder and the LLM).

### Instruction-LM ablation (Table 4)
- Smaller instruction LMs (11B FLAN-T5-xxl, 52B Cohere) **still improve the unsupervised Contriever**; larger models give larger gains. This is the key result for **local-first feasibility**.
- **Caveat:** less powerful instruction LMs can *slightly degrade* a *fine-tuned* encoder (HyDE is not intended for that regime).

---

## 5. Common implementation pitfalls

1. **Latency & cost tradeoff.** HyDE trades latency and token cost for recall: every query requires an **extra LLM generation** (a few hundred tokens) plus one extra embedding. It "earns its keep only when the query–document asymmetry is the actual bottleneck." If your queries already match your documents well, HyDE adds cost without benefit.
2. **Hallucination is real but usually harmless.** The hypothetical document can contain false details. The paper's defense is that the encoder's dense bottleneck filters hallucinated specifics. This is not guaranteed — a badly hallucinated document can still pull the vector toward the wrong neighborhood. Guardrails (e.g. grounding the generated doc, capping generation length) are recommended.
3. **Underperforms when asymmetry isn't the bottleneck.** Independent testing (["When does HyDE help RAG? I tested 3 query types and it failed on two"](https://pub.towardsai.net/when-does-hyde-help-rag-i-tested-3-query-types-and-it-failed-on-two-c8946453de34)) shows HyDE helps mainly for **vocabulary-mismatch / paraphrase** queries, and can hurt on queries where the direct query embedding is already well-aligned.
4. **Instruction under-specification.** The paper attributes HyDE's weaker FiQA/DBPedia results to under-specified instructions; the instruction prompt materially affects quality and must be tuned per domain.
5. **Multilingual degradation.** Small encoders saturate as language count scales, and the LLM is under-trained for low-resource languages — HyDE's multilingual gains are smaller.
6. **Don't pair with a fine-tuned retriever expecting gains.** HyDE is most valuable in the *no-relevance-labels* regime; a weak instruction LM can slightly hurt an already fine-tuned retriever.
7. **Non-determinism.** LLM sampling (temperature) makes the hypothetical document non-deterministic, so retrieval results can vary run-to-run — relevant for reproducibility and testing.

---

## 6. Astrographer-specific analysis

### 6.1 Auspicion feasibility (D2 local-first)
- **Runs on local LLMs/embeddings — yes.** HyDE needs (a) an instruction-following LLM for generation and (b) a contrastive embedding model. Both are available locally (e.g. Ollama/llama.cpp for the LLM; sentence-transformers / local embedding endpoints for the encoder). **No training is required**, which is the decisive local-first advantage — HyDE is a pure inference-time technique.
- **Compute/latency overhead.** The cost is **one extra LLM generation per query** (a few hundred tokens) plus one embedding. On a local LLM this adds **seconds of latency** per query, which is the main D2 concern. The paper's Table 4 shows an **11B FLAN-T5-xxl still improves retrieval**, so a mid-size local model is viable; a small local model may degrade quality (pitfall #6).
- **No cloud dependency.** HyDE itself introduces no cloud requirement; it only needs the local LLM + local embeddings already assumed by D2. It is fully compatible with the `local` RAG query source.

### 6.2 Astrographer synergy (graphical RAG/wiki)
Astrographer stores documents as **Provident graphs** (nodes + edges) with a **fact-node single-source-of-truth** model, and retrieves graph-aware via the Incanter engine. HyDE maps onto this in several ways:

1. **Hypothetical Provident graph snippet.** Instead of a flat hypothetical paragraph, the LLM can be instructed to emit a **hypothetical graph snippet** — a small set of candidate `fact` nodes and edges that would answer the query. Embedding that snippet (or its serialized form) and retrieving by it targets the graph's fact nodes directly, rather than flat chunks.
2. **Matching fact nodes.** Because a `fact` node is the single source of truth referenced/embedded elsewhere, HyDE's "answer-against-answer" geometry is a natural fit: generate a hypothetical *fact statement*, embed it, and retrieve the closest real fact nodes. This directly attacks the query-vocabulary gap for fact lookup.
3. **Fact/reference consistency model.** HyDE can be steered to generate a hypothetical document that *references* fact nodes (mirroring the cross-link/data-embed model), so the retrieved neighborhood is anchored to the canonical fact nodes rather than to duplicate prose.
4. **Graph-RAG precedent.** [HyP-KGRAG: Hypothetical Path-Based Knowledge Graph Retrieval Augmented Generation with DeepSeek](https://doi.org/10.5445/ir/1000188236) ([CEUR PDF](https://ceur-ws.org/Vol-4079/paper4.pdf)) extends exactly this idea — generating **hypothetical paths** in a knowledge graph and retrieving by them — demonstrating the technique transfers to graph retrieval, not just flat text.
5. **Query-source fit.** HyDE is a **query-time** transform that can wrap any of the `local | incanter | zodiac` sources: generate the hypothetical doc, embed it, then pass the resulting vector (or the generated text) into the chosen source's retrieval. It is orthogonal to the graph engine and composes cleanly with `ragQuery`/`ragStream`.

### 6.3 Integration path — query-time (recommended)
- **Query-time (recommended).** HyDE is inherently a **query-side** technique: generate a hypothetical document per query, embed it, retrieve. It requires **no ingestion-time changes** to the Provident graph or the Incanter index. This is the low-risk path and matches the paper's intended usage. It can be exposed as an **optional query mode** (e.g. `local+hyde`, `incanter+hyde`) so it is opt-in and does not add latency to every query.
- **Ingestion-time (not recommended).** Pre-generating hypothetical documents at ingestion is not the standard HyDE pattern and would bloat the index with synthetic content that risks polluting the fact-node single-source-of-truth model. Avoid unless a specific use case (e.g. known recurring query patterns) justifies it.

### 6.4 Recommendation — **SHOULD HAVE** (as an opt-in query mode)
- **Rationale:** HyDE is a cheap, training-free, local-first-compatible retrieval boost that directly targets the query–document vocabulary gap — a real concern for a wiki where users phrase queries differently from how facts are stored. It composes cleanly with the graph engine and the fact-node model, and has a published graph-RAG precedent (HyP-KGRAG).
- **Why not MUST HAVE:** Astrographer's graph-aware retrieval already mitigates some of the vocabulary gap via graph structure and entity/relation matching; HyDE is an enhancement, not a prerequisite. Its per-query LLM latency conflicts with D2 local-first responsiveness if made the default.
- **Why not NICE TO HAVE / DISCARD:** The training-free, no-ingestion-change nature makes it unusually low-cost to add, and it is a well-validated technique (ACL 2023, strong BEIR/DL results). It should be **opt-in** (not default) to protect latency, and gated behind a configurable query mode so users on fast local hardware can enable it.
- **Suggested shape:** a `hyde` query-mode wrapper that (1) generates a hypothetical fact/graph snippet via the local LLM, (2) embeds it, (3) routes the vector into `local | incanter | zodiac`, with a configurable toggle and a latency budget. Park the graph-snippet variant (6.2.1) as a follow-up refinement.

---

## 7. Source URL list

**Primary / paper**
- https://arxiv.org/abs/2212.10496
- https://arxiv.org/pdf/2212.10496
- https://ar5iv.labs.arxiv.org/html/2212.10496
- https://aclanthology.org/2023.acl-long.99.pdf
- https://boston.lti.cs.cmu.edu/luyug/HyDE/HyDE.pdf

**Practical guides / implementations**
- https://www.freecodecamp.org/news/what-is-hyde-how-to-improve-rag-with-hypothetical-documents/
- https://docs.haystack.deepset.ai/docs/next/hypothetical-document-embeddings-hyde
- https://docs.wunderland.sh/docs/guides/hyde-retrieval
- https://github.com/imanoop7/RAG-with-Hyde

**Graph-RAG application**
- https://doi.org/10.5445/ir/1000188236 (HyP-KGRAG)
- https://ceur-ws.org/Vol-4079/paper4.pdf (HyP-KGRAG PDF)
- https://doi.org/10.5281/zenodo.19394029 (Hybrid RAG on serverless PostgreSQL: HyDE vs graph-based vs fusion)

**Independent evaluations / comparisons**
- https://pub.towardsai.net/when-does-hyde-help-rag-i-tested-3-query-types-and-it-failed-on-two-c8946453de34
- https://dreaming.press/posts/2026-06-23-query-rewriting-vs-hyde-vs-multi-query-rag.html
- https://zeroentropy.dev/concepts/query-expansion/
- https://vucense.com/dev-corner/advanced-rag-hyde-reranking-hybrid-search-2026/
- https://www.arxiv.org/pdf/2506.21568 (1B/4B local LLM + RAG + HyDE evaluation)
