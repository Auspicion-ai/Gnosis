# §7.2 P1a — Document-CRUD wire contract — LIVE-SCENARIO PENDING BATTERY

- **Unit:** §7.2 P1a — document-CRUD wire-contract unit (the 11 §4.1 request/response wire shapes).
- **Date:** 2026-09-10.
- **Author-role:** live-scenario-runner.
- **Status:** **PENDING** (parked — **not** a gate failure).
- **Source set transcribed:** `docs/specs/p1a-document-crud-wire-greens.md` (19 scenarios S1–S19, all **GREEN**) → the live actions below.

## 1. Why this is a PENDING battery

P1a is a **wire-contract unit** — it freezes the 11 §4.1 document-CRUD request/response
wire shapes (pure codecs + endpoint constants + RBAC caller shape). It is **not a server
and not a client** (`docs/specs/p1a-document-crud-wire.md` §1, §2 "NOT in scope": a hosted
HTTP or native-IPC server → deferred to **P2**; the shell-side client → deferred to **A1**).

The live surface for the CRUD wire shapes requires two things, of which **one now exists**:

1. the **`gnosis-server` binary crate (P2)** — **LANDED (2026-09-10)**: the server host that
   serves the document-CRUD endpoints over loopback (`127.0.0.1`), and
2. the **Astrographer-side CRUD routing client (A1)** — **still deferred** (the only remaining
   park condition).

Both were deferred follow-ons in the approved roadmap
(`docs/integrations/astrographer-interface-implementation.md` §4.6: "The full `RagStore`
CRUD routing (§4.1–§4.4) is **deferred** to a later unit, once the engine freezes CRUD
wire shapes — the shell must not invent them unilaterally"; §2: the server host is a
"separate thin binary crate in the Gnosis workspace (e.g. `gnosis-server`, a `[[bin]]`
depending on the engine lib + axum/hyper/tower)").

**Current repo state (verified 2026-09-10):** `Cargo.toml` now declares the `gnosis`
(`src/main.rs` — a boot scaffold that prints a message), `gnosis-eval`, and
**`gnosis-server`** (`src/bin/gnosis_server.rs` + `src/server.rs`, axum in `Cargo.toml`)
binaries. The `gnosis-server` bin exists and serves the document-CRUD endpoints on
loopback `127.0.0.1` (confirmed by the P2 live battery's 15/15 server-endpoint scenarios
PASS). The Astrographer CRUD routing client is **not** wired. The wire codecs themselves
(`src/wire/crud.rs`) are **pure functions** — synchronous, no tokio, no I/O, no engine
instance — and are already verified by the blind-greens run.

Consequently **no live-scenario can currently be executed**: there is no running engine
process serving the document-CRUD endpoints over a wire, no HTTP endpoint to issue
`POST /documents` / `GET /documents` against, and no Astrographer CRUD client to drive.
Every live-scenario derivable from the greens set is therefore **parked** here for a later
iteration of this runner. **Parked scenarios are NOT a failure** — the gate outcome for
P1a is PENDING, with the in-repo blind-greens (19/19 + 35 + 8) as the current evidence base.

### REVISIT CONDITION

This battery resumes (the gate may be re-run live) when the following holds:

1. the **Astrographer CRUD routing client (A1)** is wired to consume the same paths + shapes.
   (The **`gnosis-server` binary crate (P2)** now exists and builds — `src/bin/gnosis_server.rs`
   + `src/server.rs`, axum in `Cargo.toml` — and a **live Gnosis engine** runs behind it serving
   the document-CRUD endpoints on loopback `127.0.0.1`, confirmed by the P2 live battery's
   15/15 server-endpoint scenarios PASS. So the P2 + live-engine conditions are met; only A1
   remains.)

**The live check that ends the park** (both must pass):

- `curl http://127.0.0.1:<port>/documents` returns a **`DocumentList`** envelope, AND
- `POST /documents` with a `createDocument` request envelope returns a **`Document`** envelope.

Until then this battery records the exact scenarios to execute at that point. No engine
code (`src/`) or test code (`tests/`) is modified by this battery.

## 2. The live scenarios (a later runner executes these against the live transport)

Each row: the scenario (faithful transcription of the corresponding greens scenario —
**no new behavior invented**), the exact action a live consumer performs, and the
EXPECTED observable (the exact wire bytes/status/outcome pinned by the contract
`docs/specs/p1a-document-crud-wire.md` §4/§6/§7/§8/§9 golden vectors + the greens set).

