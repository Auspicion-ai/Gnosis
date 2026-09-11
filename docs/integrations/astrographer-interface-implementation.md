# Astrographer ↔ Gnosis — Interface Implementation Guide

- **Status:** **IMPLEMENTATION GUIDE** for the Astrographer shell to connect to and
  use the Gnosis engine. Drafted 2026-09-09.
- **Proposal gate:** **PASS — scoped per Architecture A1** (2026-09-09; validity
  VALID-WITH-CONTENTIONS, critique 3-blocking, architecture A1, change-analysis
  PASS). The scoped deliverable is the **retrieval trio + health** wire client;
  full `RagStore` CRUD routing, the server-host binary crate, and CRUD wire shapes
  are **deferred** (see §4.6, §10.4). The 8 required amendments are landed in this
  guide.
- **Edge:** Astrographer (shell) ↔ Gnosis (engine) — the **proxy seam**
  (`docs/specs/gnosis.md` §5.1).
- **Wire contract:** `docs/specs/engine-wire-contract.md` (the F2 mechanism-agnostic
  wire contract, decision `F2-WIRE-CONTRACT-A1`).
- **Owner:** the Astrographer shell unit (this is the shell-side implementation
  guide; Gnosis ships the wire codecs + the engine, not the HTTP server/client).

---

## 1. Architecture overview

Gnosis is a **pure headless backend** — it has **no MCP surface and no GUI surface**
(§4.6.2). The Astrographer Electron shell provides the GUI + MCP surfaces that
**proxy** Gnosis's API. The shell talks to Gnosis over the **`RagStore` interface**
(§4.1.5, persistence) + the **query/stream/engine-status API** (§4.6.1, retrieval).

The engine runs as a **separate process** (the multithread-capable Rust engine). The
shell talks to it over **IPC/HTTP**. A future engine swap is isolated to this seam.

```
┌─────────────────────────────┐        ┌─────────────────────────────┐
│  Astrographer (Electron)    │  wire  │  Gnosis (Rust engine)        │
│  GUI + MCP surfaces         │ ─────► │  RagStore + ragQuery/        │
│  createEngineRagStore proxy │  HTTP  │  ragStream/engine-status     │
│  (this guide)               │  /SSE  │  (the engine, already built) │
└─────────────────────────────┘        └─────────────────────────────┘
```

**What the shell owns (this guide):** the HTTP/native-IPC transport decision, the
shell-side HTTP/SSE **client**, bind/auth/TLS + loopback policy, the retrieval-trio
wire client + decode-then-validate, HTTP-status rendering, READY observation, and
D2 engine-absent shell behavior. The **server host** is a separate thin binary crate
in the Gnosis workspace (the engine lib stays at zero new runtime deps); the shell
does not host the Rust server.

**What Gnosis owns (already built):** the wire codecs (`src/wire/`), the engine
(`src/store/mod.rs`), the `ragQuery`/`ragStream`/`getEngineStatus` surface, and the
`getCommunityContext` accessor. Gnosis ships **no** HTTP server.

---

## 2. The transport (the shell's decision)

The F2 wire contract is **mechanism-agnostic** — it pins the wire *serialization*
but leaves the *transport* to the shell. The shell must decide:

- **HTTP/REST + SSE** (recommended — `ragStream` is spec'd as SSE, and the crate
  already leans HTTP via `reqwest`/`wiremock`), or
- **a native IPC channel** (e.g. stdio, a Unix socket, or a named pipe).

**Recommendation: HTTP/REST + SSE over loopback.** The server binds to
`127.0.0.1` (loopback-only by contract — see §8 security). The server is a
**separate thin binary crate in the Gnosis workspace** (e.g. `gnosis-server`, a
`[[bin]]` depending on the engine lib + axum/hyper/tower) — the engine *lib* stays
at zero new runtime deps. The shell's `createEngineRagStore` proxy (generalizing
Incanter's `createRemoteRagStore`) is the **client**: it issues REST calls for the
retrieval surface and subscribes to SSE for `ragStream`.

