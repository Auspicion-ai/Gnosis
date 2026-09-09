# §5.1 Property register — §4.4 Consistency enforcement

| Header field | Value |
| --- | --- |
| Unit | §4.4 Consistency enforcement (Gnosis, production graph/vector engine) |
| Canonical contract | `docs/specs/gnosis.md` §§4.4.1, 4.4.1a, 4.4.2, 4.4.3, 4.4.4, 4.4.5 |
| Concurrency contract | `docs/research/gnosis-data-structures-concurrency-plan.md` (SHARDED-RWLOCK-STORE, ARC-SHARED-ENGINE, LOCK-ORDER-REF-SHARD-SIDECAR) |
| Implementation | `src/store/mod.rs` (`RagStore` seam + `Store`); public types/methods re-exported from `src/lib.rs` |
| Spec section | §5.1 of this register file (author) |
| Date | 2026-09-09 (aligned with the §4.4-DONE entry in `docs/next-steps.md`; the register derives from the current GREEN §4.4 surface) |
| Role | `role_spec_writer` — typed §5.x property register, artifact #1 of the new mandatory PBT gate |

---

## PBT-gate note

In force for **every code-bearing unit** (Rust adaptation of the JS gate-preset
persona). Three REQUIRED artifacts per unit; this file is artifact **#1**:

1. **#1 — THIS file.** A typed property register by the SpecWriter: classed,
   universally-quantified invariants that are black-box observable through ONLY
   the crate's public API (the `gnosis::*` re-exports of `src/lib.rs`), TRUE of
   the current GREEN implementation, with a reusable generator strategy-id and a
   concrete assertion per generated case. No fail-state rows (§6/FS-n), no
   gap/parked rows (§/F-). Invariants only.
2. **#2 (TestWriter, later).** An EXECUTED property layer under `cargo test`
   with a **deterministic pinned seed**, **≤100 generated cases per row**,
   **≤400 total cases**, and **stop-after-5** (≤5 distinct held/broken
   counterexamples per row). Each row is tagged **HELD** or **BROKEN** plus the
   `strategy-id` it exercised.
3. **#3 (Adversarial reviewer, after).** A **READ-ONLY** PBT audit — checks
   over-strength per row (does an invariant assert more than the GREEN code
   provides?), generator coverage, prose-vs-property counterexamples, and
   negative-generator requests. Reviewers NEVER run generators.

Register invariants are black-box assertions over the public surface. The test
harness must use the following public operations to build/observe stores:

- Setup/mutation: `Arc<Store>` (`Store::new()`), `create_wiki`, `create_document`,
  `update_document(&UpdateDocumentRequest)`, `update_fact(&UpdateFactRequest)`,
  `archive_document`, `publish_document`, `unpublish_document`,
  `set_reference_state`, `re_sync_embed`, `re_derive_community`,
  `declare_community(&DeclareCommunityOptions)`.
- Read/derive: `get_document`, `get_consistency_report(&WikiId)`,
  `get_fact(&WikiId, &str)`, `community_state(&CommunityId)`,
  `Store::epoch()` and `Store::journal_len()`.
- Public value types: `ConsistencyReferenceReport` (`document_id`, `node_id`,
  `kind`, `state`, `target`, `cross_wiki`), `ReferenceState`
  (`Fresh|Stale|Resolved|Broken`), `CommunityState` (`Fresh|Stale`), `EdgeKind`
  (`Link|Embed|Crosslink`), `Node`, `Edge`, `Graph`, `NodeKind`, `DocState`,
  `StoreError`.

The report's `state` is read from the edge's stored `ReferenceState`, or — for
an edge stored with `state: None` — derived from the **target's liveness** and,
for a snapshot embed, **snapshot-vs-canonical** (§4.4.2/§4.4.3;
`derive_reference_state`, F2/F4/F5). A snapshot embed is an `Embed` edge, or a
`Crosslink` whose source reference node carries a copied `value`; a pure `Link`
(or a snapshot-less `Crosslink`) is a live reference (`is_snapshot_embed`).