**All 19 scenarios are live-runnable now that P2 exists** — the wire shapes cross the wire
via the `gnosis-server` host (confirmed live by the P2 battery's 15/15 server-endpoint
scenarios PASS); the remaining park is the Astrographer-side client (A1). Each is mapped to
a live scenario id (L1–L19) that exercises the same behavior over a real HTTP transport.

### 2.1 Request envelope + args over HTTP (S1, S2)

| Scenario | Live action | Expected observable |
| --- | --- | --- |
| **L1** (← S1) — versioned request envelope + camelCase `"method"` + `"args"` for all 11 methods | live consumer issues each of the 11 CRUD calls over HTTP (e.g. `POST /documents`, `GET /documents/:id`, `POST /wikis`, `GET /wikis`) | every request arrives wrapped in `{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"method":"<camelCase>","args":{…}}}`; the 11 `"method"` values are pairwise-distinct, non-empty, camelCase (`createDocument`, `getDocument`, `updateDocument`, `deleteDocument`, `publishDocument`, `unpublishDocument`, `archiveDocument`, `listDocuments`, `createWiki`, `getWiki`, `listWikis`) |
| **L2** (← S2) — per-method `args` wire JSON (camelCase top-level, serde-frozen body verbatim) | consumer sends each method's args and inspects the decoded request | `args` uses camelCase top-level fields (`caller`, `wikiId`, `documentId`, `name`, `body`) with the serde-frozen store body embedded verbatim under `body` (snake_case keys preserved, `null` for absent `Option`); e.g. `createDocument` → `{"caller":"user:alice","wikiId":"w1","body":{"title":"Getting Started","tags":["guide"],"author":"alice"}}` |

### 2.2 Response envelope + result bodies over HTTP (S3, S4)

| Scenario | Live action | Expected observable |
| --- | --- | --- |
| **L3** (← S3) — response envelope: camelCase `"method"` + `"result"` for all 11; `deleteDocument` void → `"result":null` | consumer issues each of the 11 CRUD calls and inspects the response | every response payload carries `"method":"<camelCase>"` + `"result"`; `deleteDocument` returns `"result":null` (the void result) |
| **L4** (← S4) — result serde bodies (snake_case, PascalCase enums, `null` for absent `Option`) | consumer inspects each response's `"result"` body | `Document` → `{"document_id":…,"wiki_id":…,"revision":…,"state":"Draft|Published|Archived","graph":{"nodes":[…],"edges":[…]},…,"author":<string|null>}` (snake_case, `state` PascalCase, `author` `null` when `None`); `DocumentList` → `{"items":[…],"total":…,"page":…,"page_size":…}`; `Wiki` → `{"wiki_id":…,"name":…}`; `WikiList` → a JSON array of `Wiki` bodies |

### 2.3 Error envelope + §11 status over HTTP (S5, S6)

| Scenario | Live action | Expected observable |
| --- | --- | --- |
| **L5** (← S5) — error envelope `{"code","message"}` under `"error"`; `ConflictError` → `"conflict"` | consumer drives a `ConflictError` (e.g. a stale-base-revision `updateDocument`) | response payload carries `"method"` + `"error":{"code":"conflict","message":"optimistic-concurrency conflict: stale base revision"}`; HTTP status → **409** |
| **L6** (← S6) — §6.1 exhaustiveness: the 7 CRUD-reachable `StoreError` variants map to §11 statuses over HTTP | consumer drives each reachable fail-state (where live-drivable) and asserts the rendered status | `DocumentNotFound` → 404, `WikiNotFound` → 404, `ValidationError` → 400, `ConflictError` → 409, `DocumentInUse` → 409, `InvalidState` → 409, `UnresolvedReference` → 422 — each a §11 row; no new status/code appears from the engine |

### 2.4 NEW-2 request-decode outcome over HTTP (S7, S14)

| Scenario | Live action | Expected observable |
| --- | --- | --- |
| **L7** (← S7) — NEW-2 request-decode outcome (transport-level 400/422, NOT 502, NOT a `StoreError`) | consumer hand-sends malformed/unknown requests | `"method":"bogus"` → **422**; missing `"args"` → **400**; `schemaVersion:99` → **400**; `id_format:"uuid-v4"` → **400**; non-object payload → **400**. None is a `StoreError`-mapped 502; none has a §11 row |
| **L14** (← S14) — V-14 request-decode outcome mapping | consumer hand-sends the V-14 cases | `"method":"bogus"` → **422**; missing `"args"` → **400**; `schemaVersion:99` → **400**; none is a `StoreError`; none has a §11 row |

