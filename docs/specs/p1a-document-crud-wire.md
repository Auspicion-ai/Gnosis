# §7.2 P1a — Document-CRUD wire contract (the 11 §4.1 request/response wire shapes)

- **Unit:** §7.2 P1a — document-CRUD wire-contract unit (the MVP wire unit, roadmap §5.1).
- **Status:** **CONTRACT (REALIZED-GREEN — LANDED 2026-09-10).** This is the
  code-bearing TDD wire unit contract. It freezes the **11 §4.1 document-CRUD request/response
  wire shapes** and the CRUD request/response codecs + decode-then-validate + RBAC credential
  shape + endpoint-path ownership. It **extends** the F2 wire layer (`docs/specs/engine-wire-contract.md`);
  it does **not** replace it. The graph/fact/consistency/RAG-companion wire shapes (P1b–P1e) are
  **DEFERRED follow-ons, NOT part of this unit.**
- **Gate:** proposal-review **PASSED** (roadmap §5.1). This spec is the §5.1 P1a deliverable; the
  TestWriter derives its red set from this contract alone, then the Implementer lands the least
  `src/wire/` code to green.
- **Extends:** `docs/specs/engine-wire-contract.md` (F2 — the `Envelope`,
  `CURRENT_SCHEMA_VERSION=1`, `idFormat:"opaque-string-v1"`, the "body via serde, don't re-case"
  rule, decode-then-validate, the `wire_code()` error table, the §11 HTTP-status map (21 rows),
  the golden vectors V-1..V-9, the SSE framing). P1a reuses all of it verbatim and adds the CRUD
  surface on top.