**Engine discovery:** the shell discovers the server's bind address via config, a
CLI arg, or an env var (e.g. `GNOSIS_BIND=127.0.0.1:PORT`). The engine's boot→READY
lifecycle (construct `Arc<Store>` → build `DerivedIndexes` → wire the embedding
provider → transition to `READY`) is **engine-internal**; the shell's role is to
**observe** READY via `getEngineStatus` (poll/subscribe on `state` + `subsystems`)
and gate RAG calls on `state == Ready`.

---

## 3. The wire contract (what the shell must implement identically)

The shell's client must reproduce the wire shapes exactly. The canonical shapes are
frozen in `docs/specs/engine-wire-contract.md` §4–§12; the golden conformance
vectors (V-1..V-9) are the byte-exact reference. Key shapes:

### 3.1 The versioned envelope

Every payload is wrapped in an envelope. Wire keys are **camelCase**:

```json
{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":<chunk | result | error json>}
```

- `schemaVersion` = `1` (the current version; a future value is the extensibility
  seam).
- `idFormat` = `"opaque-string-v1"` (ids cross the wire as opaque strings; RFC-4122
  is NOT implemented — see the UUID-v4 deferral in §4.1 of the wire contract).
- `payload` = the canonical chunk/result/error JSON.

### 3.2 Canonical chunk JSON (in `envelope.payload` and the SSE `data:` line)

| Cargo variant | wire JSON | notes |
| --- | --- | --- |
| `RagChunk::Result(r)` | `{"type":"result","result":<RagResult body>}` | `trace` is required. |
| `RagChunk::Done` | `{"type":"done"}` | terminal, no payload. |
| `RagChunk::Error(e)` | `{"type":"error","code":"<wire_code>","message":"<Display text>"}` | `message` = `format!("{}", e)`. |

**Body shape note (serde-frozen).** The `<RagResult body>` is exactly what serde
emits for the frozen `RagResult`/`RagResultItem`/`RagTrace` types — **snake_case**
field keys (`document_id`, `node_id`, `top_k`), **PascalCase** enum-unit values
(`"Local"`, `"Zodiac"`, `"Flat"`, `"Graph"`, `"Vector"`, `"Hybrid"`), `RagTrace`
**externally-tagged** by variant name (`{"Flat":{…}}`, `{"Graph":[…]}`, `{"Hybrid":{…}}`),
id newtypes as their inner strings, and optional fields (`parent`, `stale`,
`blocked_by`) always present, `null` when `None`. See §12 V-5 for the pinned full
body. Only the wire-defined types (`Envelope`, `HealthReport`, the
chunk/`code`/`message`/`type` wrapper) use camelCase at their top level.

### 3.3 Canonical error JSON (the non-chunk codec)

