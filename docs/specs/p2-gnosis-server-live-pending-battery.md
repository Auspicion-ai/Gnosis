# §7.2 P2 — `gnosis-server` binary crate — LIVE-SCENARIO PENDING BATTERY (MCP/UI parity)

- **Unit:** §7.2 P2 — the `gnosis-server` `[[bin]]` crate (the live HTTP/SSE transport host).
- **Date:** 2026-09-10 (**§3.5/§4/§6 reconciled + §7/§8 added by U5's gate 8, 2026-09-22**).
- **Author-role:** live-scenario-runner.
- **Status:** **PENDING** (parked — **not** a gate failure).
- **Source set transcribed:** `docs/specs/p2-gnosis-server-greens.md` (15 scenarios S1–S15, all **GREEN**).

## 1. What is already verified live (NOT parked)

The **server-endpoint scenarios S1–S15** are **live-runnable NOW** and were executed against
the running `gnosis-server` bin (bound to `127.0.0.1:<port>`) plus the compiled
`tests/blind_p2_gnosis_server_greens.rs` (15/15) and `tests/gnosis_server_e2e.rs` (**28/28** — the landed
file has 28 `#[test]`/`#[tokio::test]`s; **27** was the count after U2's live cases landed, and **16/16** was
the count at the original P2 run before U2's live cases
landed). **All 15 server-endpoint scenarios PASS live** (see the runner's report). Those are
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

### 3.5 The U2/U3 live obligations (rows `R-L1`/`R-L2` — the register rows' named homes; **`R-L3` added 2026-09-22 by U5's spec gate**; **its PASS (v) is PARKED — POST-GREEN SPEC AMENDMENT, 2026-09-22, item 5 of the U5 amendment below**)

**These two rows are NOT shell-parity scenarios and do NOT depend on A1 or on the Astrographer app.** They are the
two **live obligations** the U2/U3 typed §5.x registers (`docs/specs/p2-gnosis-server.md` §9.5.1/§9.5.2) name but a
`tests/` row cannot cover: `R-L1` is `P-TP-2`'s (β) positive observable (the handler→engine pass-through, observable
only across the handler) and `R-L2` is `P-IM-9`'s provider-reachable boot half (which needs a **controlled
provider** — an Ollama-compatible endpoint that answers `/api/tags`). **A third row, `R-L3`, was added
2026-09-22 by U5's spec gate** (`docs/specs/p2-gnosis-server.md` §9.5.5) as that unit's **live** obligation —
the boot index build observed through the bin — and follows the same discipline (running bin alone, controlled
provider, no shell). Each is cited from its register row as **that obligation's home**, so none falls between
the TestWriter and this runner. Their precondition is therefore
**not** this battery's M1–M20 revisit condition (A1 wired + the Astrographer app running) but simply a **running
`gnosis-server` bin with its own reachable provider**; **both rows have since been executed live against the bin
alone — `R-L1` on 2026-09-17 (U2's half) and `R-L2` on 2026-09-17 (U3's half), each PASS; see the two
annotations at the end of this document.** The **M1–M20** parity rows remain pending on the shell (A1 + a
running app), unchanged. **Doc-review note (2026-09-17, gate 8):** §3.5's two row records and §4's scope note
were re-read against the tree and are current — `R-L1`/`R-L2` carry their executed-live PASS records, the
**M1–M20** rows keep their park and their revisit condition verbatim, and nothing in this battery is owed.
Record: `archive/reviews/2026-09-17-u3-status-honesty-doc-review.md`.

