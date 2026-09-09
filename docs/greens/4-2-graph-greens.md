# §4.2 Knowledge graph — blind greens

Derived **from the documentation only** (`docs/specs/gnosis.md` §4.2.1–§4.2.9,
§4.4.2, §6) by the Blind-Test Writer. Validation: ran the built crate's
integration suites (`CARGO_HOME=/tmp/gnosis-cargo-home cargo test`, 8 suites,
**233 tests green**) and mapped documented behaviors to exercising tests in
`tests/graph_integration.rs` (66 tests). **No behavior was verified by reading
`src/`.**

Legend: `GREEN` = demonstrably exhibited; `NOT VERIFIED` = documented but not
demonstrated by a test.

## 4.2.1–4.2.4 Model

| Documented behavior | Expected observable result | Validating test(s) | Status |
| --- | --- | --- | --- |
| Node kinds `content` / `fact` / `reference` | graph nodes are typed; fixtures build all three | throughout `graph_integration.rs` | GREEN (structural) |
| Edge kinds incl. `doc-head` / `doc-end` / `doc-child` (exactly one head/end per doc) | valid graphs require one head + one end | `update_document_invalid_graph_is_validation_error` (store suite rejects missing head/end) | GREEN |
| `link` stores only the target pointer (no value copy); `embed` stores a snapshot | reference node `value` is `None` for a link, `Some(...)` for an embed | `ref_node` fixtures; embed state semantics in §4.4 suite | GREEN (structural, exercised via §4.4) |
| `reference`→`fact` graph is resolved in dependency order | target resolved before dependent | `resolve_references_walks_in_dependency_order` | GREEN |

## 4.2.5 Adjacency methods

| Documented behavior | Expected observable result | Validating test(s) | Status |
| --- | --- | --- | --- |
| `edgesFrom` returns outgoing edges | correct count / kind / source | `edges_from_returns_outgoing_edges` | GREEN |
| `edgesFrom` unknown doc → `DocumentNotFound` | `DocumentNotFound` | `edges_from_unknown_document_is_document_not_found` | GREEN |
| `edgesFrom` invalid node → `ValidationError` | `ValidationError` | `edges_from_invalid_node_is_validation_error` | GREEN |
| `edgesTo` returns incoming edges | correct kind / target | `edges_to_returns_incoming_edges` | GREEN |
| `edgesTo` unknown doc / invalid node fail-states | as documented | `edges_to_unknown_document…`, `edges_to_invalid_node…` | GREEN |
| `edgesByKind` filters outgoing edges by kind | only the kind returned | `edges_by_kind_filters_by_kind` | GREEN |
| `edgesByKind` unknown doc / invalid node fail-states | as documented | `edges_by_kind_invalid_node_is_validation_error` | GREEN |
| `edgesByKind` invalid `kind` → `ValidationError` | `ValidationError` | — | NOT VERIFIED (`kind` is a Rust-typed `EdgeKind`; an out-of-set kind is not representable from a typed test) |
| `edgesForDocument` returns all edges | all 4 edges | `edges_for_document_returns_all_edges` | GREEN |
| `edgesForDocument` unknown doc → `DocumentNotFound` | `DocumentNotFound` | `edges_for_document_unknown_document_is_document_not_found` | GREEN |
| `docHeadForDocument` returns the head node | correct node id / value / kind | `doc_head_for_document_returns_head_node` | GREEN |
| `docHeadForDocument` unknown doc → `DocumentNotFound` | `DocumentNotFound` | `doc_head_unknown_document_is_document_not_found` | GREEN |
| `docHeadForDocument` on a doc with no `doc-head` edge → `InvalidState` | `InvalidState` | `doc_head_without_doc_head_edge_is_invalid_state` | GREEN |
| concurrent graph readers see the same graph | 8 racing readers agree | `concurrent_adjacency_readers_see_same_graph` | GREEN |

## 4.2.6 Topological resolution