- **Contract cross-refs:** `docs/specs/gnosis.md` §4.1.3 (the 11 document-store operations),
  §4.1.1 (the DRAFT→PUBLISHED→ARCHIVED state machine), §4.1.4 (optimistic concurrency,
  `ConflictError` = 409), §4.4.3 (the publish gate, `UnresolvedReference`), §4.4.5 (delete
  integrity, `DocumentInUse`), §6 (FS-1..FS-26); `docs/specs/engine-wire-contract.md` (F2, the
  base wire layer); `docs/specs/7-2-wire-property-register.md` (the F2 register format this
  unit's §5.x register mirrors).
- **Date:** 2026-09-09. **Author-role:** spec_writer.
- **Scope:** `src/wire/` (a new `crud.rs` + the re-export surface in `src/lib.rs`). Companion PBT
  register: **authored HERE (§5.x)** per the F2 precedent; the TestWriter executes it.

---

## 0. Compile-horizon review (status / what it asks / feasibility / units + gaps / costs-benefits)

- **Status:** CONTRACT (REALIZED-GREEN — LANDED 2026-09-10). The P1a wire unit is **implemented
  and green** (see the P1a DONE row in `docs/next-steps.md`). It is a **code-bearing TDD wire
  unit** (the §5.x Property register is MANDATORY per the PBT gate).
- **What it asks.** Freeze the concrete wire serialization for the **11 §4.1 document-CRUD
  request/response shapes** crossing the Astrographer-shell → Gnosis-engine proxy seam (§4.1.5):
  `createDocument`, `getDocument`, `updateDocument`, `deleteDocument`, `publishDocument`,
  `unpublishDocument`, `archiveDocument`, `listDocuments`, `createWiki`, `getWiki`, `listWikis`.
  Each request is a **request envelope** (a `"method"` discriminator + per-method `args` in the
  `payload`); each response is a **response envelope** (the `"method"` discriminator + the
  serde-frozen result body, or the `{"code","message"}` error via the non-chunk codec). P1a pins
  the CRUD request/response encode + decode, the CRUD decode-then-validate, the RBAC credential
  shape on mutating requests, the endpoint-path ownership (H4), and the §5.x Property register.
- **Feasibility verdict.** **FEASIBLE.** The F2 wire layer already provides the `Envelope`, the
  `wire_code()`/`from_wire` error codec, the §11 HTTP-status map, and the decode-then-validate
  pattern. All 11 CRUD body types (`CreateDocumentRequest`, `UpdateDocumentRequest`,
  `ListDocumentsFilter`, `Document`, `DocumentSummary`, `DocumentList`, `Wiki`, `WikiId`,
  `DocumentId`, `DocState`, `Graph`, `Node`, `Edge`) are already `Serialize`/`Deserialize`
  (§4.1.3, `src/store/mod.rs`). P1a adds only the wire-defined CRUD wrapper types (camelCase
  top-level) + the CRUD codecs + the CRUD validation + the endpoint constants. **No new runtime
  dependencies** (serde/serde_json suffice). The only new wire-layer type is a CRUD-specific
  `DecodeError` variant (`UnknownMethod`) and a CRUD-specific `ValidationFailure`/`CrudResponseError`
  surface — both wire-layer extensions, not changes to the frozen §4.1 store types.
- **The units + gaps.** **In scope:** the 11 CRUD request/response wire shapes; the CRUD
  request/response encode + decode; CRUD decode-then-validate; the RBAC credential shape (H3);
  the endpoint-path constants (H4); the §5.x Property register. **Explicitly deferred (gaps for
  later units, NOT this one):** the graph/fact/consistency/RAG-companion wire shapes (P1b–P1e);
  the HTTP server + status rendering (P2); the shell-side client (A1); the SSE surface for CRUD
  (there is none — CRUD is request/response; the SSE event schema stays retrieval-only); the
  RFC-4122 id adoption (reused unchanged via the `idFormat` seam); the RBAC *enforcement*
  semantics (the engine is the enforcer; P1a pins only the credential *shape*). — RECONCILED 2026-09-22
  (`RBAC-DOC-DRIFT`): the engine pins the credential **SHAPE** + its decode-layer **presence check** only;
  the authority mapping and the deny are **SHELL**-side — see `GNOSIS-RBAC-EDIT-ENFORCEMENT` /
  `docs/HANDOFF.md` §8-RBAC. The historical clause stands as written.
- **Costs-benefits.** **Cost:** one new `src/wire/crud.rs` module + a small re-export surface +
  the CRUD conformance + property test suites. **Benefit:** the 11 CRUD wire shapes are frozen
  byte-exact and shared as the cross-repo conformance fixture, so P2 (server) and A1 (client)
  consume the **same** paths and shapes; the RBAC credential shape and endpoint ownership are
  pinned before any server/client code; the §5.x register makes the CRUD codecs property-tested
  (PBT gate) exactly as F2 was.

---

## 1. What P1a asks

P1a freezes the **11 §4.1 document-CRUD request/response wire shapes** (the MVP wire unit). It
extends the F2 wire layer: every CRUD request + response is wrapped in the F2 `Envelope`
(`{schemaVersion, idFormat, payload}`, `CURRENT_SCHEMA_VERSION=1`, `idFormat:"opaque-string-v1"`).
A CRUD request is a **request envelope** (a request `"method"` + per-method `args` in the
`payload`); a CRUD response is a **response envelope** (the `"method"` discriminator + the
serde-frozen result body, or the `{"code","message"}` error via the non-chunk codec). The
UUID-v4 deferral seam is reused unchanged — ids cross as opaque strings.

P1a is **not** a server, **not** a client, and ships **no new runtime deps**. It delivers only
the **mechanism-agnostic CRUD wire codecs + validation + endpoint constants + RBAC credential
shape + the §5.x Property register**. The graph/fact/consistency/RAG-companion wire shapes
(P1b–P1e) are deferred follow-ons, NOT part of this unit.

---

## 2. Scope guardrails

**In scope (this contract):**
- The 11 CRUD request/response wire shapes (§4.1.3), each wrapped in the F2 `Envelope`.
- The CRUD request/response encode + decode: `encode_crud_request`/`decode_crud_request`,
  `encode_crud_response`/`encode_crud_error`/`decode_crud_response`.
- CRUD decode-then-validate: a malformed CRUD response body → `EngineError` (502); a well-formed
  body failing a CRUD-specific invariant → the appropriate `ValidationFailure` outcome.
- The RBAC credential shape (H3) on the 7 mutating request envelopes.
- The endpoint-path ownership (H4): the 11 pinned MVP paths as shared `ENGINE_ENDPOINTS`-style
  constants.
- The §5.x Property register (MANDATORY, PBT gate).
- The NEW-2 qualification: the server-side request-decode outcome (400/422, NOT 502) is a
  transport-level status, NOT a `StoreError` wire code, and has NO §11 row.

**NOT in scope (explicit):**
- A hosted HTTP or native-IPC server (deferred to P2).
- The shell-side client (deferred to A1).
- The graph/fact/consistency/RAG-companion wire shapes (P1b–P1e — deferred follow-ons).
- HTTP-status **rendering** (the §11 map is reused as reference; P2 renders it).
- Bind/auth/TLS + loopback enforcement (recorded shell-owned, as in F2).
- The RBAC **enforcement** semantics (the engine is the enforcer; P1a pins only the credential
  shape). — RECONCILED 2026-09-22 (`RBAC-DOC-DRIFT`): the engine pins the credential **SHAPE** + its
  decode-layer **presence check** only; the authority mapping and the deny are **SHELL**-side — see
  `GNOSIS-RBAC-EDIT-ENFORCEMENT` / `docs/HANDOFF.md` §8-RBAC. The historical clause stands as written.
- RFC-4122 id adoption (reused unchanged via the `idFormat` seam).
- Any SSE surface for CRUD (CRUD is request/response; the SSE event schema stays retrieval-only).
- Zero new runtime dependencies.

---

## 3. Module map (`src/wire/`)

P1a adds one new module to the F2 `src/wire/` tree and extends the re-export surface in
`src/lib.rs`. The F2 modules (`envelope.rs`, `codecs.rs`, `decode.rs`, `error.rs`, `status.rs`,
`sse.rs`, `mod.rs`) are **unchanged** except for the additions noted below.

| file | concern | exported public API (P1a additions) |
| --- | --- | --- |
| `src/wire/crud.rs` (NEW) | the 11 CRUD request/response wire shapes + codecs + validation + endpoint constants | `CrudMethod`, `CrudRequestArgs`, `CrudResult`, `CrudResponseError`, `CrudValidationFailure`, `encode_crud_request`, `decode_crud_request`, `encode_crud_response`, `encode_crud_error`, `decode_crud_response`, `validate_crud_result`, `ENGINE_ENDPOINTS`, `ENDPOINT_*` constants |
| `src/wire/decode.rs` (extended) | adds one CRUD-specific `DecodeError` variant | `DecodeError::UnknownMethod(String)` (new variant; all existing variants unchanged) |
| `src/wire/mod.rs` (extended) | declares + re-exports the new module | `pub mod crud;` + flat re-exports `CrudMethod`, `CrudResult`, `CrudResponseError`, `CrudValidationFailure` |
| `src/lib.rs` (extended) | re-export surface | `pub use self::wire::crud;` + flat re-exports `CrudMethod`, `CrudResult`, `CrudResponseError`, `CrudValidationFailure` |

**Re-export surface to add in `src/lib.rs`** (the paths the TestWriter calls):

```rust
pub mod wire;                       // already present (F2)
pub use self::wire::crud;           // NEW
pub use self::wire::crud::{CrudMethod, CrudResult, CrudResponseError, CrudValidationFailure}; // NEW
```

The TestWriter reaches the surface as `gnosis::wire::crud::{…}` and via the flat re-exports
`gnosis::CrudMethod`, `gnosis::CrudResult`, `gnosis::CrudResponseError`,
`gnosis::CrudValidationFailure`. The F2 `Envelope`, `DecodeError`, `ValidationFailure`,
`HealthReport`, `error`, `codecs`, `decode`, `sse`, `status` surfaces are unchanged and reused.

---

## 4. Canonical wire shapes

### 4.1 The versioned envelope (reused unchanged from F2 §4.1)

Every CRUD request + response is wrapped in the F2 `Envelope`:

```json
{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":<crud request | crud response json>}
```

`CURRENT_SCHEMA_VERSION = 1`, `ID_FORMAT_OPAQUE_STRING_V1 = "opaque-string-v1"`. The UUID-v4
deferral seam is reused unchanged: `DocumentId`/`WikiId` cross the wire as **opaque strings**;
the wire never parses or validates them as UUIDs. `"opaque-string-v1"` is the only implemented
`id_format`; a future value is the extensibility seam, not code in P1a.

### 4.2 The CRUD request envelope (the request discriminator + per-method args)

The request `payload` is a wire-defined object with a camelCase `"method"` discriminator + a
per-method `"args"` object:

```json
{"method":"<camelCase method>","args":{<per-method args>}}
```

**`"method"` values (camelCase, matching the §4.1.3 operation names):**

| `CrudMethod` variant | wire `"method"` value |
| --- | --- |
| `CreateDocument` | `"createDocument"` |
| `GetDocument` | `"getDocument"` |
| `UpdateDocument` | `"updateDocument"` |
| `DeleteDocument` | `"deleteDocument"` |
| `PublishDocument` | `"publishDocument"` |
| `UnpublishDocument` | `"unpublishDocument"` |
| `ArchiveDocument` | `"archiveDocument"` |
| `ListDocuments` | `"listDocuments"` |
| `CreateWiki` | `"createWiki"` |
| `GetWiki` | `"getWiki"` |
| `ListWikis` | `"listWikis"` |

The 11 values are **pairwise distinct, non-empty, camelCase** (the §5.x `P-IM-4` row).

**Per-method `args` shapes.** The `args` object is a wire-defined type with **camelCase top-level
fields** (`caller`, `wikiId`, `documentId`, `name`, `body`). The serde-frozen store body is
embedded verbatim under a `body` key (the F2 "body via serde, don't re-case" rule — the body's
snake_case field keys and PascalCase enum-unit values are preserved exactly). The RBAC `caller`
field (H3, §8) is present **only** on the 7 mutating methods.

| method | mutating? | `args` shape |
| --- | --- | --- |
| `createDocument` | **yes** | `{"caller":"…","wikiId":"<opaque>","body":<CreateDocumentRequest serde>}` |
| `getDocument` | no | `{"documentId":"<opaque>"}` |
| `updateDocument` | **yes** | `{"caller":"…","documentId":"<opaque>","body":<UpdateDocumentRequest serde>}` |
| `deleteDocument` | **yes** | `{"caller":"…","documentId":"<opaque>"}` |
| `publishDocument` | **yes** | `{"caller":"…","documentId":"<opaque>"}` |
| `unpublishDocument` | **yes** | `{"caller":"…","documentId":"<opaque>"}` |
| `archiveDocument` | **yes** | `{"caller":"…","documentId":"<opaque>"}` |
| `listDocuments` | no | `{"wikiId":"<opaque>","body":<ListDocumentsFilter serde>}` |
| `createWiki` | **yes** | `{"caller":"…","name":"<wiki name>"}` |
| `getWiki` | no | `{"wikiId":"<opaque>"}` |
| `listWikis` | no | `{}` |

**Serde-frozen body shapes (embedded under `body`, unchanged from the store types' serde
output):**
- `CreateDocumentRequest` → `{"title":"…","tags":[…],"author":"…"}` (snake_case keys; `tags`/`author`
  are `Option` — always present, `null` when `None`).
- `UpdateDocumentRequest` → `{"base_revision":<u64>,"graph":{…},"title":"…","tags":[…]}` (snake_case;
  `graph` is the `Graph` serde body `{"nodes":[…],"edges":[…]}`; `title`/`tags` are `Option`).
- `ListDocumentsFilter` → `{"state":<DocState|null>,"tag":<string|null>,"page":<u64|null>,"page_size":<u64|null>}`
  (snake_case; `DocState` serializes PascalCase `"Draft"`/`"Published"`/`"Archived"`; `null` when `None`).

**RBAC `caller` field (H3).** The `caller` field is a **string** carrying the caller's
edit-authority credential (an opaque application-level credential; the engine is the RBAC
enforcer). It is present on the 7 mutating request args and **absent** on the 4 read-only request
args (`getDocument`, `listDocuments`, `getWiki`, `listWikis`). It is distinct from transport auth
(token/TLS, GUI-only). P1a pins the **shape** (a string field named `caller` on mutating request
args); the credential's value semantics are engine-enforced and out of scope.

### 4.3 The CRUD response envelope (the response discriminator + result/error body)

The response `payload` is a wire-defined object with a camelCase `"method"` discriminator + either
a `"result"` field (the serde-frozen result body) or an `"error"` field (the `{"code","message"}`
non-chunk codec body):

```json
{"method":"<camelCase method>","result":<serde-frozen result body>}
```
or
```json
{"method":"<camelCase method>","error":{"code":"<wire_code>","message":"<Display text>"}}
```

**Per-method result shapes (the `"result"` field):**

| method | result type | `"result"` serde body |
| --- | --- | --- |
| `createDocument` | `Document` | the `Document` serde body (snake_case; `state` PascalCase; `graph` nested; `author` `null` when `None`) |
| `getDocument` | `Document` | the `Document` serde body |
| `updateDocument` | `Document` | the `Document` serde body |
| `deleteDocument` | `()` (void) | `null` |
| `publishDocument` | `Document` | the `Document` serde body |
| `unpublishDocument` | `Document` | the `Document` serde body |
| `archiveDocument` | `Document` | the `Document` serde body |
| `listDocuments` | `DocumentList` | the `DocumentList` serde body `{"items":[…],"total":<u64>,"page":<u64>,"page_size":<u64>}` |
| `createWiki` | `Wiki` | the `Wiki` serde body `{"wiki_id":"<opaque>","name":"…"}` |
| `getWiki` | `Wiki` | the `Wiki` serde body |
| `listWikis` | `Vec<Wiki>` | a JSON array of `Wiki` serde bodies |

**`Document` serde body (the §4.1.1 shape, unchanged):**
```json
{"document_id":"<opaque>","wiki_id":"<opaque>","revision":<u64>,"state":"Draft|Published|Archived","graph":{"nodes":[…],"edges":[…] },"title":"…","created_at":"<ISO-8601>","updated_at":"<ISO-8601>","tags":[…],"author":<string|null>}
```

**`CrudResult` enum (the typed result surface):**
```rust
pub enum CrudResult {
    Document(Document),      // createDocument / getDocument / updateDocument / publishDocument / unpublishDocument / archiveDocument
    DeleteDocument,          // deleteDocument (void)
    DocumentList(DocumentList), // listDocuments
    Wiki(Wiki),              // createWiki / getWiki
    WikiList(Vec<Wiki>),     // listWikis
}
```

**`CrudMethod` enum (the typed discriminator):**
```rust
pub enum CrudMethod {
    CreateDocument, GetDocument, UpdateDocument, DeleteDocument,
    PublishDocument, UnpublishDocument, ArchiveDocument,
    ListDocuments, CreateWiki, GetWiki, ListWikis,
}
```
Serialized as the camelCase strings of §4.2.

**`CrudRequestArgs` enum (the typed request-args surface):**
```rust
pub enum CrudRequestArgs {
    CreateDocument { caller: String, wiki_id: WikiId, body: CreateDocumentRequest },
    GetDocument { document_id: DocumentId },
    UpdateDocument { caller: String, document_id: DocumentId, body: UpdateDocumentRequest },
    DeleteDocument { caller: String, document_id: DocumentId },
    PublishDocument { caller: String, document_id: DocumentId },
    UnpublishDocument { caller: String, document_id: DocumentId },
    ArchiveDocument { caller: String, document_id: DocumentId },
    ListDocuments { wiki_id: WikiId, body: ListDocumentsFilter },
    CreateWiki { caller: String, name: String },
    GetWiki { wiki_id: WikiId },
    ListWikis,
}
```
Each variant serializes to the flat `args` object of §4.2 (the implementer uses serde with
`#[serde(flatten)]` on the `body` field or an equivalent so the serde-frozen body keys are
preserved verbatim; the wire shape is what the golden vectors pin).

### 4.4 Canonical error JSON (reused unchanged from F2 §4.3)

`{"code":"<wire_code>","message":"<Display text>"}` — the non-chunk codec body, embedded under
the response `"error"` field. `message` = `format!("{}", e)`; for `ValidationError(m)` this is the
carried detail `m`. The `wire_code()` table and `from_wire` reverse lookup are **unchanged** (F2
§5).

### 4.5 SSE for the CRUD surface (explicit note)

**CRUD is request/response — there is NO SSE surface for the 11 CRUD methods.** The F2 SSE event
schema (`event: result|done|error`, `sse.rs`) stays **retrieval-only** (the §4.6.1 `ragStream`
trio). P1a adds no SSE framing, no `SseEventType` variants, and no CRUD event types. A TestWriter
must assert that the CRUD codecs never touch `sse.rs` and that the SSE schema is unchanged.

---

## 5. The §5.x Property register (MANDATORY — PBT gate)

This register is the contract for the **property-based-testing gate** on the P1a CRUD wire unit.
The TestWriter's executed layer runs under `cargo test` with a **deterministic pinned seed**,
**≤100 generated cases per register row**, **≤400 total cases** across the unit's whole property
layer, **stop-after-5** (report ≤5 distinct held/broken counterexamples per row), and records each
row as **held** or **broken** together with its `Strategy-id`. The adversarial reviewer then reads
this register with the executed artifacts and performs a **read-only** PBT audit; reviewers never
run generators.

Authoring rule: `Property-id` = `P-<CLASS>-<N>`, `CLASS ∈ {IM, SM, TP}`, **≤ 8 rows**; invariants
are **crisp, universally-quantified, black-box-observable through the crate's public wire surface
only**, and are **TRUE of the current GREEN implementation** (implied by the contract — they are
**not** pending features). **There are no fail-state rows** (no `§6`/`FS-*` rows) and **no
gap/parked rows** — invariants only.

**Reserved-variant discipline applied.** The CRUD codecs are **total over the closed method set**
(11 `CrudMethod` variants, all encodable) and the closed result set (5 `CrudResult` variants). The
only generator restriction is **well-formedness on the decode side**: never generate a
`CrudResult` whose body violates a CRUD-specific invariant (e.g. a `createDocument` result with
`revision != 0`), a non-`"gnosis"`-style foreign body, an unknown `schema_version`/`id_format`, or
a malformed request/response — those are `DecodeError`/`CrudResponseError` fail-states (the
auditor's negative-generator territory), **not** invariant rows. The wire unit has **no** reserved
fail-variant rows (no `FS-*` rows) by the invariant-only rule.

## Property table

| Property-id | Class | Invariant | Strategy-id | Observable-as-property |
|---|---|---|---|---|
| `P-IM-1` | IM | **CRUD request round-trip identity.** For **any** `(method, args)` pair over the 11 `CrudMethod` variants with well-formed per-method `args` (the serde-frozen bodies built per §4.2), decoding the envelope that `encode_crud_request` produced returns the identical `(method, args)` — the `method` discriminator and every `args` field (incl. the RBAC `caller` on mutating methods and the serde-frozen `body` verbatim) round-trip element-wise. | `strat:crud-request-roundtrip` | ∀ generated `(method, args)`: `decode_crud_request(&encode_crud_request(method, args)).unwrap().eq(&(method, args))` — `method` `==`, `args` element-wise `==` (caller, ids, and the full serde-frozen body). |
| `P-IM-2` | IM | **CRUD response round-trip identity.** For **any** `(method, result)` pair where `result` is a **well-formed** `CrudResult` for that method (satisfying the CRUD-specific invariants of §4.3 — e.g. `createDocument` → `Document` with `revision == 0` and `state == Draft`), decoding the envelope that `encode_crud_response` produced returns the identical `CrudResult`; the `method` discriminator and the serde-frozen result body round-trip element-wise. | `strat:crud-response-roundtrip` | ∀ well-formed `(method, result)`: `decode_crud_response(&encode_crud_response(method, result)).unwrap().eq(&result)` — the `CrudResult` variant and every field (ids, revision, state, graph, title, timestamps, tags, author, list items/total/page/page_size, wiki name) `==`. |
| `P-IM-3` | IM | **CRUD error round-trip.** For **any** `(method, e)` pair over the 11 methods and the full 21-variant `StoreError` taxonomy, decoding the envelope that `encode_crud_error` produced returns `Err(CrudResponseError::Store(e))` — the carried `StoreError` (incl. `ValidationError(m)` preserving `m`) round-trips through the non-chunk codec. | `strat:crud-error-roundtrip` | ∀ `(method, e)`: `decode_crud_response(&encode_crud_error(method, e))` matches `Err(CrudResponseError::Store(e2))` with `e2 == e` (for `ValidationError`, `matches!(.., StoreError::ValidationError(m2) if m2 == m)`). |
| `P-IM-4` | IM | **CRUD method discriminator uniqueness + non-empty.** The 11 `CrudMethod` variants map to **11 pairwise-distinct non-empty camelCase strings**, and the mapping is a deterministic pure function of the variant (the same variant always yields the same string; the same string never names two variants). | `strat:crud-method-unique` | ∀ distinct `a,b: CrudMethod`: `a.method_str() != b.method_str()`; `!a.method_str().is_empty()`; `a.method_str() == a.method_str()` on repeat and for equal variants. |
| `P-SM-1` | SM | **Envelope round-trip preserves `schema_version` + `id_format` + payload identity for CRUD envelopes.** For **any** CRUD request/response/error envelope the codecs build, serializing and re-parsing (`to_json`/`from_json`) preserves the three fields; `schema_version` is `CURRENT_SCHEMA_VERSION` (=1) and `id_format` is `ID_FORMAT_OPAQUE_STRING_V1` on every encoder output. | `strat:crud-envelope-stable` | ∀ CRUD envelope `env` (from `encode_crud_request`/`encode_crud_response`/`encode_crud_error`): `from_json(&to_json(&env)?).unwrap().eq(&env)` (payload `Value` `==`-equal); `env.schema_version == CURRENT_SCHEMA_VERSION`; `env.id_format == ID_FORMAT_OPAQUE_STRING_V1`. |
| `P-SM-2` | SM | **RBAC `caller` credential survives the request round-trip on mutating methods.** For **any** mutating `(method, args)` (the 7 mutating methods), the decoded args' `caller` **equals** the encoded args' `caller`; the 4 read-only methods carry **no** `caller` field. | `strat:crud-caller-preserved` | ∀ mutating `(method, args)`: `decode_crud_request(&encode_crud_request(method, args)).unwrap().1` has `caller == args.caller`; ∀ read-only `(method, args)`: the decoded args have no `caller` (the `args` object has no `"caller"` key). |
| `P-SM-3` | SM | **Endpoint-path uniqueness + bijection.** The 11 pinned endpoint paths (§7) are **pairwise distinct** and each maps to **exactly one** `CrudMethod`; the `ENGINE_ENDPOINTS` table is a bijection between the 11 paths and the 11 methods. | `strat:crud-endpoint-unique` | ∀ distinct `(path_a, method_a), (path_b, method_b)` in `ENGINE_ENDPOINTS`: `path_a != path_b`; `method_a != method_b`; the table has exactly 11 rows, one per `CrudMethod`. |
| `P-TP-1` | TP | **Encoder never emits a CRUD response body the decoder rejects (decode-then-validate total on well-formed input).** For **any** well-formed `(method, result)`, `encode_crud_response` produces an envelope whose `payload` is accepted by `decode_crud_response` **and** passes the CRUD-specific validation; the codec path never surfaces `EngineError`/`CrudResponseError::Decode`/`CrudResponseError::Validation` on an encoder-produced body. | `strat:crud-encode-validates` | ∀ well-formed `(method, result)`: `decode_crud_response(&encode_crud_response(method, result)).is_ok()`; equivalently `validate_crud_result(method, &result).is_ok()` for every encoder input. |

**Class tally:** IM ×4, SM ×3, TP ×1 = **8 rows ≤ 8** ✔.

## Generator-coverage note (per row — boundary + adversarial input shapes)

- **`P-IM-1 — strat:crud-request-roundtrip`.** Generate one `args` per `CrudMethod` variant (all 11). For the body-carrying methods (`createDocument`, `updateDocument`, `listDocuments`), vary the serde-frozen body: `title`/`name` over {empty, single-char, multi-codepoint UTF-8, 200/100-char boundary}; `tags` over {empty, one, many, `None`}; `author` over {`None`, `Some`}; `base_revision` over {0, 1, u64::MAX}; `graph` over {empty nodes/edges, one node, many nodes/edges}; `state` over all three `DocState` values; `page`/`page_size` over {1, 100, `None`}. For the id-carrying methods, vary the opaque id strings (empty, UUID-shaped, arbitrary). For the mutating methods, vary `caller` over {empty, `"user:alice"`, multi-codepoint UTF-8}. Boundary: `listWikis` with `args == {}` (empty object).
- **`P-IM-2 — strat:crud-response-roundtrip`.** The same body corpus as `P-IM-1` plus: a `Document` with `revision` at 0 and non-zero; `state` over all three `DocState` values; `created_at`/`updated_at` over ISO-8601 strings; a `DocumentList` with `items` over {empty, one, many} and `total`/`page`/`page_size` consistent; a `Wiki` with `name` over the boundary; a `WikiList` over {empty, one, many}. **Do not** generate a `CrudResult` that violates a CRUD-specific invariant (e.g. `createDocument` → `Document` with `revision != 0` or `state != Draft`) — those are `CrudResponseError::Validation` fail-reasons, excluded (validator-rejected).
- **`P-IM-3 — strat:crud-error-roundtrip`.** Enumerate all 21 `StoreError` variants (incl. `ValidationError` with empty, single-byte-UTF8, and multi-codepoint `m`) across a representative subset of the 11 methods; assert `decode_crud_response(&encode_crud_error(method, e))` returns `Err(CrudResponseError::Store(e2))` with `e2 == e`. Boundary: `ValidationError` with a message that is itself JSON-object-shaped (must not be reinterpreted as structure).
- **`P-IM-4 — strat:crud-method-unique`.** Enumerate all 11 `CrudMethod` variants in a fixed order plus several random permutations; for each pairwise-distinct pair assert different strings; assert `!method_str().is_empty()` and `method_str().is_ascii()`. No cross-variant collisions allowed.
- **`P-SM-1 — strat:crud-envelope-stable`.** Across `encode_crud_request` (all 11 methods), `encode_crud_response`, and `encode_crud_error` outputs: `from_json(&to_json(&env)).unwrap() == env`; every field asserted (`schema_version==1`, `id_format=="opaque-string-v1"`). Boundary: payload values that contain nested objects/arrays with both keys and non-ASCII content (envelope `Value` equality is deep). This row does **not** exercise unknown `schema_version`/`id_format` — those are decode fail-states, excluded.
- **`P-SM-2 — strat:crud-caller-preserved`.** For each of the 7 mutating methods, vary `caller` over {empty, `"user:alice"`, multi-codepoint UTF-8, a string containing `"caller"`/`"body"`}; assert the decoded `caller` `==` the encoded `caller`. For each of the 4 read-only methods, assert the decoded `args` object has no `"caller"` key.
- **`P-SM-3 — strat:crud-endpoint-unique`.** Enumerate the 11 `ENGINE_ENDPOINTS` rows; assert pairwise-distinct paths and pairwise-distinct methods; assert the table has exactly 11 rows, one per `CrudMethod` (a bijection).
- **`P-TP-1 — strat:crud-encode-validates`.** Same well-formed `(method, result)` corpus as `P-IM-2`; for each, assert `decode_crud_response(&encode_crud_response(method, result)).is_ok()` **and** `validate_crud_result(method, &result).is_ok()`. This closes the encode→decode→validate totality loop so no encoder output ever trips `CrudResponseError::Decode`/`CrudResponseError::Validation`.

## API notes for the TestWriter

- **Pure/synchronous surface.** All P1a CRUD wire fns are synchronous pure fns over values —
  `#[test]`, no tokio runtime, no engine instance, no store call. Build `CrudMethod`/
  `CrudRequestArgs`/`CrudResult`/`Document`/`Wiki`/`DocumentList`/`CreateDocumentRequest`/
  `UpdateDocumentRequest`/`ListDocumentsFilter`/`StoreError` values directly from the public
  `src/store/mod.rs` types re-exported in `src/lib.rs`.
- **Wire API (from this contract).** `crud::{ CrudMethod, CrudRequestArgs, CrudResult,
  CrudResponseError, CrudValidationFailure, encode_crud_request, decode_crud_request,
  encode_crud_response, encode_crud_error, decode_crud_response, validate_crud_result,
  ENGINE_ENDPOINTS, ENDPOINT_* }`; reused F2 `envelope::{ Envelope, current_schema_version,
  ID_FORMAT_OPAQUE_STRING_V1 }`; `error::{ StoreError::wire_code, StoreError::from_wire }`.
- **`eq` vs structural identity.** `decode_*(&encode_*(&x)).unwrap() == x` uses the types'
  `PartialEq` (derived on `CrudMethod`, `CrudRequestArgs`, `CrudResult`, `Document`, `Wiki`,
  `DocumentList`, `Envelope`). For `ValidationError` use `matches!(.., StoreError::ValidationError(m2)
  if m2 == m)`.
- **Reserved-exclusion note for the generator.** The CRUD codecs are total over the closed method
  and result sets, so there are **no** reserved wire variants; the only generator restriction is
  **well-formedness on the decode side**: never generate a `CrudResult` that violates a
  CRUD-specific invariant, an unknown `schema_version`/`id_format`, or a malformed
  request/response — those are `DecodeError`/`CrudResponseError` fail-states (the auditor's
  negative-generator requests), **not** invariant rows.
- **Determinism/seeding.** Use one deterministic pinned seed per binary, ≤100 cases per row,
  ≤400 total; stop-after-5; report each row held/broken with its `Strategy-id`.

---

## 6. The error codec + §11 map are UNCHANGED (21 rows) — qualified to the `StoreError` taxonomy (NEW-2)

### 6.1 The `wire_code()` table and §11 HTTP-status map are reused verbatim

The F2 `StoreError::wire_code()` table (21 variants, `docs/specs/engine-wire-contract.md` §5) and
the §11 HTTP-status map (21 rows) are **unchanged**. P1a asserts **exhaustiveness**: every
document-CRUD fail-state is a `StoreError` variant already present in the §11 map. The
CRUD-reachable `StoreError` variants are a **subset** of the 21:

| CRUD method | reachable `StoreError` variants | §11 status |
| --- | --- | --- |
| `createDocument` | `WikiNotFound`, `ValidationError` | 404, 400 |
| `getDocument` | `DocumentNotFound` | 404 |
| `updateDocument` | `DocumentNotFound`, `ValidationError`, `ConflictError` | 404, 400, **409** |
| `deleteDocument` | `DocumentNotFound`, `DocumentInUse` | 404, 409 |
| `publishDocument` | `DocumentNotFound`, `UnresolvedReference` | 404, 422 |
| `unpublishDocument` | `DocumentNotFound`, `InvalidState` | 404, 409 |
| `archiveDocument` | `DocumentNotFound`, `InvalidState` | 404, 409 |
| `listDocuments` | `WikiNotFound`, `ValidationError` | 404, 400 |
| `createWiki` | `ValidationError` | 400 |
| `getWiki` | `WikiNotFound` | 404 |
| `listWikis` | — (no fail-state) | — |

The 7 CRUD-reachable variants (`DocumentNotFound`, `WikiNotFound`, `ValidationError`,
`ConflictError`, `DocumentInUse`, `InvalidState`, `UnresolvedReference`) are all in the §11 map.
**No new §11 row is added.** The `wire_code()`/`from_wire`/`code_table` surface is unchanged.

### 6.2 NEW-2 — the server-side request-decode outcome (transport-level, NOT a `StoreError`)

A **malformed CRUD request** or an **unknown method** is **NOT a `StoreError` variant** and has
**NO §11 row**. P1a defines the **server-side request-decode outcome** as a **transport-level
status** (rendered by P2), **NOT 502**:

| `decode_crud_request` error | transport status | rationale |
| --- | --- | --- |
| `DecodeError::InvalidJson(_)` (unparseable JSON) | **400** | malformed request body |
| `DecodeError::InvalidEnvelope(_)` (missing `method`/`args`, wrong types) | **400** | malformed request body |
| `DecodeError::UnknownMethod(_)` (unknown `"method"` value) | **422** | well-formed JSON, unrecognized method (unprocessable) |
| `DecodeError::UnsupportedSchemaVersion(_)` | **400** | envelope-level, request-decode path |
| `DecodeError::UnknownIdFormat(_)` | **400** | envelope-level, request-decode path |

**Qualification of the "no new statuses" claim.** The "no new statuses" claim is qualified to the
**`StoreError` taxonomy**: the §11 map (21 rows) covers every `StoreError` fail-state and is
unchanged. The request-decode outcome (400/422) is a **transport-level status**, not a `StoreError`
wire code, and therefore does **not** add a §11 row. This is the NEW-2 distinction: a malformed
CRUD request / unknown method is a **client error (4xx)**, never a `StoreError`-mapped 502.

---

## 7. REST endpoint-path ownership (H4)

P1a pins the document-CRUD endpoint paths so P2 (server) and A1 (client) consume the **same**
paths. The pinned MVP paths become the shared `ENGINE_ENDPOINTS`-style constants consumed by both
P2 and A1:

| `CrudMethod` | pinned endpoint path | HTTP verb |
| --- | --- | --- |
| `CreateDocument` | `POST /documents` | POST |
| `GetDocument` | `GET /documents/:id` | GET |
| `UpdateDocument` | `POST /documents/:id/update` | POST |
| `DeleteDocument` | `DELETE /documents/:id` | DELETE |
| `PublishDocument` | `POST /documents/:id/publish` | POST |
| `UnpublishDocument` | `POST /documents/:id/unpublish` | POST |
| `ArchiveDocument` | `POST /documents/:id/archive` | POST |
| `ListDocuments` | `GET /documents` | GET |
| `CreateWiki` | `POST /wikis` | POST |
| `GetWiki` | `GET /wikis/:id` | GET |
| `ListWikis` | `GET /wikis` | GET |

**`deleteDocument` path decision.** The exact `deleteDocument` path is the wire contract's
decision (the shell does NOT invent it). P1a pins it as **`DELETE /documents/:id`** (the RESTful
DELETE verb on the document resource).

**Constants.** P1a pins both a table constant and individual constants:

```rust
pub const ENGINE_ENDPOINTS: &[(&str, CrudMethod)] = &[
    ("POST /documents", CrudMethod::CreateDocument),
    ("GET /documents/:id", CrudMethod::GetDocument),
    ("POST /documents/:id/update", CrudMethod::UpdateDocument),
    ("DELETE /documents/:id", CrudMethod::DeleteDocument),
    ("POST /documents/:id/publish", CrudMethod::PublishDocument),
    ("POST /documents/:id/unpublish", CrudMethod::UnpublishDocument),
    ("POST /documents/:id/archive", CrudMethod::ArchiveDocument),
    ("GET /documents", CrudMethod::ListDocuments),
    ("POST /wikis", CrudMethod::CreateWiki),
    ("GET /wikis/:id", CrudMethod::GetWiki),
    ("GET /wikis", CrudMethod::ListWikis),
];

pub const ENDPOINT_CREATE_DOCUMENT: &str = "POST /documents";
pub const ENDPOINT_GET_DOCUMENT: &str = "GET /documents/:id";
pub const ENDPOINT_UPDATE_DOCUMENT: &str = "POST /documents/:id/update";
pub const ENDPOINT_DELETE_DOCUMENT: &str = "DELETE /documents/:id";
pub const ENDPOINT_PUBLISH_DOCUMENT: &str = "POST /documents/:id/publish";
pub const ENDPOINT_UNPUBLISH_DOCUMENT: &str = "POST /documents/:id/unpublish";
pub const ENDPOINT_ARCHIVE_DOCUMENT: &str = "POST /documents/:id/archive";
pub const ENDPOINT_LIST_DOCUMENTS: &str = "GET /documents";
pub const ENDPOINT_CREATE_WIKI: &str = "POST /wikis";
pub const ENDPOINT_GET_WIKI: &str = "GET /wikis/:id";
pub const ENDPOINT_LIST_WIKIS: &str = "GET /wikis";
```

The 11 paths are **pairwise distinct** and the table is a **bijection** between the 11 paths and
the 11 methods (the §5.x `P-SM-3` row).

---

## 8. RBAC credential shape (H3)

Gnosis accepts a credential on the **mutating** CRUD calls to confirm whether the caller has edit
access (the engine is the RBAC enforcer). — RECONCILED 2026-09-22 (`RBAC-DOC-DRIFT`): the engine pins
the credential **SHAPE** + its decode-layer **presence check** only; the authority mapping and the
deny are **SHELL**-side — see `GNOSIS-RBAC-EDIT-ENFORCEMENT` / `docs/HANDOFF.md` §8-RBAC. The
historical clause stands as written. P1a pins the RBAC credential shape on the mutating
request envelopes: a **`caller` field** (camelCase) on the 7 mutating request `args` objects,
carrying the caller's edit-authority credential as an **opaque string** (§4.2). This is distinct
from the transport auth (token/TLS, GUI-only).

- **Mutating methods (require `caller`):** `createDocument`, `updateDocument`, `deleteDocument`,
  `publishDocument`, `unpublishDocument`, `archiveDocument`, `createWiki`.
- **Read-only methods (do NOT require `caller`):** `getDocument`, `listDocuments`, `getWiki`,
  `listWikis`.

The `caller` field is a **string**; its value semantics (which credential grants edit authority)
are engine-enforced and out of P1a's scope. — RECONCILED 2026-09-22 (`RBAC-DOC-DRIFT`): *"engine-enforced"*
is the pre-re-scope wording; the engine pins the credential **SHAPE** + its decode-layer **presence
check** only (a missing `caller` on the 7 mutating methods ⇒ 400, `p1a` §10), and the
credential's **value semantics live in the SHELL's authority mapping** — the engine holds none and
never passes `caller` to the store. See `GNOSIS-RBAC-EDIT-ENFORCEMENT` / `docs/HANDOFF.md` §8-RBAC.
The historical clause stands as written. P1a pins only the **shape** (a `caller` string field on
the mutating request args, absent on the read-only request args). The §5.x `P-SM-2` row asserts the
`caller` survives the request round-trip on mutating methods and is absent on read-only methods.

---

## 9. Golden conformance vectors (V-10+, following F2 §12)

Exact bytes/JSON a conforming implementation (the engine P1a codecs and the later P2 server / A1
client) must reproduce. All envelopes use `schemaVersion:1`, `idFormat:"opaque-string-v1"`.

**V-10 — `encode_crud_request(CreateDocument, …)`** (a representative createDocument request
envelope; `caller` present because createDocument is mutating):
```
{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"method":"createDocument","args":{"caller":"user:alice","wikiId":"w1","body":{"title":"Getting Started","tags":["guide"],"author":"alice"}}}}
```

**V-11 — `encode_crud_response(CreateDocument, CrudResult::Document(…))`** (a representative
createDocument response envelope; the `Document` body via serde — snake_case keys, `state`
PascalCase `"Draft"`, `graph` nested, `revision:0`):
```
{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"method":"createDocument","result":{"document_id":"d1","wiki_id":"w1","revision":0,"state":"Draft","graph":{"nodes":[],"edges":[]},"title":"Getting Started","created_at":"2026-09-09T00:00:00Z","updated_at":"2026-09-09T00:00:00Z","tags":["guide"],"author":"alice"}}}
```

**V-12 — `encode_crud_error(UpdateDocument, &StoreError::ConflictError)`** (a CRUD error envelope;
`ConflictError` → `"conflict"` → §11 HTTP 409):
```
{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"method":"updateDocument","error":{"code":"conflict","message":"optimistic-concurrency conflict: stale base revision"}}}
```

**V-13 — round-trip identity samples across the document method families** (the TestWriter
asserts `decode_crud_request(&encode_crud_request(m, a)) == (m, a)` and
`decode_crud_response(&encode_crud_response(m, r)) == r` for representative values of each family):
- **Document family** (`createDocument`/`getDocument`/`updateDocument`/`publishDocument`/
  `unpublishDocument`/`archiveDocument`): a `Document` with `revision:0`, `state:"Draft"`, empty
  graph, `author:null` round-trips byte-for-byte.
- **List family** (`listDocuments`): a `DocumentList` with `items:[…]`, `total`, `page:1`,
  `page_size:20` round-trips.
- **Wiki family** (`createWiki`/`getWiki`/`listWikis`): a `Wiki` `{"wiki_id":"w1","name":"My Wiki"}`
  and a `WikiList` round-trip.
- **Void family** (`deleteDocument`): `CrudResult::DeleteDocument` encodes to `"result":null` and
  decodes back to `DeleteDocument`.

**V-14 — the request-decode outcome (NEW-2) mapping** (transport-level, NOT a `StoreError`):
- A request envelope with `"method":"bogus"` → `decode_crud_request` → `Err(DecodeError::UnknownMethod("bogus"))` → P2 renders **422**.
- A request envelope with a missing `"args"` → `Err(DecodeError::InvalidEnvelope(_))` → P2 renders **400**.
- A request envelope with `schemaVersion:99` → `Err(DecodeError::UnsupportedSchemaVersion(99))` → P2 renders **400**.
- None of these is a `StoreError`; none has a §11 row.

---

## 10. Valid/happy + fail states per function (TestWriter assertion guide)

| wire fn | valid/happy | fail-state |
| --- | --- | --- |
| `encode_crud_request(method, args)` | any `(method, args)` → request envelope with canonical JSON (serde-frozen body verbatim) | none (serialize can't fail on our payloads) |
| `decode_crud_request(env)` | round-trips all 11 methods | `InvalidJson` (malformed JSON), `InvalidEnvelope` (missing `method`/`args`), `UnknownMethod` (unknown `"method"`), `UnsupportedSchemaVersion`, `UnknownIdFormat` |
| `encode_crud_response(method, result)` | any well-formed `(method, result)` → response envelope with the serde result body | none |
| `encode_crud_error(method, err)` | any `(method, err)` → response envelope with `{"code","message"}` | none |
| `decode_crud_response(env)` | well-formed result body, method matches, CRUD invariants hold → `Ok(result)` | `CrudResponseError::Store(e)` (carried error), `CrudResponseError::Decode(_)` (malformed body → `EngineError`/502), `CrudResponseError::Validation(_)` (well-formed body failing a CRUD invariant → `EngineError`/502) |
| `validate_crud_result(method, result)` | valid result → `Ok(())` | `CrudValidationFailure::UnexpectedRevision`, `UnexpectedState`, `InvalidPagination`, `UnexpectedVoid` |
| `ENGINE_ENDPOINTS` / `ENDPOINT_*` | the 11 pinned paths, pairwise distinct, bijective with the 11 methods | none (pure constants) |

**CRUD-specific validation invariants (`validate_crud_result` / `CrudValidationFailure`):**

```rust
pub enum CrudValidationFailure {
    /// createDocument must yield revision == 0.
    UnexpectedRevision { expected: u64, actual: u64 },
    /// publish/unpublish/archive must yield the documented state.
    UnexpectedState { expected: DocState, actual: DocState },
    /// listDocuments must yield page >= 1, 1 <= page_size <= 100.
    InvalidPagination { page: u64, page_size: u64 },
    /// deleteDocument must yield void (null result).
    UnexpectedVoid,
}
```

| method | validation invariant |
| --- | --- |
| `createDocument` | `Document.revision == 0` and `Document.state == Draft` |
| `getDocument` | none (any well-formed `Document`) |
| `updateDocument` | none (any well-formed `Document`; the `revision + 1` check is a caller-side concern against the request's `base_revision`) |
| `deleteDocument` | the result must be void (`CrudResult::DeleteDocument`) |
| `publishDocument` | `Document.state == Published` |
| `unpublishDocument` | `Document.state == Draft` |
| `archiveDocument` | `Document.state == Archived` |
| `listDocuments` | `DocumentList.page >= 1`, `1 <= page_size <= 100` |
| `createWiki` | none (any well-formed `Wiki`) |
| `getWiki` | none (any well-formed `Wiki`) |
| `listWikis` | none (any well-formed `Vec<Wiki>`) |

**Cross-cutting TestWriter states:**
- **Unknown `id_format`:** decode of a CRUD envelope with `id_format:"uuid-v4"` →
  `UnknownIdFormat` → request-decode 400 (request) / `CrudResponseError::Decode` → `EngineError`/502 (response).
- **Unknown `schema_version`:** decode of `schema_version:99` → `UnsupportedSchemaVersion(99)` →
  request-decode 400 (request) / `CrudResponseError::Decode` → `EngineError`/502 (response).
- **Empty `payload`:** `InvalidEnvelope` for a CRUD request/response with a missing/empty payload.
- **Method/result mismatch:** a response whose `"method"` does not match the `CrudResult` variant
  (e.g. `"method":"getWiki"` with a `Document` result) → `CrudResponseError::Decode` (the decoder
  decodes the result to the variant the method implies; a mismatch is a malformed body).
- **RBAC:** a mutating request with no `caller` field is a **request-decode 400** (the `caller` is
  required on mutating methods); a read-only request with a `caller` field is tolerated (the field
  is ignored) — P1a pins the shape, not the enforcement.

---

## 11. Cross-references and ownership hand-off

- **Contract authority:** `docs/specs/gnosis.md` §4.1.3, §4.1.1, §4.1.4, §4.4.3, §4.4.5, §6.
- **Base wire layer:** `docs/specs/engine-wire-contract.md` (F2 — the `Envelope`, the error codec,
  the §11 map, decode-then-validate, the golden vectors V-1..V-9, the SSE framing).
- **Register format precedent:** `docs/specs/7-2-wire-property-register.md` (the F2 register this
  unit's §5.x register mirrors).
- **PBT:** the §5.x register (authored HERE) + the TestWriter's executed property layer.
- **Conformance tests (TestWriter):** `tests/crud_wire_conformance.rs` (or an extension of
  `tests/wire_conformance.rs`).

**Owned by a later unit (not P1a, per §2):**
1. The HTTP server + status rendering (P2) — consumes `ENGINE_ENDPOINTS` + the §11 map + the
   request-decode outcome table (§6.2).
2. The shell-side client (A1) — consumes the same paths + shapes.
3. The graph/fact/consistency/RAG-companion wire shapes (P1b–P1e — deferred follow-ons).
4. The RBAC **enforcement** semantics (the engine is the enforcer; P1a pins only the credential shape) —
   RECONCILED 2026-09-22 (`RBAC-DOC-DRIFT`): the engine pins the credential **SHAPE** + its
   decode-layer **presence check** only; the authority mapping and the deny are **SHELL**-side — see
   `GNOSIS-RBAC-EDIT-ENFORCEMENT` / `docs/HANDOFF.md` §8-RBAC. The historical clause stands as written.
5. Bind-loopback + auth/TLS policy (recorded shell-owned, as in F2).

---

## 12. What the TestWriter derives (red-set readiness)

- `crud_wire_conformance.rs`: §4.2/§4.3 wire shapes for all 11 methods; §6.1 exhaustiveness (the
  CRUD-reachable `StoreError` variants are a subset of the 21 §11 rows); §6.2 the request-decode
  outcome table (400/422, NOT 502); §7 the endpoint-path constants (11 rows, bijective); §8 the
  RBAC `caller` shape (present on mutating, absent on read-only); §9 golden
  vectors V-10..V-12 byte-for-byte (V-13 round-trip identity, V-14 the
  request-decode outcome); §10 valid/happy + fail states per function (incl. all
  `CrudResponseError` routes and all `CrudValidationFailure` variants).
- **Byte-for-byte golden assertions.** V-10..V-12 pin the **exact string** each codec must emit —
  the raw `serde_json::to_string` output on the given input value (no re-formatting). They cover
  the wire-defined camelCase surfaces (`method`, `args`, `caller`, `wikiId`, `documentId`, `name`,
  `body`, `result`, `error`) and the serde-frozen bodies (`CreateDocumentRequest`,
  `UpdateDocumentRequest`, `ListDocumentsFilter`, `Document`, `DocumentList`, `Wiki` — snake_case
  keys, PascalCase enums, id newtypes as strings, `null` for absent `Option` fields). The red tests
  assert equality **against these exact bytes**.
- `props_crud_wire.rs`: the 8 invariant rows of the §5.x register (FILE 2 asserts decoded-value
  `eq` against the input, **not** byte identity — it is independent of the literal JSON in FILE 1).
- The red set is derived **from this contract alone**; the implementer lands the least `src/wire/`
  code to green.

---

## 13. What the spec does NOT do (constraints honored)

- This is a **TDD wire unit spec** — it includes the §5.x Property register (PBT gate) but does
  **NOT** author the property tests (the TestWriter does) and does **NOT** author implementation.
- It does **NOT** touch `src/` or `tests/` — it writes **only** the spec file.
- It does **NOT** change the F2 error codec or the §11 map (21 rows) — it reuses them verbatim and
  qualifies the "no new statuses" claim to the `StoreError` taxonomy (NEW-2).
- It does **NOT** add an SSE surface for CRUD (CRUD is request/response; the SSE event schema stays
  retrieval-only).
- It does **NOT** change the frozen §4.1 store types — the CRUD body shapes reuse their serde
  output verbatim ("body via serde, don't re-case").
