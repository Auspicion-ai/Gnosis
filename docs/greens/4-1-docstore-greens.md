# §4.1 Document store — blind greens

Derived **from the documentation only** (`docs/specs/gnosis.md` §4.1.1–§4.1.5,
§6) by the Blind-Test Writer. Validation was performed by running the built
crate's integration suites (`CARGO_HOME=/tmp/gnosis-cargo-home cargo test`,
8 suites, **233 tests green**) and reading the test names/assertions in
`tests/store_integration.rs` to map each documented behavior to an exercising
test. **No behavior below was verified by reading `src/`.**

Legend: `GREEN` = demonstrably exhibited by the built crate (an exercising test
passes); `NOT VERIFIED` = documented but no test demonstrates it (or the test
only proves a weaker property).

## 4.1.1 Document unit

| Documented behavior | Expected observable result | Validating test(s) | Status |
| --- | --- | --- | --- |
| New `Document` at `revision = 0`, state `DRAFT`, belongs to the supplied wiki | `createDocument` returns rev 0 / DRAFT / the wiki | `create_document_initial_state_revision_zero_draft` | GREEN |
| `createdAt` / `updatedAt` present (ISO-8601 UTC metadata) | both fields non-empty on a created doc | `create_document_initial_state_revision_zero_draft` | GREEN (fields present; **format** not asserted — see NOT VERIFIED) |
| `tags` metadata stored | created doc carries the provided tags | `create_document_initial_state_revision_zero_draft` | GREEN |
| `author` metadata stored | created doc carries the provided author | — | NOT VERIFIED (no test asserts the `author` field) |
| Document identity stable / immutable once created | same `documentId` returned on re-read; reads resolve to that document | `create_document_initial_state_revision_zero_draft` + `get_document_returns_document_at_current_revision` | GREEN (stable identity demonstrated) |
| `documentId` is a **UUID v4**, globally-unique, never reused | engine-assigned ids are UUID-v4 and never collide | — | NOT VERIFIED (no test asserts the id format / reuse guarantee; tests use arbitrary string ids elsewhere and only round-trip engine ids) |
| State machine `DRAFT → PUBLISHED → ARCHIVED`, incl. unpublish (PUBLISHED→DRAFT), archive from both, ARCHIVED terminal | legal transitions true; self-transitions / ARCHIVED→anything false | `doc_state_legal_transitions` | GREEN |

## 4.1.3 Store operations