`{"code":"<wire_code>","message":"<Display text>"}` — used when an error is encoded
on its own (e.g. as a query's error outcome); the chunk encoder adds the
`"type":"error"` discriminator on top.

### 3.4 Canonical SSE frame (`ragStream`)

Single event, LF line endings, terminal blank line:

```
event: <type>
data: <canonical chunk JSON, single line>
<blank line>
```

`<type>` ∈ `{result, done, error}` and **must equal** the `"type"` field of the data
JSON. `data:` is single-line JSON (no multi-line SSE continuation). No `id:`/`retry:`
fields. `ragStream` is **single-shot** — it emits at most one `result` (the full
`RagResult`) then `done`, or one `error` then `done`.

### 3.5 Health report (`getEngineStatus`)

```json
{"schemaVersion":1,"idFormat":"opaque-string-v1","state":"Ready","version":"…","subsystems":{"store":true,"graph":true,"lexical":true,"vector":true,"embedding":true,"reranker":true},"lastError":null}
```

- `state` ∈ `{"Ready","Starting","Degraded","Unavailable"}` (PascalCase).
- `subsystems` flags: `store`, `graph`, `lexical`, `vector`, `embedding`, `reranker`.
- `lastError` is `Some` exactly when the engine is `Degraded` (a faithful
  projection; the shell must not invent one).

---

## 4. The API surface (what the shell proxies)

The shell's `createEngineRagStore` proxy exposes the **retrieval trio + health**
(the F2 wire surface). The full `RagStore` trait (all methods in `src/store/mod.rs`)
is listed below for reference, but **only the retrieval trio + health cross the wire
in this unit**; the full CRUD routing is deferred until the engine freezes CRUD wire
shapes. Grouped by concern:

### 4.1 Document store (§4.1)
`createDocument`, `getDocument`, `updateDocument`, `deleteDocument`,
`publishDocument`, `unpublishDocument`, `archiveDocument`, `listDocuments`,
`createWiki`, `getWiki`, `listWikis`.

### 4.2 Knowledge graph (§4.2)
`edgesFrom`, `edgesTo`, `edgesByKind`, `edgesForDocument`, `docHeadForDocument`,
`resolveReferences`, `addTriple`, `getTriples`, `queryTriples`, `declareCommunity`,
`getCommunity`, `listCommunities`, `updateCommunitySummary`, `setReferenceState`,
`resolveEntities`, `entityAliasCanonical`, `mergeFacts`, `getCommunityContext`.

### 4.3 Fact/citation (§4.3)
`getFact`, `createFact`, `listFacts`, `updateFact`, `proposeCandidateFact`,
`getQueryAuditLog`.

### 4.4 Consistency (§4.4)
`getConsistencyReport`, `reSyncEmbed`, `reDeriveCommunity`, `communityState`.

### 4.5 RAG / agent-memory (§4.5/§4.6.1)
`ragQuery`, `ragStream`, `getEngineStatus`, `bm25Search`, `vectorSearch`,
`getProfileSummary`.

### 4.6 The retrieval trio (the F2 wire surface — the scoped deliverable)
The F2 wire contract covers **`ragQuery` / `ragStream` / `getEngineStatus` + health**
(the §4.6.1 retrieval seam). This is the **only** wire surface this unit implements.
The full `RagStore` CRUD routing (§4.1–§4.4) is **deferred** to a later unit, once
the engine freezes CRUD wire shapes — the shell must not invent them unilaterally.

---

## 5. HTTP-status map (the shell renders this)

The shell's HTTP transport renders the engine's `StoreError` taxonomy to HTTP
status codes. The mapping is frozen in `docs/specs/engine-wire-contract.md` §11;
`ConflictError` = **409** is **mandated** (FS-4). Key rows:

| wire code | `StoreError` | HTTP status |
| --- | --- | --- |
| `"not_found"` | `DocumentNotFound` | 404 |
| `"wiki_not_found"` | `WikiNotFound` | 404 |
| `"validation_error"` | `ValidationError` | 400 |
| `"conflict"` | `ConflictError` | **409** (mandated) |
| `"doc_in_use"` | `DocumentInUse` | 409 |
| `"invalid_state"` | `InvalidState` | 409 |
| `"unresolved_reference"` | `UnresolvedReference` | 422 |
| `"engine_unavailable"` | `EngineUnavailable` | 503 |
| `"engine_error"` | `EngineError` | 502 |
| `"trace_unavailable"` | `TraceUnavailable` | 502 |
| `"hop_limit_exceeded"` | `HopLimitExceeded` | 422 |
| `"cycle_detected"` | `CycleDetected` | 409 |
| `"embedding_unavailable"` | `EmbeddingUnavailable` | 503 |
| `"vector_index_unavailable"` | `VectorIndexUnavailable` | 503 |
| `"lexical_index_unavailable"` | `LexicalIndexUnavailable` | 503 |
| `"reranker_unavailable"` | `RerankerUnavailable` | 503 |
| `"compression_failed"` | `CompressionFailed` | 500 |
| `"hyde_generation_failed"` | `HyDEGenerationFailed` | 500 |
| `"multi_query_expansion_failed"` | `MultiQueryExpansionFailed` | 500 |
| `"community_not_found"` | `CommunityNotFound` | 404 |
| `"sub_task_dag_failed"` | `SubTaskDagFailed` | 500 |

The shell's client must translate the wire `code` back into its typed error model
deterministically (so the GUI/MCP surfaces can present the right error).

---

## 6. Error handling / fail-states

- **`EngineUnavailable` (503):** the engine is not `READY` (state
  `UNAVAILABLE`/`STARTING`). The shell's D2 fallback: the shell's local-first store
  and cross-link features work without the engine; engine-backed RAG + engine
  document-store features surface `EngineUnavailable`.
- **`EngineError` (502):** the engine returned a malformed result. The shell's
  decoder must **decode-then-validate** — a malformed body → `EngineError`; a
  well-formed body missing `trace` → `TraceUnavailable`. (These are decoder-side
  outcomes; the engine has no in-repo producer for them.)
- **`TraceUnavailable` (502):** a result without a `trace`.
- **The retrieval fail-states** (FS-11..FS-19, FS-26): `HopLimitExceeded`,
  `CycleDetected`, `EmbeddingUnavailable`, `VectorIndexUnavailable`,
  `LexicalIndexUnavailable`, `RerankerUnavailable`, `CompressionFailed`,
  `HyDEGenerationFailed`, `MultiQueryExpansionFailed`, `SubTaskDagFailed` — each
  maps to a wire code + HTTP status per §5.

**Decode-then-validate in the shell client:** the shell must validate the decoded
`RagResult` (trace present, `engine == "gnosis"`, `blocked_by ⇒ RagTrace::Graph`)
so `EngineError`/`TraceUnavailable` are real transport outcomes, not silent
deserialization gaps.

**Edge cases the shell client must specify:**
- **Connection-refused vs engine-not-spawned** — both map to `EngineUnavailable`
  (503), but are distinct shell-side paths (the shell should distinguish "server
  refused" from "no server process" for diagnostics).
- **Mid-stream SSE failure** — `ragStream` is single-shot: at most one
  `result`/`error` then `done`. A partial frame or a dropped connection before
  `done` is a transport failure → `EngineUnavailable`/`EngineError`; no partial
  result is committed.
- **Unknown `schemaVersion` / `idFormat`** in a response → `EngineError` (the
  decoder rejects them as `UnsupportedSchemaVersion`/`UnknownIdFormat`).
- **Event/data type mismatch** in an SSE frame → `EngineError`.
- **HTTP-status rendering of decoder outcomes** — `EngineError` and
  `TraceUnavailable` both render as HTTP 502.

---

## 7. D2 optional-engine behavior (the shell's job)

The engine is **optional** (D2), but the framing must be precise. **Engine-absent =
UNAVAILABLE**: when the engine is absent/unreachable, the *engine's* document store
and RAG features are both unavailable. The D2 "document store works without the
engine" claim refers to the **shell's own local-first store** (the shell can run a
local JSON store without the engine), NOT the engine's `RagStore` persistence —
which is engine-side and unavailable when the engine is absent. **DEGRADED** is a
distinct state: the engine is running but a non-core subsystem (embedding/reranker)
is down; core store/graph/lexical still work. The shell must:

- **Probe readiness** via `getEngineStatus` (the `state` + `subsystems` flags).
- **Degrade gracefully** when the engine is absent/unreachable: the shell's
  local-first store and cross-link features continue; engine-backed RAG + engine
  document-store features surface `EngineUnavailable`.
- **Handle connection-refused / engine-not-spawned** as the shell's
  connection-refused → `EngineUnavailable` path (this is shell-side; Gnosis merely
  reports its own state).

---

## 8. Security (bind/auth/TLS — the shell's job)

The wire is **loopback-only by contract**. The shell must:

- **Bind the engine to loopback** (`127.0.0.1`) — the engine is a headless internal
  backend, not a public API.
- **Own auth/TLS** — the D4 security-configuration carve-out (engine credentials,
  Astral push creds, TLS/secret management, Firmament bridge auth) is **GUI-only at
  the shell** (§4.6.2). Gnosis does not hold credentials.
- **Not expose the engine as an unauthenticated public API** — the shell's proxy is
  the only surface.

---

## 9. D4 MCP-GUI parity (applies at the shell)

D4 MCP-GUI parity **applies at the shell**, not at Gnosis (§4.6.2): the shell
provides the MCP tools + GUI screens that proxy Gnosis's API. Every engine feature
must be reachable through **both** the GUI and the MCP surface. The
security-configuration carve-out is GUI-only at the shell.

---

## 10. Implementation checklist (the shell unit)

1. **Decide the transport** — HTTP/REST + SSE over loopback (recommended) or a
   native IPC channel.
2. **Implement the wire client** — reproduce the envelope, chunk JSON, error JSON,
   SSE frame, and health report shapes exactly (use the golden vectors V-1..V-9 as
   the byte-exact conformance reference).
3. **Implement decode-then-validate** — so `EngineError`/`TraceUnavailable` are real
   transport outcomes.
4. **Route the retrieval trio + health over the wire** — `ragQuery`/`ragStream`/
   `getEngineStatus` + health. (Full `RagStore` CRUD routing is **deferred** until
   the engine freezes CRUD wire shapes.)
5. **Implement the SSE client** for `ragStream` — subscribe, parse the single-event
   frames, surface `result`/`done`/`error`.
6. **Render the HTTP-status map** — translate wire codes to HTTP status + typed
   errors.
7. **Observe the engine boot→READY lifecycle** — poll/subscribe `getEngineStatus`
   until `state == Ready`; gate RAG calls on READY. (The engine's own boot wiring
   constructs the store, builds indexes, and wires the embedding provider — the
   shell does not drive engine-internal state.)
8. **Implement D2 engine-absent behavior** — probe readiness, degrade gracefully.
9. **Enforce loopback + auth/TLS** — bind to loopback, own credentials.
10. **End-to-end transport test** — the §7.2 F2 benefit: confirm the
    `EngineUnavailable`/`EngineError` split matches the real transport's failure
    modes (connection-refused, malformed frames, unknown `schemaVersion`/`idFormat`,
    mid-stream cancel).

---

## 11. Cross-references

- `docs/specs/engine-wire-contract.md` — the wire contract (the authoritative shapes
  + golden vectors + HTTP-status map).
- **Cross-repo drift control:** the golden vectors **V-1..V-9** are the shared
  cross-repo artifact — the shell's conformance suite must consume them byte-for-byte
  (ideally as a standalone JSON fixture in the Gnosis repo), and the
  `schemaVersion`/`idFormat` handshake makes a wire-contract change fail loudly on
  the shell side. Both repos run conformance against the same fixture.
- `docs/specs/7-2-f2-review.md` — the F2 proposal-review record (the shell-integration
  deferrals).
- `docs/specs/gnosis.md` §4.1.5 (RagStore seam), §4.6.1 (query surface), §4.6.2 (no
  MCP/GUI), §5.1 (the proxy seam), §6 (fail-states).
- `docs/integrations/astrographer.md` — the edge analysis.
- `docs/decisions.md` `F2-WIRE-CONTRACT-A1`, `ID-SCHEME-RECONCILE-MONOTONIC`.
- `docs/specs/f4-llm-enrichment-integration.md` — the future LLM-host integration
  (drives the enrichment surfaces over this same seam).
