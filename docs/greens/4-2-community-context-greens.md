# §7.5 F4 — Community retrieval — `getCommunityContext` blind greens

Derived **from the documentation only** (`docs/specs/f4-community-context-spec.md`
§3–§5, `docs/specs/4-2-graph-property-register.md` §7.5 F4 rows, and the public
type definitions in `src/lib.rs` + `src/store/mod.rs`) by the Blind-Test Writer.
Validation: ran independent checks through the **public `RagStore` API** in a
throwaway `#[test]` (`tests/blind_community_check.rs`, since deleted) against the
live crate. **No behavior was verified by reading the `get_community_context`
implementation body or the existing community-context tests.**

Legend: `GREEN` = demonstrably exhibited by an independent check; `NOT VERIFIED` =
documented but not demonstrated from the docs alone.

## Signature + return shape

| Documented behavior | Expected observable result | Validating check | Status |
| --- | --- | --- | --- |
| `get_community_context(&CommunityId) -> Result<CommunityContext, StoreError>` is a new read-only `RagStore` method | callable through the public trait; returns `Ok(CommunityContext)` for a declared community | `store.get_community_context(&cid).await` | GREEN |
| `CommunityContext { community_id, wiki_id, summary, members, state }` re-exported as `gnosis::CommunityContext` | type reachable at `gnosis::CommunityContext` with the five documented fields | `use gnosis::CommunityContext;` + field access | GREEN |
| `CommunityContext` derives `PartialEq`/`Eq`/`Debug`/`Clone`/`Serialize`/`Deserialize` | field-for-field `==` and `Debug`/`Clone` usable | `assert_eq!(a, b)` on two reads | GREEN |
| `community_id` equals the queried id | `ctx.community_id == cid` | H-1 | GREEN |
| `wiki_id` from the `Community` record | `ctx.wiki_id == declared wiki_id` | H-1 | GREEN |
| `summary` is the authoritative manual summary, verbatim | `ctx.summary == manual summary` | H-1 | GREEN |
| `members` is the declared member set, complete, in stored order | `ctx.members == declared node_ids` | H-1/H-10 | GREEN |
| `state` is the current `CommunityState` from the sidecar | `ctx.state == community_state(id)` | H-1/P-TP-2 | GREEN |

## §5.1 Happy states

| # | Documented behavior | Expected observable result | Validating check | Status |
| --- | --- | --- | --- | --- |
| H-1 | A community declared via `declareCommunity(members, {summary, wiki_id})` | `Ok(CommunityContext)` with `community_id == queried id`, `wiki_id == declared wiki_id`, `summary == manual summary` (verbatim), `members == declared node set` (complete, stored order), `state == Fresh` | declared a 2-member community; asserted all five fields | GREEN |
| H-2 | After `updateDocument` rewriting a member node (a `mark_communities_stale` trigger) | `state == Stale`; `summary` and `members` unchanged | rewrote a member node via `update_document`; asserted `state == Stale` and summary/members identical to H-1 | GREEN |
| H-3 | After `re_derive_community(community_id)` on the `Stale` community | `state == Fresh`; `summary` and `members` unchanged | called `re_derive_community`; asserted `state == Fresh`, summary/members unchanged | GREEN |
| H-4 | After `update_community_summary(community_id, new)` | `summary == new`; `state` unchanged | updated the summary; asserted `ctx.summary == new` and `state` unchanged | GREEN |
| H-5 | A community whose members span **two documents** | `members` contains the full cross-document set (each `(DocumentId, NodeId)` present) | declared members across two docs; asserted both present | GREEN |
| H-6 | A **single-member** community | `members.len() == 1` | declared a 1-member community; asserted `len == 1` | GREEN |
| H-7 | A member that is a **fact location** (a `(DocumentId, NodeId)` pointing at a fact node) | that member present in `members` (membership by declared id, not node kind) | declared a fact node as a member; asserted it present | GREEN |
| H-8 | A community whose staleness was **never triggered** | `state == Fresh` (the `community_state` default) | freshly declared community; asserted `state == Fresh` | GREEN |
| H-9 | **Determinism** — two consecutive reads with no intervening mutation | both return equal `CommunityContext` values (field-for-field `==`) | read twice with no mutation; `assert_eq!(a, b)` | GREEN |
| H-10 | **Membership completeness** — after declaration, `members` exactly equals the declared node set | no additions, no removals, no reordering | `assert_eq!(ctx.members, declared.to_vec())` | GREEN |
| H-11 | **Manual-override** — after a member change + `re_derive_community`, `summary` still equals the manual summary | never auto-regenerated | member change + re-derive; asserted `ctx.summary == manual summary` | GREEN |

