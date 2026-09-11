# §7.2 P1a — Document-CRUD wire contract — BLIND-GREENS set

- **Role:** blind_test_writer (docs-only). Derived from
  `docs/specs/p1a-document-crud-wire.md` (§4 wire shapes, §6 error codec + NEW-2
  request-decode outcome, §7 endpoint paths, §8 RBAC caller, §9 golden vectors
  V-10..V-14, §10 valid/happy + fail states) **ALONE**. The implementation
  (`src/wire/crud.rs` and all `src/`) was **NOT** read.
- **Executed against:** the landed P1a build via a NEW blind-greens test file
  `tests/blind_p1a_crud_wire_greens.rs` (assertions derived from the spec's
  expected behavior, not from the implementation) plus the existing
  `cargo test --test crud_wire_conformance --test props_crud_wire` binaries.
- **Date:** 2026-09-10.

Each scenario states the expected behavior in observable terms (the exact wire
JSON, the exact error outcome, the exact status). A scenario is **GREEN** only if
the live build reproduces the spec's expected behavior; a contradiction is a
**finding** (doc/spec drift or an un-hardened regression).

---

## S1 — §4.1/§4.2 request envelope: versioned envelope + camelCase `"method"` + `"args"` for all 11 methods

For **every** one of the 11 `CrudMethod` variants, `encode_crud_request(m, args)`
returns an `Envelope` whose `schema_version == CURRENT_SCHEMA_VERSION` (=1),
`id_format == ID_FORMAT_OPAQUE_STRING_V1` (=`"opaque-string-v1"`), and whose
`payload` is an object carrying exactly a `"method"` string equal to the §4.2
camelCase value and an `"args"` object.

Expected `"method"` values (pairwise distinct, non-empty, camelCase):
`createDocument`, `getDocument`, `updateDocument`, `deleteDocument`,
`publishDocument`, `unpublishDocument`, `archiveDocument`, `listDocuments`,
`createWiki`, `getWiki`, `listWikis`.

## S2 — §4.2 per-method `args` wire JSON (camelCase top-level, serde-frozen body verbatim)

The `args` object for each method uses **camelCase top-level fields** and embeds
the serde-frozen store body **verbatim** under `body` (snake_case keys preserved;
`null` for absent `Option`). Representative exact `args` JSON:

- `createDocument` → `{"caller":"user:alice","wikiId":"w1","body":{"title":"Getting Started","tags":["guide"],"author":"alice"}}`
- `getDocument` → `{"documentId":"d1"}`
- `updateDocument` → `{"caller":"user:alice","documentId":"d1","body":{"base_revision":0,"graph":{"nodes":[],"edges":[]},"title":"Getting Started","tags":["guide"]}}`
- `deleteDocument` → `{"caller":"user:alice","documentId":"d1"}`
- `publishDocument` → `{"caller":"user:alice","documentId":"d1"}`
- `unpublishDocument` → `{"caller":"user:alice","documentId":"d1"}`
- `archiveDocument` → `{"caller":"user:alice","documentId":"d1"}`
- `listDocuments` → `{"wikiId":"w1","body":{"state":"Draft","tag":null,"page":1,"page_size":20}}`
- `createWiki` → `{"caller":"user:alice","name":"My Wiki"}`
- `getWiki` → `{"wikiId":"w1"}`
- `listWikis` → `{}`

## S3 — §4.3 response envelope: camelCase `"method"` + `"result"` for all 11; `deleteDocument` void → `"result":null`

For **every** method, `encode_crud_response(m, result)` returns an `Envelope`
whose `payload` carries a `"method"` string equal to the §4.2 camelCase value and
a `"result"` field. For `deleteDocument` the `"result"` is exactly JSON `null`
(the void result).

## S4 — §4.3 result serde bodies (snake_case, PascalCase enums, `null` for absent `Option`)

The `"result"` field is the serde-frozen body. Representative exact JSON:

- `Document` (createDocument/getDocument/updateDocument/publishDocument/
  unpublishDocument/archiveDocument) →
  `{"document_id":"d1","wiki_id":"w1","revision":0,"state":"Draft","graph":{"nodes":[],"edges":[]},"title":"Getting Started","created_at":"2026-09-09T00:00:00Z","updated_at":"2026-09-09T00:00:00Z","tags":["guide"],"author":"alice"}`
  — snake_case keys, `state` PascalCase `"Draft"`, `graph` nested, `author`
  `null` when `None`.
- `DocumentList` (listDocuments) →
  `{"items":[…],"total":<u64>,"page":<u64>,"page_size":<u64>}`.
- `Wiki` (createWiki/getWiki) → `{"wiki_id":"w1","name":"My Wiki"}`.
- `WikiList` (listWikis) → a JSON array of `Wiki` bodies.

