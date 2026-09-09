# §7.2 F2 — Wire unit — LIVE-SCENARIO PENDING BATTERY

- **Unit:** §7.2 F2 — engine transport/API reconciliation (wire contract + codecs).
- **Date:** 2026-09-09.
- **Author-role:** live-scenario-runner.
- **Status:** **PENDING** (parked — **not** a gate failure).
- **Source set transcribed:** `docs/greens/7-2-wire-greens.md` (50 GREEN / 1 RED
  resolved-as-doc-fix / 3 NOT-VERIFIED) → the live actions below.

## 1. Why this is a PENDING battery

Gnosis is a **headless backend** (AGENTS.md: it has **no MCP surface and no GUI
surface**; §4.6.2 of `docs/specs/gnosis.md`; D4 parity applies at the Astrographer
shell, not at Gnosis). Its only API surface is the `ragQuery` / `ragStream` /
`getEngineStatus` retrieval trio plus the just-landed wire contract.

The F2 unit itself deliberately ships **no server, no client, no live process
transport** (`docs/specs/engine-wire-contract.md` §2 "NOT in scope": a hosted
HTTP or native-IPC server, a shell-side SSE client, bind/auth/TLS, loopback
enforcement, boot→READY lifecycle — all owned by a **later shell-integration
unit**, `7-2-f2-review.md` §"Deferred / OUT of F2"). F2 delivers only the
**mechanism-agnostic codecs + validation + single-event SSE framing + health
serialization + versioned envelope**, all exercised **in-repo** by
`tests/wire_conformance.rs` + `tests/props_wire.rs` (360 tests green).

Consequently **no live-scenario can currently be executed**: there is no running
engine process the runner can proxy over a wire, no SSE client to subscribe with,
no HTTP/native-IPC endpoint to issue `ragQuery`/`getEngineStatus` over, and no
MCP/GUI surface to drive. Every live-scenario derivable from the greens set is
therefore **parked** here for a later iteration of this runner. **Parked
scenarios are NOT a failure** — the gate outcome for F2 is PENDING, with the
in-repo blind-greens (360 green) as the current evidence base.

### REVISIT CONDITION

This battery resumes (the gate may be re-run live) when the **later
shell-integration unit** lands:

1. the **HTTP-over-native-IPC decision + server host** (axum/hyper/tower deps) that
   exposes the retrieval trio over a real process transport, and
2. a **shell-side SSE client** consuming the §4.4 single-event schema,
giving the live-scenario runner a **live process transport + a real consumer**
to run against.

Until then this battery records the exact scenarios to execute at that point. No
engine code (`src/`) or test code (`tests/`) is modified by this battery.

## 2. The live scenarios (a later runner executes these against the live transport)

Each row: the scenario (faithful transcription of the corresponding greens row —
**no new behavior invented**), the exact action a live consumer performs, and the
EXPECTED observable (the exact wire bytes/event/outcome pinned by the contract
`docs/specs/engine-wire-contract.md` §12 golden vectors + `docs/greens/7-2-wire-greens.md`).

### 2.1 Wire-code map on the live wire (`error.rs` · green §5 rows)

| Scenario | Live action | Expected observable |
| --- | --- | --- |
| S-L1 — `ragQuery` returns a `"not_found"` error | live consumer issues a `ragQuery` for a nonexistent document/wiki so the engine surfaces `DocumentNotFound` | wire chunk/error envelope payload `{"type":"error","code":"not_found","message":"…"}`; HTTP status → 404 when the shell renders §11 |
| S-L2 — optimistic-concurrency conflict surfaces as `"conflict"` | consumer drives a stale-base-revision mutation so the engine emits `ConflictError` | payload `{"type":"error","code":"conflict","message":"optimistic-concurrency conflict: stale base revision"}` (V-2); HTTP → **409** (FS-4 mandated) |
| S-L3 — validation error carries its message on the wire | consumer issues a query the engine rejects with `ValidationError("empty query")` | code stays `"validation_error"`; the carried message surfaces in `"message":"empty query"` (V-3), not in the code |
| S-L4 — every reachable FS variant maps to one stable code live | (where reachable via live drive) assert each produced error's `code` equals its §5 pinned string, distinct/non-empty | wire `code` ∈ the §5 21-row table; a foreign/unknown code never appears from the engine |

### 2.2 Envelope on the wire (`envelope.rs` · green §4.1/§10 rows)

