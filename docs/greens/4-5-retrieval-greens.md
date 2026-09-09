# §4.5 RAG/agent-memory retrieval — blind greens

Derived **from the documentation only** (`docs/specs/gnosis.md` §4.5.1–§4.5.5,
§4.5.4, §6) by the Blind-Test Writer. Validation: ran the built crate's
integration suites (`CARGO_HOME=/tmp/gnosis-cargo-home cargo test`, 8 suites,
**233 tests green**) and mapped documented behaviors to exercising tests in
`tests/retrieval_stack_integration.rs` (18), `tests/rag_query_integration.rs`
(34), `tests/agent_memory_integration.rs` (6). **No behavior was verified by
reading `src/`.**

Legend: `GREEN` = demonstrably exhibited; `NOT VERIFIED` = documented but not
demonstrated by a test; *(PARKED / SHOULD HAVE / shell-side)* = out of the
current crate behavior surface per the spec.

## 4.5.1 Query modes

| Documented behavior | Expected observable result | Validating test(s) | Status |
| --- | --- | --- | --- |
| `flat` — top-k by combined score, base mode | returns top-k with `engine==gnosis` + flat trace | `rag_query_flat_mode_returns_top_k_with_flat_trace` | GREEN |
| `graph` — multi-hop traversal (§4.5.2) | see below | below | GREEN |
| `vector` — embedding similarity only (dense leg, not a lexical fallback) | a seeded vector-leg hit with non-lexical content returned | `rag_query_vector_mode_returns_vector_leg_hits` | GREEN |
| `hybrid` — RRF fusion of graph + vector + lexical legs | all three distinct legs contribute | `rag_query_hybrid_mode_merges_three_distinct_legs` | GREEN |
| `hybrid` trace lists `legs: ['graph','vector','lexical']` | trace field correct | same test | GREEN |

## 4.5.2 Multi-hop traversal

| Documented behavior | Expected observable result | Validating test(s) | Status |
| --- | --- | --- | --- |
| `maxHops` 1–5, default 3 | valid range enforced | `rag_query_max_hops_out_of_range_is_validation_error` (FS-3) | GREEN (range checks) |
| graph mode resolves through the `reference`→`fact` graph and returns the ordered path as the `graph` trace | citations include the reached fact; ordered `{from,to,edge,state}` steps | `rag_query_graph_mode_resolves_reference_to_fact` | GREEN |
| traversal exceeding `maxHops` without resolving → `HopLimitExceeded` | `HopLimitExceeded` | `rag_query_graph_mode_hop_limit_exceeded` | GREEN |
| `reference`→`fact` cycle → `CycleDetected` | `CycleDetected` | `rag_query_graph_mode_cycle_detected` | GREEN |
| traversal resolving no target returns a **valid empty result** (`results:[]`,`citations:[]` + `blockedBy` of BROKEN/STALE blockers) | empty with `blockedBy` listing the blocker | `rag_query_graph_mode_empty_result_with_blocked_by` | GREEN |
| a missing target derived from liveness → empty result + `blockedBy` (no phantom walk, no error) | empty, blocked-by entry | `rag_query_graph_mode_derives_blocked_from_missing_target` | GREEN |
| `blockedBy` only on empty results | absent when some roots resolve | `rag_query_graph_mode_blocked_by_only_on_empty_results` | GREEN |
| `filters?` shape: `{nodeKind, edgeType, target, state}`; restricts what the walk/retrieval considers | nodeKind=content returns content nodes | `rag_query_filters_restrict_retrieved_nodes_by_node_kind`; malformed `edgeType` → `ValidationError` (`rag_query_malformed_filters_edge_type_is_validation_error`) | GREEN |
| `expand: 'parent'` — top `maxParentContext` results carry `parent {documentId,title,snippet,stale}`; beyond the cap without one | exactly `maxParentContext` hits carry a parent | `rag_query_expand_parent_caps_expanded_hits_by_max_parent_context` | GREEN |
| a `STALE` embed's parent carries `stale: true` | retrieved ref with stale parent flagged | `rag_query_stale_embed_parent_carries_stale_flag` | GREEN |
| deterministic walk/ordering (ties by `(documentId,nodeId)` asc) | `graph`/`hybrid` ordering deterministic across instances and runs | `graph_hybrid_ordering_is_deterministic` | GREEN |

## 4.5.3 Retrieval stack