| Scenario | Live action (against the running `gnosis-server` bin, no shell) | Precondition | Pass / fail criterion |
| --- | --- | --- | --- |
| **`R-L1`** — the wire `topK`/`filters`/`mode` visibly change the response (the home of `P-TP-2`'s (β); U2) | `POST /rag/query` with the envelope-strict body, three probes: (i) `{"query":"<q>","topK":1,"filters":{"nodeKind":"fact"}}`, (ii) the same query with `{"topK":50}`, (iii) the same query with `{"mode":"bm25"}` (and a casing variant, e.g. `{"mode":"BM25"}`) | a **READY** engine (a reachable provider) with a seeded corpus that contains at least one `fact` node and more than 1 retrievable item | **PASS:** probes (i)/(ii) return **200** `RagResult`s whose **`results` array** carries **≤ 1** / **≤ 50** entries respectively (`RagResult.results` — the frozen wire/type field is **`results`**, not `items`, `src/store/mod.rs:758-765`) and whose `results` match only `fact` nodes for (i) — i.e. `topK` and `filters` **visibly changed the response**; probe (iii) is the same request as (ii) except that the *only* added key is `mode`, and because the token is unrecognized it must be **400 `validation_error`** — pinning that the handler read the wire `mode` through the shared decoder on a **READY** boot too (the pre-U1 handler ignored `mode` entirely, so a 200 here is the inert-option defect). **FAIL:** any probe returns a response the wire values cannot explain (e.g. `topK:1` still yields more than 1 result, `filters` yields non-`fact` nodes, or the unrecognized `mode` yields 200/503 instead of 400 `validation_error`). A `503` from a not-READY boot is **not** a pass: it means the precondition failed, and the row is re-parked, not failed. |
| **`R-L2`** — a provider-reachable boot reports `Ready` with `embedding:true` (the home of `P-IM-9`'s live half; U3) | boot the bin with a **controlled provider** reachable at the configured Ollama-compatible endpoint, then `GET /engine/status` | a **controlled provider** answering `/api/tags` (nothing else in the store is required — no index build) | **PASS:** the `HealthReport` reports `state: "Ready"` **and** `subsystems.embedding == true`, `subsystems.vector == false`, `subsystems.reranker == false`, with `store`/`graph`/`lexical` all `true` — i.e. no false capability claim and no `Ready`-implies-everything reading. **FAIL:** `state` is not `Ready` with the provider reachable (the boot's own branch is wrong), or any flag contradicts its capability predicate — in particular `embedding:false` (the flag did not follow the provider) or `vector:true`/`reranker:true` at U3-time (both are false until U5 / forever without a reranker). ***(U5 amendment, 2026-09-22: the `vector == false` cell above is the U3-time reading and is superseded for the post-U5 tree — a provider-reachable boot builds the index, so the honest value is `vector: true` (`docs/specs/p2-gnosis-server.md` §9.5.5's contract table (1)). The U5-time criterion is `state:"Ready"` + `embedding:true` + **`vector:true`** + `reranker:false`, core `true`; the row's own executed-live record at the foot of this file stands as the 2026-09-17 U3-time observation. The build's own live obligation is the new row **`R-L3`**.)*** |
| **`R-L3`** — the boot index build is real on a live bin, and `mode=vector` serves on it (the home of U5's live obligation; **U5 — AUTHORIZED 2026-09-22, code owed**; *post-green, 2026-09-22: code reported landed and the criterion statuses are as this file's U5 amendment, now with **two** parks — PASS (iii) per item 3(a) and PASS **(v)** per item **5** — so the row's executable set is (i)/(ii)/(iv), and PASS (v) is **NOT live-verified***; run result and criterion-status reconciliation, gate 8, 2026-09-22: **EXECUTED LIVE** — `R-L3` (i)/(ii)/(iv) **PASS live** on a controlled provider (the bin answering `/api/tags`), with the provider-absent boot and the configured-but-unreachable boot re-confirmed in the same pass; (iii) **PARKED** (the seeded-corpus criterion — the bin's boot store is in-memory and empty, so its CRUD-only seeding surface cannot precede a boot-time build; reason re-confirmed live) and (v) **PARKED** (the failed-`Reachable`-build branch — an empty boot corpus makes **zero** `embed` calls, so the build has no live path to fail; reason re-confirmed live; **lib-level only**, `p2` §9.5.5's `P-IM-15`). The **FS-14 READY-index-free mapping is now PARKED as structurally non-reachable through the bin post-U5** — the only READY boot the bin produces is the `Reachable` one, which builds an index (an empty index included), so no post-U5 boot reaches `vectors: None` while READY; that mapping's home is the lib-level state-6 instance (`tests/rag_query_integration.rs`'s caller-built READY store) — see the park note below. **R-L2's amended cell was re-verified live in the same pass:** `Ready` + `vector:true` + `embedding:true` + `reranker:false` (core `true`), `mode=vector` ⇒ **200**. **Doc-drift finding F1 (fixed in place, below):** the (iv)/(v) clauses asserted a non-READY boot's `mode=vector` ⇒ 503 `vector_index_unavailable`; the live bin returns **503 `engine_unavailable`** (FS-8) — the reading `p2` §9.5.5's `P-SM-7` and REMAND-2 MUST-FIX 1 already pin for a non-READY store.
**The (iv)/(v) restatement follows on this row's continuation line below** (the gate-8 reassembly of the criterion column; the pre-edit text stands as the record). | **Criterion column — restated (gate 8, 2026-09-22; the pre-edit criterion column's continuation follows as the dated record).** | boot the bin with a **controlled provider** reachable at the configured Ollama-compatible endpoint, `POST /rag/query` with the envelope-strict body `{"query":"<q>","mode":"vector"}` (and, with a seeded corpus, a second probe whose query matches a seeded node), then `GET /engine/status`; **plus a fifth probe on a second spawn whose controlled endpoint answers `/api/tags` but errors on `/api/embed`** (the failed-`Reachable`-build branch) | a **controlled provider** answering **both** `/api/tags` (the boot probe) **and** `/api/embed` (the build's per-node embeddings + the query's own embedding); a store that may be **empty** (the empty-store case is itself a criterion — see below); for probe (v) a provider whose `/api/embed` response makes the bin's own parse fail — **the exact requirement (REMAND-2 NOTE 10, 2026-09-22):** a body that is **not parseable as JSON**, or a parseable body **without a usable `embeddings[0]` array**, which is what maps to `StoreError::EmbeddingUnavailable` (`src/bin/gnosis_server.rs:348-357`); a **non-2xx status is NOT what maps it** — the provider code never inspects `resp.status()` (`:342-357`; only the send error at `:347` and the `embeddings[0]` parse at `:348-357` produce that variant), so a 4xx/5xx answer that *did* carry a valid `embeddings[0]` array would **succeed** and the probe would not exercise the failed-build branch. The **pre-remand wording** ("a non-2xx status or a body without an `embeddings[0]` array … both mapped to `StoreError::EmbeddingUnavailable`") is superseded in place. `/api/tags` still answers **2xx** (`:364-375`, the probe) | **PASS (all of):** (i) `GET /engine/status` reports `state:"Ready"`, `subsystems.vector == true`,
`embedding:true`, `reranker:false`, core `true` — on a **fresh** boot with an **empty** store too (an empty index is still `Some`); (ii) the `mode=vector` probe returns **HTTP 200** with a `RagResult` whose `trace` is the vector trace (`engine:"gnosis"`) — **never** 503 `vector_index_unavailable` on a provider-reachable boot; (iii) **PARKED with the reason recorded (REMAND-1, 2026-09-22) — the seeded-corpus criterion is not executable against the bin alone today.** What was asserted ("on a **seeded** corpus the returned `results` are drawn from that corpus's nodes, and each returned node's `value` is the text the provider was asked to embed") has two defects: (a) **`value` is not on the wire** — each result item carries `snippet`, `documentId` and `nodeId` (`RagResultItem`, `src/store/mod.rs:688-699`), so no wire assertion can read a node's `value`; and (b) the bin hard-codes `Store::new()` and builds the index at boot (`src/bin/gnosis_server.rs:381`, `:414` — *gate-8, 2026-09-22: the build **call** is `let built = build_boot_vector_index(&store, wired.as_ref()).await` at `:414`; the cell's earlier `:404` citation is stale — it lands in the §9.5.5 (U5) comment block that introduces the call (`:403-413`), while `Store::new()` at `:381` is correct*), while the **only** seeding surface the running bin exposes is its own CRUD routes (`POST /wikis`, `POST /documents`, `POST /documents/:id`) which necessarily run **after** the build read the store — and U5 has **no rebuild vehicle** (a boot-only build), so a corpus seeded post-boot can never appear in the boot index. **Restated criterion (the executable form):** every returned `results[i].documentId`/`nodeId` must be a member of the seeded `(documentId, nodeId)` set whose nodes carry `value: Some(_)`, and each `snippet` must be that node's authored text or a prefix-derived excerpt of it — assertable **when** a seeding mechanism exists that runs **before** the boot (a durable/seed-file boot path, or the register's own recipe against a lib-level boot: `docs/specs/p2-gnosis-server.md` §9.5.5's "corpus seeding" clause, `RagStore::create_wiki` → `create_document` → `update_document` with the doc-head/doc-end `Graph`). **Until such a boot-time seeding path is authorized, (iii) is parked — not failed, not silently dr
opped** (the empty-store and provider-absent contrasts (i)/(iv) still run, and the lib-level seeded cases are U5's `P-IM-10`/`P-IM-11`/`P-SM-7`, already executable); (iv) a **provider-absent** boot (a separate `env_remove` spawn) still reports `subsystems.vector == false` and a `mode=vector` probe still **503 `engine_unavailable`** (FS-8) — the honest contrast for a **non-READY** boot (*superseded wording, kept as the record: this clause read "still **503 `vector_index_unavailable`**"; the live run's gate-6 finding **F1** (2026-09-22) and `p2` §9.5.5's `P-SM-7` / §5.9's precedence bullet pin the READY-gated reading — FS-14 is unreachable on a non-READY store, and `p2` §9.5.5's valid/fail state 6 is where FS-14 lives*); (v) **the failed-`Reachable`-build branch (REMAND-1, the home of `P-IM-15`'s live half):** with the controlled endpoint of the "for probe (v)" cell, `GET /engine/status` reports `state:"Degraded"` (never `"Ready"`), `subsystems.vector == false`, `subsystems.embedding == true`, `reranker:false`, core `true`, with the store's **fixed** `lastError` `"a non-core subsystem (embedding/reranker) is unavailable"` (`src/store/mod.rs:4226-4230` — *gate-8, 2026-09-22: re-read, the string literal is at **`:4230`** and the `if state == Degraded` block is `:4229-4233`; the block-level spelling stands, the precise-site form is recorded here*), and the `mode=vector` probe is **503 `engine_unavailable`** (FS-8) (*superseded wording, kept as the record: this clause read "the `mode=vector` probe is **503 `vector_index_unavailable`** (an unbuilt index wins over the wired provider — §5.9's order pin, `src/store/mod.rs:4448-4454`)". The failed build applies `Degraded` — a **non-READY** state — so the pre-READY gate (`src/store/mod.rs:4078-4080`) fires **before** any leg check: this is the gate-6 **F1** correction, identical in kind to `p2` §9.5.5's REMAND-2 MUST-FIX 2 and already pinned by `p2` §9.5.5's `P-IM-15` row / failure table row 3. **PARKED** for the live run by `p2` §9.5.5's POST-GREEN SPEC AMENDMENT item 5 (an empty boot corpus makes **zero** `embed` calls, so the build cannot fail live) and by this file's U5 amendment item 5 below; §5.9's order pin still
 governs the *query-time* leg order on a READY store). **FAIL:** `vector:false` with the provider reachable (no build happened); a build over the **wrong** corpus (e.g. only the queried wiki's nodes, or only nodes with non-empty `value`s after a partial pass); a **partial** index observable after an `embed` failure (the status still says `Ready` + `vector:true` while only some nodes are indexed); a `mode=vector` 503 on a provider-reachable boot; **on probe (v):** `state:"Ready"`, `vector:true`, `embedding:false` (the provider was dropped from the wiring), a reworded `lastError`, or any status/code other than 503 `engine_unavailable` (*superseded wording, kept as the record: this sentence read "or any status/code other than 503 `vector_index_unavailable`" — **F1**, gate 6/8, 2026-09-22: probe (v)'s expected status is FS-8 `engine_unavailable` on the *correct* code, and the mirror-image error the FAIL set must catch is a run that returns `vector_index_unavailable` there; the criterion is **PARKED** for live execution, so nothing was scored against it*); or any **new** wire code/status on any of the probes. |

## 4. Parity re-check, not new coverage

**Scope note (§3.5).** The paragraph below applies to the **M1–M20** parity rows. The **`R-L1`/`R-L2`** rows of
§3.5 are **not** parity re-checks: they are the U2/U3 registers' two **live obligations**, which are **NOT** already
covered by any live layer (there is no in-repo test that can reach them), so they are genuinely **new live coverage**
for those two obligations — not a re-confirmation. ***(U5 annotation, 2026-09-22:** the same scope note applies to
§3.5's third row **`R-L3`** (U5's boot-build live obligation) — it is also new live coverage and also **not** an
M1–M20 parity row; **two** of its criteria are **parked** — PASS (iii), the seeded corpus, per item 3(a) of the U5
amendment below (it needs a boot-time seeding mechanism the bin does not have), and **PASS (v), the
failed-`Reachable`-build branch, per item 5 of that amendment (POST-GREEN SPEC AMENDMENT, 2026-09-22: the bin's boot
store is empty, so a reachable provider's build makes **zero** `embed` calls and cannot fail live)** — while
(i)/(ii)/(iv) remain executable against the running bin alone.**)*** ***(Gate 6/8 annotation, 2026-09-22 — the live run happened and the parks were re-confirmed: `R-L3` (i)/(ii)/(iv) **PASSED live** against the running bin with a controlled provider; the provider-absent and configured-but-unreachable spawns were re-confirmed as FS-8 contrasts in the same pass; (iii) and (v) were re-confirmed **PARKED** with the reasons above and item 5's empty-boot-corpus reason respectively; and the **FS-14 READY-index-free mapping** joins the park list as **structurally non-reachable through the bin post-U5** — see §8 below and the `R-L3` row's description cell.**)***

Every scenario above is **already verified live at the server-endpoint layer** (S1–S15 all
PASS against the running `gnosis-server` bin, plus the compiled blind-greens 15/15 and e2e
**28/28** binaries; **27/27** was the count after U2 and **16/16** at the P2 landing). The pending battery adds **no new behavior and no new test coverage**. Its
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
- `tests/blind_p2_gnosis_server_greens.rs` (15/15), `tests/gnosis_server_e2e.rs` (**28/28**, the landed
  count — 27 was the post-U2 count and 16/16 the pre-U2 count), `tests/blind_u2_query_post_greens.rs`
  (**24/24**, U2's blind set) and `tests/blind_u3_status_honesty_greens.rs` (**13/13**, U3's blind set) —
  the in-repo + live server-endpoint verification layer.
- `tests/blind_u5_boot_vector_index_greens.rs` (**27/27** — **U5's** blind set, 2026-09-22: 2 live-HTTP + 25
  lib-level) and its report `docs/specs/u5-boot-vector-index-greens.md`. Gate 8 re-read that report's claims
  against `p2` §9.5.5 in the same pass and corrected its "628 baseline" label: **628** is the interim
  post-green/pre-blind count, so the arithmetic is **604 + 24 U5 in-crate = 628, then + 27 blind = 655** — the
  figure the tree reports.

This battery modifies **only** this document. No `src/` or `tests/` change. No commit.

**Row-index note (added by the second register remand, 2026-09-16).** The **M1–M20** rows above are the MCP/UI
parity battery and are governed by the revisit condition in §2 (A1 wired **and** the Astrographer app running).
The **`R-L1`/`R-L2`** rows added in **§3.5** are **not** parity scenarios: they are the two live obligations the
U2/U3 typed property registers name (`docs/specs/p2-gnosis-server.md` §9.5.1 `P-TP-2` (β) and §9.5.2 `P-IM-9`), and
they are executable against the **running bin alone** (with a reachable/controlled provider) — they neither need
nor wait on A1 or the shell. The M1–M20 rows, their numbering, and §2's revisit condition are unchanged by §3.5.
***(U5 annotation, 2026-09-22:** the same rule covers §3.5's added row **`R-L3`** — it is the live obligation U5's
typed register names (`docs/specs/p2-gnosis-server.md` §9.5.5; the build + `mode=vector` on a booted bin), homed
here so no row of that register is left without a home.**)***

**`R-L2` executed live — PASS (2026-09-17, U3 live-battery annotation).** The U3 half of §3.5 has been run
against a **controlled provider**: with an Ollama-compatible endpoint answering `/api/tags` reachable at the
configured `GNOSIS_SERVER_OLLAMA_URL`, `GET /engine/status` on the booted bin returned

```
{"schemaVersion":1,"idFormat":"opaque-string-v1","state":"Ready","version":"0.1.0",
 "subsystems":{"store":true,"graph":true,"lexical":true,"vector":false,"embedding":true,"reranker":false},
 "lastError":null}
```

i.e. the **observed vector** is `{store:true, graph:true, lexical:true, vector:false, embedding:true,
reranker:false}` with `state:"Ready"` and `lastError:null` — exactly the register's `R-L2` PASS criterion
(`P-IM-9`'s live half), **and the status surface agrees with the query surface in the same session**: a
`mode=vector` request against the *same* process returns **503 `vector_index_unavailable`** (FS-14), which is
precisely what `vector:false` claims — so the honest flag and the leg's outcome are consistent rather than
contradictory. The **configured-but-unreachable** boot was verified in the same pass: `state:"Degraded"`,
`subsystems.embedding == false`, `vector:false`, `reranker:false`, core `true`, with the pinned
`lastError` (`"a non-core subsystem (embedding/reranker) is unavailable"`) — the `P-IM-8`/V-8.2 pair. **U3's
live battery is therefore 9/9 rows PASS live**, and this row is no longer pending. **Residual recorded, not
absorbed:** after a boot whose provider *was* reachable and then became unreachable, the status stays
byte-identical (`state:"Ready"`, `embedding:true`, `lastError:null`), because nothing re-evaluates
`EngineState` after boot — filed as an **OPEN** defect row in `docs/defects.md` (owner: a later
transport/status unit; **not** a U3 regression — every pinned U3 row passes).

**`R-L1` executed live — PASS (2026-09-17, proofread-pass annotation).** The U2 half of §3.5 has been run
against a **READY** boot with a real provider: the wire `topK`/`filters` **visibly changed** the response and
an unrecognized `mode` returned **400 `validation_error`** — i.e. `R-L1` **PASSES** and `P-TP-2`'s (β)
obligation is discharged. **`R-L2` is likewise discharged live (the paragraph above, 2026-09-17)**, so
§3.5 carries no pending row of its own; the **M1–M20** parity rows remain pending on the shell (A1 + a
running app) exactly as before. ***(POST-GREEN SPEC AMENDMENT, 2026-09-22 — scoping the clause above: "no
pending row of its own" still holds of the two **historical** rows `R-L1`/`R-L2` (both PASS live), but the
§3.5 **third** row `R-L3` (added 2026-09-22, i.e. after this paragraph was written) now carries **two parked
criteria** — PASS (iii) and PASS (v), per item 3(a) and the new item 5 of the U5 amendment below — while its
(i)/(ii)/(iv) stay executable against the running bin alone. `R-L3` is **not executed yet**, so §3.5 is not
fully discharged.**)*

**U5 amendment — the `R-L3` row added, and `R-L2`'s `vector` cell re-pointed (2026-09-22, SPEC-GATE annotation, docs-only).**
`docs/specs/p2-gnosis-server.md` **§9.5.5** authored U5's contract + typed register (`P-IM-10`…`P-IM-15`,
`P-SM-7`, `P-TP-5`) after the user's go-ahead (`docs/specs/gnosis-grq-inbound-review.md` §14 POST-RECORD
UPDATE 1, **Q3 = "(B) U5 ONLY"**). Two consequences for **this** file, both **additive** and both **owed to
U5's unit**:

