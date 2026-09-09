# Astral — wiki publishing (via the shell's push)

**Edge:** Gnosis → Astral via the shell's push (`docs/specs/gnosis.md` §5.2).

## Nature

Astral is the locally-runnable webhost that receives published documentation as
Provident graphs. Gnosis publishes through the Astrographer shell's push.

## Mechanism

The shell serializes Gnosis's published documents to the canonical
`provident-graph/1` schema and pushes them to Astral (decision GRAPH-PUSH-FORMAT,
idempotent via `contentHash`). Gnosis is the graph-data owner; the shell owns the
push transport + push-status UI.

## Value

Gnosis's published documents become Astral-hosted wiki pages automatically —
"simple and automatic wiki hosting and publishing" (D3).

## Contract refs

`docs/specs/gnosis.md` §5.2, §4.4 (consistency), decision GRAPH-PUSH-FORMAT.

## Fail-states

`PushRejected` (Astral reachable but rejects: non-2xx, `SchemaMismatch`, auth
failure); `AstralUnavailable` (Astral unreachable). `UnresolvedReference` blocks
push when the document has `BROKEN`/`STALE` references (§4.4.3).
