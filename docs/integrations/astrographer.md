# Astrographer — Gnosis's front-end (the proxy seam)

**Edge:** Astrographer (shell) ↔ Gnosis (engine) — the **proxy seam** (`docs/specs/gnosis.md` §5.1).

## Nature

Astrographer is Gnosis's **front-end**. Gnosis is a **pure backend**: it has
**no MCP surface and no GUI surface** (§4.6.2). The Astrographer Electron shell
provides the GUI + MCP surfaces that proxy Gnosis's API.

## Mechanism

The shell proxies engine calls to Gnosis over the **`RagStore` interface**
(`docs/specs/gnosis.md` §4.1.5, persistence) + the **query/stream/engine-status
API** (§4.6.1, retrieval). This is the natural IPC/process boundary: Gnosis
runs as a separate process (the multithread-capable Rust engine); the shell
talks to it over IPC/HTTP. A future engine swap is isolated to this seam.

## The contract the shell proxies

- **Document store (§4.1):** `createDocument`/`getDocument`/`updateDocument`/
  `deleteDocument`/`publishDocument`/`unpublishDocument`/`archiveDocument`/
  `listDocuments`/`createWiki`/`getWiki`/`listWikis`.
- **Knowledge graph (§4.2):** adjacency methods, topological resolution, the
  triple operations, entity resolution, manual overrides.
- **Fact/citation (§4.3):** `getFact`/`listFacts`/`getProfileSummary`,
  `getQueryAuditLog` (the audit-log recording is engine-side; the MCP tool + GUI
  panel are shell-side).
- **Consistency (§4.4):** `getConsistencyReport`.
- **RAG (§4.6.1):** `ragQuery`/`ragStream`/`getEngineStatus`.

## Value

The shell gains the full engine work-package (document store, knowledge graph,
fact/citation tracking, consistency enforcement, RAG/agent-memory retrieval)
**without owning the graph/vector data or math** (`docs/specs/gnosis.md` §5.1).

## D4 parity

D4 MCP-GUI parity **applies at the shell**, not at Gnosis (§4.6.2): the shell
provides the MCP tools + GUI screens that proxy Gnosis's API. The
security-configuration carve-out (engine credentials, Astral push creds,
TLS/secret mgmt, Firmament bridge auth) is GUI-only at the shell.

## Fail-states

The engine is **optional** (D2): the shell's document store and cross-link
features work fully without the engine; only RAG query features require it.
`EngineUnavailable`/`EngineError` are surfaced when the engine is unreachable or
returns a malformed result.
