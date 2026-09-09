# §4.1 Document-Store Property Register (PBT gate, artifact #1)

- **Unit:** §4.1 document store — `src/store/mod.rs`, exercised by `tests/store_integration.rs`
- **Behavior contract:** `docs/specs/gnosis.md` §4.1.1–§4.1.5 (document model, wiki model, store API, optimistic concurrency)
- **Date:** 2026-09-09
- **Author role:** spec_writer (this register is the **property register**, PBT-gate artifact **#1** of 3)

## Register contract

This register is the **property-based-testing (PBT) gate contract** for the §4.1
document-store unit. It is the first of three mandatory artifacts per
code-bearing unit:

1. **this register** (spec_writer) — the typed property rows below;
2. an **executed property layer** (TestWriter) run under `cargo test` with a
   **deterministic pinned seed**, ≤100 generated cases per register row,
   ≤400 total cases across the unit's whole property layer, stop-after-5
   (report ≤5 distinct held/broken counterexamples per row), each row recorded
   as **held** or **broken** with its strategy-id;
3. a **read-only PBT audit** (adversarial reviewer) — over-strength reasoning per
   row, generator-coverage audit, prose counterexamples, negative-generator
   requests. The reviewer never runs generators.

The property layer must satisfy **this register only**, runs under `cargo test`
with a deterministic pinned seed, and observes the crate's **public API only**
(types re-exported in `src/lib.rs` from `src/store/mod.rs`: `Store`, `Document`,
`DocState`, `DocumentId`, `WikiId`, `StoreError`, `Graph`, `Node`, `NodeId`,
`NodeKind`, `Edge`, `EdgeKind`, `ReferenceState`, `CreateDocumentRequest`,
`UpdateDocumentRequest`, `ListDocumentsFilter`, `DocumentList`, `DocumentSummary`,
and the `RagStore` methods `create_document`, `get_document`, `update_document`,
`delete_document`, `publish_document`, `unpublish_document`, `archive_document`,
`list_documents`, `create_wiki`, `get_wiki`, `list_wikis`).

Every row is a **universally-quantified invariant that is TRUE of the current
GREEN implementation** (the invariants the already-green example suite implies).
There are **no fail-state rows** (no reference to §6 `FS-*` fail-states) and no
gap/parked rows. Each `DocumentId`/`WikiId` used is created through the public
API; documents are always created against a pre-existing wiki so a success is
unambiguously an invariant hold, never masked by `WikiNotFound`.

## Property table

| Property-id | Class | Invariant | Strategy-id | Observable-as-property |
|---|---|---|---|---|
| P-IM-1 | IM | `createDocument` returns a **stable, never-reused** identity: while a document lives, its `DocumentId` never changes and always resolves to the same document; an engine-assigned `DocumentId` is never reassigned, so a re-created document receives a distinct id. *(The "never reused / distinct-on-recreate" clause is guaranteed by the **monotonic engine-id scheme** — `doc-N` backed by an `AtomicU64` counter in `src/store/mod.rs` — **not** an RFC-4122 UUID-v4 contract. This row pins the monotonic engine ids, not §4.1.1's UUID-v4 wording; see `docs/pending.md` "Engine-assigned `documentId` are not UUID v4".)* | `strat:create-roundtrip-id` | For any generated create sequence: every `get_document(id)` returns a `Document` with `document_id == id`; after `delete_document(id)`, `get_document(id)` is `DocumentNotFound`; a later `create_document` on the same wiki returns a `DocumentId` strictly different from every previously deleted id in the run. |
| P-IM-2 | IM | `createDocument`/`createWiki` validation bounds are **inclusive** in both directions: a title of exactly **200** chars and a wiki name of exactly **100** chars create successfully and **round-trip** byte-for-byte stored→read, while a title of **201** chars and a name of **101** chars are rejected with `StoreError::ValidationError`. | `strat:boundaries` | For every boundary case: `create_document(wiki, title_of_len(200))` succeeds and `get_document().title.chars().count() == 200`; `create_document(wiki, title_of_len(201))` → `ValidationError`; `create_wiki(name_of_len(100))` succeeds and `get_wiki().name == name_of_len(100)`; `create_wiki(name_of_len(101))` → `ValidationError`. |
| P-SM-1 | SM | The committed **revision is monotonic** and steps by **exactly one** per committed mutation: each successful `updateDocument` from a base revision equal to the last committed revision returns `revision == prior_committed + 1`, and a document's revision never decreases within its lifetime. *(Scope: "steps by exactly one" holds for **fresh-base** updates, i.e. `base == last committed`. The state-annotation-reconcile path of `update_document` (see P-SM-2 note) can land a jump to `current+1` that is **> base + 1** from a stale base, so that path is excluded from the "exactly one" claim.)* | `strat:create-update-seq` | For any generated create-then-update sequence from `base=0`: `update(id, base=0).revision == 1`; `update(id, base=1).revision == 2`; and so on — every consecutive successful update returns `prior_returned_revision + 1` and never a value ≤ the prior returned revision. |
| P-SM-2 | SM | Optimistic-concurrency guard is **exactly-one-winner** (§4.1.4): updates committed from the same base revision serialize so that exactly one succeeds and every other contender from that base is rejected with `StoreError::ConflictError`, advancing the stored revision by exactly one. *(Scope + exception: the "every other contender is rejected with `ConflictError`" clause applies to **content-bearing** contenders over an **annotation-free** gap. Exception: a stale base caused **solely** by a prior `set_reference_state` state-annotation bump is **RECONCILED** (lands at `current+1`), **not** rejected — `src/store/mod.rs` `reconcile_state_annotations` (§4.2.9 GRAPH-OWNS-RELATION-AND-MERGE), so both concurrent edits land without data loss.)* | `strat:revision-race-seqs` | For two `update_document(id, …)` calls carrying the **same** `base_revision = R` on a shared `Arc<Store>` released together on a multi-thread runtime: exactly one returns `Ok`, exactly one returns `Err(ConflictError)` (no panic and no other error), and the final `get_document` revision is exactly `R+1`. |
| P-SM-3 | SM | Document **state-machine legality**: a committed state is reachable only by a legal §4.1.1 transition (`DRAFT→PUBLISHED`, `PUBLISHED→DRAFT`, `DRAFT→ARCHIVED`, `PUBLISHED→ARCHIVED`); `ARCHIVED` is a terminal sink; `DocState::can_transition_to` encodes exactly the legal graph. *(Note: the state-machine drive assumes a **reference-free publishable graph** — the §4.4 publish gate's `UnresolvedReference` is a separate fail-state (returned as `Err`), **not** a §4.1 state violation. The 3×3 `can_transition_to` matrix check is a **self-referential regression guard** over the legal transition set.)* | `strat:state-machine-seq` | For any generated (possibly empty) sequence of `publish`/`unpublish`/`archive` operations that return `Ok`, the committed state after each step satisfies `can_transition_to(prior_state, new_state) == true`; once state is `Archived` no subsequent transition in the prefix succeeds; and for every pair of `DocState` values, `can_transition_to(a, b)` matches the legal transition set. |
| P-TP-1 | TP | `listDocuments` pagination is **lossless and exhaustive**: over a fixed wiki (no concurrent mutation), walking consecutive pages at a fixed `page_size` yields exactly the generated document set — no omissions, no duplicates — and `total` is identical on every page. | `strat:pagination-pages` | Generate a set `S` of documents in one wiki; for `page_size ∈ {1, 2, mid, ≥ |S|}` walk pages `1..=ceil(|S|/page_size)` collecting item `document_id`s: the collected **set** equals the generated id set (each id present, none missing), no id appears more than once across pages, and every page reports `total == |S|`. |
| P-TP-2 | TP | `updateDocument` **preserves metadata** the caller did not supply: an update whose `title`/`tags` are `None` leaves the prior title, tags, and author unchanged; supplying only `title` or only `tags` updates that field while preserving the others; `author` is preserved across **every** update (it is not a field of `UpdateDocumentRequest`). | `strat:metadata-preserve-update` | Create with a known `author` and tag set; apply any sequence of graph-only updates (`title: None`, `tags: None`) and of partially-supplied updates: after each committed update, `get_document` reports the supplied fields updated exactly when supplied and the unsupplied fields (`author`, and `title`/`tags` not supplied) byte-for-byte unchanged from the last committed value. |
| P-TP-3 | TP | **Delete-then-get** semantics: a successful `deleteDocument` makes the id permanently disappear (`getDocument` → `DocumentNotFound`, no longer in `listDocuments`), and a subsequent `createDocument` yields a **brand-new** document (distinct id, `revision 0`, `DRAFT`), never a revival of the deleted one. | `strat:delete-recreate` | For any create→delete→create sequence: after delete, `get_document(id)` is `DocumentNotFound` and the id is absent from every `list_documents` page; the re-created document has a `DocumentId` != the deleted id, `revision == 0`, `state == DocState::Draft`, and is independently listable. |

## Generator-coverage note

For each property row, the deterministic generator should **also** attempt the
following boundary and adversarial input shapes so the executed layer and the
later read-only audit can check coverage:

- **P-IM-1 (`strat:create-roundtrip-id`)** — `create_document(wiki, …)` on the *same* wiki many times (ids must stay distinct within a run); delete a middle id and re-create (new id must differ); `get_document` on a live id interleaved with unrelated updates to other docs; an empty-wiki first create; multiple wikis and confirms a created id never resolves to a document in a different wiki than the one it was created in.
- **P-IM-2 (`strat:boundaries`)** — title length exactly `0`/`1`/`199`/`200`/`201`/very-large max; wiki name `0`/`99`/`100`/`101`; non-ASCII and multi-byte-`char` titles (bounding is by `chars().count()`, so a 200-`char` BMP and a 200-`char` astral input are both `200` and both valid); a title of only-whitespace characters (valid length, kept as a shape rather than a fail expectation).
- **P-SM-1 (`strat:create-update-seq`)** — update sequences of length 0/1/2/N; base always exactly the last committed revision; alternating graph-only and title-bearing updates; a long monotonically-increasing ladder (revision must track 1 per commit).
- **P-SM-2 (`strat:revision-race-seqs`)** — two same-base contenders (barrier-released on `flavor = "multi_thread"`) over an `Arc<Store>`; a race on a fresh `revision 0` doc; a three-way same-base race (exactly one winner, the rest `ConflictError`); contenders with different payloads (the winner's payload must be the one that persists); a repeated race re-based on the newly committed revision.
- **P-SM-3 (`strat:state-machine-seq`)** — sequences covering every legal transition and the terminal sink: `DRAFT→PUBLISHED→DRAFT→ARCHIVED`; `DRAFT→ARCHIVED` (terminal, no further committed change from `publish`/`unpublish`/`archive`); `PUBLISHED→ARCHIVED`; repeated `publish`/`unpublish`/`archive` after `Archived` (no committed change); full enumeration of the 3×3 `can_transition_to` matrix against the spec's legal graph.
- **P-TP-1 (`strat:pagination-pages`)** — an **empty** wiki (`total == 0`, page 1 empty); `page_size == 1` and `page_size > total`; a `page` beyond the last page (empty `items`, correct `total`); `page_size` boundaries `1` and `100`; filters combined (`state`, `tag`) so the "generated set `S`" is the *filtered* set, not the whole wiki; a static wiki with no writes between page walks (the order-stability precondition for the no-duplicate/no-omission claim).
- **P-TP-2 (`strat:metadata-preserve-update`)** — a graph-only update right after create (author/tags/title from create preserved); an update that supplies `title` only and `tags: None` (tags preserved, title replaced); an update that supplies `tags` only (title preserved); repeated updates alternating supplied/unsupplied (each unsupplied field keeps its **last committed** value); author set at create and never touched by any update.
- **P-TP-3 (`strat:delete-recreate`)** — delete the only document in a wiki, then re-create (still a fresh distinct doc); delete-then-get on a never-created id (must be `DocumentNotFound` with no side effect); delete one of several, then `list_documents` (deleted absent, others present); repeated create→delete→create cycles (ids strictly monotonic/distinct across the whole run).

## TestWriter handoff notes (API-surface facts pinned by the current GREEN code)

- `Store` is `new()`-constructible and all `RagStore` methods are `async`; the
  concurrency rows (P-SM-2) need the store shared as `Arc<Store>` and a
  multi-thread runtime (e.g. `#[tokio::test(flavor = "multi_thread")]`) plus a
  `tokio::sync::Barrier` so the contenders genuinely contend.
- `updateDocument` never exposes `author` — `UpdateDocumentRequest` has only
  `{base_revision, graph, title: Option, tags: Option}`; `author` is always
  preserved from the last committed document (relevant to P-TP-2).
- Validation bounds are by `chars().count()`: document **title ≤ 200**, wiki
  **name ≤ 100**; `page` ≥ 1 and `page_size ∈ 1..=100` (default `20`).
- `create_document` on an unknown wiki / `list_documents` on an unknown wiki
  returns `WikiNotFound`; generators must therefore always create the wiki first
  so success is unambiguously the invariant holding (not masked by `WikiNotFound`).
- Engine-assigned ids are monotonic (`doc-N` / `wiki-N`); a deleted id is never
  reassigned. The `doc_id`/`wiki` string-wrapping helpers are from
  `tests/store_integration.rs`. For graph-bearing calls, **prefer
  `valid_graph_for(&DocumentId)`** (from `tests/props_store.rs`, which builds
  node ids matching the real document) over the id-mismatched `valid_graph()`.
