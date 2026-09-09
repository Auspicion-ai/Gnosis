# Gnosis — Active Defect / Requirement-Gap List

Maintained by the document-archival loop. Open gaps on top; fixed rows below.
A defect in a foundation/package (or a gap owned by another suite tool) is a
HANDOFF item — never patched here. A host-side finding (this project's `src/`)
is fixed here.

## OPEN

_(none open — the §4.1 host defects below are FIXED; see FIXED.)_

## FIXED

| Defect | Where fixed | How |
| --- | --- | --- |
| §4.1 `list_documents` panics on an out-of-range page / `page: u64::MAX` (overflow + slice OOB) | `src/store/mod.rs` | Clamp: `if start >= total { return empty page }` before any arithmetic/slice; saturating offset. Regression-tested (`list_documents_page_beyond_range_...`, `list_documents_page_u64_max_...`). |
| §4.1 `delete_document` delete-gate TOCTOU — a reference added between the scan and the remove leaves a dangling reference | `src/store/mod.rs` | Store-wide `reference_lock: RwLock<()>`: delete takes `write()` for the whole scan+remove; edge-adding mutations (`create`/`update_document`) take `read()`. Acquired before shard locks, fixed order → no deadlock. |
| §4.1 publish gate ignored `crosslink` edges in `Broken`/`Stale` | `src/store/mod.rs` | Gate now matches `link`/`embed`/`crosslink` uniformly: `BROKEN || STALE ⇒ UnresolvedReference`. Regression-tested (`publish_document_with_{broken,stale}_crosslink_...`). |
| §4.1 publish gate trusted a caller-supplied reference `state` (e.g. fabricated `Resolved` to a nonexistent target) | `src/store/mod.rs` | Publish derives reference state from target existence: a missing/archived target ⇒ `Broken` ⇒ `UnresolvedReference`; cross-wiki edges skip local verification. Regression-tested (`publish_document_with_resolved_link_to_nonexistent_target...`). |
| §4.1 `concurrent_*` tests ran on a single-threaded runtime with no `.await` → false security | `tests/store_integration.rs` | Converted to `#[tokio::test(flavor="multi_thread", worker_threads=4)]` with a `tokio::sync::Barrier`; validated 5× at `--test-threads=4`. |
| §4.1 WRITER-ACTOR-JOURNAL (ACTIVE) not honored — mutations were direct, no journal/epoch feed | `src/store/mod.rs` | Added minimal `MutationJournal` (`RwLock<Vec<JournalEntry>>` + `AtomicU64 epoch`) appended in each mutation critical section + `Store::epoch()`/`journal_len()` accessors, honoring the single-writer/epoch-feed intent for the §4.2–§4.4 index-rebuild/audit layers. |
| §4.3 `delete_document` left dangling fact citations (gate scanned only edges, not `fact_store`) + a commit-time grounding TOCTOU | `src/store/mod.rs` | Delete gate scans `fact_store` for citing facts → `DocumentInUse`; fact commits take `reference_lock.read()` and re-verify grounding inside `fact_store.write()`. Regression-tested. |
| §4.3 `update_fact` accepted empty/whitespace value | `src/store/mod.rs` | Trim → `ValidationError` before applying. Regression-tested. |
| §4.3 whitespace-only `factKey`/`value` passed schema conformance (`.is_empty()` only) | `src/store/mod.rs` | Trim before `is_empty` in `create_fact` + `propose_candidate_fact` gate. Regression-tested. |
| §4.3 unknown-wiki handling inconsistent/absent on the fact surface | `src/store/mod.rs` | `WikiNotFound` (FS-2) up front on `create_fact`/`update_fact`/`propose_candidate_fact`/`get_fact`. Regression-tested. |
| §4.3 test-impl doc mismatch: header claimed the gate handled cross-field consistency | `tests/facts_integration.rs` (+ §4.5) | Cross-field consistency declared **deferred to §4.5** (needs the embedding leg); header corrected; no fake branch. |