1. **`R-L2`'s PASS cell is re-pointed, not deleted.** It carried the U3-time reading `vector == false` for a
   provider-reachable boot; under §9.5.5's contract table (1) such a boot **builds** the index (an empty
   store included, since an empty `VectorIndex` is still `Some`), so the honest post-U5 value is
   **`vector: true`** — the dated note sits in the cell, and the 2026-09-17 executed-live record above stands
   as the **U3-time** observation it was (it is not a failure, and it is not rewritten). Until a runner
   re-executes it post-U5, `R-L2` reads "PASS at U3-time; re-run under U5's tree for the amended cell".
2. **`R-L3` is new and is U5's live home.** §9.5.5's rows are **lib-level** (`build_boot_vector_index` is a
   lib seam, all 8 rows assertable with a `Store` + an injected provider), so exactly **one** obligation is
   live-only: the built index observed through the **running bin** and a `mode=vector` request that
   **serves** on a provider-reachable boot. `R-L3` carries it, with its own precondition (a controlled
   provider answering **both** `/api/tags` and `/api/embed`), its PASS criteria (i)–(iv) — **plus (v), added by
   REMAND-1 (item 3(b) below), for the failed-`Reachable`-build branch** — and its FAIL set
   (which includes the **partial-index** observable — the case §9.5.5's `P-IM-13` forbids at lib level and
   this row re-checks end to end). `R-L3` is **not yet executed** (U5's code is owed); it is **not** an
   M1–M20 parity row and does **not** wait on A1 or the shell — exactly like `R-L1`/`R-L2`.
   ***(GATE 6/8 UPDATE, 2026-09-22 — `R-L2`'s amended cell WAS re-executed post-U5 in the live pass
   (`Ready` + `vector:true` + `embedding:true` + `reranker:false`, core `true`; `mode=vector` ⇒ **200**), so the
   clause above ("Until a runner re-executes it post-U5 …") is discharged; and `R-L3` **WAS executed live** —
   (i)/(ii)/(iv) PASS, (iii)/(v) PARKED, with the FS-14 state-6 mapping parked as structurally non-reachable
   through the bin post-U5. See §8. The two clauses above stand as the pre-run record.)***
3. **REMAND-1 corrections to `R-L3` (2026-09-22, same-day one-pass remand on U5's spec gate — docs-only).**
   Two of that row's criteria were not executable as written, so both are now pinned instead of ambiguous:
   **(a) PASS (iii) is PARKED with its reason recorded.** It asserted that "each returned node's `value` is
   the text the provider was asked to embed", but the wire carries **`snippet`** (with `documentId`/`nodeId`)
   and never `value` (`RagResultItem`, `src/store/mod.rs:688-699`); and it presupposed a **seeded** corpus at
   boot while the bin hard-codes `Store::new()` (`src/bin/gnosis_server.rs:381`) and builds the index at boot
   (`:404`) with **no rebuild vehicle**, so a corpus seeded through the bin's own CRUD routes necessarily
   arrives **after** the build. The row now states the executable form (result-item membership in the seeded
   `(documentId, nodeId)` set + a `snippet` drawn from that node's authored text) and records the park: the
   criterion runs only when a **boot-time** seeding mechanism exists (the register's own recipe, `p2` §9.5.5's
   "corpus seeding" clause, is lib-level today). The empty-store, provider-absent and failed-build contrasts
   keep running, and the lib-level seeded cases remain `P-IM-10`/`P-IM-11`/`P-SM-7`.
   **(b) PASS (v) is new — the failed-`Reachable`-build branch now has a live home.** With a controlled
   provider answering `/api/tags` (probe succeeds) but **erroring on `/api/embed`**, the booted bin must
   report `state:"Degraded"`, `vector:false`, `embedding:true`, `reranker:false`, core `true`, the fixed
   `lastError`, and `mode=vector` ⇒ **503 `engine_unavailable`** (FS-8 — **REMAND-2 MUST-FIX 2 (2026-09-22): this clause previously read `503 vector_index_unavailable`, which was wrong for this branch; the failed `Reachable` build applies `Degraded` (`p2` §9.5.5's failure-table row 3), a **non-READY** state, so the pre-READY gate (`src/store/mod.rs:4078-4080`, `p2` §5.9's precedence bullet) fires **before** any leg check and the row would otherwise report a FALSE FAIL against correct code**) — so U5's `P-IM-15` is assertable both
   lib-level (`p2` §9.5.5's pinned failure order) and live, and neither is the other's substitute. **The row
   index, the M1–M20 rows, §2's revisit condition and `R-L1`/`R-L2`'s historical text are untouched.**

4. **REMAND-2 corrections to `R-L3` (2026-09-22, the second same-day one-pass remand on U5's spec gate —
   docs-only; the row keeps its id, precondition, row index and (i)–(iv)/(v) criteria).** Two corrections and
   one reconciliation, all recorded in item 3's cells:
   **(a) MUST-FIX 2 — the failed-build query probe expects FS-8, not FS-14.** PASS (v)'s `mode=vector` probe
   asserts **503 `engine_unavailable`** (FS-8): the failed `Reachable` build applies `Degraded` (`p2` §9.5.5's
   failure-table row 3), a **non-READY** state, and the pre-READY gate (`src/store/mod.rs:4078-4080`; `p2`
   §5.9's precedence bullet) fires **before** any leg check, so the request can never reach
   `VectorIndexUnavailable`. The pre-remand cell read `503 vector_index_unavailable` and would have made this
   row report a **FALSE FAIL against correct code** — the class §5.9's "Interlock with U5" bullet and §6
   forbid. **The row's FAIL set therefore includes the mirror-image error:** a live run that returns
   **503 `vector_index_unavailable`** on the failed-build spawn is a **spec-drift finding, not a pass** (the
   status is state-gated), and `state:"Ready"` / `vector:true` on that spawn remain failures as recorded.
   **(b) NOTE 10 — the precondition names what the controlled provider must actually return.** The `/api/embed`
   side must make the bin's parse fail (an unparseable body, or a parseable body without a usable
   `embeddings[0]` array — `src/bin/gnosis_server.rs:348-357`); the **HTTP status alone is not what maps to
   `EmbeddingUnavailable`** (the provider never inspects `resp.status()`, `:342-357` — a non-2xx answer
   carrying a valid `embeddings[0]` array would succeed). The pre-remand "non-2xx status … both mapped" text is
   superseded in place.
   **(c) `P-SM-7` reconciliation (MUST-FIX 1's live echo).** `R-L2`'s 2026-09-17 executed-live record (which
   observed `vector:false` + `mode=vector` ⇒ 503 `vector_index_unavailable` on a **READY** U3-time boot)
   **stays as the U3-time observation** and is **consistent** with the corrected reading — that process was
   `Ready`, which is exactly the state FS-14 requires. No historical PASS record is rewritten.

5. **POST-GREEN SPEC AMENDMENT — `R-L3`'s PASS (v) is PARKED (2026-09-22, same day as the U5 green; docs-only,
   SPECWRITER pass on U5's landed unit; no `src/`, `tests/` or other-spec edit). ONE new item (5); the row keeps
   its id, kind, tag, strategy ids, precondition and every criterion.** The Implementer's report proved that item
   3(b)'s and item 4(a)'s clause — the failed-`Reachable`-build boot asserted **live** — is **not live-reachable
   with today's bin**, so the historical text of items 3(b)/4(a) (and of the row's own fifth probe) is the
   pre-green record and this item is the authority for the criterion's status:
   **(a) PASS (v) is PARKED, and here is the exact reason.** It is **structurally non-exercisable at the bin
   level**: the bin's boot **constructs an in-memory, empty store** (no corpus before the build; `p2` §9.5.5(2)'s
   *empty store* corpus row, and §9.5.5(6)'s no-persistence clause — the store is in-memory and the
   durability unit is **NOT ACTIVE**), so a **reachable** provider's build makes **zero** `embed` calls — *n* = 0
   embeddable nodes ⇒ `Ok(Some(VectorIndex::default()))`, an **empty** index, **not** an error. The only
   `/api/embed` the boot's build could consult is the per-embeddable-node call, and with no embeddable node that
   path is never taken. **(b) The probe's own observed outcome (as reported by the Implementer, not a new live
   claim by this pass).** A controlled endpoint that answers `/api/tags` **200** and `/api/embed` **500** yields
   `Ready` + `vector:true` over the empty index; `mode=vector` on that READY process then fails inside the
   **query's own** embedding with **503 `embedding_unavailable`** (**FS-13**, a **query-time** leg failure —
   `p2` §9.5.5's fail-state table item 7 / §5.9's order pin), **NOT** `Degraded`. So a runner following item
   3(b)'s literal text would have scored a **FALSE FAIL against correct code** — the class §5.9's Interlock
   bullet and §6 forbid, exactly as REMAND-2 MUST-FIX 2 recorded for the FS-8/FS-14 mix-up; the **pre-green**
   FAIL-set sentence that treats any non-`Degraded` outcome on that spawn as a failure is **superseded in place**.
   **(c) Lib-level home (the criterion is not deleted and not un-asserted).** The failure branch is asserted at
   the **lib level only**: `p2` §9.5.5's `P-IM-15` (`strat:boot-index-failure`, contract table (1) row 3 +
   contract table (4)'s pinned failure order — the provider **is** wired, so `Degraded` + `vector:false` +
   `embedding:true` + the store's own fixed `lastError` + discarded flag vector) executed by the conformance
   layer's **state 4** (`tests/u5_boot_vector_index_conformance.rs` — `u5_state_4_mid_build_error_installs_no_index_and_degrades`,
   with the boundary *k* = 1 / *k* = *n* and the pinned call order pinned again by
   `u5_failure_branch_call_order_is_the_pinned_order`). A lib-level store can carry a corpus, so it **can** drive
   a failing `embed`; the live bin cannot. **No third home is created.** **(d) The un-park trigger.** The park
   lifts — and PASS (v) reverts to a **live** criterion in its item 3(b)/4(a) reading (`Degraded`,
   `vector:false`, `embedding:true`, `reranker:false`, core `true`, the fixed `lastError`, and `mode=vector` ⇒
   **503 `engine_unavailable`**, FS-8) — when a boot path carries a **corpus before the bind**: a pre-bind
   seed-file or durable-boot corpus, i.e. the **durability unit** (`ENGINE-DURABLE-CORPUS-DIRECTION`, DIRECTION
   ONLY / NOT ACTIVE today, `p2` §9.5.5(6)), **or** any boot path that carries documents **before** the index
   build. Until then the criterion is **PARKED, not failed**, and **the failure branch is NOT live-verified**:
   the status here is "proven at lib level; live half parked, with the reason above recorded".
   **(e) What does not move.** No row was added or deleted, `R-L3`'s id / kind / tag / strategy ids are unchanged,
   `R-L1`/`R-L2`, the M1–M20 rows, §2's revisit condition and §4's scope note for those rows are untouched (the
   scope note carries only the (iii)+(v) park annotation), and no new authority is claimed. The historical text
   of items 3(b)/4(a) and of the row's fifth probe is **kept as the pre-green record**.