### 2.5 Endpoint-path ownership + RBAC caller over HTTP (S8, S9)

| Scenario | Live action | Expected observable |
| --- | --- | --- |
| **L8** (← S8) — §7 endpoint-path ownership (H4): 11 rows, pairwise distinct, bijective | consumer issues each of the 11 pinned paths and asserts each routes to exactly one method | `POST /documents`, `GET /documents/:id`, `POST /documents/:id/update`, `DELETE /documents/:id`, `POST /documents/:id/publish`, `POST /documents/:id/unpublish`, `POST /documents/:id/archive`, `GET /documents`, `POST /wikis`, `GET /wikis/:id`, `GET /wikis` — all live-routable, pairwise distinct, bijective with the 11 methods |
| **L9** (← S9) — §8 RBAC `caller` shape (H3): present on the 7 mutating, absent on the 4 read-only | consumer issues a mutating call (e.g. `POST /documents`) and a read-only call (e.g. `GET /documents`) | the 7 mutating request args carry a `caller` **string** field; the 4 read-only request args carry **no** `caller` field; a mutating call with no `caller` → **400** (request-decode) |

### 2.6 Golden vectors over HTTP (S10, S11, S12)

| Scenario | Live action | Expected observable |
| --- | --- | --- |
| **L10** (← S10) — V-10 golden vector (byte-for-byte) | consumer sends the exact V-10 `createDocument` request envelope (`caller:"user:alice"`, `wikiId:"w1"`, body `{"title":"Getting Started","tags":["guide"],"author":"alice"}`) | the server decodes it to `CreateDocument` with those exact args; the request bytes match V-10 exactly |
| **L11** (← S11) — V-11 golden vector (byte-for-byte) | consumer issues `createDocument` and inspects the response | the response `"result"` body is the exact V-11 `Document` body: `{"document_id":"d1","wiki_id":"w1","revision":0,"state":"Draft","graph":{"nodes":[],"edges":[]},"title":"Getting Started","created_at":"2026-09-09T00:00:00Z","updated_at":"2026-09-09T00:00:00Z","tags":["guide"],"author":"alice"}` |
| **L12** (← S12) — V-12 golden vector (byte-for-byte) | consumer drives an `updateDocument` that hits `ConflictError` | HTTP **409** + the exact V-12 error envelope: `{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"method":"updateDocument","error":{"code":"conflict","message":"optimistic-concurrency conflict: stale base revision"}}}` |

### 2.7 Round-trip identity + valid/happy over HTTP (S13, S15)

| Scenario | Live action | Expected observable |
| --- | --- | --- |
| **L13** (← S13) — V-13 round-trip identity across the four method families | consumer round-trips representative values of each family over the wire | **Document family** (`createDocument`/`getDocument`/`updateDocument`/`publishDocument`/`unpublishDocument`/`archiveDocument`): a `Document` with `revision:0`, `state:"Draft"`, empty graph, `author:null` round-trips; **List family** (`listDocuments`): a `DocumentList` with `items`, `total`, `page:1`, `page_size:20` round-trips; **Wiki family** (`createWiki`/`getWiki`/`listWikis`): a `Wiki` `{"wiki_id":"w1","name":"My Wiki"}` and a `WikiList` round-trip; **Void family** (`deleteDocument`): `"result":null` round-trips |
| **L15** (← S15) — §10 valid/happy: request + response round-trip for all 11 methods | consumer issues all 11 CRUD calls with well-formed args and inspects the responses | each request decodes to the sent `(method, args)`; each response decodes to the well-formed `result` satisfying the CRUD invariants |

### 2.8 Fail-state response routes + validation invariants over HTTP (S16, S17)

| Scenario | Live action | Expected observable |
| --- | --- | --- |
| **L16** (← S16) — §10 fail-states: `decode_crud_response` routes | consumer drives or hand-builds the fail-state responses | error envelope → carried `StoreError`; method/result mismatch (e.g. `"method":"getWiki"` with a `Document` result) → `CrudResponseError::Decode` → **502**; response `schemaVersion:99` → **502**; response `id_format:"uuid-v4"` → **502**; error envelope with unknown `code` → **502**; `createDocument` result with `revision != 0` → `CrudResponseError::Validation` → **502** |
| **L17** (← S17) — §10 `validate_crud_result` invariants | consumer drives the invariant-violating results | `createDocument` with `revision != 0` → `UnexpectedRevision`; `publishDocument` with `state != Published` → `UnexpectedState`; `unpublishDocument` with `state != Draft` → `UnexpectedState`; `archiveDocument` with `state != Archived` → `UnexpectedState`; `listDocuments` with `page == 0` or `page_size == 0` or `page_size > 100` → `InvalidPagination`; `deleteDocument` with a non-void result → `UnexpectedVoid` — each surfaces as `EngineError`/502 over HTTP |

