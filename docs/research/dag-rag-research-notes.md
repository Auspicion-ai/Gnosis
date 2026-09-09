# DAG-based RAG (Directed Acyclic Graph answer/retrieval structure) — Research Notes

**Date:** 2025-09-07
**Topic:** DAG-based RAG — organizing retrieval/answer structure as a directed acyclic graph of sub-tasks or document relationships
**Status:** Research / reference notes (not a spec)
**Audience:** Astrographer (Graphical RAG engine + document store/wiki over Provident graphs; consumes Incanter Graph-RAG via `ragQuery`/`ragStream`)

---

## 1. Mechanism — how DAG-based RAG works

DAG-based RAG treats the *reasoning/answer structure* of a query as a
**directed acyclic graph** rather than a flat retrieve-then-answer pipeline or a
linear chain. Two distinct (often conflated) flavors:

1. **Sub-task DAG (decomposition + dependency ordering).** The query is
   decomposed into sub-problems/sub-questions, and a DAG models the **logical
   dependencies** among them. Retrieval and reasoning then proceed in
   **topological order** — a sub-task is only answered once its dependencies
   are resolved, and its answer feeds the next sub-task. This is the core of
   **LogicRAG** ("You Don't Need Pre-built Graphs for RAG", AAAI 2026): it
   decomposes the query, builds a DAG of sub-problems, **linearizes the graph
   via topological sort**, and retrieves/answers sub-problems in that order to
   support coherent multi-step reasoning — all at inference time, with **no
   pre-built graph**.
2. **Document/evidence DAG (relationship structure).** The retrieved evidence
   is organized as a graph of documents/entities and their relationships, and
   answer generation walks that graph. **DAGR** (Decomposition Augmented Graph
   Retrieval, 2025) decomposes complex queries, retrieves **linked textual
   subgraphs** guided by a weighted similarity over both the original and
   decomposed queries, and builds a **question-specific knowledge graph** to
   guide answer generation — suited to multi-hop QA over graph-structured data.

The unifying idea: **structure the answer as a DAG so each node's computation
depends only on its ancestors** — dependency-ordered, multi-step reasoning with
acyclicity guaranteeing termination and a valid evaluation order.

---

## 2. Problem solved

- **Dependency-ordered, multi-step reasoning.** Flat RAG retrieves once and
  answers; it fails on multi-hop questions where intermediate facts must be
  established before later ones. A DAG makes the dependency order explicit and
  enforces it via topological sort.
- **No pre-built graph required (LogicRAG).** Pre-built GraphRAG graphs are
  costly to build (token cost, update latency) and may not match the logic
  structure a given query needs. Building the DAG **at inference time** avoids
  that cost and adapts the structure to the query.
- **Acyclicity guarantees termination** and a well-defined evaluation order —
  no cycles, no infinite retrieval loops, deterministic sub-task resolution.

---

## 3. Astrographer synergy

Astrographer's documents are **Provident graphs** — directed topological
multigraphs (nodes + edges). This is a natural fit for DAG-based RAG:

- The wiki's **reference graph** (`reference` → `fact` edges) is already a
  directed structure. Resolving a `reference` node's value requires resolving
  its target `fact` first — a **dependency-ordered** resolution that is
  exactly a topological walk over the reference graph.
- A `fact` node is the **single source of truth**; a DAG evaluation order lets
  Astrographer resolve facts in dependency order and reuse each resolved fact
  across multiple dependents (no redundant re-resolution).
- **Incanter integration:** `ragQuery`/`ragStream` can be driven to answer
  sub-tasks in topological order, streaming each resolved sub-answer into the
  next. The DAG can be derived from the Provident graph's existing edges
  (reference→fact) rather than built from scratch.
- **D2 local-first / D1 open-source:** inference-time DAG construction (LogicRAG
  style) needs no external graph service — it runs locally over the Provident
  graph. **D4 MCP-GUI parity:** the DAG plan and per-node resolution are both
  exposable as MCP endpoints and GUI steps.

---

## 4. Recommendation

- **MUST HAVE** — **Topological resolution of `reference` → `fact` dependencies.**
  The Provident graph is already a directed multigraph; resolving references in
  topological order is the core, low-risk win and directly enables
  dependency-ordered multi-step answers. This is the foundation everything else
  builds on.
- **SHOULD HAVE** — **Inference-time sub-task DAG (LogicRAG-style) for complex
  queries.** Decompose multi-hop queries into a sub-problem DAG, topologically
  sort, and drive `ragQuery`/`ragStream` per node. High value for multi-hop
  wiki questions; moderate complexity. Build on the MUST-HAVE topological
  resolver.
- **NICE TO HAVE** — **Question-specific evidence subgraph (DAGR-style).**
  Retrieve linked textual subgraphs over the reference graph and build a
  per-query knowledge graph to guide generation. Adds fidelity for graph-heavy
  queries but overlaps with the SHOULD-HAVE; defer.
- **DISCARD** — **Pre-built global GraphRAG graph construction** (RAPTOR-style
  recursive abstraction / full-corpus graph build). Conflicts with D2
  local-first cost/latency and D1 simplicity; the Provident graph already
  provides the structure, so a separate pre-built graph is redundant.

---

## 5. Pitfalls

1. **DAG ≠ tree.** A node may have multiple parents; naive "chain" reasoning
   misses shared dependencies. Use a proper topological sort, not a linear
   decomposition, or you lose the acyclicity guarantee.
2. **Decomposition quality is the ceiling.** If the query is decomposed into
   wrong sub-problems, the DAG is wrong regardless of retrieval quality — the
   router/decomposer, not the retriever, bounds accuracy (same lesson as
   Adaptive-RAG's classifier).
3. **Cycles in the source graph.** The Provident graph is a multigraph; a
   `reference`→`fact` cycle (or a fact referencing itself transitively) breaks
   topological ordering. Detect and break cycles (or park them) before
   resolution.
4. **Token/latency cost of per-node retrieval.** Each DAG node may trigger a
   retrieval + LLM call; a wide DAG multiplies cost. Cache resolved facts
   (single source of truth) and reuse across dependents.
5. **"DAG-based RAG" is overloaded.** It covers both sub-task DAGs (LogicRAG)
   and evidence/document DAGs (DAGR). Be explicit about which structure
   Astrographer builds, or the design and tests will be ambiguous.

---

## Source URLs

- LogicRAG — *You Don't Need Pre-built Graphs for RAG: Retrieval Augmented Generation with Adaptive Reasoning Structures* (AAAI 2026): https://arxiv.org/html/2508.06105v1 and https://ojs.aaai.org/index.php/AAAI/article/view/40278
- DAGR — *Decomposition Augmented Graph Retrieval with LLMs*: https://arxiv.org/abs/2506.13380 and https://arxiv.org/html/2506.13380v2
- StepChain GraphRAG — *Reasoning Over Knowledge Graphs for Multi-Hop QA*: https://arxiv.org/pdf/2510.02827
- HopRAG — *Multi-Hop Reasoning for Logic-Aware RAG*: https://aclanthology.org/2025.findings-acl.97.pdf
- PathwiseRAG — *Multi-Dimensional Exploration and Integration Framework*: https://aclanthology.org/2025.emnlp-main.1167.pdf
- GraphRAG — *From Local to Global: A GraphRAG Approach to Query-Focused Summarization*: https://arxiv.org/pdf/2404.16130
- DAG-RAG reference repo (community): https://github.com/WYI1223/DAG-RAG