**Nothing else in this file moves:** the **M1–M20** rows, their numbering, §2's revisit condition and §4's
parity scope note are **unchanged**; §3.5's two original rows keep their historical text; the row-index note
above still governs (§3.5's rows are executable against the running bin alone — **the `R-L3` PASS (iii) park
recorded in item 3(a) of the U5 amendment above is a criterion that needs a boot-time seeding
mechanism, and it is parked, not failed**); ***(POST-GREEN SPEC AMENDMENT, 2026-09-22 — the "one criterion"
reading of the clause above is superseded: it is **two** parked criteria now, (iii) per item 3(a) and **(v)**
per item 5, the latter because an **empty** boot corpus cannot drive a build failure live; the clause's rule —
parked, not failed — is unchanged and now covers both. `R-L3`'s remaining executable set is
(i)/(ii)/(iv).)*** **No `src/`, `tests/` or other spec edit is made by this
annotation.** ***(REMAND-1 correction marker (2026-09-22, same-day one-pass remand on `p2` §9.5.5):** `R-L3`'s
PASS (iii) is **restated and PARKED** with its reason (the wire carries `snippet`, and the bin's CRUD-only
seeding cannot precede a boot-time build) and PASS **(v)** is **added** for the failed-`Reachable`-build
branch (`Degraded`, `vector:false`, `embedding:true`, `mode=vector` ⇒ 503 — **the code pinned by REMAND-2 is
`engine_unavailable`** (item 4(a) above; the pre-REMAND-2 `vector_index_unavailable` reading is superseded in
place); ***POST-GREEN SPEC AMENDMENT, 2026-09-22: that added (v) is itself **PARKED** — see item 5 above; the
pre-green wording here is kept as the record***); both changes are recorded in item
3 of the U5 amendment above. The row keeps its id, its precondition and its (i)/(ii)/(iv) criteria; no
criterion was deleted without a recorded reason.**)***

