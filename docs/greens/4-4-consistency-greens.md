# §4.4 Consistency enforcement — blind greens

Derived **from the documentation only** (`docs/specs/gnosis.md` §4.4.1–§4.4.5,
§6) by the Blind-Test Writer. Validation: ran the built crate's integration
suites (`CARGO_HOME=/tmp/gnosis-cargo-home cargo test`, 8 suites, **233 tests
green**) and mapped documented behaviors to exercising tests in
`tests/consistency_integration.rs` (22), `tests/store_integration.rs` (50).
**No behavior was verified by reading `src/`.**

Legend: `GREEN` = demonstrably exhibited; `NOT VERIFIED` = documented but not
demonstrated by a test.

## 4.4.1/4.4.1a Invariant + staleness propagation

| Documented behavior | Expected observable result | Validating test(s) | Status |
| --- | --- | --- | --- |
| fact value is the single source of truth; embeds must reflect it, links resolve live | embed snapshots mirror the canonical `Fact.value` | throughout the suite | GREEN |
| on fact update: every `embed` snapshotting it → `STALE` (same wiki) | embed becomes `STALE` | `fact_update_marks_embeds_stale_links_resolved_and_community_stale` | GREEN |
| on fact update: `link` to the fact stays `RESOLVED` (resolved live) | link stays `RESOLVED` | same test | GREEN |
| on fact update: **cross-wiki** embed → `STALE` (across wikis) | crosslink embed becomes `STALE` | same test (report on wiki w2) | GREEN |
| on fact update: community incorporating the fact → `STALE` | `community_state` == Stale | same test | GREEN |
| embeds **created after** the propagation scan with a stale snapshot are still reported `STALE` | new doc's stale embed reported `STALE` | `propagation_covers_embeds_created_after_fact_update` | GREEN |
| propagation bumps the referencing document's `revision` | referencing doc's rev increases after the fact-update propagation | `fact_propagation_bumps_referencing_document_revision` | GREEN |
| archiving a reference target marks a `link` `BROKEN` and an `embed` `STALE` | link `BROKEN`, embed `STALE` | `archiving_target_marks_link_broken_and_embed_stale` | GREEN |
| a member-node change marks its community `STALE` (third propagator) | newly declared community `Fresh` → `Stale` after member edit | `member_node_change_marks_community_stale` | GREEN |

## 4.4.2 Reference states

| Documented behavior | Expected observable result | Validating test(s) | Status |
| --- | --- | --- | --- |
| a `link`/`embed`/`crosslink` is in exactly one of `FRESH`/`STALE`/`RESOLVED`/`BROKEN` | each state can be set and reported | `set_reference_state_marks_each_valid_state` (graph suite); report state rows | GREEN |
| reference state is derived from **target liveness**, not caller-verbatim | a `None`-state link to a missing target → `BROKEN`; `None`-state embed of an archived target → `STALE` | `none_state_reference_edges_derive_from_target_liveness` | GREEN |
| repointing a link re-stamps state from the new target's liveness | repoint to missing → `BROKEN`; back to live → `RESOLVED` | `repointing_link_restamps_state_from_target_liveness` | GREEN |
| a live `crosslink` (no snapshot) to a present target → `RESOLVED` (not `FRESH`) | `RESOLVED` | `crosslink_live_link_with_none_state_is_resolved` | GREEN |

## 4.4.3 Re-sync / re-derive / publish gate

| Documented behavior | Expected observable result | Validating test(s) | Status |
| --- | --- | --- | --- |
| re-syncing a `STALE` embed updates its snapshot to the canonical value and bumps the referencing doc's `revision` | snapshot `MIT`, rev +1, embed `FRESH` | `resync_embed_updates_snapshot_and_bumps_revision` | GREEN |
| re-sync carries the **new** canonical value after a fact change | snapshot `B`, rev +1, `FRESH` | `resync_embed_after_fact_change_carries_new_canonical` | GREEN |
| re-sync unknown document → `DocumentNotFound` | `DocumentNotFound` | `resync_embed_unknown_document_is_document_not_found` | GREEN |
| re-deriving a `STALE` community clears staleness and preserves the manual summary | `Fresh`, summary unchanged | `rederive_community_clears_stale_flag` | GREEN |
| re-derive unknown community → `CommunityNotFound` | `CommunityNotFound` | `rederive_community_unknown_is_community_not_found` | GREEN |
| `community_state` unknown → `CommunityNotFound` | `CommunityNotFound` | `community_state_unknown_is_community_not_found` | GREEN |
| publish gate: a `STALE` embed (unsynced) blocks `publishDocument` → `UnresolvedReference`; publish succeeds after re-sync | blocked then published | `stale_embed_blocks_publish_until_resynced` | GREEN |
| publish gate: a `STALE` community (underived) blocks publish of a member doc; succeeds after re-derive | blocked then published | `stale_community_blocks_publish_until_rederived` | GREEN |
| publish gate: a `BROKEN` link blocks publish even after re-syncing an unrelated embed | still `UnresolvedReference` | `broken_link_blocks_publish_even_after_resync_of_unrelated_embed` | GREEN |

## 4.4.4 Consistency report

| Documented behavior | Expected observable result | Validating test(s) | Status |
| --- | --- | --- | --- |
| `getConsistencyReport(wikiId)` returns one row per reference edge in the wiki with `{documentId, nodeId, kind, state, target, crossWiki}` | rows for embed/link/crosslink, correct fields, `crossWiki` flag | `consistency_report_lists_every_reference_with_fields` | GREEN |
| report scope is source-doc-in-wiki (crosslink source in the wiki is included) | w2's report surfaces its own crosslink | same test | GREEN |
| unknown wiki → `WikiNotFound` | `WikiNotFound` | `consistency_report_unknown_wiki_is_wiki_not_found` | GREEN |

## 4.4.5 Reference integrity on delete + concurrency

| Documented behavior | Expected observable result | Validating test(s) | Status |
| --- | --- | --- | --- |
| `deleteDocument` fails with `DocumentInUse` if another doc references the target (no dangling references) | `DocumentInUse` on a cross-document link target | `delete_document_when_referenced_is_document_in_use` (store suite) | GREEN |
| only `link`/`embed`/`crosslink` reference edges trigger the delete gate | non-reference cross-doc edges don't block | `delete_document_with_non_reference_edge_to_other_doc_is_allowed` | GREEN |
| concurrent re-sync of the same embed is atomic | every contender succeeds; embed deterministically `FRESH` with canonical snapshot | `concurrent_resync_of_same_embed_is_atomic` | GREEN |
| concurrent fact-update + report reads are coherent (no torn state) | readers see only FRESH or STALE for the embed, never torn | `concurrent_fact_update_with_report_reads_are_coherent` | GREEN |
| publish/fact/resync stress completes without deadlock or panic | all handles resolve (multi-thread) | `publish_fact_commit_resync_stress_completes` | GREEN |

## NOT VERIFIED — §4.4

None. Every documented §4.4 behavior has an exercising, passing test.
