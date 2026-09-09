# Solomon — cross-instance search

**Edge:** Solomon ↔ Gnosis (`docs/specs/gnosis.md` §5.4).

## Nature

Solomon is the **decentralized peer-network search** algorithm that links and
searches across suite instances (Astrographer, Zodiac, Astral, and now Gnosis-
backed instances).

## Mechanism

A Gnosis-backed instance joins the Solomon peer network; `solomon_search` fans a
query out to peers and aggregates results (`docs/specs/solomon.md` §4.2). Gnosis
results carry result provenance (`peerId`/`instanceType`) for traceability.

## Value

Cross-instance access to Gnosis's knowledge graph / document store — a Gnosison
instance's data becomes part of a searchable whole across instances (D3).

## Contract refs

`docs/specs/gnosis.md` §5.4, `docs/specs/solomon.md` §4.2/§4.5.1.

## Fail-states

Peer auth failure (GUI-only, security carve-out at the shell); peer unreachable;
no peers joined (search returns empty, not an error). The cross-peer ordering gap
is a Phase-1 aggregation concern (`solomon.md` §7.4).