> **SpecWriter decisions recorded** (per task instructions):
> - **LOCK-ORDER-REF-SHARD-SIDECAR** is respected: `publish_document` validates
>   its whole §4.4.3 gate under **read** locks, then takes the single shard
>   `write()` only for the `DRAFT→PUBLISHED` transition. Consequently the
>   register asserts publish-gate **consistency and atomicity** (no partial
>   state change, no deadlock under concurrent publish+fact-commit), but does
>   NOT assert "a concurrent target mutation between validation and the
>   transition is re-validated" — the current GREEN code does not re-run the
>   target-liveness check under the write, so that claim is excluded (P-BROKEN
>   would be flagged by the read-only audit).
> - **Reference states are DERIVED from target liveness**, never fabricated
>   from caller input: `derive_unstated_reference_states` re-stamps `None`-state
>   non-cross-wiki edges from the new target; explicit caller states on
>   non-cross-wiki edges are still honored by the report (stored verbatim) —
>   the no-fabrication invariant (P-IM-3) covers the `None` and "must-have-live-
>   target-for-Fresh/Resolved" cases that hold in GREEN.
> - **Propagation bumps revision + journal**: `update_fact` →
>   `propagate_fact_staleness`, `archive_document` →
>   `propagate_archived_target` each write `revision + 1` + a journal entry; the
>   state-effect of re-sync/re-derive is idempotent but the revision still bumps
>   — so idempotency (P-SM-2) is scoped to **derived state**, monotonicity
>   (P-SM-3) to **revision/epoch**.

---

## Property table

`CLASS ∈ {IM, SM, TP}` · Property-id `P-<CLASS>-<N>` · `Strategy-id` = reusable
generator name · `Observable-as-property` = concrete assertion on every
generated case.

