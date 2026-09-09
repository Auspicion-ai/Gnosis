# §4.6 Query modes / surfaces + §6 fail-state catalogue — blind greens

Derived **from the documentation only** (`docs/specs/gnosis.md` §4.6.1–§4.6.2,
§4.3.3/§4.3.4, §6) by the Blind-Test Writer. Validation: ran the built crate's
integration suites (`CARGO_HOME=/tmp/gnosis-cargo-home cargo test`, 8 suites,
**233 tests green**) and mapped documented behaviors to exercising tests in
`tests/rag_query_integration.rs` (34), `tests/facts_integration.rs` (36).
**No behavior was verified by reading `src/`.**

Legend: `GREEN` = demonstrably exhibited; `NOT VERIFIED` = documented but not
demonstrated by a test; *(transport/boundary)* = not drivable from a typed test
per the §4.6/§4.3.3 contract decisions in the suites.

## 4.6.1 `ragQuery` validations (FS-3)

| Documented behavior | Expected observable result | Validating test(s) | Status |
| --- | --- | --- | --- |
| empty `query` → `ValidationError` | `ValidationError` | `rag_query_empty_query_is_validation_error` | GREEN |
| `topK < 1` → `ValidationError` | `ValidationError` | `rag_query_top_k_below_one_is_validation_error` | GREEN |
| `topK > 50` → `ValidationError` | `ValidationError` | `rag_query_top_k_above_fifty_is_validation_error` | GREEN |
| `maxHops` out of 1–5 → `ValidationError` | `ValidationError` for 0 and 6 | `rag_query_max_hops_out_of_range_is_validation_error` | GREEN |
| malformed `filters` (out-of-set `edgeType`) → `ValidationError` | `ValidationError` | `rag_query_malformed_filters_edge_type_is_validation_error` | GREEN |
| `multiQuery.n` out of range (`0`) → `ValidationError` | `ValidationError` | `rag_query_multi_query_n_zero_is_validation_error` | GREEN |
| `binaryCandidatePool` not positive (`0`) → `ValidationError` | `ValidationError` | `rag_query_binary_candidate_pool_zero_is_validation_error` | GREEN |
| invalid `mode`, invalid `compression`, non-boolean `hyde`, non-boolean `binaryFirstPass`, non-boolean-object `subTaskDag` → `ValidationError` | `ValidationError` | — | NOT VERIFIED (Rust-typed enums/bool/struct make these unrepresentable from a typed test; validated at the §5.1 transport boundary per the suite header) |

## 4.6.1 `ragQuery` / `ragStream` fail-states (FS)

| Doc. fail-state | Documented trigger | Validating test(s) | Status |
| --- | --- | --- | --- |
| FS-8 `EngineUnavailable` | engine not `READY` (UNAVAILABLE/STARTING) | `rag_query_engine_not_ready_is_engine_unavailable`, `rag_stream_engine_not_ready_is_engine_unavailable`, `get_engine_status_unavailable_blocked_for_rag_query` | GREEN |
| FS-9 `EngineError` | engine returns a malformed result | — | NOT VERIFIED *(boundary)* |
| FS-10 `TraceUnavailable` | results without a trace | `rag_result_trace_is_always_present_and_mode_shaped` (type-level only) | NOT VERIFIED (only the non-optional-type property is asserted; no runtime test drives a missing trace) |
| FS-11 `HopLimitExceeded` | graph traversal exceeds `maxHops` | `rag_query_graph_mode_hop_limit_exceeded`; `resolve_references_over_hop_cap_is_hop_limit_exceeded` | GREEN |
| FS-12 `CycleDetected` | `reference`→`fact` cycle | `rag_query_graph_mode_cycle_detected`; `resolve_references_cycle_is_cycle_detected` | GREEN |
| FS-13 `EmbeddingUnavailable` | embedding provider unreachable (vector/hybrid/hyde) | `rag_query_vector_mode_unavailable_provider_is_embedding_unavailable`, `vector_search_embedding_provider_unreachable_is_unavailable`, `wiremock_http_embedding_provider_404_is_embedding_unavailable` | GREEN |
| FS-14 `VectorIndexUnavailable` | vector index not built | `rag_query_vector_mode_index_not_built_is_vector_index_unavailable`, `vector_search_vector_index_not_built_is_unavailable` | GREEN |
| FS-15 `LexicalIndexUnavailable` | BM25 index not built | `bm25_search_lexical_index_not_built_is_unavailable` | GREEN (index level; hybrid-`ragQuery` path NOT VERIFIED — see §4.5) |
| FS-16 `RerankerUnavailable` | reranker model unavailable | — | NOT VERIFIED *(boundary; no reranker seam)* |
| FS-17 `CompressionFailed` | compressor fails and can't degrade | — | NOT VERIFIED *(boundary)* |
| FS-18 `HyDEGenerationFailed` | hyde generation fails | — | NOT VERIFIED *(boundary)* |
| FS-19 `MultiQueryExpansionFailed` | query expansion fails | — | NOT VERIFIED *(boundary)* |
| FS-26 `SubTaskDagFailed` | sub-task DAG decomposition fails | — | NOT VERIFIED *(boundary; `subTaskDag` is a typed param, no decomposition-driven test)* |
| FS-20/21/22 `PushRejected`/`AstralUnavailable`/`SchemaMismatch` | §5.2 Astral push | — | N/A (push transport is shell-side; not in the crate) |
| FS-23 `ZodiacUnavailable` | a Zodiac query degrades to local-only | — | NOT VERIFIED (no Zodiac seam in the crate; only `source: 'local'` appears in traces, never degradation of a `'zodiac'` leg) |
| FS-24 `EnrichmentBudgetExceeded` | parked F4 | — | N/A (parked surface, §7.5) |
| FS-25 `CommunityNotFound` | unknown community | `get_community_unknown_is_community_not_found`, `update_community_summary_unknown_…`, `community_state_unknown_is_community_not_found`, `rederive_community_unknown_…` | GREEN |

