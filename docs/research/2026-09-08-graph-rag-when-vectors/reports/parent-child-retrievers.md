# Parent-Child Retrievers (Hierarchical Chunking / Retrieval)

- **Topic:** Parent-child retrievers — hierarchical chunking and retrieval where
  small "child" chunks are indexed for precise retrieval but the larger "parent"
  chunk (or document) is returned as context.
- **Date:** 2026-09-08
- **Tier:** SHOULD
- **Report slug:** `parent-child-retrievers`
- **Primary consumers:** Astrographer, Zodiac. Secondary: Incanter, Familiar,
  Solomon, Astral, Mystery.
- **Status:** Web-grounded research report (not a behavior contract). Does not
  modify any `docs/specs/*.md`, code, test, or tracker.

---

## §1 What the technique is (web-grounded, cited)

Parent-child retrieval (also called **hierarchical chunking**, **small-to-big
retrieval**, or **parent-document retrieval**) is a two-level retrieval pattern
that decouples the *indexing unit* from the *context unit*:

1. **Index small "child" chunks.** The corpus is split into small chunks
   (sentences, short paragraphs, or small graph nodes) that are embedded and
   indexed. Small chunks give **precise, high-recall retrieval** — the query
   matches a tight, relevant unit rather than a large noisy block.
2. **Return the larger "parent" as context.** Each child carries a pointer to a
   larger **parent** (a section, a full document, or a higher-level node). When
   a child is retrieved, the retriever returns the **parent** (or expands the
   child to the parent) so the generator LLM sees surrounding context, not just
   the isolated snippet.

