# §7.2 P2 — `gnosis-server` binary crate (the live HTTP/SSE transport host)

- **Unit:** §7.2 P2 — the `gnosis-server` `[[bin]]` crate (roadmap §5.2). The live
  transport host for the F2 retrieval trio + health AND the P1a document-CRUD surface.
- **Status:** **CONTRACT (REALIZED-GREEN — LANDED 2026-09-10).** This is the
  code-bearing TDD unit contract for the server binary. It pins the crate layout, the
  loopback bind, the REST + SSE endpoints, the READY lifecycle, the server-side §11
  status rendering + the NEW-2 request-decode outcome, the RBAC `caller` threading, the
  end-to-end transport test, and the §5.x Property register (MANDATORY, PBT gate).
- **Gate:** proposal-review **PASSED** (roadmap §5.2, decision `GNOSIS-CRUD-SURFACE-CONFIRMED`).
  This spec is the §5.2 P2 deliverable; the TestWriter derives its red set from this
  contract alone, then the Implementer lands the least `src/bin/gnosis_server.rs` code to green.
- **Consumes:** `docs/specs/p1a-document-crud-wire.md` (P1a — the 11 §4.1 wire shapes, the
  `ENGINE_ENDPOINTS`/`ENDPOINT_*` constants, the §6.2 request-decode outcome table 400/422,
  the RBAC `caller` shape, the golden vectors V-10..V-14) and
  `docs/specs/engine-wire-contract.md` (F2 — the retrieval trio + health wire shapes, the
  `Envelope`, the §11 HTTP-status map (21 rows), the SSE framing, the golden vectors V-1..V-9).
- **Contract cross-refs:** `docs/specs/gnosis.md` §4.1.3, §4.1.4, §4.4.3, §4.4.5, §4.6.1, §6;
  `src/store/mod.rs` (`Store`, `DerivedIndexes`, `EmbeddingProvider`, `getEngineStatus`/
  `EngineStatus`/`EngineSubsystems`/`EngineState`, `RagStore`, `StoreError` (21 variants));
  `Cargo.toml` (the current workspace/bin structure).
- **Date:** 2026-09-10. **Author-role:** spec_writer.
- **Scope:** a new `[[bin]]` (`src/bin/gnosis_server.rs`) + a reviewable dependency addition
  scoped to the bin. The engine **lib stays at zero new runtime deps**. Companion PBT
  register: **authored HERE (§5.x)** per the F2 precedent; the TestWriter executes it.

---

## 0. Compile-horizon review (status / what it asks / feasibility / units + gaps / costs-benefits)

- **Status:** CONTRACT (REALIZED-GREEN — LANDED 2026-09-10). The P2 server unit is
  **implemented + green** (`src/bin/gnosis_server.rs` + `src/server.rs` + the re-export
  surface in `src/lib.rs` + the `[[bin]]`/axum in `Cargo.toml`; see the P2 DONE row in
  `docs/next-steps.md`). This is a
  **code-bearing TDD unit** (the §5.x Property register is MANDATORY per the PBT gate).
- **What it asks.** Host the **retrieval trio + health** (`POST /rag/query`, `GET /rag/stream`
  SSE, `GET /engine/status`) AND the **11 §4.1 document-CRUD REST endpoints** in a new thin
  `gnosis-server` `[[bin]]` crate. The server binds loopback-only to `127.0.0.1`, renders the
  §11 HTTP-status map server-side (plus the NEW-2 request-decode outcome 400/422, NOT 502),
  threads the RBAC `caller` through the request-decode layer, runs the READY boot lifecycle, and
  ships an end-to-end transport test against the real server.
- **Feasibility verdict.** **FEASIBLE.** The engine lib already exposes the `RagStore` trait
  (the 11 CRUD methods + `rag_query`/`rag_stream`/`get_engine_status`), the `Store` type, the
  `DerivedIndexes`, the `EmbeddingProvider` seam, and the `StoreError` taxonomy. P1a already
  froze the CRUD wire codecs + the `ENGINE_ENDPOINTS`/`ENDPOINT_*` constants + the request-decode
  outcome table. F2 already froze the retrieval-trio wire + the §11 map + the SSE framing. P2
  adds only the thin transport layer: axum/hyper/tower routing, the loopback bind, the
  status-rendering glue, and the READY boot wiring. **The engine lib stays at zero new runtime
  deps** — the server framework is scoped to the bin.
