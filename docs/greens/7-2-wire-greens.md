# §7.2 F2 Engine wire contract — blind greens

Derived **from the documentation only** (`docs/specs/engine-wire-contract.md`
§1–§13 + `docs/specs/7-2-wire-property-register.md`) by the Blind-Test Writer.
Validation: ran fresh throwaway assertions against the built crate's **public
wire surface** (`gnosis::wire`, `gnosis::StoreError`, `gnosis::Envelope`,
`gnosis::DecodeError`, `gnosis::ValidationFailure`, `gnosis::HealthReport`) via a
scratch integration test that was removed after the run. **No behavior was
verified by reading `src/wire/*.rs`, `tests/wire_conformance.rs`, or
`tests/props_wire.rs`.**

Legend: `GREEN` = independently demonstrated by running the crate's public API
(assertion executed); `RED` = contract-described behavior does **not** match what
the crate actually exhibits (a genuine contract-vs-impl discrepancy requiring a
trip back); `NOT VERIFIED` = documented but could not be exercised from the docs
alone.

## §5 — `StoreError::wire_code` (exhaustive 21-variant code map)

| Documented behavior | Expected observable result | Verifying run | Status |
| --- | --- | --- | --- |
| Full 21-variant one-to-one, stable, unique, non-empty code strings (§5 table) | every constructible variant's `.wire_code()` equals the pinned string; pairwise distinct; non-empty; ascii | scratch test: constructed all 21 variants via public constructors, compared each `.wire_code()` against the §5 table, asserted pairwise uniqueness + non-empty + ascii | GREEN |
| `ValidationError(m)` carries its message; wire code stays `"validation_error"` regardless of `m` | two different `m` → same `"validation_error"` code; `m` surfaces only in the wire `"message"` field | scratch test: `.wire_code()` for `ValidationError("empty query")` and `ValidationError("x")` both `"validation_error"`; `encode_chunk(Error(..))` payload `"message"` == the carried `m` | GREEN |
| `wire_code` is pure/total over the closed enum, never panics/empty | every variant returns non-empty | scratch test: built every variant, asserted `!code.is_empty()` | GREEN |
| Reverse lookup `from_wire(code, msg)` → variant; `"validation_error"`+msg → `ValidationError(m)` | canonical code → that unit variant; `"validation_error"` + message → `ValidationError(m)`; unknown/foreign code → `None` | scratch test: `from_wire` over several canonical codes; `from_wire("validation_error", Some("m"))` → `ValidationError("m")`; `from_wire("bogus", None)` → `None` | GREEN |
| `Conflicts` FS-4 pinned `"conflict"` (§12 V-7 samples) | `DocumentNotFound→"not_found"`, `ConflictError→"conflict"`, `EngineUnavailable→"engine_unavailable"` | scratch test: sampled these three | GREEN |

## §4.1 / §10 — Envelope

| Documented behavior | Expected observable result | Verifying run | Status |
| --- | --- | --- | --- |
| `Envelope { schema_version, id_format, payload }`; snake_case Rust fields, camelCase wire keys | `serde_json::to_string` / `to_json` emits `schemaVersion`/`idFormat` (not snake_case) | scratch test: built `Envelope` with `with_payload`, asserted `to_json` string contains `"schemaVersion"`/`"idFormat"` and not `"schema_version"`/`"id_format"` | GREEN |
| `to_json`/`from_json` round-trip preserves all three fields | `from_json(&to_json(&env)) == env` (payload deep-equal) | scratch test: round-tripped encode_chunk/encode_result/encode_error envelopes | GREEN |
| `CURRENT_SCHEMA_VERSION == 1`, `ID_FORMAT_OPAQUE_STRING_V1 == "opaque-string-v1"`, `current_schema_version() == 1` | constants hold | scratch test: asserted constants + fn | GREEN |
| `with_payload` → schema_version=1, id_format="opaque-string-v1" | envelope defaults correct | scratch test: asserted fields | GREEN |
| `from_json` → `InvalidJson` on parse error; `InvalidEnvelope` on missing/wrong-typed top-level field | malformed JSON → `InvalidJson`; missing `payload` → `InvalidEnvelope` | scratch test: fed `"not json"`, and a JSON lacking `payload` | GREEN |
| `from_json` does **not** validate schema_version/id_format (transport does) | `from_json` accepts a foreign `schema_version`/`id_format`; surfaced later at decode | scratch test: `from_json` over an envelope with `schemaVersion:99` → `Ok`; then decode-path check below | GREEN |