| Documented behavior | Expected observable result | Validating test(s) | Status |
| --- | --- | --- | --- |
| references resolve in topological order (facts before dependents) | `f_leaf < r_mid < r_top` in resolution order | `resolve_references_walks_in_dependency_order` | GREEN |
| each resolved fact reused across all dependents (no redundant resolution) | shared leaf appears exactly once | `resolve_references_reuses_facts_across_dependents` | GREEN |
| `reference`→`fact` cycle → `CycleDetected` | `CycleDetected` | `resolve_references_cycle_is_cycle_detected` | GREEN |
| hop cap exceeded → `HopLimitExceeded` | `HopLimitExceeded` | `resolve_references_over_hop_cap_is_hop_limit_exceeded` | GREEN |
| `maxHops` bounded to 1..=5 (0 and 6 → `ValidationError`) | `ValidationError` for 0 and 6 | `resolve_references_zero_max_hops_is_validation_error`, `…_over_five_max_hops_…` | GREEN |
| unknown wiki → `WikiNotFound` on the resolver | `WikiNotFound` | `resolve_references_unknown_wiki_is_wiki_not_found` | GREEN |

## 4.2.7 Triples

| Documented behavior | Expected observable result | Validating test(s) | Status |
| --- | --- | --- | --- |
| `addTriple` stores a `relation` edge owned by the graph (single source of truth) | one `Relation` edge, correct source/target/`relationType` | `add_triple_stores_relation_edge_in_document`, `triple_relation_type_is_recoverable_from_graph_edge` | GREEN |
| `addTriple` returns `{subject, relation, object, relationType, createdAt}` | all fields returned | `add_triple_stores_relation_edge_in_document` | GREEN |
| `addTriple` empty relation → `ValidationError` | `ValidationError` | `add_triple_empty_relation_is_validation_error` | GREEN |
| `addTriple` unknown subject/object doc → `DocumentNotFound` | `DocumentNotFound` | `add_triple_unknown_subject_document_is_document_not_found` | GREEN |
| `addTriple` invalid node → `ValidationError` | `ValidationError` | `add_triple_invalid_node_is_validation_error` | GREEN |
| `addTriple` unknown wiki → `WikiNotFound` | `WikiNotFound` | `add_triple_unknown_wiki_is_wiki_not_found` | GREEN |
| `getTriples` direction `out` → triples where node is subject | relation + object correct | `get_triples_out_direction_returns_subjected_triples` | GREEN |
| `getTriples` direction `in` → triples where node is object | subject correct | `get_triples_in_direction_returns_objected_triples` | GREEN |
| `getTriples` direction `both` → all touching | all 3 | `get_triples_both_direction_returns_all_touching` | GREEN |
| `getTriples` filtered by `relationType` | only matching | `get_triples_filters_by_relation_type` | GREEN |
| `getTriples` unknown wiki → `WikiNotFound` | `WikiNotFound` | `get_triples_unknown_wiki_is_wiki_not_found` | GREEN |
| `getTriples` invalid node / empty relationType → `ValidationError` | `ValidationError` | `get_triples_invalid_node_is_validation_error`, `…_empty_relation_type_…` | GREEN |
| `queryTriples` relation wildcard → all matching | both `depends_on` triples | `query_triples_relation_wildcard_returns_all_matching` | GREEN |
| `queryTriples` subject wildcard → outgoing | one out-triple | `query_triples_subject_wildcard_returns_outgoing` | GREEN |
| `queryTriples` unknown wiki → `WikiNotFound` | `WikiNotFound` | `query_triples_unknown_wiki_is_wiki_not_found` | GREEN |
| `queryTriples` `limit < 1` / `> 100` → `ValidationError` | `ValidationError` | `query_triples_limit_zero…`, `…_over_one_hundred…` | GREEN |
| triple whose subject/object node is deleted cascades away | triple gone; re-add yields exactly one (no zombie/duplicate) | `triples_cascade_when_subject_node_is_removed`, `…_object_node…`, `triple_cascade_removes_relation_edge_and_does_not_duplicate_on_readd` | GREEN |

## 4.2.8 Manual override (communities + reference state)

