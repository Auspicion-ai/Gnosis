# Multi-Query Retrieval — Research Notes

**Topic:** Multi-Query Retrieval (query fan-out / query expansion for RAG)
**Date:** 2026-09-06
**Status:** Research / reference notes (not a spec). Includes Astrographer-specific
analysis (Auspicion feasibility, graphical-RAG synergy, integration path,
recommendation).

---

## 1. Original concept & primary documentation

Multi-Query Retrieval is a **query-time retrieval technique** (not a single
peer-reviewed paper with one canonical citation). It was popularized by the
LangChain `MultiQueryRetriever` and is now a standard "advanced RAG" building
block taught in RAG courses and implemented across every major RAG framework.

| Item | Source |
| --- | --- |
| **LangChain `MultiQueryRetriever` (reference)** | [reference.langchain.com — MultiQueryRetriever](https://reference.langchain.com/python/langchain-classic/retrievers/multi_query/MultiQueryRetriever) |
| **LangChain `MultiQueryRetriever` (source)** | [langchain_classic/retrievers/multi_query.py](https://github.com/langchain-ai/langchain/blob/c4c91d9cd34ac53c7c73162671f98b7769a40123/libs/langchain/langchain_classic/retrievers/multi_query.py) |
| **RAG fundamentals course (mechanism walkthrough)** | [The Neural Base — Multi-query retrieval](https://theneuralbase.com/rag-fundamentals/learn/advanced/multi-query-retrieval/) |
| **Haystack `MultiQueryEmbeddingRetriever`** | [Haystack docs — MultiQueryEmbeddingRetriever](https://docs.haystack.deepset.ai/docs/3.1-unstable/multiqueryembeddingretriever) |
| **Query fan-out explainer** | [Geodocs.dev — What is query fan-out?](https://geodocs.dev/technical/what-is-query-fan-out-optimizing-rag) |
| **Technique comparison (Query Rewriting vs HyDE vs Multi-Query)** | [dreaming.press — Fixing the RAG Question, Not the Index](https://dreaming.press/posts/2026-06-23-query-rewriting-vs-hyde-vs-multi-query-rag.html) |
| **Production explainer** | [Medium — Retrieval Is the Bottleneck](https://medium.com/@mudassar.hakim/retrieval-is-the-bottleneck-hyde-query-expansion-and-multi-query-rag-explained-for-production-c1842bed7f8a) |
| **Reference implementation (atomic-rag)** | [rohinp/atomic-rag — multi-query-expansion.md](https://github.com/rohinp/atomic-rag/blob/main/docs/techniques/multi-query-expansion.md) |

**Related / derivative research (for depth, not the core technique):**
- **DMQR-RAG** — *Diverse Multi-Query Rewriting for RAG* (arXiv 2411.13154):
  improves the *diversity* of the generated query set, addressing the
  redundancy problem in naive multi-query. https://arxiv.org/html/2411.13154
- **Multi-Head RAG** — *Solving Multi-Aspect Problems with LLMs* (arXiv
  2406.05085): decomposes a query into multiple aspects, each retrieved
  separately. https://arxiv.org/pdf/2406.05085
- **MMLF** — *Multi-query Multi-passage Late Fusion Retrieval* (NAACL 2025
  Findings): fuses multiple query/passage retrievals at a late stage.
  https://aclanthology.org/2025.findings-naacl.367.pdf
- **Beyond Single Embeddings** — *Capturing Diverse Targets with Multi-Query
  Retrieval* (arXiv 2511.02770). https://arxiv.org/html/2511.02770
- **GraphSearch** — *An Agentic Deep Searching Workflow for Graph RAG* (arXiv
  2509.22009): graph-specific multi-query / iterative search.
  https://arxiv.org/html/2509.22009
- **Youtu-GraphRAG** — *Vertically Unified Agents for Graph RAG Complex
  Reasoning* (arXiv 2508.19855). https://arxiv.org/html/2508.19855v1
- **StepChain GraphRAG** — *Reasoning Over Knowledge Graphs for Multi-Hop QA*
  (arXiv 2510.02827). https://arxiv.org/pdf/2510.02827

---

## 2. Exact technical mechanism

### Core idea
A single user query is often **vague, ambiguous, or phrased differently from
the vocabulary used in the documents**. A single embedding of that query may
land in a "dead zone" of the vector space and miss the relevant passages. The
Multi-Query technique **uses an LLM to generate several paraphrased /
decomposed variants of the original query**, retrieves for **each** variant,
then **merges and de-duplicates** the union of results before generation.

### The pipeline (query-time, per user query)
1. **Query expansion (LLM call #1).** Prompt the LLM: *"Generate N (e.g. 3–5)
   different versions of this user question to retrieve relevant documents
   from a vector database. By generating multiple perspectives on the user
   question, your goal is to help the user overcome some of the limitations of
   the distance-based similarity search."* The LLM returns N query strings.
2. **Fan-out retrieval (N retrieval calls).** For each generated query, run the
   normal retrieval (embedding similarity over the index). This is the
   **N× retrieval cost** — the defining cost of the technique.
3. **Merge + de-duplicate.** Union the result sets from all N queries. Because
   the same document/passage can be retrieved by multiple variants, results are
   de-duplicated (by document id / node id / content hash). Optionally
   re-rank or weight by how many variants retrieved each item.
4. **Generation (LLM call #2).** Feed the merged, de-duplicated context to the
   generator LLM to produce the final answer.

### Variants
- **Naive multi-query:** N paraphrases of the whole query (LangChain default).
- **Decomposition / sub-query:** split a compound question into independent
  sub-questions, each retrieved separately (overlaps with query decomposition
  and Multi-Head RAG's aspect decomposition).
- **Diverse multi-query (DMQR-RAG):** explicitly prompt for *diverse* variants
  to reduce redundancy in the generated set, improving coverage per retrieval
  call.
- **Late fusion (MMLF):** retrieve per query, then fuse at a late stage with
  learned weights rather than a simple union.

### Where it sits vs. related techniques
- **Query Rewriting** — rewrites the query *once* into a better single query
  (often with a dedicated rewrite model). Multi-Query generates *many* variants
  and retrieves for all.
- **HyDE** — generates a *hypothetical document* and embeds that instead of the
  query. Multi-Query embeds many query strings.
- **RAG-Fusion** — a related technique that additionally uses **reciprocal rank
  fusion (RRF)** to score the merged results across the multiple ranked lists,
  rather than a plain union. Multi-Query is the retrieval-side half; RRF is the
  merge-side refinement.

---

## 3. Problem solved

Multi-Query Retrieval targets the **retrieval-recall bottleneck** — the failure
mode where the *right* document exists in the index but the *query embedding*
doesn't surface it. It addresses:

1. **Vocabulary mismatch.** The user's phrasing differs from the document's
   wording. Paraphrases bridge the lexical/semantic gap (e.g. "how do I
   terminate a process" → "kill a running program", "stop a background task").
2. **Vague / ambiguous queries.** A single ambiguous query embeds to a fuzzy
   centroid; multiple interpretations each retrieve their own relevant region.
3. **Compound / multi-aspect questions.** A question with several facets is
   poorly served by one embedding. Decomposition retrieves each facet's
   evidence separately (Multi-Head RAG formalizes this).
4. **Recall improvement.** The union of N retrievals has strictly higher recall
   than any single retrieval (more candidates reach the generator). This is the
   primary, well-documented benefit.
5. **Robustness to embedding-model blind spots.** If the embedding model is weak
   on a particular phrasing, another variant may still hit.

**The trade-off it does NOT solve:** it does not improve *precision* by itself —
it adds candidates, so it can also add noise (see §9).

---

## 4. Key benchmarks & performance gains

There is **no single canonical benchmark** for the naive technique (it is a
framework feature, not a paper). Evidence is scattered across derivative papers
and framework docs. Reported effects:

- **Recall:** the union of N retrievals raises recall over single retrieval;
  this is the consistent, mechanism-guaranteed gain. The magnitude depends on
  query diversity and index quality.
- **DMQR-RAG (arXiv 2411.13154):** reports that *diverse* multi-query rewriting
  improves retrieval and downstream QA over naive multi-query and single-query
  baselines, primarily by cutting redundant queries and improving coverage.
- **Multi-Head RAG (arXiv 2406.05085):** reports gains on multi-aspect QA
  benchmarks by decomposing queries into aspects and retrieving each — evidence
  that decomposition-style multi-query helps on compound questions.
- **MMLF (NAACL 2025 Findings):** reports that late-fusion of multi-query /
  multi-passage retrieval improves over single-query retrieval on its QA
  benchmarks.
- **Local-model evidence (relevant to D2):** *Dissecting Agentic RAG: A
  Component Ablation for Multi-Hop QA with a Local 7B Model* (arXiv
  2606.21553) ablates agentic-RAG components on a **local 7B model** and shows
  retrieval-side components (including multi-query-style fan-out) contribute
  meaningfully to multi-hop QA — evidence the technique is viable on local
  models, not just frontier APIs.

> **Caveat:** treat all headline numbers as dataset-specific. The robust,
> transferable claim is **"multi-query raises recall at N× retrieval cost"**;
> the *precision* and *end-to-end answer* gains are workload-dependent and must
> be measured on Astrographer's own corpus.

---

## 5. Auspicion feasibility (D2 — local-first)

**Verdict: FEASIBLE on local LLMs, with a real compute/latency cost that must be
budgeted.**

- **Runs on local LLMs.** The technique needs only an LLM that can paraphrase a
  query — a small local model (7B-class and below) is sufficient for the
  expansion step. The *Dissecting Agentic RAG* ablation (arXiv 2606.21553)
  demonstrates retrieval-side fan-out working on a local 7B model. No cloud
  dependency is introduced; D2 holds.
- **Compute/latency overhead is the defining cost.** The technique is
  **N× retrievals + 1 extra LLM call** per user query:
  - **+1 LLM call** to generate the variants (small, but on a local model this
    is wall-clock latency on top of the answer generation).
  - **N× embedding/retrieval calls** against the index. On a local vector index
    this is CPU/GPU-bound; on a graph engine (Incanter) it is N graph traversals.
  - **Merge + de-dup** is cheap (in-memory set union by node/document id).
  - Net effect: **query latency roughly scales with N** (e.g. 3–5× the
    single-query retrieval time), plus one extra LLM round-trip. For a
    local-first wiki this is acceptable for interactive queries but must be
    surfaced in the UI (D4 parity: the GUI and MCP must both expose the
    fan-out setting and its latency).
- **Cost control levers (local-friendly):**
  - Cap N (3 is a common default; 5 is generous).
  - Use a **small dedicated expansion model** (or the same local model with a
    short prompt) rather than the full generator.
  - Cache expansions for repeated queries.
  - Make fan-out **opt-in / configurable** (a `multiQuery: {enabled, n}` knob)
    so simple queries don't pay the N× cost.
- **D1 (AGPL-3.0) / D4 (MCP-GUI parity):** no licensing or parity blocker — the
  technique is a pure orchestration pattern over the existing `ragQuery`/
  `ragStream` surface; it can be exposed identically through GUI and MCP.

---

## 6. Astrographer synergy (graphical RAG / wiki)

Multi-Query Retrieval is a strong fit for Astrographer's **graph-aware** model,
and the graph substrate makes it *more* valuable than in flat-text RAG:

1. **Sub-queries scoped to different graph regions.** Because documents are
   Provident graphs (nodes + edges), each generated query variant can be
   **scoped to a different graph region** — e.g. one variant targets the
   `fact`-node neighborhood, another targets `content` nodes, another targets a
   specific Wiki or cross-wiki region. This turns the naive "N paraphrases of
   the whole query" into **N region-scoped retrievals**, which is exactly the
   graph-aware fan-out that flat RAG cannot express. (GraphSearch, arXiv
   2509.22009, and Youtu-GraphRAG, arXiv 2508.19855, explore this direction for
   graph RAG.)
2. **Wiki cross-linking.** Astrographer's wiki model has cross-wiki references
   (§4.1.2 of the Astrographer contract). A multi-query pass can generate
   variants that explicitly target referenced/embedded documents, so a query
   about a fact surfaces both the canonical `fact` node and the documents that
   `link`/`embed` it — reinforcing the single-source-of-truth model.
3. **The fact/reference model.** The consistency invariant (§4.2.3) means a
   `fact` node is the source of truth and `embed` nodes snapshot it. Multi-query
   retrieval can be biased to return the **canonical `fact` node** (not a stale
   `embed` snapshot) by generating a variant that names the `factKey` or targets
   `fact`-kind nodes — a graph-aware precision improvement that flat RAG can't
   do.
4. **Compound questions over the graph.** Multi-hop questions ("which
   documents reference fact X and are published?") decompose naturally into
   sub-queries that traverse different edges — aligning with the graph's
   structure and with the decomposition variant of multi-query.
5. **Merge by node identity.** De-duplication is cleaner in a graph store:
   results merge by **node id / document id** (stable identities), not by fuzzy
   text similarity, so the merge step is exact and cheap.

**Net synergy:** Multi-Query is not just a recall patch for Astrographer — the
graph substrate lets each variant be **region-scoped and node-kind-aware**, and
the stable node identities make merging exact. This is a genuinely
graph-native application of the technique.

---

## 7. Integration path — ingestion-time or query-time?

**Query-time (recommended).** Multi-Query Retrieval is inherently a
**query-time** technique: it operates on the user's query at retrieval time and
needs no change to the index or the stored graph.

- **Where it lives in Astrographer:** as an **orchestration layer in front of
  the Incanter `ragQuery`/`ragStream` surface** (§4.3 of the Astrographer
  contract). Astrographer (or Incanter) generates N variants, issues N
  `ragQuery` calls (or one `ragStream` fan-out), merges by node/document id, and
  returns a single `RagResult`. The `source` union (`local`|`incanter`|`zodiac`)
  is preserved per result.
- **Ingestion-time is NOT applicable.** There is nothing to precompute at
  ingestion for the naive technique (unlike, say, graph construction or
  community summarization). The only ingestion-adjacent optimization is
  **caching expansions** for repeated queries, which is a query-time cache, not
  an ingestion-time transform.
- **Where the fan-out should be implemented:** the cleanest boundary is
  **inside Incanter** (the Rust engine owns retrieval), so Astrographer's
  `ragQuery` stays a single call and the N× fan-out is hidden behind the engine
  boundary. If Incanter does not yet support it, Astrographer can implement the
  fan-out client-side by issuing N `ragQuery` calls and merging — but this
  multiplies HTTP round-trips and should be a stopgap, not the design. This is
  a **handoff candidate to Incanter** (see §9, and per AGENTS.md this repo never
  patches a tool's source — it records the gap in `docs/defects.md`/HANDOFF).
- **D4 parity:** the fan-out setting (`enabled`, `n`, optional region scoping)
  must be reachable through both the GUI RAG panel and the `rag_query`/
  `rag_stream` MCP tools.

---

## 8. Recommendation

**SHOULD HAVE** (with a scoped, configurable first pass).

**Rationale:**
- **High value for the core product.** Astrographer's defining feature is
  graph-aware retrieval over a wiki with cross-links and fact/reference
  consistency. Multi-Query's recall gain directly improves the RAG query
  surface, and the graph substrate makes it *more* powerful than in flat RAG
  (region-scoped, node-kind-aware variants, exact merge by node id — §6).
- **D2-compatible.** Runs on local LLMs; the only cost is N× retrieval + one
  extra LLM call, which is controllable via an `n` cap and a small expansion
  model (§5).
- **Why not MUST HAVE:** it is an *enhancement* to an existing retrieval
  surface, not a prerequisite for the document-store/wiki core. The core
  (cross-link/embed consistency, graph push, wiki) works without it. It should
  not block the Astrographer MVP.
- **Why not NICE TO HAVE:** the recall bottleneck it fixes is a first-order
  quality issue for a RAG engine, and the graph-native version is cheap to add
  once the `ragQuery` surface exists. Deferring it entirely would leave a
  known, well-understood quality gap.
- **Recommended first pass:** implement as a **configurable query-time
  orchestration** (default `enabled: false` or `n: 3`), with region/node-kind
  scoping as the graph-native enhancement, and a small local expansion model.
  Prefer implementing the fan-out inside Incanter (engine boundary) over a
  client-side N× HTTP loop.

---

## 9. Common implementation pitfalls

1. **Precision risk / noise amplification.** Multi-Query raises recall by adding
   candidates, but it can also add **irrelevant** candidates. The generator may
   then be distracted by noise. Mitigate with a merge-time re-rank (e.g.
   reciprocal rank fusion) and by scoping variants to relevant graph regions.
2. **Hallucination amplification.** The extra LLM call (expansion) is itself a
   generation step that can produce **off-topic or fabricated query variants**,
   which then retrieve irrelevant context that the generator may treat as
   authoritative. Validate/constrain the expansion prompt; consider a small
   dedicated expansion model.
3. **Prompt sensitivity.** The quality of the generated variants is highly
   sensitive to the expansion prompt. A poorly worded prompt yields near-duplicate
   variants (no diversity → no recall gain, just N× cost). DMQR-RAG exists
   precisely because naive prompts produce redundant queries. Test the prompt on
   your corpus.
4. **Cost / latency blow-up.** N× retrieval + 1 LLM call per query. On a local
   model this is wall-clock latency, not just tokens. Without an `n` cap or
   opt-in default, every query pays the fan-out cost. Make it configurable and
   default conservatively.
5. **Redundant queries.** Naive multi-query often generates variants that are
   near-identical, wasting retrieval budget. Use diversity prompting or a
   dedup step on the generated queries themselves.
6. **Merge correctness.** De-duplication must be by **stable identity** (node id
   / document id / content hash), not fuzzy text, or you re-introduce duplicates
   and lose the recall benefit. Astrographer's stable node/document ids make this
   exact (§6.5).
7. **Where the fan-out lives.** If implemented client-side in Astrographer as N
   HTTP `ragQuery` calls, it multiplies round-trips and couples the client to the
   fan-out policy. The engine boundary (Incanter) is the correct home. **Handoff
   candidate:** record an Incanter gap (multi-query fan-out support) in
   `docs/defects.md` + `docs/HANDOFF.md` rather than patching Incanter from this
   repo.
8. **D4 parity.** If the fan-out is added only to the GUI RAG panel and not the
   `rag_query`/`rag_stream` MCP tools, it is a D4 violation and a review finding.
9. **Not a precision fix.** Multi-Query does not fix a weak retriever or a poor
   graph; it only widens the candidate set. Fix retrieval/graph quality first,
   then add fan-out.

---

## 10. Source URL list

**Primary / framework docs**
- https://reference.langchain.com/python/langchain-classic/retrievers/multi_query/MultiQueryRetriever
- https://github.com/langchain-ai/langchain/blob/c4c91d9cd34ac53c7c73162671f98b7769a40123/libs/langchain/langchain_classic/retrievers/multi_query.py
- https://theneuralbase.com/rag-fundamentals/learn/advanced/multi-query-retrieval/
- https://docs.haystack.deepset.ai/docs/3.1-unstable/multiqueryembeddingretriever
- https://geodocs.dev/technical/what-is-query-fan-out-optimizing-rag
- https://github.com/rohinp/atomic-rag/blob/main/docs/techniques/multi-query-expansion.md

**Technique comparisons / production explainers**
- https://dreaming.press/posts/2026-06-23-query-rewriting-vs-hyde-vs-multi-query-rag.html
- https://medium.com/@mudassar.hakim/retrieval-is-the-bottleneck-hyde-query-expansion-and-multi-query-rag-explained-for-production-c1842bed7f8a

**Derivative / research papers**
- https://arxiv.org/html/2411.13154 (DMQR-RAG)
- https://doi.org/10.48550/arxiv.2411.13154 (DMQR-RAG DOI)
- https://arxiv.org/pdf/2406.05085 (Multi-Head RAG)
- https://aclanthology.org/2025.findings-naacl.367.pdf (MMLF)
- https://arxiv.org/html/2511.02770 (Beyond Single Embeddings)
- https://arxiv.org/html/2606.21553v1 (Dissecting Agentic RAG — local 7B ablation)

**Graph-RAG multi-query / multi-hop (Astrographer-relevant)**
- https://arxiv.org/html/2509.22009 (GraphSearch)
- https://arxiv.org/html/2508.19855v1 (Youtu-GraphRAG)
- https://arxiv.org/pdf/2510.02827 (StepChain GraphRAG)