| Documented behavior | Expected observable result | Validating test(s) | Status |
| --- | --- | --- | --- |
| lexical BM25 leg over `factKey`/`title`/`tags`/node text | exact-term hit returned | `bm25_search_returns_exact_matches_over_fact_key_and_text` | GREEN |
| `bm25_search` when BM25 index not built → `LexicalIndexUnavailable` | `LexicalIndexUnavailable` | `bm25_search_lexical_index_not_built_is_unavailable` | GREEN |
| vector leg: top-k by cosine similarity | most-similar first | `vector_search_returns_top_k_by_cosine_similarity` | GREEN |
| vector leg: unreachable embedding provider → `EmbeddingUnavailable` | `EmbeddingUnavailable` | `vector_search_embedding_provider_unreachable_is_unavailable`, `rag_query_vector_mode_unavailable_provider_is_embedding_unavailable` | GREEN |
| vector leg: vector index not built → `VectorIndexUnavailable` | `VectorIndexUnavailable` | `vector_search_vector_index_not_built_is_unavailable`, `rag_query_vector_mode_index_not_built_is_vector_index_unavailable` | GREEN |
| vector search is wiki-scoped (no cross-wiki leakage) | wiki B returns no wiki-A vectors | `vector_search_does_not_leak_vectors_across_wikis` | GREEN |
| embedding-provider seam honored over HTTP (no live Ollama) | wiremock 2xx serves the vector leg; 404 → `EmbeddingUnavailable` | `wiremock_http_embedding_provider_serves_the_vector_seam`, `…_404_is_embedding_unavailable` | GREEN |
| RRF fusion: `RRF(d)=Σ 1/(k+rank)`, `k=60` default, ties by `(documentId,nodeId)` ascending — exact merge | hand-derived ordering reproduced | `rrf_fusion_merges_ranked_lists_exactly_by_score_then_id`, `rrf_fusion_breaks_ties_by_document_then_node_id_ascending` | GREEN |
| multi-query fan-out: N variants retrieved and merged by stable identity (not ignored) | enabled result is a strict superset of single-shot | `multi_query_fan_out_is_not_ignored` | GREEN |
| contextual compression: `filter` mode is a binary keep/drop post-retrieval | filter changes the returned snippet set | `compression_filter_changes_the_snippet_set` | GREEN |
| compression `extract` (Phase 2) and `graph` (Phase 3) modes run | no error | `compression_extract_mode_runs`, `compression_graph_mode_runs` | GREEN |
| HyDE opt-in routes the hypothetical text through the vector leg | `hyde:true` surfaces the hypothetical-seeded hit; `hyde:false` doesn't | `hyde_opt_in_routes_hypothetical_through_vector_leg` | GREEN |
| cross-encoder reranking (two-stage retrieve-and-rerank) | reranked candidates; `RerankerUnavailable` on missing model | — | NOT VERIFIED (no injectable reranker seam; explicitly a boundary fail-state in the retrieval-stack suite) |
| compressor failure degrades gracefully to uncompressed (not a query failure) | query still returns | — | NOT VERIFIED (no test simulates a compressor failure) |

## 4.5.3a Multiple vector fields (coarse-to-fine)

| Documented behavior | Expected observable result | Validating test(s) | Status |
| --- | --- | --- | --- |
| a node/chunk carries at least `full`; `binary`/`other` optional-additive | `full`-only vector search works | `vector_search_returns_top_k_by_cosine_similarity` | GREEN |
| vector index keyed by `(documentId,nodeId,fieldType)` | seeded `full`+`binary` entries honored | `vector_search_binary_first_pass_narrows_pool_then_full_cosine` | GREEN |
| coarse first pass (binary Hamming) → narrowed pool → full cosine second pass | binary-first-pass returns correct top-k | same test | GREEN |
| `binaryFirstPass` when the binary index is unbuilt **degrades to the `full` search** (not an error) | still returns hits | `vector_search_binary_first_pass_degrades_to_full_when_binary_unbuilt` | GREEN |
| `binaryCandidatePool: None` applies the spec default **10× topK** cap | pool bounded, best hit beyond cap excluded | `binary_candidate_pool_none_default_caps_pool_at_tenx_topk` | GREEN |
| invalid `binaryCandidatePool` (`0`) → `ValidationError` | `ValidationError` | `vector_search_binary_candidate_pool_zero_is_validation_error`, `rag_query_binary_candidate_pool_zero_is_validation_error` | GREEN |
| `other` field type is indexable/storeable per `fieldType` | — | — | NOT VERIFIED (tests only exercise `full` and `binary`; spec treats `other` as a third field type; §4.6 lists `subTaskDag`/`other` params as typed, non-drivable) |

## 4.5.4 Agent-memory surface