### 2.9 Cross-cutting request-decode states + SSE unchanged (S18, S19)

| Scenario | Live action | Expected observable |
| --- | --- | --- |
| **L18** (← S18) — §10 cross-cutting request-decode states | consumer hand-sends the cross-cutting requests | a mutating request with **no** `caller` field → **400** (request-decode); a read-only request with a `caller` field is **tolerated/ignored** (decodes to the read-only args); `listWikis` with non-empty `args` → **400** |
| **L19** (← S19) — §4.5 SSE is unchanged and the CRUD codecs never touch `sse.rs` | consumer verifies the live SSE surface and the CRUD responses | the SSE event schema stays retrieval-only: exactly the 3 F2 `SseEventType` variants (`result`/`done`/`error`), no CRUD event types; no CRUD request/response/error envelope payload carries SSE framing keys (`event`/`data`/`type`) |

## 3. Parity re-check, not new coverage

Every scenario above is **already verified in-repo** by the blind-greens run
(`tests/blind_p1a_crud_wire_greens.rs` — 21 `#[test]` = S1..S19 + 1 extra S8
endpoint-constant assertion + 1 supporting envelope-constant assertion, all
pass) plus the existing
`cargo test --test crud_wire_conformance --test props_crud_wire` binaries (35 + 8 pass)
at the **codec/decode/encode layer** — the exact golden bytes, round-trips, and
decode-then-validate/error/status mappings above are asserted there today. The pending
battery adds **no new behavior and no new test coverage**. Its entire value is
**re-confirming the SAME behavior end-to-end** over a **real HTTP transport with a real
Astrographer CRUD client** — i.e. a **parity re-check** that the encoder/decoder behavior
proven in-repo is identical over an actual `POST /documents` / `GET /documents` round
trip, including the live render of the §11 HTTP-status map and the NEW-2 request-decode
outcome (400/422) that P2 renders.

## 4. How the later runner executes this

Once P2 (the `gnosis-server` binary crate) and A1 (the Astrographer CRUD routing client)
land, this runner will:

1. **Boot the live engine behind the server** — run the `gnosis-server` binary so a live
   Gnosis engine serves the document-CRUD endpoints on loopback (`127.0.0.1`).
2. **Confirm the park is over** — `curl http://127.0.0.1:<port>/documents` returns a
   `DocumentList` envelope AND `POST /documents` with a `createDocument` request envelope
   returns a `Document` envelope.
3. **Issue the CRUD calls over the live transport** — exercise L1..L19, driving the engine
   through the states the scenarios require (create/get/update/delete/publish/unpublish/
   archive/list, wiki CRUD, conflict, validation error, unknown method, malformed request,
   foreign `schema_version`/`id_format`, invariant-violating results).
4. **Record live pass/fail per scenario** — each row above that passes live confirms the
   in-repo parity; a scenario whose **live** result **contradicts the greens/contract** is
   **a finding** (a real regression or a doc/spec drift — a live failure is never a pass).
   Any parked rows whose transport still does not exist are re-parked, not failed.

## 5. Authoritative refs

- `docs/specs/p1a-document-crud-wire.md` — the P1a wire contract (envelope §4.1, request §4.2, response §4.3, error §4.4, SSE §4.5, error codec + §11 map §6, endpoint paths §7, RBAC caller §8, golden vectors V-10..V-14 §9, valid/fail states §10).
- `docs/specs/p1a-document-crud-wire-greens.md` — the blind-greens set (19 scenarios S1–S19, all GREEN) this battery transcribes.
- `docs/integrations/astrographer-interface-implementation.md` — the stated deferral (full CRUD routing deferred to a later unit; the `gnosis-server` binary crate = P2; the shell-side CRUD client = A1).
- `docs/specs/7-2-wire-live-pending-battery.md` — the F2 pending-battery precedent this document mirrors.
- `tests/blind_p1a_crud_wire_greens.rs`, `tests/crud_wire_conformance.rs`, `tests/props_crud_wire.rs` — the in-repo verification layer (19/19 + 35 + 8 green).

This battery modifies **only** this document. No `src/` or `tests/` change. No commit.
