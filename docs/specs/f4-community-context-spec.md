# §7.5 F4 — Community retrieval — `getCommunityContext` behavior contract

- **Unit:** §7.5 F4 — community summaries, **re-scoped to a retrieval surface**
  (decision `F4-COMMUNITY-RETRIEVAL`; verdict in
  `docs/specs/f4-community-summaries-review.md`, PASS).
- **Status:** **LANDED + GREEN (2026-09-09)** — the F4 unit is implemented and
  trio-green (**411 tests**, clippy/fmt clean). This is the behavior contract the
  F4 unit's TestWriter derived its red set from; the `getCommunityContext`
  accessor is a **new read-only `RagStore` trait method** implemented in
  `src/store/mod.rs` (see the F4 DONE row in `docs/next-steps.md`). Zero runtime
  change; zero new runtime deps.
- **Gate:** proposal-review **PASSED** (re-scoped to retrieval) — decision
  `F4-COMMUNITY-RETRIEVAL` in `docs/decisions.md`; verdict + deliverable set +
  guardrails in `docs/specs/f4-community-summaries-review.md`.
- **Contract cross-refs:** `docs/specs/gnosis.md` §4.2.8 (community model:
  `declareCommunity`/`getCommunity`/`listCommunities`/`updateCommunitySummary`),
  §4.2.8.2/§4.2.8.3/§4.2.8.4 (manual-declaration-authoritative invariant),
  §4.4.1a/§4.4.3 (community staleness: `CommunityState` Fresh/Stale,
  `re_derive_community` clears STALE), §7.5 (F4); `docs/decisions.md`
  `F4-COMMUNITY-RETRIEVAL`; `docs/specs/f4-community-summaries-review.md`.
- **Date:** 2026-09-09. **Author-role:** spec_writer.
- **Scope:** `src/store/mod.rs` (`CommunityContext` struct + `get_community_context`
  impl) + the re-export surface in `src/lib.rs`. Companion PBT register:
  `docs/specs/4-2-graph-property-register.md` (F4 section).

---

## 1. What F4 asks

Add a **`getCommunityContext` retrieval accessor** — a new **read-only**
`RagStore` trait method that returns the **pre-joined community unit**: the
authoritative **manual summary** + the **member node set** + the current
**`CommunityState`**, as a single `CommunityContext` value. It is a
**retrieval surface, NOT a derivation surface**: it reuses stored data and
performs **no derivation, no generation, no mutation**.

The accessor fuses two existing reads — `get_community` (§4.2.8.2) and
`community_state` (§4.4.1a/§4.4.3) — into one pre-joined unit that `rag_query`
graph mode does not provide. It is a convenience accessor (marginal value is
modest), but it is cheap, well-bounded, and honest; it does **not** deliver
§7.5's global/theme benefit (that stays parked).

**Contract change:** a new `RagStore` trait method is a contract change (this
gate adjudicated it). It does **NOT** extend the F2 wire contract
(`docs/specs/engine-wire-contract.md` covers only the §4.6.1 retrieval trio +
health + audit); the wire codec for the community surface is owned by the later
shell-integration unit.

---

## 2. Scope guardrails

**In scope (this contract):**
- The `getCommunityContext` accessor — the new `RagStore` trait method + its
  `CommunityContext` return type + the `Store` impl.
- The F4 test-suite (TestWriter red set) + the F4 PBT rows (extend
  `docs/specs/4-2-graph-property-register.md`).
- The re-export of `CommunityContext` from `src/lib.rs`.

**NOT in scope (explicit):**
- **NOT a derivation surface** — no `derived_summary` field, no
  `deriveCommunitySummary`/propose-only API (vacuous + redundant — refuted by the
  review).
- **NOT LLM-generated community summaries** (suite-side — needs a
  text-generation provider, a NEW seam; `EmbeddingProvider` is embed-only).
- **NOT** hierarchical community detection, global/theme retrieval, or the full
  GraphRAG pipeline (parked).