---

## 8. U5's live obligation (`R-L3`, gate 6) — the executed run, the parks, and finding F1 (2026-09-22)

**Authority for this section:** the gate-6 live-scenario pass over U5's landed tree (the running
`gnosis-server` bin alone, a controlled provider, no shell — §3.5's discipline), plus the gate-8
documentation review that recorded it. **This section adds no criterion, deletes none and re-parks nothing
that was not already parked**; it is the executed-run record the `R-L3` row's status cells point at.

**1. What was executed live, and the result.** The row's remaining executable set — the clauses the parks
leave — was run against the compiled bin:

| clause | live action | result |
| --- | --- | --- |
| **(i)** | controlled provider answering `/api/tags` at the configured `GNOSIS_SERVER_OLLAMA_URL`, then `GET /engine/status` | **PASS** — `state:"Ready"`, `subsystems.vector == true`, `embedding:true`, `reranker:false`, core `true` |
| **(ii)** | `POST /rag/query` with `{"query":"<q>","mode":"vector"}` on that boot | **PASS** — **HTTP 200** `RagResult` with the vector trace, `engine:"gnosis"` |
| **(iv)** | a **separate provider-absent spawn** (provider env removed), and the configured-but-unreachable spawn as the contrast | **PASS** — the flag is `vector:false` and the `mode=vector` probe is **503 `engine_unavailable`** (FS-8), *not* FS-14; this is the contrast **F1** corrected (below) |
| **(iii)** | seeded-corpus membership | **PARKED** — reason re-confirmed live: the bin's boot store is in-memory and empty, its only seeding surface is its own CRUD routes (which run after the boot build) and U5 has no rebuild vehicle |
| **(v)** | the failed-`Reachable`-build branch | **PARKED** — reason re-confirmed live; see 3 below |

