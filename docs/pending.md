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

## SPECULATIVE

_(No Gnosis-specific speculative items yet — the parked layers are recorded in
the suite `docs/pending.md` (D3/D5/D6/D8/D9) and the spec §7 gaps.)_