## 4.6.1 `ragStream`

| Documented behavior | Expected observable result | Validating test(s) | Status |
| --- | --- | --- | --- |
| stream emits a `result` chunk (full `RagResult`) then `done`, then closes | first `Result`, last `Done` | `rag_stream_emits_result_then_done` | GREEN |
| mid-stream failure emitted as an `error` chunk then closes | `Error(CycleDetected)` present, last chunk `Done` | `rag_stream_emits_error_chunk_then_closes` | GREEN |

## 4.6.1 `getEngineStatus`

| Documented behavior | Expected observable result | Validating test(s) | Status |
| --- | --- | --- | --- |
| `READY` state + per-subsystem status when all up | `Ready`, store/graph/lexical true | `get_engine_status_ready_when_all_subsystems_up` | GREEN |
| `DEGRADED` when a non-core subsystem (embedding) is down; core still works | `Degraded`, `embedding:false`, core true | `get_engine_status_degraded_when_embedding_provider_down` | GREEN |
| `UNAVAILABLE` when not running/reachable; `ragQuery` on it → `EngineUnavailable` | `Unavailable` + `EngineUnavailable` | `get_engine_status_unavailable_blocked_for_rag_query` | GREEN |
| `STARTING` engine state is one of the documented health states | `STARTING` reported during boot | — | NOT VERIFIED (no test sets/observes the `STARTING` state) |

## 4.6.2 No MCP / no GUI surface

| Documented behavior | Note | Status |
| --- | --- | --- |
| Gnosis is a pure backend; MCP + GUI surfaces live at the Astrographer shell | architectural boundary; the crate exposes a library API (`lib.rs`) consumed by the shell | N/A (not an observable runtime behavior in this crate) |

## NOT VERIFIED / boundary — §4.6

1. `STARTING` engine state (§4.6.1) — no test observes it.
   **→ PARKED/RESERVED:** `EngineState::Starting` is a construction-time health
   state (booting); `set_engine_state` is a boot/test hook and no test drives it.
   Not reachable from the query surface. No pending row needed.
2. `EngineError` (FS-9), `TraceUnavailable` (FS-10 runtime), `RerankerUnavailable`
   (FS-16), `CompressionFailed` (FS-17), `HyDEGenerationFailed` (FS-18),
   `MultiQueryExpansionFailed` (FS-19), `SubTaskDagFailed` (FS-26) — engine-
   internal / no injectable seam, not drivable from a typed test.
   **→ SPLIT:** `EngineError`, `TraceUnavailable`, `RerankerUnavailable` are
   **PARKED/RESERVED** (no injectable seam on this crate, decision
   RESERVED-ERRVARIANTS-DISCIPLINE). `CompressionFailed` (compressor total) and
   `HyDEGenerationFailed`/`SubTaskDagFailed` (steps total/unimplemented) are
   **PARKED/RESERVED**. **`MultiQueryExpansionFailed` is a REAL GAP** (see §4.5
   NOT VERIFIED #4) — genuinely reachable from a typed test (empty/near-empty
   store + `multiQuery:{enabled,n≥2}`) and recorded in `docs/pending.md`.
3. invalid `mode` / invalid `compression` / non-boolean `hyde` /
   non-boolean `binaryFirstPass` / non-boolean-object `subTaskDag` FS-3 cases —
   unrepresentable from the typed `RagQueryOptions` (validated at the §5.1
   transport boundary).
   **→ PARKED/RESERVED:** Rust-typed enums/bools/structs make these
   unrepresentable from a typed test; they are §5.1-transport validation, not
   engine behavior. No pending row needed.
4. `ZodiacUnavailable` (FS-23) — no Zodiac leg in the crate; the local-only
   degradation of a would-be–Zodiac query is not demonstrated.
   **→ PARKED/RESERVED:** no Zodiac seam exists in this crate (D3 interconnection
   is wired at the Astrographer shell); only `source: 'local'` appears in traces.
   No pending row needed.