| Scenario | Live action | Expected observable |
| --- | --- | --- |
| S-L5 — every payload arrives inside the versioned envelope | consumer inspects each receive for `schemaVersion`/`idFormat` (camelCase) | top-level `{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":…}` (V-1..V-5); `schemaVersion`/`idFormat` — not snake_case |
| S-L6 — id-format seam: only `"opaque-string-v1"` accepted | consumer (or shell) hand-sends an envelope with `idFormat:"uuid-v4"` | engine rejects → `UnknownIdFormat` → `engine_error` outcome (RFC-4122 is the documented, not-implemented seam) |
| S-L7 — schema-version gate on the live decode | consumer hand-sends `schemaVersion:99` | engine rejects → `UnsupportedSchemaVersion(99)` → `engine_error` outcome |

### 2.3 `ragStream` SSE live (`sse.rs` · green §6/§8 rows)

| Scenario | Live action | Expected observable |
| --- | --- | --- |
| S-L8 — `event: result` then `event: done` | shell SSE client subscribes to `ragStream` for a well-formed non-blocked query | receives a `result` event carrying the full `RagResult` body, then a `done` event; frame shape `event: <type>\ndata: <single-line json>\n\n` (LF, terminal blank line) |
| S-L9 — `event: error` then `event: done` | client triggers a query the engine fails (e.g. `ValidationError`) | `event: error` + data `{"type":"error","code":"validation_error","message":"empty query"}` then `event: done` (V-6) |
| S-L10 — frame `event:` type equals data `"type"` | client (as the data producer in a loopback test) or consumer asserts event/data consistency on live frames | a mismatch frame (`event: done` + data `{"type":"error",…}`) decodes as `EventTypeMismatch` — never silently accepted |
| S-L11 — malformed `data:` line surfaces as `event: error` | client receives/hand-sends a frame with unparseable or malformed `data:` JSON | engine/consumer surfaces it as `event: error` with `engine_error` code (not a crash, not a silent drop) |
| S-L12 — single-event honesty over SSE | consumer verifies the live stream emits at most one data-carrying event then a terminal | a normal `ragStream` yields exactly `[result|error, done]`; no multi-chunk/incremental framing/continuation lines |

### 2.4 `ragQuery` result envelope over the wire (`codecs.rs`/`decode.rs`)

| Scenario | Live action | Expected observable |
| --- | --- | --- |
| S-L13 — live consumer issues `ragQuery` and receives a wire-conformant `RagResult` envelope | consumer calls `ragQuery` for a well-formed flat query | envelope payload is the exact V-5 body: snake_case keys, PascalCase enums, externally-tagged `trace` `{"Flat":{…}}`, id newtypes bare, `null` `parent`/`stale`/`blocked_by`, citations `[["d1","n1"]]`, `engine:"gnosis"` |
| S-L14 — decode-then-validate live: missing `trace` surfaces as `"trace_unavailable"` | consumer drives a query whose body lacks `trace` (or a hand-built body) | `MissingTrace` → `"trace_unavailable"` outcome (FS-10); HTTP → 502 |
| S-L15 — malformed-body live surface as `"engine_error"` | consumer hands a structurally malformed body **with a `trace` key** / `trace:null` / invalid `trace` variant | `InvalidJson` → `"engine_error"` outcome (FS-9); HTTP → 502 |
| S-L16 — pinned precedence: malformed-**and**-traceless body → `"trace_unavailable"` | consumer hands `{"query":123}` (wrong-typed and no `trace` key) | `MissingTrace` → `"trace_unavailable"` (NOT `engine_error`) — the tightened §7/V-9 "Precedence (pinned)" behavior; net HTTP 502 identical |
| S-L17 — validation invariant live: foreign engine / blocked-by-without-graph-trace | consumer drives or hand-builds a `RagResult` with `engine:"other"`, or flat-trace + `blocked_by:Some` | `WrongEngine` / `BlockedByWithoutGraphTrace` → `"engine_error"` outcome |
| S-L18 — empty/`{}` payload handled per context | consumer receives an empty `{}` payload / a done chunk with extra keys | exact `{"type":"done"}` → `Ok(Done)`; `{}` → `UnknownType` → `engine_error`; extra-key done → `InvalidEnvelope`/`UnknownType` → `engine_error` (net behavior per §13 note) |

### 2.5 Health over the wire (`status.rs` · green §9 rows)

