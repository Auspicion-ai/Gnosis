# Adaptive RAG (Router + Qualifier agents) — Research Notes

**Date:** 2025-09-05
**Topic:** Adaptive Retrieval-Augmented Generation — query-complexity routing
**Status:** Research / reference notes (not a spec)

---

## 1. Original paper & primary documentation

- **Paper:** *Adaptive-RAG: Learning to Adapt Retrieval-Augmented Large Language
  Models through Question Complexity* — Soyeong Jeong, Jinheon Baek, Sukmin
  Cho, Sung Ju Hwang, Jong C. Park (KAIST). **NAACL 2024** (long paper).
  - arXiv: https://arxiv.org/abs/2403.14403
  - HTML (v2): https://arxiv.org/html/2403.14403v2
  - ACL Anthology: https://aclanthology.org/2024.naacl-long.389/
  - Hugging Face paper page: https://huggingface.co/papers/2403.14403
- **Official code:** https://github.com/starsuzi/Adaptive-RAG
- **Featured implementations (third-party):**
  - LangGraph (Cohere Command R): `langgraph_adaptive_rag_cohere.ipynb`
    (https://github.com/langchain-ai/langgraph/blob/main/examples/rag/langgraph_adaptive_rag_cohere.ipynb)
  - LlamaIndex cookbook: `Adaptive_RAG.ipynb`
    (https://github.com/mistralai/cookbook/blob/main/third_party/LlamaIndex/Adaptive_RAG.ipynb)
  - Cohere ReAct notebook: `react_agent_adaptive_rag_cohere.ipynb`
    (https://github.com/cohere-ai/notebooks/blob/main/notebooks/react_agent_adaptive_rag_cohere.ipynb)

---

## 2. Exact technical mechanism

### Core idea
Not all user queries have the same complexity. A one-size-fits-all RAG
strategy is either wasteful (multi-step retrieval on a trivial query) or
insufficient (single-step/no retrieval on a multi-hop query). Adaptive-RAG
**dynamically selects the most suitable strategy per query** based on a
predicted **query-complexity level**.

### The three strategies (from simplest to most sophisticated)
1. **No retrieval (A)** — `LLM(q)`: answer directly from parametric knowledge.
2. **Single-step retrieval (B)** — `LLM(q, d)`: retrieve once, then answer.
3. **Multi-step / iterative retrieval (C)** — `LLM(q, d, c)`: iteratively
   retrieve and reason (e.g. IRCoT-style), for complex multi-hop queries.

### The classifier (the "qualifier"/"router")
- A **smaller language model** (trained classifier) maps a query to one of
  three labels: `o = Classifier(q) ∈ {A, B, C}`.
- **A** = straightforward, answerable by the LLM alone.
- **B** = moderate, needs at least a single retrieval step.
- **C** = complex, needs the most extensive (multi-step) solution.
- The classifier is a **T5** model in the official repo (t5-large, ~770M is
  the "Large" size used for main results; sizes 60M/223M/770M tested).

### Training the classifier (no human labels)
Labels are **automatically collected** via two strategies:
1. **Silver data from model outcomes:** run all three strategies on a query;
   if the simplest correct strategy answers correctly, assign its label.
   Tie-break rule: **prefer the simpler model** (e.g. if both single-step and
   multi-step answer correctly but no-retrieval fails → label B).
2. **Inductive dataset bias (binary data):** for queries left unlabeled (all
   three strategies failed), assign **B** to single-hop dataset queries and
   **C** to multi-hop dataset queries.
- Classifier trained with **cross-entropy loss** on these auto-collected
  (query, complexity) pairs.

### Inference
At inference, forward the query through the classifier, get `{A,B,C}`, and
dispatch to the matching strategy. The LLM and retriever internals are
unchanged — only the routing changes, so the system can move freely between
complexities without retraining the generator.

### Datasets used
- **Single-hop:** SQuAD v1.1, Natural Questions, TriviaQA.
- **Multi-hop:** MuSiQue, HotpotQA, 2WikiMultiHopQA.
- Retriever: DPR-style over Wikipedia (Wiki index ~21M passages; HotpotQA
  ~5.2M, 2Wiki ~430K, MuSiQue ~139K).

---

## 3. Key benchmarks & performance gains

### Main results (Table 1, averaged across all 6 datasets)

| Method | FLAN-T5-XL (3B) F1 | FLAN-T5-XXL (11B) F1 | GPT-3.5-Turbo F1 |
|---|---|---|---|
| No Retrieval | 21.12 | 25.14 | 48.56 |
| Single-step | 44.31 | 47.63 | 46.99 |
| Adaptive Retrieval (baseline) | 32.24 | 35.67 | 48.20 |
| Self-RAG* (LLaMA2) | 20.79 | 22.98 | 22.98 |
| **Adaptive-RAG (ours)** | **46.94** | **48.62** | **50.91** |
| Multi-step (upper bound) | 48.85 | 50.09 | 50.87 |
| Oracle Adaptive-RAG | 56.28 | 58.60 | 62.80 |

- **Adaptive-RAG beats the single-step approach** on F1 for FLAN-T5-XL
  (46.94 vs 44.31) and GPT-3.5 (50.91 vs 46.99), while using **far fewer
  steps** (GPT-3.5: 1.03 steps vs 2.81 for full multi-step).
- **Accuracy (Acc) with GPT-3.5:** Adaptive-RAG 48.97 vs single-step 45.27,
  vs multi-step 49.70 — near the multi-step upper bound at ~1/3 the steps.
- **Efficiency:** GPT-3.5 time/query 1.46s vs 3.33s for full multi-step
  (~2.3× faster) while matching accuracy.

### Classifier performance (Figure 3 / Table 3)
- Classifier accuracy ~54.5% overall (Large/770M) on the auto-labeled set.
- **Confusion matrix trends:** C (multi) misclassified as B (one) ~31%; B as
  C ~23%; A (no) misclassified as B ~47% and as C ~22%.
- **Label distribution (GPT-3.5):** A (no) 8.6%, B (one) 53.3%, C (multi)
  38.1%. Time/query: A=0.35s, B=3.08s, C=27.18s.

### Ablations
- **Training data (Table 4):** full strategy (silver + binary) F1 46.94;
  without binary 43.43; without silver 48.79 (but classifier accuracy drops
  to 40.0% and "No" class accuracy to 0.0% — silver data is needed to learn
  the no-retrieval case).
- **Classifier size (Table 6):** Small (60M) F1 45.83, Base (223M) 45.97,
  Large (770M) 46.94 — **no significant drop with smaller classifiers**,
  supporting resource-efficient deployment.

### Case study (Table 5)
- Simple NQ query: Adaptive-RAG routes to **A (no retrieval)** and answers
  "Google" correctly; Adaptive Retrieval fetches docs, is slower, and
  sometimes returns wrong info ("Microsoft").
- Complex MuSiQue query: Adaptive-RAG routes to **C (multi-step)** and finds
  "Sebastian Cabot" (John Cabot's son); Adaptive Retrieval fails to fetch the
  needed external info.

---

## 4. Common implementation pitfalls

1. **Classifier accuracy is the bottleneck.** The paper's own confusion
   matrix shows heavy A→B misclassification (~47%) and C↔B confusion. A poor
   router silently degrades accuracy (routing simple queries into expensive
   retrieval) or correctness (routing complex queries to no-retrieval). The
   gap between the real classifier and the Oracle (Table 1) is large
   (e.g. GPT-3.5: 50.91 vs 62.80 F1) — the router, not the generator, is the
   ceiling.

2. **Auto-labeling noise.** Silver labels come from "which strategy happened
   to answer correctly," which is noisy and can mislabel. The paper explicitly
   flags this as a limitation. Don't assume the classifier is ground-truth.

3. **Don't drop the silver data.** Ablation shows that without silver data the
   classifier never learns the "no retrieval" (A) class (accuracy 0.0% on A).
   Both silver + binary data are needed.

4. **Routing is not just "retrieve or not."** The original paper routes across
   three strategies; production systems (e.g. LangGraph example) extend this
   to **no-retrieval / web-search / iterative RAG**. The router must be
   re-trained or re-prompted for the actual strategy set you deploy — a router
   trained for one strategy menu won't generalize to another.

5. **Latency vs. accuracy tradeoff is real.** Multi-step (C) queries cost
   ~27s/query vs 0.35s for A. If your workload is mostly simple queries, the
   classifier's A→B misclassification (47%) will inflate cost significantly.
   Measure the actual label distribution on *your* traffic, not the paper's.

6. **Small classifier is fine — use it.** Table 6 shows 60M vs 770M barely
   differ in QA F1. Deploying a huge router wastes resources; a small
   classifier gives most of the benefit.

7. **The generator is unchanged.** Adaptive-RAG only changes routing; it does
   not improve the underlying retriever or generator. If retrieval quality is
   poor, routing won't fix it — fix the retriever first.

8. **Reproduction is heavy.** The official repo requires an Elasticsearch
   retriever server, DPR data downloads, and separate LLM servers. Budget
   significant setup time; the preprocessed data tarballs help.

9. **"Adaptive RAG" is overloaded.** The term is used for (a) this specific
   query-complexity-routing paper, and (b) a general family of adaptive
   retrieval strategies (Self-RAG, Corrective RAG, etc.). Clarify which you
   mean. Self-RAG uses reflection tokens; CRAG uses a retrieval evaluator;
   Adaptive-RAG uses a query-complexity router.

10. **Routing benchmarks are emerging.** RAGRouter-Bench
    (https://arxiv.org/abs/2602.00296, https://github.com/ziqiwang0908/RAGRouter-Bench)
    is a dedicated dataset/benchmark for adaptive RAG routing — useful for
    evaluating a router beyond the paper's 6 QA datasets.

---

## Source URLs

- Paper (arXiv abs): https://arxiv.org/abs/2403.14403
- Paper (HTML v2): https://arxiv.org/html/2403.14403v2
- ACL Anthology: https://aclanthology.org/2024.naacl-long.389/
- Hugging Face paper page: https://huggingface.co/papers/2403.14403
- Official code: https://github.com/starsuzi/Adaptive-RAG
- LangGraph example: https://github.com/langchain-ai/langgraph/blob/main/examples/rag/langgraph_adaptive_rag_cohere.ipynb
- LlamaIndex cookbook: https://github.com/mistralai/cookbook/blob/main/third_party/LlamaIndex/Adaptive_RAG.ipynb
- Cohere ReAct notebook: https://github.com/cohere-ai/notebooks/blob/main/notebooks/react_agent_adaptive_rag_cohere.ipynb
- RAGRouter-Bench paper: https://arxiv.org/abs/2602.00296
- RAGRouter-Bench code: https://github.com/ziqiwang0908/RAGRouter-Bench
- Self-RAG vs CRAG vs Adaptive-RAG comparison: https://dreaming.press/posts/self-rag-vs-corrective-rag-vs-adaptive-rag-retrieval-self-check.html
- Adaptive RAG routing (LangChain course): https://theneuralbase.com/langchain-advanced/learn/intermediate/adaptive-rag-with-routing/