| Documented behavior | Expected observable result | Validating test(s) | Status |
| --- | --- | --- | --- |
| `createDocument` → rev 0 / DRAFT | covered above | above | GREEN |
| `createDocument` on unknown `wikiId` → `WikiNotFound` | `StoreError::WikiNotFound` | `create_document_unknown_wiki_returns_wiki_not_found` | GREEN |
| `createDocument` empty title → `ValidationError` | `ValidationError` | `create_document_empty_title_is_validation_error` | GREEN |
| `createDocument` title > 200 → `ValidationError` | `ValidationError` | `create_document_overlong_title_is_validation_error` | GREEN |
| title exactly 200 is valid (inclusive bound) | creates + persists | `create_document_title_exactly_200_is_valid` | GREEN |
| `getDocument` returns the doc at its current revision | matches created rev / state | `get_document_returns_document_at_current_revision` | GREEN |
| `getDocument` unknown id → `DocumentNotFound` | `DocumentNotFound` | `get_document_unknown_id_is_document_not_found` | GREEN |
| `updateDocument` bumps `revision` to base+1, applies graph/title | rev 1 + new title | `update_document_bumps_revision_and_applies_changes` | GREEN |
| `updateDocument` invalid Provident graph → `ValidationError` | `ValidationError` (no doc-head/doc-end) | `update_document_invalid_graph_is_validation_error` | GREEN |
| `updateDocument` unknown id → `DocumentNotFound` | `DocumentNotFound` | `update_document_unknown_id_is_document_not_found` | GREEN |
| `updateDocument` stale base revision → `ConflictError` | `ConflictError` when base `0` reused after commit | `update_document_stale_base_is_conflict_error` | GREEN |
| Optimistic concurrency: two commits from the same base — exactly one wins, loser `ConflictError` | 1 success / 1 conflict, no panic | `concurrent_update_same_base_exactly_one_succeeds` (multi-thread, barrier) | GREEN |
| Concurrent readers all get the same document (coherent read) | same rev/state across 8 racing readers | `concurrent_readers_all_get_document` | GREEN |
| `deleteDocument` removes the doc | later `getDocument` → `DocumentNotFound` | `delete_document_removes_it` | GREEN |
| `deleteDocument` unknown id → `DocumentNotFound` | `DocumentNotFound` | `delete_document_unknown_id_is_document_not_found` | GREEN |
| `deleteDocument` of a referenced doc → `DocumentInUse` (§4.4.5) | `DocumentInUse` when a link targets it | `delete_document_when_referenced_is_document_in_use` | GREEN |
| non-reference edge into another doc does **not** block delete | `DocChild`/`Relation` cross-doc edges allow the delete | `delete_document_with_non_reference_edge_to_other_doc_is_allowed` | GREEN |
| `publishDocument` DRAFT→PUBLISHED | state `Published` | `publish_document_transitions_draft_to_published` | GREEN |
| resolved crosslink does not block publish | publishes | `publish_document_with_resolved_crosslink_is_allowed` | GREEN |
| publish gate: BROKEN link → `UnresolvedReference` | blocked | `publish_document_with_broken_reference_is_unresolved_reference` | GREEN |
| publish gate: STALE embed → `UnresolvedReference` | blocked | `publish_document_with_stale_embed_is_unresolved_reference` | GREEN |
| publish gate: BROKEN crosslink → `UnresolvedReference` | blocked | `publish_document_with_broken_crosslink_is_unresolved_reference` | GREEN |
| publish gate: STALE crosslink → `UnresolvedReference` | blocked | `publish_document_with_stale_crosslink_is_unresolved_reference` | GREEN |
| publish gate: a link to a nonexistent target is `UnresolvedReference` regardless of stored state | blocked (liveness-derived) | `publish_document_with_resolved_link_to_nonexistent_target_is_unresolved_reference` | GREEN |
| `publishDocument` unknown id → `DocumentNotFound` | `DocumentNotFound` | `publish_document_unknown_id_is_document_not_found` | GREEN |
| `unpublishDocument` PUBLISHED→DRAFT | state `Draft` | `unpublish_document_transitions_published_to_draft` | GREEN |
| `unpublishDocument` on non-PUBLISHED → `InvalidState` | `InvalidState` | `unpublish_document_not_published_is_invalid_state` | GREEN |
| `unpublishDocument` unknown id → `DocumentNotFound` | `DocumentNotFound` | `unpublish_document_unknown_id_is_document_not_found` | GREEN |
| `archiveDocument` DRAFT→ARCHIVED and PUBLISHED→ARCHIVED | state `Archived` | `archive_document_transitions_draft_to_archived`, `…_published_to_archived` | GREEN |
| `archiveDocument` of an ARCHIVED doc → `InvalidState` | `InvalidState` (terminal) | `archive_document_already_archived_is_invalid_state` | GREEN |
| `archiveDocument` unknown id → `DocumentNotFound` | `DocumentNotFound` | `archive_document_unknown_id_is_document_not_found` | GREEN |
| `listDocuments` → `{items,total,page,pageSize}` | correct shape + totals | `list_documents_returns_page_shape` | GREEN |
| `listDocuments` filters by `state`, `tag` | only matching docs | `list_documents_filters_by_state`, `…_by_tag` | GREEN |
| empty wiki lists empty page | `total 0`, empty items | `list_documents_on_empty_wiki_returns_empty_page` | GREEN |
| `listDocuments` on unknown wiki → `WikiNotFound` | `WikiNotFound` | `list_documents_unknown_wiki_is_wiki_not_found` | GREEN |
| `page < 1`, `pageSize < 1`, `pageSize > 100` → `ValidationError` | `ValidationError` | `list_documents_page_less_than_one…`, `…_size_less_than_one…`, `…_over_one_hundred…` | GREEN |
| out-of-range page (incl. `u64::MAX`) returns empty items, no panic | empty items, `total`/page correct | `list_documents_page_beyond_range_returns_empty_items`, `…_page_u64_max_returns_empty_items` | GREEN |
| `createWiki` returns the named wiki | wiki name round-trips | `create_wiki_returns_wiki` | GREEN |
| `createWiki` empty name → `ValidationError` | `ValidationError` | `create_wiki_empty_name_is_validation_error` | GREEN |
| `createWiki` name > 100 → `ValidationError` | `ValidationError` | `create_wiki_overlong_name_is_validation_error` | GREEN |
| wiki name exactly 100 valid (inclusive bound) | creates + persists | `create_wiki_name_exactly_100_is_valid` | GREEN |
| `getWiki` returns the wiki | name round-trips | `get_wiki_returns_wiki` | GREEN |
| `getWiki` unknown id → `WikiNotFound` | `WikiNotFound` | `get_wiki_unknown_id_is_wiki_not_found` | GREEN |
| `listWikis` returns all wikis | both created wikis listed | `list_wikis_returns_all_wikis` | GREEN |

## 4.1.4 / 4.1.5

| Documented behavior | Expected observable result | Validating test(s) | Status |
| --- | --- | --- | --- |
| optimistic concurrency enforced store-side (caller must supply base revision; stale base rejected) | ConflictError + caller re-read/re-apply | `update_document_stale_base_is_conflict_error`, `concurrent_update_same_base_exactly_one_succeeds` | GREEN |
| `RagStore` interface exposes the store operations | (architectural seam; the crate's `Store` implements these operations) | whole suite | GREEN (structural) |

## NOT VERIFIED — §4.1

1. **`documentId` is a UUID v4, globally-unique, never reused** (§4.1.1). No test
   asserts the format or the reuse/global-uniqueness guarantee; engine-assigned
   ids are only round-tripped (stable identity is GREEN, the UUID claim is not).
   **→ REAL GAP (spec-impl tension):** the engine currently emits `doc-{N}`
   (a monotonically increasing counter), **not** an RFC-4122 UUID v4, so the
   §4.1.1 UUID-v4 identity contract is not honored by the id scheme. Recorded as
   an engine-internal deferred item in `docs/pending.md`.
2. **`createdAt`/`updatedAt` are ISO-8601 UTC format** (§4.1.1). Tests assert only
   non-empty, not the format.
   **→ REAL GAP (quick correctness pin):** `iso_now()` already emits ISO-8601 UTC
   (`YYYY-MM-DDThh:mm:ss.nnnnnnnnnZ`), so only a format assertion is missing.
   Recorded in `docs/pending.md`.
3. **`author` metadata** (§4.1.1). The fixture sets `author` but no test asserts
   it is stored/returned.
   **→ REAL GAP (quick correctness pin):** the store sets `author` on
   `createDocument` and preserves it on `updateDocument`, so only a round-trip
   assertion is missing. Recorded in `docs/pending.md`.
