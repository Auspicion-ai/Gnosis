# F2 — Engine transport/API reconciliation — PROPOSAL-REVIEW record

- **Unit:** §7.2 F2 — engine transport/API reconciliation (the only OPEN unit in
  `docs/next-steps.md`).
- **Gate:** proposal-review gate (validity ∥ critique → architecture →
  change-analysis).
- **Date:** 2026-09-09.
- **Verdict:** **PASS — scoped per Architecture A1** (mechanism-agnostic wire
  contract + encoding layer), contingent on the user's go-ahead. Not a
  GATE-DECISION: F2 ships real tested artifacts.
- **Status:** proposal approved by review; code/contract delegation awaits the
  user's go-ahead.

## What F2 is

F2 pins the concrete wire serialization for the Astrographer-shell → Gnosis-engine
proxy seam (§5.1), which proxies the `RagStore` persistence interface (§4.1.5) +
the query/stream/engine-status retrieval API (§4.6.1). The logical API already
exists and is fully implemented + adversarially hardened (295 tests, trio green).
§7.2 leaves the transport mechanism ("HTTP/REST + SSE, or a native IPC channel")
as a Gnosis-repo decision and names two benefits: making the proxy seam testable
end-to-end, and confirming the `EngineUnavailable`/`EngineError` split matches the
real transport's failure modes.

## The review findings (all read-only passes)

**Validity (VALID-WITH-CONTENTIONS).** F2 is correctly the sole open unit; the
§4.6.1/§5.1 seam is the right isolated target. The crate surface is serde-ready
for all body types except two (see below). Contended: the wire format,
the deliverable depth (pin-and-prove vs full server), server ownership, the
`StoreError`/`RagChunk` wire encoding, the HTTP status map (FS-4 mandates
`ConflictError` = HTTP 409), the `EngineUnavailable`/`EngineError` split, the
shell-side D2/boot/lifecycle, and cross-repo (Astrographer) coordination.

**Critique (12 contentions).** Key: (1) the HTTP-vs-native-IPC *mechanism* may
not be F2's to decide (§7.2 lists both; §5.2/A4 precedent — "HTTP push transport …
shell-side; serialization engine-side"); (2) the UUID-v4 id decision (deferred in
`docs/pending.md` to "when the F2 IPC seam lands") must be resolved via a
`schemaVersion`/`idFormat` seam rather than silently freezing monotonic ids;
(3) `EngineError` is a wire-tier artfact with no internal producer — only
decoder-side negative-testable; (4) `StoreError`/`RagChunk` have **no** `Serialize`
derivation — encoding them is the first pinned deliverable; (5) `rag_stream` is
single-shot today (`[Result, Done]`/`[Error, Done]`) — the SSE schema must be
honest (single-event) or require a separate incremental engine unit;
(6) no boot→READY model; (7) wiring the full 34-method `RagStore` CRUD is
over-scope (scope to the §4.6.1 retrieval trio + health); (8) connection/cancel
for the streamed path + a framework choice; (9) decode-then-validate so
`TraceUnavailable`/`EngineError` become real decoder outcomes; (10) loopback-only
bind + auth/TLS shell-owned; (11) D2 engine-absent is shell-side (pin only the
reporting); (12) a uniform `StoreError`→HTTP status table.

**Architecture (decisive).** Recommends **A1**: a **mechanism-agnostic wire
contract + encoding layer** in a new `src/wire/` module with a
`tests/wire_conformance.rs` suite — **zero new dependencies** (serde/serde_json/
tokio/futures suffice); the retrieval trio + health as the wire surface; a
versioned `schemaVersion`/`idFormat` envelope for the UUID-v4 deferral; **single-
event** SSE; decode-then-validate for `EngineError`/`TraceUnavailable`; the
existing `set_engine_state(Ready)` fixture (no boot prerequisite); loopback-only-
by-contract with auth/TLS shell-owned.

## The F2 deliverable (scoped, delegable)

