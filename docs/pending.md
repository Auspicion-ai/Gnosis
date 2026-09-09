# Gnosis — Pending / Parked / Speculative

Maintained by the document-archival loop. Three kinds of rows: (a) UPSTREAM —
constraints owned by the Auspicion Suite top-level architecture repo that Gnosis
must respect; (b) DEFERRED — lower-value gaps parked until a use case surfaces;
(c) SPECULATIVE — future features with their revisit conditions.

## UPSTREAM (imported constraints)

| Constraint | Source | What Gnosis does |
| --- | --- | --- |
| Honor the canonical contract `docs/specs/gnosis.md` | `../Auspicion Suite/docs/specs/gnosis.md` | Every behavior is derived from the contract; a change goes through the suite proposal-review gate. |
| Gnosis is a pure backend — no MCP/GUI surface | spec §4.6.2 | Gnosis exposes its API to the shell; D4 parity applies at the shell. |
| Incanter is archived as a prototype | decision GNOSIS-ENGINE | Gnosis supersedes Incanter; do not reuse Incanter as the production engine. |

## DEFERRED / PARKED (from the spec §7 gaps + the parked retrieval surfaces)

| Item | Disposition |
| --- | --- |
| **F2 — engine transport/API reconciliation** | The exact IPC/process transport between the shell and Gnosis is not yet pinned to a concrete wire format (spec §7.2). Design at the shell seam unit. |
| **F3 — Adaptive RAG routing to Zodiac** | **PARKED** (spec §7.4): a query-complexity router; land after the MUST/SHOULD retrieval foundation. |
| **F4 — Community summaries** | **PARKED** (spec §7.5): the *automatic* community detection + summary generation; the **manual** `declareCommunity` override is a pinned requirement (§4.2.8). |
| **F5 — LLM-generated dynamic graph query** | **PARKED** (spec §7.6): gated on a graph-query substrate the suite lacks. |
| **F6 — RAG evaluation harness** | **SHOULD HAVE dev/QA gate** (spec §7.8): a DeepEval-style offline faithfulness/relevancy/precision-recall gate over the audit-log material; not a runtime contract element. |

## ENGINE-INTERNAL DEFERRED (surfaced by the §4.3 adversarial gate; not spec gaps)

| Item | Disposition |
| --- | --- |
| **Fact-store sharding** | `fact_store` is a single global `RwLock` (a de-facto store-wide write serialization point for fact-heavy loads). **DEFERRED:** shard by wiki or fold facts into the document shards; revisit at the §4.5 volume-tuning pass. |
| **Fact `node_id` coherence** | §4.3.1 says `documentId`/`nodeId` is the fact's location in the store, but the stored `node_id` is a fabricated `fact-{fact_key}` handle, not a live graph `fact` node (a caller `edges_from(.., "fact-lic")` gets `ValidationError`). **DEFERRED:** materialize the graph `fact` node on commit (or drop the location claim) — revisit with the §4.5 retrieval work. |
| **Engine-side query audit-log recording sink** | `getQueryAuditLog` accessor exists (`Ok(vec![])` until recording lands), but no recording feed yet. **DEFERRED to §4.5:** wire the `ragQuery`/`ragStream` recording sink + audit store so §4.3.4 returns real entries. |
| **Cross-field consistency gate** | §4.3.2a.1 lists cross-field consistency as a gate check, but a semantic content check needs the §4.5 embedding/entity-resolution leg. **DEFERRED to §4.5:** the §4.3 gate enforces schema/grounding/dedup only; cross-field is explicitly not a runtime §4.3 branch. |

## SPECULATIVE

_(No Gnosis-specific speculative items yet — the parked layers are recorded in
the suite `docs/pending.md` (D3/D5/D6/D8/D9) and the spec §7 gaps.)_
