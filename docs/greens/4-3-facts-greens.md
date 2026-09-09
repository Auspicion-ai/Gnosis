# §4.3 Fact/citation tracking — blind greens

Derived **from the documentation only** (`docs/specs/gnosis.md` §4.3.1–§4.3.4,
§4.5.4, §6) by the Blind-Test Writer. Validation: ran the built crate's
integration suites (`CARGO_HOME=/tmp/gnosis-cargo-home cargo test`, 8 suites,
**233 tests green**) and mapped documented behaviors to exercising tests in
`tests/facts_integration.rs` (36), `tests/agent_memory_integration.rs` (6),
`tests/rag_query_integration.rs` (34). **No behavior was verified by reading
`src/`.**

Legend: `GREEN` = demonstrably exhibited; `NOT VERIFIED` = documented but not
demonstrated by a test.

## 4.3.1 Fact nodes / 4.3.2 Citations

| Documented behavior | Expected observable result | Validating test(s) | Status |
| --- | --- | --- | --- |
| `factKey` stable and unique **within the Wiki** (same key in another wiki is a distinct fact) | same key stored in two wikis without conflict; distinct `documentId` | `create_fact_stores_fact_with_citations_unique_per_wiki` | GREEN |
| duplicate `factKey` within a wiki → `ConflictError` | `ConflictError` | `create_fact_duplicate_fact_key_in_wiki_is_conflict_error` | GREEN |
| fact holds `value`, `documentId`, `nodeId` | round-trip via `getFact` | `get_fact_returns_stored_fact` | GREEN |
| `updatedAt` present | non-empty (ISO-8601), refreshes on update | `get_fact_returns_stored_fact`, `update_fact_changes_value_and_citations` | GREEN (**format** not asserted) |
| minimum-citation invariant: a fact must have ≥1 citation | empty citations → `ValidationError` "at least one citation" | `create_fact_empty_citations_is_validation_error`, `update_fact_empty_citations_is_validation_error` | GREEN |
| citations deduplicated by first appearance | duplicate in input list removed | `create_fact_dedups_citations_by_first_appearance` | GREEN |
| `updateFact` changes value + citations and refreshes `updatedAt` | new value/citations, `updatedAt` ≥ prior | `update_fact_changes_value_and_citations` | GREEN |
| `updateFact` unknown fact → `DocumentNotFound` | `DocumentNotFound` | `update_fact_unknown_fact_is_document_not_found` | GREEN |
| `updateFact` empty/whitespace value → `ValidationError` | `ValidationError` | `update_fact_empty_value_is_validation_error`, `…_whitespace_value_…` | GREEN |
| `createFact` empty/whitespace value, whitespace key → `ValidationError` | `ValidationError` | `create_fact_whitespace_value…`, `…_whitespace_fact_key…` | GREEN |
| unknown wiki → `WikiNotFound` on the fact surface (`createFact`/`getFact`/`updateFact`/`proposeCandidateFact`) | `WikiNotFound` | `create_fact_unknown_wiki…`, `get_fact_unknown_wiki…`, `update_fact_unknown_wiki…`, `propose_candidate_fact_unknown_wiki…` | GREEN |
| `getFact` exact-key retrieval (not similarity) | exact key resolves; "similar" key does not | `get_fact_returns_exact_fact_key_matches_only` (agent-memory suite) | GREEN |
| `listFacts` paginated `{items,total,page,pageSize}` | correct shape | `list_facts_paginates` | GREEN |
| `listFacts` `state` filter and pagination fail-states | per filter; `page<1`/`pageSize<1`/`pageSize>100` → `ValidationError`; unknown wiki → `WikiNotFound` | `list_facts_filters_by_document_state`, `list_facts_page_less_than_one…`, `…_page_size_less_than_one…`, `…_over_one_hundred…`, `list_facts_unknown_wiki…` | GREEN |

## 4.3.2a Candidate-fact pipeline + deterministic validation