**2. `R-L2`'s amended cell — re-verified live in the same pass.** On a provider-reachable boot the
`HealthReport` reads `state:"Ready"` with `{store:true, graph:true, lexical:true, vector:true,
embedding:true, reranker:false}`, and the `mode=vector` probe on that process returns **200** — i.e. the
amended U5-time cell (`vector:true`) is the correct one, and the row no longer reads "re-run owed". The
row's 2026-09-17 executed-live record (`:186-208`) stands unchanged as the **U3-time** observation.

**3. The failed-`Reachable`-build branch (v) is PARKED, and the reason is structural.** The bin's boot
constructs an in-memory, **empty** store (`src/bin/gnosis_server.rs:381`), so a reachable provider's build
walks a **zero-node** corpus and makes **zero** `embed` calls ⇒ `Ok(Some(VectorIndex::default()))` — an
empty index, **not** an `Err` (the code's own step (1)/(2)/(3) comments at `src/store/mod.rs:5373-5413`
pin exactly this). A controlled endpoint answering `/api/tags` 200 and `/api/embed` 500 therefore yields
`Ready` + `vector:true` over the empty index, and `mode=vector` on that READY process fails only at **query
time** inside the query's own embedding ⇒ **503 `embedding_unavailable`** (FS-13). This is the same
conclusion `p2` §9.5.5's POST-GREEN SPEC AMENDMENT item 5 and its adjudication note 1 extension (a)–(d)
record; the criterion's lib-level home is `P-IM-15` / the conformance layer's state 4. **The branch is NOT
live-verified and must not be reported as such.**