- **The units + gaps.** **In scope:** the `[[bin]]` crate layout; the loopback bind; the 14
  REST/SSE endpoints (3 retrieval + 11 CRUD); the READY lifecycle; the server-side §11 status
  rendering + the NEW-2 request-decode outcome; the RBAC `caller` threading; the end-to-end
  transport test; the §5.x Property register. **Explicitly deferred (gaps for later units, NOT
  this one):** bind/auth/TLS policy (shell-owned, as in F2/P1a); the shell-side client (A1);
  the graph/fact/consistency/RAG-companion endpoints (P1b–P1e — deferred follow-ons); the RBAC
  *enforcement* semantics (the engine is the enforcer; P2 only threads the `caller`); RFC-4122
  id adoption (reused unchanged via the `idFormat` seam).
- **Costs-benefits.** **Cost:** one new `src/bin/gnosis_server.rs` + a reviewable dependency
  addition (axum/hyper/tower) scoped to the bin + the server conformance + property + e2e test
  suites. **Benefit:** the live engine endpoint the shell's document-CRUD routing (A1) and the
  live-scenario battery reach; the `EngineUnavailable`/`EngineError` split is proven over a real
  transport; the §5.x register makes the status-mapping + request-decode outcome + endpoint
  routing property-tested (PBT gate) exactly as F2/P1a were.

---

## 1. What P2 asks

P2 is a new thin `[[bin]]` crate (e.g. `gnosis-server`) in the Gnosis workspace, depending on
the engine lib + a server framework (axum/hyper/tower). The **engine lib stays at zero new
runtime deps** — only the bin gains the server framework. In the MVP, P2 hosts the **retrieval
trio + health** AND the **document-CRUD endpoints** (the 11 §4.1 surface). The endpoint paths
are **consumed from P1a (H4)** — the server does NOT invent them. Each CRUD endpoint accepts the
request envelope and returns the response envelope (or the §11-mapped error).

---

## 2. Scope guardrails

**In scope (this contract):**
- A new `[[bin]]` (`src/bin/gnosis_server.rs`) in the Gnosis workspace; the engine lib stays at
  zero new runtime deps; the server framework (axum/hyper/tower) is scoped to the bin.
- Loopback bind to `127.0.0.1` (loopback-only by contract; the shell owns bind/auth/TLS policy).
- The REST + SSE endpoints: the retrieval trio + health (`POST /rag/query`, `GET /rag/stream`
  SSE, `GET /engine/status`) PLUS the 11 document-CRUD REST endpoints (paths from P1a H4).
- The READY lifecycle: `Arc<Store>` → `DerivedIndexes` → embedding provider → `READY`; expose
  `getEngineStatus` (`READY`/`STARTING`/`DEGRADED`/`UNAVAILABLE` + subsystems).
- The server-side §11 HTTP-status rendering (the server maps the returned `StoreError` to the
  HTTP status + wire code) + the NEW-2 request-decode outcome (400/422, NOT 502).
- The RBAC `caller` threading (H3): the server threads the `caller` through the request-decode
  layer (the decode layer validates its presence on mutating requests).
- An end-to-end transport test against the real server.
- The §5.x Property register (MANDATORY, PBT gate).

**NOT in scope (explicit):**
- Bind/auth/TLS policy + loopback enforcement (recorded shell-owned, as in F2/P1a).
- The shell-side client (deferred to A1).
- The graph/fact/consistency/RAG-companion endpoints (P1b–P1e — deferred follow-ons).
- The RBAC **enforcement** semantics (the engine is the enforcer; P2 threads the `caller` only).
- RFC-4122 id adoption (reused unchanged via the `idFormat` seam).
- Any new `StoreError` variants or new §11 rows (the §11 map stays 21 rows; the request-decode
  outcome is a transport-level status, NOT a `StoreError` wire code).
