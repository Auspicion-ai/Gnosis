# Explainability and governance (sourcing and tracing)

- **Topic:** RAG explainability and governance — provenance, citation, and
  traceable retrieval paths. The source talk's point that vector RAG "lacks
  explainability" and that "governance questions are more easily answered by
  sourcing and tracing."
- **Date:** 2026-09-08
- **Tier:** MUST
- **Report slug:** `explainability-governance`
- **Primary focus:** Astrographer, Zodiac. **Secondary:** Incanter, Familiar,
  Solomon, Mystery, Astral.
- **Status:** Web-grounded research report (documentation deliverable only — no
  spec/code/test changes).

---

## §1 What the technique is (web-grounded, cited)

**Explainability in RAG** is the ability to show *why* a generated answer is
what it is — which retrieved sources it was grounded on, how those sources were
retrieved, and what path the retrieval walked to reach them. **Governance** is
the operational layer on top: the ability to **source** (attribute an answer to
specific documents/facts), **trace** (reconstruct the retrieval path), and
**audit** (record who asked what, when, and what was returned) so that
compliance, accountability, and trust questions can be answered after the fact.

The source talk frames the failure of pure vector RAG as: *"Lack
explainability. Governance questions are more easily answered by sourcing and
tracing."* The web-grounded literature supports this and gives it a concrete
shape:

1. **Grounded attribution / citation.** A trustworthy RAG answer must be
   attributable to the retrieved sources it was generated from. *Measuring and
   Enhancing Trustworthiness of LLMs in RAG through Grounded Attributions and
   Learning to Refuse* ([arXiv:2409.11242](https://arxiv.org/pdf/2409.11242))
   shows that grounding answers in retrieved evidence and *refusing* when no
   evidence supports the answer measurably improves trustworthiness. *Model
   Internals-based Answer Attribution for Trustworthy RAG*
   ([ACL 2024](https://aclanthology.org/2024.emnlp-main.347.pdf)) attributes
   each answer span to the source tokens that produced it. *RAG with Grounded
   Citations: Making Retrieval Answers Provable*
   ([aiarch.dev](https://aiarch.dev/patterns/grounded-rag)) frames citation as
   the mechanism that makes retrieval answers *provable*.

2. **Retrieval provenance / traceability.** Provenance is the record of where a
   retrieved item came from and how it was reached. *Retrieval provenance*
   ([Gautier Dorval](https://gautierdorval.com/en/definitions/retrieval-provenance/))
   defines it as the traceable origin of retrieved content; *Provenance for
   Auditable RAG* ([IJFMR](https://www.ijfmr.com/papers/2026/3/80826.pdf))
   argues provenance is the prerequisite for an auditable RAG system. In the
   graph-RAG setting, *Evaluating GraphRAG for Traceable and Interpretable
   Question Answering* ([IEEE](https://doi.org/10.1109/acdsa67686.2026.11468256))
   and *XGRAG: A Graph-Native Framework for Explaining KG-based RAG*
   ([arXiv:2604.24623](https://doi.org/10.48550/arxiv.2604.24623)) show that a
   knowledge graph gives a *walkable* trace path (nodes/edges) that a flat
   vector index cannot. TrustGraph's *Explainability*
   ([docs.trustgraph.ai](https://docs.trustgraph.ai/overview/explainability.html))
   and *query-time explainability*
   ([github.com/trustgraph-ai/trustgraph](https://github.com/trustgraph-ai/trustgraph/blob/4e3bd85a/docs/tech-specs/query-time-explainability.md))
   are concrete open-source implementations of traceable, explainable graph RAG.

3. **Governance: source authority, access control, freshness, auditability.**
   *RAG Governance: Source Authority, Access Control, Freshness, and
   Auditability* ([thomasthelliez.com](https://thomasthelliez.com/blog/rag-governance-source-authority-access-control-auditability/))
   names the four governance axes: which sources are authoritative, who may
   access them, how fresh they are, and whether every answer is auditable.
   *RAG governance: retrieval and inference control*
   ([Gautier Dorval](https://gautierdorval.com/en/frameworks/rag-governance-retrieval-and-inference-control/))
   and *Enterprise RAG Architecture: The Reference Model*
   ([scadea.com](https://scadea.com/enterprise-rag-and-permission-aware-retrieval/))
   extend this to permission-aware retrieval and inference control. *How to
   Build an Auditable Enterprise RAG Pipeline*
   ([DocuShell](https://docushell.com/blog/how-do-you-build-an-auditable-rag-pipeline-for-enterprise-documents))
   and *ContextNest: Verifiable Context Governance for Autonomous AI Agents*
   ([arXiv:2607.02116](https://arxiv.org/html/2607.02116v2)) describe the
   audit-log and context-governance layers that make RAG defensible in
   regulated environments.

4. **Evaluation.** Explainability is only meaningful if it can be measured.
   *RAGAS: Automated Evaluation of Retrieval Augmented Generation*
   ([ACL 2024](https://aclanthology.org/2024.eacl-demo.16.pdf),
   [arXiv:2309.15217](https://arxiv.org/html/2309.15217v1)) defines the standard
   RAG metrics — **faithfulness** (is the answer grounded in the retrieved
   context?) and **answer relevance** (does the answer address the question?) —
   which are the quantitative face of the talk's "sourcing and tracing."
   *Disentangling Faithfulness Hallucinations in RAG*
   ([Springer](https://link.springer.com/article/10.1007/s10994-026-07121-y))
   provides a systematic benchmark for faithfulness failures.

**Net for the suite.** Explainability/governance is not a single feature but a
**capability stack**: (a) **provenance** — every retrieved item carries its
source identity; (b) **trace path** — the retrieval path (edges walked) is
reconstructible; (c) **citation/grounding** — the answer cites the sources it
was generated from; (d) **audit** — queries and answers are logged for
after-the-fact review; (e) **evaluation** — faithfulness/answer-relevance are
measured. The suite already has strong provenance primitives in its contracts;
the work is to make them *answer-level* and *auditable*.

---

## §2 How it applies to Astrographer

Astrographer is the suite's strongest explainability surface. Its data model is
a Provident graph (`docs/specs/astrographer.md` §4.1.1) whose consistency model
already tracks every reference to its source — the talk's "sourcing and
tracing" made mechanical.

**Already in the spec (provenance primitives):**
- **Document provenance** (`astrographer.md` §4.1.1): every Document carries
  `documentId` (stable UUID), `revision` (monotonic), `author`, `createdAt`,
  `updatedAt`, `tags`. A result can be traced to a specific document revision
  and author.
- **The consistency report** (`astrographer.md` §4.2.3): `getConsistencyReport
  (wikiId)` returns every `{documentId, nodeId, kind, state, target}` reference
  in the wiki — a machine-readable "what is consistent / what is not" surface.
  This is the talk's "governance questions answered by sourcing and tracing"
  made concrete: an agent can trace a fact to its canonical source and see every
  reference to it. Exposed as MCP `get_consistency_report` (`astrographer.md`
  §4.5.1) and a GUI consistency panel (`astrographer.md` §4.6.2) — **D4 parity
  already satisfied.**
- **Exact-reference resolution** (`astrographer.md` §4.2.1–§4.2.3): a `fact` is
  resolved by exact `factKey` reference (via `link`/`embed`), not by embedding
  similarity. This is the "similarity ≠ relevance" fix and it is inherently
  explainable: the answer's source is the canonical fact node, not a
  semantically-similar-but-wrong node.
- **Push provenance** (`astrographer.md` §4.4.1): the `provident-graph/1` push
  carries `source`, `revision`, `contentHash`, `pushedAt` — so a published page
  can be traced to its pushing tool, revision, and content hash.

**Gap — answer-level citation/trace (proposed spec change).** The RAG result
shape (`astrographer.md` §4.3.2) returns `{documentId, nodeId, score, snippet,
source}` — per-item provenance, but **no trace path** (the edges walked to reach
the answer) and **no answer-level citation** (which retrieved items the
generated answer was actually grounded on). The consistency model already
records every `link`/`embed` target (`astrographer.md` §4.2.3), so a trace path
is cheap to add. **Proposed spec change** to `astrographer.md` §4.3.2: extend
`ragQuery`/`ragStream` results with (a) a `trace` field (the `reference`→`fact`
path walked) and (b) an answer-level `citations` field (the `documentId`/`nodeId`
set the answer is grounded on). This directly answers the talk's governance
point and is consistent with the DAG-RAG research's topological
`reference`→`fact` resolution (`docs/research/dag-rag-research-notes.md` §4).

**Gap — audit log (proposed spec change).** `ragQuery`/`ragStream`
(`astrographer.md` §4.3.2) have no audit trail: there is no record of who asked
what, when, and what was returned. For governance, a query audit log (query,
filters, result set, timestamp, requester) is a small, high-value addition.
**Proposed spec change** to `astrographer.md` §4.3 (a `getQueryAuditLog` surface
+ MCP tool), consistent with D4 (exposable via both GUI and MCP).

**Net for Astrographer:** the provenance substrate is already in the spec
(consistency report, exact-reference resolution, document/push provenance). The
concrete MUST/SHOULD work is **answer-level citation + trace path** on RAG
results and a **query audit log** — both proposed spec changes to §4.3.

---

## §3 How it applies to Zodiac

Zodiac is the suite's second explainability surface. Its RAG store already
carries per-item provenance back to the crawl that produced it.

**Already in the spec (provenance primitives):**
- **Crawl provenance** (`zodiac.md` §4.1): every crawl has a stable `crawl id`,
  a `target`, a `source type` (`dependency`|`web`), a `trigger`
  (`manual`|`scheduled`|`query-driven`), and start/finish timestamps. A result
  can be traced to the crawl that fetched it.
- **Embedding provenance** (`zodiac.md` §4.2): every embedded item carries a
  stable `embedding id` and a reference to its `sourceCrawlId`.
- **Result provenance** (`zodiac.md` §4.3.2): every query result carries
  `{type, itemId, snippet, sourceCrawlId, stale}` — the talk's "sourcing and
  tracing" applied to crawled data. The `stale` flag (`zodiac.md` §4.2, §6.11)
  is the freshness/governance axis: stale data is flagged, not silently served.
- **Correlation** (`zodiac.md` §4.3.3): the `zodiac.query.reply` Augur event
  carries a `correlationId` linking the reply to the originating query — a
  traceable query→reply path.

**Gap — trace path to the source crawl (proposed spec change).** Zodiac results
carry `sourceCrawlId` but not the **graph path** walked to reach the answer in
`graph`/`hybrid` mode (`zodiac.md` §4.3.2). Because Zodiac pre-graphs crawled
data into nodes+edges (`zodiac.md` §4.2), a result can carry the subgraph path
from the query to the answer — the graph-native trace that a flat vector index
cannot provide (per *XGRAG* and *Evaluating GraphRAG for Traceable and
Interpretable QA*, §1). **Proposed spec change** to `zodiac.md` §4.3.2: add a
`trace` field to graph/hybrid results (the nodes/edges walked) alongside the
existing `sourceCrawlId`.

**Gap — answer-level citation (proposed spec change).** Like Astrographer,
Zodiac returns per-item results but no answer-level citation set. **Proposed
spec change** to `zodiac.md` §4.3.2: add a `citations` field (the `itemId` set
the answer is grounded on).

**Net for Zodiac:** the provenance substrate (crawl id, sourceCrawlId, stale,
correlationId) is already in the spec. The concrete addition is a **graph trace
path** and **answer-level citation** on results — proposed spec changes to §4.3.

---

## §4 How it applies to the other suite consumers

- **Incanter** (`docs/specs/incanter.md`): Incanter is the prototype Graph-RAG
  engine Astrographer consumes over HTTP (`incanter.md` §2, §5). Its hybrid
  query result (`incanter.md` §4.8) already returns per-item provenance:
  `QueryResultItem = {chunk_id, doc_id, combined_score, vector_similarity,
  graph_tension_proximity, text}` — each result is traceable to its `doc_id` and
  `chunk_id`. The SSE stream (`incanter.md` §4.10) emits `document_state_changed`
  events with timestamps, giving an engine-side audit trail of document state
  transitions. **Recommendation:** the per-item provenance is already in spec;
  the answer-level citation/trace belongs at the Astrographer client boundary
  (§2), not in the engine. No new Incanter contract change required by this
  research.

- **Familiar** (`docs/specs/familiar.md`): Familiar is the suite's most
  governance-sensitive consumer because it is an agent harness. Its memory store
  is a **facts table** (`familiar.md` §4.1.3) where every fact carries
  `provenance {conversation_id, message_id}`, `confidence`, and a lifecycle
  `status` — the talk's "sourcing and tracing" applied to assistant memory. The
  candidate-fact pipeline (`familiar.md` §4.1.3.1) is **fail-closed**: an
  ungrounded candidate (no resolvable `provenance`) is rejected, and every
  accepted/rejected candidate is logged with a machine-actionable reason
  (**auditability**). The derived profile summary (`familiar.md` §4.1.3.2) is a
  deterministic projection of the facts table keyed to a revision, so staleness
  is detectable. **Recommendation:** Familiar already implements the strongest
  provenance + audit model in the suite (facts provenance, fail-closed
  validation, audit logging). No new Familiar contract change required by this
  research; it inherits answer-level citation through Astrographer's RAG surface
  (`familiar.md` §4.3.1, §4.3.3).

- **Solomon** (`docs/specs/solomon.md`): Solomon is cross-instance search over
  `zodiac|astral|astrographer` instances (`solomon.md` §4.2, §4.5.1). Its result
  aggregation (`solomon.md` §4.2.2) already carries **result provenance**: every
  result is `{peerId, instanceType, itemId, snippet}` — traceable to its source
  peer and instance. The `partial: true` flag and `unreachablePeers` list
  (`solomon.md` §4.2.2) are the freshness/availability governance axis. **Gap:**
  the result shape has no `sourceCrawlId`/`documentId`-style deep provenance
  (only `itemId`), so a cross-instance result cannot be traced to its underlying
  crawl/document without a follow-up call. **Recommendation:** NICE-TO-HAVE —
  extend the result shape with a deep-provenance reference (e.g. a
  `sourceRef` that resolves to the owning instance's item) once Phase-1 search
  semantics finalize (`solomon.md` §7.4). PARKED until then.

- **Mystery** (`docs/specs/mystery.md` — **absent**; `docs/pending.md` #12):
  Mystery is a message board that crosslinks information references by post
  topic (`docs/architecture-overview.md` §4.3). Its topic→reference resolution
  (`docs/pending.md` #12) is inherently a **provenance** problem: a post topic
  links to an information reference (Astrographer documentation, Astral pages),
  and that link must be traceable to the source. **Recommendation:** when the
  Mystery contract is written, model its crosslinks as **provenance-carrying
  edges** (topic → reference → source document/fact), reusing Astrographer's
  reference graph. PARKED until the spec exists.

- **Astral** (`docs/specs/astral.md`): Astral is the wiki/publishing host
  (`astral.md` §2). Its hosted units already carry **publish provenance**
  (`astral.md` §4.1): `source tool`, `push timestamp`, `last-modified`, and
  `content hash` — so a published page can be traced to its pushing tool and
  content revision. The push ingestion (`astral.md` §4.2) enforces idempotency
  via `contentHash`, giving an integrity/audit guarantee. **Recommendation:** no
  new Astral contract change required by this research; Astral's provenance
  (source tool + content hash) is already in spec. The docs-as-product
  reachability surface (GAP-4/GAP-5, `docs/defects.md`) is the complementary
  agent-facing layer.

---

## §5 Recommendation for the suite

Grounded in the four design constraints (D1 open-source AGPL-3.0, D2
local-first, D3 interconnection, D4 MCP-GUI parity).

- **MUST — Keep the existing provenance surfaces non-negotiable.** The suite
  already implements the talk's "sourcing and tracing" across its contracts:
  Astrographer's consistency report + exact-reference resolution
  (`astrographer.md` §4.2.3), Zodiac's `sourceCrawlId` + `stale`
  (`zodiac.md` §4.3.2), Familiar's facts `provenance` + fail-closed validation
  (`familiar.md` §4.1.3), Solomon's `peerId`/`instanceType` (`solomon.md`
  §4.2.2), and Astral's `source tool` + `contentHash` (`astral.md` §4.1). These
  are **already in the spec** — no change. They are the governance foundation
  and are D2-compliant (all local) and D4-compliant (exposed via GUI + MCP).

- **MUST — Add answer-level citation + trace path to RAG results.** Extend
  Astrographer's `ragQuery`/`ragStream` result (`astrographer.md` §4.3.2) and
  Zodiac's query result (`zodiac.md` §4.3.2) with a `citations` field (the
  source set the answer is grounded on) and a `trace` field (the graph path
  walked). This is the talk's governance point made answer-level, and it is
  cheap because the provenance substrate already exists. **Proposed spec change**
  to `astrographer.md` §4.3 and `zodiac.md` §4.3. D4-compliant (exposable via
  both GUI and MCP).

- **SHOULD — Add a RAG query audit log.** Record who asked what, when, and what
  was returned for `ragQuery`/`ragStream` (`astrographer.md` §4.3.2). This is
  the auditability axis of governance (per *RAG Governance* and *Auditable
  Enterprise RAG Pipeline*, §1). **Proposed spec change** to `astrographer.md`
  §4.3 (a `getQueryAuditLog` surface + MCP tool). D4-compliant.

- **SHOULD — Measure faithfulness/answer-relevance.** Adopt the RAGAS-style
  metrics (faithfulness, answer relevance; §1) as an evaluation gate for the
  RAG surfaces. This makes explainability *verifiable* rather than asserted.
  **Proposed spec change** (evaluation contract, not a runtime feature).

- **NICE-TO-HAVE — Deep provenance in Solomon results.** Extend the result shape
  (`solomon.md` §4.2.2) with a `sourceRef` that resolves to the owning
  instance's underlying item. PARKED until Phase-1 search semantics finalize
  (`solomon.md` §7.4).

- **PARKED — Full compliance/audit framework.** A suite-wide audit/retention
  framework (query retention policies, access-control integration, regulated-
  environment compliance) is valuable but gated on the tools being implemented
  and the MCP surface resolving (`docs/pending.md` #2). Revisit after the
  MUST/SHOULD provenance + audit-log work ships.

**Overall recommendation tier: MUST.** Explainability/governance is not a
foreign addition for the suite — the provenance substrate is already in the
specs (consistency report, sourceCrawlId, facts provenance, contentHash). The
concrete MUST/SHOULD work is making provenance **answer-level** (citation +
trace path) and **auditable** (query audit log), both cheap because the
substrate exists. This directly answers the source talk's governance point and
is consistent with D1 (open-source, auditable), D2 (all local), D3 (provenance
flows across the interconnected tools), and D4 (exposable via GUI + MCP).

---

## §6 Source URL list

1. https://arxiv.org/pdf/2409.11242 — *Measuring and Enhancing Trustworthiness of LLMs in RAG through Grounded Attributions and Learning to Refuse*
2. https://aclanthology.org/2024.emnlp-main.347.pdf — *Model Internals-based Answer Attribution for Trustworthy Retrieval-Augmented Generation*
3. https://aiarch.dev/patterns/grounded-rag — *RAG with Grounded Citations: Making Retrieval Answers Provable*
4. https://gautierdorval.com/en/definitions/retrieval-provenance/ — *Retrieval provenance* (definition)
5. https://www.ijfmr.com/papers/2026/3/80826.pdf — *Provenance for Auditable RAG*
6. https://doi.org/10.1109/acdsa67686.2026.11468256 — *Evaluating GraphRAG for Traceable and Interpretable Question Answering*
7. https://doi.org/10.48550/arxiv.2604.24623 — *XGRAG: A Graph-Native Framework for Explaining KG-based Retrieval-Augmented Generation*
8. https://docs.trustgraph.ai/overview/explainability.html — TrustGraph, *Explainability*
9. https://github.com/trustgraph-ai/trustgraph/blob/4e3bd85a/docs/tech-specs/query-time-explainability.md — TrustGraph, *query-time explainability*
10. https://thomasthelliez.com/blog/rag-governance-source-authority-access-control-auditability/ — *RAG Governance: Source Authority, Access Control, Freshness, and Auditability*
11. https://gautierdorval.com/en/frameworks/rag-governance-retrieval-and-inference-control/ — *RAG governance: retrieval and inference control*
12. https://scadea.com/enterprise-rag-and-permission-aware-retrieval/ — *Enterprise RAG Architecture: The Reference Model*
13. https://docushell.com/blog/how-do-you-build-an-auditable-rag-pipeline-for-enterprise-documents — *How to Build an Auditable Enterprise RAG Pipeline*
14. https://arxiv.org/html/2607.02116v2 — *ContextNest: Verifiable Context Governance for Autonomous AI Agents*
15. https://aclanthology.org/2024.eacl-demo.16.pdf — *RAGAS: Automated Evaluation of Retrieval Augmented Generation*
16. https://link.springer.com/article/10.1007/s10994-026-07121-y — *Disentangling Faithfulness Hallucinations in Retrieval-Augmented Generation*

**Additional practical tools referenced in §1 (not counted in the 16):**
https://github.com/aryanSharmaGithub/auditRag · https://github.com/dakshtrehan/ragcompliance · https://github.com/roomariz/trace-rag · https://pypi.org/project/trailrag/

**Suite context cited throughout:** `docs/specs/astrographer.md` (§4.1.1, §4.2.1–§4.2.3, §4.3.2, §4.4.1, §4.5.1, §4.6.2), `docs/specs/zodiac.md` (§4.1, §4.2, §4.3.2, §4.3.3, §6.11), `docs/specs/incanter.md` (§4.8, §4.10), `docs/specs/familiar.md` (§4.1.3, §4.1.3.1, §4.1.3.2, §4.3.1, §4.3.3), `docs/specs/solomon.md` (§4.2.2, §7.4), `docs/specs/astral.md` (§4.1, §4.2), `docs/architecture-overview.md` (§4.3, §5, §6), `docs/decisions.md` (D1–D4), `docs/pending.md` (#2, #12), `docs/defects.md` (GAP-4/GAP-5), `docs/research/dag-rag-research-notes.md` (§4).
