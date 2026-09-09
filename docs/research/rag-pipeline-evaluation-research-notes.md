# RAG Pipeline + DeepEval Evaluation — Research Notes

**Topic:** The end-to-end RAG pipeline (ingestion → chunking → embedding → retrieval → generation) and RAG evaluation with DeepEval
**Date:** 2026-09-07
**Status:** Research / reference notes (not a behavior contract)
**Scope:** General technique + Astrographer-specific feasibility/synergy analysis (Auspicion Suite)

---

## 1. Primary documentation

| Item | Source |
| --- | --- |
| **RAG pipeline overview** | [What Is a RAG Pipeline? (2026 Engineering Guide)](https://www.respan.ai/articles/what-is-a-rag-pipeline) · [Building a RAG pipeline from scratch: the decisions that actually matter](https://www.aicodex.to/articles/building-a-rag-pipeline-from-scratch) · [How to Implement a RAG System](https://www.resourcifi.com/insights/how-to-build-a-rag-system/) |
| **Chunking strategies** | [Chunking Strategies for RAG: A Complete Guide for 2026](https://atlan.com/know/chunking-strategies-rag/) · [RAG: Chunking — llmbestpractices](https://llmbestpractices.com/ai-agents/rag-chunking) |
| **Chunking pitfalls (semantic break)** | [Why the Chunking Problem Isn't Solved: How Naive RAG Pipelines Hallucinate on Long Documents](https://tianpan.co/blog/2026-04-10-rag-chunking-problem-not-solved) · [The Chunk Boundary That Bisected the Sentence Your Answer Depended On](https://tianpan.co/blog/2026-06-01-the-chunk-boundary-that-bisected-your-answer) · [Document Segmentation Matters for Retrieval-Augmented Generation](https://aclanthology.org/2025.findings-acl.422.pdf) (ACL Findings 2025) |
| **DeepEval (RAG evaluation)** | [DeepEval GitHub (confident-ai/deepeval)](https://github.com/confident-ai/deepeval/) · [DeepEval — RAG Evaluation guide](https://deepeval.com/guides/guides-rag-evaluation) · [DeepEval — Getting Started with RAG](https://deepeval.com/docs/getting-started-rag) · [DeepEval — Contextual Precision metric](https://deepeval.com/docs/metrics-contextual-precision) |

**Origin / status.** The RAG pipeline is a mature, widely-deployed pattern (Lewis et al. 2020 coined "Retrieval-Augmented Generation"). DeepEval is an open-source, local-first LLM evaluation framework (Apache-2.0) by Confident AI, with a dedicated RAG evaluation suite. Both are production-stable and well-documented.

---

## 2. Exact technical mechanism — the five pipeline stages

`ingest → chunk → embed → retrieve → generate`.

1. **Ingestion.** Documents are loaded, normalized, and parsed into a canonical form. Metadata (title, tags, source, timestamps, IDs) is extracted and attached so it can be indexed and filtered later.
2. **Chunking.** Each document is split into retrieval units ("chunks"). Strategies: **fixed-size** (token/char windows, with or without overlap), **metadata-aware** (split on headings/sections, keep metadata on each chunk), **natural break points** (paragraphs, sentences, markdown/HTML structure), and **LLM-defined breaks** (an LLM proposes semantically coherent boundaries). Chunk size/overlap is a key tuning surface.
3. **Embedding.** Each chunk is embedded into a dense vector (text and/or multi-modal) with a single embedding model; the query is embedded with the same model. Vectors are stored in a vector index (HNSW/IVF) for ANN search.
4. **Retrieval.** The query retrieves candidate chunks via **BM25** (lexical/exact-match), **semantic** (dense-vector similarity), or **hybrid** (both fused, e.g. Reciprocal Rank Fusion). Optional reranking (cross-encoder) on the top-K.
5. **Generation.** The retrieved context is injected into an LLM prompt; the LLM answers grounded in that context. Generation quality is bounded by retrieval quality — garbage-in, garbage-out.

---

## 3. Problem solved

- **RAG grounds generation in retrieved evidence**, reducing hallucination versus pure LLM generation, and lets a model answer over private/large corpora it was not trained on.
- **The pipeline is only as good as its weakest stage.** Retrieval quality caps answer quality; chunking quality caps retrieval quality. Each stage is a lever on the final answer's faithfulness and relevance.
- **Evaluation closes the loop.** DeepEval measures whether the retrieved context and the generated answer are actually faithful and relevant, so pipeline changes (chunk size, embedding model, retrieval mode) can be compared objectively instead of by eyeballing answers.

---

## 4. Chunking pitfalls

- **Chunking can break semantic meaning.** A chunk boundary that bisects a sentence, a code block, or a multi-sentence claim destroys the unit's self-contained meaning, so retrieval returns a fragment that is useless or misleading to the LLM. See [The Chunk Boundary That Bisected the Sentence Your Answer Depended On](https://tianpan.co/blog/2026-06-01-the-chunk-boundary-that-bisected-your-answer).
- **Naive fixed-size chunking hallucinates on long documents** — splitting mid-thought produces chunks that lack the context needed to answer, and the LLM fills the gap by confabulating ([Why the Chunking Problem Isn't Solved](https://tianpan.co/blog/2026-04-10-rag-chunking-problem-not-solved)).
- **Document segmentation matters.** [ACL Findings 2025](https://aclanthology.org/2025.findings-acl.422.pdf) shows segmentation strategy measurably affects retrieval and generation quality.
- **Mitigations:** overlap between chunks, metadata-aware/natural-break splitting, LLM-defined breaks, and (for graph RAG) chunking along graph structure rather than flat text.

---

## 5. Evaluation with DeepEval

- **What it is.** DeepEval is an open-source (Apache-2.0) Python evaluation framework for LLM apps, with a dedicated RAG evaluation suite. It runs **locally/offline** — it can use a local LLM (e.g. Ollama) as the judge, so no API key or cloud dependency is required (D2-compatible).
- **How it evaluates RAG.** It scores the retrieval + generation pair against the query and the ground-truth answer using LLM-as-judge metrics:
  - **Faithfulness** — is the answer grounded in the retrieved context (no hallucinated claims)?
  - **Answer relevancy** — does the answer actually address the query?
  - **Contextual precision** — are the relevant chunks ranked above irrelevant ones in the retrieved set?
  - **Contextual recall** — did retrieval surface all the chunks needed to answer?
  - Plus **G-Eval** (general LLM-as-judge) and **hallucination** metrics.
- **Open-source & local-first:** yes on both. Apache-2.0, self-hostable, local-LLM judge supported. This aligns with D1 (open-source) and D2 (local-first).

---

## 6. Astrographer synergy

Astrographer is a **Graphical RAG engine + document store/wiki** where documents are **Provident graphs** (nodes + edges) and a `fact` node is the single source of truth. It consumes the Incanter Rust Graph-RAG engine via `ragQuery`/`ragStream` (spec §4.3).

- **Pipeline mapping.** Ingestion = document store operations (`createDocument`/`updateDocument`, spec §4.1.3) producing Provident graphs. Chunking = Incanter's **dynamic chunking** (INCANTER-DISPOSITION) — for graph documents, chunking should follow **graph structure** (per-node/per-fact units) rather than flat text, which naturally avoids the semantic-break pitfall. Embedding = Incanter's local Ollama embedding leg (D2). Retrieval = Incanter's graph+vector hybrid (`ragQuery`). Generation = the LLM grounded in retrieved graph context.
- **The `fact` node is the natural chunk unit.** A `fact`'s canonical `value` is self-contained and single-source-of-truth — an ideal retrieval unit. Chunking along facts/nodes preserves semantic meaning and supports the consistency invariant (references resolve to the canonical fact, not a semantically-similar-but-wrong node).
- **How DeepEval could evaluate Astrographer's RAG.** Run DeepEval against `ragQuery`/`ragStream` outputs: **faithfulness** (is the answer grounded in the retrieved graph nodes?), **answer relevancy** (does it answer the query?), **contextual precision/recall** (did retrieval surface the right fact nodes, ranked correctly?). Because DeepEval is local-first, it can run fully offline against Incanter's local retrieval — D2-compliant. It would objectively compare chunking/retrieval changes (e.g. fact-node chunking vs flat-text chunking).
- **MCP-GUI parity (D4).** Evaluation is a dev/QA concern, not a user-facing feature; it need not be exposed on the MCP surface. But any *user-facing* retrieval-quality feature it motivates must remain reachable through both GUI and MCP (`rag_query`/`rag_stream`).

---

## 7. Recommendation (per stage)

1. **Ingestion** — reuse the document-store API as the ingestion boundary; extract metadata (`title`, `tags`, `factKey`) at write time for later filtering. **MUST HAVE** (already the store model).
2. **Chunking** — chunk along **graph structure / per-fact-node**, not flat text, to preserve semantic meaning and the single-source-of-truth invariant. **MUST HAVE** for Astrographer's defining model; Incanter's dynamic chunking is the vehicle.
3. **Embedding** — keep the local Ollama embedding leg (D2); use one consistent model across index and query (avoid model drift). **MUST HAVE** (already present).
4. **Retrieval** — keep Incanter's graph+vector hybrid; consider adding a lexical (BM25) leg for exact `factKey`/`title`/`tag` lookup (see hybrid-search notes). **SHOULD HAVE** for fact-key recall.
5. **Generation** — ground the LLM in retrieved graph context; no change needed beyond retrieval quality.
6. **Evaluation** — adopt **DeepEval** (local-first, Apache-2.0) as the RAG evaluation harness, run offline against Incanter's retrieval, and gate pipeline changes on faithfulness/relevancy/contextual precision-recall. **SHOULD HAVE** — it is the objective feedback loop the other stages need.

---

## 8. Source URL list

**Pipeline**
- https://www.respan.ai/articles/what-is-a-rag-pipeline
- https://www.aicodex.to/articles/building-a-rag-pipeline-from-scratch
- https://www.resourcifi.com/insights/how-to-build-a-rag-system/

**Chunking**
- https://atlan.com/know/chunking-strategies-rag/
- https://llmbestpractices.com/ai-agents/rag-chunking
- https://tianpan.co/blog/2026-04-10-rag-chunking-problem-not-solved
- https://tianpan.co/blog/2026-06-01-the-chunk-boundary-that-bisected-your-answer
- https://aclanthology.org/2025.findings-acl.422.pdf

**DeepEval**
- https://github.com/confident-ai/deepeval/
- https://deepeval.com/guides/guides-rag-evaluation
- https://deepeval.com/docs/getting-started-rag
- https://deepeval.com/docs/metrics-contextual-precision