## §6 — codecs (`encode_chunk`/`decode_chunk`, `encode_error`/`decode_error`, `encode_result`/`decode_result`)

| Documented behavior | Expected observable result | Verifying run | Status |
| --- | --- | --- | --- |
| `decode_chunk(&encode_chunk(c)) == c` for all three `RagChunk` variants | round-trip identity element-wise, incl. `Result` body + `Error(ValidationError(m))` preserving `m` | scratch test: round-tripped `Done`, an `Error` over representative variants, and a full `Result` | GREEN |
| `decode_error(&encode_error(e)) == e` for all 21 variants | every variant (incl. `ValidationError(m)` preserving `m`) round-trips | scratch test: round-tripped all 21 variants | GREEN |
| `decode_result(&encode_result(r)) == r` for a well-formed `RagResult` | query/results/citations/trace/blocked_by preserved | scratch test: built a well-formed flat `RagResult`, encode/decode, element-wise `eq` | GREEN |
| `encode_chunk(&Done)` → golden V-1 bytes | `{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"type":"done"}}` | scratch test: asserted exact string | GREEN |
| `encode_chunk(&Error(ConflictError))` → golden V-2 bytes | payload `{"type":"error","code":"conflict","message":"optimistic-concurrency conflict: stale base revision"}` | scratch test: asserted exact string | GREEN |
| `encode_chunk(&Error(ValidationError("empty query")))` → golden V-3 | payload `{"type":"error","code":"validation_error","message":"empty query"}` | scratch test: asserted exact string | GREEN |
| `encode_error(&WikiNotFound)` → golden V-4 | `{"schemaVersion":1,... ,"payload":{"code":"wiki_not_found","message":"wiki not found"}}` | scratch test: asserted exact string | GREEN |
| `encode_result(r_minimal)` → golden V-5 exact bytes | snake_case body keys, PascalCase enums, `{"Flat":{...}}` external-tagged trace, id newtypes bare, `null` `parent`/`stale`/`blocked_by`, citations `[["d1","n1"]]` | scratch test: asserted exact string | GREEN |
| `decode_error` fail-state: unknown code → `Err(UnknownCode)` | non-canonical code → `UnknownCode` | scratch test: envelope with `"code":"bogus"`, `decode_error` → `Err(UnknownCode)` | GREEN |
| `decode_error` fail-state: bad shape → `Err(InvalidJson)` | `{code,message}` wrong types/non-object → `InvalidJson` | scratch test: envelope with non-object payload → `Err(InvalidJson)` | GREEN |

## §7 — decode-then-validate (`decode_rag_result`, `validate_rag_result`, `decode_chunk_payload`, `outcome_of`)

