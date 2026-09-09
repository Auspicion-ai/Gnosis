# Zodiac — a second RAG data source

**Edge:** Zodiac → Gnosis (`docs/specs/gnosis.md` §5.3).

## Nature

Zodiac is the **backend/remote graphical RAG companion** that supplies **non-local
data** (dependency packages, current web data) that a purely local Gnosis cannot
obtain. It is a second RAG data source alongside Gnosis's own local store.

## Mechanism

Gnosis's `ragQuery`/`ragStream` can route a query to Zodiac (the `source:
'zodiac'` union member). Zodiac's query reply (`mode: graph|vector|hybrid`,
`sourceCrawlId` provenance) is a second leg the engine can consume
(`docs/specs/zodiac.md` §4.3).

## Value

Gnosis gains external, current data without a local copy; the query surface
fuses local + Zodiac results (D3).

## Contract refs

`docs/research/astrographer-use-cases-to-pin.md` B1.9 (adaptive RAG routing to
Zodiac, PARKED), `docs/pending.md` D3.

## Fail-states

**`ZodiacUnavailable`** (§6 FS-23): a query that would use Zodiac when Zodiac is
unreachable **degrades to local-only results, not an error** — the engine's core
function is unaffected (Zodiac is additive).