## §5.2 Fail states

| # | Documented behavior | Expected observable result | Validating check | Status |
| --- | --- | --- | --- | --- |
| F-1 | An unknown `community_id` (no matching `Community` record) | `Err(StoreError::CommunityNotFound)` — the **only** fail-state | `get_community_context(&unknown)`; asserted `matches!(err, StoreError::CommunityNotFound)` | GREEN |
| F-2 | A `community_id` whose wiki does not exist | **`Ok`** — `WikiNotFound` **cannot fire** (the wiki is read from the community record, never looked up by id) | declared a community with a `wiki_id` that was never created as a wiki; asserted `Ok` (never `WikiNotFound`) | GREEN |

## §5.3 Boundary / adversarial shapes

| Documented behavior | Expected observable result | Validating check | Status |
| --- | --- | --- | --- |
| **Single-member community** (H-6) | `members.len() == 1` | declared a 1-member community; asserted `len == 1` | GREEN |
| **Members spanning two documents** (H-5) | cross-document membership, both present | declared members across two docs; asserted both present | GREEN |
| **A member that is a fact location** (H-7) | membership by declared id, not node kind | declared a fact node as a member; asserted present | GREEN |
| **Duplicate member ids** — `declare_community` stores `node_ids` verbatim, never de-duplicated | `members` reports duplicates as stored | declared a member set with a duplicate id; asserted `ctx.members == declared.to_vec()` (duplicate preserved) | GREEN |
| **Empty member set** — cannot be constructed via `declare_community` (empty → `ValidationError`) | unreachable via the public API | not driven (unconstructible) | NOT VERIFIED (unconstructible) |
| **Empty summary** — cannot be constructed via `declare_community` (empty → `ValidationError`) | unreachable via the public API | not driven (unconstructible) | NOT VERIFIED (unconstructible) |
| **Unknown `community_id`** (F-1) | `CommunityNotFound` | F-1 | GREEN |
| **`addTriple` is NOT a staleness trigger** — only `update_document` rewriting a member node / `update_fact` on an incorporated fact mark a community `Stale` | after `addTriple` on a member node, `state` stays `Fresh` | declared a community; `add_triple` on a member node; asserted `state == Fresh` | GREEN |

## F4 PBT rows (from `docs/specs/4-2-graph-property-register.md` §7.5)

| Property | Invariant | Validating check | Status |
| --- | --- | --- | --- |
| P-IM-1 | Context determinism: two consecutive reads with no intervening mutation return field-for-field equal values | read twice, `assert_eq!(a, b)` | GREEN |
| P-IM-2 | Membership completeness: `context.members` exactly equals the declared set (complete, stored order, duplicates verbatim) | `assert_eq!(ctx.members, declared.to_vec())` | GREEN |
| P-SM-1 | Read side-effect-free: no mutation, no journal entry, no store-state change; consecutive reads equal | read twice; asserted equal and `state` unchanged | GREEN |
| P-SM-2 | State reflects staleness: `Fresh` on declaration, `Stale` after a member change, `Fresh` after `re_derive_community`; summary/members unchanged | H-1→H-2→H-3 sequence; asserted state transitions + unchanged summary/members | GREEN |
| P-TP-1 | Manual-summary authority: `summary` always equals the manual summary, never auto-regenerated; changes only via `update_community_summary` | H-11 + H-4 | GREEN |
| P-TP-2 | Faithful projection: `community_id`/`wiki_id`/`summary`/`members` from `get_community`, `state` from `community_state` | read `get_community`, `community_state`, `get_community_context`; asserted all five fields match | GREEN |

## NOT VERIFIED — F4

1. **Empty member set** (§5.3) — `declare_community` rejects an empty node set with
   `ValidationError`, so a `members.len() == 0` context is **unconstructible** via
   the public API. No independent check can drive it.
   **→ PARKED/RESERVED:** unreachable through the public API; defensive-only.
2. **Empty summary** (§5.3) — `declare_community` rejects an empty summary with
   `ValidationError`, so an empty-summary context is **unconstructible** via the
   public API. No independent check can drive it.
   **→ PARKED/RESERVED:** unreachable through the public API; defensive-only.

*(The `get_community_context` accessor is a pre-joined read of `get_community` +
`community_state`; the crate-faithful `mark_communities_stale` triggers are
`update_document` rewriting a member node and `update_fact` on an incorporated
fact — **not** `add_triple`. The H-2/H-3/P-SM-2/P-TP-1 checks used the
`update_document`-rewrites-a-member-node trigger, and the `addTriple` non-trigger
was independently verified: after `add_triple` on a member node the community
stays `Fresh`.)*