| Documented behavior | Expected observable result | Validating test(s) | Status |
| --- | --- | --- | --- |
| `getFact` returns `{factKey,value,documentId,nodeId,updatedAt,citations}` | exact-key retrieval | `get_fact_returns_stored_fact` (facts suite), `get_fact_returns_exact_fact_key_matches_only` | GREEN |
| `listFacts` paginated `{items,total,page,pageSize}` | correct shape | `list_facts_returns_paginated_shape` | GREEN |
| profile summary: `getProfileSummary(wikiId)` → `{wikiId, summary, regeneratedAt, factCount}` | factCount == N, non-empty derived summary, `regeneratedAt` set | `get_profile_summary_returns_derived_doc_with_fact_count` | GREEN |
| profile summary regenerates on fact change; never drifts (single source of truth) | factCount advances; updated `fact` value surfaces in the summary text | `get_profile_summary_regenerates_when_facts_change`, `profile_surfaces_a_fact_value_change_into_the_summary` | GREEN |
| `getProfileSummary`/`listFacts`/`getFact` unknown wiki → `WikiNotFound` | `WikiNotFound` | `get_profile_summary_unknown_wiki_is_wiki_not_found` (agent-memory); facts-suite wiki tests | GREEN |

## 4.5.5 Parked surfaces

| Documented behavior | Note | Status |
| --- | --- | --- |
| automatic community detection + summaries (F4) | PARKED | out of scope per §7.5 (manual `declareCommunity` is the pinned requirement and is GREEN) |
| LLM-generated dynamic graph query (F5) | PARKED | out of scope per §7.6 |
| manual community declaration + summary | pinned requirement | GREEN (see §4.2-4-2-greens) |

## NOT VERIFIED — §4.5

1. **Cross-encoder reranking** (§4.5.3) and its fail-state `RerankerUnavailable`
   (FS-16): no reranker seam on the crate, no test.
   **→ PARKED/RESERVED:** no injectable reranker seam exists; `RerankerUnavailable`
   is a reserved variant (decision RESERVED-ERRVARIANTS-DISCIPLINE). No pending
   row needed.
2. **Compressor failure degrades gracefully to uncompressed context** (§4.5.3)
   and `CompressionFailed` (FS-17): no test drives a compressor failure.
   **→ PARKED/RESERVED:** the local compressor (Filter/Extract/Graph) only
   transforms and degrades gracefully to uncompressed — never fails-and-degrades —
   so `CompressionFailed` is a reserved variant. Already recorded in
   `docs/pending.md` ("Non-degradable compressor leg").
3. **`other` vector field type** (§4.5.3a.1/2): only `full` and `binary` are
   exercised; the `other` field type's storage/handling is not demonstrated.
   **→ PARKED/RESERVED:** `FieldType::Other` is a third, optional-additive field
   type; it is only ever exercised via `full`/`binary` (the coarse-then-fine and
   full-cosine paths), and is not drivable as a distinct §4.6 parameter. No
   pending row needed.
4. **HyDE / multi-query / sub-task-DAG / engine-internal generation fail-states**
   (`HyDEGenerationFailed` FS-18, `MultiQueryExpansionFailed` FS-19,
   `SubTaskDagFailed` FS-26): no deterministic test drives these engine-internal
   boundary failures (§4.6 reports them as not drivable from a typed test).
   **→ SPLIT:** `HyDEGenerationFailed` and `SubTaskDagFailed` are **PARKED/
   RESERVED** (the generation/DAG steps are total or unimplemented — decision
   RESERVED-ERRVARIANTS-DISCIPLINE), and `SubTaskDagFailed` is already parked in
   `docs/pending.md`. **`MultiQueryExpansionFailed` is a REAL GAP**: `expand_query_
   variants` genuinely returns it when a multi-query fan-out with `n ≥ 2` has no
   distinct indexable term to add (`term_popularity` yields nothing — e.g. an
   empty/near-empty store), so a **typed test can drive it** without any seam.
   Recorded in `docs/pending.md`.
5. **Inference-time sub-task DAG** (§4.5.3 SHOULD HAVE): `subTaskDag` is accepted
   as a param type but no test exercises an actual sub-task-DAG decomposition.
   **→ PARKED/RESERVED:** validated-only no-op (decision SUB-TASK-DAG-VALIDATED-
   ONLY) — already parked in `docs/pending.md` ("Full sub-task DAG"). No new row.
6. **`LexicalIndexUnavailable` surfaced from `ragQuery` in `mode:'hybrid'`**
   (FS-15): demonstrated at the `bm25_search` level only; no test drives a
   `hybrid` query with a definitely-unbuilt BM25 snapshot over a non-empty store.
   **→ PARKED/RESERVED (spec-design tension):** the BM25 "index" is a live shard
   scan, so an unbuilt lexical leg is only reachable over an empty wiki — on the
   `ragQuery` surface the engine **degrades gracefully** instead of erroring.
   Already recorded as a HANDOFF spec-reconcile item (FS-13/14/15) in
   `docs/HANDOFF.md`. No pending row needed.
