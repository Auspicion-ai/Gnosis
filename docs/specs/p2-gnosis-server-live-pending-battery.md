# §7.2 P2 — `gnosis-server` binary crate — LIVE-SCENARIO PENDING BATTERY (MCP/UI parity)

- **Unit:** §7.2 P2 — the `gnosis-server` `[[bin]]` crate (the live HTTP/SSE transport host).
- **Date:** 2026-09-10.
- **Author-role:** live-scenario-runner.
- **Status:** **PENDING** (parked — **not** a gate failure).
- **Source set transcribed:** `docs/specs/p2-gnosis-server-greens.md` (15 scenarios S1–S15, all **GREEN**).

## 1. What is already verified live (NOT parked)

The **server-endpoint scenarios S1–S15** are **live-runnable NOW** and were executed against
the running `gnosis-server` bin (bound to `127.0.0.1:<port>`) plus the compiled
`tests/blind_p2_gnosis_server_greens.rs` (15/15) and `tests/gnosis_server_e2e.rs` (16/16)
binaries. **All 15 server-endpoint scenarios PASS live** (see the runner's report). Those are
**not** parked here.

## 2. Why the MCP/UI parity scenarios are a PENDING battery

The **D4 MCP-GUI feature-parity** surface for the CRUD endpoints is exercised through the
**Astrographer Electron shell** — its `gnosis.*` MCP tools and its GUI panes over the
document-CRUD surface. That surface requires the **Astrographer-side CRUD routing client
(A1)**, which is **deferred** in the approved roadmap
(`docs/integrations/astrographer-interface-implementation.md` §4.6: the full `RagStore` CRUD
routing is deferred to a later unit once the engine freezes the CRUD wire shapes; the
shell-side client = **A1**). Gnosis itself is a **headless backend with no MCP/GUI surface**
(`AGENTS.md` — D4 parity applies at the shell, not at Gnosis).

Consequently the MCP/UI parity scenarios **cannot be executed live yet**: there is no
Astrographer Electron app running with the `gnosis` group enabled, and no `gnosis.*` MCP
tool / GUI pane wired to the CRUD surface. Every MCP/UI parity scenario derivable from the
greens set is therefore **parked** here for a later iteration of this runner. **Parked
scenarios are NOT a failure** — the gate outcome for P2 is **PASS on the server-endpoint
scenarios (S1–S15)** with the MCP/UI parity surface **PENDING** (A1).

### REVISIT CONDITION

This battery resumes (the MCP/UI parity gate may be re-run live) when **both** of the
following hold:

1. the **Astrographer CRUD routing client (A1)** is wired to consume the same paths + shapes
   the `gnosis-server` bin serves, AND
2. the **Astrographer Electron app is running** with the **`gnosis` group enabled** (its
   `gnosis.*` MCP tools and GUI panes are live).

**The live check that ends the park** (both must pass):

- the Astrographer `gnosis.createDocument` MCP tool returns a **`Document`** result, AND
- the Astrographer GUI document pane reflects the created document (list + editor).

Until then this battery records the exact MCP/UI parity scenarios to execute at that point.
No engine code (`src/`) or test code (`tests/`) is modified by this battery.

## 3. The MCP/UI parity scenarios (a later runner executes these against the live shell)

Each row: the parity scenario (faithful transcription of the corresponding greens
server-endpoint scenario — **no new behavior invented**), the exact live action a consumer
performs through the Astrographer shell, and the EXPECTED observable (the same wire
bytes/status/outcome the greens pin, now rendered through the shell's MCP tool / GUI pane).

### 3.1 CRUD MCP-tool parity (one per CRUD method)

| Scenario | Live action (Astrographer `gnosis.*` MCP tool) | Expected observable (parity with greens) |
| --- | --- | --- |
| **M1** (← S10) — `gnosis.createDocument` | invoke the MCP tool with a `createDocument` request (after pre-creating the wiki) | returns a **`Document`** result; the shell renders it in the document pane |
| **M2** (← S14) — `gnosis.getDocument` | invoke the MCP tool with a `getDocument` request | returns a **`Document`** result |
| **M3** (← S14) — `gnosis.updateDocument` | invoke the MCP tool with an `updateDocument` request | returns a **`Document`** result |
| **M4** (← S14) — `gnosis.deleteDocument` | invoke the MCP tool with a `deleteDocument` request | returns a **void** (`null`) result; the document disappears from the GUI list |
| **M5** (← S14) — `gnosis.publishDocument` | invoke the MCP tool with a `publishDocument` request | returns a **`Document`** with `state:"Published"` |
| **M6** (← S14) — `gnosis.unpublishDocument` | invoke the MCP tool with an `unpublishDocument` request | returns a **`Document`** with `state:"Draft"` |
| **M7** (← S14) — `gnosis.archiveDocument` | invoke the MCP tool with an `archiveDocument` request | returns a **`Document`** with `state:"Archived"` |
| **M8** (← S14) — `gnosis.listDocuments` | invoke the MCP tool with a `listDocuments` request | returns a **`DocumentList`** result |
| **M9** (← S14) — `gnosis.createWiki` | invoke the MCP tool with a `createWiki` request | returns a **`Wiki`** result |
| **M10** (← S14) — `gnosis.getWiki` | invoke the MCP tool with a `getWiki` request | returns a **`Wiki`** result |
| **M11** (← S14) — `gnosis.listWikis` | invoke the MCP tool with a `listWikis` request | returns a **`WikiList`** result |

### 3.2 Retrieval-trio + health MCP-tool parity

| Scenario | Live action (Astrographer `gnosis.*` MCP tool) | Expected observable (parity with greens) |
| --- | --- | --- |
| **M12** (← S9) — `gnosis.ragQuery` on a not-READY engine | invoke the MCP tool with a `rag_query` request on a fresh (not-READY) server | surfaces **`EngineUnavailable`** → the shell renders **503** (never 502) |
| **M13** (← S4) — `gnosis.engineStatus` | invoke the MCP tool with an engine-status request | returns a **`HealthReport`** carrying `state`, `version`, `subsystems`, `lastError` |

### 3.3 GUI-pane parity over the CRUD surface

| Scenario | Live action (Astrographer GUI pane) | Expected observable (parity with greens) |
| --- | --- | --- |
| **M14** (← S14) — document list + editor panes | create/get/update/delete/publish/unpublish/archive a document through the GUI | each GUI action round-trips the same request → response envelope the server-endpoint scenarios pin; the pane reflects the resulting `Document` state |
| **M15** (← S14) — wiki list pane | create/get/list wikis through the GUI | each GUI action round-trips the same `Wiki`/`WikiList` envelope; the pane reflects the resulting wikis |
| **M16** (← S4) — engine-status pane | open the engine-status pane | the pane renders the `HealthReport` (`state`, `subsystems`) |

### 3.4 MCP/UI error parity

| Scenario | Live action | Expected observable (parity with greens) |
| --- | --- | --- |
| **M17** (← S11) — `ConflictError` → 409 | drive a stale-`base_revision` `updateDocument` through the MCP tool or GUI | the shell surfaces **409** + the `"conflict"` error envelope |
| **M18** (← S12) — malformed request → 400 | send an unparseable body through the MCP tool / GUI | the shell surfaces **400** |
| **M19** (← S13) — unknown method → 422 | send a well-formed envelope with an unrecognized `"method"` | the shell surfaces **422** |
| **M20** (← S15) — per-endpoint fail-state statuses | drive each documented fail-state through the MCP tool / GUI | `WikiNotFound`→404, `DocumentNotFound`→404, `ValidationError`→400, `ConflictError`→409, `DocumentInUse`→409, `InvalidState`→409, `UnresolvedReference`→422 — each rendered by the shell |

## 4. Parity re-check, not new coverage

Every scenario above is **already verified live at the server-endpoint layer** (S1–S15 all
PASS against the running `gnosis-server` bin, plus the compiled blind-greens 15/15 and e2e
16/16 binaries). The pending battery adds **no new behavior and no new test coverage**. Its
entire value is **re-confirming the SAME behavior end-to-end through the Astrographer shell's
`gnosis.*` MCP tools and GUI panes** — i.e. the **D4 MCP-GUI feature-parity** re-check that
the server behavior proven live is identical when proxied by the shell's CRUD routing client
(A1).

## 5. How the later runner executes this

Once A1 (the Astrographer CRUD routing client) is wired and the Astrographer Electron app is
running with the `gnosis` group enabled, this runner will:

1. **Confirm the park is over** — the `gnosis.createDocument` MCP tool returns a `Document`
   AND the GUI document pane reflects it.
2. **Exercise M1–M20 through the shell** — drive the CRUD methods, retrieval trio, health, and
   the fail-states through the `gnosis.*` MCP tools and the GUI panes.
3. **Record live pass/fail per scenario** — each row that passes live confirms the in-repo +
   server-endpoint parity; a scenario whose **live** result **contradicts the greens/contract**
   is **a finding** (a real regression or a doc/spec drift — a live failure is never a pass).
   Any parked rows whose shell surface still does not exist are re-parked, not failed.

## 6. Authoritative refs

- `docs/specs/p2-gnosis-server.md` — the P2 server contract (loopback bind §4, endpoints §5, READY §6, §11 status rendering + NEW-2 §7, RBAC caller §8, e2e §9, valid/fail states §10).
- `docs/specs/p2-gnosis-server-greens.md` — the blind-greens set (15 scenarios S1–S15, all GREEN) this battery transcribes.
- `docs/integrations/astrographer-interface-implementation.md` — the stated deferral (the shell-side CRUD client = A1).
- `docs/specs/p1a-document-crud-wire-live-pending-battery.md` — the P1a pending-battery precedent this document mirrors.
- `tests/blind_p2_gnosis_server_greens.rs` (15/15), `tests/gnosis_server_e2e.rs` (16/16) — the in-repo + live server-endpoint verification layer.

This battery modifies **only** this document. No `src/` or `tests/` change. No commit.