**4. The FS-14 READY-index-free mapping is PARKED as structurally non-reachable through the bin post-U5.**
`p2` §9.5.5's valid/fail **state 6** (`Err(StoreError::VectorIndexUnavailable)` ⇒ 503
`vector_index_unavailable`, FS-14) is reachable only on a **READY** store whose snapshot has
`vectors: None`. Post-U5 the bin produces exactly **one** READY boot — the `Reachable` one — and that boot
**always** installs an index (`Ok(Some(vi))`, an empty index included, `src/bin/gnosis_server.rs:414-419`),
while every non-`Reachable` boot is non-READY and therefore answers FS-8. So **no boot of this bin can
reach state 6**: the mapping's live home is structurally empty and its home is the **lib-level** instance
the spec itself names (`tests/rag_query_integration.rs:689-716`'s caller-built READY store, plus the U5
blind set's `U5-17`/`U5-18` (a)–(b) and `U5-16`'s FS-8 contrast). **Parked, not failed:** if a future boot
path can be READY without an index (a provider-reachable boot that attempts no build, or a rebuild/durability
vehicle), the mapping reverts to the live criterion **unchanged**.

**5. Finding F1 (doc-drift), and how it was fixed.** The live run found the row's **(iv)** and **(v)**
clause text asserting that a provider-absent / non-READY boot's `mode=vector` ⇒ **503
`vector_index_unavailable`**, while the bin returns **503 `engine_unavailable`** (FS-8) — the reading
`p2` §9.5.5's `P-SM-7` and §5.9's precedence bullet already pin (the pre-READY gate precedes every leg
check for non-`graph` modes). **Fixed in place in this pass, with the superseded wording kept as the dated
record:** the (iv) cell, the (v) cell and the row's FAIL-set sentence now read FS-8 `engine_unavailable`,
each carrying its `*superseded wording, kept as the record*` marker; the mirror-image error (a live run
returning `vector_index_unavailable` on a non-READY boot) is recorded as a **spec-drift finding, not a
pass**. Two stale code citations in the same cells were re-pointed: the index-build call is
`src/bin/gnosis_server.rs:414` (the `:404` spelling landed in the U5 comment block that introduces the
call, `:403-413`), while `Store::new()` at `:381` was already correct. **No `src/`, `tests/` or other spec
file is edited by this section.**