- Zero new runtime deps on the engine **lib** (the bin gains the server framework).

---

## 3. Crate layout

A new `[[bin]]` in the Gnosis workspace. The engine lib (`src/lib.rs`, `src/store/`,
`src/wire/`) is **unchanged** and stays at zero new runtime deps.

| artifact | concern | exported public API |
| --- | --- | --- |
| `src/bin/gnosis_server.rs` (NEW) | the thin transport host | `main()` (bind + serve), the router, the status-rendering glue, the READY boot wiring |
| `Cargo.toml` (extended) | a reviewable dependency addition scoped to the bin | `[dependencies]` gains axum (bin-only; the lib's `[dependencies]` is unchanged) |

**Cargo.toml impact (reviewable, scoped to the bin):**

```toml
[[bin]]
name = "gnosis-server"
path = "src/bin/gnosis_server.rs"

[dependencies]
# …existing lib deps unchanged…
# NEW — scoped to the bin only (the engine lib stays at zero new runtime deps):
axum = "0.7"
```

The `gnosis-server` bin depends on the engine lib (`gnosis`) for the `Store`, `RagStore`,
`StoreError`, `EngineStatus`, `DerivedIndexes`, `EmbeddingProvider`, and the `wire` codecs
(`crud`, `codecs`, `decode`, `sse`, `status`, `envelope`, `error`). The bin introduces
axum (and its transitive hyper/tower) into the Gnosis `Cargo.lock` — a reviewable dependency
addition scoped to the bin, not the lib (roadmap §5.2 cost row).

**axum-in-lib-deps note.** Cargo cannot scope a dependency to a `[[bin]]` — every dependency is
declared at the package level, so `axum` appears in the shared `[dependencies]` (as it does in
the current `Cargo.toml`). The "zero new runtime deps on the lib" claim therefore refers to the
**lib's code**, not its `Cargo.toml`: the engine lib (`src/lib.rs`, `src/store/`, `src/wire/`)
uses none of `axum`; only the `gnosis-server` bin imports it.

---

## 4. Loopback bind

The server binds to **`127.0.0.1`** (loopback-only by contract; the shell owns bind/auth/TLS
policy). A non-loopback bind is a **fail-state**.

- **Valid/happy:** the server binds to `127.0.0.1:<port>` and serves the endpoints.
- **Fail-state:** a bind to any non-loopback address (e.g. `0.0.0.0`, `::`, a routable
  interface IP) is rejected — the server must refuse to start (or the TestWriter asserts the
  bind target is loopback-only). The port is configurable (a CLI arg or env var); the bind
  address is **not** configurable to a non-loopback value.

---

## 5. The REST + SSE endpoints

The server hosts the **retrieval trio + health** endpoints (F2 wire: `POST /rag/query`,
`GET /rag/stream` SSE, `GET /engine/status`) PLUS the **document-CRUD REST endpoints** for the
11 §4.1 methods. **The endpoint paths are consumed from P1a (H4)** — the server does NOT invent
them. Each CRUD endpoint accepts the request envelope and returns the response envelope (or the
§11-mapped error).

### 5.1 The retrieval trio + health (F2 wire)

| endpoint | verb | wire surface | response |
| --- | --- | --- | --- |
| `POST /rag/query` | POST | request envelope → `rag_query` | response envelope (`RagResult` body) or the §11-mapped error |
| `GET /rag/stream` | GET (SSE) | query params → `rag_stream` | SSE frames (`event: result|done|error`, F2 §4.4) |
| `GET /engine/status` | GET | — | `HealthReport` JSON (F2 §9) |

### 5.2 The document-CRUD REST endpoints (P1a H4 paths)

The 11 paths are consumed **verbatim** from P1a §7 (`ENGINE_ENDPOINTS`/`ENDPOINT_*`). Each
accepts the request envelope and returns the response envelope (or the §11-mapped error):

| `CrudMethod` | pinned endpoint path | verb |
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

**Routing contract.** The routing table is a **bijection**: each of the 14 paths (11 CRUD + 3
retrieval) maps to **exactly one** handler, and each handler is reachable by exactly one path
(the §5.x `P-IM-3` row). The server does NOT invent or re-derive the CRUD paths — it consumes
the P1a constants.

---

## 6. The READY lifecycle

The boot wiring is engine-internal; the shell only observes READY.

- **Sequence:** construct `Arc<Store>` → build `DerivedIndexes` → wire the embedding provider →
  transition to `READY`; expose `getEngineStatus` (`READY`/`STARTING`/`DEGRADED`/`UNAVAILABLE` +
  subsystems).
- **Valid/happy:** after boot, `GET /engine/status` returns a `HealthReport` with
  `state:"Ready"` and the subsystem flags reflecting the wired provider.
- **Fail-states:** if the embedding provider is unreachable at boot, the engine transitions to
  `DEGRADED` (non-core subsystem) or `UNAVAILABLE` (engine not running) — the `getEngineStatus`
  report reflects the actual state; the server does NOT fabricate READY. A `rag_query`/
  `rag_stream` call while not READY returns `EngineUnavailable` (FS-8 → §11 503).

---

## 7. The HTTP-status rendering of the §11 map server-side (NEW-2)

The server maps the returned `StoreError` to the HTTP status + wire code, using the §11 map
(21 rows, F2 §11) verbatim. **NEW-2:** the server ALSO renders the **request-decode outcome** (a
malformed request envelope / unknown method → **400/422, NOT 502**) — a transport-level status
distinct from the `StoreError` taxonomy.

### 7.1 The `StoreError` → HTTP status + wire code mapping (server-side)

The server maps each returned `StoreError` to the §11 HTTP status + the `wire_code()` string.
The CRUD-reachable subset (7 variants) is pinned in P1a §6.1:

| `StoreError` | §11 status | wire code |
| --- | --- | --- |
| `DocumentNotFound` | 404 | `"not_found"` |
| `WikiNotFound` | 404 | `"wiki_not_found"` |
| `ValidationError` | 400 | `"validation_error"` |
| `ConflictError` | **409** | `"conflict"` |
| `DocumentInUse` | 409 | `"doc_in_use"` |
| `InvalidState` | 409 | `"invalid_state"` |
| `UnresolvedReference` | 422 | `"unresolved_reference"` |

The retrieval-trio reachable variants (`EngineUnavailable`→503, `EngineError`→502,
`TraceUnavailable`→502, `EmbeddingUnavailable`→503, `VectorIndexUnavailable`→503,
`LexicalIndexUnavailable`→503, `RerankerUnavailable`→503, `CompressionFailed`→500,
`HyDEGenerationFailed`→500, `MultiQueryExpansionFailed`→500, `SubTaskDagFailed`→500) map per the
full §11 table. **No new §11 row is added.**

### 7.2 NEW-2 — the request-decode outcome (transport-level, NOT a `StoreError`)

A **malformed CRUD request** or an **unknown method** is **NOT a `StoreError` variant** and has
**NO §11 row**. The server renders the request-decode outcome as a **transport-level status**,
**NOT 502** (P1a §6.2):

| `decode_crud_request` error | transport status | rationale |
| --- | --- | --- |
| `DecodeError::InvalidJson(_)` | **400** | malformed request body |
| `DecodeError::InvalidEnvelope(_)` | **400** | malformed request body |
| `DecodeError::UnknownMethod(_)` | **422** | well-formed JSON, unrecognized method (unprocessable) |
| `DecodeError::UnsupportedSchemaVersion(_)` | **400** | envelope-level, request-decode path |
| `DecodeError::UnknownIdFormat(_)` | **400** | envelope-level, request-decode path |

**Qualification.** The "no new statuses" claim is qualified to the `StoreError` taxonomy: the
§11 map (21 rows) covers every `StoreError` fail-state and is unchanged. The request-decode
outcome (400/422) is a **transport-level status**, not a `StoreError` wire code, and therefore
does **not** add a §11 row. A malformed CRUD request / unknown method is a **client error
(4xx)**, never a `StoreError`-mapped 502.

---

## 8. The RBAC caller (H3) — re-scoped to the decode layer only

The server threads the `caller` credential through the **request-decode layer only**. The
engine-side RBAC **enforcement** is a PACKAGE item (the engine is the enforcer; P2 does NOT add a
`caller` param to the engine methods). The decode layer validates the `caller`'s presence on the
mutating request envelopes; the server does NOT pass the `caller` to the engine.

- **Mutating methods (require `caller`):** `createDocument`, `updateDocument`, `deleteDocument`,
  `publishDocument`, `unpublishDocument`, `archiveDocument`, `createWiki`.
- **Read-only methods (do NOT require `caller`):** `getDocument`, `listDocuments`, `getWiki`,
  `listWikis`.

**Contract (re-scoped).** The server threads the `caller` through the request decode layer: a
mutating request with no `caller` field is a **request-decode 400** (the `caller` is required on
mutating methods, P1a §10); a read-only request that carries a `caller` is **tolerated** (the
read-only args carry no `caller` field, so an extra one is ignored). The engine-side RBAC
enforcement (a caller without edit access → the engine's RBAC error) is a PACKAGE item, NOT
implemented here. The server does NOT invent or drop the `caller` (the §5.x `P-SM-3` row).

**Engine-side enforcement is deliberately NOT implemented (2026-09-10, proposal gate —
`docs/specs/rbac-caller-review.md`).** The engine's `RagStore` mutating methods take **no `caller`
param** and the engine holds **no authority mapping** (the only "who may edit" mapping is the
shell's boot-time `AuthorityStore`). The C5 boundary classification + the pure-backend framing
place the authorization gate at the **SHELL**: the shell's A2 caller-side deny
(`handleGnosisTool` `resolveCallerCredential` → `Error('<tool>: caller has no edit authority')`,
fail-closed, before any proxy call) **is** the enforcement. Decision `GNOSIS-RBAC-EDIT-ENFORCEMENT`
amended to align with C5 (RBAC → SHELL). Residual risk (accepted): the engine's mutating methods
are callable by any process that can reach the loopback port directly; acceptable under the
loopback-only + local-shell-only trust model. Trigger condition for revisiting: a future non-shell
direct client → mitigation is a caller-aware decorator at the access boundary (Option B), NOT a
trait change.

---

## 9. An end-to-end transport test

Against the real server (the §7.2 F2 benefit): the `EngineUnavailable`/`EngineError` split over
a live transport, plus the document-CRUD round-trips.

- **`EngineUnavailable`/`EngineError` split over a live transport.** A `rag_query`/`rag_stream`
  call while the engine is not READY → `EngineUnavailable` → HTTP 503 (FS-8). A malformed
  result body → `EngineError` → HTTP 502 (FS-9). The two are distinct over the wire.
- **Document-CRUD round-trips.** A `createDocument` request envelope → a `Document` response
  envelope; a `ConflictError` → HTTP 409; a malformed request → HTTP 400; an unknown method →
  HTTP 422. Each CRUD method round-trips its request envelope → response envelope (or the
  §11-mapped error) over the live transport.

---

## 10. Valid/happy + fail states per endpoint (TestWriter assertion guide)

| endpoint | valid/happy | fail-state |
| --- | --- | --- |
| `POST /rag/query` | request envelope → `RagResult` response envelope | `EngineUnavailable`→503, `EngineError`→502, `TraceUnavailable`→502, other §11-mapped errors |
| `GET /rag/stream` | SSE frames (`result`/`done`/`error`) | `EngineUnavailable`→503, `EngineError`→502 (as an `error` SSE frame) |
| `GET /engine/status` | `HealthReport` JSON (`state`, `version`, `subsystems`, `lastError`) | none (always 200) |
| `POST /documents` | `createDocument` request → `Document` response | `WikiNotFound`→404, `ValidationError`→400 |
| `GET /documents/:id` | `getDocument` request → `Document` response | `DocumentNotFound`→404 |
| `POST /documents/:id/update` | `updateDocument` request → `Document` response | `DocumentNotFound`→404, `ValidationError`→400, `ConflictError`→409 |
| `DELETE /documents/:id` | `deleteDocument` request → void (`null`) | `DocumentNotFound`→404, `DocumentInUse`→409 |
| `POST /documents/:id/publish` | `publishDocument` request → `Document` response | `DocumentNotFound`→404, `UnresolvedReference`→422 |
| `POST /documents/:id/unpublish` | `unpublishDocument` request → `Document` response | `DocumentNotFound`→404, `InvalidState`→409 |
| `POST /documents/:id/archive` | `archiveDocument` request → `Document` response | `DocumentNotFound`→404, `InvalidState`→409 |
| `GET /documents` | `listDocuments` request → `DocumentList` response | `WikiNotFound`→404, `ValidationError`→400 |
| `POST /wikis` | `createWiki` request → `Wiki` response | `ValidationError`→400 |
| `GET /wikis/:id` | `getWiki` request → `Wiki` response | `WikiNotFound`→404 |
| `GET /wikis` | `listWikis` request → `Vec<Wiki>` response | none |

**Cross-cutting fail-states (request-decode, NEW-2):**
- A malformed request envelope (unparseable JSON, missing `method`/`args`) → **400**.
- An unknown `"method"` value → **422**.
- An unknown `schema_version`/`id_format` → **400**.
- A mutating request with no `caller` field → **400** (P1a §10).
- None of these is a `StoreError`; none has a §11 row; none is 502.

---

## 11. The §5.x Property register (MANDATORY — PBT gate)

This register is the contract for the **property-based-testing gate** on the P2 server unit.
The TestWriter's executed layer runs under `cargo test` with a **deterministic pinned seed**,
**≤100 generated cases per register row**, **≤400 total cases** across the unit's whole property
layer, **stop-after-5** (report ≤5 distinct held/broken counterexamples per row), and records
each row as **held** or **broken** together with its `Strategy-id`. The adversarial reviewer
then reads this register with the executed artifacts and performs a **read-only** PBT audit;
reviewers never run generators.

Authoring rule: `Property-id` = `P-<CLASS>-<N>`, `CLASS ∈ {IM, SM, TP}`, **≤ 8 rows**;
invariants are **crisp, universally-quantified, black-box-observable through the server's public
surface only**, and are **TRUE of the current GREEN implementation** (implied by the contract —
they are **not** pending features). **There are no fail-state rows** (no `§6`/`FS-*` rows) and
**no gap/parked rows** — invariants only.

**Reserved-variant discipline applied.** The status-mapping and request-decode-outcome
functions are **total over their closed input sets** (the 7 CRUD-reachable `StoreError` variants
and the 5 request-decode `DecodeError` variants), so rows may quantify over all of them; there
are **no** reserved fail-variant rows (no `FS-*` rows) by the invariant-only rule. The only
generator restriction is **well-formedness on the decode side**: never generate a malformed
request/response body or an unknown `schema_version`/`id_format` as an *invariant* input — those
are the request-decode fail-states (the auditor's negative-generator territory), **not**
invariant rows.

## Property table

| Property-id | Class | Invariant | Strategy-id | Observable-as-property |
|---|---|---|---|---|
| `P-IM-1` | IM | **§11 status-mapping is total over the CRUD-reachable `StoreError` variants.** For **any** of the 7 CRUD-reachable `StoreError` variants (`DocumentNotFound`, `WikiNotFound`, `ValidationError`, `ConflictError`, `DocumentInUse`, `InvalidState`, `UnresolvedReference`), the server maps it to a **defined** HTTP status (from the §11 map) + a **defined** wire code; no CRUD-reachable variant is unmapped, and the mapping is a deterministic pure function of the variant. | `strat:status-total` | ∀ `e` in the 7 CRUD-reachable variants: `server_status(e)` is `Some((status, code))` with `status` in the §11 map and `code == e.wire_code()`; equal variants → equal `(status, code)`. |
| `P-IM-2` | IM | **Request-decode outcome mapping is total.** For **any** of the 5 request-decode `DecodeError` variants (`InvalidJson`, `InvalidEnvelope`, `UnknownMethod`, `UnsupportedSchemaVersion`, `UnknownIdFormat`), the server maps it to a **defined 400/422 status, never 502**; no `DecodeError` variant is unmapped, and the mapping is a deterministic pure function of the variant. | `strat:decode-outcome-total` | ∀ `e: DecodeError` (the 5 variants): `request_decode_status(e)` is `Some(status)` with `status ∈ {400, 422}`; `status != 502`; equal variants → equal status. |
| `P-IM-3` | IM | **Endpoint routing is a bijection.** The routing table maps each of the 14 paths (11 CRUD + 3 retrieval) to **exactly one** handler, and each handler is reachable by exactly one path; the 14 paths are **pairwise distinct**. | `strat:route-bijection` | ∀ distinct `(path_a, handler_a), (path_b, handler_b)` in the routing table: `path_a != path_b`; `handler_a != handler_b`; the table has exactly 14 rows, one per endpoint. |
| `P-SM-1` | SM | **Status-mapping determinism + wire-code fidelity.** For **any** CRUD-reachable `StoreError`, the server's rendered HTTP status + wire code are stable on repeat, and the wire code **equals** `StoreError::wire_code()` (the server does not invent a code). | `strat:status-determinism` | ∀ `e` in the 7 CRUD-reachable variants: `server_status(e) == server_status(e)` on repeat; `server_status(e).code == e.wire_code()`. |
| `P-SM-2` | SM | **Request-decode outcome determinism + the 400/422 split is stable.** For **any** request-decode `DecodeError`, the server's rendered status is stable on repeat, and the 400/422 split is stable (the same variant always yields the same status). | `strat:decode-outcome-determinism` | ∀ `e: DecodeError` (the 5 variants): `request_decode_status(e) == request_decode_status(e)` on repeat; the variant→status mapping is a fixed pure function. |
| `P-SM-3` | SM | **Decode-layer `caller` validation.** For **any** mutating CRUD request envelope, the decode layer requires the `caller` (a mutating request with no `caller` → request-decode 400); for **any** read-only request, a `caller` is tolerated (the read-only args carry no `caller` field, so an extra one is ignored). | `strat:caller-threaded` | ∀ mutating `(method, args)`: `decode_crud_request` rejects a request with no `caller` (→ 400); ∀ read-only `(method, args)`: `decode_crud_request` accepts a request that carries a `caller`. |
| `P-TP-1` | TP | **Server never emits a response envelope the client decoder rejects.** For **any** CRUD method, the response envelope the server emits (result or error) is accepted by the client-side decoder (`decode_crud_response`); the server never emits a body the decoder rejects. | `strat:server-encode-validates` | ∀ CRUD method + well-formed result/error: `decode_crud_response(&server_response(method, result_or_error)).is_ok()`. |

**Class tally:** IM ×3, SM ×3, TP ×1 = **7 rows ≤ 8** ✔.

## Generator-coverage note (per row — boundary + adversarial input shapes)

- **`P-IM-1 — strat:status-total`.** Enumerate all 7 CRUD-reachable `StoreError` variants (each
  at least once) plus several random permutations; for each assert a defined `(status, code)`.
  Boundary: `ValidationError` with empty/single-byte/multi-codepoint `m` (the code stays
  `"validation_error"`); `ConflictError` → 409 (mandated, FS-4); `UnresolvedReference` → 422.
- **`P-IM-2 — strat:decode-outcome-total`.** Enumerate all 5 request-decode `DecodeError`
  variants; for each assert a defined 400/422 status, never 502. Boundary: `UnknownMethod` with
  empty/arbitrary method strings → 422; `InvalidJson`/`InvalidEnvelope`/`UnsupportedSchemaVersion`/
  `UnknownIdFormat` → 400.
- **`P-IM-3 — strat:route-bijection`.** Enumerate the 14 routing-table rows; assert pairwise-
  distinct paths and pairwise-distinct handlers; assert the table has exactly 14 rows, one per
  endpoint (a bijection). The 11 CRUD paths must equal the P1a `ENGINE_ENDPOINTS` paths verbatim.
- **`P-SM-1 — strat:status-determinism`.** Same corpus as `P-IM-1`; for each variant assert
  `server_status(e) == server_status(e)` on repeat and `code == e.wire_code()`.
- **`P-SM-2 — strat:decode-outcome-determinism`.** Same corpus as `P-IM-2`; for each variant
  assert the status is stable on repeat and the variant→status mapping is fixed.
- **`P-SM-3 — strat:caller-threaded`.** For each of the 7 mutating methods, build a request
  envelope with the `caller` removed from `args` and assert `decode_crud_request` rejects it
  (→ request-decode 400). For each of the arg-carrying read-only methods (`getDocument`/
  `listDocuments`/`getWiki`), add a `caller` to `args` and assert `decode_crud_request` accepts
  it (the caller is ignored).
- **`P-TP-1 — strat:server-encode-validates`.** For each CRUD method, generate a well-formed
  result (per P1a §4.3) and a representative error; assert `decode_crud_response` accepts the
  server's emitted response envelope. This closes the server→client encode→decode loop so no
  server output trips `CrudResponseError::Decode`/`Validation`.

## API notes for the TestWriter

- **Pure/synchronous surface.** The status-mapping (`server_status`), request-decode-outcome
  (`request_decode_status`), and routing-table (`route_bijection`) fns are pure synchronous fns
  over values — `#[test]`, no tokio runtime, no engine instance. Build `StoreError`/`DecodeError`
  values directly from the public `src/store/mod.rs` + `src/wire/` types.
- **Live-transport surface.** The end-to-end transport test (§9) runs against the real server
  (a tokio runtime + the bound `127.0.0.1:<port>`), exercising the `EngineUnavailable`/
  `EngineError` split and the document-CRUD round-trips over the wire.
- **Determinism/seeding.** Use one deterministic pinned seed per binary, ≤100 cases per row,
  ≤400 total; stop-after-5; report each row held/broken with its `Strategy-id`.

---

## 12. Cross-references and ownership hand-off

- **Contract authority:** `docs/specs/gnosis.md` §4.1.3, §4.1.4, §4.4.3, §4.4.5, §4.6.1, §6.
- **Base wire layer:** `docs/specs/engine-wire-contract.md` (F2 — the `Envelope`, the error
  codec, the §11 map, decode-then-validate, the golden vectors V-1..V-9, the SSE framing).
- **CRUD wire layer:** `docs/specs/p1a-document-crud-wire.md` (P1a — the 11 §4.1 wire shapes,
  the `ENGINE_ENDPOINTS`/`ENDPOINT_*` constants, the §6.2 request-decode outcome table, the RBAC
  `caller` shape, the golden vectors V-10..V-14).
- **Register format precedent:** `docs/specs/7-2-wire-property-register.md` (the F2 register this
  unit's §5.x register mirrors).
- **PBT:** the §5.x register (authored HERE) + the TestWriter's executed property layer.
- **Conformance tests (TestWriter):** `tests/gnosis_server_conformance.rs` (or an extension of
  the existing conformance suites) + the e2e transport test.

**Owned by a later unit (not P2, per §2):**
1. The shell-side client (A1) — consumes the same paths + shapes.
2. The graph/fact/consistency/RAG-companion endpoints (P1b–P1e — deferred follow-ons).
3. The RBAC **enforcement** semantics (the engine is the enforcer; P2 threads the `caller` only).
4. Bind-loopback + auth/TLS policy (recorded shell-owned, as in F2/P1a).

---

## 13. What the spec does NOT do (constraints honored)

- This is a **TDD unit spec** — it includes the §5.x Property register (PBT gate) but does
  **NOT** author the property tests (the TestWriter does) and does **NOT** author implementation.
- It does **NOT** touch `src/` or `tests/` — it writes **only** the spec file.
- It does **NOT** change the F2 error codec or the §11 map (21 rows) — it reuses them verbatim
  and qualifies the "no new statuses" claim to the `StoreError` taxonomy (NEW-2).
- It does **NOT** add new `StoreError` variants or new §11 rows.
- It does **NOT** change the engine lib's dependency set — the server framework (axum/hyper/
  tower) is scoped to the bin only.
