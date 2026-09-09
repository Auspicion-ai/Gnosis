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
| **Fact-store sharding** | `fact_store` is a single global `RwLock` (a de-facto store-wide write serialization point for fact-heavy loads). **DEFERRED (re-scoped 2026-09-09 after §4.5):** the per-wiki nested map already gives lookup locality and the commit paths are serialized by design (grounding under the integrity lock then commit under `fact_store.write()`), and no §4.5 load path justifies sharding yet. **Refreshed revisit condition:** only a **measured** signal — a throughput/contention benchmark or a fact-heavy multi-writer workload proving the single outer `fact_store` write lock is a bottleneck. The bounded unit, if revisited, is: shard `fact_store` by wiki-hash across N `RwLock`s (mirroring SHARDED-RWLOCK-STORE), preserving the exact commit critical section. |
| **Fact `node_id` coherence** | §4.3.1 says `documentId`/`nodeId` is the fact's location in the store, but the stored `node_id` is a fabricated `fact-{fact_key}` handle, not a live graph `fact` node. **CLOSED as a spec-vs-impl wording tension / spec-reconcile (2026-09-09):** §4.5.4 makes `factKey` the canonical retrieval identity ("similarity ≠ relevance", exact-match), so `factKey` — not `(documentId, nodeId)` — is the fact's true location identity. The engine will **not** auto-materialize a graph `fact` node on commit (that would change `create_fact`/`update_document` authoring semantics and contradict the frozen `Node`/`Fact` model + the manual-override convention §4.3.2a.3/§4.2.8; pinned tests lock in the `fact-<key>` handle). Contract §4.3.1 should reconcile to: a fact's canonical identity is `factKey`; `documentId`/`nodeId` are a derived conventional handle (`nodeId` = `fact-<key>`, corresponding to a live graph `fact` node only when the caller authored one). Reconcile at the spec seam. See `docs/defects.md` + `docs/HANDOFF.md`. |
| **Engine-side query audit-log recording sink** | **RESOLVED (2026-09-09) — the §4.5 query path delivers it.** `Store` carries `query_audit_log: RwLock<Vec<QueryAuditEntry>>`; `rag_query` appends a real entry on every committed query (filters/mode/result_count/timestamp/requester) and `rag_stream` routes through `rag_query`, so §4.3.4 `getQueryAuditLog` returns real entries. Pinned by `tests/facts_integration.rs::get_query_audit_log_returns_audit_entry_shape` and `tests/rag_query_integration.rs::rag_query_appends_query_audit_entries`. |
| **Cross-field consistency gate** | §4.3.2a.1 lists cross-field consistency as a gate check. **CLOSED as a documented non-goal / spec-reconcile (2026-09-09):** the §4.3 gate enforces schema/grounding/dedup only (`rejected_outcome` is invoked only with `code:"schema"`; no `cross-field`/`CrossField` branch exists in `src/`). Enforcing "value consistent with cited nodes' content" needs an LLM/embedding *judgment* on the commit path, which breaks the spec's deterministic fail-closed gate framing (§4.3.2a) and injects a provider dependency into the manual-override flow (§4.3.2a.3). **Recorded as a deliberate non-goal** (`code:"schema"` only). Reconcile spec §4.3.2a.1 to mark cross-field as an aspirational/offline semantic check, **not** enforced by the runtime §4.3 gate; optional re-scope to a SHOULD "future offline fact-validation pass over the §4.5 embedding leg." |
| **Full sub-task DAG (`subTaskDag`)** | Spec'd **opto-in / SHOULD-HAVE** (§4.5.4, same class as adaptive RAG F3, which is PARKED). Currently parsed + validated only; **decomposes nothing** — a single-node decomposition (the query itself) is the honest stand-in, no fake decomposition is fabricated (decision SUB-TASK-DAG-VALIDATED-ONLY). `SubTaskDagFailed` is a RESERVED variant (throwable once the real decomposition lands); no test requires it to throw. **PARKED (engine-internal deferred):** revisit with complex multi-hop queries; the DAG decomposition + a runner that fans sub-queries out (and merges their results) lands here. |
| **Non-degradable compressor leg (`CompressionFailed`)** | The local compressor is **total** (Filter/Extract/Graph only transform; it degrades gracefully to uncompressed — spec-correct). `CompressionFailed` is a **RESERVED** variant for a future compressor leg that cannot degrade on failure. **PARKED:** only implement a throw-path when a real non-degradable compressor lands; do not fabricate a failure mode. |
| **`resolve_entities` residual alias / alias-cycle behavior** | `resolve_entities` records alias→canonical for the requested set but **never removes a PRIOR alias entry** (`src/store/mod.rs` only `insert`s the durable map), so an overlapping re-resolution with a different canonical can leave a residual alias (e.g. `e1→e2` while `e2→e1`); same-call idempotence holds, full convergence does not. **ENGINE-INTERNAL DEFERRED:** **revisit condition** — a decision on whether each `resolve_entities` call should be treated as an **authoritative overwrite** (clearing/stabilizing to an **acyclic** alias graph) vs. **additive** (each call is an explicit manual override that records its requested aliases and lets prior ones persist). See `docs/defects.md` §4.2 `resolve_entities` row + the §4.2 PBT register `P-TP-2` note. |

