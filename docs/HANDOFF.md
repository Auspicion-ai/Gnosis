# Gnosis → Suite / Foundation — Issue Handoff

This is the issue-handoff document: gaps discovered while building Gnosis that
are owned by another project (the Auspicion Suite top-level repo, the
provident-ssr foundation, or another suite tool). Gnosis never patches another
project's source.

## OPEN handoff items

| Item | Owner | Detail |
| --- | --- | --- |
| **Spec §4.4.3 publish-gate crosslink ambiguity** | Auspicion Suite top-level `docs/specs/gnosis.md` | §4.4.3's publish-gate text names only "any `BROKEN` link" and "any `STALE` embed"; it does **not** enumerate `crosslink`, unlike §4.4.5's delete gate ("link/embed/crosslink") and §4.2.2 ("crosslinks resolved the same way as link/embed"). Gnosis's store + tests now **gate crosslink `Broken`/`Stale` on publish** (the §4.2.2 reading). Please reconcile the upstream contract to state the crosslink publish-gate behavior explicitly so the contract and the tests agree. |

## Round 1 — 2026-09-09

The §4.1 document store landed; the only cross-project item surfaced is the §4.4.3
crosslink publish-gate contract ambiguity above. The suite `docs/defects.md`
GAP-1/GAP-2 (multi-query, three-way lexical fusion) remain the engine-owning gaps
already re-pointed to Gnosis.