| Scenario | Live action | Expected observable |
| --- | --- | --- |
| S-L19 — Ready engine health | consumer issues `getEngineStatus` on a ready engine | report = V-8 Ready golden: `state:"Ready"`, all-`true` subsystem flags, `lastError:null` |
| S-L20 — Degraded engine health reports `last_error` present | drive the engine Degraded (embedding unavailable, `last_error:Some(..)`) then `getEngineStatus` | `state:"Degraded"`, `subsystems.embedding:false`, `lastError` = the non-core-subsystem message (V-8 Degraded golden) |
| S-L21 — Unavailable engine yields `engine_unavailable` over the wire | `getEngineStatus`/`ragQuery` on an engine in `Unavailable` state | wire error code `"engine_unavailable"` (FS-8); HTTP → 503 |
| S-L22 — report is a faithful projection (never invents `last_error`) | compare `getEngineStatus` output to the underlying `EngineStatus` | `lastError` `Some` exactly when the status had `last_error:Some`; subsystem keys unchanged (`store`/`graph`/`lexical`/`vector`/`embedding`/`reranker`); `state` is real PascalCase `EngineState` |

### 2.6 Id-format seam over the wire (V-5 · green "id newtypes")

| Scenario | Live action | Expected observable |
| --- | --- | --- |
| S-L23 — ids cross the wire as opaque strings | consumer inspects a live `ragQuery` result's ids | `document_id:"d1"`, `node_id:"n1"` as opaque strings; never parsed/validated as UUID |

## 3. Parity re-check, not new coverage

Every scenario above is **already verified in-repo** by `tests/wire_conformance.rs`
+ `tests/props_wire.rs` (360 tests green, blind-greens 50 GREEN / 1 RED
resolved-as-doc-fix / 3 NOT-VERIFIED) at the **codec/decode/encode layer** — the
exact golden bytes, round-trips, and decode-then-validate/SSE/health outcome
mappings above are asserted there today. The pending battery adds **no new
behavior and no new test coverage**. Its entire value is **re-confirming the SAME
behavior end-to-end** over a **real process transport with a real shell-side
consumer** — i.e. a **parity re-check** that the encoder/decoder behavior proven
in-repo is identical over an actual `ragStream`/`ragQuery`/`getEngineStatus` HTTP
or native-IPC round trip, including the live render of the §11 HTTP-status map and
the `Unavailable`/`Degraded` engine states the shell owns.

## 4. How the later runner executes this

Once the shell-integration unit lands the server + a live client, this runner will:

1. **Proxy the engine over the wire** — boot the engine to `Ready`
   (`set_engine_state(Ready)` fixture; the test double), then connect the live
   consumer over the HTTP-over-native-IPC server.
2. **Subscribe to `ragStream`** — exercise S-L8..S-L12, capturing the SSE frames and
   decoding them against §4.4 / the golden vectors.
3. **Issue `ragQuery` / `getEngineStatus` over the live transport** — exercise
   S-L1..S-L7, S-L13..S-L23, driving the engine through the states the scenarios
   require (validation error, conflict, missing-trace, malformed body, Degraded,
   Unavailable, foreign `schema_version`/`id_format`).
4. **Record live pass/fail per scenario** — each row above that passes live
   confirms the in-repo parity; a scenario whose **live** result **contradicts the
   greens/contract** is **a finding** (a real regression or a doc/spec drift — a
   live failure is never a pass). The NOT-VERIFIED items (see below) and any parked
   rows whose transport still does not exist are re-parked, not failed.

### NOT-VERIFIED items (noted as such in the source set; re-check at revisit)

1. `WireCodecError` reachability — the contract labels it a defensive enum with no
   tested fail-path; live, a real transport should also never surface it.
2. `UnknownCode`/`UnsupportedSchemaVersion`/`UnknownIdFormat` routes exercised via
   `decode_chunk` live (a coverage nuance over the codec-specific paths).
3. Byte-for-byte golden SSE `result` event concatenated over the live stream —
   S-L8 asserts the exact `event: result\n` + full-body `data:` line frame (V-6
   third row).

## 5. Authoritative refs

- `docs/specs/engine-wire-contract.md` — the wire contract (codes §5, envelope §4.1/§10, codecs §6, decode-then-validate §7, SSE §4.4/§8, health §9, HTTP map §11, golden vectors §12, valid/fail states §13).
- `docs/greens/7-2-wire-greens.md` — the blind-greens set this battery transcribes.
- `docs/specs/7-2-f2-review.md` — the stated deferral (server/SSE client/process transport = later shell-integration unit; F2 owns wire + codecs only).
- `tests/wire_conformance.rs`, `tests/props_wire.rs` — the in-repo verification layer.

This battery modifies **only** this document. No `src/` or `tests/` change. No commit.