- **NOT** a change to `re_derive_community` semantics (stays "clear the STALE
  flag"; automatic regeneration stays parked).
- **NOT** a change to `get_community`/`community_state` (the accessor is
  **additive**; those two methods keep their exact current behavior).
- **NOT** a change to the F2 wire contract (wire codec deferred to the
  shell-integration unit).
- **NOT** a change to `declare_community`/`update_community_summary`/
  `list_communities`/`mark_communities_stale`.
- **FS-24 `EnrichmentBudgetExceeded`** stays **RESERVED** (no budget, no
  enrichment pass, no fabrication — per RESERVED-ERRVARIANTS-DISCIPLINE).
- **Zero new runtime deps**; **PBT-GATE-MANDATORY applies**.

---

## 3. The signature + return shape

### 3.1 The new `RagStore` trait method

```rust
fn get_community_context(
    &self,
    community_id: &CommunityId,
) -> impl Future<Output = Result<CommunityContext, StoreError>> + Send;
```

Declared on the `RagStore` trait (`src/store/mod.rs:1516`), implemented by
`Store`. It is **read-only** and **additive** — it does not alter any existing
trait method.

### 3.2 The `CommunityContext` return type

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommunityContext {
    pub community_id: CommunityId,
    pub wiki_id: WikiId,
    pub summary: String,
    pub members: Vec<(DocumentId, NodeId)>,
    pub state: CommunityState,
}
```

- Defined in `src/store/mod.rs` and **re-exported from `src/lib.rs`** (the
  TestWriter reaches it as `gnosis::CommunityContext`).
- The derives are part of the contract: `PartialEq`/`Eq` let the TestWriter
  assert determinism/equality; `Debug`/`Clone` for assertions and reuse;
  `Serialize`/`Deserialize` consistent with the store types (the wire codec is
  deferred, but the type is serializable like `Community`/`CommunityState`).
- Field semantics (all **verbatim** from the stored record — no transformation):
  - `community_id` — the queried `CommunityId` (equals the argument).
  - `wiki_id` — the community's `wiki_id` (from the `Community` record).
  - `summary` — the **authoritative manual summary** (from the `Community`
    record, verbatim — never auto-regenerated).
  - `members` — the declared member node set (from the `Community` record,
    **complete** and in stored order; duplicate ids stored verbatim, never
    de-duplicated).
  - `state` — the current `CommunityState` (`Fresh`/`Stale`) from the
    `community_states` sidecar.

### 3.3 Fail-state: `CommunityNotFound` ONLY

`get_community_context` is a **`communityId`-keyed** accessor: it derives the
wiki from the community record, so **`WikiNotFound` cannot fire** — matching
`get_community`'s existing FS-25. The **only** fail-state is:

- **`StoreError::CommunityNotFound`** — when no community with the given
  `community_id` exists in the `communities` map.

No other `StoreError` variant is reachable from this accessor. In particular:
- `WikiNotFound` — **cannot fire** (the wiki is read from the community record,
  never looked up by id).
- `ValidationError` — **cannot fire** (no input validation; the only input is a
  `CommunityId`).
- `ConflictError`/`InvalidState`/`DocumentNotFound`/`DocumentInUse`/
  `UnresolvedReference`/`EngineUnavailable`/`EngineError`/`TraceUnavailable`/
  `HopLimitExceeded`/`CycleDetected`/`EmbeddingUnavailable`/
  `VectorIndexUnavailable`/`LexicalIndexUnavailable`/`RerankerUnavailable`/
  `CompressionFailed`/`HyDEGenerationFailed`/`MultiQueryExpansionFailed`/
  `SubTaskDagFailed` — **cannot fire** (read-only, no budget, no enrichment, no
  traversal, no provider).

---

## 4. Semantics

`get_community_context` is a **pre-joined read** of `get_community` +
`community_state`:

- **Reuses stored data** — reads the `communities` map (the `Community` record)
  and the `community_states` sidecar; it does **not** recompute, derive, or
  generate anything.
- **No derivation** — the `summary` is the stored manual summary, never
  regenerated from members.
- **No generation** — no LLM, no text generation, no provider call.
- **No mutation** — it takes only read locks; it never writes to `communities`,
  `community_states`, the journal, or any other store structure.
- **Side-effect-free** — no journal append, no state change, no revision bump.
- **Deterministic** — equal store state → equal `CommunityContext` (two
  consecutive reads with no intervening mutation return equal values).

**State projection rule.** The `state` field is a faithful projection of
`community_state`'s semantics: `Fresh` by default (a community whose staleness
was never triggered), `Stale` after a member node/edge change
(`mark_communities_stale`), `Fresh` again after `re_derive_community` clears the
flag. The accessor does **not** change `state`; it only reports it.

---

## 5. Documented valid/happy + fail states (TestWriter assertion guide)

### 5.1 Valid/happy states

| # | Store state | `get_community_context` returns |
| --- | --- | --- |
| H-1 | A community declared via `declareCommunity(members, {summary, wiki_id})` | `Ok(CommunityContext)` with `community_id == the queried id`, `wiki_id == the declared wiki_id`, `summary == the manual summary` (verbatim), `members == the declared node set` (complete, stored order), `state == Fresh` (a freshly declared community is `Fresh`, §4.4.3). |
| H-2 | After a member node/edge change that triggers `mark_communities_stale` — `updateDocument` rewriting a member node (line 2527), or `update_fact` on a fact the community incorporates (line 2288) | `Ok(CommunityContext)` with `state == Stale`; `summary` and `members` **unchanged** (read-only — the accessor does not mutate). |
| H-3 | After `re_derive_community(community_id)` on the `Stale` community | `Ok(CommunityContext)` with `state == Fresh`; `summary` and `members` **unchanged** (re-derive only clears the STALE flag; the manual summary is never regenerated). |
| H-4 | After `update_community_summary(community_id, new)` | `Ok(CommunityContext)` with `summary == new` (the new manual summary, §4.2.8.4 authority); `state` **unchanged** (update_community_summary does not touch `community_states`). |
| H-5 | A community whose members span **two documents** | `Ok(CommunityContext)` with `members` containing the full cross-document set (each `(DocumentId, NodeId)` present). |
| H-6 | A **single-member** community | `Ok(CommunityContext)` with `members.len() == 1`. |
| H-7 | A member that is a **fact location** (a `(DocumentId, NodeId)` pointing at a fact node) | `Ok(CommunityContext)` with that member present in `members` (membership is by declared id, not by node kind). |
| H-8 | A community whose staleness was **never triggered** | `Ok(CommunityContext)` with `state == Fresh` (the `community_state` default, §4.4.1a/§4.4.3). The accessor must defensively `unwrap_or(Fresh)` like `community_state` (line 4040) — a `Fresh` default even if no `community_states` row exists. This defensive branch is **not directly constructible** via the public API (`declare_community` always inserts a `Fresh` row, line 3181), so the TestWriter asserts the **observable** (`Fresh` when staleness was never triggered), not the unconstructible pre-population state. |
| H-9 | **Determinism** — two consecutive `get_community_context` calls with **no intervening mutation** | Both return **equal** `CommunityContext` values (field-for-field `==`). |
| H-10 | **Membership completeness** — after declaration, `members` **exactly equals** the declared node set (no additions, no removals, no reordering). |
| H-11 | **Manual-override** — after a member change + `re_derive_community`, `summary` still equals the manual summary (never auto-regenerated). |

### 5.2 Fail states

| # | Store state | `get_community_context` returns |
| --- | --- | --- |
| F-1 | An unknown `community_id` (no matching `Community` record) | `Err(StoreError::CommunityNotFound)` — the **only** fail-state. |
| F-2 | A `community_id` whose wiki does not exist | **`Ok`** — `WikiNotFound` **cannot fire** (the wiki is read from the community record; the accessor never looks up the wiki by id). This is a **documented non-fail-state** the TestWriter must assert (a community record always carries a `wiki_id`, so the accessor succeeds regardless of whether that wiki is otherwise known). |

### 5.3 Boundary / adversarial shapes (TestWriter generator guidance)

- **Single-member community** (H-6) — `members.len() == 1`.
- **Members spanning two documents** (H-5) — cross-document membership.
- **A member that is a fact location** (H-7) — membership by declared id, not
  node kind.
- **Duplicate member ids** — `declare_community` stores `node_ids` **verbatim**
  and does **not** de-duplicate, so `members` may contain duplicate ids; the
  accessor must report them **as stored** (asserted as stored).
- **Empty member set** — **cannot be constructed** via `declare_community`
  (empty node set → `ValidationError`); the generator must only emit
  non-empty member sets (a `members.len() == 0` context is unreachable).
- **Empty summary** — **cannot be constructed** via `declare_community` (empty
  summary → `ValidationError`); the generator must only emit non-empty
  summaries.
- **Unknown `community_id`** (F-1) — the negative generator confirms
  `CommunityNotFound` (a fail-state, so it is a §4.2.8.5 concern, outside the
  PBT invariant rows).

---

## 6. Cross-references

- **Gate record + verdict + deliverable set + guardrails:**
  `docs/specs/f4-community-summaries-review.md` (PASS — re-scoped to retrieval).
- **Decision:** `docs/decisions.md` `F4-COMMUNITY-RETRIEVAL`.
- **Contract authority:** `docs/specs/gnosis.md` §4.2.8 (community model),
  §4.2.8.2 (the community example), §4.2.8.3 (community storage), §4.2.8.4
  (manual override precedence — the manual summary is the single source of truth
  and is never overwritten), §4.4.1a (community staleness as the third
  propagator), §4.4.3 (enforcement rules; `re_derive_community` clears STALE),
  §7.5 (F4).
- **PBT:** `docs/specs/4-2-graph-property-register.md` (F4 section — the PBT
  rows).
- **Wire:** `docs/specs/engine-wire-contract.md` — the F4 accessor does **NOT**
  extend the F2 wire contract; the wire codec for the community surface is
  deferred to the shell-integration unit.

**Suite-side / parked (NOT Gnosis):**
- LLM-generated community summaries (needs a text-generation provider — a NEW
  seam; `EmbeddingProvider` is embed-only).
- Hierarchical community detection; global/theme retrieval (the §7.5 benefit);
  the full GraphRAG pipeline.
- Any `derived_summary` field or `deriveCommunitySummary`/propose-only API
  (vacuous + redundant — refuted).
- Any change to `re_derive_community` semantics.
- The wire codec for the community surface (deferred to the shell-integration
  unit).
- FS-24 `EnrichmentBudgetExceeded` (stays RESERVED).

---

## 7. What the TestWriter derives (red-set readiness)

- `tests/community_context_integration.rs` — the §5 happy/fail states: H-1..H-11
  and F-1/F-2, plus the §5.3 boundary shapes.
- `tests/props_community_context.rs` — the F4 PBT rows of
  `docs/specs/4-2-graph-property-register.md` (P-IM-1, P-IM-2, P-SM-1, P-SM-2,
  P-TP-1, P-TP-2).
- The red set is derived **from this contract alone**; the Implementer lands the
  least `src/store/mod.rs` + `src/lib.rs` code to green.