The canonical implementations are the **LangChain `ParentDocumentRetriever`**
([source](https://github.com/langchain-ai/langchain/blob/85a5a04210452aec2eb3a06a02961c8fefd5c8b6/libs/langchain/langchain_classic/retrievers/parent_document_retriever.py),
[explainer](https://zeroentropy.dev/concepts/parent-document-retrieval/)) and
the **LlamaIndex auto-merging / sentence-window / recursive retrievers**
([auto-merging](https://developers.llamaindex.ai/python/framework/integrations/retrievers/auto_merging_retriever/),
[sentence-window](https://developers.llamaindex.ai/python/framework-api-reference/packs/sentence_window_retriever/),
[recursive](https://github.com/run-llama/llama_index/blob/main/llama-index-packs/llama-index-packs-recursive-retriever/README.md)).
Haystack ships an `AutoMergingRetriever` with the same idea
([docs](https://docs.haystack.deepset.ai/docs/automergingretriever)).

**The four main variants** (they differ in how "parent" is defined and how the
expansion happens):

- **Parent-document retriever** — child = small chunk, parent = the whole
  document. Retrieve the chunk, return the full document. Simplest and most
  common ([Thread Transfer](https://thread-transfer.com/blog/2025-07-29-parent-document-retrieval/),
  [Barking Iguana](https://barkingiguana.com/writing/parent-document-retrieval-small-chunks-big-context/)).
- **Sentence-window retriever** — child = a single sentence, parent = a window
  of surrounding sentences. Returns the sentence plus its local context
  ([LlamaIndex](https://developers.llamaindex.ai/python/framework-api-reference/packs/sentence_window_retriever/),
  [TruLens evaluation](https://www.trulens.org/cookbook/frameworks/llamaindex/llama_index_sentencewindow/)).
- **Auto-merging retriever** — a hierarchical tree of chunks; when enough
  children of a parent are retrieved, the children are **merged back into the
  parent** and the parent is returned instead
  ([LlamaIndex](https://developers.llamaindex.ai/python/framework/integrations/retrievers/auto_merging_retriever/)).
- **Recursive retriever** — retrieve at one level, then **recursively expand to
  related nodes** (e.g. from a summary node to the underlying document nodes)
  ([LlamaIndex recursive pack](https://github.com/run-llama/llama_index/blob/main/llama-index-packs/llama-index-packs-recursive-retriever/README.md),
  [The Neural Base](https://theneuralbase.com/advanced-rag/learn/beginner/recursive-retrieval/)).

**Why it works.** Small chunks fix the classic chunking trade-off: large chunks
dilute the embedding (a query matches a big block that is only partly relevant),
while small chunks lose the surrounding context the generator needs. Parent-child
retrieval gets **both**: small-chunk precision at retrieval time and
large-context completeness at generation time
([Learnixo small-to-big](https://learnixo.io/blog/adv-rag-small-to-big),
[AI/TLDR](https://ai-tldr.dev/learn/rag/advanced-rag/parent-document-retrieval/)).
The trade-off is **context-window bloat**: returning the full parent for every
hit can flood the context window, so the parent size and the number of expanded
hits must be capped ([dreaming.press comparison](https://dreaming.press/posts/parent-document-vs-sentence-window-retrieval.html),
[Lesson 8.4](https://vivedhaelango.substack.com/p/lesson-84-parent-child-retrieval)).

**Relationship to the suite's prior RAG research.** Parent-child retrieval is
orthogonal to and composes with the techniques already researched for the suite:
it is a **retrieval-structure** technique (like the graph-aware retrieval in
`dag-rag-research-notes.md`), whereas reranking, multi-query, contextual
compression, and HyDE are **scoring/expansion** techniques. It pairs naturally
with **contextual compression** (retrieve the parent, then compress it down to
the query-relevant span) and with **reranking** (retrieve children broadly,
rerank, then expand only the top hits to their parents). The hierarchical
abstraction also echoes **RAPTOR**'s recursive tree summarization
([RAPTOR, ICLR 2024](https://proceedings.iclr.cc/paper_files/paper/2024/file/8a2acd174940dbca361a6398a4f9df91-Paper-Conference.pdf))
and the recursive-semantic chunking literature
([The Chunking Paradigm, ICNLSP 2025](https://aclanthology.org/2025.icnlsp-1.15.pdf)).

---

## §2 How it applies to Astrographer

Astrographer is the suite's **Graphical RAG engine + document store/wiki**
(`docs/specs/astrographer.md`). Its documents are **Provident graphs** (nodes +
edges), and its RAG surface consumes the Incanter engine over HTTP
(`astrographer.md` §4.3). Parent-child retrieval maps onto this model in several
concrete, graph-native ways.

**The natural parent/child split is already present in the contract.** A
**Document** is the atomic unit of authoring (`astrographer.md` §4.1.1) — the
natural **parent**. Its **nodes** (`content`, `fact`, `reference`; §4.2.1) are
the natural **children**. Incanter's **spring-tension dynamic chunking**
(`incanter.md` §4.6) already produces child chunks with stable `token_start`/
`token_end` ranges and `chunk_id`s — the child-generation mechanism exists.

**Concrete mapping to the RAG surface.** `ragQuery`/`ragStream`
(`astrographer.md` §4.3.2) return `results: [{documentId, nodeId, score,
snippet, source}]` with `topK` 1–50. A parent-child pass would:
1. Retrieve small child chunks (or nodes) via Incanter at high recall.
2. Expand each hit to its **parent Document** (or a parent section subgraph)
   before generation, so the generator sees the surrounding graph context, not
   just the isolated `snippet`.

**The fact/reference model makes this a MUST-adjacent narrow case.** The
defining invariant is that a `fact` node's canonical `value` is the single source
of truth, referenced/embedded elsewhere (`astrographer.md` §4.2.3). When a query
retrieves a `reference` or `embed` child, the correct context is the **canonical
`fact`** (and the document that owns it) — not the stale snapshot. This is
exactly a parent-child expansion: retrieve the child, resolve to the parent
fact/document. The `link`/`embed` resolution rules (§4.2.2–4.2.3) already define
how a reference resolves to its target; parent-child retrieval is the natural
retrieval-time application of that resolution. This narrow case (resolve a
retrieved reference/embed to its canonical fact) is effectively required for the
single-source-of-truth model to be usable in RAG — the same reasoning that made
exact `factKey` retrieval a narrow MUST in `hybrid-search-research-notes.md`.

**D4 parity.** Parent-child retrieval is a retrieval-mode change, not a
security-configuration feature, so it must be reachable through **both** the GUI
RAG panel (`astrographer.md` §4.6.2) and the `rag_query`/`rag_stream` MCP tools
(§4.5.1). It would surface as a query-mode parameter (e.g. `expand: 'parent'`),
not a new tool — keeping parity with the same pattern used for the compression
mode in `contextual-compression-research-notes.md`.

**Status in the spec:** **not present** — this is a **proposed spec change**.
`astrographer.md` §4.3.2 pins the `RagResult` shape and `topK`/`filters` but has
no parent-expansion parameter. The `snippet` field is the only context returned;
there is no "return the parent document/subgraph" option. Adding a parent-child
mode is a contract addition to §4.3.2 (and the MCP/GUI surfaces in §4.5.1/§4.6.2).

---

## §3 How it applies to Zodiac

Zodiac is the suite's **backend/remote RAG companion** that pre-graphs and
vector-embeds web-crawled data (`docs/specs/zodiac.md`). Parent-child retrieval
maps onto its pipeline.

**The RAG engine already has a two-level structure.** Zodiac **pre-graphs** raw
crawl material into a graph and **vector-embeds** it into a vector store
(`zodiac.md` §4.2). Each embedded item carries a stable `embedding id` and a
reference to its source crawl id. The **parent** is the pre-graphed document (or
the target's graph); the **children** are the embedded items. A parent-child pass
would retrieve small embedded items (children) and expand to the parent graph or
document for context.

**Query-reply surface.** `zodiac_query` (`zodiac.md` §4.3.1–4.3.2) supports
`mode: graph | vector | hybrid` and returns `results: [{type, itemId, snippet,
sourceCrawlId, stale}]`. A parent-child mode would add an `expand` option that
returns the parent graph/document alongside the matched item. The `stale` flag
(§4.3.2, §6.11) is directly relevant: when a child is retrieved from a `STALE`
RAG store, expanding to the parent should carry the staleness marker so the
generator does not trust stale content — the same reference-aware pattern noted
for Astrographer's `embed` snapshots.

**D4 parity.** `zodiac_query` is the MCP tool (`zodiac.md` §4.5); the GUI query
builder is the parity surface (§5.1). A parent-child `expand` mode must be
exposed on both.

**Status in the spec:** **not present** — this is a **proposed spec change**.
`zodiac.md` §4.3.2 pins the result shape and `mode` enum but has no parent
expansion. Adding it is a contract addition to §4.3.2 and the MCP surface §4.5.

---

## §4 How it applies to the other suite consumers

**Incanter** (`docs/specs/incanter.md`). Incanter is the prototype Rust Graph-RAG
engine whose **spring-tension dynamic chunking** (`incanter.md` §4.6) is the
child-generation mechanism. The `ChunkDto` (`chunk_id`, `token_start`,
`token_end`, `token_count`, `boundary_tension`) is a natural child unit; the
parent is the document. Parent-child retrieval is an **engine-side capability**
— the cleanest home is inside Incanter's `POST /v1/query` (`incanter.md` §4.8),
so Astrographer's `ragQuery` stays a single call (the same engine-boundary
argument made for multi-query in `multi-query-retrieval-research-notes.md` and
recorded as `GAP-1` in `docs/defects.md`). Because Incanter is "likely not the
final version" (INCANTER-DISPOSITION), this is a **proposed spec change** to the
prototype contract, not a MUST for the production engine.

**Familiar** (`docs/specs/familiar.md`). Familiar's memory store is a **facts
table** (`familiar.md` §4.1.3). The parked **vector-search layer** of the
agent-memory research (`docs/pending.md` D6) calls for a local embedding store
with **per-exchange chunking** — a natural parent-child structure (exchange =
parent, chunk = child). Parent-child retrieval would let Familiar retrieve a
small memory chunk and expand to the full exchange/note for context. This is a
**parked/speculative** item (D6), not in the current spec.

**Solomon** (`docs/specs/solomon.md`). Solomon's cross-instance search
(`solomon.md` §4.2) returns `results: [{peerId, instanceType, itemId, snippet}]`
and aggregates across peers (§4.2.2). Parent-child retrieval applies to how a
retrieved `snippet` expands to the full document on the source instance — but
Solomon is a **discovery/aggregation** layer, not a retriever; the expansion
would be delegated to the source instance's own RAG surface. This is a
**NICE-TO-HAVE / follow-up**, not a core Solomon change.

**Astral** (`docs/specs/astral.md`). Astral is a **webhost** for pushed Provident
graphs (`astral.md` §4.1), not a retriever. Parent-child retrieval is not
relevant to its core serving contract; it only matters if Astral later exposes a
search surface over hosted units. **Not applicable / parked.**

**Mystery.** `docs/specs/mystery.md` does not exist; the Familiar→Mystery edge
(F3) is **deferred** (`docs/pending.md`, `familiar.md` §5.6). No parent-child
analysis is possible until the Mystery contract exists. **Deferred.**

---

## §5 Recommendation for the suite

**Tier: SHOULD** (with a narrow MUST-adjacent case for Astrographer's
fact/reference resolution).

**Rationale, grounded in D1–D4:**

- **D2 (local-first).** Parent-child retrieval is fully local: it is an
  ingestion-time index build (child→parent pointers) plus a query-time expansion
  step. No cloud dependency. It reuses the local Ollama embedding leg already
  used by Incanter (`incanter.md` §4.7) and the local BM25/vector infrastructure
  from `hybrid-search-research-notes.md`. **Fully D2-compliant.**
- **D1 (open-source, AGPL-3.0).** The technique is a well-known retrieval pattern
  with open implementations (LangChain, LlamaIndex, Haystack). No licensing
  constraint; it can be implemented in the suite's own repos. **D1-compliant.**
- **D4 (MCP-GUI parity).** Parent-child retrieval is a retrieval-mode change, not
  a security-configuration feature, so it must be exposed on both the GUI and the
  MCP surface (`rag_query`/`rag_stream`, `zodiac_query`). It surfaces as a query
  parameter, not a new tool — the same parity pattern as the compression mode.
  **D4-compliant if exposed on both surfaces.**
- **D3 (interconnection).** Parent-child retrieval is a natural cross-tool
  capability: Astrographer and Zodiac both have a parent/child structure, and
  Solomon can delegate snippet→document expansion to the source instance. It
  strengthens the interconnection story. **D3-aligned.**

**Why SHOULD, not MUST.** The document-store/wiki core, the cross-link/embed
consistency invariant, and the RAG surface all work without parent-child
retrieval. It is an **enhancement to retrieval quality** (more context for the
generator), not a prerequisite for the core features — the same reasoning that
placed reranking, multi-query, contextual compression, and HyDE at SHOULD.

**Why not NICE-TO-HAVE / DISCARD.** The cost is low (a child→parent index + a
query-time expansion) and the benefit is direct: it fixes the small-chunk-vs-
big-context trade-off that a graph RAG engine faces when it returns isolated
nodes/snippets. Discarding it would leave a real quality gap in the RAG surface.

**The narrow MUST-adjacent case.** For Astrographer, resolving a retrieved
`reference`/`embed` child to its canonical `fact`/document is effectively
required for the single-source-of-truth model to be usable in RAG — the same
reasoning that made exact `factKey` retrieval a narrow MUST in
`hybrid-search-research-notes.md`. This specific resolution should be treated as
a higher-priority slice of the SHOULD.

**Suggested disposition:**
1. **SHOULD — Astrographer parent-child mode.** Add an `expand`/parent mode to
   `ragQuery`/`rag_stream` (`astrographer.md` §4.3.2) that returns the parent
   Document (or section subgraph) for a retrieved child, with a cap on expanded
   hits to bound context-window bloat. Expose on both GUI (§4.6.2) and MCP
   (§4.5.1) for D4 parity. **Proposed spec change.**
2. **SHOULD — Astrographer fact/reference resolution (narrow MUST).** When a
   retrieved child is a `reference`/`embed`, expand to the canonical `fact` and
   its owning document, honoring the §4.2.3 consistency rules (and the `STALE`
   marker). **Proposed spec change.**
3. **SHOULD — Zodiac parent expansion.** Add an `expand` option to `zodiac_query`
   (`zodiac.md` §4.3.2) that returns the parent graph/document for a matched
   embedded item, carrying the `stale` flag. **Proposed spec change.**
4. **NICE-TO-HAVE — Incanter engine-side parent-child.** Add parent expansion to
   Incanter's `POST /v1/query` (`incanter.md` §4.8) so Astrographer's `ragQuery`
   stays a single call (engine boundary). Record as an Incanter handoff
   (`docs/defects.md`/`docs/HANDOFF.md`), not a patch from this repo.
5. **PARKED — Familiar memory vector-search parent-child.** The per-exchange
   chunking of the parked D6 vector-search layer (`docs/pending.md` D6) is a
   natural parent-child structure; land it with that layer.
6. **PARKED — Solomon snippet→document expansion.** Delegate expansion to the
   source instance's RAG surface; not a core Solomon change.

---

## §6 Source URL list

- [LangChain `ParentDocumentRetriever` (source)](https://github.com/langchain-ai/langchain/blob/85a5a04210452aec2eb3a06a02961c8fefd5c8b6/libs/langchain/langchain_classic/retrievers/parent_document_retriever.py)
- [Parent-document retrieval: small-chunk index, full-doc context — ZeroEntropy](https://zeroentropy.dev/concepts/parent-document-retrieval/)
- [Parent Document Retrieval: Context-Aware Chunking — Thread Transfer](https://thread-transfer.com/blog/2025-07-29-parent-document-retrieval/)
- [Haystack `AutoMergingRetriever` — docs](https://docs.haystack.deepset.ai/docs/automergingretriever)
- [LlamaIndex Auto Merging Retriever — developer docs](https://developers.llamaindex.ai/python/framework/integrations/retrievers/auto_merging_retriever/)
- [LlamaIndex recursive retriever pack — README](https://github.com/run-llama/llama_index/blob/main/llama-index-packs/llama-index-packs-recursive-retriever/README.md)
- [LlamaIndex sentence-window retriever — API reference](https://developers.llamaindex.ai/python/framework-api-reference/packs/sentence_window_retriever/)
- [Small-to-Big Retrieval — Learnixo](https://learnixo.io/blog/adv-rag-small-to-big)
- [Recursive retrieval — The Neural Base](https://theneuralbase.com/advanced-rag/learn/beginner/recursive-retrieval/)
- [The Chunking Paradigm: Recursive Semantic for RAG Optimization — ICNLSP 2025](https://aclanthology.org/2025.icnlsp-1.15.pdf)
- [RAPTOR: Recursive Abstractive Processing for Tree-Organized Retrieval — ICLR 2024](https://proceedings.iclr.cc/paper_files/paper/2024/file/8a2acd174940dbca361a6398a4f9df91-Paper-Conference.pdf)
- [What Is Parent Document Retrieval in RAG? — AI/TLDR](https://ai-tldr.dev/learn/rag/advanced-rag/parent-document-retrieval/)
- [Parent Document vs Sentence Window vs Auto-Merging Retrieval — dreaming.press](https://dreaming.press/posts/parent-document-vs-sentence-window-retrieval.html)
- [Lesson 8.4: Parent-child retrieval and small-to-big chunking — Vivedha Elango](https://vivedhaelango.substack.com/p/lesson-84-parent-child-retrieval)
- [Parent-Document Retrieval: Small Chunks, Big Context — Barking Iguana](https://barkingiguana.com/writing/parent-document-retrieval-small-chunks-big-context/)
- [Evaluating Sentence Window RAG — TruLens](https://www.trulens.org/cookbook/frameworks/llamaindex/llama_index_sentencewindow/)
