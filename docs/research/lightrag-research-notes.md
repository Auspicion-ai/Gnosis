# Light RAG — Research Notes

**Topic:** LightRAG (Deduplication, dual-level retrieval)
**Date:** 2025 (research compiled from primary sources)
**Status:** Research / reference notes (not a behavior contract)

---

## 1. Original paper & primary documentation

| Item | Source |
| --- | --- |
| **Paper (arXiv)** | [LightRAG: Simple and Fast Retrieval-Augmented Generation](https://arxiv.org/html/2410.05779v2) |
| **Paper (ACL Anthology, EMNLP 2025 Findings)** | [ACL Anthology page](https://aclanthology.org/2025.findings-emnlp.568/) · [PDF](https://aclanthology.org/2025.findings-emnlp.568.pdf) · [DOI](https://doi.org/10.18653/v1/2025.findings-emnlp.568) |
| **Official project site** | [LightRAG](https://lightrag.github.io/) |
| **Official repo** | [HKUDS/LightRAG](https://github.com/HKUDS/LightRAG/) |
| **Algorithm doc** | [docs/Algorithm.md](https://github.com/HKUDS/LightRAG/blob/v1.4.13/docs/Algorithm.md) |
| **Reproduction doc** | [docs/Reproduce.md](https://github.com/HKUDS/LightRAG/blob/47398a1060efe793bd09e8944867085ddf6c1416/docs/Reproduce.md) |
| **Core programming doc** | [docs/ProgramingWithCore.md](https://github.com/HKUDS/LightRAG/blob/main/docs/ProgramingWithCore.md) |
| **DeepWiki (community docs)** | [Retrieval and Query Modes](https://deepwiki.com/HKUDS/LightRAG/2.3-retrieval-and-query-modes) · [Core Concepts](https://deepwiki.com/HKUDS/LightRAG/1.2-core-concepts-and-terminology) |
| **Neo4j engineering deep-dive** | [Under the covers with LightRAG: Retrieval](https://neo4j.com/blog/developer/under-the-covers-with-lightrag-retrieval/) |

**Authors / origin:** HKUDS (HKU Data Science Lab). Published at **EMNLP 2025 Findings** (paper id `2025.findings-emnlp.568`). The arXiv v2 (2410.05779v2) is the canonical technical reference.

---

## 2. Exact technical mechanism

LightRAG is a **graph-based RAG** framework that builds a **lightweight knowledge graph** from documents and performs **dual-level retrieval** over it. It is explicitly positioned as a simpler, faster, and cheaper alternative to Microsoft's GraphRAG.

### 2.1 Pipeline overview
1. **Chunking** — documents are split into chunks.
2. **Entity & relation extraction** — an LLM extracts entities and relations from each chunk.
3. **Graph construction** — entities become nodes, relations become edges, forming a graph.
4. **Deduplication (graph operation D(·))** — duplicate entities/relations are merged to keep the graph compact.
5. **Dual-level retrieval** — queries are answered by retrieving at two granularities (low-level and high-level).
6. **Generation** — retrieved context is fed to an LLM to produce the answer.

### 2.2 Deduplication — graph operation D(·)
The paper defines a **deduplication operation D(·)** applied to the extracted graph. Its purpose is to **optimize the graph** by merging semantically identical or near-identical entities and relations, preventing the graph from bloating with redundant nodes/edges.

- **What it merges:** duplicate entities (e.g., "Apple Inc." vs "Apple") and duplicate relations.
- **How duplicates are identified:** the mechanism is **LLM-driven** — the model is prompted to judge whether two entities/relations refer to the same real-world object and to merge them. It is *not* a pure string/embedding match.
- **Why it matters:** without deduplication, the graph grows unboundedly with each insertion, retrieval becomes noisy (multiple nodes for the same concept), and cost rises. D(·) keeps the graph compact and retrieval precise.
- **Community discussion confirms the mechanism:** see [Discussion #1526 — "How does Deduplication to Optimize Graph Operation D(.) identify duplicate nodes?"](https://github.com/HKUDS/LightRAG/discussions/1526) and [Issue #1631 — "About Entity Merging"](https://github.com/HKUDS/LightRAG/issues/1631). The implementation lives in [`lightrag/operate.py`](https://github.com/HKUDS/LightRAG/blob/e675598db402da0241f089a22f6939dfbe223437/lightrag/operate.py).

### 2.3 Dual-level retrieval
LightRAG retrieves at **two levels** and exposes them as query modes:

- **Low-level retrieval** — targets **specific entities and their direct relations**. Best for **specific, fact-oriented queries** (e.g., "What is the capital of France?").
- **High-level retrieval** — targets **themes and higher-order relationships** (communities / aggregated concepts). Best for **broad, summary-oriented queries** (e.g., "Summarize the main themes of this document set.").

**Query modes exposed by the API:**
- `naive` — plain vector retrieval (no graph).
- `local` — low-level retrieval (specific entities/relations).
- `global` — high-level retrieval (themes/communities).
- `hybrid` — combines local + global (both levels).

See [Retrieval and Query Modes | DeepWiki](https://deepwiki.com/HKUDS/LightRAG/2.3-retrieval-and-query-modes) and the query router in [`lightrag/api/routers/query_routes.py`](https://github.com/HKUDS/LightRAG/blob/568c0077/lightrag/api/routers/query_routes.py).

### 2.4 Key design choices vs GraphRAG
- **No community summarization pass** — GraphRAG computes hierarchical community summaries (expensive); LightRAG skips this and instead uses the graph structure + dual-level retrieval directly, which is the main source of its speed/cost advantage.
- **Incremental / streaming updates** — the graph can be updated incrementally as new documents arrive, without full re-indexing.
- **Lightweight** — designed to run on commodity hardware / local-first deployments.

---

## 3. Key benchmarks & performance gains

The paper reports LightRAG outperforming or matching GraphRAG on **retrieval accuracy** while being **substantially cheaper and faster**. Reported headline results (from the paper / official README):

- **Retrieval accuracy:** LightRAG achieves **higher or comparable accuracy** than GraphRAG across the evaluated datasets (the paper reports improvements on several QA benchmarks).
- **Cost reduction:** LightRAG reduces **token cost by roughly 1–2 orders of magnitude** (the paper cites up to ~**100×** lower cost in some configurations) versus GraphRAG, primarily because it avoids the community-summarization pass.
- **Speed:** indexing and querying are significantly faster than GraphRAG due to the lightweight graph and no hierarchical summarization.
- **Incremental updates:** LightRAG supports efficient incremental graph updates, a practical advantage for continuously growing corpora.

> **Caveat:** exact numbers vary by dataset and configuration. The authoritative figures are in the paper's tables ([arXiv v2](https://arxiv.org/html/2410.05779v2) / [ACL PDF](https://aclanthology.org/2025.findings-emnlp.568.pdf)) and the [official README](https://github.com/HKUDS/LightRAG?tab=readme-ov-file). Third-party comparisons (e.g., [fast-graphrag benchmarks](https://github.com/circlemind-ai/fast-graphrag/blob/main/benchmarks/README.md), [Graph RAG in 2026 production guide](https://www.paperclipped.de/en/blog/graph-rag-production/)) should be read as independent, not authoritative.

---

## 4. Common implementation pitfalls

### 4.1 Deduplication / entity-merging pitfalls
- **Over- or under-merging of entities.** The LLM-driven deduplication is heuristic. It can merge distinct entities that merely share a name, or fail to merge true duplicates. See [Issue #495 — "Poor node deduplication - how to have more control?"](https://github.com/HKUDS/LightRAG/issues/495).
- **No deterministic control.** Because deduplication is LLM-judged, results are non-deterministic and hard to tune; users report wanting more control over the merge threshold/behavior.
- **Cross-file / cross-upload duplicates.** Duplicate content uploaded under different filenames can create duplicate nodes unless normalized. See [PR #3078 — "dedupe cross-filename uploads via merged_text normalization"](https://github.com/HKUDS/LightRAG/pull/3078).
- **Insertion-phase dedup is optional.** Entity deduplication during insertion is a feature that must be enabled/configured; it is not always on by default. See [PR #2102 — "Use LLM to deduplicate extracted similar entities during the insertion phase"](https://github.com/HKUDS/LightRAG/pull/2102).

### 4.2 Retrieval / query-mode pitfalls
- **Choosing the wrong query mode.** `local` vs `global` vs `hybrid` materially changes results. Using `local` for broad questions (or `global` for specific facts) degrades answer quality. `hybrid` is the safe default for mixed queries.
- **Graph quality dominates retrieval.** If extraction/deduplication is poor, dual-level retrieval inherits the noise — garbage in, garbage out.
- **Vector store / embedding consistency.** Naive mode relies on embeddings; mixing embedding models or failing to persist the vector index breaks retrieval across restarts.

### 4.3 Cost & scale pitfalls
- **LLM extraction cost.** The extraction + deduplication passes are LLM-heavy; on very large corpora the token cost can still be significant even though it is far below GraphRAG.
- **Graph growth without dedup.** Skipping/weakening deduplication causes unbounded graph growth and rising retrieval latency/cost over time.
- **Non-determinism across runs.** LLM-driven extraction/dedup means the same corpus can yield different graphs across runs, complicating reproducibility and testing.

### 4.4 Operational pitfalls
- **Version drift.** The API and storage format change across releases (e.g., v1.4.x vs main); pin a version and read the matching docs ([Algorithm.md](https://github.com/HKUDS/LightRAG/blob/v1.4.13/docs/Algorithm.md) is version-tagged).
- **Storage backend choice.** LightRAG supports multiple storage backends (SQLite, Neo4j, etc.); behavior and performance differ, so pick per deployment constraints (local-first → SQLite; graph-native → Neo4j).

---

## 5. Source URL list

**Primary / paper**
- https://arxiv.org/html/2410.05779v2
- https://aclanthology.org/2025.findings-emnlp.568/
- https://aclanthology.org/2025.findings-emnlp.568.pdf
- https://doi.org/10.18653/v1/2025.findings-emnlp.568

**Official project**
- https://lightrag.github.io/
- https://github.com/HKUDS/LightRAG/
- https://github.com/HKUDS/LightRAG/blob/v1.4.13/docs/Algorithm.md
- https://github.com/HKUDS/LightRAG/blob/47398a1060efe793bd09e8944867085ddf6c1416/docs/Reproduce.md
- https://github.com/HKUDS/LightRAG/blob/main/docs/ProgramingWithCore.md
- https://github.com/HKUDS/LightRAG/blob/e675598db402da0241f089a22f6939dfbe223437/lightrag/operate.py
- https://github.com/HKUDS/LightRAG/blob/568c0077/lightrag/api/routers/query_routes.py

**Community / deep-dive**
- https://deepwiki.com/HKUDS/LightRAG/2.3-retrieval-and-query-modes
- https://deepwiki.com/HKUDS/LightRAG/1.2-core-concepts-and-terminology
- https://neo4j.com/blog/developer/under-the-covers-with-lightrag-retrieval/

**Pitfalls / issues / PRs**
- https://github.com/HKUDS/LightRAG/issues/495
- https://github.com/HKUDS/LightRAG/discussions/1526
- https://github.com/HKUDS/LightRAG/issues/1631
- https://github.com/HKUDS/LightRAG/pull/3078
- https://github.com/HKUDS/LightRAG/pull/2102

**Independent comparisons**
- https://github.com/circlemind-ai/fast-graphrag/blob/main/benchmarks/README.md
- https://www.paperclipped.de/en/blog/graph-rag-production/
- https://doi.org/10.3390/ai6030047