## S5 — §4.4 error envelope: `{"code","message"}` under `"error"`; `ConflictError` → `"conflict"`

`encode_crud_error(m, &e)` returns an `Envelope` whose `payload` carries the
`"method"` discriminator and an `"error"` object with exactly `"code"` (the
`wire_code()`) and `"message"` (the `Display` text). For
`StoreError::ConflictError` the code is `"conflict"` (→ §11 HTTP 409).

## S6 — §6.1 exhaustiveness: the 7 CRUD-reachable `StoreError` variants are a subset of the 21 §11 rows

`code_table()` has **exactly 21 rows**. Each of the 7 CRUD-reachable variants
(`DocumentNotFound`, `WikiNotFound`, `ValidationError`, `ConflictError`,
`DocumentInUse`, `InvalidState`, `UnresolvedReference`) has a `wire_code()`
present in the table. No new §11 row is added.

## S7 — §6.2 NEW-2 request-decode outcome (transport-level 400/422, NOT 502, NOT a `StoreError`)

`decode_crud_request` maps malformed/unknown requests to `DecodeError` variants
(transport-level statuses, never a `StoreError`-mapped 502, no §11 row):

- `"method":"bogus"` → `Err(DecodeError::UnknownMethod("bogus"))` → **422**.
- missing `"args"` → `Err(DecodeError::InvalidEnvelope(_))` → **400**.
- `schemaVersion:99` → `Err(DecodeError::UnsupportedSchemaVersion(99))` → **400**.
- `id_format:"uuid-v4"` → `Err(DecodeError::UnknownIdFormat(_))` → **400**.
- non-object payload → `Err(DecodeError::InvalidEnvelope(_))` → **400**.

## S8 — §7 endpoint-path ownership (H4): 11 rows, pairwise distinct, bijective

`ENGINE_ENDPOINTS` has **exactly 11 rows**, one per `CrudMethod`, with pairwise
distinct paths and pairwise distinct methods (a bijection). Each `ENDPOINT_*`
constant equals the pinned path:

`POST /documents`, `GET /documents/:id`, `POST /documents/:id/update`,
`DELETE /documents/:id`, `POST /documents/:id/publish`,
`POST /documents/:id/unpublish`, `POST /documents/:id/archive`,
`GET /documents`, `POST /wikis`, `GET /wikis/:id`, `GET /wikis`.

## S9 — §8 RBAC `caller` shape (H3): present on the 7 mutating, absent on the 4 read-only

The 7 mutating request args (`createDocument`, `updateDocument`, `deleteDocument`,
`publishDocument`, `unpublishDocument`, `archiveDocument`, `createWiki`) carry a
`caller` **string** field. The 4 read-only request args (`getDocument`,
`listDocuments`, `getWiki`, `listWikis`) carry **no** `caller` field.

## S10 — §9 V-10 golden vector (byte-for-byte)

`encode_crud_request(CreateDocument, …)` with `caller:"user:alice"`,
`wikiId:"w1"`, body `{"title":"Getting Started","tags":["guide"],"author":"alice"}`
must emit **exactly**:

```
{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"method":"createDocument","args":{"caller":"user:alice","wikiId":"w1","body":{"title":"Getting Started","tags":["guide"],"author":"alice"}}}}
```

## S11 — §9 V-11 golden vector (byte-for-byte)

`encode_crud_response(CreateDocument, CrudResult::Document(…))` with the §4.3
`Document` body must emit **exactly**:

```
{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"method":"createDocument","result":{"document_id":"d1","wiki_id":"w1","revision":0,"state":"Draft","graph":{"nodes":[],"edges":[]},"title":"Getting Started","created_at":"2026-09-09T00:00:00Z","updated_at":"2026-09-09T00:00:00Z","tags":["guide"],"author":"alice"}}}
```

## S12 — §9 V-12 golden vector (byte-for-byte)

`encode_crud_error(UpdateDocument, &StoreError::ConflictError)` must emit
**exactly**:

```
{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"method":"updateDocument","error":{"code":"conflict","message":"optimistic-concurrency conflict: stale base revision"}}}
```

## S13 — §9 V-13 round-trip identity across the four method families

- **Document family** (`createDocument`/`getDocument`/`updateDocument`/
  `publishDocument`/`unpublishDocument`/`archiveDocument`): a `Document` with
  `revision:0`, `state:"Draft"`, empty graph, `author:null` round-trips
  (`decode_crud_response(&encode_crud_response(m, r)) == r`).
- **List family** (`listDocuments`): a `DocumentList` with `items:[…]`, `total`,
  `page:1`, `page_size:20` round-trips.
- **Wiki family** (`createWiki`/`getWiki`/`listWikis`): a `Wiki`
  `{"wiki_id":"w1","name":"My Wiki"}` and a `WikiList` round-trip.