| Documented behavior | Expected observable result | Verifying run | Status |
| --- | --- | --- | --- |
| well-formed `RagResult` body → `Ok(r)` (trace present, engine `"gnosis"`) | decode succeeds | scratch test: decoded the V-5 body via `decode_rag_result` | GREEN |
| body missing `trace` → `Err(MissingTrace)` → `outcome_of` → `Error(TraceUnavailable)` (V-9) | missing-trace decode fails with `MissingTrace`; outcome chunk is `Error(TraceUnavailable)` | scratch test: built JSON without `trace`, `decode_rag_result` → `Err(MissingTrace)`, `outcome_of` → `Error(TraceUnavailable)` | GREEN |
| structurally malformed body → `Err(InvalidJson)` → `outcome_of` → `Error(EngineError)` (V-9) | a `null`/non-object body, or a wrong-typed body **with a `trace` key**, or `trace:null`, or an unknown/invalid trace variant → `InvalidJson` → `EngineError` outcome | scratch test: `null` → `InvalidJson`; `{"query":123,"trace":{…valid…}}` → `InvalidJson`; `trace:null` → `InvalidJson`; `trace:{"Bogus":{}}` → `InvalidJson` — all mapped via `outcome_of` → `EngineError`. **BUT a body that is wrong-typed and trace-less (`{"query":123}`) returns `MissingTrace` → `EngineError`'s sibling `TraceUnavailable` (contract's §7/V-9 "wrong field type → InvalidJson" does **not** hold for that joint case) — see the RED note below** | RED (partial) |
| `validate_rag_result` on valid result → `Ok(())` | passes | scratch test: validated the well-formed result | GREEN |
| `validate_rag_result`: `engine != "gnosis"` → `WrongEngine` | fail-state surfaced | scratch test: `RagResult` with `engine:"other"`, validator → `WrongEngine` | GREEN |
| `validate_rag_result`: `blocked_by: Some` + non-graph trace → `BlockedByWithoutGraphTrace` | fail-state surfaced; and `decode_result`/`decode_chunk` → `Err(ValidationFailed(_))` | scratch test: flat trace + `blocked_by: Some`, validator → `BlockedByWithoutGraphTrace`; encode → decode_result → `Err(ValidationFailed)`; `outcome_of` → `Error(EngineError)` | GREEN |
| `MissingTrace` alone → `TraceUnavailable` outcome; every other `DecodeError`/`ValidationFailed` → `EngineError` outcome | `outcome_of` maps per the §7 table | scratch test: mapped representative failures | GREEN |
| encoder never emits a body the decoder+validator reject (P-TP-1) | `validate_rag_result(&decode_rag_result(&encode_result(r).payload)) == Ok` for well-formed `r` | scratch test: for a well-formed `r`, decode+validate succeeds; `decode_result(&encode_result(r)).is_ok()` | GREEN |
| unknown `schema_version:99` → `UnsupportedSchemaVersion(99)` → `EngineError` outcome | decode rejects foreign version | scratch test: envelope with `schemaVersion:99`, `decode_chunk` → `Err(UnsupportedSchemaVersion)`; `outcome_of` → `Error(EngineError)` | GREEN |
| unknown `id_format:"uuid-v4"` → `UnknownIdFormat` → `EngineError` outcome | decode rejects foreign id-format | scratch test: envelope with `idFormat:"uuid-v4"`, `decode_chunk` → `Err(UnknownIdFormat)`; `outcome_of` → `Error(EngineError)` | GREEN |
| unknown chunk `"type"` → `UnknownType` → `EngineError` outcome | decode rejects a foreign discriminator | scratch test: envelope payload `{"type":"bogus"}` → `UnknownType` | GREEN |

## §8 — SSE (`encode_event` / `decode_event` / `event_type`)

