# §7.2 F2 — Engine wire contract (mechanism-agnostic codecs + encoding layer)

- **Unit:** §7.2 F2 — engine transport/API reconciliation (the last OPEN unit in
  `docs/next-steps.md`).
- **Status:** **CONTRACT (PLANNED-for-F2)** — this is the behavior contract the
  F2 unit's TestWriter derives its red set from, and the wire schema the
  Astrographer shell will implement identically. It pins the codec + validation
  + SSE + health + envelope layer **only**. It does **not** build a server, does
  **not** add runtime dependencies, and does **not** mutate the frozen §4.1/§4.5
  types.
- **Gate:** proposal-review **PASSED** — Architecture A1 (`7-2-f2-review.md`,
  decision `F2-WIRE-CONTRACT-A1`). Code/contract delegation awaits the user's
  go-ahead.
- **Contract cross-refs:** `docs/specs/gnosis.md` §4.6.1 (the retrieval trio),
  §4.1.5 (`RagStore` persistence seam — the wire covers the **retrieval trio**,
  not CRUD), §4.1.4/FS-4 (`ConflictError` = HTTP 409), §4.3.4 (audit), §6
  (FS-1..FS-26 for the code map); `docs/specs/7-2-f2-review.md` (the verdict +
  deliverable set + scope guardrails); `docs/decisions.md` `F2-WIRE-CONTRACT-A1`.
- **Date:** 2026-09-09. **Author-role:** spec_writer.
- **Scope:** `src/wire/` + the re-export surface in `src/lib.rs`. Companion PBT
  register: `docs/specs/7-2-wire-property-register.md`.

---

## 1. What F2 asks

