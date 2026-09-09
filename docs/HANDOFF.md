# Gnosis → Suite / Foundation — Issue Handoff

This is the issue-handoff document: gaps discovered while building Gnosis that
are owned by another project (the Auspicion Suite top-level repo, the
provident-ssr foundation, or another suite tool). Gnosis never patches another
project's source.

## OPEN handoff items

| Item | Owner | Detail |
| --- | --- | --- |
| **Spec §4.4.3 publish-gate crosslink ambiguity** | Auspicion Suite top-level `docs/specs/gnosis.md` | §4.4.3's publish-gate text names only "any `BROKEN` link" and "any `STALE` embed"; it does **not** enumerate `crosslink`, unlike §4.4.5's delete gate ("link/embed/crosslink") and §4.2.2 ("crosslinks resolved the same way as link/embed"). Gnosis's store + tests now **gate crosslink `Broken`/`Stale` on publish** (the §4.2.2 reading). Please reconcile the upstream contract to state the crosslink publish-gate behavior explicitly so the contract and the tests agree. |
| **Spec §4.2.7.2 / §4.2.9.1 graph-owned realization** | Auspicion Suite top-level `docs/specs/gnosis.md` | The §4.2 adversarial gate (decision **GRAPH-OWNS-RELATION-AND-MERGE**) realized the triple/merge/resolution model as **graph-owned data**: the `relation` edge carries `relationType`; `merge_facts` persists the merged fact; `resolve_entities` writes a durable alias→canonical mapping. The contract should be **reconciled to match this** (it currently implies edge-property `relationType` and a persisted merged-fact record, and `resolveEntities`'s `ResolutionResult` shape treats it as a plan/descriptor while the behavior is effectful). |
| **Spec FS-13/14/15 want index-not-built errors in hybrid, but the engine degrades legs gracefully (FS-15 design tension)** | Auspicion Suite top-level `docs/specs/gnosis.md` | The BM25 "index" is currently a **live shard scan**, so "index not built" is reached only when the queried wiki has **no documents** — effectively **unreachable on the `ragQuery` surface** (a wiki with docs always has a lexical leg). `LexicalIndexUnavailable` is only surfaced via the `bm25_search` API. The spec's FS-13 (index not built on hybrid) / FS-14 (vector) / FS-15 (BM25) rows want index-not-built errors in hybrid, while §4.5.1 also says legs **degrade gracefully on a failing leg**. Please **reconcile**: pin whether an unbuilt/unavailable lexical index on the hybrid `ragQuery` surface is an error or a degraded-to-empty leg (the engine currently degrades gracefully). Recorded as a spec-reconcile / HANDOFF item rather than forcing an unreachable error on `ragQuery`. |

## Round 1 — 2026-09-09

The §4.1 document store landed; the only cross-project item surfaced is the §4.4.3
crosslink publish-gate contract ambiguity above. The suite `docs/defects.md`
GAP-1/GAP-2 (multi-query, three-way lexical fusion) remain the engine-owning gaps
already re-pointed to Gnosis.