## DOC-REVIEW GAPS (surfaced by the 2026-09-09 core doc-review; not spec gaps)

Real gaps found while reconciling the greens/specs to the crate. Each is a
documented-but-untested behavior that **is** implementable/checkable from the
current crate. Preferred recording over forcing a test in this pass because the
session is docs-only (read-only on `src/`/`tests/`).

| Item | Disposition / revisit condition |
| --- | --- |
| **Engine-assigned `documentId` (and `wikiId`) are not UUID v4** | Spec §4.1.1/§4.1.2 say `documentId`/`wikiId` are "stable, globally-unique … (UUID v4)". The crate assigns `doc-{N}` / `wiki-{N}` (a monotonic counter prefix), **not** an RFC-4122 UUID v4. This is a **REAL spec-impl tension** (a MUST-level contract claim not honored by the id scheme). **DISPOSITION:** deferred — either adopt UUID v4 for engine-assigned ids at the §5.1 engine-seam hardening pass, or formally reconcile the upstream contract to permit stable monotonic ids. Revisit when the F2 IPC seam lands (the id format is an on-the-wire contract). |
| **`createdAt`/`updatedAt` ISO-8601 UTC format not pinned by any test** | Spec §4.1.1/§4.3.1 require ISO-8601 UTC timestamps. The crate's `iso_now()` does emit `YYYY-MM-DDThh:mm:ss.nnnnnnnnnZ` (ISO-8601 UTC), but no test asserts the format (only non-empty). **DISPOSITION: quick correctness pin** — a format assertion in the store + facts suites would close it; deferred because the session is docs-only. Revisit at the next test-authoring pass. |
| **`author` metadata not round-trip-tested** | Spec §4.1.1 exposes `author` (string). The store sets `author` on `createDocument` and preserves it across `updateDocument`, but no test asserts it is stored/returned. **DISPOSITION: quick correctness pin** — an `author` round-trip + preserved-on-update assertion closes it; deferred because the session is docs-only. Revisit at the next test-authoring pass. |
| **`MultiQueryExpansionFailed` (FS-19) fail-state is reachable but untested** | `expand_query_variants` genuinely returns `MultiQueryExpansionFailed` when a multi-query fan-out (`multiQuery:{enabled,n≥2}`) has **no distinct indexable term to add** (e.g. an empty/near-empty store: `term_popularity` yields nothing). Unlike the other engine-internal variants it is **drivable from a typed test** with no seam. **DISPOSITION: quick correctness pin** — seed a store with few/no indexable terms, run a multi-query `n≥2`, assert `MultiQueryExpansionFailed`; deferred because the session is docs-only. Revisit at the next test-authoring pass. |

## SPECULATIVE

_(No Gnosis-specific speculative items yet — the parked layers are recorded in
the suite `docs/pending.md` (D3/D5/D6/D8/D9) and the spec §7 gaps.)_