| Documented behavior | Expected observable result | Validating test(s) | Status |
| --- | --- | --- | --- |
| valid candidate passes the gate **and commits** | `accepted: true` + persisted fact | `valid_candidate_passes_gate_and_commits` | GREEN |
| schema conformance: empty `value` rejected fail-closed | structured `{code,field,message}`, no commit | `candidate_empty_value_rejected_with_structured_reason_and_not_committed` | GREEN |
| schema conformance: missing/empty `factKey` rejected | structured, no commit | `candidate_missing_fact_key_rejected_with_structured_reason_and_not_committed` | GREEN |
| schema conformance: empty `citations` rejected (minimum-citation) | structured, no commit | `candidate_empty_citations_rejected_with_structured_reason_and_not_committed` | GREEN |
| schema: whitespace-only `factKey`/`value` rejected | structured, no commit | `candidate_whitespace_fact_key…`, `candidate_whitespace_value…` | GREEN |
| grounding/provenance: dangling citation → `ValidationError` "citation does not resolve" | `ValidationError`, no commit | `candidate_dangling_citation_is_validation_error_and_not_committed` | GREEN |
| grounding: citation to a **deleted document** never commits | `ValidationError`, no commit | `candidate_citing_a_deleted_document_is_not_committed` | GREEN |
| dedup/conflict: duplicate `factKey` → `ConflictError`, pre-existing fact untouched | `ConflictError`, manual value persists | `candidate_duplicate_fact_key_is_conflict_error_and_not_committed` | GREEN |
| manual `createFact` override passes the gate and is authoritative | conflicting candidate rejected; manual value persists | `manual_fact_is_authoritative_and_not_overwritten_by_conflicting_candidate` | GREEN |
| concurrent `createFact` same key — exactly one wins | 1 ok / N−1 `ConflictError`, single committed fact | `concurrent_create_fact_unique_key_exactly_one_wins` | GREEN |
| concurrent candidate same key — exactly one commits atomically | 1 accepted | `concurrent_candidate_same_key_commits_atomically_exactly_one` | GREEN |
| **cross-field consistency** (value consistent with cited nodes' content) is a validation check | rejected if inconsistent | — | NOT VERIFIED (no test drives a value-vs-content mismatch; the facts suite explicitly defers this to §4.5 where the embedding leg exists) |

## 4.3.3 / 4.3.4 Provenance + audit log

| Documented behavior | Expected observable result | Validating test(s) | Status |
| --- | --- | --- | --- |
| `getQueryAuditLog()` returns `[{query, filters, mode, resultCount, timestamp, requester}]` | typed shape + populated `QueryAuditEntry` | `get_query_audit_log_returns_audit_entry_shape` | GREEN |
| every `ragQuery` appends an audit entry | N queries → N entries with query/mode/filters/requester/timestamp | `rag_query_appends_query_audit_entries` (rag-query suite) | GREEN |
| provenance `trace` present on every result, mode-shaped | `RagTrace` non-optional; flat/vector/hybrid/graph shapes | `rag_result_trace_is_always_present_and_mode_shaped` (+ flat/vector/hybrid graph-mode tests) | GREEN (type-shape; runtime `TraceUnavailable` is a boundary — see §4.6 file) |

## NOT VERIFIED — §4.3

1. **Cross-field consistency gate** (§4.3.2a.1). No test proposes a fact whose
   value contradicts its cited nodes' content and asserts rejection. The facts
   suite states this check is deferred to §4.5 (it requires the embedding leg),
   and no §4.5 test exercises it either.
   **→ REAL GAP (deferred):** already recorded as an ENGINE-INTERNAL DEFERRED row
   ("Cross-field consistency gate") in `docs/pending.md`. Needs the §4.5
   embedding/entity-resolution leg; revisit at that pass.
2. **`createdAt`/`updatedAt` are ISO-8601 UTC format** (§4.3.1). Tests assert
   non-empty only, not the format.
   **→ REAL GAP (quick correctness pin):** fact `updatedAt` is stamped via the
   same `iso_now()` (emits `YYYY-MM-DDThh:mm:ss.nnnnnnnnnZ`, ISO-8601 UTC) —
   only a format assertion is missing. Folded into the ISO-8601 pending row in
   `docs/pending.md`.