| Documented behavior | Expected observable result | Verifying run | Status |
| --- | --- | --- | --- |
| `decode_event(&encode_event(c)) == c` for all three variant classes | SSE round-trip identity | scratch test: round-tripped `Done`, `Error`, `Result` events | GREEN |
| single-event framing §4.4: `event: <type>\ndata: <json>\n\n` | one `event:`/`data:` line, LF, trailing blank line | scratch test: asserted frame starts `event: `, has exactly one `data:` line, ends `\n\n`, no `\r` | GREEN |
| `<type>` == data `"type"`; mismatch → `Err(EventTypeMismatch)` | wrong event line → `EventTypeMismatch` decoded | scratch test: hand-built frame `event: done` + `data: {"type":"error",...}` → `EventTypeMismatch` | GREEN |
| `Done` frame V-6 exact | `event: done\ndata: {"type":"done"}\n\n` | scratch test: asserted exact string | GREEN |
| `Error(ValidationError("empty query"))` frame V-6 exact | `event: error\ndata: {"type":"error","code":"validation_error","message":"empty query"}\n\n` | scratch test: asserted exact string | GREEN |
| `Result(r)` frame V-6 = `event: result` + `data: {"type":"result","result":{…}}` | result event carries the full body | scratch test: asserted prefix `event: result` + data `"type":"result"` and body round-trips | GREEN |
| malformed frame (no `event:`/`data:` lines or trailing non-blank content) → `Err(InvalidEnvelope)` | invalid frame rejected | scratch test: frame `garbage\n` → `InvalidEnvelope`; trailing content frame → `InvalidEnvelope` | GREEN |
| unparseable `data:` value → `Err(InvalidJson)` | bad data JSON → `InvalidJson` | scratch test: frame with non-JSON data → `InvalidJson` | GREEN |

## §9 — Health (`health` / `HealthReport`)

| Documented behavior | Expected observable result | Verifying run | Status |
| --- | --- | --- | --- |
| `health(&EngineStatus) -> HealthReport`; pure deterministic function (P-SM-3) | equal status → equal report; report mirrors state/version/subsystems/last_error | scratch test: same status twice → equal reports; `last_error.is_some() == input.is_some()` | GREEN |
| Ready engine → V-8 golden JSON | `{"schemaVersion":1,"idFormat":"opaque-string-v1","state":"Ready","version":"…","subsystems":{"store":true,...all-true...},"lastError":null}` | scratch test: built all-true `EngineStatus{Ready}` and asserted exact serialized string | GREEN |
| Degraded engine → V-8 golden JSON | `state:"Degraded"`, `subsystems.embedding:false`, `lastError:"a non-core subsystem (embedding/reranker) is unavailable"` | scratch test: built `EngineStatus{Degraded, embedding:false, last_error:Some(..)}` and asserted exact serialized string | GREEN |
| state emits real PascalCase `EngineState` value | `Ready`/`Starting`/`Degraded`/`Unavailable` | scratch test: swept all four states, asserted `state` serialization | GREEN |
| report maps `EngineStatus` field-by-field (does not invent a `last_error`) | `last_error` `Some` exactly when input's is `Some` | scratch test: `last_error:None` → report `lastError:null`; `Some` → mirrored | GREEN |
| `subsystems` inner flags keep single-word names, unchanged by top-level rename | `store`/`graph`/`lexical`/`vector`/`embedding`/`reranker` | scratch test: asserted subsystem keys in the serialized report | GREEN |

## Id-format seam / id newtypes (§4.1 UUID deferral, V-5)

| Documented behavior | Expected observable result | Verifying run | Status |
| --- | --- | --- | --- |
| ids (`DocumentId`/`NodeId`/`WikiId`) cross the wire as opaque strings | `document_id:"d1"`, `node_id:"n1"` in the result JSON; never parsed/validated as UUID | scratch test: `encode_result` on the minimal result → `document_id":"d1"`, `node_id":"n1"` (V-5 bytes) | GREEN |
| citations serialize as nested string pairs `[["d1","n1"]]` | citations shape in V-5 | scratch test: asserted `"citations":[[...d1...n1...]]` in serialized body | GREEN |
| `"opaque-string-v1"` is the only implemented `id_format`; RFC-4122 is the documented, not-implemented extensibility seam | `UnknownIdFormat` for a future value (e.g. `"uuid-v4"`) | scratch test: decode of `idFormat:"uuid-v4"` → `UnknownIdFormat` (row also in §7) | GREEN |

## Cross-cutting TestWriter states (§13)