| Documented behavior | Expected observable result | Validating test(s) | Status |
| --- | --- | --- | --- |
| `declareCommunity` stores a `community` node + members + summary; `getCommunity` round-trips | identical members/summary/wiki | `declare_community_create_and_get_round_trip` | GREEN |
| `listCommunities` returns all declared in the wiki | both communities | `list_communities_returns_declared` | GREEN |
| `updateCommunitySummary` updates the summary | summary changed + persisted | `update_community_summary_changes_summary` | GREEN |
| manual declaration/summary is authoritative, never overwritten by automatic computation | summary persists across a member-touching change | `community_manual_summary_is_authoritative_and_persists` | GREEN |
| `declareCommunity` empty node set / empty summary → `ValidationError` | `ValidationError` | `declare_community_empty_members…`, `…_empty_summary…` | GREEN |
| `getCommunity`/`updateCommunitySummary` unknown id → `CommunityNotFound` | `CommunityNotFound` | `get_community_unknown…`, `update_community_summary_unknown…` | GREEN |
| `listCommunities` unknown wiki → `WikiNotFound` | `WikiNotFound` | `list_communities_unknown_wiki_is_wiki_not_found` | GREEN |
| `setReferenceState` marks `FRESH`/`STALE`/`RESOLVED`/`BROKEN` and persists | each state applied + persisted in the graph | `set_reference_state_marks_each_valid_state` | GREEN |
| `setReferenceState` invalid state → `ValidationError` | `ValidationError` | `set_reference_state_invalid_state_is_validation_error` | GREEN |
| `setReferenceState` preserves other nodes/edges and advances revision by one | no clobber, rev +1 | `set_reference_state_preserves_other_edges_and_nodes` | GREEN |
| concurrent `setReferenceState` + `updateDocument` lose no edit | both edits survive | `concurrent_set_reference_state_and_update_document_do_not_lose_updates` | GREEN |

## 4.2.9 Entity resolution / merge

| Documented behavior | Expected observable result | Validating test(s) | Status |
| --- | --- | --- | --- |
| `resolveEntities` merges/aliases others to the `canonicalId` | merged pairs + aliases point to canonical | `resolve_entities_merges_and_aliases_to_canonical` | GREEN |
| `resolveEntities` without canonical picks one canonical | canonical ∈ input, rest aliased | `resolve_entities_without_canonical_picks_one` | GREEN |
| `resolveEntities` persists a durable alias→canonical mapping | `entityAliasCanonical(alias)` == canonical | `resolve_entities_records_durable_alias_to_canonical_mapping` | GREEN |
| `resolveEntities` empty set / invalid canonical → `ValidationError` | `ValidationError` | `resolve_entities_empty_entities…`, `…_invalid_canonical…` | GREEN |
| `resolveEntities` unknown doc → `DocumentNotFound` | `DocumentNotFound` | `resolve_entities_unknown_document_is_document_not_found` | GREEN |
| `mergeFacts` keeps canonical value + union of citations, persists | canonical value, 3-citation union, durable | `merge_facts_unions_citations_and_keeps_canonical_value`, `merge_facts_persists_merged_fact_to_store` | GREEN |
| `mergeFacts` conflicting values, no resolution → `ConflictError` | `ConflictError` | `merge_facts_conflicting_values_is_conflict_error` | GREEN |
| `mergeFacts` empty `factKeys` / invalid canonical key → `ValidationError` | `ValidationError` | `merge_facts_empty_keys…`, `…_invalid_canonical_key…` | GREEN |
| a merge leaving the canonical fact with **zero citations** → `ValidationError` (§4.2.9.4) | `ValidationError` | — | NOT VERIFIED (degenerate: every fact already requires ≥1 citation, so a union of valid facts is non-empty; no test drives it) |

## NOT VERIFIED — §4.2

1. **`edgesByKind` invalid `kind` → `ValidationError`** (§4.2.6/FS-3). `kind` is a
   Rust-typed `EdgeKind`; an out-of-set kind is not representable from a typed
   test, so no test drives it.
   **→ PARKED/RESERVED:** boundary-only — the out-of-set `kind` is unrepresentable
   from the typed `EdgeKind`, so this is §5.1-transport validation (a typed test
   cannot trigger it). No pending row needed.
2. **`mergeFacts` → `ValidationError` when the merged fact would have zero
   citations** (§4.2.9.4 / §4.3.2). No test exercises the zero-citation merge
   outcome; given the minimum-citation invariant it may be unreachable via the
   public API.
   **→ PARKED/RESERVED:** effectively unreachable — every source fact already
   satisfies the ≥1-citation invariant (§4.3.2 `a fact must have at least one
   citation`), so a union of valid facts is always non-empty; the outcome is
   defensive-only. No pending row needed.

*(Note: `resolve_references`, `entityAliasCanonical`, and the `community`/`Member`
node/edge kinds are crate-internal surfacing of these documented behaviors; they
are exercised via the public operations above.)*
