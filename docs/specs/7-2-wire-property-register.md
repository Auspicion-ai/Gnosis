# §7.2 F2 Wire Contract — Typed Property Register (PBT gate)

- **Unit:** §7.2 F2 wire contract — `src/wire/`. **Spec:** `docs/specs/engine-wire-contract.md` (the F2 compile-horizon behavioral contract) over the §4.6.1 retrieval trio.
- **Scope/API:** the mechanism-agnostic wire codecs + decode-then-validate + single-event SSE + health + envelope in `src/wire/`, re-exported from `src/lib.rs` (`pub mod wire` + flat re-exports `Envelope`, `DecodeError`, `ValidationFailure`, `HealthReport`; `StoreError::wire_code` inherent method). Exercised synchronously — no runtime, no engine instance, no `rag_query` call needed. The codecs operate on in-memory `RagChunk`/`StoreError`/`RagResult`/`EngineStatus` values the generator builds directly.
- **Exercised by:** the F2 conformance suite `tests/wire_conformance.rs` + the executed property layer `tests/props_wire.rs` (TestWriter's red set, RED-first from the contract).
- **Date:** 2026-09-09. **Role:** spec_writer (this register is the F2 PBT-gate artifact **#1** of 3; #2 = executed `props_wire.rs`, #3 = read-only audit).
- **Status:** PBT-gate artifact #1 — the typed property register, invariant-only.

## PBT-gate note

This register is the contract for the **property-based-testing gate** on the F2
wire unit. The TestWriter's executed layer runs under `cargo test` with a
**deterministic pinned seed**, **≤100 generated cases per register row**,
**≤400 total cases** across the unit's whole property layer, **stop-after-5**
(report ≤5 distinct held/broken counterexamples per row), and records each row as
**held** or **broken** together with its `Strategy-id`. The adversarial reviewer
then reads this register with the executed artifacts and performs a **read-only**
PBT audit (per-row over-strength reasoning, generator-coverage check, prose
counterexamples, negative-generator requests); reviewers never run generators.

Authoring rule: `Property-id` = `P-<CLASS>-<N>`, `CLASS ∈ {IM, SM, TP}`, **≤ 8
rows**; invariants are **crisp, universally-quantified, black-box-observable
through the crate's public wire surface only**, and are **TRUE of the current
GREEN implementation** (implied by the contract — they are **not** pending
features). **There are no fail-state rows** (no `§6`/`FS-*` rows) and **no
gap/parked rows** — invariants only.

**Reserved-variant discipline applied.** Two F2 facts bound what this register
may assert. **(1) The codecs are total over the closed type sets.** `RagChunk`
(3 variants) and `StoreError` (21 variants) have no reserved variants — every
variant is encodable, so rows may quantify over all of them; there are no
"reserved" wire codes. **(2) `blocked_by` is graph-mode-only.** A well-formed
`RagResult` produced by the engine carries `blocked_by` **only** for a graph-mode
empty result (§4.6.1 / §4.3.3); the validator asserts `blocked_by ⇒
RagTrace::Graph`. Rows must therefore generate `blocked_by: Some(..)` only paired
with a `RagTrace::Graph` trace, and `WrongEngine`/`BlockedByWithoutGraphTrace`
are the **reserved** validator fail-reasons (negative-generator territory for the
auditor, not invariant rows). The wire unit has **no** reserved fail-variant rows
(no `FS-*` rows) by the invariant-only rule.

## Property table

| Property-id | Class | Invariant | Strategy-id | Observable-as-property |
|---|---|---|---|---|
| `P-IM-1` | IM | **`RagChunk` codec round-trip identity (all three variants, well-formed `Result`).** For **any** `RagChunk` whose `Result(r)` payload is a **well-formed** `RagResult` — `engine == "gnosis"` with a trace valid for its mode (the decode-validates precondition, mirrored from `P-IM-2`/`P-TP-1`) — decoding the envelope that `encode_chunk` produced returns the identical chunk; this holds for `Done`, for `Error(e)` over the full 21-variant `StoreError` taxonomy, and for `Result(r)` over well-formed `RagResult` values of all four trace shapes. A `Result(r)` built by this generator satisfies that precondition by construction (see the coverage note); a non-conforming `r` is a decode-side fail-state, excluded here. | `strat:chunk-roundtrip` | ∀ generated `chunk: RagChunk` (with `Result` well-formed per the invariant): `decode_chunk(&encode_chunk(&chunk)).unwrap().eq(&chunk)` — element-wise `==` (not just type-tag), including `Error(ValidationError(m))` preserving `m` and the full `Result` body (query/results/citations/trace/blocked_by). |
| `P-IM-2` | IM | **`RagResult` codec bijectivity.** `encode_result`/`decode_result` are an exact round-trip: for **any** well-formed `RagResult` (trace present, `engine == "gnosis"`, cross-field-consistent `blocked_by`), decoding the encoded envelope returns a `RagResult` **equal** to the input in every field and field order. | `strat:result-bijective` | ∀ well-formed `r: RagResult`: `decode_result(&encode_result(&r)).unwrap().eq(&r)` (results vector element-wise: key, `score`, `snippet`, `source`, `parent`, `stale`; citations `==`; trace `==`; blocked_by `==`). |
| `P-IM-3` | IM | **`StoreError::wire_code` uniqueness + stability + non-empty.** The 21 distinct variants map to **21 pairwise-distinct non-empty `&'static str` codes**, and the mapping is a deterministic pure function of the variant (the same variant always yields the same code; the same code never names two variants). | `strat:code-unique` | ∀ distinct `a,b: StoreError` (sampled across all 21 variants): `a.wire_code() != b.wire_code()`; `!a.wire_code().is_empty()`; `a.wire_code() == a.wire_code()` on repeat and for equal variants. |
| `P-IM-4` | IM | **SSE single-event framing round-trip.** For **any** `RagChunk`, the SSE frame `encode_event` produces decodes back to the identical chunk, and its framing matches the pinned single-event contract (`event: <type>\ndata: <json>\n\n`, `<type>` matching the data `"type"`). | `strat:sse-roundtrip` | ∀ `chunk`: `decode_event(&encode_event(&chunk)).unwrap().eq(&chunk)`; the frame starts with `event: `, contains exactly one `data:` line whose JSON `"type"` equals the `event:` value, and ends with a trailing blank line (`\n\n`). |
| `P-SM-1` | SM | **Envelope round-trip preserves `schema_version` + `id_format` + payload identity.** For **any** envelope the codecs build, serializing and re-parsing (`to_json`/`from_json`) preserves the three fields; `schema_version` is `CURRENT_SCHEMA_VERSION` (=1) and `id_format` is `ID_FORMAT_OPAQUE_STRING_V1` on every encoder output. | `strat:envelope-stable` | ∀ `chunk`/`r`/`e`: `env = encode_chunk|encode_result|encode_error`; `from_json(&to_json(&env)?).unwrap().eq(&env)` (payload `Value` `==`-equal); `env.schema_version == CURRENT_SCHEMA_VERSION`; `env.id_format == ID_FORMAT_OPAQUE_STRING_V1`. |
| `P-SM-2` | SM | **`ValidationError`'s message survives the error codec round-trip.** For **any** carried message `m`, `encode_error`/`decode_error` return a `StoreError::ValidationError` whose inner string **equals** `m`; the standalone error payload wire code is the fixed `"validation_error"` regardless of `m`. | `strat:validation-msg` | ∀ `m: String`: `decode_error(&encode_error(&StoreError::ValidationError(m.clone()))).unwrap()` matches `StoreError::ValidationError(m2)` with `m2 == m`; the payload JSON's `"code"` is `"validation_error"`. |
| `P-SM-3` | SM | **Health report is a deterministic pure function of `EngineStatus`.** For **any** `EngineStatus` sampled in this row (all four `EngineState` values, arbitrary `version`, representative subsystem-flag masks within the ≤100-case-per-row budget), `health` yields the same `HealthReport` on repeated calls, and equal `EngineStatus` values yield element-wise equal reports (same `state`, `version`, `subsystems`, `last_error`; `last_error` matches the input — `Some` only where the input's is `Some`). The determinism/faithfulness invariant holds for **any** status the generator samples; the masks are a representative sample, not an "all masks" enumeration. | `strat:health-determinism` | ∀ generated `status` (varied `state ∈ {Ready,Starting,Degraded,Unavailable}`, arbitrary `version`, representative subsystem-flag masks — see the coverage note): `health(&status).eq(&health(&status))`; `health(&status).last_error.is_some() == status.last_error.is_some()`; `health(&clone(status))==health(&status)` element-wise. |
| `P-TP-1` | TP | **Encoder never emits a body the decoder rejects (decode-then-validate total on well-formed input).** For **any** well-formed `RagResult`, `encode_result` produces an envelope whose `payload` is accepted by `decode_rag_result` **and** passes `validate_rag_result`; the codec path never surfaces `EngineError`/`TraceUnavailable` on an encoder-produced body. | `strat:encode-validates` | ∀ well-formed `r`: `let env = encode_result(&r)`; `decode_rag_result(&env.payload)` → `Ok(r2)` with `validate_rag_result(&r2)` → `Ok(())`; equivalently `decode_result(&env).is_ok()`. |

**Class tally:** IM ×4, SM ×3, TP ×1 = **8 rows ≤ 8** ✔.

## Generator-coverage note (per row — boundary + adversarial input shapes)

- **`P-IM-1 — strat:chunk-roundtrip`.** Generate one `Done`; `Error(e)` for each of the 21 `StoreError` variants (incl. `ValidationError` with empty, single-byte-UTF8, and multi-codepoint `m`; every unit variant at least once); `Result(r)` covering **all four** `RagTrace` shapes — `Flat(TraceDescriptor)` and `Vector(TraceDescriptor)` (vary `top_k` 1/50, `source` Local/Zodiac), `Graph(Vec<GraphTraceStep>)` (0, 1, many steps; each step varied edge/state), `Hybrid(HybridTrace)` (legs `["graph","vector","lexical"]` order) — plus `Result(r)` with `blocked_by: None` and with `Some(..)` (graph-only, per the discipline). Boundary: empty `results`/`citations`; a single item; `score` at 0.0/1.0/NaN is **not** generated (serialization is lossless but NaN payload equality is out of scope); `parent: None`/`Some`; `stale: None`/`Some(true/false)`.
- **`P-IM-2 — strat:result-bijective`.** The same `RagResult` corpus as `P-IM-1` plus: a long `query`/`snippet` string; `engine:"gnosis"` only (a non-`"gnosis"` `engine` is the `WrongEngine` reserved fail-reason and is excluded — the validator would reject it); citations that duplicate or reorder result keys (the codec is transparent to scope/order); a result whose `trace` mode matches the result (flat/vector → `TraceDescriptor`; graph + optional `blocked_by`; hybrid → `HybridTrace`). Do **not** generate an empty-trace or mode-mismatched body — those are `MissingTrace`/`BlockedByWithoutGraphTrace` fail-reasons, excluded (validator-rejected).
- **`P-IM-3 — strat:code-unique`.** Enumerate all 21 variants in a fixed order plus several **random permutations**; for each of the pairwise-distinct pairs assert different codes; equality check on a `ValidationError` (same variant, any two `m`) → same `"validation_error"` code; assert `!code.is_empty()` and `code.is_ascii()`. Boundary: the unit variants (`DocumentNotFound`, `ConflictError`, `EngineUnavailable`, `CommunityNotFound`, `SubTaskDagFailed`, …) each once; `ValidationError` with several `m`. No cross-variant collisions allowed.
- **`P-IM-4 — strat:sse-roundtrip`.** For the whole round-trip corpus of `P-IM-1`, assert `decode_event(&encode_event(c)) == c`; separately assert the **framing shape**: exactly one `event:` line whose value equals the data `"type"`, exactly one `data:` line, single-line JSON (no `\r` continuation), and a terminal blank line (`…\n\n`). Boundary: an error event and a done event (payload must be exactly `{"type":"done"}`); a result event with the full body. Do **not** feed a hand-crafted malformed frame here (that is a `DecodeError` fail-state, not an invariant).
- **`P-SM-1 — strat:envelope-stable`.** Across `encode_chunk` (all variants), `encode_result`, and `encode_error` outputs: `from_json(&to_json(&env)).unwrap() == env`; every field asserted (`schema_version==1`, `id_format=="opaque-string-v1"`). Boundary: payload values that contain nested objects/arrays with both keys and non-ASCII content (envelope `Value` equality is deep). This row does **not** exercise unknown `schema_version`/`id_format` — those are decode fail-states, excluded.
- **`P-SM-2 — strat:validation-msg`.** Vary `m` across: empty string; whitespace-only; one ASCII char; multi-char UTF-8 (é, 汉字, emoji); a string containing `"code"`/`"message"` (to prove field escaping); a very long string. Assert the decoded `ValidationError` inner string `== m` and the wire `"code"=="validation_error"`. Boundary: `m` that is itself a JSON object-shaped string (must not be reinterpreted as structure).
- **`P-SM-3 — strat:health-determinism`.** Sweep `state` over all four `EngineState` values; `version` over {empty, `"0.0.0"`, the crate's `CARGO_PKG_VERSION`}; `subsystems` over representative flag masks — all-true, all-false, `embedding:false` only (the DEGRADED archetype), `reranker:false`, and a few arbitrary masks — staying within the ≤100-case-per-row budget; determinism holds for any sampled status (an exhaustive 2^6 enumeration would overrun the row budget and is not claimed). Pair `last_error` with `Some` only when the input's is `Some` (the impl sets `Some` iff `Degraded`) and confirm the report mirrors it. Repeat each call twice; clone-equal check.
- **`P-TP-1 — strat:encode-validates`.** Same well-formed `RagResult` corpus as `P-IM-2` (graph `blocked_by:Some(..)` included, engine `"gnosis"` only); for each, assert `decode_result(&encode_result(&r)).is_ok()` **and** `validate_rag_result(&decode_rag_result(&encode_result(&r).payload)).is_ok()`. This closes the encode→decode→validate totality loop so no encoder output ever trips `EngineError`/`TraceUnavailable`.

## API notes for the TestWriter

- **Pure/synchronous surface.** All F2 wire fns are synchronous pure fns over
  values — `#[test]`, no tokio runtime, no engine instance, no
  `rag_query`/`rag_stream` call. Build `RagChunk`/`StoreError`/`RagResult`/
  `EngineStatus` values directly from the public `src/store/mod.rs` types
  re-exported in `src/lib.rs` (`RagResult`, `RagResultItem`, `RagTrace`,
  `TraceDescriptor`, `GraphTraceStep`, `HybridTrace`, `EngineStatus`, `EngineState`,
  `EngineSubsystems`, `Source`, `RagParent`, `BlockedBy`, `StoreError`,
  `DocumentId`, `NodeId`, `WikiId`).
- **Wire API (from `docs/specs/engine-wire-contract.md`).** `envelope::{
  Envelope, current_schema_version, ID_FORMAT_OPAQUE_STRING_V1 }`;
  `error::{ StoreError::wire_code, StoreError::from_wire, code_table }`;
  `codecs::{ encode_chunk, decode_chunk, encode_error, decode_error,
  encode_result, decode_result }`; `decode::{ decode_rag_result,
  validate_rag_result, decode_chunk_payload, outcome_of, DecodeError,
  ValidationFailure }`; `sse::{ encode_event, decode_event, event_type,
  SseEventType }`; `status::{ health, HealthReport }`.
- **`eq` vs structural identity.** `decode_*(&encode_*(&x)).unwrap() == x` uses
  the types' `PartialEq` (derived on `RagChunk`, `StoreError`, `RagResult`,
  `RagResultItem`, `RagTrace`, `Envelope`, `HealthReport`). For `ValidationError`
  use `matches!(.., StoreError::ValidationError(m2) if m2 == m)`.
- **Reserved-exclusion note for the generator.** The codecs are total over the
  closed sets, so there are **no** reserved wire variants; the only generator
  restriction is **well-formedness on the decode side**: never generate a
  non-`"gnosis"` `engine`, a `blocked_by:Some(..)` without a `RagTrace::Graph`, a
  trace-less/mismatched body, an unknown `schema_version`/`id_format`, or a
  malformed SSE frame — those are `DecodeError`/`ValidationFailure` fail-states
  (the auditor's negative-generator requests), **not** invariant rows.
- **Determinism/seeding.** Use one deterministic pinned seed per binary, ≤100
  cases per row, ≤400 total; stop-after-5; report each row held/broken with its
  `Strategy-id`.