| Documented behavior | Expected observable result | Verifying run | Status |
| --- | --- | --- | --- |
| wire is transparent to wiki scope — a `RagResult` referencing a foreign wiki round-trips byte-for-byte | a cross-wiki `RagResult` encodes/decodes identically | scratch test: built a `RagResult` with `engine:"gnosis"` + opaque ids, round-tripped (covered by §6 result round-trip) | GREEN |
| empty `payload` handled per context | `InvalidEnvelope` for a result/done chunk with missing payload; `done` only `Ok` if payload is exactly `{"type":"done"}` | scratch test: `null` payload → `InvalidEnvelope`; exact `{"type":"done"}` → `Ok(Done)`; `{"type":"done","x":1}` → `InvalidEnvelope`; `{"type":"result"}` (missing body) → error. **Detail:** an empty-object `{}` payload → `UnknownType` (not `InvalidEnvelope` as the §13 "Empty payload" bullet literally states) — same `EngineError` outcome, so net wire behavior matches | GREEN (note) |

## RED — contract/impl discrepancy (must go back)

**1. `decode_rag_result` precedence when a body is *both* malformed and trace-less.** The
contract (§7, §13, V-9) pins: "structurally malformed (wrong field types …) → `InvalidJson` →
`EngineError` outcome" and "structurally **well-formed** but trace absent → `MissingTrace` →
`TraceUnavailable`". Observed against the live crate: for the input `{"query":123}` — a body
whose `query` field is the **wrong type** (so not "well-formed") and which also carries **no
`trace` key** — `decode_rag_result` returns `Err(MissingTrace)` and `outcome_of` therefore
maps it to `Error(StoreError::TraceUnavailable)`, **not** the `InvalidJson`→`EngineError` the
contract's "wrong field type → InvalidJson → EngineError" claims. The trace-**presence** key
check runs **before** field-type deserialization, so `MissingTrace` wins over malformedness.

- Concrete observed route: `{"query":123}` → `MissingTrace` → `TraceUnavailable`.
- The contract-conforming route still fires for malformed-but-present-trace inputs:
  `{"query":123,"trace":{…valid Flat…}}` → `InvalidJson`; `{"query":"q","trace":null}` →
  `InvalidJson`; `trace:{"Bogus":{}}` → `InvalidJson`; `null` body → `InvalidJson` — all map
  to `EngineError`.
- **Impact:** the wire **code** differs (`trace_unavailable` vs `engine_error`). Both map to
  HTTP **502** (§11), so the shell-rendered status is identical; the divergence is observable
  only at the wire-code/outcome layer. Either the contract's §7/§13/V-9 wording must be
  tightened to "a wrong-typed field → `InvalidJson` **only when a `trace` key is present**;
  otherwise `MissingTrace`", or the impl must defer `MissingTrace` until after structural
  well-formedness is confirmed. **This is the one candidate that must go back** for a
  contract/impl reconciliation decision.

**2. Empty-object `{}` payload → `UnknownType`** (not the §13 bullet's `InvalidEnvelope`).
Net wire outcome is the same `EngineError`, so I record it as GREEN-with-note above rather
than a separate RED, but it is the same class of contract-wording slack and should be folded
into the reconciliation.

## NOT VERIFIED

1. **`WireCodecError` as a reachable fail-path** (§6): the contract itself labels it
   "a defensive enum, not a tested fail-path" reachable only on a non-object
   payload map the codecs never produce. Not independently exercised.
2. **`decode_chunk`'s `UnknownCode`/`UnsupportedSchemaVersion`/`UnknownIdFormat`
   routes exercised via `decode_chunk` rather than the equivalent `decode_error`/
   `decode_result` paths** — covered via `decode_chunk`/`decode_result` for the
   outcome mapping. The distinct codec-specific combination is documented
   (same `DecodeError` set), so this is a coverage nuance, not a gap.
3. **Byte-for-byte golden SSE result event (V-6 third row full body)** — the result
   body is asserted through result round-trip; the exact concatenated `event:
   result\n` + full-body `data:` line is asserted for its frame prefix and body,
   not the single literal V-6 string (V-6 shows `data: {…}` with the §12 V-5 body,
   which itself is asserted verbatim under §6).