**Built + tested in-repo (`src/wire/` + `tests/wire_conformance.rs` + `tests/props_wire.rs`):**
- `error.rs` — `StoreError::wire_code() -> &'static str` (stable code per FS variant).
- `codecs.rs` — JSON codecs for `RagChunk`, `StoreError`, `RagResult` (the two
  non-`Serialize` types encoded in the wire layer, not by mutating the frozen
  store enum).
- `decode.rs` — decode-then-validate (`decode_rag_result` + `validate_rag_result`):
  malformed body → `EngineError`; missing `trace` → `TraceUnavailable`.
- `sse.rs` — single-event SSE encode/decode (`type: result|done|error`).
- `status.rs` — schemaVersion-aware health serialization of the already-`Serialize`
  `EngineStatus`.
- `envelope.rs` — `{schemaVersion, idFormat, payload}`.
- `tests/wire_conformance.rs` + `tests/props_wire.rs` (TestWriter's red set).
- **PBT gate:** a wire property register `docs/specs/7-2-wire-property-register.md`
  (≤8 invariant-only rows) + executed `props_wire.rs` + read-only audit — per
  decision PBT-GATE-MANDATORY (codec round-trip is a natural property).

**Recorded as Gnosis decisions in a new spec `docs/specs/engine-wire-contract.md`**
(the contract the Astrographer shell implements identically): the versioned wire
schema + `schemaVersion`, the SSE event-type table, the full `StoreError`→wire-code
table, and the **documented** HTTP-status map (incl. `ConflictError` = HTTP 409
per FS-4).

**Deferred / OUT of F2 (belted to a later shell-integration unit):** the
HTTP-vs-IPC transport decision + server host (and its axum/hyper/tower deps), the
shell-side SSE client, bind/auth/TLS + loopback enforcement, full `RagStore` CRUD
routing, the engine boot→READY lifecycle + its end-to-end transport test, HTTP-status
rendering, D2 engine-absent shell behavior, the RFC-4122 id-scheme change, and
incremental multi-chunk `rag_stream` streaming (a separate engine unit).

## What a later shell-integration unit must own

1. The HTTP-over-native-IPC decision and the server host.
2. The shell-side SSE client consuming the pinned event schema.
3. Bind-loopback + auth/TLS policy (recorded shell-owned).
4. Full `RagStore` persistence CRUD routing (deferred).
5. The engine boot→READY lifecycle + its end-to-end transport test.
6. Rendering the documented HTTP-status map (FS-4 → 409, etc.).
7. D2 engine-absent degrade behavior at the shell.

## Residual risks / open items to be told to the user before approval

- **No running server from F2.** Mechanism deliberately deferred; F2 delivers the
  wire contract + codecs + conformance only.
- **`EngineError` verified only at the decode layer** (malformed-body fixtures),
  not against a live engine (the engine can't produce one in-repo); its HTTP-status
  rendering lands in the shell unit.
- **UUID-v4 deferred** to the suite/HANDOFF via the `idFormat` seam; ids stay
  opaque strings. Requiring RFC-4122 now is a different, expanding unit (repins
  suites + a `uuid` dep).
- **No incremental streaming.** SSE is single-event (honest to single-shot);
  multi-chunk streaming needs a separate `rag_stream` engine unit.
- **`StoreError`/`RagChunk` become encodable** — a real change near the frozen
  §4.1/§4.5 types, mitigated by keeping encoding in `src/wire/` rather than
  mutating the store enum's derive surface.

## Next steps (after user go-ahead, gate order)

1. **SpecWriter** authors `docs/specs/engine-wire-contract.md` (compile-horizon
   review format) + the wire property register;
2. reviewer loop on that contract → empty;
3. **TestWriter** writes `wire_conformance.rs` + `props_wire.rs` from the contract
   alone → red set;
4. **Implementer** lands least code (`src/wire/`);
5. adversarial → blind-greens → trio green.