- **Void family** (`deleteDocument`): `CrudResult::DeleteDocument` encodes to
  `"result":null` and decodes back to `DeleteDocument`.

## S14 — §9 V-14 request-decode outcome (NEW-2) mapping

- `"method":"bogus"` → `Err(DecodeError::UnknownMethod("bogus"))` → **422**.
- missing `"args"` → `Err(DecodeError::InvalidEnvelope(_))` → **400**.
- `schemaVersion:99` → `Err(DecodeError::UnsupportedSchemaVersion(99))` → **400**.
- None of these is a `StoreError`; none has a §11 row.

## S15 — §10 valid/happy: request + response round-trip for all 11 methods

- `decode_crud_request(&encode_crud_request(m, args)) == (m, args)` for all 11.
- `decode_crud_response(&encode_crud_response(m, result)) == result` for all 11
  (well-formed results satisfying the CRUD invariants).

## S16 — §10 fail-states: `decode_crud_response` routes

- Error envelope → `Err(CrudResponseError::Store(e))` (carried `StoreError`).
- Method/result mismatch (e.g. `"method":"getWiki"` with a `Document` result) →
  `Err(CrudResponseError::Decode(_))` (malformed body → `EngineError`/502).
- Response with `schemaVersion:99` → `Err(CrudResponseError::Decode(_))`.
- Response with `id_format:"uuid-v4"` → `Err(CrudResponseError::Decode(_))`.
- Response error envelope with unknown `code` → `Err(CrudResponseError::Decode(UnknownCode))`.
- `createDocument` result with `revision != 0` →
  `Err(CrudResponseError::Validation(_))` (well-formed body failing a CRUD
  invariant → `EngineError`/502).

## S17 — §10 `validate_crud_result` invariants

- Happy path: `Ok(())` for all 11 well-formed results.
- `createDocument` with `revision != 0` → `UnexpectedRevision { expected:0, actual }`.
- `publishDocument` with `state != Published` → `UnexpectedState { expected:Published, actual }`.
- `unpublishDocument` with `state != Draft` → `UnexpectedState { expected:Draft, actual }`.
- `archiveDocument` with `state != Archived` → `UnexpectedState { expected:Archived, actual }`.
- `listDocuments` with `page == 0` or `page_size == 0` or `page_size > 100` →
  `InvalidPagination { page, page_size }`.
- `deleteDocument` with a non-void result → `UnexpectedVoid`.
- `getDocument`/`updateDocument`/`createWiki`/`getWiki`/`listWikis` accept any
  well-formed result of the right variant (no invariant).

## S18 — §10 cross-cutting request-decode states

- A mutating request with **no** `caller` field → `Err(DecodeError::InvalidEnvelope(_))`
  (request-decode **400**).
- A read-only request with a `caller` field is **tolerated/ignored** (decodes to
  the read-only args).
- `listWikis` with non-empty `args` → `Err(DecodeError::InvalidEnvelope(_))`.

## S19 — §4.5 SSE is unchanged and the CRUD codecs never touch `sse.rs`

The SSE event schema stays retrieval-only: exactly the 3 F2 `SseEventType`
variants (`result`/`done`/`error`), no CRUD event types. No CRUD request/response/
error envelope payload carries SSE framing keys (`event`/`data`/`type`).

---

## Execution record

| Scenario | Spec source | Result |
|---|---|---|
| S1 | §4.1/§4.2 | **GREEN** |
| S2 | §4.2 | **GREEN** |
| S3 | §4.3 | **GREEN** |
| S4 | §4.3 | **GREEN** |
| S5 | §4.4 | **GREEN** |
| S6 | §6.1 | **GREEN** |
| S7 | §6.2 | **GREEN** |
| S8 | §7 | **GREEN** |
| S9 | §8 | **GREEN** |
| S10 | §9 V-10 | **GREEN** |
| S11 | §9 V-11 | **GREEN** |
| S12 | §9 V-12 | **GREEN** |
| S13 | §9 V-13 | **GREEN** |
| S14 | §9 V-14 | **GREEN** |
| S15 | §10 | **GREEN** |
| S16 | §10 | **GREEN** |
| S17 | §10 | **GREEN** |
| S18 | §10 | **GREEN** |
| S19 | §4.5 | **GREEN** |

**Aggregate: 19/19 scenarios GREEN (0 FAILED).** Executed via
`tests/blind_p1a_crud_wire_greens.rs` (21 `#[test]` = S1..S19 + 1 extra S8
endpoint-constant assertion + 1 supporting envelope-constant assertion) — all
pass — plus the existing
`cargo test --test crud_wire_conformance --test props_crud_wire` binaries
(35 + 8 pass). No scenario contradicted the spec; no finding.
