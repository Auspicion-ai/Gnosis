# §4.2.9 `resolveEntities` authoritative-overwrite — blind greens

Derived **from the documentation only** (`docs/decisions.md`
RESOLVE-ENTITIES-AUTHORITATIVE-OVERWRITE; `docs/specs/gnosis.md` §4.2.9.1,
§4.2.9.2, §4.2.8.4; `docs/specs/4-2-graph-property-register.md` P-TP-2) by the
Blind-Test Writer. Validation: ran an **independent** throwaway integration test
(`tests/blind_resolve_check.rs`, since deleted) against the live crate through
the public `RagStore` API (`resolve_entities` / `entity_alias_canonical` /
`create_document` / `update_document` / `create_wiki`). **No behavior was
verified by reading `src/store/mod.rs` `resolve_entities`/`entity_alias_canonical`
implementation bodies or the existing `tests/props_graph.rs` /
`tests/graph_integration.rs` resolve_entities tests.**

Legend: `GREEN` = the live crate exhibited the documented behavior under an
independent check; `RED` = the crate contradicts the docs (a genuine finding);
`NOT VERIFIED` = documented but not demonstrable from the docs alone.

## Decided semantics under test (RESOLVE-ENTITIES-AUTHORITATIVE-OVERWRITE)

- `resolveEntities` is an **authoritative overwrite**: each call re-stabilizes
  the durable alias→canonical map to an **acyclic, flat** alias graph.
- The chosen canonical is **never itself an alias** (its prior entry is dropped).
- Every alias in the requested set points **directly** to the canonical; any
  prior alias that pointed to a now-re-aliased node is re-pointed via
  **path-compression**.
- A later manual call supersedes an earlier one (**manual-vs-manual
  last-write-wins**); the §4.2.9.2/§4.2.8.4 "never overwritten by an
  **automatic** computation" invariant is preserved (only manual calls touch the
  map).
- **Same-call idempotence** (P-TP-2) holds; **overlap convergence is guaranteed**
  — no residual alias, no cycle.

## Scenarios

| # | Documented behavior | Expected observable result | Independent check (throwaway assertion) | Status |
| --- | --- | --- | --- | --- |
| 1 | **Basic resolution** — `resolveEntities([e1,e2], {canonicalId: e1})` aliases e2 to e1; the canonical is never itself an alias | `entityAliasCanonical(e2) == Some(e1)`; `entityAliasCanonical(e1) == None` | After the call, read `entity_alias_canonical(&e2)` and `entity_alias_canonical(&e1)` through the public API | GREEN |
| 2 | **Same-call idempotence** (P-TP-2) — re-applying the identical call leaves the durable map unchanged, no alias flips | equal `ResolutionResult`; every recorded alias still maps to the same canonical | Snapshot `entity_alias_canonical` for each alias, re-apply the identical call, re-read — same canonical, and the two `ResolutionResult`s are equal | GREEN |
| 3 | **Overlap flip** — `resolveEntities([e1,e2], {canonicalId: e1})` then `resolveEntities([e1,e2], {canonicalId: e2})` | `entityAliasCanonical(e1) == Some(e2)`; `entityAliasCanonical(e2) == None`; no residual, no cycle | After the second call, read both aliases; assert e1→e2 and e2→None | GREEN |
| 4 | **Chain flatten (path-compression)** — `resolveEntities([e2,e3], {canonicalId: e2})` then `resolveEntities([e1,e2], {canonicalId: e1})` | `entityAliasCanonical(e3) == Some(e1)` (path-compressed, not `Some(e2)`) | After the second call, read `entity_alias_canonical(&e3)` | GREEN |
| 5 | **Convergence back** — after a flip, re-apply the original call | full convergence, no residual alias | After re-applying the original call, every alias in the set maps directly to the canonical; no alias maps to a non-canonical | GREEN |
| 6 | **Fail-states** — empty `entityIds` → `ValidationError`; `canonicalId` not among `entityIds` → `ValidationError`; unknown document → `DocumentNotFound` | the three documented `StoreError` variants | Call with empty slice, with a canonical not in the set, and with an unknown document id; match the returned error | GREEN |
| 7 | **Return shape** — `ResolutionResult { merged, aliases, canonical_id }` well-formed | `canonical_id == e1`; `aliases` contains `{alias: e2, canonical: e1}`; `merged` contains `{from: e2, to: e1}` | Inspect the returned `ResolutionResult` fields | GREEN |

## Result

- **GREEN: 7 / 7** — every scenario derived from the docs was independently
  demonstrated against the live crate through the public API.
- **RED: 0** — no docs-vs-crate contradiction observed.
- **NOT VERIFIED: 0** — every scenario was runnable from the docs alone.

## Verification detail (how each GREEN was established)

The throwaway `tests/blind_resolve_check.rs` built a `Store`, created a wiki and
a document carrying content nodes `e1`/`e2`/`e3` (a valid Provident graph with
`doc-head`/`doc-end` edges), then drove `resolve_entities` and
`entity_alias_canonical` through the public `RagStore` trait:

1. **Basic resolution** — `resolve_entities(&[e1,e2], {canonical_id: Some(e1)})`
   returned a `ResolutionResult`; `entity_alias_canonical(&e2)` returned
   `Some(e1)` and `entity_alias_canonical(&e1)` returned `None`.
2. **Same-call idempotence** — re-applied the identical call; the second
   `ResolutionResult` was `==` the first, and `entity_alias_canonical(&e2)`
   still returned `Some(e1)` (no flip).
3. **Overlap flip** — applied `resolve_entities(&[e1,e2], {canonical_id:
   Some(e2)})`; `entity_alias_canonical(&e1)` returned `Some(e2)` and
   `entity_alias_canonical(&e2)` returned `None` (no residual, no cycle).
4. **Chain flatten** — applied `resolve_entities(&[e2,e3], {canonical_id:
   Some(e2)})` then `resolve_entities(&[e1,e2], {canonical_id: Some(e1)})`;
   `entity_alias_canonical(&e3)` returned `Some(e1)` (path-compressed).
5. **Convergence back** — after the flip in (3), re-applied the original
   `resolve_entities(&[e1,e2], {canonical_id: Some(e1)})`; the map fully
   converged (e2→e1, e1→None, no residual).
6. **Fail-states** — `resolve_entities(&[], …)` → `StoreError::ValidationError`;
   `resolve_entities(&[e1,e2], {canonical_id: Some(e3)})` (e3 not in the set) →
   `StoreError::ValidationError`; `resolve_entities(&[unknown], …)` →
   `StoreError::DocumentNotFound`.
7. **Return shape** — the returned `ResolutionResult` had `canonical_id == e1`,
   `aliases == [{alias: e2, canonical: e1}]`, and `merged == [{from: e2, to: e1}]`.

## NOT VERIFIED — none

Every scenario in this set was runnable from the docs alone and was
independently demonstrated. No documented behavior was left unverified.