| Property-id | Class | Invariant | Strategy-id | Observable-as-property |
| --- | --- | --- | --- | --- |
| `P-IM-1` | IM | **Determinism.** `get_consistency_report(w)` is a pure function of the store state: for a fixed store, two consecutive calls with no intervening mutation return **identical ordered sequences** (element-wise, including order); the output changes between two calls only if a mutation committed between them. | `ref_store_basic` | Call `get_consistency_report(w)` twice back-to-back; assert `Vec<ConsistencyReferenceReport>` equality (`a == b`). Then commit one mutation (e.g. one `update_fact`) and assert the before/after reports differ in at least one row. |
| `P-IM-2` | IM | **Scope + well-formedness.** The report contains exactly one row per reference edge (`kind ∈ {Link, Embed, Crosslink}`) whose **source document** is `w`; every row is well-typed (`kind ∈ {Link, Embed, Crosslink}`, `state ∈ {Fresh, Stale, Resolved, Broken}`, `target` a `(DocumentId, NodeId)`, `cross_wiki ∈ {true, false}`); the multiset of `(document_id, node_id, kind)` row keys equals the reference-edge key set read from the authoritative documents of `w`. | `ref_store_basic` | For each document of `w` (`get_document`), scan `graph.edges` and collect `(docId, e.source.node_id, e.kind)` for reference kinds; assert this set equals the report's key set and `|report|` == the reference-edge count. Assert each row's `kind`/`state`/`target`/`cross_wiki` field invariants. |
| `P-IM-3` | IM | **No fabrication — state derives from target liveness.** For every non-`cross_wiki` report row, the reported `state` is ground truth from the store, never fabricated: a row reports `Resolved` (live reference) only if its target is a **live node** (existing, non-archived) or a **committed fact location**; a row reports `Fresh` (snapshot embed) only if its target is live **and** its reference node's snapshot equals the target's canonical value; a `None`-stored edge's reported `state` equals the derivation from target liveness + snapshot-vs-canonical. Equivalently: **no non-cross-wiki row reports `Resolved`/`Fresh` while its target is missing or archived**, and no snapshot embed whose held snapshot differs from canonical reports `Fresh`. *(Scope: the no-fabrication claim applies to rows derived from edges stored with `state: None`; rows with an explicit caller state are stored verbatim and are out of scope.)* | `mixed_liveness` | For each non-cross_wiki row: if `state == Resolved`, assert `target` is live per the generator's known sets (its doc exists, is non-`Archived`, has the node; or it is a seeded fact location); if `state == Fresh`, assert target live AND snapshot == canonical (reconstruct canonical via the seeded fact value / `get_document` of the target node). |
| `P-SM-1` | SM | **Fact-update propagation monotonicity.** After `update_fact` changes a fact's canonical value to `v'`, every snapshot-embed edge (same-wiki `Embed`, or `Crosslink` whose source node carries a copied snapshot) whose target is the changed fact location and whose held snapshot ≠ `v'` is reported `STALE`; a `Link` to that fact (no snapshot) stays `Resolved`; and every community whose member set includes that fact location reports `community_state == Stale`. | `chain_of_many` + `stale_many_refs` | Seed fact `(d, fact-K)` = "A", create M referrers embedding "A" + one `Link` to it + one cross-wiki `Crosslink` embed, declare a community over `(d, fact-K)`; `update_fact` → "B"; assert every embed row is `Stale` (in both wikis' reports), the `Link` row is `Resolved`, `community_state` is `Stale`, and no embed row is `Fresh`. |
| `P-SM-2` | SM | **Fixpoint / idempotency (derived state).** The report read is side-effect-free (a fixpoint of the read: consecutive reads with no mutation are equal). Re-running a corrective operation over an already-consistent store leaves the **derived** state unchanged: `re_sync_embed` of an already-`Fresh` embed yields a report row still `Fresh`; `re_derive_community` of an already-`Fresh` community leaves `community_state == Fresh`. | `ref_store_basic` | Two consecutive reports equal (re-assert P-IM-1). Then on an already-Fresh embed call `re_sync_embed`; assert its row stays `Fresh`. On an already-Fresh community call `re_derive_community`; assert `community_state` stays `Fresh`. (Revision/epoch still bump — out of scope for this row, covered by P-SM-3.) |
| `P-SM-3` | SM | **Revision/epoch monotonicity under propagation.** Any propagation that mutates a referencing document strictly bumps that document's `revision` (**strictly increased; +1 per referencing edge when the propagation touches one edge per referencing doc**) and records a journal entry; `re_sync_embed` bumps the referencing document's `revision` by exactly +1; `Store::epoch()` is non-decreasing across the whole run and equals the count of committed mutations (`journal_len()`). | `stale_many_refs` | Before/after each triggering mutation (`update_fact`, `archive_document`, `re_sync_embed`), read `get_document(d).revision` and `epoch()`/`journal_len()`; assert revision strictly increased (propagation) and `re_sync_embed` == +1; assert `epoch()` non-decreasing and `epoch() == journal_len()` at every observation. |
| `P-TP-1` | TP | **Target-change re-stamp.** When a reference edge written with caller `state == None` is repointed via `update_document` to a target `T`, the edge's derived state immediately reflects `T`'s liveness (and, for a snapshot embed, `T`'s canonical value vs the held snapshot): to a live target → `Resolved`/`Fresh`; to a missing/archived target → `Broken`/`Stale`; repointing back restores the earlier state. The report never stores or reports a stale caller edge-state verbatim for a `None`-state edge. | `repoint_matrix` | Seed live target doc with node `n1` and a ghost `(missingDoc, node)`; a referrer's `Link` (state `None`) → report `Resolved`; `update_document` repointing it to the ghost (state `None`) → report `Broken`; repoint back to `n1` → report `Resolved` again. Repeat with an `Embed` (state `None`) with matching / mismatching snapshot (`Fresh` ↔ `Stale`). |
| `P-TP-2` | TP | **Publish-gate consistency + atomicity.** For a document **in a state reachable to `Published`** (an already-`Published`/`Archived` doc returns `InvalidState` legitimately), `publish_document(d)` returns `Ok(state == Published)` iff, at validation, none of `d`'s reference edges is reported `Broken` or (stored-or-derived) `Stale` (every reference is `Resolved`/`Fresh` against a live, non-archived target, or honored cross-wiki) **and** no node of `d` is a member of a `Stale` community. When it fails it returns `Err(UnresolvedReference)` (or `InvalidState`) and the document's state is **unchanged** (no partial state mutation). A successfully published document, read back, has every non-cross-wiki reference with a live target and `state ∈ {Resolved, Fresh}`. | `publish_matrix` | (a) all-clean refs → publish `Ok` and state `Published`; (b) a `Stale` embed → `Err(UnresolvedReference)`, state still `Draft`; after `re_sync_embed` → `Ok`; (c) a `Broken` link → `Err`; (d) a node that is a member of a `Stale` community → `Err`; after `re_derive_community` → `Ok`. On every failure assert `get_document(d).state` is unchanged. After a success assert the re-read report shows all non-cross-wiki refs `Fresh`/`Resolved` with live targets. |

---

## Generator-coverage note

One reusable generator is named per row. Per row, the generator MUST cover the
boundary **and** adversarial shapes below; these are coverage requirements on
strategy-id, not additional invariants.

- **P-IM-1 / P-IM-2 (`ref_store_basic`).** Cover: a wiki with 0, 1, and many
  referencing documents (`M` up to the per-row case bound); a `Link`, an
  `Embed`, and a `Crosslink`; `state ∈ {None, Some(Fresh), Some(Stale),
  Some(Resolved), Some(Broken)}`; a target that is live, missing, archived, and
  **self-referential** (a node in the same document); a second wiki whose doc
  carries a `Crosslink` into `w` (must appear in the *other* wiki's report, not
  `w`'s).
- **P-IM-3 (`mixed_liveness`).** Draw targets from {live node, missing doc,
  archived doc, committed-fact location} and store edges with `state: None` so
  the report MUST derive truth rather than replay a caller default. Boundary:
  a snapshot-less `Crosslink` (→ `Resolved`, not `Fresh`), a snapshot embed of
  an archived target (→ `Stale`), a `None`-state link to a missing target
  (→ `Broken`).
- **P-SM-1 (`chain_of_many`, `stale_many_refs`).** A linear **chain of
  referencing documents** all snapshotting the same fact; a **fact that is
  stale with many referencing docs** (same wiki `M` embeds + a `Link` + at
  least one cross-wiki `Crosslink` embed + a community incorporating the fact);
  assert the single `update_fact` propagation marks all embeds (both wikis)
  and the community, and leaves the `Link` `Resolved`. Boundary: an embed whose
  snapshot **ALREADY equals `v'`** is **STILL marked `STALE`** by the
  unconditional propagation scan (spec §4.4.3 marks **every** snapshot embed of a
  changed fact `STALE`; the scan does not compare snapshot-to-canonical) — only a
  subsequent `re_sync_embed` restores `Fresh`. Assert it reports `Stale` (never
  `Fresh`) after `update_fact`, then `Fresh` after `re_sync_embed`; the generator
  should include one such coincidentally-equal embed.
- **P-SM-2 (`ref_store_basic`).** Boundary: an already-`Fresh` embedded fact; an
  already-`Fresh` community; a re-sync on an embed whose target value equals the
  snapshot.
- **P-SM-3 (`stale_many_refs`).** Boundary: `M = 0` (no referencing doc — no
  revision bump expected), `M = 1`, and `M` many; a re-sync of an embed and an
  archive-driven propagation; read `epoch()` after every mutation to assert
  non-decreasing + `== journal_len()`.
- **P-TP-1 (`repoint_matrix`).** Boundary: **archive of a cited target**;
  repoint of a link to a **newly-created** vs **missing** target; repoint an
  embed whose snapshot matches / mismatches the new canonical; repoint back and
  forth (round-trip stability); a self-referential repoint.
- **P-TP-2 (`publish_matrix`).** Boundary: all-clean publish; a `Stale` embed
  blocks then clears after `re_sync_embed`; a `Broken` link is **never resolved
  by re-syncing an unrelated embed**; a `Stale` community blocks until
  `re_derive_community`; a `BROKEN`+`STALE` mix. Adversarial concurrent shape:
  **concurrent publish + fact-commit + re-sync sequences** launched behind a
  `Barrier` on a multi-thread tokio runtime, targeting docs whose reference
  targets/facts live in the same shard space — assert every handle **completes
  (no deadlock)**, no handle panics, and the final `get_consistency_report`
  still satisfies P-IM-3 (no fabricated resolved/fresh row over a missing/
  archived target). This exercises LOCK-ORDER-REF-SHARD-SIDECAR without holding
  two shard writes.

---

## Notes for the TestWriter

- **Build helpers.** Reuse the shape of `tests/consistency_integration.rs`:
  `wiki(s)`, `did(s)`, `nid(s)`, `content_node`, `ref_node` (snapshot = embed,
  `None` = live link), `edge`, `new_wiki`, `new_doc`, `apply_graph`,
  `citation_graph`, `seed_fact` (fact lives at `(src.document_id,
  nid(fact-{key}))`). A snapshot embed's reference node `value` carries the
  copied snapshot; a `Link` node's `value` is `None`.
- **Report scope note.** The report emits one row per reference edge whose
  **source** doc is in `w` — including a `Crosslink` sourced in `w` (target
  elsewhere). A `Crosslink` sourced in another wiki is NOT part of `w`'s report.
- **Cross-wiki caveat (P-IM-3).** Skip the liveness assertion for rows with
  `cross_wiki == true` (target lives in another store, unverifiable locally);
  the invariant's universal quantification covers non-cross-wiki rows only.
- **Crosslink snapshot semantics (F5b).** A `Crosslink` node with `value: None`
  is a **live reference** → derive `Resolved` (never `Fresh`). Only include it
  as a snapshot embed (→ `Fresh`/`Stale`) when its source node carries a
  snapshot.
- **Idempotency scope (P-SM-2).** Revision/epoch still bump on re-sync/re-derive;
  assert the row/community derived state, not the revision.
- **Failure rows are excluded.** None of the above invites a fail-state
  assertion (no §6/FS-n rows). Use `StoreError::UnresolvedReference` ONLY within
  P-TP-2's "failure returns `Err` and leaves state unchanged" observable — that
  is an invariance over the gate, not a fail-state inventory.
- **Case budget.** Per PBT gate: deterministic pinned seed, ≤100 generated
  cases per row, ≤400 total, stop-after-5, each row tagged HELD/BROKEN +
  strategy-id. The named strategies are shared across rows; do not let one row
  consume the whole 400-case budget.
