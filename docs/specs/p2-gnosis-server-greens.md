# §7.2 P2 — `gnosis-server` binary crate — BLIND-GREENS SET

- **Author-role:** blind_test_writer.
- **Source:** `docs/specs/p2-gnosis-server.md` ONLY (the P2 behavior contract). No
  implementation (`src/bin/gnosis_server.rs`, `src/server.rs`, or any `src/` file) was read.
- **Gate:** blind-greens + live-scenario after the greens (Auspicion Suite process gate 5).
- **Method:** each scenario states the expected behavior in observable terms (the exact
  endpoint, the exact status, the exact response envelope), derived from the spec's §4
  (loopback bind), §5 (the 14 endpoints), §6 (READY lifecycle), §7 (§11 status rendering +
  NEW-2 request-decode outcome), §8 (RBAC caller threading), §9 (e2e transport), and §10
  (valid/happy + fail states). The scenarios are then RUN against the actual build via
  `tests/blind_p2_gnosis_server_greens.rs` (the pure server fns `gnosis::server_status`,
  `gnosis::request_decode_status`, `gnosis::route_bijection`, plus the `gnosis-server` bin
  for the e2e legs).

---

## S1 — §4 Loopback bind: the server binds to `127.0.0.1` and serves.

The server binds to **`127.0.0.1:<port>`** (loopback-only by contract; §4). Observable: the
`gnosis-server` bin started with `--port <port>` becomes reachable at
`http://127.0.0.1:<port>` and answers `GET /engine/status` with HTTP 200. A non-loopback bind
is a fail-state (not exercised here; the shell owns bind policy).

## S2 — §5.2 The routing table is a bijection of exactly 14 rows.

`route_bijection()` returns exactly **14** rows (11 CRUD + 3 retrieval), one per endpoint
(§5.2 routing contract, P-IM-3). Observable: `route_bijection().len() == 14`; the 14 paths are
pairwise distinct; the 14 handlers are pairwise distinct.

## S3 — §5.2 The 11 CRUD paths equal the P1a `ENGINE_ENDPOINTS` verbatim.

The server does NOT invent or re-derive the CRUD paths (§5.2). Observable: the routing table
contains exactly the 11 `ENGINE_ENDPOINTS` paths verbatim, and the 3 retrieval paths
`POST /rag/query`, `GET /rag/stream`, `GET /engine/status` are present.

## S4 — §6 READY lifecycle: `GET /engine/status` returns a `HealthReport` (always 200).

After boot, `GET /engine/status` returns a `HealthReport` JSON carrying `state`, `version`,
`subsystems`, and `lastError`; the endpoint is always 200 (§6, §10). Observable: HTTP 200 and
the JSON has `state` and `subsystems` keys.

## S5 — §7.1 The 7 CRUD-reachable `StoreError` variants map to their exact §11 status + wire code.

`server_status(e)` is `Some((status, code))` for each of the 7 CRUD-reachable variants, with
the exact pinned values (§7.1 table): `DocumentNotFound`→(404,`not_found`),
`WikiNotFound`→(404,`wiki_not_found`), `ValidationError`→(400,`validation_error`),
`ConflictError`→(409,`conflict`), `DocumentInUse`→(409,`doc_in_use`),
`InvalidState`→(409,`invalid_state`), `UnresolvedReference`→(422,`unresolved_reference`).

## S6 — §7.1 The retrieval-trio reachable variants map per the full §11 table.

`server_status(e)` maps the retrieval-trio variants to their exact §11 status (§7.1):
`EngineUnavailable`→503, `EngineError`→502, `TraceUnavailable`→502, `EmbeddingUnavailable`→503,
`VectorIndexUnavailable`→503, `LexicalIndexUnavailable`→503, `RerankerUnavailable`→503,
`CompressionFailed`→500, `HyDEGenerationFailed`→500, `MultiQueryExpansionFailed`→500,
`SubTaskDagFailed`→500.

## S7 — §7.2 NEW-2: the 5 request-decode `DecodeError` variants map to 400/422, never 502.

`request_decode_status(e)` is `Some(status)` for each of the 5 `DecodeError` variants with the
exact pinned transport status (§7.2 table): `InvalidJson`→400, `InvalidEnvelope`→400,
`UnknownMethod`→422, `UnsupportedSchemaVersion`→400, `UnknownIdFormat`→400. None is 502.

## S8 — §8 RBAC caller threading: mutating requests carry `caller`; read-only do not.

The 7 mutating methods (`createDocument`, `updateDocument`, `deleteDocument`,
`publishDocument`, `unpublishDocument`, `archiveDocument`, `createWiki`) require `caller`;
the 4 read-only (`getDocument`, `listDocuments`, `getWiki`, `listWikis`) do not (§8).
Observable: the encoded mutating request envelopes carry a string `caller` on `args`; the
encoded read-only envelopes carry no `caller`; a mutating request with the `caller` removed
fails `decode_crud_request` (request-decode 400); a read-only request with an added `caller`
is accepted (tolerated).

## S9 — §9 e2e: a `rag_query` while the engine is not READY → `EngineUnavailable` → 503.

Over the live transport, `POST /rag/query` on a fresh (not-READY) server returns HTTP **503**
(FS-8, §9). Observable: the live response status is 503.

## S10 — §9 e2e: `createDocument` request envelope → `Document` response envelope.

`POST /documents` with a well-formed `createDocument` envelope (after pre-creating the wiki)
returns HTTP **200** and a response envelope that decodes to `CrudResult::Document` (§9, §10).

## S11 — §9 e2e: a `ConflictError` → HTTP 409.

A stale `base_revision` update over the live transport returns HTTP **409** (§9, §10).

## S12 — §9 e2e: a malformed request → HTTP 400.

An unparseable JSON body to `POST /documents` returns HTTP **400** (§9, §10, NEW-2).

## S13 — §9 e2e: an unknown method → HTTP 422.

A well-formed JSON envelope with an unrecognized `"method"` to `POST /documents` returns HTTP
**422** (§9, §10, NEW-2).

## S14 — §9 e2e: the remaining CRUD round-trips.

Each of `getDocument`, `deleteDocument`, `publishDocument`, `unpublishDocument`,
`archiveDocument`, `listDocuments`, `createWiki`, `getWiki`, `listWikis` round-trips its
request envelope → response envelope (or the §11-mapped error) over the live transport with
HTTP 200 (§9, §10).

## S15 — §10 per-endpoint fail-state status mapping.

Each endpoint's documented fail-state `StoreError` maps to its exact §11 status (§10 table):
`POST /documents`→`WikiNotFound`404/`ValidationError`400; `GET /documents/:id`→404;
`POST /documents/:id/update`→404/400/409; `DELETE /documents/:id`→404/409;
`POST /documents/:id/publish`→404/422; `POST /documents/:id/unpublish`→404/409;
`POST /documents/:id/archive`→404/409; `GET /documents`→404/400; `POST /wikis`→400;
`GET /wikis/:id`→404; `POST /rag/query`→503/502/502.

---

**Scenario count: 15 (S1–S15).** Run: `tests/blind_p2_gnosis_server_greens.rs`.