Pin the concrete wire serialization for the Astrographer-shell → Gnosis-engine
proxy seam (§5.1), restricted to the §4.6.1 **retrieval trio** — `ragQuery`
(`RagResult`), `ragStream` (`RagChunk`), `getEngineStatus` (`EngineStatus`,
health) — **plus** the two §4.3.4 audit entry and the SSE framing needed to carry
them. The transport mechanism (HTTP/REST + SSE vs a native IPC channel) is
**deliberately deferred** to a later shell-integration unit; F2 delivers only the
**mechanism-agnostic wire codecs + validation + SSE single-event framing + health
serialization + versioned envelope**. This makes the proxy seam testable
end-to-end (the review's named benefit) and confirms the
`EngineUnavailable`/`EngineError` split against the encoded failure modes.

F2 is **not** a server, **not** a client, and ships **no new runtime deps**
(serde / serde_json / tokio / futures already present suffice). The §4.1.5
`RagStore` persistence surface (34 methods) is out of scope; only the retrieval
trio crosses the wire.

---

## 2. Scope guardrails

**In scope (this contract):**
- `StoreError::wire_code()` — the stable, unique code string per §6 FS variant.
- JSON codecs for the two non-`Serialize` types — `RagChunk` and `StoreError` —
  encoded in `src/wire/`, **not** by mutating the frozen store enum's derive
  surface.
- `RagResult` (already `Serialize`) encoding; the wire wraps it in the envelope
  and adds the decode-then-validate step.
- **decode-then-validate** — malformed body → `EngineError` outcome; well-formed
  body missing `trace` → `TraceUnavailable` outcome (FS-9 / FS-10 realized at the
  decoder).
- **Single-event SSE** framing (`event: result|done|error`), honest to the
  single-shot `rag_stream` (`[Result, Done]` / `[Error, Done]`).
- **Health** serialization of the already-`Serialize` `EngineStatus`.
- The versioned **envelope** `{schemaVersion, idFormat, payload}` carrying the
  deferred UUID-v4 id decision.

**NOT in scope (explicit):**
- A hosted HTTP or native-IPC server (deferred to the shell-integration unit).
- Full `RagStore` CRUD routing.
- HTTP-status **rendering** (this contract documents the map as reference only;
  the shell renders it).
- Bind/auth/TLS + loopback enforcement (recorded **shell-owned**).
- Boot→READY lifecycle (the existing `set_engine_state(Ready)` fixture suffices).
- A shell-side SSE client.
- RFC-4122 id adoption (deferred to the suite/HANDOFF via the `idFormat` seam).
- Incremental multi-chunk `rag_stream` streaming (a separate `rag_stream` engine
  unit).
- Zero new runtime dependencies.

---

## 3. Module map (`src/wire/`)

| file | concern | exported public API |
| --- | --- | --- |
| `src/wire/mod.rs` | declares + re-export the submodules | `pub mod error; pub mod codecs; pub mod decode; pub mod sse; pub mod status; pub mod envelope;` + flat re-exports (below) |
| `error.rs` | `StoreError` → stable code string + reverse lookup | `impl StoreError { pub fn wire_code(&self) -> &'static str }`; `pub fn from_wire(code: &str, message: Option<&str>) -> Option<StoreError>`; `pub fn code_table() -> &'static [WireCodeRow]` |
| `codecs.rs` | JSON codecs for `RagChunk`, `StoreError`, `RagResult` (envelope-returning) | `encode_chunk`, `decode_chunk`, `encode_error`, `decode_error`, `encode_result`, `decode_result`, `encode_event`(→ sse), `WireCodecError` |
| `decode.rs` | decode-then-validate | `DecodeError`, `ValidationFailure`, `decode_rag_result`, `validate_rag_result`, `decode_chunk_payload`, `outcome_of`. `outcome_of` maps a `DecodeError` → `RagChunk::Error(StoreError::EngineError | StoreError::TraceUnavailable)` |
| `sse.rs` | single-event SSE framing | `SseEventType`, `encode_event`, `decode_event` |
| `status.rs` | schemaVersion-aware health | `HealthReport`, `health(&EngineStatus) -> HealthReport` |
| `envelope.rs` | the versioned envelope | `Envelope`, `CURRENT_SCHEMA_VERSION`, `ID_FORMAT_OPAQUE_STRING_V1`, `current_schema_version()`, `Envelope::{to_json, from_json}` |

**Re-export surface to add in `src/lib.rs`** (the paths the TestWriter calls):

```rust
pub mod wire;
pub use self::wire::codecs;
pub use self::wire::decode;
pub use self::wire::envelope;
pub use self::wire::error;   // unconditional: crate-root `gnosis::error` (no collision exists)
pub use self::wire::sse;
pub use self::wire::status;
pub use self::wire::envelope::Envelope;
pub use self::wire::decode::{DecodeError, ValidationFailure};
pub use self::wire::status::HealthReport;
```

The TestWriter reaches the surface as `gnosis::wire::{codecs, decode,
envelope, error, sse, status}` and via the flat re-exports `gnosis::Envelope`,
`gnosis::DecodeError`, `gnosis::ValidationFailure`, `gnosis::HealthReport`,
`gnosis::error`.
`wire_code` is an inherent method on the existing `gnosis::StoreError` (no
trait/impl change to the store enum; the `impl` lives in `error.rs`).

---

## 4. Canonical wire shapes

### 4.1 The versioned envelope (`envelope.rs`)

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")] // wire keys are schemaVersion / idFormat (shell-friendly)
pub struct Envelope {
    pub schema_version: u32,
    pub id_format: String,
    pub payload: serde_json::Value,
}

pub const CURRENT_SCHEMA_VERSION: u32 = 1;
pub const ID_FORMAT_OPAQUE_STRING_V1: &str = "opaque-string-v1";
pub fn current_schema_version() -> u32 { CURRENT_SCHEMA_VERSION }

impl Envelope {
    pub fn to_json(&self) -> Result<String, WireCodecError>; // serde_json::to_string
    pub fn from_json(s: &str) -> Result<Envelope, DecodeError>; // parse + shape-check
    pub fn with_payload(payload: serde_json::Value) -> Envelope; // schema_version=1, id_format="opaque-string-v1"
}
```

The struct keeps **snake_case Rust fields** (`schema_version`, `id_format`) for the
TestWriter's field access, and the `#[serde(rename_all = "camelCase")]` attribute
makes `serde_json::to_string` emit the **camelCase wire keys** below; `Deserialize`
accepts exactly those same camelCase keys. `to_json`/`from_json` use the same
serde derive path, so the JSON example, the §10 struct, and the §12 golden vectors
all serialize identically (they cannot disagree — they are the same two functions).

Envelope JSON (canonical — equals `to_json(&Envelope{…})` exactly; no spaces after
`:`/`,` and field-order fixed by the struct):
```json
{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":<chunk | result | error json>}
```

**UUID-v4 deferral seam.** Ids (`DocumentId::0`, `NodeId::0`, `WikiId::0`) cross
the wire as **opaque strings** — the wire never parses or validates them as
UUIDs. Changing the id scheme (e.g. adopting RFC-4122 later) **never changes the
wire shape**; it only changes what opaque string the engine emits, plus
optionally a new `id_format` value. `"opaque-string-v1"` is the only implemented
`id_format`; a future `id_format` value is the **extensibility seam**, not code
in F2. RFC-4122 is **not** implemented.

### 4.2 Canonical chunk JSON (identical in `envelope.payload` and the SSE `data:` line)

`RagChunk` is **not** `Serialize`; the wire represents it as a single JSON object
with a discriminative `"type"`:

| Cargo variant | wire JSON | notes |
| --- | --- | --- |
| `RagChunk::Result(r)` | `{"type":"result","result":<RagResult body via serde>}` | `RagResult` is `Serialize`; `trace` is required (non-optional). |
| `RagChunk::Done` | `{"type":"done"}` | terminal, no payload. |
| `RagChunk::Error(e)` | `{"type":"error","code":"<wire_code>","message":"<Display text>"}` | `message` = `format!("{}", e)`; for `ValidationError(m)` this is the carried detail `m`. |

**Body shape note (serde-frozen).** The `<RagResult body via serde>` in the `Result`
row is **exactly** what `serde_json::to_string` emits for the real frozen
`RagResult`/`RagResultItem`/`RagTrace` types — the wire does **not** re-case, re-tag,
or hand-reroute it. That means: **snake_case** field keys (`document_id`, `node_id`,
`top_k`); **PascalCase** enum-unit values (`"Local"`, `"Zodiac"`, `"Flat"`,
`"Graph"`, `"Vector"`, `"Hybrid"`); `RagTrace` **externally-tagged** by variant name
(`{"Flat":{…}}`, `{"Graph":[…]}`, `{"Hybrid":{…}}`); id newtypes (`DocumentId`,
`NodeId`) serialized as their inner strings; optional fields without
`skip_serializing_if` (`parent`, `stale`, `blocked_by`) always present, `null` when
`None`. These body types are frozen (§4.1/§4.5) and immutable in F2; see §12 V-5 for
the pinned full body. Only the wire-defined types (**`Envelope`**, **`HealthReport`**,
the chunk/`code`/`message`/`type` wrapper) use camelCase at their top level.

### 4.3 Canonical error JSON (the non-chunk codec, `codecs::encode_error`)

`{"code":"<wire_code>","message":"<Display text>"}`. This is the shape used when
an error is encoded on its own (e.g. as a query's error outcome); the chunk
encoder adds the `"type":"error"` discriminator on top.

### 4.4 Canonical SSE frame (`sse.rs`)

Single event, LF line endings, terminal blank line:

```
event: <type>
data: <canonical chunk JSON, single line>
<blank line>
```

`<type>` ∈ `{result, done, error}` and **must equal** the `"type"` field of the
data JSON (a mismatch is a `DecodeError`). `data:` is the single-line JSON
(no multi-line SSE continuation in F2). No `id:`/`retry:` fields.

---

## 5. `error.rs` — `StoreError::wire_code` table (exhaustive)

`pub fn wire_code(&self) -> &'static str`. The full 21-variant (the current
`Store` taxonomy) one-to-one map. Each code is **stable** (frozen by this
contract), **unique** (pairwise distinct), **non-empty**, derived from the real
enum only — never invented:

| `StoreError` variant | §6 FS | wire code |
| --- | --- | --- |
| `DocumentNotFound` | FS-1 | `"not_found"` |
| `WikiNotFound` | FS-2 | `"wiki_not_found"` |
| `ValidationError(String)` | FS-3 | `"validation_error"` |
| `ConflictError` | FS-4 | `"conflict"` |
| `DocumentInUse` | FS-5 | `"doc_in_use"` |
| `InvalidState` | FS-6 | `"invalid_state"` |
| `UnresolvedReference` | FS-7 | `"unresolved_reference"` |
| `EngineUnavailable` | FS-8 | `"engine_unavailable"` |
| `EngineError` | FS-9 | `"engine_error"` |
| `TraceUnavailable` | FS-10 | `"trace_unavailable"` |
| `HopLimitExceeded` | FS-11 | `"hop_limit_exceeded"` |
| `CycleDetected` | FS-12 | `"cycle_detected"` |
| `EmbeddingUnavailable` | FS-13 | `"embedding_unavailable"` |
| `VectorIndexUnavailable` | FS-14 | `"vector_index_unavailable"` |
| `LexicalIndexUnavailable` | FS-15 | `"lexical_index_unavailable"` |
| `RerankerUnavailable` | FS-16 | `"reranker_unavailable"` |
| `CompressionFailed` | FS-17 | `"compression_failed"` |
| `HyDEGenerationFailed` | FS-18 | `"hyde_generation_failed"` |
| `MultiQueryExpansionFailed` | FS-19 | `"multi_query_expansion_failed"` |
| `CommunityNotFound` | FS-25 | `"community_not_found"` |
| `SubTaskDagFailed` | FS-26 | `"sub_task_dag_failed"` |

**(21 variants total.** This is the true count of the `pub enum StoreError` at
`src/store/mod.rs:944-1006` — the reviewed list's "22" was a count slip; the
one-to-one code set is exactly as enumerated above.)

**`ValidationError` carries its message.** The message `m` appears only in the
wire `"message"` field; the wire code stays the fixed `"validation_error"` for
every instance of the variant. `wire_code()` therefore returns `&'static str`
regardless of the carried message.

**Reverse lookup.**

```rust
pub fn from_wire(code: &str, message: Option<&str>) -> Option<StoreError>
```

Maps the canonical code → the unit variant; for `"validation_error"` uses
`message` (required) → `ValidationError(msg.into())`; returns `None` for any
non-canonical / unknown / foreign code. Bijective for the 20 unit variants and
for `validation_error` paired with a message.

```rust
pub struct WireCodeRow { pub code: &'static str, pub variant_name: &'static str, pub display: &'static str }
pub fn code_table() -> &'static [WireCodeRow]; // the 21 rows above, for docs/tests
```

**Fail-state / throw patterns for `wire_code`:** no internal failure; it is a
pure, total function over the closed enum. It never panics, never returns an
empty string, and never returns the same string for two distinct variants.

---

## 6. `codecs.rs` — JSON codecs

All codecs return/receive an `Envelope` (the wire wraps every payload). All
`decode_*` path is **decode-then-validate** (see §7).

```rust
// RagChunk — encode the chunk into an envelope (payload = canonical chunk JSON).
pub fn encode_chunk(chunk: &RagChunk) -> Envelope;
// inverse: decode the envelope back to the chunk, mapping result-body failures
// into EngineError / TraceUnavailable outcomes (decode.rs).
pub fn decode_chunk(env: &Envelope) -> Result<RagChunk, DecodeError>;

// StoreError — standalone error codec (payload = {"code","message"}).
pub fn encode_error(err: &StoreError) -> Envelope;
pub fn decode_error(env: &Envelope) -> Result<StoreError, DecodeError>;

// RagResult — the result body is already Serialize; the wire wraps it + validates.
pub fn encode_result(res: &RagResult) -> Envelope;   // payload = RagResult body
pub fn decode_result(env: &Envelope) -> Result<RagResult, DecodeError>; // decode-then-validate
```

`WireCodecError` (internal serialization error; only reachable on a
non-object payload map, which the codecs never produce — a defensive enum, not a
tested fail-path).

**Round-trip identity (the core contract):**
- `decode_chunk(&encode_chunk(c)) == Ok(c)` for every `RagChunk` (all three
  variants), with `Result` bodies round-tripping through the validation step.
- `decode_error(&encode_error(e)) == Ok(e)` for every `StoreError` (21 variants,
  incl. `ValidationError(m)` preserving `m`).
- `decode_result(&encode_result(r)) == Ok(r)` for every well-formed `RagResult`
  (query, results, engine `"gnosis"`, citations, trace matching the result,
  optional `blocked_by`).

---

## 7. `decode.rs` — decode-then-validate

This is the realization of FS-9 (malformed body → `EngineError`) and FS-10
(well-formed body, missing `trace` → `TraceUnavailable`).

```rust
pub enum DecodeError {
    InvalidJson(String),                 // serde parse/shape failure
    UnknownType(String),                 // chunk payload "type" not in {result,done,error}
    MissingTrace,                        // well-formed result body, no "trace"
    UnsupportedSchemaVersion(u32),       // envelope.schema_version != CURRENT_SCHEMA_VERSION
    UnknownIdFormat(String),             // envelope.id_format not a known value
    InvalidEnvelope(String),             // missing schema_version/id_format/payload, or SSE frame malformed
    UnknownCode(String),                 // error codec: no variant maps to this code
    EventTypeMismatch { event: String, data_type: String }, // SSE event line != data "type"
    ValidationFailed(ValidationFailure), // a valid structural body failed validation
}

pub enum ValidationFailure {
    MissingTrace,                        // trace absent
    WrongEngine(String),                 // RagResult.engine != "gnosis"
    BlockedByWithoutGraphTrace,          // blocked_by is Some but trace is not RagTrace::Graph
}
```

**Functions:**

```rust
// Strict decode of a RagResult body from a JSON value.
pub fn decode_rag_result(json: &serde_json::Value) -> Result<RagResult, DecodeError>;
//   - structurally malformed (wrong field types, invalid enum, etc.)  -> Err(InvalidJson(_))   [FS-9 route]
//   - structurally well-formed but trace absent                        -> Err(MissingTrace)      [FS-10 route]
//   - otherwise -> Ok(valid RagResult)

// Post-decode validation of a constructed/decoded RagResult against the
// §4.6.1 / §4.3.3 invariants: engine == "gnosis"; trace present; a present
// blocked_by implies a RagTrace::Graph trace (blocked_by is only produced by the
// graph empty-result path).
pub fn validate_rag_result(res: &RagResult) -> Result<(), ValidationFailure>;

// Decode the canonical chunk JSON (envelope.payload or the SSE data line).
pub fn decode_chunk_payload(payload: &serde_json::Value) -> Result<RagChunk, DecodeError>;

// Map a DecodeError to the wire outcome chunk (FS-9 / FS-10).
pub fn outcome_of(e: DecodeError) -> RagChunk;
//   InvalidJson | UnknownType | UnsupportedSchemaVersion | UnknownIdFormat
//   | InvalidEnvelope | UnknownCode | EventTypeMismatch | ValidationFailed(_)
//        -> RagChunk::Error(StoreError::EngineError)
//   MissingTrace
//        -> RagChunk::Error(StoreError::TraceUnavailable)
```

**Contract rules:**
- A body that decodes but fails `validate_rag_result` surfaces as an `EngineError`
  **outcome** (`decode_chunk`/`decode_result` return `Err(ValidationFailed(_))`,
  and the caller maps it via `outcome_of`). `MissingTrace` alone → `TraceUnavailable`
  outcome.
- The encoder (`encode_result`) **never** emits a body that `decode_rag_result` +
  `validate_rag_result` rejects: `validate_rag_result(&decode_rag_result(&encoded)?.unwrap())`
  is `Ok` for every encoder input (the P-TP-1 row of the register).

---

## 8. `sse.rs` — single-event SSE

```rust
pub enum SseEventType { Result, Done, Error }

pub fn event_type(chunk: &RagChunk) -> SseEventType;  // Result/Done/Error
pub fn encode_event(chunk: &RagChunk) -> String;      // §4.4 framing; data = canonical chunk JSON
pub fn decode_event(frame: &str) -> Result<RagChunk, DecodeError>;
```

**Contract rules:**
- `encode_event` emits exactly one event: `event: <type>\ndata: <json>\n\n`.
- `decode_event(&encode_event(c)) == Ok(c)` for all three variant classes
  (round-trip). On decode: `InvalidEnvelope` for a frame with no `event:`/`data:`
  lines or trailing non-blank content; `InvalidJson` for an unparseable `data:`;
  `EventTypeMismatch` when `event:` != data `"type"`; result bodies route through
  decode-then-validate.
- **Single-shot honesty:** F2 emits at most one data-carrying event then a
  `done` (or `error` then `done`). No multi-chunk/incremental framing, no
  continuation lines.

---

## 9. `status.rs` — health

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")] // wire keys: schemaVersion / idFormat / lastError
pub struct HealthReport {
    pub schema_version: u32,      // CURRENT_SCHEMA_VERSION
    pub id_format: String,        // ID_FORMAT_OPAQUE_STRING_V1
    pub state: EngineState,       // reused store enum (Serialize) → PascalCase ("Ready"/"Degraded")
    pub version: String,
    pub subsystems: EngineSubsystems, // store/graph/lexical/vector/embedding/reranker (Serialize)
    pub last_error: Option<String>,
}

pub fn health(status: &EngineStatus) -> HealthReport;
```

Like the `Envelope`, the `HealthReport` struct keeps **snake_case Rust fields** and a
`#[serde(rename_all = "camelCase")]` attribute, so it serializes
`schemaVersion`/`idFormat`/`lastError` on the wire (consistent with the envelope).
It **maps** `EngineStatus` field-by-field into its own shape (it does **not** embed
the raw `EngineStatus`), so the emitted `state` value is the real serde output of the
reused `EngineState` enum — PascalCase `"Ready"`/`"Starting"`/`"Degraded"`/
`"Unavailable"`.

**Contract rules:**
- Pure, deterministic function of `EngineStatus` (no hidden state, no I/O): equal
  `EngineStatus` → equal `HealthReport` (the P-SM-3 row of the register).
- Maps `status.state`, `status.version`, `status.subsystems`, and
  `status.last_error` verbatim (`state` emits the PascalCase `EngineState` value).
  The `subsystems` inner flags keep their snake_case single-word field names
  (`store`, `graph`, `lexical`, `vector`, `embedding`, `reranker`) — unchanged by the
  top-level rename.
- `last_error` is `Some` exactly when `status.last_error` is `Some` (the §4.6.1
  `getEngineStatus` impl populates it only in `DEGRADED` state; the report is a
  faithful projection and does **not** invent a `last_error`).

---

## 10. `envelope.rs` — helpers (recap of §4.1)

`Envelope` is `Serialize`/`Deserialize` (the TestWriter round-trips it through
`serde_json::to_string`/`from_str` or via `to_json`/`from_json`). `to_json` never
fails for our payload types; `from_json` returns `DecodeError::InvalidJson` on a
parse error and `DecodeError::InvalidEnvelope` when any of the three required
top-level fields are absent or wrongly typed. `from_json` does **not** itself
validate `schema_version`/`id_format` against the constants (that is a
`decode_chunk`/transport concern); unknown values surface as
`UnsupportedSchemaVersion`/`UnknownIdFormat` at the decode step.

---

## 11. Documented HTTP-status map (reference-only — rendered by the shell unit, not built here)

This is a **documentation table for the Astrographer shell unit**, which owns the
HTTP transport + status rendering. F2 ships **no** HTTP code and **no** status
rendering; the mapping is frozen here so the shell (and a TestWriter's conformance
assertions, where relevant) implements it identically. `ConflictError` = **409**
is **mandated** (FS-4, `docs/specs/gnosis.md` §4.1.4).

| wire code | `StoreError` | documented HTTP status | rationalization |
| --- | --- | --- | --- |
| `"not_found"` | `DocumentNotFound` | 404 | FS-1 |
| `"wiki_not_found"` | `WikiNotFound` | 404 | FS-2 |
| `"validation_error"` | `ValidationError` | 400 | FS-3 |
| `"conflict"` | `ConflictError` | **409** | FS-4 (mandated) |
| `"doc_in_use"` | `DocumentInUse` | 409 | FS-5 (referential delete conflict) |
| `"invalid_state"` | `InvalidState` | 409 | FS-6 (state-transition conflict) |
| `"unresolved_reference"` | `UnresolvedReference` | 422 | FS-7 (publish gate, unprocessable) |
| `"engine_unavailable"` | `EngineUnavailable` | 503 | FS-8 |
| `"engine_error"` | `EngineError` | 502 | FS-9 (upstream produced malformed result) |
| `"trace_unavailable"` | `TraceUnavailable` | 502 | FS-10 |
| `"hop_limit_exceeded"` | `HopLimitExceeded` | 422 | FS-11 |
| `"cycle_detected"` | `CycleDetected` | 409 | FS-12 |
| `"embedding_unavailable"` | `EmbeddingUnavailable` | 503 | FS-13 |
| `"vector_index_unavailable"` | `VectorIndexUnavailable` | 503 | FS-14 |
| `"lexical_index_unavailable"` | `LexicalIndexUnavailable` | 503 | FS-15 |
| `"reranker_unavailable"` | `RerankerUnavailable` | 503 | FS-16 |
| `"compression_failed"` | `CompressionFailed` | 500 | FS-17 |
| `"hyde_generation_failed"` | `HyDEGenerationFailed` | 500 | FS-18 |
| `"multi_query_expansion_failed"` | `MultiQueryExpansionFailed` | 500 | FS-19 |
| `"community_not_found"` | `CommunityNotFound` | 404 | FS-25 |
| `"sub_task_dag_failed"` | `SubTaskDagFailed` | 500 | FS-26 |

---

## 12. Golden conformance vectors

Exact bytes/JSON a conforming implementation (the engine F2 codecs and the later
shell) must reproduce. All envelopes use `schemaVersion:1`,
`idFormat:"opaque-string-v1"`.

**V-1 — `encode_chunk(&RagChunk::Done)`:**
```
{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"type":"done"}}
```

**V-2 — `encode_chunk(&RagChunk::Error(StoreError::ConflictError))`:**
```
{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"type":"error","code":"conflict","message":"optimistic-concurrency conflict: stale base revision"}}
```

**V-3 — `encode_chunk(&RagChunk::Error(StoreError::ValidationError("empty query".into())))`:**
```
{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"type":"error","code":"validation_error","message":"empty query"}}
```
(`"message"` carries the variant's detail; the code stays `"validation_error"`.)

**V-4 — `encode_error(&StoreError::WikiNotFound)` (standalone non-chunk codec):**
```
{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"code":"wiki_not_found","message":"wiki not found"}}
```

**V-5 — `encode_result(r)` for a minimal flat `RagResult`**
(`query:"q"`, one item, `engine:"gnosis"`, flat `TraceDescriptor` trace). This is the
**exact** byte output of `serde_json::to_string` on the frozen `RagResult`/
`RagResultItem`/`TraceDescriptor`/`Source`/`QueryMode` values below: snake_case keys,
PascalCase enum values, external-tagged `RagTrace`, id newtypes as bare strings,
`null` for absent `parent`/`stale`/`blocked_by`:
```
{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"query":"q","results":[{"document_id":"d1","node_id":"n1","score":0.9,"snippet":"s","source":"Local","parent":null,"stale":null}],"engine":"gnosis","citations":[["d1","n1"]],"trace":{"Flat":{"mode":"Flat","engine":"gnosis","top_k":10,"source":"Local"}},"blocked_by":null}}
```
(`blocked_by` is `null` when absent — the field is `#[serde(default)]`, which only
defaults **deserialization**; the field is always **serialized**.)

**V-6 — the three SSE event frames** (from `encode_event`):
- `RagChunk::Done`:
  ```
  event: done
  data: {"type":"done"}
  <blank line>
  ```
- `RagChunk::Error(ValidationError("empty query".into()))`:
  ```
  event: error
  data: {"type":"error","code":"validation_error","message":"empty query"}
  <blank line>
  ```
- `RagChunk::Result(r)` (the §12 V-5 result): `event: result` + `data: {"type":"result","result":{…V-5 payload body…}}`.

**V-7 — three representative `wire_code` samples:** `DocumentNotFound`→`"not_found"`,
`ConflictError`→`"conflict"`, `EngineUnavailable`→`"engine_unavailable"`.

**V-8 — health reports** (canonical `HealthReport` JSON — camelCase top-level per §9, `state` the real PascalCase `EngineState`; `subsystems` flags single-word, unchanged):
- Ready engine (`EngineStatus{state:Ready, version:"…", subsystems:all-true, last_error:None}`):
  ```
  {"schemaVersion":1,"idFormat":"opaque-string-v1","state":"Ready","version":"…","subsystems":{"store":true,"graph":true,"lexical":true,"vector":true,"embedding":true,"reranker":true},"lastError":null}
  ```
- Degraded engine (`state:Degraded, subsystems.embedding:false,
  last_error:Some("a non-core subsystem (embedding/reranker) is unavailable")`):
  ```
  {"schemaVersion":1,"idFormat":"opaque-string-v1","state":"Degraded","version":"…","subsystems":{"store":true,"graph":true,"lexical":true,"vector":true,"embedding":false,"reranker":true},"lastError":"a non-core subsystem (embedding/reranker) is unavailable"}
  ```

**V-9 — decode-then-validate vector pair:**
- A body missing `trace` → `DecodeError::MissingTrace` → `outcome_of` → `RagChunk::Error(StoreError::TraceUnavailable)`.
- A truncated/`null` body or wrong field type → `DecodeError::InvalidJson(_)` → `outcome_of` → `RagChunk::Error(StoreError::EngineError)`.

---

## 13. Valid/happy + fail states per function (TestWriter assertion guide)

| wire fn | valid/happy | fail-state |
| --- | --- | --- |
| `StoreError::wire_code()` | returns a stable, non-empty, unique code for all 21 variants | none (total, pure) |
| `StoreError::from_wire` | canonical code → that variant; `validation_error` + message → `ValidationError(m)` | unknown/foreign code → `None` |
| `encode_chunk` | any `RagChunk` → envelope with canonical JSON (result body via serde) | none (serialize can't fail on our payloads) |
| `decode_chunk` | round-trips all three variant classes | `InvalidJson`, `UnknownType`, `MissingTrace`, `UnsupportedSchemaVersion`, `UnknownIdFormat`, `InvalidEnvelope`, `EventTypeMismatch`, `ValidationFailed` |
| `encode_error` | 21 variants → `{"code","message"}` envelope | none |
| `decode_error` | canonical `{code,message}` → that variant | `UnknownCode` for a non-canonical code; `InvalidJson` for a bad shape |
| `encode_result` | any well-formed `RagResult` → envelope wrapping the serde body | none |
| `decode_result` | well-formed body, trace present, engine `"gnosis"` → `Ok(r)` | missing trace → `MissingTrace`; malformed → `InvalidJson`; invariant break → `ValidationFailed` |
| `validate_rag_result` | valid result → `Ok(())` | `MissingTrace`; `WrongEngine`; `BlockedByWithoutGraphTrace` |
| `encode_event` | any `RagChunk` → single SSE frame | none |
| `decode_event` | round-trips all three event types | frame malformed (`InvalidEnvelope`), bad JSON (`InvalidJson`), event/data type mismatch (`EventTypeMismatch`), result-body fails validation |
| `health(&EngineStatus)` | any `EngineStatus` → deterministic `HealthReport` | none (pure) |
| `Envelope::to_json`/`from_json` | round-trips `schema_version/id_format/payload` | `from_json`: `InvalidJson` (parse), `InvalidEnvelope` (missing/typed-weak field) |

**Cross-cutting TestWriter states:**
- **Cross-wiki:** a `RagResult` whose items/citations reference documents from a
  wiki other than the requested one is a *data* concern of the engine (already
  guaranteed by §4.5); the wire codec does **not** enforce wiki scope and must
  round-trip such a result byte-for-byte (it is transparent to scope).
- **Unknown `id_format`:** decode of an envelope with `id_format:"uuid-v4"` →
  `UnknownIdFormat` → `EngineError` outcome (the RFC-4122 seam is documented, not
  implemented).
- **Unknown `schema_version`:** decode of `schema_version:99` →
  `UnsupportedSchemaVersion(99)` → `EngineError` outcome.
- **Empty `payload`:** `InvalidEnvelope` for a result/done chunk; a `done` chunk
  with a non-`{}` payload → still `Ok(Done)` only if payload is `{"type":"done"}`
  (exact), else `InvalidEnvelope`/`UnknownType`.

---

## 14. Cross-references and ownership hand-off

- **Contract authority:** `docs/specs/gnosis.md` §4.6.1, §4.1.5, §4.1.4, §4.3.4, §6.
- **Gate record + scope guardrails:** `docs/specs/7-2-f2-review.md`
  (`F2-WIRE-CONTRACT-A1`); decision row `docs/decisions.md` `F2-WIRE-CONTRACT-A1`.
- **PBT:** `docs/specs/7-2-wire-property-register.md` (+ TestWriter's
  `tests/props_wire.rs`).
- **Conformance tests (TestWriter):** `tests/wire_conformance.rs`.

**Owned by a later shell-integration unit** (not F2, per §2):
1. The HTTP-over-native-IPC decision + the server host (axum/hyper/tower deps).
2. The shell-side SSE client consuming the §4.4 event schema.
3. Bind-loopback + auth/TLS policy (recorded shell-owned) + loopback enforcement.
4. Full `RagStore` persistence CRUD routing.
5. The engine boot→READY lifecycle + its end-to-end transport test.
6. **Rendering** the §11 HTTP-status map (incl. `ConflictError`=409).
7. D2 engine-absent degrade **behavior** at the shell (F2 pins only the reporting).

---

## 15. What the TestWriter derives (red-set readiness)

- `wire_conformance.rs`: §5 table exhaustiveness + uniqueness; §6 round-trips
  across all `RagChunk`/`StoreError`/`RagResult` variants; §7 decode-then-validate
  incl. all `DecodeError` routes; §8 SSE round-trip + all `DecodeError`/frame
  fail-states; §9 health determinism; §12 golden vectors byte-for-byte.
- **Byte-for-byte golden assertions.** V-1..V-8 pin the **exact string** each codec
  must emit — the raw `serde_json::to_string`/`encode_event` output on the given
  input value (no re-formatting). They cover the two wire-defined camelCase surfaces
  (`Envelope`, `HealthReport`) and the serde-frozen body (`RagResult`/`RagTrace`/
  `RagResultItem`/`Source`/`QueryMode` — snake_case keys, PascalCase enums,
  externally-tagged trace, id newtypes as strings, `null` for absent `parent`/`stale`/
  `blocked_by`). The red tests assert equality **against these exact bytes**.
- `props_wire.rs`: the 8 invariant rows of the PBT register (FILE 2 asserts
  decoded-value `eq` against the input, **not** byte identity — it is independent of
  the literal JSON in FILE 1 and needs no change alongside the V-5/V-8 rewrites).
- The red set is derived **from this contract alone**; the implementer lands the
  least `src/wire/` code to green.
