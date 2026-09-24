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

## U1 — contract amendment (2026-09-16)

- **Unit:** **U1** of the inbound Astrographer feature-request set (GR-1..GR-9) — the **contract
  amendment** for the `gnosis-server` host. **Docs-only**: this pass writes
  `docs/specs/p2-gnosis-server.md` + `docs/specs/engine-wire-contract.md` + one `docs/HANDOFF.md`
  reconcile row. **No `src/`, no `tests/`, no `Cargo.toml`, no cargo run.**
- **Authority:** the gate-1 record `docs/specs/gnosis-gr-inbound-review.md` — its §Per-GR verdict
  table, §The four rulings, §Ordered workstream (the U1 row), §Blast radius + reconcile order,
  §Regression watch, and its **§Go-ahead record: U0 landed + U1 AUTHORIZED (2026-09-16)**. The two
  gate decisions (`GNOSIS-CHANGE-CURSOR`, `SUBSYSTEM-FLAG-CAPABILITY-SEMANTICS`) are **ACTIVE** rows
  in `docs/decisions.md`.
- **The implementing units are NOT yet authorized.** **U2** (query POST contract), **U3** (status
  honesty), **U4** (change cursor + paged reads + `GET /changes`), **U5** (boot vector-index build)
  remain **HELD**; nothing in this amendment authorizes their code. Every code-bearing statement
  below names the unit that owes the code, and every such statement is marked **U2/U3/U4/U5-time**.
  **SUPERSEDED (2026-09-16): U2 and U3 are AUTHORIZED** — the user authorized the two code-bearing units
  and this file §9.5 now carries their **authored** typed registers (§9.5.1 U2 = 7 rows, §9.5.2 U3 = 5 rows,
  execution plan §9.5.3), so for U2/U3 only the **code** is owed; **U4 and U5 remain NOT authorized**. The
  U1-amendment clause above stands as the historical record of what *this amendment* authorized.
  **LANDED (2026-09-17): U2's code LANDED REALIZED-GREEN and U3's code LANDED-GREEN** — so **no U2/U3 code is
  owed any longer**; **U4 and U5 remain the only NOT-authorized code-bearing units** (U5 owns the boot
  vector-index build, i.e. the one-value `vector:true` flip). This clause is the current reading; the two
  clauses above stand as the authorization record.
- **What changed in this file (all additive; existing numbering preserved, sub-sections added):**
  - **§2** — three new `NOT in scope` bullets (the refused revision route + `stale_revision`; the
    transport-level decode codes are not §11 rows; the route-growth amendment rule).
  - **§5.2** — the routing contract **restated as a growth invariant**; the **"exactly 14 rows"
    fixed-count clause is DELETED** (the only outright deletion U1 makes — a fixed count is not an
    invariant once routes may be added; the **14-row table stands as the current state at U1**).
  - **§5.3** — the `POST /rag/query` request/response contract (envelope-strict, camelCase
    `ragQuery` options, bare-`RagResult` response).
  - **§5.4** — the **single mode rule** shared by the POST and the SSE path.
  - **§5.5** — the **structured transport request-decode error body** (the NEW-2 body).
  - **§5.6** — the **change cursor** + the `GET /changes` SSE change feed (contract pinned here;
    code lands in **U4**).
  - **§5.7** — the **bounds discipline** for the future paginated read routes (**U4**).
  - **§5.8** — the subsystem-flag capability semantics (contract pinned in F2 §9.1; **U3 owed the
    code and LANDED it 2026-09-17**, **U5** still owes the index build).
  - **§5.9** — the **FS-3 vs FS-13/14/15** engine-side reconcile (explicit-leg errors, fusion degrades).
  - **§7.2** — the NEW-2 decode-outcome table annotated with the transport code strings (§5.5) and the
    shared-rendering statement.
  - **§10** — the `/rag/query` + `/rag/stream` rows and the cross-cutting fail-states extended.
  - **§11** — **`P-IM-3` RESTATED** as the growth invariant (no fixed count) + its generator-coverage
    note + register notes naming the units that owe their own typed rows.
  - **§12/§13** — cross-references and the does-NOT list extended.
- **Zero new register rows land in U1.** This unit is **contract-only**: it **restates** `P-IM-3` in
  place and adds **no** property/register row. Justification: a docs-only unit adds no behavior of its
  own to test; the **code-bearing units each owe their own typed §5.x rows** in their own spec (see the
  register notes in §11) — **U2** owes rows for the shared query decoder, the **`filters` canonical-token
  mapping**, the **token resolver** (`mode`/`expand`/`compression`), the **wrongly-typed ⇒ absent** rule,
  the SSE fail-state status and the transport decode-code mapping; **U3** owes a row pinning the
  status-flag derivation (a flag is `true` only when the subsystem's full query-time capability is wired
  and functional for the current store); **U4** owes rows for the cursor's monotonicity/step-1
  semantics, the change-feed frame ordering, the `kind` tolerance/rename binding and the page-cap
  bounds; **U5** owes the boot-index-build row. This is the **justified zero-row
  exemption** for a docs-only unit (per decision `PBT-GATE-MANDATORY`), matching the gate-1 workstream
  row's treatment of U0/U1. **(The remand does not change this: the rows it adds to §11's owed-table are
  sketches of U2's/U4's obligations — no `Property-id` is claimed by U1 — so U1's register stays at its
  original 7 rows.)**
- **Known transient divergence (deliberate — recorded so it is not read as drift; RESOLVED by U3, 2026-09-17).** F2's §12 V-8
  golden bodies are **restated in U1's contract text** for the post-U3 flag semantics, while the flag
  **code** change lands in **U3**: until U3 landed, `tests/wire_conformance.rs:1024-1059` still asserted
  the pre-U1 literals and still **passed**, because the code still emitted them. The literal edit landed
  **in the same unit as the flag change** (U3), per the gate-1 blast-radius row — the test now asserts the
  amended literals at `:1061-1112` and the producer coupling is pinned in the same file
  (`boot_wiring_couples_to_the_derived_read`, `:1129`). Same shape for F2 §9's
  V-8 discussion.
- **The regression guards this amendment honors** (gate-1 §Regression watch): **unknown payload keys
  are tolerated** and **absent `mode` ⇒ `Flat`** (§5.3/§5.4), so the existing e2e POSTs that carry an
  extra `args` key (`tests/gnosis_server_e2e.rs:188-198`,
  `tests/blind_p2_gnosis_server_greens.rs:437-447`) still reach the READY gate and still return
  **503**; the token check is **purely additive at the wire layer** (the U2 resolver of §5.4 — **not**
  an arm inside `validate_rag_options`, whose checks and their internal order are unchanged,
  `src/store/mod.rs:4330-4375`; the pre-U3 citation was `:4293-4338`; **superseded-by-the-remand** for the pre-remand "added additively to
  `validate_rag_options`" reading, F2/§5.3); and the §11 map stays **21 rows** (§7.2/§5.2).
- **Docs-only scope confirmation:** no `src/`/`tests/`/cargo change; no edit to the canonical
  `docs/specs/gnosis.md` (its reconcile request is a `docs/HANDOFF.md` row) and no edit to the frozen
  P1a `docs/specs/p1a-document-crud-wire.md` shapes; no new property rows.
- **U1 REMAND (2026-09-16) — the spec-gate reviewer returned 4 MUST-FIX + 7 SHOULD-FIX + 6 NOTES; all 17
  are addressed in place by this pass** (the finding-by-finding disposition is reported with this
  remand; no finding is *deliberately* left unaddressed). The substantive changes this remand makes to
  the pinned text are: **(F1)** `filters` now pins the **canonical §4.5.2 shape**
  (`docs/specs/gnosis.md:692-694`) and **NOT** the store's serde casing, and is explicitly **not**
  passed through to `QueryAuditFilters` (§5.3); **(F2)** the `mode`/`expand`/`compression` **token**
  check is relocated to the **U2 wire resolver only** (the option fields are typed enums, so no
  unrecognized token can reach `validate_rag_options`), the `expand` 400 is annotated as a **new state
  created by this amendment** (FS-3's canonical enumeration, `docs/specs/gnosis.md:1027`, lists
  `mode`/`compression` but not `expand`), and the upstream ask now requests that addition (§5.3/§5.4);
  **(F3)** §5.9 extends the narrowing to **`docs/specs/gnosis.md:872`'s `ragQuery` throw column and
  FS-13's `hyde: true` clause** (`:1037`); **(F4)** the SSE mode fail-state's HTTP status is pinned to **400**
  (§5.4/§10); **(F5)** present-but-wrongly-typed option values are pinned as **absent** (⇒ documented
  default) with `multiQuery.n`'s default pinned; **(F7)** the permitted additive status signal is
  scoped to **`HealthReport`**, and **`EngineSubsystems` (a canonical §4.6.1 store type) MUST NOT gain,
  lose or re-type fields**; **(F8)** the `docs/HANDOFF.md` reconcile ask is extended (cursor +
  `GET /changes` + the flag-capability semantics) and the older FS-13/14/15 row is annotated
  **SUPERSEDED (engine-side landed by U1)** without deletion; **(F9)** the companion F2 contract's
  refused cursor semantics
  are narrowed to "no request-side **as-of-N / point-in-time** cursor semantics" and a U4 page token is
  pinned as a **resume token** with defined not-comparable behavior; **(F10)** its §4.4 closed
  event-type set is annotated as the **retrieval** family; **(F11)** the change feed's `kind` gains an
  **unknown-token tolerance** rule and a conformance binding on renames. The NOTES (F12–F17) are
  applied where they belong: **F12**/`F13`/`F15` and **F17(ii)/(iii)** in
  `docs/specs/engine-wire-contract.md` (§7, §7.1, §12), **F14** in its §4.6, **F16** in its §2, and
  **F17(i)** here (§12's renumbered ownership list).
  **No pinned content that is still true was deleted**: superseded statements are marked
  **Superseded-by-U1 / Superseded-by-this-remand** in place.
- **U1 REMAND-2 (2026-09-16) — the verification reviewer confirmed F1–F17 (F5 PARTIAL) and returned
  one MUST-FIX (N1) + two SHOULD-FIX (N2, N3) + three NOTES (N4–N6); all six are addressed here and in
  `docs/specs/engine-wire-contract.md`.** The substantive changes: **(N1)** the `expand`/`compression`
  token rule's "on **both** paths" clause is deleted — `mode` is the **only** token with a POST **and**
  an SSE outcome, `expand`/`compression` are **POST-payload-only**, and `?expand=`/`?compression=` on
  `/rag/stream` is **ignored ⇒ the option's default, never a 400** (§5.4; the SSE fail-state rows of
  §5.3/§5.4/§10 and F2 §4.5/§13 are aligned to the mode-only claim), while §5.4 **names the pre-U4 SSE
  surface** (`query`/`topK`/`mode` exactly) and assigns the other seven canonical `ragStream` params to
  **U4's** spec (not an upstream ask); **(N2)** `expand:5`/`compression:null` join the wrongly-typed
  corpus (§5.3/§10/§11 and F2 §13) and the token rule now states that **only a string reaches the token
  check**; **(N3)** `docs/HANDOFF.md`'s addendum now states row 51's relationship explicitly
  (superseded-by-this-addendum for its canonical-ask half; its query-body half lives on); **(N4)** a
  per-option **reachability** note lands in §5.3 (which mode consumes which option; `subTaskDag` inert,
  no read site) + a mode qualifier on the `binaryFirstPass`/`binaryCandidatePool` fail-state row;
  **(N5)** §5.3 pins `filters: {}` ⇒ `Some(all-None)` **present/valid/honored** and states that the wire
  surface does **not** expose `nodeKind:'community'` (store-API-only); **(N6)** the F7 blast-radius
  citation is corrected to **nine** struct literals (the `:281` signature is excluded). **No pinned
  content that is still true was deleted; the register is still exactly 7 landed rows and the owed-table
  sketches stay id-less.**
- **Zero new register rows still holds after the remand**: the F1–F17 fixes **restate or annotate**
  pinned text; none adds a behavior, a route, a §11 row, a register row, or a U2/U3/U4/U5 contract.
  **N1–N6 likewise**: each **restates, narrows or corrects** an existing pin (and the two N1 clauses are
  a **deletion of an unreachable claim plus a named scope boundary**), so **zero** property rows are
  added — §11's property table stays at **7** `Property-id` rows and the register notes' owed sketches
  stay **id-less**.

- **Bookkeeping pass (2026-09-16; docs-only — the closing verification's 4 must-fix findings, fixed in
  place with the historical text retained).** The closing verification of the U2/U3 register pass returned
  **4 small must-fix bookkeeping findings**, all recorded here as fixed in place: **(B1)** the F2 header's
  pre-authorization clause (`docs/specs/engine-wire-contract.md:29`, and the in-kind parenthetical at its
  `:124`) is annotated with the same dated supersession marker as §U1's — clause now reading **U2/U3
  AUTHORIZED (2026-09-16; registers authored in `p2` §9.5.1/§9.5.2); U4/U5 NOT authorized**;
  **(B2)** Appendix B's status footer in `docs/specs/gnosis-gr-inbound-review.md` (previously `:642-648`)
  is annotated with the dated supersession sentence, so that file's own post-record note (`:18-22`) is no
  longer falsified; **(B3)** §9.5.3.2's item 6 parenthetical is corrected so **all three accounts** of the
  **nine** (§9.5.1's consolidation record, §11's UPDATE sentence, item 6) name the **same ninth row** (the
  `QueryAuditFilters`-touching mapping row the first register remand added) and the **same three
  pre-remand omissions** (`P-IM-5` wrongly-typed, `P-TP-2` split, `P-SM-4` parity); **(B4)** the **≤400
  case cap's scope is pinned per-unit** in §9.5.3 (and quoted in §11 and §12 item 7): **each unit's
  property layer ≤ 400 — the landed `p2` layer 310 and the U2 layer (245 as the pre-landing indicative
  budget, **400** as the landed executed layer — §9.5.3) each satisfy
  it; the binary total is the sum of the units' layers and is not itself capped**, with decision
  `PBT-GATE-MANDATORY` cited as the authority. **No pinned rule, register-row substance or row count
  changed; no row was deleted; no per-row cap was altered; every superseded clause was kept and annotated
  in place.** The row counts stand at **U2 = 7 / U3 = 5** (**twelve** injective tags over §9.5's twelve
  rows — 7 + 5 — §9.5.3; the pre-landing "ten" count was an arithmetic slip, corrected by the proofread
  pass), and the U4/U5 owed
  sketches stay **owed and id-less**.

- **U2/U3 typed-property-register pass (2026-09-16; docs-only, ADDITIVE — no U1 text rewritten).** The user
  **authorized U2 and U3** (the two code-bearing units). Per decision `PBT-GATE-MANDATORY`, each
  code-bearing unit must carry its own typed §5.x register **before** a TestWriter may derive tests, so this
  pass authors the rows U1 deliberately left as **id-less owed sketches** (§11's register notes): **U2 ⇒
  §9.5.1** (7 rows: `P-IM-4`, `P-IM-5`, `P-IM-6`, `P-SM-4`, `P-TP-2`, `P-TP-3`, `P-TP-4`) and **U3 ⇒ §9.5.2**
  (5 rows: `P-IM-7`, `P-IM-8`, `P-IM-9`, `P-SM-5`, `P-SM-6`), plus the shared execution plan (**§9.5.3**),
  the per-row coverage notes (**§9.5.4**) and the §11 owed-table annotations. **Every U1-pinned rule above
  stands unmodified** — the mode rule, the
  `filters` canonical mapping, the token-location (F2), the wrongly-typed rule (F5), the SSE 400 pin (F4),
  the POST-payload-only split (N1), the transport decode body (F15), the capability semantics and the frozen
  `EngineSubsystems` rule (F7) are **reused as written**, and no row re-scopes them. The U1 register (§11)
  stays at its **7 landed rows**; the U4/U5 owed sketches stay **id-less and untouched** (those units remain
  unauthorized). The items the frozen contract could not be turned into a faithful **pure** row are
  recorded as **adjudication notes** — four in §9.5.1 (the bin-level handler pass-through, now **named as a
  live obligation**; the bin-level shared rendering; the store-API-only `nodeKind: 'community'`; the
  `mode`-as-exception reconciliation) and two in §9.5.2 (the boot branch lives in
  the bin, so `P-IM-9`'s assertion splits across a **lib-visible boot-wiring function** and a named
  live-battery half; §6's prose state set is wider than the boot
  path's reachable set, so no generator may assert the unreachable state) — recorded, **not** invented, and
  **not** U1 re-opens (none of them contradicts a U1 pin).
- **U2/U3 REMAND — the register reviewer's 11 MUST-FIX + 4 SHOULD-FIX + 1 NOTE are addressed in place
  (2026-09-16; docs-only).** The reviewer (read-only) audited §9.5 and returned **F1–F11 (must-fix),
  F12–F15 (should-fix) and F16 (note)**; §9.5.3.1 records the **row-by-row disposition** and the full
  resolution. The substantive changes: **(F1)** U3's three IM rows are **renumbered** `P-IM-5/6/7` ⇒
  `P-IM-7/8/9` (a within-file id collision that also collided the seed tags and listed nine tags for ten
  rows), with every citation moved and the tag list re-derived injective; **(F2)** the execution budget is
  corrected to `7×35 + 5×25 = 245 + 125 = 370 ≤ 400` (the pre-remand 405 exceeded the cap it claimed to
  respect) — **that 370 is the pre-landing indicative budget; the executed U2 layer landed at 400
  (`48/100/70/80/23/41/38`, `tests/props_gnosis_server.rs:26-40`), see §9.5.3**; **(F3/F9)** `P-IM-4` is restated over two disjoint sets (envelope/transport ⇒ `Err`;
  everything else ⇒ `Ok` + §5.3's mapping) with **one** transport code pinned for a non-object `payload`;
  **(F4)** §5.3 gains a **per-key wrongly-typed table covering all 14 keys** and `mode` appears exactly once
  (a non-string ⇒ absent ⇒ `Flat`), with `P-IM-5` citing the table; **(F5)** `P-IM-6` pins the casing policy,
  the wrongly-typed `target` member, `target` extras and unknown members (`nodekind`, the store's serde
  spellings); **(F6)** the decoder takes the **SSE params** as a named input
  (`SseParams { query, top_k, mode }`) so the SSE half of the token rule is expressible;
  **(F7)** `P-IM-5`'s domain is bounded and its path quantifier restricted to the keys each seam carries;
  **(F8)** `P-TP-3` is split — the pure half stays in the lib, the rendered-body half moves to the
  conformance layer with the **named** golden assertion (`v15_request_decode_error_body_exact`, F2
  V-15/V-15.1) and the message contract restated (verbatim, may be empty);
  **(F10)** `P-TP-2` is purely the decode-level identity **and** names its live obligations (α e2e, β live
  battery); **(F11)** `P-IM-9` requires a **lib-visible boot-wiring function** with a pinned signature and
  names the provider-reachable outcome as live-battery-only; **(F12)** `P-SM-6` is the `health` faithfulness
  projection only (the frozen shape moves to `health_report_shape_frozen`, and **no `HealthReport` field
  lands in U3**); **(F13)** `EngineState`'s readiness axis vs the capability flags is pinned (with a
  `docs/defects.md` row for the divergence); **(F14)** §9.5.4 adds a coverage note per row; **(F15)**
  `P-SM-4` names its live wiring assertions (`?mode=bm25` ⇒ 400; `?mode=HYBRID` ⇒ non-400);
  **(F16)** U3's mechanism is pinned as a **read-time derivation inside `get_engine_status`**. **No row was
  deleted, both row counts are unchanged (7 + 5, ≤8 each), no id was reused for a different claim, and no
  landed U1 rule text was edited.**

- **U3 LANDED-GREEN (2026-09-17; the status-honesty unit — annotation only, no U1 rule edited).** The
  read-time flag derivation, the lib boot seam (`boot_wiring`), the bin boot, the inert `set_subsystems`, the
  unchanged six-`bool` `EngineSubsystems`, the unchanged `HealthReport` and the amended V-8.1/V-8.2 goldens are
  all **landed** (§9.5.2's landed bullet records the verified state: five rows HELD, 127 executed ≤ 400, blind
  set 13/13, live battery 9/9 incl. `R-L2`, `cargo test` **604 passed / 0 failed** serial — green **with and
  without** `GNOSIS_SERVER_OLLAMA_URL` set, `fmt`/`clippy`/`build` clean). The U3-time citations in this file
  that drifted when the unit's in-tree comments shifted the cited sites (the `subsystems` field, the legacy
  all-`true` literal, `set_subsystems`, the `get_engine_status` derivation, the boot's `main()`, the
  `tests/wire_conformance.rs` V-8 fixtures and the `tests/props_gnosis_server.rs` tag/seed constants) are
  **re-pointed in place with their pre-U3 numbers kept as provenance**. Two U3-time findings are filed **OPEN**
  in `docs/defects.md` (**P-8** post-boot provider loss is invisible to `/engine/status`; **P-9**
  `encode_result`'s `expect` panic path) — neither is a U3 regression. The U3 **DONE** row and the
  documentation-review record are **not** written by this pass (gate 8 owns them).

- **U5 AUTHORIZED + spec gate (2026-09-22; docs-only, ADDITIVE — no landed clause rewritten).** The user's
  go-ahead record (`docs/specs/gnosis-grq-inbound-review.md` **§14 POST-RECORD UPDATE 1**, **Q3 = "(B) U5
  ONLY"**) authorizes the **boot vector-index build** as the first code-bearing unit since U2/U3. That
  record's §15.2/§15.4 already named U5's same-unit obligations; **§5.8 and §9.5 carried U5's obligation row
  while its typed rows stayed owed and id-less** (§11's register notes, §9.5's header). This pass is U5's
  **spec gate**: it authors **(a)** U5's contract and **(b)** U5's typed §5.x register in **§9.5.5** (8 rows,
  `P-IM-10`…`P-IM-15` / `P-SM-7` / `P-TP-5`, ≤ 8 ✔), plus that unit's **execution plan**, **per-row
  coverage notes** and the "**must move in the same unit**" table. **Every earlier clause stands as
  written**: no U1 pin, no U2/U3 register row's substance and no row count changes (U2 = 7, U3 = 5, U5 = 8,
  each ≤ 8; the ids continue the single per-file sequence — the highest p2 id claimed before this pass was
  `P-IM-9`/`P-SM-6`/`P-TP-4`); **U4's owed row stays owed and id-less** (U4 remains **HELD**); and U5's
  authorization changes **nothing** about the §11 map (21 rows), the `StoreError` taxonomy, the route table
  or the frozen canonical types.
- **U5's same-unit edits outside this file (recorded here so the supervisor's reconcile order is complete;
  2026-09-22).** **`tests/wire_conformance.rs`** — the V-8.1 `"vector"` literal, the
  `honest_ready_subsystems()` fixture it projects and the `boot_wiring_couples_to_the_derived_read` probes
  (§9.5.5's move table, with `file:line`; **REMAND-1 pinned those lines exactly: only `:1195`, `:1204` and
  `:1235` move/changed — probes (2)/(3) at `:1209`/`:1218` and probe (4)'s assertions stay**; ***REMAND-2
  NOTE 6 / REMAND-3 NOTE 6, 2026-09-22 — read this record's `:1235` as the redundant-line **drop**, not a third
  *value* edit: the pinned scope is **exactly two** value edits (`:1195` → `boot_snapshot(true)`, `:1204` →
  `reached.vector`) plus the two prose reconciliations (`:1205`/`:1191`), §9.5.5's move table***). ***(Gate-8
  re-read, 2026-09-22 — the landed anchors, so the U3-time numbers above cannot be mistaken for current: fixture
  `:718` (doc comment `:708-717`), the V-8.1 literal `:1109` (message `:1110-1111`), probe (1) `:1208-1223`
  (input `:1213`, assertion `:1222`, message `:1223`, comment `:1208`), probes (2)/(3) `:1228`/`:1237`, probe (4)
  `:1244-1256` (input `:1250`, `u5_expected` `:1253`). The three U3-time numbers above are superseded by those
  and are kept as the REMAND record.)*** **`docs/specs/engine-wire-contract.md`** — U5-labelled notes on
  §9.1's `vector` flag row and on §12's V-8.1/V-8.2 stage labels (its **type** rules, the frozen
  `EngineSubsystems`/`HealthReport` shapes and the §11 map are **untouched**). **`docs/specs/p2-gnosis-server-live-pending-battery.md`**
  — the live battery's **`R-L2`** row carries the pre-U5 `vector == false` expectation and would otherwise
  go red on the next live run, so U5's dated note flips that cell and adds its own row **`R-L3`**
  (the boot-build live obligation); the **M1–M20** rows and §2's revisit condition are **not disturbed**.

- **U5 POST-RED-PHASE REGISTER AMENDMENT (2026-09-22; docs-only, ADDITIVE — no landed clause rewritten, no
  row added/deleted/renumbered).** The spec gate returned **EMPTY (round 5)** and U5's red stage has now run
  (15 tests in `tests/u5_boot_vector_index_conformance.rs` plus the eight register rows' `#[test]`s and a
  budget-discipline test in `tests/props_gnosis_server.rs`); the TestWriter surfaced two spec-level items
  that this pass fixes **in the spec**, not in a test. **§9.5.5's post-red-phase register amendment** is that
  artifact, and it (i) **amends and reconciles the caps** — per-row `60/50/40/45/45/30/45/40 = 355 ≤ 400`,
  each ≤ 100, **a cap being a maximum rather than an expected count**, with the measured layer **314
  executed / 355 caps** (the landed `P-IM-10` corpus needs **55** cases: 9 corpus variants × 5 provider
  shapes + 10 `p == None` cases, so the pinned 45 was unachievable as written — the `340`/`1177` figures are
  kept everywhere as **annotated superseded records**, and the amended total is `1192`); (ii) **pins the
  `NaN` comparison rule** the two determinism/verbatim rows left implicit (contract table (2)'s new
  *vector comparison / `NaN`* row: same `len()` + non-`NaN` elements bit-for-bit + `NaN` asserted only in
  the `NaN` position + no tolerance; `P-TP-5`'s corpus keeps its deliberate `NaN`/`±∞` shapes); (iii)
  **corrects the move table's count** (it is **11 lines = a header + a separator + 9 data rows**; the dated
  "12 rows" reading was a count slip with nothing missing, and with the new Implementer-obligation row the
  table stands at **10 data rows**); and (iv) **records the `src/store/mod.rs` derivation-site comment**
  (its *"`false` until U5's boot index build"* wording at `:4211-4212`) as an **Implementer obligation in the
  same unit** — pinned in the same-unit table in §9.5.5 because it is not a test-artifact move and no
  test can observe it. **Every rule set is intact**: ≤ 100 per row, ≤ 400 per unit, the pinned seed +
  `row_seed(tag)` tags, stop-after-5, held/broken + `Strategy-id` reporting, and the one-pass remand rule;
  **all 8 row ids, kinds, strategy ids and tags are unchanged and no new authority is claimed** (no new §11
  row, `StoreError` variant, wire code, route, accessor or frozen-shape change). This pass **edits only this
  file** — no `src/`, no `tests/`, no other spec, and no tracker.

- **U5 POST-GREENS VERIFICATION STATUS (gate 8, 2026-09-22; docs-only, ADDITIVE — the blocks above stand as
  the pre-green records).** U5's code **LANDED-GREEN** and the full gate chain has now run, so the unit's
  status marker reads **LANDED-GREEN (gate 8 reconciled 2026-09-22)** everywhere §9.5.5's rows are cited.
  **What was verified (against the landed tree, this pass — read, not re-run):** `build_boot_vector_index`'s
  signature and body (`src/store/mod.rs:5369-5414`) match §9.5.5's surface table row 1 exactly (three
  outcomes; `Ok(None)` iff no provider; `Ok(Some(vi))` with `entries` possibly empty; `Err` only
  `StoreError::EmbeddingUnavailable`; `FieldType::Full` only; one strictly-sequential `embed` per embeddable
  node; no env var, no availability probe); the name is re-exported lib-side (`src/lib.rs:120-126`); the
  bin's boot wiring (`src/bin/gnosis_server.rs:381-437`) implements the pinned order — build at `:414`, the
  composed snapshot at `:416-419`, the failed-`Reachable` case degrading through
  `boot_wiring(BootProvider::Unreachable, …)` with the returned flag vector discarded (`:425-432`), then
  `swap_snapshot` + `set_embedding_provider` (only when wired) + `set_engine_state` (`:433-437`); and the
  derivation-site comment obligation is **discharged** (`src/store/mod.rs:4210-4215` now states the
  post-U5 predicate). **Numeric claims reconciled:** the governing set is the **amended** one — caps
  **355 ≤ 400** (`60/50/40/45/45/30/45/40`), measured layer **314 executed**, binary total
  **`310 + 400 + 127 + 355 = 1192`**; the `340`/`1177` pair appears only as an annotated superseded record.
  **Test-count claims:** `cargo test` **655 passed / 0 failed** serial (baseline 604, +24 U5 in-crate, +27
  blind), hermetic with and without `GNOSIS_SERVER_OLLAMA_URL`; `cargo fmt --check` exit 0; `cargo clippy
  --all-targets` 0 warnings; `cargo build` clean. **Layer boundaries restated so nothing is over-credited:**
  the in-crate rows cover the envelope/pure/lib surfaces; the blind set
  (`tests/blind_u5_boot_vector_index_greens.rs`, 27/27) adds two live-HTTP rows; the **gate-6 live pass**
  covered the assembled/server HTTP surface only, and the **failed-`Reachable`-build branch is lib-level
  only — its live criterion is PARKED, not live-verified**. **What stays OPEN (not closed, not weakened):**
  `U5-ADV-1`…`U5-ADV-5` and `P-9`'s **fired trigger** in `docs/defects.md`, and the PBT audit's **T1–T10**
  negative-generator list as an undis-charged **TestWriter** obligation. Review record:
  `archive/reviews/2026-09-22-u5-boot-vector-index-doc-review.md` (gitignored provenance).

- **U2 red-phase ambiguity pin (2026-09-16; docs-only, ADDITIVE — no pinned rule rewritten).** The U2
  TestWriter's triage surfaced **one** ambiguity in the landed U2 contract: §5.3's 14-key table "absent ⇒"
  column and §5.4's "absent ⇒ `Flat`" cell were readable as **field-level** decoder outputs (`Some(Flat)`),
  which would contradict `P-IM-5`/`P-TP-2`'s own element-wise `== None` wording. This pass **pins the
  two-layer reading once** in §5.3 (the new bullet naming layer 1 = the wire decoder's `Option`-preserving
  `None`, layer 2 = the engine's query-time default, with the "well-formed string ⇒ `Some(token)`" clause)
  and **cross-references it** from §5.4 (the "how to read this table" paragraph preceding the mode table,
  plus the boundary-state parenthetical). The companion F2 item is the **V-15.1** binding note in
  `docs/specs/engine-wire-contract.md` §12 (the `message` string is the implemented renderer's own output;
  code/status/header/keyset are the byte-binding pins). **No other pinned rule changed**: no register row was
  added or deleted (§11's property table is still **7** `Property-id` rows, §9.5.1/§9.5.2 still **7 + 5**),
  no §11/§12 row or cross-reference moved, and §5.3's exception set (`query`, `filters`, the non-member-string
  token ⇒ `ValidationError`) is unchanged. F2 §7.1's `message` rule is **not** edited by this pass.

- **U2 adversarial-pass corrections (2026-09-16; docs-only, note/cell corrections — NO rule changed).** The
  post-green adversarial pass on U2 returned **three spec-side must-fix items**, all corrected in place:
  **(A)** §9.5.4's `P-IM-6` coverage note is restated — it had asserted `Some(all-None)` for the store's own
  serde shape (`{"node_kind":"Content","target":["d1","n1"]}`), contradicting §5.3's shape rule and `P-IM-6`'s
  own row text (a shape-mismatched `target` ⇒ `Err(Validation)`), and it failed to separate the two different
  negatives; the note now names its instances precisely (store-serde **shape** mismatch ⇒ `Err(Validation)` =
  the **D3** pass-through guard; a genuinely unknown member **name** with a valid value ⇒ `Ok(Some(all-None))`);
  **(B)** §9.5.4's `P-TP-4` coverage note is corrected from three rejected `RagResult` classes to the **two**
  representable ones (an `engine` ≠ `"gnosis"` — including the empty string — and a `blocked_by` without a graph
  trace), because `RagResult.trace` is **not** an `Option`, so a traceless result is **not representable in
  Rust** and is **not** a coverage gap; **(C)** §5.3's per-key table `query` cell (and the two-layer bullet's
  `query` exception) is restated so the two halves are explicit and cannot be read against `P-IM-4`'s
  "(B) every other input ⇒ `Ok`" clause: a **present non-string** `query` ⇒ decoder
  `Err(QueryDecodeError::Validation(ValidationError))`, an **absent** `query` ⇒ `""` at layer 1 ⇒ the engine's
  FS-3 **400 `validation_error`** at layer 2 — with the four other clauses that stated the old single half
  reconciled to the same two-half reading (`P-IM-5`'s row text parenthetical, the wrongly-typed bullet's
  `query` parenthetical, §5.3's "Non-string or absent `query`" bullet, and §10's cross-cutting exceptions
  list; §10's fail-state-table cell
  `query` absent/empty/non-string stays, since it claims no layer and its 400 outcome is unchanged). All three
  items are **pin clarifications**: no register row was added or
  deleted (§11's property table is still **7** `Property-id` rows, §9.5.1/§9.5.2 still **7 + 5**, twelve injective
  tags), no cross-reference moved, no pinned rule was substantively changed, and the **wire outcome is
  unchanged** (400 `validation_error` on both `query` halves).

- **U6 AUTHORIZED + spec gate (2026-09-22; docs-only, ADDITIVE — no landed clause rewritten).** The user's
  go-ahead — *"open a follow-up honesty/freshness unit now"* (**2026-09-22**) — authorizes **U6, the
  vector-index freshness / honesty unit** (the follow-up the U5 landing records already pointed at:
  `docs/next-steps.md`'s U5 DONE row, `docs/pending.md`, `docs/HANDOFF.md`). This pass is **U6's spec gate**:
  it authors **(a)** U6's contract and **(b)** U6's typed §5.x register in **§9.5.6** (**6 rows**:
  `P-IM-16`, `P-IM-17`, `P-IM-18`, `P-IM-19`, `P-SM-8`, `P-TP-6` — ≤ 8 ✔), plus that unit's **execution plan**,
  its six **per-row coverage notes** in §9.5.4, its "**must move in the same unit**" table and its
  **reconciliation table** for the superseded clauses. **The unit's charter is exactly three of U5's
  adversarial findings, all OPEN in `docs/defects.md`:** **`U5-ADV-1`** (the primary: the boot index is
  frozen, so a post-boot write makes `mode=vector` a silent 200 with missing hits while `subsystems.vector`
  stays `true` — live-CONFIRMED at gate 6), **`U5-ADV-4`** (the build's `Err` discarded with no diagnostics)
  and **`U5-ADV-2`** (the pre-bind build has no *real* bound — `reqwest::Client::new()` carries no timeout).
  **Explicitly out of scope, with the reason (U6's own table states each):** **`U5-ADV-3`** (a
  spec/consumer decision that would move a §11/V-8.2 golden literal), **`U5-ADV-5`** (the lock-`unwrap`
  panic class — a store-wide decision in its own unit) and **`P-9`** (package/foundation — the F2 wire
  foundation's, with U5's verbatim-vector clause **not** to be patched). **The pinned honesty predicate is
  `subsystems.vector == snapshot().vectors.is_some() ∧ snapshot().epoch == self.epoch()`** — derived at read
  time, never stored; the boot's composed snapshot must carry `store.epoch()` so a fresh boot is not
  stale-by-construction; and the vector leg refuses on the **same** predicate (`Err(VectorIndexUnavailable)`,
  FS-14 ⇒ 503) rather than serving stale hits **— REMAND-1, 2026-09-22: "the vector leg" means the
  **explicit** sites (`vector_search`, `vector_query`); the **fusion** site consults the same predicate with
  the refuse **suppressed** (a stale index degrades the `hybrid` vector leg to empty and still returns 200),
  per §9.5.6's per-site consultation clause**. **Every earlier clause stands as written**: no U1 pin, no
  U2/U3 and no U5 register row's substance, id, tag, cap or count changes (U2 = 7, U3 = 5, U5 = 8, U6 = 6,
  each ≤ 8; the ids continue the single per-file sequence — the highest p2 ids claimed before this pass were
  `P-IM-15`/`P-SM-7`/`P-TP-5`); **U4's owed row stays owed and id-less** (U4 remains **HELD**); and U6's
  authorization changes **nothing** about the §11 map (21 rows), the `StoreError` taxonomy (21 variants), the
  route table (14 rows) or the frozen canonical types. **`docs/decisions.md`/`docs/defects.md`/
  `docs/pending.md`/`docs/next-steps.md`/`docs/HANDOFF.md` are NOT edited by this pass** (trackers are the
  supervisor's) — U6's out-of-scope dispositions and its **five open questions** are recorded in §9.5.6 for
  the supervisor to land.
- **U6's same-unit edits outside this file (recorded here so the supervisor's reconcile order is complete;
  2026-09-22).** ***(REMAND-1 correction, 2026-09-22 — this list is the red set's single authority and was
  re-read against the tree this pass: the U5 conformance anchors it named as "corpus-seeded snapshots" are
  **not** the moving ones, and two whole test files were missing. The corrected list is below; §9.5.6's
  same-unit table carries each site's obligation and line anchors.)*** **`tests/`** — the fixture rule and the
  named sites of §9.5.6's "must move in the same unit" table: the **three shared fixtures** with a hard-coded
  `epoch: 1` (`tests/rag_query_integration.rs:272-282`'s `seed_vector_index`, `tests/retrieval_stack_integration.rs:131-141`'s
  `seed_vectors`, `tests/props_retrieval.rs:246-256`'s `seed_vectors`) and their named read/assertion sites — **plus (REMAND-2 MUST-FIX 1/3, 2026-09-22) the sites that round re-derived:** the U5 conformance snapshots `tests/u5_boot_vector_index_conformance.rs:459-462`/`:517-520`/`:900-903` (moved to the **epoch-alignment** obligation: each also feeds a **corpus-seeded** store, per §9.5.6's same-unit rows) and `tests/retrieval_stack_integration.rs`'s **three INLINE** snapshots `:407-411`/`:516-520`/`:883-887` (with their `vector_search`/`mode=Vector` `rag_query` consumers, per that file's new same-unit row); the
  **corpus-seeded** U5 conformance snapshots `tests/u5_boot_vector_index_conformance.rs:517-520` and `:900-903`
  (state 2 and contract table (1) — **these two MOVE**, by the per-site rule: each label is *also* consumed by a
  corpus-seeded vector leg (`seed_store(&[])` epoch **`2`**; `seed_store(&mixed_corpus())` epoch **`3`**), while
  the label the fresh-store `wired_status` helper receives (`:377-393`, its own `Store::new()` at `:386`) stays
  epoch-`0` — so the fixture is **split**, not "aligned"; **REMAND-2 MUST-FIX 1, 2026-09-22: the pre-remand
  "the sites at `:459-462`/`:517-520` do NOT move" reading is superseded**) — **and its sibling `:459-462`
  (state 1) stays ALIGNED with NO edit** (its only consumers are `boot_wiring` `:463`, which reads no store, and
  the fresh-store `wired_status` `:470`; the corpus-seeded store of that test is consumed by the build
  assertions, and "aligning" this label to epoch `3` would make the helper's read stale and turn a passing
  assertion red) — plus the U5 state-7 snapshot at `:850-853` (single-label, `epoch: store.epoch()`); the U5 property-layer probes in `tests/props_gnosis_server.rs:7322-7361` (the
  `P-IM-14` corpus, re-derived per probe) and `:7823-7826`/`:7846-7849`/`:8069-8072` (the `P-IM-15`/`P-SM-7`
  corpus-seeded swaps), the `P-SM-5` mutation control at `:5367-5414` and the blind U3 populated-index scenario
  (`tests/blind_u3_status_honesty_greens.rs:479-484` with its `epoch: 1` fixture at `:157`, plus the
  pre-U6 predicate sentence in `assert_flags_match_predicates`, `:192-202`), plus the **new** stale-index case
  (**HOME PINNED to `tests/u5_boot_vector_index_conformance.rs` — REMAND-2 NOTE 10, 2026-09-22: the pre-remand
  list left the file open; no `file:line` claim depends on the choice, because the case is new, and a
  TestWriter moving it to a separate U6 file must record that at the same-unit table's row**) and the **new**
  six-cap `u6_layer_budget_discipline` test; **`src/`** — the boot's composed snapshot
  (`src/bin/gnosis_server.rs:416-419`, gaining `epoch: store.epoch()`), the stderr diagnostic before that
  composition, the provider client (`:327`, now `provider_client(PROVIDER_REQUEST_TIMEOUT)`) **and (REMAND-2 NOTE 9, 2026-09-22) the provider transport-error mapping comment at `src/bin/gnosis_server.rs:347`** (the `StoreError::EmbeddingUnavailable` mapping the timeout contract hangs on — comment-only, no code change; §9.5.6's same-unit table carries the row), the **lib seam**
  (`src/lib.rs`: `PROVIDER_REQUEST_TIMEOUT` + `provider_client` + the crate-root re-export), the `boot_wiring`
  comment sites (`src/lib.rs:50-67`, `:79-89` — kept at the pre-U6 predicate **deliberately**, with the
  "assertion surface, not the derivation" clarification) and the derivation-site comment
  (`src/store/mod.rs:4210-4215`); **`docs/specs/engine-wire-contract.md`**
  — U6-labelled notes on §9.1's `vector` flag row and §12's V-8.1/V-8.2 stage labels (its type rules, the
  frozen `EngineSubsystems`/`HealthReport` shapes and the §11 map are **untouched**); and
  **`docs/specs/p2-gnosis-server-live-pending-battery.md`** — the **new** live row **`R-L4`** (the
  post-mutation honesty criterion) plus the §8 item 4 note that the FS-14 state-6 mapping's park reason is
  superseded (its criterion moves to the live row), while **`R-L3` (iii)/(v) stay PARKED unchanged** and
  `R-L2`'s cell is **not** re-pointed (U6 does not change a boot-time flag value). **The `tests/` items are the
  TestWriter's, the `src/` items the Implementer's, and the two spec-file items this pass does not own (they
  are the supervisor's same-unit landing — §9.5.6's same-unit table marks the battery row and the
  wire-contract rows as such, so the red set has one authority).**

- **U2 object-valued-option field-level pin (2026-09-16; docs-only, ADDITIVE — a pin CLARIFICATION, no rule
  changed).** The **blind-greens gate's `U2-13` finding** (`docs/specs/u2-query-post-contract-greens.md`
  §Failures — the scenario `u2_post_object_member_misuse_is_absent_layer1`) exposed a genuine **two-reading
  hole** in §5.3: for an **object-valued** option (`multiQuery`, `subTaskDag`) **present with a wrongly-typed
  documented member** (`{"multiQuery":{"enabled":"yes"}}`), §5.3's per-key cells could be read either as
  **(i)** the whole option is absent ⇒ the field is **`None`**, or **(ii)** the member is defaulted *inside a
  present object* ⇒ `Some({enabled:false, …})`. **The supervisor ruled (i)**, consistent with §5.3's
  two-layer bullet ("the decoder NEVER substitutes a typed default value") and the F5 wrongly-typed rule, and
  the Implementer has landed that behavior in `src/wire/query.rs` (`enabled_object`, `:183-190` — **the
  pre-generalization name; after the supervisor's ruling 2 (2026-09-16) the landed function is `object_option`,
  `:188-202`, total over every declared member** — the `docs/defects.md` **P-6** row records the rename and the
  generalization). This pass
  **closes the hole so the two readings cannot recur**: §5.3's two object cells now state the outcome **at
  layer 1** (non-object, or a wrong-typed documented member ⇒ the field is `None`; a **well-formed** object
  keeps its mapping, with member-level defaults — `n = 3`, `enabled = false` — applying **only inside** a
  well-formed object), the two-layer bullet gains the object-valued-option clause **and**
  **`.multi_query == None`** in its layer-1 field enumeration (which previously named `sub_task_dag` and
  omitted `multi_query`), §9.5.4's `P-IM-5` coverage note and §9.5.1's `P-IM-5` row are aligned to the same
  field-level reading (with the three member-misuse instances named as part of the adversarial corpus), and
  the four clauses that still carried the unqualified reading (§5.3's wrongly-typed bullet corpus, §10's
  cross-cutting wrongly-typed list, §11's `P-IM-5` sketch row, §12's additions list) are reconciled to it.
  **This is a pin clarification and NOT a rule change: the wire outcome is identical** — the engine reads
  `enabled = false` either way (`src/store/mod.rs:4088-4091`; the pre-U3 citation was `:4076-4082`; `sub_task_dag` has no read site), as the same
  report's `U2-13b` live scenario confirms, so no consumer-visible 4xx/5xx changes. **The field-level reading
  is now the only one this contract permits** — a test or implementation asserting a member-level `Some` for a
  wrongly-typed documented member is asserting a behavior this contract does not have. **No register row was
  added or deleted** (§11's property table is still **7** `Property-id` rows, §9.5.1/§9.5.2 still **7 + 5**,
  twelve injective tags), no cross-reference moved, no row count changed, and no other pinned rule was edited.
  The matching F2 cross-reference is the object-member sentence in
  `docs/specs/engine-wire-contract.md` §4.5. **(Scope note, 2026-09-16 — supervisor ruling 2:** the field-level
  reading this bullet pins is the **only** permitted one, and the clause it aligns to **stands un-narrowed**:
  the member-type check is now **total over the members each option declares** (`enabled` a bool, `n` a u64 for
  `multiQuery`; `enabled` a bool for `subTaskDag`), so a wrongly-typed `n` is the same misuse class as a
  wrongly-typed `enabled`. See §U1's resolution bullet for reported divergence #11 and `docs/defects.md`'s
  closed `P-6`.)**

- **U2 REALIZED-GREEN — LANDED (2026-09-17; annotation by the proofread pass — no pinned rule changed).** The
  query-POST contract's **code has landed**: `src/wire/query.rs` (`QueryPath`/`SseParams`/`QueryDecodeError`;
  `decode_query_request` — envelope-strict `schemaVersion` → `idFormat` → object `payload` → the 14-key
  mapping; the canonical §4.5.2 `filters` mapping; the three ASCII-case-insensitive token resolvers;
  `request_decode_code`/`request_decode_message`; `encode_result_checked`), its re-exports
  (`src/wire/mod.rs`, `src/lib.rs`), and the bin wiring in `src/bin/gnosis_server.rs` — the shared JSON
  decode-error renderer (`:62-75`, landed; **the `:61-74` spelling the U2-landed bullet and §11's `P-TP-3`
  row carried is stale** — the U3 landing shifted the fn's opening brace to `:62` — `Content-Type: application/json` exactly) called by both the CRUD and the
  query handler, the **SSE pre-stream 400 + single `error` frame** (`:100-115`, landed; **the `:99-114`
  spelling is stale for the same reason**, `Content-Type:
  text/event-stream`), the post-stream frame via `encode_event` (`:273`) and the POST handler's
  `encode_result_checked` (`:219`; the pre-U3 citation was `:218`). Verified state at U2's landing: **`cargo test` 582 passed / 0 failed** — **the current tree's baseline is 604 passed / 0 failed after U3 (serial, 2026-09-17)**;
  `cargo fmt --check` exit 0; `cargo clippy --all-targets` 0 warnings; `cargo build` clean; the blind-greens
  set `tests/blind_u2_query_post_greens.rs` **24/24** (its `U2-13` finding is fixed and the spec hole it
  exposed is closed by the object-valued-option field-level pin above); the live battery's **`R-L1` PASSED
  live** on a READY boot (the wire `topK`/`filters` visibly changed the response; an unrecognized `mode` ⇒ 400
  `validation_error`). The U2 property layer's executed case budget is the landed **400**
  (`48/100/70/80/23/41/38`, `tests/props_gnosis_server.rs:26-40`; each row ≤ 100 and the layer ≤ 400 ✔ — the
  per-unit cap of §9.5.3). **Consequence:** the **"(U1; code lands in U2)" / "U2-time"** markers throughout
  this file and in `docs/specs/engine-wire-contract.md` §4.5/§7.1 now read **LANDED** (their historical
  wording is kept; U3/U4/U5 markers are unaffected), and the code-line citations the landing moved have been
  re-pointed where they describe the **landed** state (citations explicitly labelled *pre-U1* keep the
  historical pre-U1 line numbers).

- **Proofread-pass doc corrections (2026-09-17; wording/reference/numeric only — no pinned rule changed).**
  **(a)** §9.5.1 `P-TP-2`'s (β) observable and the live battery's `R-L1` row now name the **real frozen wire
  field** — the **`results` array** (`RagResult.results`, `src/store/mod.rs:758-765`) — instead of
  `items.len()` (F-3). **(b)** The SSE **`Content-Type` is pinned to `text/event-stream` for every SSE
  response** (§5.4/§5.6, and F2 §4.4); the pre-existing `text/plain; charset=utf-8` rendering of the
  post-stream `StoreError` `error` frame is recorded as `docs/defects.md` **P-5** (owner: the transport unit /
  the SSE branch) instead of being silently absorbed (F-2). **(c)** §9.5.1 `P-TP-4`'s observable now
  enumerates the **two representable** rejected classes, with §5.3's encoder clause and F2 §13's
  `validate_rag_result` cell aligned to it (a traceless result is not representable, `RagResult.trace` is not
  an `Option`). **(d)** §9.5's row/tag counts are corrected to **twelve** (U2 = 7 + U3 = 5) and §9.5's
  **executed** U2 layer to **400** (§9.5.3). **(e)** The stale `tests/props_gnosis_server.rs` citations
  (`:112-118`, `:120-127`, `:48-118`, the seven `row_seed` call sites) are re-pointed to the current tree.

- **Reported divergence #11 (proofread pass, 2026-09-17) — RESOLVED 2026-09-16 by the supervisor's ruling;
  the CLAUSE STANDS and is now IMPLEMENTED (defect `P-6` closed).** §5.3's `multiQuery`
  cell, its two-layer bullet, §9.5.1's `P-IM-5`, §9.5.4's `P-IM-5` note, §10's cross-cutting list, §11's
  `P-IM-5` sketch and F2 §4.5/§13 pin a **wrongly-typed `n` member**
  (`{"multiQuery":{"enabled":true,"n":"3"}}`) as **field-level `None`**; the then-landed decoder kept the object
  and took the `n` default (`enabled_object`, `src/wire/query.rs:183-190`, plus `:129-134` ⇒
  `Some(MultiQueryOptions{ enabled:true, n:3 })`), and the executed property layer asserted exactly that landed
  reading (`tests/props_gnosis_server.rs:1187-1193`, `:1335-1350`; row `U2PIM5`) — indeed the TestWriter's own
  comment stated that "a wrongly-typed `n` inside a well-formed object is **not** a misuse: §5.3 pins
  `enabled:true` + wrongly-typed `n` ⇒ the documented `n` default `3`, i.e. the field IS `Some(_)`"
  (`:1170-1172`), i.e. it cited §5.3 for the reading §5.3's own words contradicted. Implementing the clause
  (extend the decoder) or narrowing it (to the `enabled` member) was a **rule-level** decision; it was recorded
  as `docs/defects.md` **P-6** — and the **supervisor ruled the clause correct and NOT narrowed**.
  **Resolution (2026-09-16):** the decoder is being **generalized to every documented member** — `enabled` a
  bool and `n` a u64 for `multiQuery`; `enabled` a bool for `subTaskDag` (`object_option` +
  its member-type arms, `src/wire/query.rs:178-202` — `member.is_boolean()`/`member.as_u64()?`, so a present
  wrongly-typed documented member returns `None` — now total over the documented members of both object-valued
  options), so `{"multiQuery":{"enabled":true,"n":"3"}}` ⇒ `multi_query == None` as pinned, while an
  **omitted** `n` inside a **well-formed** object still defaults to `3`. The code change and the
  TestWriter's aligned probe land in the **same U2 pass**; **no pinned rule was narrowed**, and the
  `docs/defects.md` **P-6** row is closed as **FIXED/CLOSED**.

- **Reported divergence #12 (proofread pass, 2026-09-17) — RESOLVED 2026-09-16 by the supervisor's ruling:
  the WORDING was corrected (no behavior change).** §5.3's per-key
  table said for `topK`/`maxHops`/`maxParentContext`/`binaryCandidatePool` that "a **number** out of
  `1..=50`/`1..=5` ⇒ FS-3 400"; the landed decoder reads those keys with `Value::as_u64`
  (`src/wire/query.rs:119`, `:126`, `:128`, `:140`), so a **non-u64 JSON number** (negative, fractional — e.g.
  `topK: -1`, `topK: 10.5`) is **not** a number it can represent ⇒ **absent ⇒ the option's documented
  default**, not a 400; only a **present u64 that is out of the pinned range** reaches the engine's
  `ValidationError` (400). The two halves are now pinned **as two distinct halves** in every cell that carried
  the claim (§5.3's per-key table's numeric rows, its fail-state table's range row and its wrongly-typed
  bullet, §10's cross-cutting list, and F2 §4.5/§13), so the text **cannot** be read as claiming a 400 for
  `-1`/`10.5`; the historical wording is kept above as the record of what was fixed. **No behavior changed and
  no code was touched** — this was a docs-side wording defect, recorded as `docs/defects.md` **P-7** and closed
  by this pass; the `docs/HANDOFF.md` note that filed it is annotated with the resolution.

- **U2 POST-GREENS DOC-REVIEW (2026-09-16; docs-only, no pinned rule, row, count or reference changed).** The
  documentation reviewer reconciled this file's U2-bearing sections (§5.2–§5.5, §9.5.1, §9.5.3, §9.5.4, §10,
  §11, §12) against the **landed** tree and corrected, in place, the entries that had drifted from it: the
  `sse_params.top_k` line citations (`src/wire/query.rs:155` ⇒ **`:157`**, three sites), the
  `u2_layer_budget_discipline` citation (`:3968-4003` ⇒ **`:4034-4069`**), the `enabled_object` ⇒
  **`object_option`** rename note, the non-object-`payload` fail-state row (now the landed
  `invalid_envelope`-only transport outcome, F9) and the three "U2-time … today" markers that described the
  **pre-U2** tree (`schemaVersion`/`idFormat` checks, the worked examples), the `multiQuery`-divergence bullet's
  `:138` ⇒ **`:140`**, the `SubTaskDagOptions` citation (type at `:627`, field at `:664`) and §12 item 4's
  "code is still owed" clause ⇒ **LANDED**. **No pinned rule, register row, row/tag count or cross-reference was
  changed** — the 7 + 5 §9.5 rows, the twelve tags, the seven landed §11 rows, the 21-row §11 map and the
  14-row route bijection are intact. Review record:
  `archive/reviews/2026-09-16-u2-query-post-contract-doc-review.md` (gitignored provenance).

- **U3 POST-GREENS DOC-REVIEW (2026-09-17; docs-only, no pinned rule, row, count or reference changed).** The
  documentation reviewer reconciled this file's U3-bearing sections (§5.8, §6, §9.5.2/§9.5.3/§9.5.4, §10–§12,
  §U1) against the **landed** tree and corrected, in place, the entries that had drifted from it: the
  `EngineSubsystems` construction literal (the post-U3 `:1714-1725` spelling ⇒ **`:1718-1725`**, the literal
  now sitting under its U3 comment block `:1714-1717`; the pre-U3 `:1710-1717` kept as provenance — §5.8
  carries the field-vs-literal note), the three residual `:61-74` / `:99-114` renderer citations (⇒
  **`:62-75`** / **`:100-115`**, the U2-landed bullet and §11's `P-TP-3` row), and §12 item 5's
  "V-8.1/V-8.2 literal edit **owed** in the same unit" clause ⇒ **LANDED with U3 (2026-09-17)**. Verified
  against the tree before the edit: §9.5.2's five rows `P-IM-7` 25 / `P-IM-8` 26 / `P-IM-9` 26 / `P-SM-5` 25 /
  `P-SM-6` 25 = **127 executed ≤ 400** (tags `U3PIM7`…`U3PSM6`, `tests/props_gnosis_server.rs:59-76`,
  `:278-295`, `u3_layer_budget_discipline` `:5689`), the golden `tests/wire_conformance.rs:1061-1112` +
  `boot_wiring_couples_to_the_derived_read` `:1129`, `health_report_shape_frozen` `:1263`, the lib seam
  `boot_wiring` (`src/lib.rs:69-110`), the derivation (`src/store/mod.rs:4193-4232`), the boot
  (`src/bin/gnosis_server.rs:378-408`) and the blind set's **13/13**. **No pinned rule, register row, row/tag
  count or cross-reference was changed** — the 7 + 5 §9.5 rows, the twelve tags, the seven landed §11 rows,
  the 21-row §11 map and the 14-row route bijection are intact. Review record:
  `archive/reviews/2026-09-17-u3-status-honesty-doc-review.md` (gitignored provenance).

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
  id adoption (reused unchanged via the `idFormat` seam). — RECONCILED 2026-09-22 (`RBAC-DOC-DRIFT`):
  the engine pins the credential **SHAPE** + its decode-layer **presence check** only; the authority
  mapping and the deny are **SHELL**-side — see `GNOSIS-RBAC-EDIT-ENFORCEMENT` / `docs/HANDOFF.md`
  §8-RBAC. The historical clause stands as written.
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
  — RECONCILED 2026-09-22 (`RBAC-DOC-DRIFT`): the engine pins the credential **SHAPE** + its
  decode-layer **presence check** only; the authority mapping and the deny are **SHELL**-side — see
  `GNOSIS-RBAC-EDIT-ENFORCEMENT` / `docs/HANDOFF.md` §8-RBAC. The historical clause stands as written.
- RFC-4122 id adoption (reused unchanged via the `idFormat` seam).
- Any new `StoreError` variants or new §11 rows (the §11 map stays 21 rows; the request-decode
  outcome is a transport-level status, NOT a `StoreError` wire code).
- Zero new runtime deps on the engine **lib** (the bin gains the server framework).
- **(U1)** A **consumer-visible store-wide revision**. `GET /snapshot?revision=…` and a
  `stale_revision` error are **REFUSED, not parked** (gate-1 ruling 1; §5.6). Reasons, pinned:
  (a) under `SHARDED-RWLOCK-STORE` there is no global store lock, so no point-in-time read across
  shards exists to back the claim; (b) the only store-wide counter is the derived-index `epoch`, which
  legitimately advances on propagation/state-annotation journal mutations too
  (`src/store/mod.rs:2284`, `:2356`, `:3358` — ops `propagate_fact_staleness`,
  `propagate_archived_target`, `set_reference_state`), so a consumer keyed on it would see phantom
  changes; (c) a `stale_revision` variant's generating step cannot run — barred by
  `RESERVED-ERRVARIANTS-DISCIPLINE`; and (d) §11's map may not grow (21 rows). As-of-N reconstruction
  is a **separate journal-retention concern** (boundary **C4**,
  `docs/research/astrographer-engine-shell-boundary.md:86`), **not** a query surface.
- **(U1)** Any **new route row** added outside a spec-amendment unit that extends
  `route_bijection()` **and** its conformance/property assertions in the same unit (§5.2).
- **(U1)** Any **new §11 wire code** for a request-decode failure. The transport codes of §5.5 are
  **transport-level**, live outside the §11 map, and are **disjoint** from `StoreError::wire_code()`
  (`src/wire/error.rs`); specifically there is **no** `stale_revision` and **no** `decode_failed` code.

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

**U1 note.** The `GET /changes` SSE change feed is **not** part of this trio: it is a separate,
**U4-time** route pinned in §5.6 (it does not exist at U1, and it is **not** a row of the §5.2 table).
The `POST /rag/query` and `GET /rag/stream` rows' request/response semantics are pinned in §5.3/§5.4.

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

**Routing contract (RESTATED by U1 — a growth invariant, not a fixed count).**

> **Superseded-by-U1:** the pre-U1 wording pinned the routing table as a **fixed 14-row** bijection
> ("each of the 14 paths (11 CRUD + 3 retrieval) maps to exactly one handler … the table has exactly
> 14 rows"). The **"exactly 14 rows" clause is DELETED** — the **only** outright deletion U1 makes.
> A fixed count was a snapshot of the surface at P2, not an invariant: pinning it would turn every
> future route into a contract violation instead of an amendment, and U4 is already scheduled to add
> `GET /changes`. The **14-row table above stands as the current state of the surface at U1** — it is
> correct today and is what `P-IM-3`'s conformance assertions enumerate.

The invariant (the §5.x `P-IM-3` row) is:

1. every `(verb, path)` row in the routing table maps to **exactly one** distinct handler;
2. every handler is reachable by **exactly one** `(verb, path)` row;
3. the **11 document-CRUD paths equal the P1a `ENGINE_ENDPOINTS` rows verbatim**
   (`docs/specs/p1a-document-crud-wire.md` §7; the §5.2 table above);
4. the **retrieval trio is present**: `POST /rag/query`, `GET /rag/stream`, `GET /engine/status`;
5. **no fixed row count** is part of the invariant.

**Amendment rule (pinned).** A new route may be added **only by a spec-amendment unit** that, **in the
same unit**, (a) enumerates each new `(verb, path, handler, response kind)` row; (b) extends
`route_bijection()` (`src/server.rs:68-87`) **and** the live axum `router()`
(`src/bin/gnosis_server.rs:290-304`, post-U2) consistently; and (c) extends `P-IM-3`'s conformance/property
assertions (§11) in the same unit — including superseding any assertion that pins the old count
(`tests/gnosis_server_conformance.rs:305`, `tests/props_gnosis_server.rs:702` — the count check inside
`p_im_3_route_bijection`, `:696-707`; the pre-U2 citation `:608` is stale —
`tests/blind_p2_gnosis_server_greens.rs:218`).

**The §11 map stays 21 rows.** A new route brings **no** new `StoreError` variant and **no** new §11
wire code: its handler returns one of the existing 21 rows or a transport-level status. Specifically
there is **no `stale_revision`** code and **no `decode_failed`** code — a request-decode failure is the
NEW-2 **transport-level** outcome (400/422, never 502, never a §11 row; §7.2 / §5.5).

The server does NOT invent or re-derive the CRUD paths — it consumes the P1a constants.

### 5.3 `POST /rag/query` — the pinned request/response contract (U1; **code LANDED in U2 — 2026-09-17**; the historical "code lands in U2" heading wording read LANDED per §U1's U2-landed bullet)

**Authority.** gate-1 ruling 3 (`docs/specs/gnosis-gr-inbound-review.md`), with the GR-1 and GR-2
verdict rows. **Contract only** — the decoding/validation code lands in **U2**; this sub-section is
what U2's TestWriter derives its red set from. Pre-U1 state it replaces: the POST handler reads only
`payload.get("query")` and calls `store.rag_query(&query, &RagQueryOptions::default())`
(`src/bin/gnosis_server.rs:162-168` in the **pre-U1 tree**; the landed handler is `:206-214`), so the
caller's whole options surface is silently inert.

**Envelope-strict, exactly like CRUD (no back-compat tolerance).** The request body MUST be the F2
envelope (`{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{…}}`, F2 §4.1).
`POST /rag/query` MUST validate `schemaVersion` and `idFormat` exactly as the CRUD decode path does
(`decode_crud_request`, `src/wire/crud.rs:305-310`): **`schemaVersion` first** →
`UnsupportedSchemaVersion`, **then `idFormat`** → `UnknownIdFormat`; both are the NEW-2 transport
**400** (§7.2 / §5.5).

- **A bare (non-envelope) body is REJECTED.** GR-1's request for back-compat tolerance is **REFUSED**:
  it would make the query surface envelope-optional while CRUD stays envelope-strict, and the envelope
  is the engine's own landed contract (`Envelope::from_json`, pre-U1 line `src/bin/gnosis_server.rs:157-161`;
  the landed query handler's envelope parse is `:215-218` (the pre-U3 citation was `:207-210`); the
  P1a precedent `docs/specs/p1a-document-crud-wire.md` §4.2/§6.2).
- The envelope check MUST live in a **shared wire decoder** used by both the POST path and the SSE
  path, so there is **one place** to satisfy (U2-time). The SSE path carries no body — it takes query
  params — so what is genuinely shared between the two paths is the **mode rule** of §5.4 (plus a
  shared envelope-check helper for the POST path); U2 MUST NOT duplicate either rule in the two
  handlers.

**Request payload — the camelCase `ragQuery` options object.** The payload is the §4.6.1 options
object (`docs/specs/gnosis.md:872`) with the keys below. Every key is **optional except `query`**.
**Unknown/extra payload keys are TOLERATED and ignored** — the existing CRUD decoder sets that
precedent (no `deny_unknown_fields` anywhere in `src/wire/`; `decode_crud_request` reads only
`method`/`args` and requires nothing else, `src/wire/crud.rs:311-326`). This tolerance is
**load-bearing**: the existing e2e POSTs carry an extra `args` key
(`tests/gnosis_server_e2e.rs:188-198`).

| payload key (wire, camelCase) | JSON type | `RagQueryOptions` field | absent ⇒ | engine fail-state |
| --- | --- | --- | --- | --- |
| `query` | string | (the `query` argument) | `""` ⇒ FS-3 `ValidationError("query must be non-empty")` | 400 `validation_error` |
| `mode` | string | `mode` | `Flat` | **§5.4** (wire-resolver-level: unrecognized ⇒ 400 `validation_error`) |
| `topK` | number | `top_k` | `10` | FS-3, **range only**: a **present u64** not in `1..=50` ⇒ 400 `validation_error`; a present value that is **not a u64** (negative/fractional — `-1`, `10.5`) ⇒ absent ⇒ the default `10`, **never** a 400 (the two halves are pinned in the per-key table below) |
| `wikiId` | string | `wiki_id` | `None` (no wiki scope) | — |
| `filters` | object (**canonical §4.5.2 shape** — §5.3's "Value tokens" bullet) | `filters` | `None` | FS-3 (malformed ⇒ 400 `validation_error`) |
| `maxHops` | number | `max_hops` | `3` | FS-3, **range only**: a **present u64** not in `1..=5` ⇒ 400 `validation_error`; a present value that is **not a u64** ⇒ absent ⇒ the default `3`, **never** a 400 (per-key table below) |
| `expand` | string | `expand` | `None` (= `'none'`) | **wire-resolver-level (POST only)** (unrecognized token ⇒ 400 `validation_error`; a **new state created by this amendment** — §5.4). On the **SSE** surface the param **does not exist** ⇒ ignored ⇒ `'none'`, **never** a 400 (§5.4) |
| `maxParentContext` | number | `max_parent_context` | `5` | — |
| `multiQuery` | object `{enabled, n}` | `multi_query` | disabled (`enabled:false`) | FS-3, **member-type + range split**: a **present non-u64 `n` member** — or a present non-bool `enabled` member — ⇒ the whole field is **`None`** ⇒ the default (disabled), **never** a 400; a **present u64** `n == 0` with `enabled:true` ⇒ 400 `validation_error` (per-key table below) |
| `compression` | string | `compression` | `None` (= `'none'`) | **wire-resolver-level (POST only)** (unrecognized ⇒ 400 `validation_error`; FS-3's canonical "invalid `compression`"). On the **SSE** surface the param **does not exist** ⇒ ignored ⇒ `'none'`, **never** a 400 (§5.4) |
| `hyde` | boolean | `hyde` | `false` | — |
| `binaryFirstPass` | boolean | `binary_first_pass` | `false` | — |
| `binaryCandidatePool` | number | `binary_candidate_pool` | `10 × topK` | FS-3, **range only**: `0` with `binaryFirstPass: true` ⇒ 400 `validation_error` (a **present u64** out of its permitted set); a present value that is **not a u64** ⇒ absent ⇒ the default `10 × topK`, **never** a 400 (per-key table below) |
| `subTaskDag` | object `{enabled}` | `sub_task_dag` | disabled | — |

- **`multiQuery`'s documented members — BOTH are type-checked (`enabled` a bool, `n` a u64); a wrongly-typed
  documented member ⇒ the whole option is `None` (pinned; the clause stands and is implemented — supervisor
  ruling 2, 2026-09-16).** `{"multiQuery":{"enabled":true,"n":"3"}}` ⇒ **`multi_query == None`** — the field,
  never a present object whose `n` was defaulted inside it — exactly as `{"multiQuery":{"enabled":"yes"}}`
  does; the same member-type rule applies to `subTaskDag`'s single documented member (`enabled`, a bool). The
  **decoder implements this by checking every documented member** the option declares (`enabled` ⇒ bool, `n` ⇒
  u64; `object_option` and its member-type arms, `src/wire/query.rs:178-202`), so the rule is total over the
  members each option declares and not narrower than this pin. **The engine-side read of `sub_task_dag` is
  unchanged** (`object_option` now returns `(enabled, n)` for both object keys and the `subTaskDag` call site
  takes only `enabled`, `:141-142`) — the option stays inert, exactly as the reachability note below states.
- **`multiQuery.n` default (pinned — not left in a doc comment).** When `multiQuery` is present with
  `enabled: true` and `n` **omitted**, the decoder supplies **`n = 3`** — the engine's own documented
  default (`MultiQueryOptions::default()`, `src/store/mod.rs:593-599`; the `n` field is `u64`, not
  optional, so a decoded `{enabled:true}` MUST carry `n: 3`). `{enabled:false, n: <anything>}` is inert
  (the engine only reads `n` when `enabled` is `true`, `src/store/mod.rs:4088-4091`; the pre-U3 citation was `:4076-4082`).

- **Per-option reachability (N4 — pinned so "option accepted" is never read as "option honored").**
  Only `filters` is proven **reachable end-to-end** today (its read sites are listed in the explicit
  `filters` bullet under this table). For the remaining options the reachability status is pinned here,
  with the read sites named; a U2 test that asserts only "the option decoded" is a **partially-blind**
  claim, and a test that asserts an **effect** for an option on a leg that does not consume it is
  asserting a **latent engine gap**, not this contract:
  - **`subTaskDag` — INERT on the engine side (GR-2-adjacent; U2 does not change it).**
    `sub_task_dag` appears **only** as the declaration (`SubTaskDagOptions` — the type at
    `src/store/mod.rs:627`, the `RagQueryOptions` field at `:664`) and
    as the `SubTaskDagFailed` error variant (`:1022`); it has **no read site** anywhere in the query
    paths, so decoding it to `RagQueryOptions.sub_task_dag` produces an option the engine **never
    consults**. U2 decodes it (a wire-contract obligation) and MUST NOT claim it is honored; whether the
    engine ever consumes it is **not** a U1/U2 deliverable.
  - **`binaryFirstPass` / `binaryCandidatePool` — consumed by the VECTOR leg only**
    (`src/store/mod.rs:4455-4460`, `:4532-4534`, reached via `:5098`; the pre-U3 citations were
    `:4419-4422`, `:4496-4501`, `:5061`), i.e. by `mode=vector` and by
    `mode=hybrid`'s vector leg (where a failing/absent leg **degrades to empty**, §5.9) — **inert in
    `mode=flat` and `mode=graph`**.
  - **`multiQuery` — consumed at the fan-out** (`src/store/mod.rs:4088-4091`, `:5224-5252`; the pre-U3
    citations were `:4076-4082`, `:5200`), and its
    `enabled` fail-state is reachable only on a READY engine (FS-19, the fail-state table below).
  - **`hyde`, `maxParentContext`, `wikiId`, `mode`, `topK`, `maxHops` — consumed on the modes §5.9
    names** (`hyde`/`wikiId`/`topK` on the retrieval options; `maxHops` by the graph walk).

- **Value tokens (casing).** `mode` is the four lowercase §4.6.1 tokens
  (`flat|graph|vector|hybrid`), matched **case-insensitively**; `expand` (`none|parent`) and
  `compression` (`none|filter|extract|graph`) are the canonical §4.6.1 tokens (**`docs/specs/gnosis.md:872`**)
  and follow the **same casing policy** (ASCII case-insensitive; an unrecognized token ⇒ 400
  `validation_error`, never a silent default). **Only a string reaches the token check (N2):** the
  resolver of §5.4 is entered **only** for a JSON **string** value, so a present value of any other JSON
  type (`expand: 5`, `compression: null`, `compression: []`, `mode: 5`) is a **wrongly-typed value ⇒
  absent ⇒ that option's default** (the wrongly-typed rule below) and **not** an unrecognized-token 400.
  The `expand`/`compression` token rules are **POST-payload-only** (the SSE surface has no such params —
  §5.4). All three are decoded by the **wire-layer token
  resolvers of §5.4** (one per token family) — see the next bullet: they are **not** checked by
  `validate_rag_options`.
- **`filters` — the canonical §4.5.2 shape, NOT the store's serde casing (F1; supersedes the pre-remand
  pin).** **Superseded-by-this-remand:** U1's first pass pinned the three enum-valued `filters` members
  as the **store's serde casing** (PascalCase unit values plus `Community`/`DocHead`/`…`, and
  `filters.target` as the serde **2-element array** `[documentId, nodeId]`), justified by "the filter
  body is the store's own serde shape" and passed through to `QueryAuditFilters`. **That pin was wrong
  and is replaced by the canonical one below** (it also contradicted F2's own casing policy for every
  other enum-valued request field in the same payload).

  **The pin.** `filters` is the **§4.5.2-pinned shape** (`docs/specs/gnosis.md:692-694`), which is a
  **request** shape and is therefore taken from the canonical contract, not from a store type:

  | `filters` member | wire tokens (this contract) | decoded to (frozen store types) |
  | --- | --- | --- |
  | `nodeKind` | `'content'` \| `'fact'` \| `'reference'` | `NodeKind::Content`/`Fact`/`Reference` (`src/store/mod.rs:97-108`) |
  | `edgeType` | `'link'` \| `'embed'` \| `'crosslink'` | `EdgeKind::Link`/`Embed`/`Crosslink` (`:112-132`) |
  | `target` | `{documentId, nodeId}` — **an object with both members**, not an array | `(DocumentId, NodeId)` (the store's tuple field, `:475`) |
  | `state` | `'FRESH'` \| `'RESOLVED'` \| `'STALE'` \| `'BROKEN'` | `ReferenceState::Fresh`/`Resolved`/`Stale`/`Broken` (`:136-145`) |

  - **The U2 decoder MAPS these tokens to the frozen store types** — it does **not** deserialize the
    payload into `QueryAuditFilters` directly. The store's serde casing (`node_kind` field names, the
    tuple `target`, PascalCase unit values) is the **response/audit** shape **only** (F2 §4.2's
    body-shape note, which governs the `RagResult`/`QueryAuditEntry` **response** body —
    `docs/specs/engine-wire-contract.md` §4.2/§4.3.4 — and **not** caller-supplied request payloads).
  - **`filters` is NOT passed to `QueryAuditFilters` via `serde_json::from_value` (pinned).**
    `QueryAuditFilters` (`src/store/mod.rs:471-477`) carries **no `#[serde(rename_all = "camelCase")]`**,
    so the camelCase keys cannot bind; and it has **no `deny_unknown_fields`**, so a literal
    pass-through would **silently deserialize to an all-`None` filter** — the GR-2 defect class
    (a caller-visible option that is inert). U2 MUST map the four members explicitly (the mapping table
    above) and MUST NOT hand the raw payload value to serde for this type.
  - **`nodeKind` accepts exactly the three §4.5.2 tokens** — `'community'` is **not** a member of the
    pinned shape, so a `filters.nodeKind: "community"` is **malformed** ⇒ FS-3 `ValidationError` ⇒
    **400 `validation_error`** (the store's `NodeKind` has four variants, `:97-108`; **the wire shape
    has three**, because the canonical pin fixes the wire shape — the fourth is reachable only through
    the store API, exactly as `EdgeKind`'s six non-referencing variants are).
  - **`edgeType` accepts exactly the three validator-accepted kinds** — `'link'`/`'embed'`/`'crosslink'`.
    An `edgeType` outside those three (e.g. `"DocHead"`, `"relation"`, `"member"`) is **400
    `validation_error`**: `validate_rag_options` rejects every `EdgeKind` outside
    `{Link, Embed, Crosslink}` with `"filters.edgeType must be link, embed, or crosslink"`
    (`src/store/mod.rs:4311-4321`). The store's **9** `EdgeKind` variants (`:112-132`) therefore do
    **not** widen the wire token set — the pinned wire set and the validator-agreed set are the same
    three tokens, which is what keeps the wire shape and the engine's range check consistent.
  - **`state`** is the four `'FRESH'|'RESOLVED'|'STALE'|'BROKEN'` tokens of §4.5.2 (the canonical
    uppercase forms; the store's `ReferenceState` serde values are `"Fresh"|"Stale"|"Resolved"|
    "Broken"`), matched **case-insensitively** under the same policy as the other tokens.
  - **Reachability honesty.** With the canonical pin the decoded `filters` **is** honored by the engine
    (`RagQueryOptions.filters` reaches the flat leg (`flat_query`, `src/store/mod.rs:4101`), the hybrid
    graph/lexical legs (`:4477`, `:4486`), the graph walk (`:4374`), the multi-query fan-out
    (`:5200`), and the §4.3.4 audit entry (`:4149`)), so `filters` stops being the inert option GR-2
    reported. An all-`None` `QueryAuditFilters` is produced **only** when the caller sends `{}` (every
    member absent) — and that is exactly the case a camelCase pass-through would also produce for a
    fully-populated payload, which is the defect.
- **Wrongly-typed option values decode as ABSENT (pinned — closes the POST/SSE divergence).** For every
  optional key **other than `query` and `filters`**, a value that is **present with the wrong JSON type**
  (e.g. `topK:"10"`, `hyde:"yes"`, `wikiId:5`, `maxHops:null`, `binaryFirstPass:1`,
  `multiQuery:{enabled:true}` without `n`, `subTaskDag:5`, **`expand:5`**, **`compression:null`**, and the
  **object-member misuse** instances `{"multiQuery":{"enabled":"yes"}}`, `{"multiQuery":{"enabled":true,"n":"3"}}`,
  `{"subTaskDag":{"enabled":"yes"}}` — for the two object-valued keys a wrongly-typed **documented member** is
  **field-level `None`**, never a defaulted member inside a present object, exactly as the two-layer bullet's
  object-valued-option clause and the per-key table's two object cells state; the member-type check is **total
  over the members each option declares** — `enabled` (bool) and `n` (u64) for `multiQuery`, `enabled` only
  for `subTaskDag` — so a present non-u64 `n` is a misuse like any other, ruling 2, 2026-09-16)
  decodes to **`None`** ⇒ that
  option's **documented default** (the "absent ⇒" column above) — it is **not** a decode failure and
  **not** a 400. **A present number that is not a u64 is inside this rule** (`topK:-1`, `topK:10.5`,
  `maxHops:-2` ⇒ `None` ⇒ the documented default); **only a present u64 outside its pinned range** reaches
  the engine's FS-3 400 (the type-vs-range bullet below pins the two halves). This is exactly how the SSE path already treats an unparseable `topK`
  (pre-U2 the SSE handler parsed it tolerantly, `src/bin/gnosis_server.rs:194`; U2 moved the parse into the
  shared decoder, `sse_params.top_k.and_then(|s| s.parse::<u64>().ok())`, `src/wire/query.rs:157`) and how the
  pre-U1 POST path treats a non-string `query` (`:162-167`), so the two paths agree by construction.
  **`query` is one exception**
  (**absent** ⇒ `""` ⇒ FS-3 400 `validation_error`; a **present non-string** ⇒ the decoder's own
  `Err(QueryDecodeError::Validation(ValidationError))` ⇒ 400 `validation_error` — the two halves of
  the table's `query` cell) and **`filters` is the other**
  (its type/shape **is** checked — the next bullet). A numeric option that decodes as
  absent therefore falls to its default (`10`/`3`/`5`/`10×topK`) and **not** to a range failure.
  **The per-key ruling is the table below, and it is total over the payload's 14 documented keys** (added by
  the register remand, F4 — so a generator never has to decide per key whether the expected observable is
  `Ok + None` or `Err`; the register row `P-IM-5`, §9.5.1, **cites this table rather than restating it**):

  | payload key | wrongly-typed value ⇒ | the option's reading |
  | --- | --- | --- |
  | `query` | a **present non-string is a decoder-level `Err(Validation)`** (below); **absent is NOT an `Err` at the decoder** | **absent** ⇒ layer 1: the decoder returns **`""`** (its own documented exception) ⇒ layer 2: FS-3 `ValidationError("query must be non-empty")` ⇒ 400 `validation_error`; a **present non-string** (`5`/`null`/`{}`/`[]`) ⇒ decoder **`Err(QueryDecodeError::Validation(ValidationError))`** ⇒ 400 `validation_error` (the two halves are distinct: one layer each, same wire status) |
  | `mode` | absent ⇒ **`Flat`** | **not a JSON string** (`5`/`null`/`{}`/`[]`) ⇒ absent; **a string** goes to the §5.4 token rule |
  | `topK` | absent ⇒ **`10`** | **TYPE (half i)** — not a **u64** (`"10"`, `null`, `-1`, `10.5`, `{}`) ⇒ `top_k == None` ⇒ absence; **RANGE (half ii)** — a **present u64** outside `1..=50` (`0`, `51`, `u64::MAX`) ⇒ FS-3 400 |
  | `wikiId` | absent ⇒ **no wiki scope** | not a string ⇒ `wiki_id == None` |
  | `filters` | **`Err`** | non-object, wrongly-typed member, unrecognized token ⇒ 400 `validation_error`; `null` / `{}` are valid (below) |
  | `maxHops` | absent ⇒ **`3`** | **TYPE (half i)** — not a **u64** (`"3"`, `null`, `-1`, `2.5`) ⇒ `max_hops == None` ⇒ absence; **RANGE (half ii)** — a **present u64** outside `1..=5` (`0`, `6`) ⇒ FS-3 400 |
  | `expand` | absent ⇒ **`'none'`** | not a string ⇒ `expand == None`; **a string** goes to the §5.4 token rule (**POST payload only**) |
  | `maxParentContext` | absent ⇒ **`5`** | not a **u64** (`"5"`, `null`, `-1`, `2.5`) ⇒ `max_parent_context == None` (⇒ absence; **no range fail-state is pinned for this key** — see the type-vs-range bullet below) |
  | `multiQuery` | absent ⇒ **disabled** | **layer 1 — the field, not the member:** a **non-object**, **or an object whose documented member (`enabled` **a bool**, or `n` **a u64**) is present with the wrong type**, ⇒ `multi_query == None` (**layer 1: the whole option is absent** — the decoder MUST NOT answer with a present object whose member was defaulted inside); **the member check is total over the documented members the option declares** (a present non-u64 `n` is a wrongly-typed documented member, so `{"enabled":true,"n":"3"}` ⇒ `None`, ruling 2, 2026-09-16). A **well-formed** object keeps its mapping, and **inside a well-formed object** an **omitted** member takes its documented default — `{}` ⇒ `Some(MultiQueryOptions{ enabled:false, n:3 })`, `{"enabled":true}` ⇒ `Some({enabled:true, n:3})`, `{"enabled":false,"n":7}` ⇒ `Some({enabled:false, n:7})`; layer 2: `enabled` absent/`None` ⇒ **disabled**, an omitted `n` ⇒ **`n = 3`**; **the range fail-state is member-level and applies only to a present u64 `n`**: `enabled:true` with `n == 0` ⇒ 400 (§10's range row), while an `n` out of range under `enabled:false` is inert |
  | `compression` | absent ⇒ **`'none'`** | not a string ⇒ `compression == None`; **a string** goes to the §5.4 token rule (**POST payload only**) |
  | `hyde` | absent ⇒ **`false`** | not a boolean ⇒ `hyde == None` |
  | `binaryFirstPass` | absent ⇒ **`false`** | not a boolean ⇒ `binary_first_pass == None` |
  | `binaryCandidatePool` | absent ⇒ **`10 × topK`** | **TYPE (half i)** — not a **u64** (`"10"`, `null`, `-1`, `7.5`) ⇒ `binary_candidate_pool == None` ⇒ absence; **RANGE (half ii)** — `0` with `binaryFirstPass: true` ⇒ FS-3 400 (a present u64) |
  | `subTaskDag` | absent ⇒ **disabled** | **layer 1 — the field, not the member:** a **non-object**, **or an object whose documented member `enabled` is present with the wrong type**, ⇒ `sub_task_dag == None` (**layer 1: the whole option is absent** — never a present `SubTaskDagOptions` with a defaulted member); a **well-formed** object keeps its mapping, and **inside a well-formed object** an **omitted** `enabled` member takes its documented default (`{}` ⇒ `Some(SubTaskDagOptions{ enabled:false })`); layer 2: `None` ⇒ **disabled** (and `sub_task_dag` is inert on the engine side — the reachability note below) |

  **Two keys are the `Err` exceptions — `query` and `filters`; every other key's wrong type is a silent
  default.** `mode` is therefore **inside** the absent rule for non-string values and reaches the token rule
  **only** as a string (§5.4's table); the same holds for `expand`/`compression`. The store's own serde
  casing is **not** the wire casing for `filters` (see the `filters` bullet below).

- **THE TYPE-vs-RANGE BOUNDARY FOR THE NUMERIC KEYS, AND FOR `multiQuery.n` (pinned; supervisor ruling 1,
  2026-09-16 — the two distinct halves of every "out of range" cell).** `topK`, `maxHops`,
  `maxParentContext`, `binaryCandidatePool` and the `multiQuery.n` **member** are the payload's numeric
  inputs. For every one of them the contract has **two distinct halves**, and no cell of this sub-section may
  be read as one claim:
  1. **A present value that is not a u64 ⇒ ABSENT ⇒ the option's documented default** (layer 1: the field's
     own `Option` is `None`; layer 2: the engine's default) — the wrongly-typed rule above, **no `Err` and
     never a 400**. This is the half that covers a **negative or fractional JSON number** (`topK:-1`,
     `topK:10.5`, `maxHops:-2`, `2.5`) — `serde_json`'s `as_u64` does not represent it, so the decoder reads
     it exactly as an absent key (`src/wire/query.rs:119`, `:126`, `:128`, `:140`). It is **not** claimable
     as a 400 anywhere in this contract.
  2. **A present u64 that is out of the pinned range ⇒ `ValidationError`** — FS-3, **400
     `validation_error`** — `topK` ∉ `1..=50`, `maxHops` ∉ `1..=5`, `binaryCandidatePool == 0` with
     `binaryFirstPass: true`, `multiQuery.n == 0` with `enabled: true`. These are the only numeric 400s, and
     each is a **u64** by construction.
  **`multiQuery.n` is checked at the MEMBER level (ruling 2):** half 1 is the **documented-member type
  check** (`n` present and not a u64 ⇒ the **whole option** is `None` — §5.3's `multiQuery` cell and the
  member bullet above, **not** a defaulted `n` inside a present object), while half 2 (`n == 0` with
  `enabled:true` ⇒ 400) applies only when `n` **is** a present u64. A test asserting a 400 for
  `topK:-1`/`topK:10.5` — or a `Some(...)` for a wrongly-typed `n` — asserts a behavior this contract does
  **not** have.

- **THE TWO-LAYER READING OF THE `absent ⇒` COLUMN (pinned; this is the one place it is stated — §5.4's
  table cross-references this bullet).** The table's **"absent ⇒"** cells and §5.4's "absent ⇒ `Flat`" cell
  name the **effective reading at query time**, which is a statement about the **engine**, and they are
  **never** to be read as the decoded value of the store's own `Option` field. The two layers, both true at
  their own level:
  1. **Wire-decoder layer.** An **absent** token/key — and a **wrongly-typed** token/key (the per-key table
     above) — decodes to the **`None`** of the store's own `Option` field, i.e. the decoder's
     `Option`-preserving representation is what U2's decoder returns: `RagQueryOptions.mode == None` (never
     `Some(Flat)`), `.expand == None`, `.compression == None`, `.top_k == None`, `.max_hops == None`,
     `.wiki_id == None`, `.hyde == None`, `.max_parent_context == None`, `.binary_first_pass == None`,
     `.binary_candidate_pool == None`, **`.multi_query == None`**, `.sub_task_dag == None`. **The decoder NEVER substitutes a typed
     default value** — it applies no documented default of its own and writes none into the options — and the
     only decoded-value exceptions are the pins this sub-section already carries: **an absent** `query` decodes to
     `""` (⇒ the engine's FS-3 400 `validation_error`), a **present non-string** `query` is the row's second
     **`Err`** — the decoder returns `Err(QueryDecodeError::Validation(ValidationError))` instead of a value (a
     **decoder-level** `Err`, never `""`; the per-key table's `query` cell states both halves) — while `filters`
     is **schema-validated** (and `filters: {}` decodes to `Some(all-None)`, per its bullet below), and a
     **string** token that is not a vocabulary member ⇒
     `ValidationError` (§5.4's token rules). **The well-formed-string token case is the ONLY case that
     produces `Some(token)` for those three token fields** — a string that **is** a vocabulary member (any ASCII
     casing) ⇒ `Some(QueryMode::…)`/`Some(ExpandMode::…)`/`Some(CompressionMode::…)`; every other value of those
     keys (absent, non-string, or a non-member string, the last on the paths that read the token) is `None` or the
     §5.4 `Err` above, never a decoder-supplied default. **(The other `Option` fields' `Some` values are the
     *named* mappings of the per-key table — `Some(42)` for a well-formed `topK`, `Some(filters)` for a
     schema-valid `filters`, `Some(MultiQueryOptions{…})`/`Some(SubTaskDagOptions{…})` for a **well-formed
     object** — and never the substitution of a documented default: the two object-valued fields are `None`
     whenever the object is missing, is not an object, or carries a wrongly-typed documented member, exactly as
     that table's two object cells state.)**

     **The same field-level rule governs the two OBJECT-VALUED option keys (`multiQuery`, `subTaskDag`) — a
     wrongly-typed *member* is a wrongly-typed *input*, and the decoder answers with `None`, not with a
     defaulted member inside a present object (pinned; closes the two-reading hole the blind-greens gate's
     `U2-13` finding exposed; generalized to **every** declared member by ruling 2, 2026-09-16).** Layer 1 is
     therefore read **at the field**: a **non-object** value, **or an object whose documented member
     (`enabled` — a bool — and `n` — a u64 — for `multiQuery`; `enabled` — a bool — for `subTaskDag`) is
     present with the wrong type**, ⇒ `multi_query == None` / `sub_task_dag == None` — **never
     `Some(…enabled:false…)`**. The member-type check is **total over the documented members each option
     declares**: a present non-bool `enabled` **and** a present non-u64 `n` are both wrongly-typed inputs
     that make the whole field `None`. The **member-level** default applies **only inside a well-formed
     object**, where it is exactly the mapping
     the per-key table's cells name (an **omitted** `enabled` ⇒ `false`; an omitted `n` ⇒ `3`), so
     `{"multiQuery":{}}` ⇒ `Some(MultiQueryOptions{ enabled:false, n:3 })` and
     `{"multiQuery":{"enabled":true}}` ⇒ `Some({enabled:true, n:3})`, while
     `{"multiQuery":{"enabled":"yes"}}`, `{"multiQuery":{"enabled":true,"n":"3"}}` and
     `{"subTaskDag":{"enabled":"yes"}}` ⇒ the field is `None`. **This is the only reading of those two keys'
     cells**: the "absent ⇒ disabled" phrasing of their "absent ⇒" cells is the **layer-2 engine reading**
     (`multi_query.enabled`, absent ⇒ disabled — the next bullet), not a licence for a layer-1
     `Some(disabled)`.
  2. **Engine/query layer.** The documented defaults — `mode` absent ⇒ `Flat`, `compression` absent ⇒
     `'none'`, `expand` absent ⇒ `'none'`, `topK` absent ⇒ `10` — are applied **at query time, by the
     engine**, from those `None`s: `mode.unwrap_or(QueryMode::Flat)` (`src/store/mod.rs:4060`, `:4168`),
     `expand != Some(ExpandMode::Parent)` (`:4936`), `compression.unwrap_or(CompressionMode::None)`
     (`:4130`), and the numeric/option defaults of the same call sites. They are **not** values the decoder
     writes into the options.
  **Consequence for the TestWriter (why this is pinned).** The element-wise `== None` assertions of
  `P-IM-5`/`P-TP-2` (§9.5.1) and §5.4's absence cell therefore **cannot** be read as contradicting each other:
  the register rows assert the **decoder's** output (layer 1 — `mode == None`, not `Some(Flat)`), and §5.4's
  table describes the **engine's** effective resolution (layer 2). A test that asserts `Some(Flat)` for an
  absent or wrongly-typed `mode`, or `Some(10)` for an absent/wrongly-typed `topK`, asserts a decoder behavior
  this contract does **not** have (the wrongly-typed wording "⇒ absent ⇒ default" of `P-IM-5`/`P-TP-2` is read
  at layer 1 as `== None` and at layer 2 as the query-time default); conversely, a test that asserts an
  effect of that default asserts the engine's reading (layer 2) and not the decoder's output. **The same
   consequence holds for the two object-valued keys:** a test that asserts
   `Some(MultiQueryOptions{ enabled:false, n:3 })` (or `Some(SubTaskDagOptions{ enabled:false })`) for
   `{"multiQuery":{"enabled":"yes"}}` / `{"subTaskDag":{"enabled":"yes"}}` — or for a **non-object** value —
   asserts the member-level reading this contract **ruled out** (the supervisor's ruling on the blind-greens
   `U2-13` finding); the field is `None`, while `Some(…)` is asserted **only** for a **well-formed** object
   (`{}` included).

- **Scalar options: the same two layers apply.** The engine's own defaults for the non-enum keys are read at
  query time too — e.g. `options.top_k.unwrap_or(10)`, `options.max_hops.unwrap_or(3)`,
  `options.max_parent_context.unwrap_or(5)`, `options.binary_candidate_pool.unwrap_or(10 * top_k)`,
  `options.hyde.unwrap_or(false)` — so the table's `10`/`3`/`5`/`10 × topK`/`false` cells are **layer-2**
  readings (never a `Some(default)` written by the decoder), exactly as for `mode`/`expand`/`compression`
  above; the engine's `multiQuery` reading is `multi_query.enabled` (absent ⇒ disabled) via the
  `Option`-preserving field (`src/store/mod.rs:4088-4091`; the pre-U3 citation was `:4076-4082`).

- **`filters` is the exception to the wrongly-typed rule — its type IS checked (pinned).** `filters`
  has a **schema** in the canonical §4.5.2 pin and FS-3 names **"malformed `filters`"** as a
  `ValidationError`, so:
  - `filters: null` ⇒ **absent** (no filter) — **valid**;
  - `filters: {}` ⇒ an **all-`None`** `QueryAuditFilters` (present, every member absent) — **valid**;
  - a **non-object** `filters` (`[]`, `"x"`, `5`) ⇒ **400 `validation_error`** (malformed — *not*
    "absent": there is no documented default for a value of the wrong shape here, unlike the scalar
    options above);
  - an **object** whose members are wrongly typed (`{"nodeKind":5}`, `{"target":"d1"}`) ⇒ **400
    `validation_error`** (malformed member);
  - an object with an **unrecognized token** (`{"edgeType":"docHead"}`) ⇒ **400 `validation_error`**
    (§5.3's token table; the validator's own arm for a decoded non-referencing kind is
    `src/store/mod.rs:4311-4321`).
  - **`filters: {}` yields `Some(QueryAuditFilters { node_kind: None, edge_type: None, target: None,
    state: None })` — an all-`None` filter that is PRESENT, VALID and HONORED (N5, pinned).** It is the
    identity filter: it matches every node the other legs admit, which is exactly why the F1 property is
    stated negatively (the GR-2 defect is `Some(all-None)` produced from a **fully-populated**
    camelCase payload, not the caller's deliberate `{}`). The store-side corner at
    `src/store/mod.rs:4781-4782` (a predicate that returns `true` for every node when `node_kind` alone
    is absent) is therefore the **expected** outcome for the `{}` case and **not** a defect.
  - **The wire surface does NOT expose `nodeKind: 'community'` (N5, pinned).** The fourth `NodeKind`
    variant is reachable **only through the store API in Rust** (`src/store/mod.rs:4776-4779`), never
    through this request shape: `filters.nodeKind: "community"` is **400 `validation_error`** (the three
    pinned tokens above). A U2 test asserting a `community`-scoped wire filter is asserting a surface
    this contract does not have.
- **`filters: null`** decodes as **absent** (no filter) under the rule above; `filters: {}` decodes to
  an **all-`None`** `QueryAuditFilters` (present, but every member absent) — both are **valid**, and
  neither is malformed (the `{}` case's exact reading and its owed-property expectation are pinned in the
  bullet above, N5).
- **Range/consistency validation is NOT re-implemented at the wire layer — but the enum-TOKEN check
  lives there (F2; corrected by this remand).** Every **range** fail-state in the table is the engine's
  existing `validate_rag_options` (`src/store/mod.rs:4330-4375`; the pre-U3 citation was `:4293-4338`), reached because the decoded
  `RagQueryOptions` is passed through to `rag_query`/`rag_stream`. U1 pins the **decoding**, not a
  second validator. **The `mode`/`expand`/`compression` token check is NOT part of
  `validate_rag_options` and cannot be**: the three fields are **typed enums**
  (`RagQueryOptions.mode: Option<QueryMode>`, `.expand: Option<ExpandMode>`,
  `.compression: Option<CompressionMode>`, `src/store/mod.rs:639-667`), so an **unrecognized token is
  unrepresentable** in the parameter and `validate_rag_options` (`:4330-4375`; the pre-U3 citation was `:4293-4338`) contains **no**
  mode/expand/compression arm at all. The token check therefore lives **only** in the **U2 wire resolver
  of §5.4**, which runs **before** `validate_rag_options` is called — that ordering is exactly what
  makes precedence step (2) precede step (3) below. **Superseded-by-this-remand:** the pre-remand text
  read "the mode check of §5.4 is added **additively** to `validate_rag_options` with **unchanged error
  precedence**", which is structurally impossible and left U2 unable to locate the check; the
  **"unchanged error precedence"** claim survives as the *internal* order of the checks that remain
  inside `validate_rag_options` (`validate_rag_options` itself is **not edited by any unit** — U2 adds
  no arm to it).
- **`requester` is NOT part of this request contract.** The canonical §4.6.1 signature does not
  include it (`docs/specs/gnosis.md:872`) and the engine takes no `caller`/`requester` parameter
  (C5 + `GNOSIS-RBAC-EDIT-ENFORCEMENT`: the engine discards `caller` after the decode layer,
  pre-U2 line `src/bin/gnosis_server.rs:73-123`; the landed decode/dispatch path is `:121-201`). A `requester` key in the payload is therefore an
  **unknown/extra key**: tolerated and ignored, with `RagQueryOptions.requester` left `None` (the
  §4.3.4 audit entry records `""`, `src/store/mod.rs:4153`).
- **Non-string or absent `query`.** An **absent** `query` decodes to `""` — the same value the pre-U1 handler
  produced (`payload.get("query").and_then(as_str).unwrap_or("")`, pre-U1 line
  `src/bin/gnosis_server.rs:162-167`) — so
  that half's outcome is the engine's existing `ValidationError("query must be non-empty")` → **400
  `validation_error`**, **not** a transport decode error. (That message is already pinned by
  `tests/rag_query_integration.rs:290-299`.) A `query` that is **present but not a JSON string** is the
  table's other half: it is **not** coerced to `""` — the decoder returns
  `Err(QueryDecodeError::Validation(ValidationError))` at layer 1, which renders **400 `validation_error`**
  as well (the wire outcome is the same; only its producer differs — decoder vs. engine).

**Response body — the bare `RagResult` (NOT the SSE chunk wrapper).** On success the response is the F2
envelope (§4.1) whose `payload` is the **bare serde `RagResult` body** (F2 §4.2 / §12 V-5: snake_case
keys, PascalCase enum values, externally-tagged `trace`, `null` for absent `parent`/`stale`/`blocked_by`)
— **not** the SSE `{"type":"result","result":…}` discriminator wrapper. That discriminator belongs to
the `result` SSE frame only (F2 §4.2 / §4.4). `encode_result` (`src/wire/codecs.rs`) is the encoder.

**The encoder must never emit a body the result validator rejects (pinned).** The handler MUST NOT
return a 200 carrying a `RagResult` that `validate_rag_result` (F2 §7) rejects — whose **landed arms are
exactly two** (`engine != "gnosis"`, and `blocked_by` without a graph trace, `src/wire/decode.rs:63-71`); a
**traceless** result is **not representable** (`RagResult.trace: RagTrace` is non-optional,
`src/store/mod.rs:758-765`), so it is **not** a rejected class here. Such a body is rendered as
`StoreError::EngineError` → **502** (FS-9) instead of a silently un-decodable 200. **Reachability note (no invention):** today this
path is **not reachable from the live engine** — `rag_query` always sets `engine: "gnosis"` and always
produces a trace (`src/store/mod.rs:4413-4419` — flat; the pre-U3 citations `:4138-4145`/`:4124` are stale) — so this is a **U2-time totality
requirement** (it closes the encoder→decoder loop at the transport, extending F2's `P-TP-1`), tested at
the **pure** level; it is **not** a currently-reachable HTTP fail-state and must not be asserted as one.

**Fail-states (this endpoint) — all render through §7.1/§7.2.** Every §11-mapped error renders as an
error envelope (`{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"code":…,"message":…}}`
via `encode_error`); every transport-level failure renders per §5.5.

| condition | outcome | status | code |
| --- | --- | --- | --- |
| malformed JSON / bare non-envelope body | transport decode | **400** | `invalid_json` / `invalid_envelope` (§5.5) |
| **non-object `payload`** (**U2-time — LANDED 2026-09-17** F13: the shared decoder's object check (`src/wire/query.rs:83-87`) makes this row the landed transport decode outcome, with **exactly one** code — **`invalid_envelope`** — per `P-IM-4`/F9; **historical**: pre-U2 `Envelope::from_json` accepted any payload `Value` (`src/wire/envelope.rs:65-68`), the query handler's `payload.get("query")` missed and the outcome was `query=""` ⇒ **400 `validation_error`**, *not* a transport code) | transport decode | **400** | `invalid_envelope` (§5.5; F9) |
| unknown `schemaVersion` (**U2-time on this path — LANDED 2026-09-17**: the shared decoder checks it first, `src/wire/query.rs:73-77`; historical: the pre-U2 POST never checked it) | transport decode | **400** | `unsupported_schema_version` |
| unknown `idFormat` (**U2-time on this path — LANDED 2026-09-17**: checked second by the shared decoder, `src/wire/query.rs:78-82`) | transport decode | **400** | `unknown_id_format` |
| unrecognized `mode` / `expand` / `compression` **token** — resolved by the **U2 wire resolver** *before* `validate_rag_options` (§5.3/§5.4); `expand` is a **new state created by this amendment** (§5.4). **N1:** the **SSE** half of this row exists for **`mode` alone** — `expand`/`compression` are POST-payload-only and their params do not exist on `/rag/stream` | `StoreError::ValidationError` (FS-3 for `mode`/`compression`; `expand` added upstream via the U1 HANDOFF ask) | **400** (POST; and SSE for `mode`: 400 + one `error` frame) | `validation_error` |
| `query` absent/empty/non-string | `StoreError::ValidationError` (FS-3) | **400** | `validation_error` |
| `topK` / `maxHops` / `filters` / `multiQuery.n` / `binaryCandidatePool` out of range (`topK:0`/`51`, `maxHops:0`/`6`, `multiQuery:{enabled:true,n:0}`, `binaryCandidatePool:0` with `binaryFirstPass:true`) | `StoreError::ValidationError` (FS-3) | **400** | `validation_error` — **the range half only: every instance named here is a present `u64`**; a present value that is **not** a `u64` ⇒ absent ⇒ the option's default and **no** 400 (§5.3's type-vs-range bullet, ruling 1), and for `multiQuery.n` a wrongly-typed `n` ⇒ the **whole option is `None`** (§5.3's member rule, ruling 2). **For `binaryCandidatePool: 0` with `binaryFirstPass: true` the qualifier is: mode-qualified** (N4 — the validator rejects it in **every** mode, `src/store/mod.rs:4330-4336`, while the option is consumed by the **vector leg only**, so the fail-state is reachable in `flat`/`graph` where the option is **inert**; tests MUST NOT read a 400 in `flat` as proof the option is honored — §5.3's reachability note) |
| `multiQuery.enabled` with no distinct term to fan out to (READY engine) | `StoreError::MultiQueryExpansionFailed` (FS-19) — **reachable**, `src/store/mod.rs:5250-5252` (the pre-U3 citation was `:5212-5216`) | **500** | `multi_query_expansion_failed` |
| engine not `READY` — **every mode except `graph`** | `StoreError::EngineUnavailable` (FS-8) | **503** | `engine_unavailable` |
| `mode=graph` walk exceeding `maxHops` (reachable: `graph_walk`, `src/store/mod.rs:4718`; the pre-U3 citation was `:4680-4681`) | `StoreError::HopLimitExceeded` (FS-11) | **422** | `hop_limit_exceeded` |
| `mode=graph` walk detecting a `reference`→`fact` cycle (reachable: `src/store/mod.rs:4721`; the pre-U3 citation was `:4683-4684`) | `StoreError::CycleDetected` (FS-12) | **409** | `cycle_detected` |
| `mode=vector` with no built vector index, engine `READY` | `StoreError::VectorIndexUnavailable` (FS-14, explicit-leg) | **503** | `vector_index_unavailable` (§5.9) |
| `mode=vector` with an index built but no embedding provider | `StoreError::EmbeddingUnavailable` (FS-13) | **503** | `embedding_unavailable` |
| engine returns a validator-rejected result (**U2-time totality pin; not reachable today**) | `StoreError::EngineError` (FS-9) | **502** | `engine_error` |

- **Precedence (pinned, and it matters for tests):** (1) transport decode (envelope shape → version →
  id format) → (2) the **wire token resolver** (the §5.4 `mode` rule, plus the `expand`/`compression`
  token rules — the latter two on the **POST payload only**, N1) → (3) `StoreError::ValidationError`
  from `validate_rag_options` (the range/consistency
  checks) → (4) the READY gate → (5) the mode's own leg fail-states. Step (2) is **outside** the
  validator and **before** it — that is what makes it precede step (3) (see the `validate_rag_options`
  note above: an unrecognized token cannot reach the validator at all). So an unrecognized `mode` on a
  **not-READY** engine is **400, not 503**; an unrecognized `mode` **and** an out-of-range `topK` in the
  same request is the **mode** 400 (step 2 before step 3); and every non-`graph` mode on a not-READY
  engine is **503 `engine_unavailable` before** any leg availability is consulted
  (`src/store/mod.rs:4071-4080`; the pre-U3 citation was `:4059-4068`). `mode=graph` is **not** READY-gated
  (`src/store/mod.rs:4078` — a deterministic local walk on the core store; the pre-U3 citation was `:4066`).
- **`unknown_method` is unreachable on this path.** A `ragQuery` payload has no `method`
  discriminator, and U2's shared query decoder MUST NOT consult one: a payload that carries a
  `"method"` key is just another tolerated extra key. `DecodeError::UnknownMethod` → **422** belongs to
  the CRUD *method* decoder only (`src/wire/crud.rs:320-321`; `request_decode_status`,
  `src/server.rs:56`).

### 5.4 The single mode rule (shared by `POST /rag/query` and `GET /rag/stream`)

**Authority.** gate-1 ruling 3. The rule is **stated once here** and both paths MUST call it — the
POST payload key `mode` and the SSE query param `mode` resolve through the identical function
(U2-time — **LANDED**: one mode resolver, `resolve_query_mode(raw: Option<&str>) -> Result<QueryMode, StoreError>`
(`src/wire/query.rs:290-305`), called by both the POST and the SSE path — the name and the shared call are the
landed ones; the behavior below is pinned). The same single-sourcing requirement applies to the
`expand`/`compression` token resolvers named below: **one resolver per token family, called by every
path that reads that token** — U2 MUST NOT implement a token rule twice. **Which path reads which token
is pinned in the next paragraph** (`mode`: POST **and** SSE; `expand`/`compression`: the POST payload
only — the SSE surface has no such query params).

**The token resolver covers all three enum-valued option tokens (F2).** The same U2 resolver family
resolves `expand` (`none|parent`) and `compression` (`none|filter|extract|graph`) under the identical
casing policy, yielding `ExpandMode`/`CompressionMode`. **The `mode` token is the ONLY one of the three
with both a POST and an SSE outcome; `expand`/`compression` are POST-payload-only tokens** (N1 — this
supersedes the pre-remand "on **both** paths, exactly as for `mode`" clause, whose SSE half was
unreachable):

- **`mode` — two paths.** Both paths read the token: the POST payload key `mode` and the SSE query param
  `mode` resolve through the **identical** resolver (the table + fail-states below), so an unrecognized
  `mode` string ⇒ `StoreError::ValidationError` on **both** paths.
- **`expand`/`compression` — POST payload only.** On the **SSE surface those query params do not exist**
  in the pre-U4 surface: the SSE handler fills only the three-param `SseParams` carrier
  (`src/bin/gnosis_server.rs:246-283`, landed; pre-U3 `:245-263`, pre-U2 the handler built
  `RagQueryOptions { top_k, mode, ..Default::default() }` inline at `:190-203`), so its
  resolver is **never called for those two tokens** and the **SSE path adds no new query params in U2**
  (a `?expand=parent` param is simply absent ⇒ the option's default). **A `?expand=`/`?compression=`
  param is therefore ignored and yields the option's default — NEVER a 400** (and never an `error`
  frame). The SSE 400/`error`-frame outcome therefore belongs to **`mode` alone**; the two token
  resolvers are called by the **POST** path's decode.
- **`mode` — one rule, both readings.** An unrecognized `mode` token is the only token whose fail-state
  is pinned on the SSE surface: `?mode=bm25` ⇒ **HTTP 400 + one `error` frame** (F4, below); an
  unrecognized `expand`/`compression` token is a **POST-only** 400.

**A resolver is shared by both paths only where the parameter exists on both** — the single-sourcing
requirement is that any path which **reads** one of these tokens uses the one resolver, so the **rule**
MUST NOT diverge; two rules where one parameter exists is the defect this closes, and inventing an SSE
resolver call for a parameter that surface does not read is **not** a fidelity improvement.

**The check lives here (the wire layer) and nowhere else** — the three option fields are typed enums, so
no unrecognized token can reach `validate_rag_options`, which has no arm for them (§5.3). **New state
created by this amendment (pinned, no invention):** the canonical `ragQuery` throw column and FS-3's
closed catalogue (`docs/specs/gnosis.md:872`, `:1027`) enumerate **invalid `mode`** and **invalid
`compression`** but **not** invalid `expand` — the engine-side `expand` 400 is therefore a state **U1
creates**, and the `docs/HANDOFF.md` reconcile ask requests that the canonical contract add it (or that
the engine drop the `expand` 400 pin); until upstream answers, **the engine-side pin stands** (400
`validation_error`, never a silent default). The case-insensitive generalization is likewise a **benign
superset** of the canonical lowercase tokens and is **not** subject to that reconcile.

**How to read this table (pinned; the two-layer rule is stated once in §5.3 and cross-referenced here).**
The **`resolved`** column is the **effective reading at query time** — layer 2 of §5.3's two-layer bullet —
**not** the value the decoder writes into the options: for an **absent** or **wrongly-typed** `mode` the
decoder's output is **`RagQueryOptions.mode == None`** (`Some(Flat)` is **not** a decoder output) and the
engine resolves `None` to `Flat` (`mode.unwrap_or(QueryMode::Flat)`, `src/store/mod.rs:4060`, `:4168`), so
this table's "absent ⇒ `Flat`" cell and `P-IM-5`/`P-TP-2`'s element-wise `== None` assertions (§9.5.1) hold at
their own layers and **do not contradict** each other. This cross-reference adds **no** rule beyond §5.3's —
in particular it does **not** soften the token fail-state: a **string** that is not one of the four is still
`StoreError::ValidationError(…)` ⇒ **400**, on the paths that read the token (the table's last row).

| `mode` input (POST payload `mode` / SSE param `mode`) | resolved | outcome |
| --- | --- | --- |
| **absent** (`None`) | `Flat` | flat mode |
| **present but not a JSON string** (e.g. `"mode":5`, `"mode":null`, `"mode":{}`) | `Flat` | decodes as **absent** (the wrongly-typed rule below) ⇒ flat mode — the token rule is never invoked |
| `"flat"` (any ASCII casing: `flat`, `Flat`, `FLAT`, `fLaT`) | `Flat` | flat mode |
| `"graph"` (any ASCII casing) | `Graph` | graph mode |
| `"vector"` (any ASCII casing) | `Vector` | vector mode — explicit leg; §5.9 |
| `"hybrid"` (any ASCII casing) | `Hybrid` | hybrid fusion; §5.9 |
| **a string that is not one of the four** — including `""`, whitespace, `"bm25"`, `"flat "`, `"flat\n"`, `"hybrid "` | — | `StoreError::ValidationError(…)` |

**Fail-state rendering (pinned; `mode` on both paths — see the split above):**

- **POST:** **400** + the §11 wire code **`validation_error`** (the §11 map is reused verbatim —
  `ValidationError` → 400, `src/server.rs:23`; F2 §11) rendered as the error envelope via
  `encode_error` (the same shape as any other §11-mapped error on this route).
- **SSE `GET /rag/stream`: HTTP `400`** — *this is the `mode` token's outcome only* (the SSE surface
  reads no `expand`/`compression` param — see the split above). The same `StoreError::ValidationError` →
  **400** rule the POST path uses (`server_status(&ValidationError(…)) == Some((400, "validation_error"))`,
  `src/server.rs:23`), rendered by the U2 post-U2 pre-stream renderer
  (`sse_decode_error_response`, `src/bin/gnosis_server.rs:100-115`; pre-U2 the same frame came out of the
  handler's own `StoreError` branch, now `:272-281`; the pre-U3 citations were `:99-114` / `:264-281`) — with **one** SSE frame as the response **body**:
  `{"type":"error","code":"validation_error","message":"<detail>"}` in the F2 §4.4 single-event framing
  (`event: error\ndata: …\n\n`) — the F2 §4.2 error data shape — and then the stream closes.
  **Both halves are pinned (status *and* body):** U2 MUST NOT route the mode failure through a path
  that skips the status line (`encode_event` alone is a framing helper and implies no status), and the
  error frame MUST NOT be emitted under a `200`. For an `error` **frame inside an already-opened `200`
  stream** the §10/§9 statuses are unchanged (a mid-stream `StoreError` is emitted as an `error` frame
  on the open `200`); this 400 is specifically the **pre-stream** mode failure, where no stream has
  opened. The landed renderer emits the frame via `encode_event` (`src/bin/gnosis_server.rs:105`; the pre-U3 citation was `:104`) — the
  option the pin allows — and the post-stream `StoreError` branch does the same (`:273`); the **status is
  400** and the bytes are pinned by §4.4/§4.2.
- **SSE `Content-Type` (F-2, pinned by the proofread pass, 2026-09-17 — a header pin, no wire-outcome
  change).** **Every SSE response carries `Content-Type: text/event-stream` — exactly that media type, with
  no `charset` parameter:** the pre-stream `error`-frame responses of this sub-section (the mode 400 *and*
  the §11-mapped pre-stream `StoreError` frames, e.g. the not-READY **503**), the `200` frame stream, and the
  U4 `cursor`/`change` feed of §5.6. Rationale (pinned): the response body *is* an SSE stream/frame, so the
  JSON bodies' media type (§5.5's `application/json`) never applies to it, and a consumer that keys on the
  media type must not have to special-case which branch produced the frame. The landed state is **partially
  divergent — recorded, not absorbed:** the U2 pre-stream renderer emits `text/event-stream`
  (`src/bin/gnosis_server.rs:100-115`, the header set at `:109`; the pre-U3 citations were `:108`), while the **pre-existing** post-stream `StoreError` branch returns the
  frame bytes through axum's `String` responder, i.e. `text/plain; charset=utf-8` (`:272-281`; the pre-U3 citation was `:264-281`) — filed as
  `docs/defects.md` **P-5** (owner: the transport unit / the SSE branch; not a U2 regression).
- **Never a silent default.** Pre-U1 the SSE path matched **case-sensitively** and silently coerced an
  unknown `mode` to `None` ⇒ `Flat` (`src/bin/gnosis_server.rs:195-201` in the **pre-U1 tree**); the POST path
  ignored the field entirely (`src/bin/gnosis_server.rs:162-168`, pre-U1). Both are the defect GR-2 reports;
  both are closed by
  this rule. On the **SSE** surface the "never a silent default" obligation therefore concerns **`mode`**
  only: an unrecognized `?expand=`/`?compression=` param **cannot** be defaulted silently, because that
  param is **not read at all** (N1).
- **Never a decode-level 422.** `UnknownMethod` (422) is for an unknown CRUD *method name* only. The
  mode rule produces `StoreError::ValidationError`/400 — it never produces a `DecodeError` and never a
  422, and it is **not** a transport decode error (§5.5's code table does not contain a mode code).
- **Canonical authority:** FS-3 already names "invalid `mode` (not `'flat'`/`'graph'`/`'vector'`/
  `'hybrid'`)" (`docs/specs/gnosis.md:1027`); the engine simply never honored it on the wire.

**Boundary states (pin them so the TestWriter has a definite outcome).** An **empty** `mode` value
(`?mode=` on SSE; `"mode":""` on POST) is **present-but-unrecognized** ⇒ `ValidationError`
(**400** on POST / **400** + the `error` frame body on SSE — §5.4's status pin) — *not* "absent".
Whitespace-padded values are likewise unrecognized; the rule does **not** trim, and it compares the token
case-insensitively only. Absent means **the key/param is not present at all** (and a **non-string**
`mode`, e.g. `"mode":5`, decodes as absent ⇒ `Flat` — §5.3's wrongly-typed rule, read at the two layers of
§5.3's "**THE TWO-LAYER READING OF THE `absent ⇒` COLUMN**" bullet: the **decoded** field is `None`, while
**`Flat` is the engine's query-time reading**). Absent ⇒ `Flat` is
what keeps the existing e2e POSTs green: they carry no `mode`
(`tests/gnosis_server_e2e.rs:188-198`, `tests/blind_p2_gnosis_server_greens.rs:437-447`), so they still
reach the READY gate and still return **503** while the engine is not READY.

**The pre-U4 SSE surface reads exactly `query`, `topK`, `mode` (N1 — named, so it is not an unowned
gap).** The canonical `ragStream` signature additionally lists `wikiId?`, `filters?`, `maxHops?`,
`maxParentContext?`, `multiQuery?`, `compression?` and `hyde?` (`docs/specs/gnosis.md:873`); **none of
those seven is part of the pre-U4 SSE surface** — a `GET /rag/stream` query param outside
`query`/`topK`/`mode` (e.g. `?wikiId=w1`, `?filters=…`, `?maxHops=2`) is an **unknown param, ignored,
with the option left at its default** (never a 400, never a silent option change), exactly as an unknown
payload key is tolerated on POST (§5.3). **Their wire treatment is U4's to pin** (in U4's own spec, as
the route-parameter growth §5.2 requires) — **not** an upstream `docs/HANDOFF.md` ask: the canonical contract
already carries these params as an options object, and no canonical sentence is contradicted by the
engine's narrower SSE surface, so no `docs/HANDOFF.md` row is owed for this. Until U4 pins them, the
only truthful statement is the three-param surface, and the SSE parity claims of §10/§5.4 are scoped to
`mode`.

**`topK` on the SSE path is unchanged by U1.** `GET /rag/stream`'s `topK` param is parsed tolerantly
(pre-U2 in the handler, `params.get("topK").and_then(|v| v.parse().ok())`,
`src/bin/gnosis_server.rs:194`; post-U2 in the shared decoder, `sse_params.top_k.and_then(|s|
s.parse::<u64>().ok())`, `src/wire/query.rs:157`): an
unparseable value is treated as absent ⇒ default `10`, while an out-of-range value (e.g. `0`, `51`)
reaches `validate_rag_options` ⇒ 400 `validation_error`. U1 pins **no change** here (the GR-2 ruling
covers `mode`; tightening numeric parsing is not authorized). The POST path must apply the same
`topK` decoding (`number` → `top_k`), with the same engine-side range fail-state.

### 5.5 The structured transport request-decode error body (NEW-2; U1 pins the shape, **U2 LANDED it — 2026-09-17**)

**Authority.** gate-1 ruling 2 + the GR-1 verdict row. Pre-U1 a request-decode failure returned the
literal plain-text body `"request decode failed"` with axum's `text/plain; charset=utf-8`
(`decode_error_response`, `src/bin/gnosis_server.rs:57-66` in the **pre-U1 tree**) — which the consumer's
masking client
cannot surface. U1 pins the replacement. **Code lands in U2 — LANDED (2026-09-17):** the shared renderer is
`decode_error_response` (`src/bin/gnosis_server.rs:62-75`; the pre-U3 citation was `:61-74`), returning the JSON body below.

**Shape (pinned).**

- **`Content-Type: application/json` — EXACTLY (F15).** The header value is the bare media type, with
  **no `charset` parameter** (byte-exact `content-type: application/json`), which is what F2 §12's V-15
  byte-exact claim requires; the pre-U1 plain-text body carried axum's
  `text/plain; charset=utf-8`. No envelope wrapper.
- **Body:** `{"code":"<transport code>","message":"<detail>"}` — the F2 §4.3 canonical error JSON,
  **not** envelope-wrapped. Rationale (pinned): the envelope is precisely what failed to decode, so an
  envelope-shaped error response would assert an envelope contract on a request that did not satisfy
  it, and the consumer must be able to parse the error even when its own envelope assumptions are
  wrong. The body is therefore the **minimal** `{"code","message"}` pair.

**The code table (derived from the existing `DecodeError` variants, `src/wire/decode.rs:10-32`).** The
`code` values are **transport-level**, **distinct from `StoreError::wire_code()`**, and MUST NOT be
added to the §11 map:

| `DecodeError` variant (`src/wire/decode.rs`) | transport code | transport status | `message` |
| --- | --- | --- | --- |
| `InvalidJson(String)` (l. 12) | `"invalid_json"` | **400** | the carried string, verbatim |
| `InvalidEnvelope(String)` (l. 22) | `"invalid_envelope"` | **400** | the carried string, verbatim |
| `UnsupportedSchemaVersion(u32)` (l. 18) | `"unsupported_schema_version"` | **400** | `"unsupported schemaVersion: {v}"` |
| `UnknownIdFormat(String)` (l. 20) | `"unknown_id_format"` | **400** | the carried string, verbatim |
| `UnknownMethod(String)` (l. 27) | `"unknown_method"` | **422** | the carried string, verbatim |
| any other `DecodeError` variant (`UnknownType`, `MissingTrace`, `UnknownCode`, `EventTypeMismatch`, `ValidationFailed`) | — (`None`) | — | never rendered as a request-decode body |

- **Status rule.** The status is exactly the existing NEW-2 transport outcome
  (`request_decode_status`, `src/server.rs:51-62`): **400/422, never 502, never a §11 row.** The code
  mapping MUST be **total over the same domain** as `request_decode_status` (the 5 request-decode
  variants) and `None` outside it, so a status and a code are always defined together (U2-time — **LANDED**: the
  sibling pure fn is `request_decode_code(e: &DecodeError) -> Option<&'static str>`, `src/wire/query.rs:339-350`;
  the mapping is pinned).
- **`message` (restated by the register remand, F8 — the pre-remand "non-empty for every render" clause is
  too strong).** The four string-carrying variants carry their string **verbatim, and it MAY be empty**
  (`InvalidJson(m)`/`InvalidEnvelope(m)`/`UnknownIdFormat(m)`/`UnknownMethod(m)` — the decoder does not
  substitute a placeholder); **`UnsupportedSchemaVersion(v)` is the one variant whose message is
  guaranteed non-empty** (`"unsupported schemaVersion: {v}"`, a decimal render of the value). TestWriters may
  assert
  `code` + status **byte-stably**, and `message` byte-stably for the pinned cases (e.g.
  `UnknownMethod("bogus")` ⇒ `"bogus"`); they must **not** assert serde's own parse-error text for
  `InvalidJson` (it is a serde-implementation string, not a contract token). The register's `P-TP-3`
  (§9.5.1) pins the **pure** code/status/disjointness half only; the rendered bytes are the
  `v15_request_decode_error_body_exact` golden (F2 §12 V-15/V-15.1).
- **Disjointness (testable).** Every transport code is **absent** from the §11 map: for each of the five,
  `StoreError::from_wire(code, Some(msg)) == None` (`src/wire/error.rs`) — a client must not feed these
  codes to the `StoreError` reverse lookup.
- **Shared rendering.** The JSON body replaces the plain text for **every** route that can fail to
  decode a request — the CRUD handler (`src/bin/gnosis_server.rs:192-205`) and the query handler
  (`:207-240`; the pre-U3 citations were `:191-201`, `:206-214`; the pre-U2 line numbers were `:143-153`/`:157-161`) call the same
  `decode_error_response` (`:62-75`; the pre-U3 citation was `:61-74`). Both therefore changed together in U2. **No
  existing test pins the plain-text body or its content type** (a sweep of `tests/` finds no assertion
  on `"request decode failed"`), so this is not a regression.
- **The SSE route's own header is NOT this body's media type (pinned; §5.4's F-2 pin).** Every SSE
  response — the pre-stream `error`-frame responses included — carries `Content-Type: text/event-stream`,
  **not** `application/json`: this sub-section's JSON body belongs to the POST/CRUD request-decode paths
  (and to the SSE handler's transport-failure fallback only, where no frame can be formed). The one
  pre-existing divergence (the post-stream `StoreError` frame rendered as `text/plain; charset=utf-8`) is
  `docs/defects.md` **P-5**.
- The `None` domain of the mapping is **unreachable on the request-decode path**: `UnknownType` /
  `MissingTrace` / `UnknownCode` / `EventTypeMismatch` / `ValidationFailed` are response/SSE-side
  variants (F2 §7/§8) and never reach `decode_error_response`. The current `.unwrap_or(400)` fallback
  in `decode_error_response` stays **defensive only** and is never exercised.

**Worked example (byte-pinned).**

A CRUD request envelope with an unknown method (the status was reachable pre-U1 as *status-only*; the JSON body
itself was U2-time and **LANDED 2026-09-17** — `decode_error_response`, `src/bin/gnosis_server.rs:62-75`; the pre-U3 citation was `:61-74`):

```
POST /documents HTTP/1.1
content-type: application/json

{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"method":"bogus","args":{}}}
```

```
HTTP/1.1 422 Unprocessable Entity
content-type: application/json

{"code":"unknown_method","message":"bogus"}
```

And the query-path envelope-level failure (U2-time — **LANDED 2026-09-17**: the shared decoder checks the
version first, `src/wire/query.rs:73-77`; historical: the pre-U2 `POST /rag/query` did not check the
version):

```
HTTP/1.1 400 Bad Request
content-type: application/json

{"code":"unsupported_schema_version","message":"unsupported schemaVersion: 99"}
```

### 5.6 The change cursor + the `GET /changes` SSE change feed (U1 pins the contract; code lands in U4)

**Authority.** gate-1 **ruling 1** (+ Appendix A) and the **GR-4/GR-5** verdict rows; decision
`GNOSIS-CHANGE-CURSOR` (**ACTIVE**). The **wire definition** of the cursor is pinned in
`docs/specs/engine-wire-contract.md` §4.6 and the feed's framing/field table in §4.7; this sub-section
pins the **route** and the consumer-facing behavior. **Code lands in U4** (the cursor accessor, the
journal `kind` + affected ids, the route row, and `P-IM-3`'s extension all land together in U4).

**No consumer-visible store-wide revision (the refusal, restated for this surface).**
`GET /snapshot?revision=…` and a `stale_revision` error are **REFUSED, not parked** (§2, and the
reasons there). As-of-N reconstruction is a **journal-retention** concern — boundary **C4**
(`docs/research/astrographer-engine-shell-boundary.md:86`) — belonging to the future durability design
unit, **never** to a query route.

**The route (U4-time).** `GET /changes` — SSE, no request body, no required query params, **no READY
gate** (the cursor is a property of the store's journal, so a cold-start consumer can always learn the
current cursor; the feed is not a retrieval call and MUST NOT return `EngineUnavailable`).
Row added to `route_bijection()` **and** `router()` **and** `P-IM-3`'s assertions in U4 (§5.2's
amendment rule). It carries **no** §11-mapped fail-state: the subscribe path performs no fallible store
operation.

**Framing.** The feed reuses the F2 §4.4 **single-event framing verbatim** (`event: <type>\ndata:
<single-line JSON>\n\n`, LF, terminal blank line, **no `id:`/`retry:`, no multi-chunk/continuation
lines**, and the `event:` label MUST equal the data JSON's `"type"`), and extends the **event-type
family** with `cursor` and `change` (F2 §4.7). Its **media type is the SSE one** — `Content-Type:
text/event-stream`, exactly, no `charset` (the §5.4 F-2 pin applies to *every* SSE response, this feed
included) — and it does **not** reuse the `result`/`done`/`error`
retrieval family's payload shapes — `done` never terminates the feed (the stream is a subscription),
and `F2`'s `decode_event` (a `RagChunk` decoder) MUST reject a `cursor`/`change` frame as
`DecodeError::UnknownType` (reachable today: `decode_chunk_payload`'s catch-all, `src/wire/decode.rs:114`),
so the two families can never be confused. A U4-time `decode_change_event` owns the change family's
round-trip.

**Frames.**

| # | event | data JSON | meaning |
| --- | --- | --- | --- |
| 1 | `cursor` | `{"type":"cursor","cursor":"<opaque>"}` | **cursor-first frame**, sent on **every** connection (cold start included): the consumer learns the current cursor before any change frame |
| 2..n | `change` | `{"type":"change","cursor":"<opaque>","wikiId":<string\|null>,"kind":"<token>","nodeIds":[…],"edgeIds":[…],"timestamp":"<ISO-8601>"}` | one frame per committed journal entry, in **commit order** (strictly increasing `cursor`) |
| fail | `error` | `{"type":"error","code":"…","message":"…"}` | F2 §4.2's error data shape, reusable if the stream ever fails after the head; none is reachable on the subscribe path (§5.6's route note) |

- `cursor` — the **opaque change cursor** (F2 §4.6): a **string**, decimal rendering of the committed
  journal `seq`; advances by exactly 1 per committed journal entry; process-lifetime monotonic.
- `wikiId` — the affected wiki's opaque id, or `null` when the entry is not wiki-scoped. **Reachability
  note (no invention):** every one of the 20 `op` values appended today is wiki-scoped (a document, a
  wiki, or a wiki-owned fact/community/triple change), so `null` is a **U4-time** possibility that the
  contract permits — a U4-owned entry kind that affects no single wiki is what would produce it. U4 MUST
  NOT invent a non-wiki-scoped entry merely to exercise `null`; if no such kind exists when U4 lands,
  the `null` case is documented-but-unreachable and must not be asserted as a live state.
- `kind` — the committed journal entry's operation token, from the closed set appended at the call
  sites today: `create_document`, `update_document`, `delete_document`, `publish_document`,
  `unpublish_document`, `archive_document`, `create_wiki`, `add_triple`, `declare_community`,
  `update_community_summary`, `set_reference_state`, `resolve_entities`, `merge_facts`, `create_fact`,
  `update_fact`, `propose_candidate_fact`, `re_sync_embed`, `re_derive_community`,
  `propagate_fact_staleness`, `propagate_archived_target` (`src/store/mod.rs:2284-4018`). Any new
  `append_journal` call site extends this wire-visible set **and U4's spec** in the same unit.
  - **Unknown-token tolerance (pinned, F11).** A **consumer MUST tolerate an unknown `kind`**: the frame
    is still a valid change frame and the consumer treats it as an **opaque change** (invalidate/resync
    on the affected `wikiId` + ids; it MUST NOT discard the frame and MUST NOT fail the stream). The
    engine may therefore add a kind without breaking an older consumer.
  - **Rename binding (pinned, F11).** The engine MAY rename an `op` literal **only** through an amendment
    unit that updates **this table and F2 §4.7's `kind` closed set and U4's conformance rows in the same
    unit**. The vocabulary is engine-internal today (`op` is an `&'static str` at the append site,
    `src/store/mod.rs:1820-1831`) and no `tests/` assertion binds the literals, so a rename would
    otherwise change the consumer-visible vocabulary silently: the conformance assertion over the
    `op` set is therefore **owed by U4's spec** alongside the `kind` field itself.
- `nodeIds` / `edgeIds` — the affected ids. **`nodeIds`** are `NodeId` opaque strings (the store's id
  newtypes cross the wire as opaque strings, F2 §4.1). **`edgeIds`** is an array of opaque strings whose
  **element encoding U4 must define** — the store has **no edge-id type** today (an edge is identified
  structurally: `Edge { source, target, kind, … }`, `src/store/mod.rs:163-176`), so U1 pins the
  **field** (present, array of strings, possibly empty) and leaves the identity encoding to U4's own
  spec. `JournalEntry` today carries **no** affected ids and no kind beyond its `op` string
  (`{seq, op, base_revision, timestamp}`, `src/store/mod.rs:944-953`) — the ids/kind extension is U4's.
- `timestamp` — the committed entry's ISO-8601 UTC timestamp (`iso_now()`,
  `src/store/mod.rs:1829`).

**Staleness contract (what a consumer may assume).** Pinned:

1. **The cursor is a notification trigger, not a read barrier.** Between a successful write and the
   delivery of its frame, a concurrent read the consumer issues may or may not include the write. A
   consumer MUST NOT treat "I have seen cursor N" as "a read I issue now would return the state at N".
2. **The cursor is not point-in-time and not a validation token.** It MUST NOT be used for cache
   *validation*, optimistic concurrency, or state reconstruction (F2 §4.6: per-document `revision`
   remains the only concurrency token). Its only sanctioned uses are **invalidation, resync and
   de-dup**.
3. **A frame's cursor advances by exactly 1** over the previous frame's cursor in the same process
   (each committed journal entry has its own frame — no coalescing of distinct seqs into one cursor
   value; frames may in principle repeat a `kind` but never a `cursor`).
4. **Frames may be lost** (slow consumer, dropped connection) — the feed is **not** a durable log.
5. **The cursor alone says nothing about *what* changed.** The frames carry `kind` + `wikiId` + ids so
   a consumer can discriminate (the propagation kinds `propagate_fact_staleness` /
   `propagate_archived_target` / `set_reference_state` are ordinary entries in the sequence, exactly as
   the ruling states — the cursor advances 1 per committed entry, including those).
6. **Process-lifetime only.** There is no durable store: after an engine restart the cursor restarts
   and **may repeat** values from the previous process. A consumer MUST NOT persist a cursor across a
   restart and treat it as comparable; the **cursor-first frame** is authoritative for the current
   process. (Durability is the parked GR-7/`ENGINE-DURABLE-CORPUS-DIRECTION` concern.)

**Reconnect / resync rule (pinned).** There is **no retained replay log** and **no
`?since=<cursor>` backfill**: a consumer that missed frames cannot ask for them. On reconnect or when
it detects that a frame's cursor jumped by more than 1 (or that the cursor-first frame is ahead of the
last cursor it saw), the consumer **MUST re-read** the affected state from the engine's read routes and
then resume de-dup from the cursor-first frame. A de-dup/coalescing cache is bounded by
**SHELL-2** (consumer side); the engine guarantees only order + step-1 within a process.

**Read routes are U4, not U1.** The paginated, wiki-scoped node/edge read routes that make a resync
possible (GR-4 re-shaped) are **U4**; U1 pins only the cursor, the feed and the bounds discipline
(§5.7).

### 5.7 Bounds discipline for the paginated read routes (U1 pins the rule; code lands in U4)

**Authority.** gate-1's residual-risk row: *"Unbounded payloads on the new list routes — MUST be
mitigated inside U4 (page-size cap + bounds, mirroring `DocumentList{page,page_size,total}`); shipping
U4 without caps is a **defect, not a residual**"*.

Pinned now so U4 cannot land an uncapped route:

- Every paginated read route MUST carry a **page size cap of 100** and a **`page >= 1`** rule,
  mirroring the existing `DocumentList` precedent: `page_size` outside `1..=100` or `page < 1` ⇒
  **`StoreError::ValidationError("page must be >= 1 and page_size must be in 1..=100")`** → **400
  `validation_error`** (`src/store/mod.rs:2733-2735`, `:3628-3630`; the same rule the CRUD
  `listDocuments` route already renders, §7.1/§10).
- The response shape MUST carry the total/bounds metadata analogous to
  `DocumentList{items, total, page, page_size}` (`src/store/mod.rs:930-935`), so a consumer can tell
  whether it has paged to the end.
- **Over-cap is a fail-state, never a silent truncation**: returning the first 100 rows for a
  `page_size` of 1000 (or an unbounded response) is a contract violation.
- An optional `cursor`-tagged read parameter (GR-4's re-shaped ask) is **U4's** to pin, including
  whether a cursor that is not comparable to the current process's is tolerated (it must be — see
  §5.6's restart rule). **Pinned now so U4 cannot land an undefined one (F9):** on a read route the
  `cursor` field is a **resume token**, not an as-of-N cursor (F2 §4.6 refuses request-side
  **as-of-N / point-in-time** semantics only), and it MUST be **tolerated-but-DEFINED** — a cursor
  that is not comparable to the current process (a stale/foreign/too-large value, e.g. after a restart)
  MUST NOT be silently ignored as if it were absent and MUST NOT fail: the pinned outcome is that the
  request **yields the first page** (the consumer's resync contract, §5.6's reconnect rule). **U4's spec
  owes this pin explicitly** (it may replace "first page" with another *defined* outcome only by pinning
  it there, with F2 §4.6/§4.7 updated in the same unit).
- **These routes do not exist at U1.** No path in the §5.2 table is added by U1; the amendment lands
  with the routes in U4 (§5.2's amendment rule).

### 5.8 Subsystem-flag capability semantics (contract pinned in F2 §9.1; **U3 LANDED-GREEN 2026-09-17** — the boot index build is **U5**'s; **U5 AUTHORIZED 2026-09-22, its contract + typed register authored in §9.5.5**)

**Authority.** gate-1 **ruling 4**; decision `SUBSYSTEM-FLAG-CAPABILITY-SEMANTICS` (**ACTIVE**).

- An `EngineSubsystems` flag means **"this subsystem's full query-time capability is wired and
  functional for the current store"** — **not** "a provider exists".
- Under that definition the **six hard-coded `true` flags** at construction
  (`src/store/mod.rs:1718-1725` — the LEGACY all-`true` literal, **which now sits under its own U3 comment
  block at `:1714-1717`**; **the pre-U3 citation `:1710-1717` predates both, and the `:1714-1725` spelling
  this section carried after the U3 landing mis-joined the comment block to the literal** — the drift the U3
  comment insertions caused) were **false claims while they were the status producer**:
  `reranker: true` names a subsystem with **no implementation anywhere in `src/`**, and `vector: true` is
  false while the boot path leaves `vectors: None` (`swap_snapshot(DerivedIndexes::default())`,
  `src/bin/gnosis_server.rs:390-404`, landed; pre-U3 `:385`, pre-U2 `:329` — the old `:385` now falls
  inside the boot's U3 wiring comment).
  **`embedding: true` was also a false claim in the DEGRADED case**: the server's boot path never called
  `set_subsystems` (`src/store/mod.rs:1868-1876` — write-only and **observationally inert** after U3;
  the pre-U3 citation was `:1862-1863`), so a boot with no reachable provider still reported
  `embedding: true` while the state was `DEGRADED`
  (`src/bin/gnosis_server.rs:391-408`, landed; pre-U3 `:390-391`, pre-U2 `:334-336`).
  **U3 LANDED-GREEN (2026-09-17): the derivation now makes the flag honest at read time**
  (`get_engine_status`, `src/store/mod.rs:4193-4232`), so the construction literal above is a **dead
  value read by nothing** — the historical defect text stands as the record, not as current behavior.
  `lexical: true` **is** honest (a live shard-scan BM25 leg, no prebuilt index), and so are `store`/`graph`.
- **`GET /engine/status` remains the single status surface**, always **200**, and **no new endpoint**
  is added (F2 §9.1 pins the additive amendment; p2 §6/§10 keep the always-200 row).
- The **additive** signal needed to express "capability wired vs index built" (for `vector`/`reranker`)
  is permitted **only** through the F2 §9.1 amendment, and the register row that pins the status
  projection is discussed there.
- **Where the additive signal may land (F7; pinned).** The additive change MUST be a new field on
  **`HealthReport`** — F2's own wire type (`src/wire/status.rs:14-21`) — or a **documented value
  convention within the existing six booleans**. `EngineSubsystems` is a **canonical §4.6.1 store
  type** (`docs/specs/gnosis.md:874`; `src/store/mod.rs:568-575`), **frozen**: it **MUST NOT gain, lose
  or re-type any field**. Adding a field there is *not* an F2-amendment-only change — it is a change to
  the frozen store surface, and it would break every `EngineSubsystems { … }` struct literal —
  **twenty-three** of them across `tests/` (***REMAND-2 SHOULD-FIX 6, 2026-09-22 — the pre-remand text said
  "nine" and cited four anchors that do not exist in the current tree; the figure is now **re-counted from
  the tree this pass** and reconciled with `docs/specs/engine-wire-contract.md` §9.1's F7 census, which
  publishes the same **23** and the same per-file enumeration***): `tests/wire_conformance.rs` **5**
  (`:698`, `:714`, `:727`, `:770`, `:1266` — the `derived_read` helper's return-type-only mention at `:1180`
  is **not** a literal), `tests/props_wire.rs` **5** (`:282`, `:689`, `:697`, `:705`, `:713`),
  `tests/props_gnosis_server.rs` **10** (`:4231`, `:4253`, `:4591`, `:4599`, `:5000`, `:5497`, `:5505`,
  `:5513`, `:5522`, `:5572`), `tests/blind_u3_status_honesty_greens.rs` **3** (`:164`, `:916`, `:1074`), and
  `tests/rag_query_integration.rs` **0** (**zero** `EngineSubsystems` occurrences — the pre-remand
  `:1649`/`:1668` anchors and the `tests/props_wire.rs:282`-as-signature reading were both phantom: `:282`
  is a literal, its `fn` **signature** being `:281**) —
  and force a canonical-contract amendment. **U3's blast radius is therefore `HealthReport` + `status.rs` + the `V-8` golden literals
  only**; if U3 wants the capability-vs-index distinction, it adds the signal to `HealthReport` and the
  V-8.x literals in the same unit.
- **Unit split (pinned):** the flag **code** change lands in **U3** (`reranker: false`; `vector: false`
  until U5; flags derived from real state), the **boot index build** in **U5**, and the **V-8 golden
  literal edit lands in the same unit as the flag change** (U3) — see this file's §U1 status block for
  the deliberate transient divergence.
- **U5's answer to this section's open question (2026-09-22; docs-only, ADDITIVE — the clause above is the
  pre-U5 record and stands as written).** ***(U5 spec-gate REMAND-1 clarification, 2026-09-22 (same
  pass-level date as the entry below): the `vector` predicate keeps its reading for **every** snapshot,
  while the **V-8.1 mask** is the mask of a `Reachable` boot **whose snapshot has `vectors: Some`** — a
  `Reachable` boot over a snapshot with `vectors: None` (no build was attempted) derives `vector:false`
  by the same predicate and reports a `Ready` + `vector:false` mask, never V-8.1. The unconditional rule
  is `flags.vector == snapshot().vectors.is_some()`; no row in §9.5.5 may read V-8.1 as "`Reachable` ⇒
  `vector:true`" without the `Some` half.**)*** The `vector` predicate `snapshot().vectors.is_some()` is
  **unchanged** by U5; what U5 changes is **what the boot puts in that snapshot** — a **reachable** provider
  means the boot builds the index (an **empty** index over an empty store is still
  `vectors: Some(_)` ⇒ `vector: true`), while `Absent`/`Unreachable` builds **no** index ⇒
  `vectors: None` ⇒ `vector: false` (the honest value, and the one `engine-status`'s e2e probe and the
  live battery's `R-L2` observe). **Pinned in §9.5.5** (`strat:vector-flag-flip`), where the three
  `BootProvider` outcomes, the corpus, the field choice, the failure outcome and the hermeticity rule are
  answered; the V-8.1 `"vector"` literal and the fixture helpers move **in that same unit** (F2 §12's
  golden-literal discipline, §9.5.5's "must move in the same unit" table).
- **U6's amendment to this section's `vector` predicate (2026-09-22; docs-only, ADDITIVE — the U5 clauses above
  stand as the U5-time record and are not rewritten).** The **U6** honesty/freshness unit (§9.5.6) narrows the
  `vector` flag's capability predicate from *"the current snapshot carries a vector index"* to *"the current
  snapshot carries an index that is **not stale**"*:
  **`subsystems.vector == snapshot().vectors.is_some() ∧ snapshot().epoch == self.epoch()`**. Everything else
  in this section is unchanged: the flag is still a **read-time derived projection** (never a stored mask, F16),
  `store`/`graph`/`lexical` stay always-`true`, `reranker` stays `false`, `EngineSubsystems` stays **frozen**
  (six `bool`s; U6 adds **no** `HealthReport`/`EngineSubsystems` field and **no** new flag) — **the freeze is a
  freeze of its SHAPE, not of its VALUES (REMAND-1, 2026-09-22, NOTE 12: the distinction must be stated inside
  the section that changes the derivation):** U6 moves the `vector` flag's **value** (it gains the epoch term)
  while its **type, arity and field set do not move** — six `bool`s, nothing added, lost or re-typed, no
  `HealthReport` field and no new flag, and the canonical §4.6.1 store type (`docs/specs/gnosis.md:874`;
  `src/store/mod.rs:568-575`) is untouched. Reading "frozen" as "the flags' values are fixed" is wrong and
  predates even U3 (whose derivation already moved `vector`/`embedding`/`reranker` at read time, `:1614-1616`)** —
  and `GET /engine/status` stays the single always-200 status surface. **The U5-time sentence above ("the `vector`
  predicate is **unchanged** by U5; U5 changes only *what the boot puts in the snapshot*") is TRUE OF U5 and
  stands as its record** — U6 is the unit that changes the predicate, and it does so by **adding** the epoch
  term, which is why U5's boot-built index keeps reading `true` (the boot composes its snapshot at
  `store.epoch()`, §9.5.6's `P-IM-18`) while a **post-boot mutation** now makes the flag honest (`false`) and
  the `mode=vector` read **loud** (FS-14 `vector_index_unavailable` ⇒ 503). **V-8.1's preconditions are now
  THREE** — `Reachable` **and** a snapshot whose `vectors` is `Some` (the U5 REMAND-1 clarification above)
  **and** that snapshot being **epoch-aligned** (`snapshot.epoch == store.epoch()`) — because a stale
  index-bearing snapshot on a `Reachable` boot reports `Ready` + `vector:false` (not V-8.1); the **V-8.1
  literal itself does not move** (REMAND-1, 2026-09-22: this third precondition is the same one
  `tests/wire_conformance.rs`'s fixture comment and §9.5.6's same-unit table pin). The V-8.1/V-8.2 literals and the
  fixture helpers do **not** move in U6 (a fresh `Store::new()`-based probe is already epoch-aligned); only
  the **comments** that state the superseded predicate are reconciled (§9.5.6's same-unit table).

### 5.9 Retrieval-semantics reconcile — FS-3 vs FS-13/14/15 (U1 pins the ruling; no code change)

**Authority.** gate-1 **ruling 5** (+ Appendix B), accepted by the go-ahead; the long-open
`docs/HANDOFF.md` FS-13/14/15 row is closed by this amendment (engine-side) with an upstream reconcile
row added to `docs/HANDOFF.md`. **This ruling describes behavior the engine already has** — it is a
documentation reconcile, not a code change; the only code-bearing consequence is U2's mode decode
(§5.4) and the pin is written once in F2 §16.

**The rule, stated once: explicit-leg errors, fusion degrades.**

| request shape | leg availability | outcome | status | code |
| --- | --- | --- | --- | --- |
| **explicit leg named** — `mode=vector` | vector index not built | `StoreError::VectorIndexUnavailable` (FS-14 — **on a READY store only**; a non-READY engine short-circuits to the FS-8 row below) | **503** | `vector_index_unavailable` |
| **explicit leg named** — `mode=vector` | index built, no embedding provider wired | `StoreError::EmbeddingUnavailable` (FS-13) | **503** | `embedding_unavailable` |
| **fusion** — `mode=hybrid` | vector leg absent (no index) or failing (no provider / embed error) | **degrades that leg to empty**, query succeeds | **200** | — |
| **fusion** — `mode=flat` | lexical leg (a live shard scan) | succeeds; no index dependency | **200** | — |
| **explicit/any** — `mode=graph` | no index dependency; **not** READY-gated (`src/store/mod.rs:4078`; the pre-U3 citation was `:4066`) | succeeds (subject to the walk's own FS-11 `HopLimitExceeded` / FS-12 `CycleDetected`, §5.3) | **200** (else 422/409) | — / `hop_limit_exceeded` / `cycle_detected` |
| any mode | engine not `READY` (except `graph`) | `StoreError::EngineUnavailable` (FS-8) | **503** | `engine_unavailable` |
| any mode | invalid `mode` **option value** | `StoreError::ValidationError` (FS-3) | **400** | `validation_error` |
| **fusion** — `mode=hybrid` | **`hyde: true`** with no/unreachable provider (index built or not) | **degrades the vector leg to empty**, query succeeds (**F3**: FS-13's `hyde: true` clause is narrowing-only — `hyde` names *what* to embed, not a leg to fail on) | **200** | — |
| **explicit leg named** — `mode=vector` | **`hyde: true`**, index built, no/unreachable provider | `StoreError::EmbeddingUnavailable` (FS-13 — the explicit leg fails before the HyDE routing) | **503** | `embedding_unavailable` |
| **no embedding leg** — `mode=flat` (or `graph`) | **`hyde: true`** | **the flag is inert** — `flat_query`/`graph_walk` never touch the embedding provider (`flat_query` calls only the lexical leg, `src/store/mod.rs:4393-4419`; the pre-U3 citation was `:4370-4382`), so there is no provider dependency to fail on and **no** `EmbeddingUnavailable` is ever produced | **200** | — |

- **Precedence (pinned).** The READY gate precedes every leg check for non-`graph` modes
  (`src/store/mod.rs:4078-4080`; the pre-U3 citation was `:4066-4068`): an unbuilt index can only surface as
  `VectorIndexUnavailable` on a **READY** engine — an unbuilt index on a **non-READY** engine (an `Absent`
  boot's `Unavailable`, an `Unreachable` boot's `Degraded`, or the failed-`Reachable`-build branch's
  `Degraded`) surfaces as **FS-8 `EngineUnavailable` → 503 `engine_unavailable` instead**, and the FS-14 row
  above is to be read with that qualifier (***REMAND-2 MUST-FIX 1, 2026-09-22: this qualifier is what the U5
  rows `P-SM-7`/`P-IM-15` and the live battery's `R-L3` (v) assert***). Within `mode=vector` the **index is checked before the
  provider** (`src/store/mod.rs:4448-4454`; the pre-U3 citation was `:4411-4417`), so an unbuilt index wins over an absent provider.
- **The narrowing this ruling pins (F3 — the reconcile is NOT complete without these two clauses).** Both
  the canonical §4.6.1 `ragQuery` **throw column** and **FS-13** state the embedding condition
  **without a mode qualifier**, so the engine-side reading below is a **narrowing of two canonical
  sentences**, not just of FS-14:
  1. **`docs/specs/gnosis.md:872`'s `ragQuery` throw column** — "`EmbeddingUnavailable` if the embedding
     provider is unreachable and a **vector/hybrid/hyde** leg requires it; `VectorIndexUnavailable` if
     the vector index is not built" — is read engine-side as **`mode=vector` only** (explicit leg): the
     `hybrid` and `hyde` clauses of that sentence are **narrowed** by this ruling, and the
     `VectorIndexUnavailable` clause likewise (an unbuilt index is a fusion **degrades-to-empty**, not an
     error). Under §6's **fail-state completeness rule** (`docs/specs/gnosis.md:1052-1055`) an operation
     table **is** an authority table, which is why this sentence must be named, not passed over.
  2. **FS-13's `hyde: true` clause** (`docs/specs/gnosis.md:1037`) — "in `mode: 'vector'`/`'hybrid'`
     **(or with `hyde: true`)** when the embedding provider is unreachable" — is **not** a mode
     condition: `hyde` names **what to embed** (a generated hypothetical document), not a leg to fail
     on. Pinned outcome with `mode=hybrid` + `hyde: true` + `READY` + **no/unreachable provider**: the
     leg **degrades to empty and the query returns 200** — the provider error is absorbed by the fusion
     path (`src/store/mod.rs:4526-4556`; the pre-U3 citation was `:4503-4513`), **exactly as in the unqualified hybrid case**.
     `EmbeddingUnavailable` stays reachable **only** via `mode=vector`, and only when the index exists
     (`:4448-4454`; the pre-U3 citation was `:4411-4417`). A `mode=vector` + `hyde: true` request with a built index and no provider is still
     `EmbeddingUnavailable` → 503 (the explicit leg fails first). **`mode=flat`/`graph` + `hyde: true`
     is inert, not a degraded leg:** those paths never call the embedding provider at all
     (`flat_query` = the lexical leg only, `:4393-4419`; the pre-U3 citation was `:4370-4382`), so there is nothing to degrade and no
     `EmbeddingUnavailable` — the flag simply has no effect, which is the strongest form of "no provider
     error" and is **not** an FS-13 outcome.
  3. **`HyDEGenerationFailed` (FS-18) is untouched by this ruling** and stays unreachable: the
     hypothetical-generation step is total (`generate_hypothetical` never constructs the variant,
     `src/store/mod.rs:5136-5160`; the pre-U3 citation was `:4429-4434`), so a provider failure while embedding the hypothetical surfaces as
     the provider error — which, on the fusion path, the degradation above absorbs.
- **FS-13/14 therefore describe the explicit-leg (and single-leg) case only.** FS-14's
  `mode: 'vector'`/`'hybrid'` wording (`docs/specs/gnosis.md:1038`) is read engine-side as
  **explicit-leg only**; hybrid never fails for an absent/failing vector leg (§4.5.1's graceful
  degradation, `src/store/mod.rs:4526-4556`, `:4493-4560`; the pre-U3 citations were `:4449-4454`, `:4489-4513`).
- **FS-15 is unreachable on the `ragQuery` surface.** The lexical "index" is a live shard scan, so a
  wiki with documents always has a lexical leg; `LexicalIndexUnavailable` is surfaced **only** via the
  `bm25_search` API (reachable there when the queried wiki has no documents,
  `src/store/mod.rs:4270-4272`; the pre-U3 citation was `:4233-4234`).
- **Invalid option values are FS-3, a different condition from an unbuilt index.** `mode`/
  `expand`/`compression` token failures and the numeric range failures are **400 `validation_error`**
  (§5.4/§5.3; for `expand`/`compression` the 400 is **POST-only** — no SSE param exists, N1) — never a
  503 and never a "leg unavailable" code.
- **The trace names the legs.** A hybrid result carries
  `RagTrace::Hybrid(HybridTrace { legs: ["graph","vector","lexical"], … })`
  (`src/store/mod.rs:733-740`, `:4546-4556`). **Note (no invention):** the frozen `HybridTrace` has no
  per-leg availability field, so the legs list is the **fused** leg list and does **not** record which
  leg degraded to empty; a per-leg availability marker would be a change to a frozen §4.5 type and is
  **out of U1's scope** (F2 does not mutate frozen types).
- **Testable assertions the ruling yields** (for U2's/U5's TestWriters, not new U1 rows): (i) an
  explicit-leg request for an unbuilt leg errors with the pinned code; (ii) a fusion request **never**
  fails because one leg is absent; (iii) adding a leg never shrinks the result set; (iv) `mode=flat`
  never fails for want of an index.
- **Interlock with U5.** Once the boot index build lands (U5), the explicit-leg error becomes
  reachable-but-rare — the honest end state. Until U5, `mode=vector` is unusable on a booted server
  (accepted residual risk, gate-1 §Residual risk register).
  ***(U6 addition, 2026-09-22 — ADDITIVE; the U5-time bullet above stands as its record. This marker is the
  one §9.5.6's reconciliation table promised and the pre-remand pass never landed — REMAND-1 anchor/landing
  fix.)*** **The explicit-leg FS-14 reachability set gains a THIRD instance: a `mode=vector` request on a
  READY store whose snapshot carries an index that is STALE (`snapshot().epoch != Store::epoch()`) is
  `Err(StoreError::VectorIndexUnavailable)` ⇒ 503 `vector_index_unavailable`** (§9.5.6's `P-IM-16`/`P-IM-17`,
  its valid/fail state 3). The three instances are (a) no index at all (`vectors: None`), (b) a caller-built
  READY store with no index (the U5-time instance), and (c) the **stale** index — each on a **READY** store,
  with the pre-READY gate (the precedence bullet above) still answering **FS-8** first for any non-READY
  engine, and each rendering through the **existing** FS-14 row: **no new variant, no new code, no new row**.
  **The fusion rows above are unchanged:** `mode=hybrid` with a **stale** index still **degrades the vector leg
  to empty and returns 200** (the same graceful degradation as an absent leg — the refuse is suppressed at the
  fusion site, §9.5.6's per-site consultation clause). **Consequence for the U5-time sentence above:**
  *"Until U5, `mode=vector` is unusable on a booted server"* stands as the **pre-U5 record**; **after U6** the
  honest end state is that `mode=vector` on a booted server is usable **only while the store has not moved since
  the boot build**, so a post-boot mutation makes it loud-unusable (FS-14) until a restart with a reachable
  provider — U6 adds **no** rebuild vehicle (§9.5.6's cost and interaction clauses). **The live home of this
  reachability is the battery's row `R-L4`**, and the supersession of the parked FS-14 mapping's *reason* is
  recorded in that file's §8 item 4 — both are the **supervisor's** same-unit edits (§9.5.6's same-unit table);
  this pass edits neither file.***

---

## 6. The READY lifecycle

The boot wiring is engine-internal; the shell only observes READY.

- **Sequence:** construct `Arc<Store>` → build `DerivedIndexes` → wire the embedding provider →
  transition to `READY`; expose `getEngineStatus` (`READY`/`STARTING`/`DEGRADED`/`UNAVAILABLE` +
  subsystems).
  ***(U5 addition, 2026-09-22 — ADDITIVE; the clause above stands as the pre-U5 record of the sequence.)***
  **The pre-U5 sequence omits the boot index build, so the sequence the post-U5 tree implements is:**
  construct `Arc<Store>` → build `DerivedIndexes` (**the boot snapshot**) → **probe the provider and build
  the boot vector index** (`build_boot_vector_index(&store, provider_ref)`; `Ok(None)` for
  `Absent`/`Unreachable`, `Ok(Some(vi))` **or** `Err(EmbeddingUnavailable)` for `Reachable` — §9.5.5's
  contract table (1)) → **compose the snapshot** (`DerivedIndexes { vectors: built.ok().flatten(),
  ..boot_snapshot }`) → wire the embedding provider (**in the `Reachable` case, and also when a `Reachable`
  build failed** — the probe did succeed) → apply the branch's `EngineState` (`Ready` for a successful
  `Reachable` build; `Degraded` for a failed `Reachable` build or an `Unreachable` probe; `Unavailable` for
  `Absent`) → transition to `READY` (only in the successful-`Reachable` case); expose `getEngineStatus`
  (`READY`/`STARTING`/`DEGRADED`/`UNAVAILABLE` + subsystems). The build sits **between** `DerivedIndexes`
  and the state transition, exactly as §9.5.5's contract table (1) and its ordered clause pin; this bullet
  adds the step and changes **no** part of the historical sequence, the state names or the flags.
  ***(U6 addition, 2026-09-22 — ADDITIVE; the U5 bullet above stands as its record.)*** **The composed
  snapshot also carries the store's epoch** — the composition's terms are
  `DerivedIndexes { vectors: built.ok().flatten(), epoch: <the store's epoch at composition>, ..boot_snapshot }`
  — because U6's `vector` predicate compares `snapshot().epoch` with `Store::epoch()` (§9.5.6's `P-IM-18`,
  and its `P-IM-16` for the predicate). **No sequence step is added or reordered**: the epoch is a **field of
  the composition step**, in **both** branches (a successful `Reachable` build and a failed one), and the
  only behavioural consequence on this sequence is that a freshly booted server's index reads **fresh**
  (`vector:true`; `Ready`) while a store mutated **after** the boot reads `vector:false` and answers FS-14 —
  no new state, no new flag, no reworded `last_error`.
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
full §11 table. **No new §11 row is added** (U1 restates this: the map stays **21 rows**, and there is
specifically no `stale_revision` and no `decode_failed` code — §5.2).

**Which of those variants a `ragQuery` request can actually reach (U1, §5.9).**
`VectorIndexUnavailable`/`EmbeddingUnavailable` are reachable only as **explicit-leg** outcomes
(`mode=vector`) on a `READY` engine; a **fusion** request (`mode=hybrid`, `mode=flat`) degrades a
failing/absent leg to empty instead of erroring, so those codes are **not** reachable from a fusion
request. **This includes the two canonical sentences the ruling narrows (F3, §5.9):** the
`ragQuery` **throw column** in §4.6.1 (`docs/specs/gnosis.md:872` — "…and a vector/hybrid/**hyde** leg
requires it") and **FS-13's `hyde: true` clause** (`:1037`) are read engine-side as
**`mode=vector`-only**; with `mode=hybrid` + `hyde: true` and no/unreachable provider the leg
**degrades to empty** and the query returns **200** (`src/store/mod.rs:4526-4556`; the pre-U3 citation was `:4503-4513`) — so
`EmbeddingUnavailable` is **not** reachable from a hybrid request with `hyde: true` either (and with
`mode=flat`/`graph` + `hyde: true` it is unreachable because those paths never call the provider,
`:4393-4419`; the pre-U3 citation was `:4370-4382`).
`LexicalIndexUnavailable` is **not** reachable on the `ragQuery`/`ragStream` surface at all
(the lexical leg is a live shard scan; the variant surfaces via the `bm25_search` API only). The codes
`RerankerUnavailable` / `HyDEGenerationFailed` / `SubTaskDagFailed` / `CompressionFailed` name components
that are **not wired into the query path today** and are **never constructed anywhere in `src/`**
(a sweep for their construction sites finds only the enum, `wire_code`/`from_wire`, `server_status`, the
`Display` arm, and doc comments) — their §11 rows are unchanged and simply the reserved variants they
always were (`RESERVED-ERRVARIANTS-DISCIPLINE`). `MultiQueryExpansionFailed` **is** reachable from
`ragQuery` with `multiQuery: {enabled: true}` when there is no distinct term to fan out to
(`src/store/mod.rs:5250-5252`; the pre-U3 citation was `:5212-5216`).

### 7.2 NEW-2 — the request-decode outcome (transport-level, NOT a `StoreError`)

A **malformed CRUD request** or an **unknown method** is **NOT a `StoreError` variant** and has
**NO §11 row**. The server renders the request-decode outcome as a **transport-level status**,
**NOT 502** (P1a §6.2):

| `decode_crud_request` error | transport status | transport code (U1, §5.5) | rationale |
| --- | --- | --- | --- |
| `DecodeError::InvalidJson(_)` | **400** | `"invalid_json"` | malformed request body |
| `DecodeError::InvalidEnvelope(_)` | **400** | `"invalid_envelope"` | malformed request body |
| `DecodeError::UnknownMethod(_)` | **422** | `"unknown_method"` | well-formed JSON, unrecognized method (unprocessable) |
| `DecodeError::UnsupportedSchemaVersion(_)` | **400** | `"unsupported_schema_version"` | envelope-level, request-decode path |
| `DecodeError::UnknownIdFormat(_)` | **400** | `"unknown_id_format"` | envelope-level, request-decode path |

**Qualification.** The "no new statuses" claim is qualified to the `StoreError` taxonomy: the
§11 map (21 rows) covers every `StoreError` fail-state and is unchanged. The request-decode
outcome (400/422) is a **transport-level status**, not a `StoreError` wire code, and therefore
does **not** add a §11 row. A malformed CRUD request / unknown method is a **client error
(4xx)**, never a `StoreError`-mapped 502.

**U1 additions to NEW-2.**

- **The body is JSON, not plain text (§5.5).** `decode_error_response`
  (`src/bin/gnosis_server.rs:62-75`, landed post-U2; the pre-U3 citation was `:61-74`, pre-U2 `:57-66`) returns
  `Content-Type: application/json` and
  `{"code":"<transport code>","message":"<detail>"}` — the transport code column above. This is a
  **transport-level** body: the codes are disjoint from the 21 §11 wire codes (`StoreError::from_wire`
  returns `None` for each), so no §11 row is added and no `StoreError` variant is added.
- **One shared rendering.** The same function renders the decode-error body for every route that can
  fail to decode a request — the CRUD handler and the query handler
  (`src/bin/gnosis_server.rs:192-205`, `:207-240`, landed; the pre-U3 citations were `:191-201`, `:206-214`, pre-U2 `:143-153`, `:157-161`). U2 changed
  both together; no existing test pins the pre-U1 plain-text body.
- **The status domain is closed.** `request_decode_status` (`src/server.rs:51-62`) is total over exactly
  the 5 variants above and `None` elsewhere; the other `DecodeError` variants are response/SSE-side and
  are never rendered as a request-decode outcome (the `unwrap_or(400)` fallback is defensive only). The
  transport code mapping (§5.5) MUST have the identical domain.
- **The query surface's code subset.** On `POST /rag/query` the reachable codes are `invalid_json`,
  `invalid_envelope`, plus `unsupported_schema_version` / `unknown_id_format`
  (**U2-time** — the pre-U2 query handler did not check the envelope version/format,
  `src/bin/gnosis_server.rs:157-161` in the **pre-U2 tree**; landed at `:206-214` via the shared decoder).
  **`unknown_method` is never a query-surface outcome** (a
  `ragQuery` payload has no `method` discriminator; §5.3).
- **The mode rule is NOT a NEW-2 outcome.** An unrecognized `mode` is a `StoreError::ValidationError`
  (§11 → 400 `validation_error`, §5.4), never a transport code and never a 422.

---

## 8. The RBAC caller (H3) — re-scoped to the decode layer only

The server threads the `caller` credential through the **request-decode layer only**. The
engine-side RBAC **enforcement** is a PACKAGE item (the engine is the enforcer; P2 does NOT add a
`caller` param to the engine methods). — RECONCILED 2026-09-22 (`RBAC-DOC-DRIFT`): the engine pins the
credential **SHAPE** + its decode-layer **presence check** only; the authority mapping and the deny are
**SHELL**-side — see `GNOSIS-RBAC-EDIT-ENFORCEMENT` / `docs/HANDOFF.md` §8-RBAC. The historical clause
stands as written. The decode layer validates the `caller`'s presence on the
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
implemented here. — RECONCILED 2026-09-22 (`RBAC-DOC-DRIFT`): *"a PACKAGE item, NOT implemented here"* is
the pre-re-scope wording and overstates the engine's residual duty — the engine pins the credential
**SHAPE** + its decode-layer **presence check** only, and the authority mapping and the deny are
**SHELL**-side — see `GNOSIS-RBAC-EDIT-ENFORCEMENT` / `docs/HANDOFF.md` §8-RBAC. The historical clause
stands as written. The server does NOT invent or drop the `caller` (the §5.x `P-SM-3` row).

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

## 9.5 The U2 + U3 typed property registers (authored here — the code-bearing units' §5.x rows)

**Authority.** decision `PBT-GATE-MANDATORY` (`docs/decisions.md`); the gate-1 workstream rows for **U2** and
**U3** (`docs/specs/gnosis-gr-inbound-review.md` §Ordered workstream); the U1 contract pins that name these
rows as owed (`p2` §U1 status block, §11 "Register notes"); the **authoring rule** of the F2 register
(`docs/specs/7-2-wire-property-register.md:21-26`): `Property-id = P-<CLASS>-<N>`, `CLASS ∈ {IM, SM, TP}`,
**≤ 8 rows per unit**, invariants that are **TRUE of the unit's GREEN implementation** (never an acceptance
criterion, never a pending feature), and **no fail-state rows as such** — a fail-state may appear only inside
a *total-function/error-shape* row, where the invariant is "the outcome is `Err(k)` **exactly where** the
contract pins `Err(k)`".

**Where these rows live, and why (explicit).** U2's and U3's rows land **in this file**, in this section —
`§9.5.1` (U2) and `§9.5.2` (U3) — because **§12's ownership list hands exactly these two concerns to these two
units of *this* spec** (§12 item 4 = U2, item 5 = U3/U5). Cross-**file** row namespaces stay per-unit (the p2
register is a distinct namespace from the F2 one: both files legitimately contain a `P-IM-1`, and
`tests/props_gnosis_server.rs` and `tests/props_wire.rs` execute them independently), so **U2's ids continue
the p2 numbering** (`P-IM-4`, `P-SM-4`, `P-TP-2..4`) and **U3's ids continue it again**. `docs/specs/7-2-wire-property-register.md` legitimately keeps its own `P-IM-4`/`P-SM-4`, and
`docs/specs/4-3-facts-property-register.md` its own `P-SM-4`/`P-SM-5`/`P-TP-2` — **cross-file look-alikes are
out of this register's namespace and MUST NOT be touched** (this pass does not edit those files).
**Neither unit's rows are added to `docs/specs/7-2-wire-property-register.md`**: that file
is the **F2** unit's register (its header names `src/wire/` as its scope and `tests/props_wire.rs` as its
executed layer), and adding U3 rows there would silently expand the F2 unit's property layer. The F2 file's
§9.1 register note is **unchanged** by this pass and now **cross-references the U3 rows here** (see §9.5.2);
its existing `P-SM-3` row is **not** modified, superseded or re-scoped.

**Within-file id allocation: ONE sequence for the whole of §9.5 (the id-collision ruling, F1).** Because U2's
and U3's rows share this file **and** the execution plan derives each row's seed tag from its `Property-id`
bytes (a non-injective `id → tag` map would put two rows in one stream and break per-row held/broken
reporting), the **twelve** rows of §9.5 are allocated from a **single per-file sequence**: U2 holds
`P-IM-4/5/6`, `P-SM-4`, `P-TP-2/3/4` (unchanged — they were already the lowest free p2 ids for their classes),
and U3's three IM rows took the three **next free** IM ids after the highest p2 IM id claimed anywhere in this
file — `P-IM-7`, `P-IM-8`, `P-IM-9` (**renumber-with-provenance, §9.5.3.1 REMAND entry**). The three U3 ids this
pass vacated (`P-IM-5`, `P-IM-6` are U2's; the third was never distinct) are **not** reused by U3 for any
other claim, and no id anywhere in this file is reused for a **different** claim. U3's `P-SM-5`/`P-SM-6` and
U2's `P-SM-4` do not collide (distinct numbers) and stay. **The unit remains part of a row's identity in
citation** (`§9.5.1`'s `P-IM-4` and `P-IM-9`'s unit) — but the *ids* are unique across §9.5.
***(U6 addition, 2026-09-22 — ADDITIVE:** the sequence above is unchanged; **U6's six rows (§9.5.6) continue
it at the next free numbers** — `P-IM-16`…`P-IM-19`, `P-SM-8`, `P-TP-6` — so the §9.5 id space now holds
**twenty-four** rows over four units (U2 7 + U3 5 + U5 8 + U6 6), each with its own `U<unit>`-prefixed
injective tag (`U2*`/`U3*`/`U5*`/`U6*`). No earlier id is renumbered or reused, and the unit-prefixed tag
convention (§9.5.3, re-derived for U5) is what keeps the per-row seed streams disjoint across all four
units.**)***

**The register-notes scope note (read this before the tables).** Every row below is a claim about values
**produced by the code paths named in its "code site" column** — e.g. "the status a `Store` reports", not "any
value a caller can force through the test hooks".

**(Two-layer note, 2026-09-16 — annotation only; the rows' claims are unchanged.)** Where a row's invariant
speaks of a wrongly-typed/absent option decoding to "its documented default" (`P-IM-5`) or of the options'
element-wise identity (`P-TP-2`), the **row's observable is the DECODER's output** — layer 1 of §5.3's
two-layer bullet: the store's own `Option` field is **`None`** (`RagQueryOptions.mode == None`, never
`Some(Flat)`; `.top_k == None`, never `Some(10)`), and only a **well-formed vocabulary string** yields
`Some(token)`. The **documented default itself** ("absent ⇒ `Flat`" / `10`) is the **engine's query-time
reading** — layer 2, applied by `rag_query`/`rag_stream` from that `None` (`src/store/mod.rs:4060`, `:4130`,
`:4168`, `:4936`) — so the rows' `== None` assertions and §5.3's/§5.4's "absent ⇒ default" cells hold at
their own layers and do not contradict each other. No row is added, deleted or re-scoped by this note and no
generator domain changes. **(2026-09-16 addendum, annotation only — rulings 1/2:** the `None` at layer 1 is
also the answer for a **present value that is not a `u64`** (`topK:-1`, `topK:10.5` ⇒ `.top_k == None`, **no**
400 — §5.3's type-vs-range bullet, whose **range** half stays the engine's FS-3 400 for a present
**out-of-range `u64`**), and for a **wrongly-typed documented member** of an object-valued option (`n`
included: `{"multiQuery":{"enabled":true,"n":"3"}}` ⇒ `.multi_query == None`, §5.3's member bullet). The two
halves are distinct claims about distinct inputs; neither is a re-scoping of `P-IM-5`.)**

- **U3's flag values have exactly one producer after U3: the status read itself (F16, pinned; LANDED
  2026-09-17).** `EngineSubsystems`
  is **derived inside `get_engine_status`** (`src/store/mod.rs:4193-4232`, the **only** `EngineStatus`
  constructor in `src/`; the pre-U3 citation was `:4181-4195`) from the store's own state — the current derived snapshot, the provider seam, and the
  three always-true capability predicates (§9.5.2's table). U3 therefore **does not** wire the mutators
  (`swap_snapshot`, `set_embedding_provider`) to write a stored flag vector, and the boot
  (`src/bin/gnosis_server.rs:377-415`, landed; pre-U3 `:377-400`, pre-U2 `:321-344`) sets **only** `swap_snapshot` + `set_embedding_provider` +
  `set_engine_state` — it has **no** flag obligation to update. The stored `subsystems: RwLock<EngineSubsystems>`
  field (`src/store/mod.rs:1642` — the pre-U3 citation was `:1638`; **the literal that initializes it is at
  `:1718-1725`**, its preceding U3 comment block occupying `:1714-1717`, while the pre-U3 citation
  `:1710-1717` predates both — see the field-vs-literal note in §5.8) is **not** the value the read returns: it is
  retained only so the existing `Store::set_subsystems` / `Store::set_engine_state` test hooks
  (`:1862-1876`, the `set_engine_state` hook at `:1862-1866` and the write-only `set_subsystems` hook at
  `:1868-1876`; the pre-U3 citation was `:1856-1864`) keep compiling and keep meaning what they mean.
- **What a generator may therefore sample.** The honest state vector is the **store's own state** —
  `Store::new()`'s construction state, the snapshot (`snapshot().vectors.is_some()`, `swap_snapshot`), and the
  provider seam (`embedding_provider().is_some()`, `set_embedding_provider`) — plus `set_engine_state` for the
  `state` axis. A generator MUST NOT present a deliberately-injected **false mask** (e.g. `set_subsystems` with
  the pre-U1 all-`true` literal) as evidence **against** `P-IM-7`/`P-IM-8`/`P-IM-9`: after U3 that write changes
  nothing observable, and a row asserted through a verbatim-writing hook would prove the hook, not the
  derivation (exactly the F11 defect this pass removes). This keeps the two existing integration assertions
  green (`tests/rag_query_integration.rs:1645-1683` injects a mask and asserts only the **core** trio
  `store`/`graph`/`lexical`, so it neither contradicts nor pins the non-core flags).

**Rows the earlier units already landed in this file (unchanged).** `P-IM-1`/`P-IM-2`/`P-IM-3` /
`P-SM-1`/`P-SM-2`/`P-SM-3` / `P-TP-1` (§11) are the **P2** unit's rows and are **not** touched, re-scoped or
renumbered by this pass. Nothing in U2/U3 weakens them: `P-IM-3` stays the **growth invariant** (no fixed
count; the current table is still 14 rows), and `P-IM-2`/`P-SM-2`'s 5-variant request-decode domain is
**unchanged** (U2 adds a **code** mapping beside `request_decode_status`, not a status or a variant).

### 9.5.1 U2 — the query POST contract (7 rows ≤ 8)

**Scope of the unit.** The shared `ragQuery` payload decoder with the canonical §4.5.2 `filters` mapping
(**not** a `serde_json::Value`-shaped pass-through into `QueryAuditFilters`); envelope strictness on
`POST /rag/query` (a bare body stays rejected); the **single** mode/`expand`/`compression` token resolver
shared by POST and SSE (`expand`/`compression` are POST-payload-only); the structured transport decode-error
body (JSON, `Content-Type: application/json`, the five `DecodeError`-derived codes with their 400/422
statuses, never a §11 row, never `text/plain`), rendered by the **one** shared function both the CRUD and the
query handler call; and the rule that the encoder never emits a body the result validator rejects.

**The observable surface (named so the TestWriter has a target, not a guess).** The decoder, the token
resolver(s), the transport-code mapping and the checked encoder are **pure synchronous fns in the engine lib**
(the precedent is §11's "API notes for the TestWriter": `server_status`/`request_decode_status`/
`route_bijection` are lib fns re-exported from `src/lib.rs`). The **names are U2's**; the **behavior** is
pinned here and in §5.3–§5.5. The canonical shapes the rows below are written against:

| lib surface (name is U2's) | signature | contract |
| --- | --- | --- |
| the shared query decoder | `decode_query_request(path: QueryPath, env: &Envelope, sse_params: SseParams) -> Result<(String, RagQueryOptions), QueryDecodeError>` where `QueryPath ∈ {Post, Sse}` and `SseParams` is the SSE query-param carrier (**F6** — see the signature note below) | §5.3 (envelope-strict, camelCase payload, the 14-key table, wrongly-typed ⇒ absent), §5.5 (transport codes) |
| the decode-level error type | `QueryDecodeError::{ Transport(DecodeError), Validation(StoreError) }` | a **transport** variant is rendered by the **shared** decode-error renderer → 400/422 + its transport code; a **validation** variant is the §11-mapped `ValidationError` → 400 `validation_error`. The token resolver reports through the `Validation` arm — which is what makes the mode/`expand`/`compression` failure **400 `validation_error`** and **never a decode-level 422** (§5.4) |
| the mode resolver | `resolve_query_mode(raw: Option<&str>) -> Result<QueryMode, StoreError>` | §5.4's table (the name is the one §5.4 already suggests) |
| the `expand`/`compression` resolvers | one per token family, `Option<&str>` → `Result<ExpandMode\|CompressionMode, StoreError>` | §5.4; **POST-payload callers only** |
| the transport code mapping | `request_decode_code(e: &DecodeError) -> Option<&'static str>` | §5.5's 5-row table; the **same domain** as `request_decode_status` (`src/server.rs:51-62`) |
| the checked result encoder | `encode_result_checked(res: &RagResult) -> Result<Envelope, StoreError>` | F2 §4.5's encoder/validator rule: a `validate_rag_result`-rejected result ⇒ `Err(StoreError::EngineError)` (FS-9 ⇒ 502), never a 200 carrying an un-decodable body |

**The signature note (F6 — pinned so the SSE half of the token rule is expressible, not implied).** The token
that has **both** a POST and an SSE reading is `mode`, and on the SSE surface its input is the **query param**
`?mode=`, which is **not** part of `Envelope` (`Envelope` is `{schemaVersion, idFormat, payload}`,
`src/wire/envelope.rs:12-18`). A decoder that received only `(path, &Envelope)` therefore **could not**
distinguish "param absent" from "param present but unrecognized" — the very distinction `P-SM-4` (the mode
rule) and `P-IM-4` (the transport/envelope dichotomy) quantify over — so U2's decoder **takes the SSE params
as a third input**:

- **`SseParams` (P = F6's ruling).** The SSE half is carried as the param list, one `Option<&str>` per token the
  pre-U4 SSE surface reads: `SseParams { query: Option<&str>, top_k: Option<&str>, mode: Option<&str> }`
  (equivalently `&[(String, String)]` of the parsed query params, in which case "absent" is simply "no `mode`
  entry"). **Which argument carries `mode` on the SSE path is pinned: the `sse_params.mode` field** (the SSE
  param value, `None` when the param is absent), and **`mode` on the POST path is read from `env.payload["mode"]`**
  — the decoder MUST NOT merge the two inputs (a POST body that carried no `mode` key MUST NOT be defaulted from
  an SSE param list and vice versa). On `QueryPath::Post` the `sse_params` argument is **unread** (the POST path
  has no query params); on `QueryPath::Sse` the `env` argument is the **synthesized canonical envelope** the SSE
  handler builds (`{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{}}`, §5.4 — the handler has no
  request body) and the payload is thereby empty, so the SSE path's only inputs are the three params.
- **`top_k` is a string on the SSE path** (pre-U2 the handler parsed it tolerantly,
  `params.get("topK").and_then(|v| v.parse().ok())`, `src/bin/gnosis_server.rs:194`; U2 moved the parse into
  this decoder, `sse_params.top_k.and_then(|s| s.parse::<u64>().ok())`, `src/wire/query.rs:157`): the tolerant
  parse is **unchanged by U1/U2** (§5.4's `topK`-unchanged note) and
  is expressed as `Option<&str>` here, so "absent" and "unparseable" collapse to the same default `None` ⇒ `10`
  — a pinned boundary, not an omission.
- **A split entry point is the accepted alternative** (`decode_query_post(env)` + `decode_query_sse(params)`),
  provided **both** funnel into the one shared `decode_query_request` body above and the `mode` resolution goes
  through the one `resolve_query_mode`. Either spelling satisfies this note; the obligation is that the SSE token
  input is **a decoder argument**, never a value the handler pre-resolves.

| Property-id | Class | Invariant | Strategy-id | Observable-as-property | Code site / contract clause |
|---|---|---|---|---|---|
| `P-IM-4` | IM | **The query decoder is envelope-strict and fail-closed, and its `(query, options)` mapping is element-wise — stated over two DISJOINT outcome sets (F3/F9).** **(A) Envelope/transport failures ⇒ `Err`:** `schemaVersion` is checked **first** (unknown ⇒ `UnsupportedSchemaVersion`), `idFormat` **second** (unknown ⇒ `UnknownIdFormat`), and a **non-object `payload`** ⇒ transport failure (F9: **`InvalidEnvelope` — exactly one code**, the other staying unreachable on this path). **(B) Every other input ⇒ `Ok((query, options))`**, mapped element-wise per §5.3's key→field table (unknown keys ignored and changing nothing); no third success path, no partial decode, no silently dropped key. A **bare** (non-envelope) body is always set (A). This row owns **only** the envelope dichotomy and the key→field mapping — the per-key wrong-type/default rules are `P-IM-5`'s, the `filters` mapping is `P-IM-6`'s and the token rule is `P-SM-4`'s; "well-formed" is **not** this row's predicate (F3). | `strat:query-decode-total` | ∀ payload `p` (a JSON object): `decode_query_request(Post, &envelope(1, "opaque-string-v1", p), SseParams::default())` is `Ok((q, o))`, with `q` and every `o.<field>` `==` §5.3's mapped value (`topK:42` ⇒ `top_k == Some(42)`; `query` verbatim; `filters` per `P-IM-6`); ∀ `v != 1`: `Err(Transport(UnsupportedSchemaVersion(v)))`; ∀ `f != "opaque-string-v1"`: `Err(Transport(UnknownIdFormat(f)))`; ∀ **non-object** `payload` (`5`, `null`, `"hi"`, `[]`, `true`): `Err(Transport(InvalidEnvelope(_)))` — **never `InvalidJson` on this path**, and the envelope body's own non-object case is likewise `InvalidEnvelope` (`src/wire/envelope.rs:44-46`); ∀ bare body: `Err`; every `Err` is `Transport(e)` with `request_decode_status(&e)` `Some(status)` (never `None`), so the rendering is always 400/422 (`unknown_method` being CRUD-only on this path, §5.3/§7.2); precedence: `schemaVersion` before `idFormat` (an envelope with both unknown ⇒ `UnsupportedSchemaVersion`). | `src/wire/` U2's shared query decoder (the module file is U2's to name; §5.3); `src/wire/crud.rs:305-310` (the precedence precedent); `src/wire/envelope.rs:41-74` (`from_json` — the object check U2's payload check mirrors); §5.3's request table; §5.5 |
| `P-IM-5` | IM | **A wrongly-typed optional value decodes as ABSENT ⇒ its documented default, and the per-key ruling is the 14-key table of §5.3 (F4/F7).** The rule's quantified domain is **§5.3's per-key wrongly-typed table** (14 rows: every documented key, each with its pinned wrong-type outcome) — this row **cites that table and enumerates no exceptions in prose** (the pre-remand "other than `query` and `filters`" phrasing is superseded: `mode` is **inside** this row's domain for **non-string** values, and only a **string** reaches `P-SM-4`'s resolver). §5.3's table pins the two outcomes: **"wrongly-typed ⇒ absent/default"** for the 12 non-schema keys, **"wrongly-typed ⇒ `Err(Validation(_))`"** for `query` (**at the decoder for a present non-string**; **`""` ⇒ FS-3** for an absent one — §5.3's `query` cell) and `filters` (schema-checked, `P-IM-6`). A well-typed value decodes to `Some(mapped)`; `{"multiQuery":{"enabled":true}}` (object, `n` omitted) ⇒ `Some(MultiQueryOptions{ enabled: true, n: 3 })` (§5.3's pinned `n` default). **The two OBJECT-VALUED keys are ruled at the FIELD level, and their member-level defaults apply only inside a well-formed object (pin clarification, blind-greens `U2-13`; §5.3's two-layer bullet, object-valued-option clause + the two object cells of §5.3's per-key table):** `{"multiQuery":{"enabled":"yes"}}`, `{"multiQuery":{"enabled":true,"n":"3"}}` and `{"subTaskDag":{"enabled":"yes"}}` ⇒ the decoded field is **`None`** (`multi_query == None` / `sub_task_dag == None`) — a present object with a wrongly-typed **documented member** is **not** a `Some` carrying a defaulted member, and **the member-type check is total over the members each option declares** (`enabled` ⇒ a **bool**, `n` ⇒ a **u64**, per §5.3's member bullet; *ruling 2, 2026-09-16: the clause stands and is implemented, defect `P-6` closed — a present non-u64 `n` is a misuse, not a defaulted member*). **This row asserts nothing about what the engine does with a decoded option** (N4: acceptance, not consumption — §5.3's reachability note), and nothing about inputs that are in-range/token-valid. | `strat:query-option-defaults` | **Domain (F7, bounded):** `query` a well-formed **non-empty** string; every option with a documented **range** fail-state (`topK`, `maxHops`, `filters`, `multiQuery.n`, `binaryCandidatePool` — §5.3's fail-state table) either **in range** or pinned to `None`/absent; no unrecognized token string. Inside that domain: ∀ `(k, v)` from the wrongly-typed corpus ∪ an absent-`k` control ∪ a well-typed control, **on each path that reads `k`** (POST reads the payload's 14 keys; the SSE path reads `query`/`topK`/`mode` through `sse_params` — the seam of the §9.5.1 signature note — and on the SSE path every key is a **string or absent**, so its wrong-typed reading is the `Option<&str>` `None`): the decoded `RagQueryOptions.<field> == None`, the decode is `Ok`, and **no `Err` is produced for these inputs** (a type slip is never a 400/422) — the row's excluded fail-states (an empty/absent `query` ⇒ 400 `validation_error`; an out-of-range `topK`/`maxHops`/`binaryCandidatePool`/`multiQuery.n` ⇒ 400; an unrecognized token string ⇒ 400; a malformed `filters` ⇒ 400) are **not** generated here, and a generator that varies them is outside this row (each is pinned by `P-IM-4`/`P-IM-6`/`P-SM-4` and §5.3's fail-state table). Well-typed controls ⇒ `Some(the mapped value)` (`wikiId:"w1"` ⇒ `wiki_id == Some(WikiId("w1"))`; `topK:42` ⇒ `top_k == Some(42)`); `{"multiQuery":5}` ⇒ `multi_query == None`; `{"multiQuery":{"enabled":true}}` ⇒ `Some(MultiQueryOptions{ enabled: true, n: 3 })`; `expand`/`compression` are exercised **on POST only** (no SSE param exists — N1). | `src/bin/gnosis_server.rs:194` (the tolerant-`topK` precedent the rule generalizes); **§5.3's per-key wrongly-typed table (the canonical 14-key ruling)**; §5.4's `mode` row; §9.5.1's decoder-signature note (the SSE inputs) |
| `P-IM-6` | IM | **`filters` is mapped member-by-member from the canonical §4.5.2 tokens onto the frozen store types — never serde-deserialized into `QueryAuditFilters`, never silently all-`None`, and total over its member space (F5).** The **casing policy is pinned (F5)**: the three token members `nodeKind`/`edgeType`/`state` are matched **ASCII-case-insensitively**, and `state` **accepts the store's own `"Fresh"`-style casing as a casing variant of `FRESH`** (§5.3's token table) — while the **member names themselves are exact** (`nodeKind`, `edgeType`, `target`, `state`; unknown members are ignored, see below). A **string** `nodeKind` outside `content/fact/reference`, a **string** `edgeType` outside `link/embed/crosslink`, or a **string** `state` outside `FRESH/RESOLVED/STALE/BROKEN` (any ASCII casing) ⇒ `Err(Validation(ValidationError(_)))`; a **wrongly-typed member** (`{"nodeKind":5}`, `{"state":5}`, `{"edgeType":[]}`, `{"target":"d1"}`) ⇒ `Err(Validation(_))`; a `target` that is not an object **with both** `documentId` and `nodeId` present as strings ⇒ `Err(Validation(_))`; a well-formed subset ⇒ `Ok(Some(filters))` with **exactly** the named members mapped; absent or `null` ⇒ `Ok(None)`. The mapping is **explicit** — not the store type's serde binding, which could not bind the camelCase keys at all (`QueryAuditFilters`, `src/store/mod.rs:471-477`, has no `rename_all` and no `deny_unknown_fields`). | `strat:filters-mapping` | ∀ well-formed subset: `options.filters == Some(QueryAuditFilters{ node_kind, edge_type, target: Option<(DocumentId, NodeId)>, state })` — **exact** field equality, so a camelCase-keyed payload can never yield `Some(all-None)`; ∀ out-of-set token (incl. `nodeKind:"community"`, `edgeType:"docHead"`) ⇒ `Err(Validation(_))`; ∀ **wrongly-typed member** (`nodeKind`/`edgeType`/`state`/`target`) ⇒ `Err(Validation(_))` (**never** "absent", unlike the scalar options); ∀ `target` **missing either member**, or with a **non-string** member ⇒ `Err(Validation(_))`; ∀ `target` **with an extra member** ⇒ `Ok` with the two named members mapped and the extra one **ignored** (the tuple has no third slot); ∀ **unknown member** (`{"nodekind":"content"}`, `{"nodeKind":"content","extra":1}`, `{"target":{"documentId":"d1","nodeId":"n1","x":1}}`) ⇒ `Ok` (tolerated, contributes no option) — so a payload whose **only** members are unknown ⇒ exactly `Some(all-None)`, which the F1 negative property MUST NOT be read against (it quantifies over payloads naming **recognized** members); `{"filters":null}` ⇒ `None`; `{"filters":{}}` ⇒ `Some(all-None)` (**present/valid/honored** — the caller's identity filter, §5.3 N5); ∀ casing variant `v` of `FRESH`/`RESOLVED`/`STALE`/`BROKEN` (incl. the store's `"Fresh"`) ⇒ the same mapped `ReferenceState` as the canonical uppercase token; `{"state":"fresh"}` ⇒ `Fresh`, `{"nodeKind":"Content"}` ⇒ `Content` (ASCII-case-insensitive); **the whitespace boundary is pinned only for `mode` (§5.4's no-trim rule) — this row does NOT extend it to the `filters` tokens**, so a padded `{"nodeKind":"content "}` is asserted by **no** row here (a generator MUST NOT invent that state; if U2 needs it pinned it is a §5.3 amendment, not a register claim). **The store's serde casing is not the wire casing:** `{"state":"Fresh"}` is accepted as a *casing variant of the pinned token*, while the **serde shape** of `QueryAuditFilters` (snake_case member names, the 2-element `target` array) is **not** accepted at all — a payload spelled in the store's own serde shape (`{"node_kind":"Content","target":["d1","n1"]}`) is an object with **zero recognized members** ⇒ `Some(all-None)`, the GR-2 defect instance the negative property targets. | `src/store/mod.rs:471-477` (the frozen filter type); §5.3's `filters` mapping table (the canonical pin, `docs/specs/gnosis.md:692-694`); §5.9's `edgeType` range check (`src/store/mod.rs:4311-4321`) |
| `P-SM-4` | SM | **The enum-token rule is total, case-insensitive, path-aware, single-sourced — and its single-sourcing is asserted at the WIRING layer, not only as a property of the function (F15).** For any **string** token on a path that **reads** that token: an ASCII-casing variant of a vocabulary member (`flat/graph/vector/hybrid`; `none/parent`; `none/filter/extract/graph`) resolves to the typed enum, and any other string (incl. `""`, whitespace, `"bm25"`, `"flat "`) yields `Err(ValidationError(_))` — there is **no** third outcome (never a silent default, never a transport code, never a 422). For a token value that is **not** a string the resolver is **not invoked**: the value decodes as absent ⇒ the option's default (`mode` ⇒ `Flat`; the other two ⇒ `'none'`). The rule is **single-sourced**: one resolver, `resolve_query_mode(raw: Option<&str>) -> Result<QueryMode, StoreError>` (§9.5.1's surface table), which the **POST decoder** calls with `env.payload["mode"].as_str()` and the **SSE decoder** calls with `sse_params.mode` (§9.5.1's F6 signature note); `expand`/`compression` have **no** SSE input at all (their params do not exist on `/rag/stream`), so the rows quantifying over them are POST-scoped by construction. **Function-level agreement is not the pin** — a hand-rolled `match m.as_str()` inside the SSE handler would leave the function property green while the two paths diverge, so this row additionally carries the three **named wiring assertions** below. | `strat:token-resolver` | ∀ token string `s` (incl. all ASCII casings of each member) on a reading path: `Ok(enum)` iff `s` is a member, else `Err(Validation(ValidationError(_)))`; ∀ non-string token value: `Ok` + the option's default; ∀ `mode` input `m`: `decode_query_request(Post, …)` and `decode_query_request(Sse, …)` (the F6 seam) yield the **same** classification of `m` (same `Ok(mode)` / same `Err` family); and for **every** U2 path, `expand`/`compression` are reachable only from a POST payload. **Named wiring assertions (live-transport; F15):** (i) `GET /rag/stream?query=x&mode=bm25` ⇒ **HTTP 400** with the single `error` frame body (`code:"validation_error"`) then close — a `200` with an `error` frame, or a `200` `result` frame, falsifies the row on the SSE path; (ii) `GET /rag/stream?query=x&mode=HYBRID` ⇒ **status other than 400** (503 `engine_unavailable` on today's not-READY engine; 200 on a READY one) — proving the handler routed the param through the shared resolver rather than a local `match`; (iii) `GET /rag/stream?query=x&expand=parent&compression=nope` ⇒ **not** 400 (the params are not read — N1). (i)–(iii) are the register's live obligations on the SSE handler; they are asserted by the e2e/conformance layer named in §9.5.3. | §5.4 (the single rule + its POST/SSE split; the `mode` table; the empty/whitespace boundary states); F2 §4.5's wire-encoding table; `src/store/mod.rs:639-667` (the typed enums — why no token can reach `validate_rag_options`) |
| `P-TP-2` | TP | **The decoder's output IS the wire options, element-wise — stated purely as the DECODE-level identity (F10), with the handler pass-through named as a live obligation in this same row.** For **any** well-formed request, the decoder's output is a single value pair `(query, RagQueryOptions)` that (a) carries the caller's `query` string verbatim, (b) has **no** option silently reset: a key the payload names well-formedly is `Some(mapped)` (never `None`), and a key the payload omits is `None`, and (c) is **element-wise equal** to the `RagQueryOptions` value the contract names for that payload. The `requester` key is **never** part of it (unknown key ⇒ `requester == None`). **The handler obligation this row does NOT cover is named here rather than disclaimed (F10 + adjudication 1):** "the options handed to the engine == the options the payload names" is identity-by-construction at lib level and observable **only** across handler→engine, so it is pinned as a **required live assertion** with its observable named — (α) **negative probe, live e2e against today's not-READY server:** `POST /rag/query` with `{"query":"x","mode":"bm25"}` ⇒ **400 `validation_error`** (the handler read the *wire* value through the shared decoder; the pre-U1 handler ignored `mode` and would have returned 503), asserted by a **new test in `tests/gnosis_server_e2e.rs`** (named `rag_query_unrecognized_mode_is_400_e2e`, modelled on the existing `rag_query_not_ready_returns_503`); (β) **positive observable, live-scenario battery (`docs/specs/p2-gnosis-server-live-pending-battery.md`), precondition a READY boot with a reachable provider:** `POST` with `{"topK":1,"filters":{"nodeKind":"fact"}}` and `{"topK":50}` return **200** `RagResult`s whose **`results` array** carries **≤ 1** / **≤ 50** entries (`RagResult.results`, `src/store/mod.rs:758-765` — the frozen wire/type field is **`results`**, not `items`) and whose `results` match only `fact` nodes — i.e. the wire `topK`/`filters` **visibly changed the response** on the engine's own audit surface | `strat:query-options-identity` | ∀ well-formed `(query, payload)`: `let (q, o) = decode_query_request(Post, &env, SseParams::default())`; `q == payload["query"]`; ∀ documented key `k` present well-formedly: `o.<field(k)>.is_some()` and `== Some(expected)`; ∀ documented key absent: `o.<field(k)>.is_none()`; `o.requester.is_none()` in every case. **Honesty qualifier (N4, §5.3):** this half asserts the options **value**, never that a leg **consumed** an option (`subTaskDag` has no read site; `binaryFirstPass`/`binaryCandidatePool` are vector-leg-only). **The bin half is (α)+(β) above**, not a disclaimer. | §5.3's payload table, the `multiQuery.n = 3` default and the per-option reachability note; `src/store/mod.rs:638-667`; `tests/gnosis_server_e2e.rs` (`rag_query_not_ready_returns_503` as the (α) model) |
| `P-TP-3` | TP | **The transport decode-code mapping is total over the request-decode domain, status-consistent, and disjoint from §11 — the PURE half only (F8).** For every request-decode `DecodeError` variant there is exactly one transport code and it is defined **exactly where** `request_decode_status` is defined (same domain, `None` outside it); each code is non-empty; its status is the `request_decode_status` status (400 for the four, 422 for `UnknownMethod`) and is **never 502**; and every code is **absent** from the 21-row §11 map (`StoreError::from_wire(code, Some(msg)) == None`). A `StoreError` keeps its own §11 `wire_code()` — the two vocabularies never mix. **The RENDERED-BODY half is NOT this row (F8):** the body bytes/`Content-Type` are produced by the bin-private `decode_error_response` (`src/bin/gnosis_server.rs:62-75`, landed post-U2; the pre-U3 citation was `:61-74`; the pre-U2 lines were `:57-66`), which is **unreachable from `tests/`** (`[[bin]]`), so it is a **conformance/e2e obligation** — see the named golden in the `strat:` cell. | `strat:transport-code-total` | ∀ `e: DecodeError` in the 5-variant corpus: `request_decode_code(&e)` is `Some(c)`, `c` is non-empty, `request_decode_status(&e)` is `Some(st)` with `st ∈ {400, 422}`, and `StoreError::from_wire(c, Some("m")) == None`; ∀ `e` outside the 5 (`UnknownType`/`MissingTrace`/`UnknownCode`/`EventTypeMismatch`/`ValidationFailed`): `request_decode_code(&e) == None` **and** `request_decode_status(&e) == None`; ∀ `StoreError`: `wire_code()` is **not** in the transport code set. **Path note:** on the query surface the reachable subset is the four 400 codes only — `unknown_method` is CRUD-only (§5.3/§7.2), so its row is exercised on the CRUD path. **The rendered-body half, moved and NAMED (F8):** the golden is F2 §12's **V-15** (two byte-exact bodies: `unknown_method` ⇒ 422, `unsupported_schema_version` ⇒ 400) plus its **new sibling V-15.1** — the **query-path** body `{"code":"invalid_envelope","message":"<verbatim>"}` for a **non-object payload** (`P-IM-4`'s case (A) instance, U2-time) — and both are created/asserted by the conformance test **`v15_request_decode_error_body_exact`** in `tests/wire_conformance.rs` (the V-n naming convention of `v8_health_reports_exact`/`v9_decode_then_validate_routes_exact`), driven at the transport by the e2e assertion **`malformed_request_returns_400`**-style `tests/gnosis_server_e2e.rs` cases. **Message contract (restated per F8):** the four string-carrying variants carry their string **verbatim and it MAY be empty** (`InvalidJson(m)`/`InvalidEnvelope(m)`/`UnknownIdFormat(m)` carry it as-is), while `UnsupportedSchemaVersion(v)` ⇒ **non-empty** (`"unsupported schemaVersion: {v}"`) — the pre-remand "both keys non-empty" clause is superseded. | §5.5's code table + its status/disjointness rules; §7.2; `src/server.rs:51-62` (`request_decode_status`); `src/wire/error.rs` (`from_wire`); F2 §7.1 + F2 §12's **V-15/V-15.1**; `tests/wire_conformance.rs` |
| `P-TP-4` | TP | **The encoder never emits a body the result validator rejects (the transport rendering of it is total).** For **any** well-formed `RagResult` — F2's `P-TP-1` corpus: `engine == "gnosis"`, trace present and mode-consistent, `blocked_by` only with a `RagTrace::Graph` — the checked encoder returns the envelope whose payload is accepted by `decode_rag_result` **and** passes `validate_rag_result`; conversely, for any result the validator **rejects**, the checked encoder returns `Err(StoreError::EngineError)` and its §11 status is **502** (FS-9) — a rejected body is never rendered as a 200. | `strat:encode-result-checked` | ∀ well-formed `r`: `let env = encode_result_checked(&r)` is `Ok(env)` and `validate_rag_result(&decode_rag_result(&env.payload)?) == Ok(())`; ∀ rejected `r` (**exactly the two representable classes**: a non-`"gnosis"` `engine`, and a `blocked_by` present without a graph trace — a **traceless** result is **not representable** (`RagResult.trace` is not an `Option`) and is therefore **not** a rejected class here, §9.5.4): `encode_result_checked(&r) == Err(StoreError::EngineError)` and `server_status(&EngineError) == Some((502, "engine_error"))`. **Reachability (pinned, no invention):** with today's engine every `rag_query` result validates (`src/store/mod.rs:4413-4419` — the flat trace construction; the pre-U3 citations `:4124`/`:4138-4145` are stale), so the `Err` half is a **U2-time totality requirement** tested at the pure level — **not** a live HTTP fail-state (§5.3, F2 §4.5). | §5.3's "encoder must never emit a body the result validator rejects"; F2 §4.5's encoder/validator rule + §7's `validate_rag_result`; `src/store/mod.rs:4413-4419`, `:4477-4482` |

**Class tally (U2):** IM ×3 (`P-IM-4`, `P-IM-5`, `P-IM-6`), SM ×1 (`P-SM-4`), TP ×3 (`P-TP-2`, `P-TP-3`,
`P-TP-4`) = **7 rows ≤ 8** ✔.

**Consolidation record (what was merged, and why — the cap is hard).** §11's owed table carries **nine**
obligations for U2 — **eight** named in the register-notes sketches at U1 **plus the one row the first register
remand added** (the `QueryAuditFilters`-touching mapping, which is the §11 row annotated "merged into `P-IM-6`"),
so **the §11 UPDATE sentence and this record count the same nine** (§9.5.3.2 REMAND-2's item 6 corrects the
pre-remand "six"/"eight" mismatch). **The composition of the nine is fixed and identical in all three accounts
(this record, §11's UPDATE sentence, §9.5.3.2 item 6): the eight U1 register-notes sketches = the shared
envelope-strict query decoder, the wrongly-typed-⇒-absent rule, the `filters` canonical mapping, the mode rule,
the token-resolver totality, the SSE/POST parity, the transport decode-code mapping and the query-POST
options pass-through; the ninth = the row the first register remand added (the `QueryAuditFilters`-touching
mapping merge).** The pre-remand sentence that counted these as **"six"** **omitted three** of the nine — the
**`P-IM-5` wrongly-typed** row, the **`P-TP-2` split** row and the **`P-SM-4` parity** row — and the ninth is
**not** the `P-IM-5` wrongly-typed row (that row is one of the three the pre-remand count omitted, not the row
the remand added); §11's UPDATE sentence and §9.5.3.2 item 6 carry this same naming, so no account of the nine
is left calling a different row "the ninth". The ≤8-row cap was honoured by **merging three pairs into one row each** rather
than
dropping any: §11's *"shared query decoder is envelope-strict and tolerant of unknown payload keys"* + its
*"wrongly-typed optional value decodes as ABSENT"* sketch → **`P-IM-4`** (envelope/`fail-closed`) + **`P-IM-5`**
(the default rule) — kept separate because their quantified domains differ (whole-envelope vs per-key);
§11's *"`filters` decoding is the canonical mapping"* + *"the `filters` mapping is the only decoder path that
touches `QueryAuditFilters`"* + *"`filters: {}` ⇒ `Some(all-None)`"* → **`P-IM-6`** (one row: a positive
mapping claim **plus** its negative form — a literal serde pass-through is refuted by the same exact-equality
observable); §11's *"the mode rule is total, single-sourced and case-insensitive"* + *"the token resolver is
total over all three token families"* + *"SSE/POST parity includes the fail-state status"* → **`P-SM-4`** (one
row: one rule, three vocabularies, the path-aware domain). §11's *"the query POST path honors the options
surface"* lands as **`P-TP-2`**'s identity half, with its bin-level half **named as the (α) e2e probe and the
(β) live-battery observable in that same row** (F10 + adjudication 1). **Nothing was left out**: the mapping
from §11's owed rows to these ids is complete and is stated row-by-row in §11's updated table.

**"TRUE of the unit's GREEN implementation" — checked, not assumed.** Each row is an invariant of the
**post-U2** tree, which is what the row language ("decodes as absent", "⇒ `Err`") asserts. Where U2 **closes**
a pre-U2 divergence the row was therefore **red before U2 landed and is green after** — that is the intended
red→green shape, not a contradiction. **The U2 code has since landed (2026-09-17), so the "broken against
today's code" half of this paragraph is historical:** pre-U2, `mode` was matched **case-sensitively** and
silently coerced to `None`
(⇒ `Flat`) on the SSE path (`src/bin/gnosis_server.rs:195-201`, pre-U2) and **ignored entirely** on the POST
path (`:162-168`, pre-U2), so `P-SM-4` was broken against that code and is held against U2's (including its
(i)/(ii)
wiring assertions); the same held for
`P-IM-4`'s envelope checks on that path (`:157-161`, pre-U2 — it did not check `schemaVersion`/`idFormat`) and for
`P-IM-6` (the payload was never mapped to `filters` at all). Rows `P-TP-3`/`P-TP-4` partly held pre-U2
(the transport **status** mapping and F2's encoder-validator totality existed; only the **code** fn and the
checked-encoder wrapper were new; `P-TP-3`'s `InvalidEnvelope`-only non-object outcome was U2-time, since pre-U2
that input returned `400 validation_error` with no transport code at all — §5.3's fail-state table). Recorded
here so the PBT audit reads a **U2-relative** register and not "already-green" rows; **all seven U2 rows are
HELD against the landed tree** (§U1's U2-landed bullet).

**Adjudication notes (things the frozen contract could not be turned into a faithful pure row — recorded,
not invented).**

1. **The handler's pass-through is not purely observable (bin-only code) — NAMED as an obligation, not
   disclaimed (F10 + adjudication 1).** §5.3 pins that the decoded `RagQueryOptions` is **passed to**
   `rag_query`/`rag_stream` (§5.3's pre-U1-state sentence names the handler,
   `src/bin/gnosis_server.rs:162-168`), but the handler lives in a `[[bin]]` target (`Cargo.toml`'s
   `[[bin]]`; `src/bin/gnosis_server.rs`) which is **not reachable from an integration test** — `tests/` sees
   the lib only (`src/lib.rs`). The faithful formulation is therefore split **with both halves homed**:
   **`P-TP-2`** pins the **decode-level** identity (pure, testable) **and** carries the handler obligation as
   the **named live assertions** (α) — the new e2e test `rag_query_unrecognized_mode_is_400_e2e` in
   `tests/gnosis_server_e2e.rs`, which proves the handler read the wire value on today's not-READY server —
   and (β) — the live-scenario battery's `topK`/`filters`-changed-the-response observable on a READY boot
   (`docs/specs/p2-gnosis-server-live-pending-battery.md`, precondition: a reachable provider). Exactly §11's
   API-notes split between "pure/synchronous surface" and "live-transport surface". *No contract change is
   requested and no obligation is left without a layer* — this is a test-surface boundary with a named home.
2. **§5.5's "one shared rendering" is a CRUD+query bin-level claim — ACCEPTED, with F8's two conditions
   folded in.** The **code** mapping (`request_decode_code`) is pure and is pinned by `P-TP-3`; the
   **byte-exact** body/`Content-Type` claim (§5.5, F15: `application/json` with no `charset`) is a
   **rendering** obligation in `src/bin/gnosis_server.rs:62-75` (landed; pre-U2 `:57-66`; **the `:61-74`
   spelling this bullet carried is stale — the U3 landing shifted the fn to `:62-75`**) and is asserted by the e2e/golden layer **F2
   V-15 + the new V-15.1**, created by the named conformance assertion
   `v15_request_decode_error_body_exact` in `tests/wire_conformance.rs` — (a) the rendered-body clause is
   **dropped from `P-TP-3`'s invariant** and moved to that golden, and (b) the golden is **named**. Recorded
   so no reviewer reads the register as claiming the bytes.
3. **No row quantifies over `nodeKind: 'community'` as *valid*.** §5.3 (N5) pins that the wire surface does
   **not** expose the fourth `NodeKind` variant; `P-IM-6` therefore asserts its rejection and **not** a
   mapping to `NodeKind::Community` — the store-API-only reachability is pinned at `src/store/mod.rs:4776-4779`
   and is outside this unit's wire contract.
4. **The non-string `mode` value's outcome is pinned as "absent ⇒ `Flat`", and it is *not* an absent key
   (F5's `mode` reconciliation, restated).** §5.3's rule sentence and §5.4's table are the landed authority
   (byte-unchanged): the wrongly-typed rule's **two exceptions are `query` and `filters`**, and **`mode`
   appears as an exception nowhere** — a present non-string `mode` is inside `P-IM-5`'s domain (absent ⇒
   `Flat`), while only a **string** `mode` reaches `P-SM-4`'s resolver. §5.3's per-key table (added by this
   remand) makes that reading **decidable per key**, which was the gap that made a generator unable to choose
   between `Ok + None` and `Err`; the pre-remand prose in `P-IM-5` ("other than `query` and `filters`" *plus*
   a separate `mode` sentence) is superseded by that table. **(2026-09-16 addendum, annotation only:** the
   outcome "absent ⇒ `Flat`" is read at §5.3's two-layer bullet — the **decoder** returns
   `RagQueryOptions.mode == None` for a present non-string (and for an absent key), and `Flat` is the
   **engine's** query-time reading (`mode.unwrap_or(QueryMode::Flat)`, `src/store/mod.rs:4060`, `:4168`);
   only a well-formed **string** token yields `Some(QueryMode::…)`. This restates the same pin, it does not
   relocate it: the authority is still §5.3's rule sentence + §5.4's table, and neither is byte-changed.)**

### 9.5.2 U3 — status honesty (5 rows ≤ 8; **LANDED-GREEN 2026-09-17**)

**Scope of the unit.** Make the six `EngineSubsystems` flags mean **"this subsystem's full query-time
capability is wired and functional for the current store"**: derive them from real state instead of the
hard-coded `true`s (`src/store/mod.rs:1718-1725` — the literal, under its U3 comment block `:1714-1717`; the
pre-U3 citation is `:1710-1717` and the post-U3 `:1714-1725` spelling is stale, §5.8); `store`/`graph`/`lexical` true under capability semantics;
**`reranker` false** (no reranker implementation exists anywhere in `src/`); **`vector` false until U5's boot
index build lands**. **How U3 derives them (F16, pinned):** the flags are computed **inside
`get_engine_status`** (`src/store/mod.rs:4193-4232`, the **only** `EngineStatus` constructor in `src/`; the pre-U3 citation was `:4181-4195`) from
the store's own state — the current snapshot (`snapshot().vectors.is_some()`), the provider seam
(`embedding_provider().is_some()`), and the three always-true capability predicates — so the **boot lifecycle
(`src/bin/gnosis_server.rs:391-408`, landed; pre-U3 `:380-393`, pre-U2 `:324-337` — it never calls `set_subsystems`) keeps its flag obligation at
zero**: it wires what it wired and sets the `state` axis, and the flags follow from that wiring because they
are a read-time projection of it. The mutators (`swap_snapshot`, `set_embedding_provider`) are thereby
**not** wired to write a stored mask, and `Store::set_subsystems` (`src/store/mod.rs:1868-1876`; the pre-U3 citation was `:1862-1863`) stays a test hook whose value
the derived read no longer returns — the **field** it writes (`subsystems: RwLock<EngineSubsystems>`,
`src/store/mod.rs:1642`) is initialized by the LEGACY literal at **`:1718-1725`** (§9.5's register-notes scope note, §5.8's field-vs-literal note). **U3 LANDED-GREEN (2026-09-17):** the
derivation is landed as typed here; the five property rows all HELD (`P-IM-7` 25 / `P-IM-8` 26 / `P-IM-9` 26 /
`P-SM-5` 25 / `P-SM-6` 25 = **127 executed ≤ 400**), and the boot is `main()`'s three-way `BootProvider`
probe + `boot_wiring(...)` call (`src/bin/gnosis_server.rs:378-408`). The same pin governs `P-IM-9`'s
`boot_wiring(...)`: the boot applies **only the returned `EngineState`** (plus its own wiring — the snapshot swap
and the provider), and the returned `EngineSubsystems` is the **DERIVED vector**, returned as the **assertion
surface** — never a mask the boot writes, and never a value a test may write through the hook and read back
(BLOCKING item 4 of the second remand). Any additive capability-vs-index signal lands on **`HealthReport`** (F2's wire type) — never
on the **frozen** `EngineSubsystems` (§4.6.1) — **and U3 adds none: no `HealthReport` field lands in U3**
(F12). The **V-8 / V-8.1 / V-8.2 golden literals change in the same
unit as the flag change** (§5.8) — **and they did (LANDED 2026-09-17):** `tests/wire_conformance.rs:1061-1112`
now asserts the amended literals (the pre-U3 citation `:1024-1061` names the pre-U1 literals that stayed green
only until U3 flipped them, exactly as this clause required).

**The `main()` function** (`src/bin/gnosis_server.rs:377-415`, landed; pre-U3 `:377-400`, pre-U2 `:321-344`) becomes "probe the provider → build `BootProvider` → call `boot_wiring` → **apply the returned `EngineState`** plus the boot's own wiring (`set_engine_state` + `swap_snapshot`, and `set_embedding_provider` only in the `Reachable` case)"; the returned `EngineSubsystems` is **not applied** — it is the derived vector (BLOCKING item 4 of the second remand), so **no** flag mask is written. The lib test therefore asserts **the function's output plus an independent derived read**: ∀ `p ∈ BootProvider`, `snap`: `let (st, f) = boot_wiring(p, &snap)`; `st == Unavailable | Degraded | Ready` exactly as above; and after applying the wiring to a real `Store` (`set_engine_state(st)`, `swap_snapshot(snap)`, and `set_embedding_provider` only in the `Reachable` case — **never** `set_subsystems`) the store's own derived read `s.get_engine_status().await.subsystems == f == the derived capability vector` (`Reachable` ⇒ exactly `{store:true, graph:true, lexical:true, vector:false, embedding:true, reranker:false}` at U3-time; `Absent`/`Unreachable` ⇒ exactly `{…, embedding:false, vector:false, reranker:false}`); the comparison of the pair against the derived read **is** the pin (a disagreement is a broken row), and the lib half MUST NOT write the vector through `set_subsystems` and read it back (that proves the hook, not the derivation). *(Superseded clause: the pre-remand observable read the returned `EngineSubsystems` as a value the boot "applies" and asserted the store's derived read "after applying them" — i.e. it licensed exactly the mask write the register forbids.)* *Live-battery-only half (named, with its precondition — the F11 ruling):* the **provider-reachable** boot outcome as a *live server* state (`Ready` + `embedding:true` from `GET /engine/status`) requires a **controlled provider** (an Ollama-compatible endpoint that answers `/api/tags`) and is therefore asserted in the **live-scenario battery** (`docs/specs/p2-gnosis-server-live-pending-battery.md`), not by a `tests/` row; the provider-**absent** boot (the honest `embedding:false`) is transport-assertable today and is pinned as the e2e assertion **`engine_status_reports_no_false_embedding_claim`** on the booted server.

**Capability predicates (pinned so `P-IM-7`/`P-IM-8`/`P-IM-9` have an exact right-hand side).** A flag is `true` iff:

| flag | capability predicate (the U3 derivation) | evidence of the honest value today |
| --- | --- | --- |
| `store` | the sharded `Store` is the engine ⇒ **always `true`** | `src/store/mod.rs` (the store itself) |
| `graph` | the graph/node/edge walk surface is implemented ⇒ **always `true`** | `graph_query`/`graph_walk`, `src/store/mod.rs:4603`, `:4707` |
| `lexical` | the lexical leg is a **live shard scan** needing no prebuilt index ⇒ **always `true`** | `lexical_rank`/`bm25_search`, `src/store/mod.rs:4852`, `:4261-4274` |
| `embedding` | a query-surface `Arc<dyn EmbeddingProvider>` is wired for this store | `Store::embedding_provider()` (`src/store/mod.rs:1889-1895`; the pre-U3 citation was `:1881-1883`) is `Some` / `None` — and **this is the flag the DEGRADED boot got wrong before U3** (`src/bin/gnosis_server.rs:391-408`, landed; pre-U3 `:390-391`, pre-U2 `:334-336` — the boot sets `Degraded` and never writes a flag) |
| `vector` | the **current derived snapshot** carries a vector index | `snapshot().vectors.is_some()` (`src/store/mod.rs:1849-1853`, `:4294-4297`; the pre-U3 citations were `:1843-1845`, `:4257-4260`); the boot path leaves `vectors: None` (`src/bin/gnosis_server.rs:390-404`, landed; pre-U3 `:385`, pre-U2 `:329`) ⇒ **`false` until U5** |
| `reranker` | a reranker implementation exists and is wired into the query path ⇒ **`false` in every reachable state** | no reranker exists anywhere in `src/` (§5.8, gate-1 GR-3) |

| Property-id | Class | Invariant | Strategy-id | Observable-as-property | Code site / contract clause |
|---|---|---|---|---|---|
| `P-IM-7` | IM | ***(was `P-IM-5` — renumbered by the F1 ruling; §9.5.3.1 REMAND.)*** **A flag may be `true` only for a subsystem whose query-time capability is actually wired for the current store (capability semantics, not "a provider exists").** For **any** store state in the sampled corpus, each flag equals its capability predicate, element-wise; the derivation is the **read-time projection inside `get_engine_status`** (F16), so no caller-visible value depends on a written mask; in particular `reranker` is `false` in **every** reachable state, and `vector` is `false` exactly while the current snapshot has no vector index (so a fresh store and a booted server are both `false` until U5). | `strat:flag-truth-capability` | ∀ state `s` in the corpus (produced by `Store::new()` + `swap_snapshot` + `set_embedding_provider` + `set_engine_state`, **not** by a false-mask `set_subsystems` write): `let f = s.get_engine_status().await.subsystems`; `f.store == true ∧ f.graph == true ∧ f.lexical == true`; `f.reranker == false`; `f.vector == s.snapshot().vectors.is_some()`; `f.embedding == s.embedding_provider().is_some()`; **and the projection is write-independent:** `s.set_subsystems(<all-true>)` followed by a re-read leaves `f` unchanged (the write is not the producer). No flag is `true` without its capability's state witness (the right-hand side above). | §5.8 / F2 §9.1 (capability semantics); the hard-coded construction literal at `:1718-1725` (under its U3 comment block `:1714-1717`; **dead value after U3** — the pre-U3 citation `:1710-1717` and the post-U3 `:1714-1725` spelling are both stale, §5.8), `:1849-1853`, `:1889-1895`, `:4193-4232` (`get_engine_status` — the F16 derivation site; pre-U3 `:4181-4195`), `:1868-1876` (the write-only, observationally inert `set_subsystems` hook; pre-U3 `:1862-1863`) |
| `P-IM-8` | IM | ***(was `P-IM-6` — renumbered by the F1 ruling; §9.5.3.1 REMAND.)*** **Embedding-flag honesty in the absent-provider case: the flag follows the provider, the engine state does not excuse the claim.** For **any** state in which the capability predicate for `embedding` is `false` — the construction state (`EngineState::Unavailable`, no provider: `src/store/mod.rs:1708-1713`; the pre-U3 citation was `:1709-1723`), a store with no provider wired, a booted server whose provider was absent or unreachable — `embedding` is `false`; in particular `Degraded` **never** implies `embedding: true` (today's defect) and `Unavailable` never claims a provider either. Symmetrically, a store whose provider **is** wired reports `embedding: true`. The flag is about the wired **capability**, not about live provider reachability (which `EngineState` already reports) — so a wired-then-unreachable provider keeps `embedding: true` and the honest signal in that case is the `state`/`last_error` pair. | `strat:embedding-flag` | ∀ `s` in the corpus (incl. the two explicit instances `{state: Degraded, provider: None}` and `{state: Unavailable, provider: None}`): `f.embedding == s.embedding_provider().is_some()`; the `{Degraded, None}` instance additionally asserts `f.vector == false` and `f.reranker == false` — the honest V-8.2 mask. | §5.8 (the DEGRADED false-claim; **U3 LANDED-GREEN 2026-09-17 — the claim is now honest at read time**); F2 §9.1's flag table; `src/bin/gnosis_server.rs:391-408` (landed; pre-U3 `:390-391`, pre-U2 `:334-336`); `src/store/mod.rs:1708-1713` (construction), `:1878-1895` (the provider seam; pre-U3 `:1873-1883`), `:4193-4232` (the F16 derivation; pre-U3 `:4181-4195`) |
| `P-IM-9` | IM | ***(was `P-IM-7` — renumbered by the F1 ruling; §9.5.3.1 REMAND.)*** **The boot lifecycle's wiring determines the flags — a boot with no reachable provider makes no `embedding` claim, and the flag vector is the derived vector of WHAT WAS WIRED (F11: the lib half goes through the lib-visible boot-wiring helper, never through `set_subsystems`).** For **any** boot outcome, the engine state is set by the boot's own branch (provider reachable ⇒ `Ready`; otherwise `Unavailable`/`Degraded` — never a fabricated `Ready`) **and** the derived flag vector equals the capability vector of what that boot actually wired: `embedding == (a provider was wired)`, `vector == (the current snapshot has a vector index)`, `store`/`graph`/`lexical` `true`, `reranker` `false`. So a provider-absent boot reports `{store:true, graph:true, lexical:true, vector:false, embedding:false, reranker:false}` — **never** `embedding:true`; a reachable-provider boot reports the same vector with `embedding:true` **and nothing else changed** (`vector` stays `false` until U5). | `strat:boot-flag-honesty` | **Layer pinned (F11) + the boot-wiring seam named.** *Lib level:* the boot's wiring is a **required lib-visible function** in U3, pinned here as **`boot_wiring(provider: BootProvider, snapshot: &DerivedIndexes) -> (EngineState, EngineSubsystems)`** where **`BootProvider ∈ {Absent, Unreachable, Reachable(Arc<dyn EmbeddingProvider>)}`** (the name is the unit's; the behavior, the enum and the argument list are pinned) — a **pure function of what the boot wired** that consults no live network: `Absent` ⇒ `(Unavailable, {embedding:false, …})`; `Unreachable` ⇒ `(Degraded, {embedding:false, …})`; `Reachable(_)` ⇒ `(Ready, {embedding:true, …})`; in all three, `vector == snapshot.vectors.is_some()`, `store`/`graph`/`lexical` `true`, `reranker` `false`. **The three-outcome enum is the F11 ruling's shape** — it is what makes §9.5.2's adjudication note 2's "three boot outcomes" three *distinct inputs* — but **the second element of the return is NOT a value the boot writes (BLOCKING item 4 of the second remand: pinned here so the row cannot be read as licensing a mask write).** Of `boot_wiring(...)`'s two outputs the boot applies **only the `EngineState`**; the returned **`EngineSubsystems` is the DERIVED vector** — the read-time projection of the store's own state that `get_engine_status` performs (§9.5's register-notes scope note, F16) — **written nowhere** by the boot or by any mutator, and **returned solely as the assertion surface** (a lib test can compare it without a live store read). The boot itself wires the snapshot (the `swap_snapshot` equivalent) and the provider (`set_embedding_provider`), sets the `state`, and **never** writes a flag mask (`set_subsystems` stays a test hook — §9.5.2's mechanism note). *The lib assertion is therefore two-sided:* `boot_wiring(...)`'s returned pair **plus an independent derived read** of the store's own flag vector after the boot's wiring is applied (e.g. the `get_engine_status(...).subsystems` of the store whose snapshot/provider the boot wired) are asserted **element-wise equal**. The pair is what the wiring *demonstrates*, the derived read is what the store *reports*, and a disagreement is a broken row. | §6 (the READY lifecycle); §5.8's boot clause; F2 §9.1; `src/bin/gnosis_server.rs:380-393` (landed; pre-U2 `:324-337`) (today: no `set_subsystems` call), `:321-344` (`main`); `src/store/mod.rs:1862-1863` (the unused hook), `:4181-4195` (the derivation) |
| `P-SM-5` | SM | **The status read is a deterministic, side-effect-free projection of the store's state.** For **any** store state, repeated `get_engine_status()` calls with no intervening mutation return **element-wise identical** `EngineStatus` values, and the call itself mutates nothing observable: the journal/epoch counter does not advance and the derived snapshot (its `Arc` identity and its `vectors` presence) is unchanged — which is exactly what the F16 read-time derivation requires (a pure read has nothing to write). | `strat:status-pure-read` | ∀ `s`: `let a = s.get_engine_status().await; let b = s.get_engine_status().await;` ⇒ `a == b` (state, version, every flag, `last_error`); and `s.epoch()`/`s.journal_len()` are identical before and after **and** the snapshot's `Arc` identity/`vectors.is_some()` is identical before and after. | `src/store/mod.rs:4193-4232` (a read of two locks + a clone, plus the F16 flag derivation — still read-only; pre-U3 `:4181-4195`); `:1814-1823` (epoch/journal observers; pre-U3 `:1808-1815`); `:1849-1853` |
| `P-SM-6` | SM | **The status projection stays FAITHFUL — the row is the `health(&status)` projection only (F12; the shape half moved out).** For **any** `EngineStatus`: `health(&status)` mirrors `state`, `version`, all six flags and `last_error` (`Some` iff the input's is `Some`), and invents nothing (the envelope constants are the pinned current version + `opaque-string-v1`, never derived from the input). | `strat:status-faithful` | ∀ `status` (all four states, representative masks): `let h = health(&status)`; `h.state == status.state`, `h.version == status.version`, `h.last_error.is_some() == status.last_error.is_some()`, each of the six flags `h.subsystems.<f> == status.subsystems.<f>`, and `h.schema_version == current_schema_version() ∧ h.id_format == ID_FORMAT_OPAQUE_STRING_V1`. **The frozen-shape half is NOT this row** — a generator can only sample values that already exist, so "the six fields are still `bool`" is an **acceptance/frozen-type criterion**, not a falsifiable property (F12); it is asserted at the **conformance layer** by the named assertion **`health_report_shape_frozen`** in `tests/wire_conformance.rs` (compile-time field accesses + `serde_json::to_value(&subsystems)` yielding exactly the six keys), alongside F2 §9.1's F7 rule and the V-8.x literals. **And U3 lands NO additive `HealthReport` field** (F12): the F2 §9.1 additive permission stays **unused** in U3 — `HealthReport` keeps exactly `{schemaVersion, idFormat, state, version, subsystems, lastError}` (`src/wire/status.rs:12-21`) — so there is no field for a row to verify and none is claimed. | F2 §9.1 (F7: `EngineSubsystems` MUST NOT gain/lose/re-type a field; U3's blast radius is `HealthReport` + `status.rs` + the V-8 literals); `src/wire/status.rs:12-35`; `src/store/mod.rs:568-575`; the untouched F2 `P-SM-3` (`docs/specs/7-2-wire-property-register.md`); F2 §12's V-8/V-8.1/V-8.2 (the golden edit lands in U3) |

**Class tally (U3):** IM ×3 (`P-IM-7`, `P-IM-8`, `P-IM-9`), SM ×2 (`P-SM-5`, `P-SM-6`) = **5 rows ≤ 8** ✔.

**Red-before-green (U3 — RED STAGE RECORDED, NOW LANDED-GREEN 2026-09-17).** `P-IM-7`/`P-IM-8`/`P-IM-9` were **broken against
the pre-U3 code** — the six flags were hard-coded `true` at construction (`src/store/mod.rs:1718-1725`; the
pre-U3 citation was `:1710-1717`) and the
boot path never updated them (`src/bin/gnosis_server.rs:390-408`; the pre-U3 citation was `:385-393`, pre-U2 `:329-337`) — and **held** against U3's derivation;
`P-SM-5`/`P-SM-6` hold today and must keep holding (they are the U3 blast-radius guards: the status read must
stay pure and the `health` projection must stay faithful). `P-IM-9`'s `boot_wiring` fn is **new code** in
U3's unit (F11), so its lib-level assertion is red until it lands.

**U3 notes (pinned so the rows are not read wider than they are).**

- **The V-8 golden literals are a conformance obligation, not a row.** §5.8/F2 §12's V-8.1/V-8.2 literals are
  **byte-exact conformance vectors** whose edit lands in **this same unit** as the flag change
  (`tests/wire_conformance.rs:1061-1112` — the pre-U3 citation was `:1024-1061`; literals + `all_true_subsystems()` at `:697-706` — the pre-U3 citation was `:689-698`). Rows `P-IM-7..9`
  quantify over **status values**; the literal bytes are asserted by the conformance suite in the same unit —
  the two layers are complementary, and the golden edit is **not** a property row.
- **U3's code is LANDED-GREEN (2026-09-17) — recorded here so no clause below reads as owed.** The five rows
  all **HELD** with the pinned tags (`U3PIM7`…`U3PSM6`): `P-IM-7` 25 cases, `P-IM-8` 26, `P-IM-9` 26,
  `P-SM-5` 25, `P-SM-6` 25 = **127 executed ≤ 400** (the ≤400 is the **per-unit** cap; `tests/props_gnosis_server.rs:59-76`
  for the layer's own arithmetic block, `:278-295` for the caps/`U3_EXECUTED_TOTAL`, and
  `u3_layer_budget_discipline`, `:5689`), the boot-wiring seam `boot_wiring(...)` is landed in the lib
  (`src/lib.rs:69-110`), the V-8.1/V-8.2 golden literals are amended **in the same unit**
  (`tests/wire_conformance.rs:1061-1112` + `boot_wiring_couples_to_the_derived_read`, `:1129`), the blind set is
  **13/13 PASS** (`tests/blind_u3_status_honesty_greens.rs` + `docs/specs/u3-status-honesty-greens.md`), the live
  battery is **9/9 rows PASS live** (including **`R-L2`**), and the suite is **604 passed / 0 failed**
  (`cargo test`, serial), green **with and without** `GNOSIS_SERVER_OLLAMA_URL` set (the suite is hermetic);
  `cargo fmt --check` exit 0, `cargo clippy --all-targets` 0 warnings, `cargo build` clean. The U3 DONE row and
  the review record are **not** written here — they belong to the supervisor and the documentation-review gate.
  (Two U3-time findings are filed as OPEN defect rows — post-boot provider loss is invisible to `/engine/status`,
  and `encode_result`'s `expect` panic path — see `docs/defects.md`.)
- **`vector` flips to `true` in U5, not U3.** Under `P-IM-7`'s predicate `f.vector == snapshot().vectors.is_some()`,
  the U5 unit's boot index build flips both the **live** flag and V-8.1's `"vector"` literal **in the same
  unit** (§5.8; F2 §12). Until then, `vector: false` is the honest value and `mode=vector` on a booted server
  is the accepted, documented residual (FS-14 ⇒ 503, §5.9).
- **The register row §11 named for U3 is landed as `P-IM-7`** (its "a subsystem flag is `true` iff that
  subsystem's full query-time capability is wired and functional for the current store" sketch), with the
  DEGRADED-boot and boot-wiring obligations split into `P-IM-8`/`P-IM-9` and the projection obligations into
  `P-SM-5`/`P-SM-6` so each row has one falsifiable observable. Nothing in the U1-pinned §5.8 text is re-scoped.
  *(The id in this bullet was `P-IM-5` pre-remand; the §9.5.3 REMAND entry records the renumbering.)*
- **`EngineState` is a READINESS axis, not a capability axis (F13, pinned — resolves the `Ready` +
  `reranker:false` mismatch).** `Ready` asserts **exactly three things**: the construction/boot wiring completed,
  the store/graph/lexical core is up, and **an embedding provider was wired and reachable** (§6 / §9.5.2's
  boot-outcome table). It asserts **nothing** about `vector` or `reranker` — under §9.5.2's capability
  predicates `vector` is `true` iff the current snapshot carries a vector index (U5) and `reranker` is `false`
  in every reachable state at U3 (no implementation exists in `src/`). The combination
  `{state: Ready, vector: false, reranker: false}` is therefore **legitimate and expected**, not a mismatch,
  and the two axes are read together: the **flags** say what is wired, the **state** says whether the engine is
  serving (plus `last_error` for the degradation reason). **`P-IM-7`/`P-IM-8` MUST NOT be read as "`Ready` ⇒
  every flag the state names is `true`", and no row asserts that.** The divergence from the gate-1 prose that
  implied a single capability axis is recorded in `docs/defects.md` (OPEN row **"`Ready` does not imply every
  subsystem flag is `true` (state vs capability axis)"**), which also records that U3 is **not** asked to
  tighten the boot's `Ready` condition (a tightening would require a vector index and a reranker — i.e. U5 and
  a reranker implementation — to boot READY at all).

**U3 adjudication notes (same discipline as §9.5.1's: recorded, not invented).**

1. **The boot branch lives in the bin, so `P-IM-9`'s assertion is split across two layers — and both layers are
   NAMED with an observable (F11 + adjudication 4).** The boot
   lifecycle is `main()` in `src/bin/gnosis_server.rs:377-415` (landed; pre-U3 `:377-400`, pre-U2 `:321-344`) — a `[[bin]]` target, **not** reachable from
   `tests/` (`src/lib.rs` exposes the lib only). The **derived-vector** half of `P-IM-9` is therefore
   asserted at **lib level through the required lib-visible boot-wiring function `boot_wiring(...)`** pinned in
   `P-IM-9`'s observable cell (a pure function of *what the boot wired*, of whose two outputs the boot applies
   **only the `EngineState`** — the `EngineSubsystems` it also returns is the **derived vector**, written nowhere,
   and exists as the **assertion surface**; BLOCKING item 4 of the second remand) —
   **not** by writing the vector through `set_subsystems` and reading it back (that proves the hook, not the
   derivation: F11), and **not** by inventing a lib boot seam the unit does not create. The **provider-reachable**
   boot outcome (`Ready` + `embedding:true`) needs a **controlled provider** and is therefore
   **live-battery-only**, with that precondition stated in `P-IM-9` and its home named as the battery's
   **named row `R-L2` (§3.5)**
   (`docs/specs/p2-gnosis-server-live-pending-battery.md`); the **provider-absent** outcome is asserted at the
   **transport level** by the named e2e assertion `engine_status_reports_no_false_embedding_claim` on the
   booted server. §6's READY-lifecycle text is unchanged; this is a test-surface boundary with a named home,
   **not** a contract gap — and *no* U1 pin is re-opened by it. ***(REMAND-1, 2026-09-22: the "§6 … unchanged"
   half now needs the qualifier — U5's dated step was **added** to §6's sequence (after `DerivedIndexes`,
   before the state transition) so the file no longer carries two accounts of the boot; §6's historical
   sentence, the state names and the flags are still as they were, and this note's own point (the boot branch
   is a `[[bin]]` path with a lib-visible seam) is unaffected.**)***
2. **The boot path's reachable state set is narrower than §6's wording (no defect, pinned here so a
   generator does not assert an unreachable state).** `main()` sets `Ready` when a provider is reachable and
   `Degraded` when a configured provider is unreachable
   (`src/bin/gnosis_server.rs:391-408`, landed; pre-U3 `:386-393`, pre-U2 `:330-337`); the *absent*-provider branch leaves the construction value
   `EngineState::Unavailable` (`src/store/mod.rs:1713`; the pre-U3 citation was `:1709`). So the live boot produces `Ready` or `Degraded` or
   `Unavailable`, while §6's prose ("`DEGRADED` (non-core subsystem) or `UNAVAILABLE` (engine not
   running)") is a **superset** of that. `P-IM-8`/`P-IM-9` therefore quantify over the three boot outcomes
   with the state asserted as "`Ready` iff the provider was reachable, else `∈ {Degraded, Unavailable}`" —
   **never** "always `Degraded` when the provider is absent" (a state the code does not produce, and the
   thing this note exists to prevent a TestWriter from asserting). This is a **documentation-boundary**
   observation about §6's prose, **not** a U1 rule: §6's sentence stands as written and U3 is not asked to
   change it. *(The `boot_wiring` signature of `P-IM-9` expresses this: "provider reachable" is the
   `BootProvider::Reachable(provider)` input, "configured but unreachable" is `BootProvider::Unreachable`,
   and the *absent* case is `BootProvider::Absent` — so the three outcomes are three distinct inputs and no
   generator has to guess which one a bare `None` means.)*

### 9.5.3 The property-layer execution plan (U2 + U3, one block)

**Applies to both units.** The TestWriter's executed layer is written per unit from the rows above, as **one
`#[test]` per `Property-id`**, tagged with its `Strategy-id`, reporting **held/broken per row**.

- **Command.** `cargo test` (the whole suite; a unit's property layer must not be run in isolation from the
  conformance layer in the final trio). The repo has a gitignored, intentional sandbox-local cargo home at
  `.cargo-home/`; if writes to `~/.cargo` are blocked in the sandbox, run with
  `CARGO_HOME=$PWD/.cargo-home cargo test` (or `CARGO_HOME=/tmp/gnosis-cargo-home cargo test`).
- **Deterministic seed pin.** **One** pinned master seed for the unit's property binary, mixed with a
  per-row tag — the house harness's existing rule
  (`tests/props_gnosis_server.rs:147`: `SEED = 0x9E37_79B9_7F4A_7C15`, `splitmix64` at `:150-156`; the Xoshiro256**
  `Rng` at `:159-202` and `row_seed(tag)` at `:205` — the pre-U2 citation `:48-118` and the pre-U3 citation
  `:109-118`/`:120-163`/`:167` are stale). U2/U3 rows use
  the **same** `SEED`. **The tag convention is UNIT-DISCRIMINATED inside
  this binary (re-derived by the second register remand — the collision ruling).** The executed layer is
  **one binary per file** — `tests/props_gnosis_server.rs` executes this p2 register — and that file
  **already** builds its seven landed §11 rows with the same `SEED` convention and the tags `PIM1`/`PIM2`/
  `PIM3`/`PSM1`/`PSM2`/`PSM3`/`PTP1` (the tag constants at `tests/props_gnosis_server.rs:210-216` — the
  pre-U3 citation was `:172-178` — `SEED` at
  `:147`; the seven
  `row_seed(...)` call sites at `:675`, `:727`, `:777`, `:849`, `:902`, `:953`, `:1011` — the **evidence** for
  this ruling; the pre-U3 citations `:595`/`:647`/`:697`/`:769`/`:822`/`:873`/`:931`, and the pre-U2 `:112-118` / `:507-843`, are stale). The pre-remand scheme (`P-<CLASS>-<N>` ⇒ `P<CLASS><N>`) therefore **collided with those
  landed tags** (`PIM4`… is disjoint by number, but the scheme put the §9.5 rows in the same tag namespace as
  the landed rows **and** was non-injective across units — the exact failure F1's rule exists to prevent), so
  U2/U3 rows take an **injective, unit-prefixed** tag: **`U<unit>` + the `P<CLASS><N>` id form**, i.e. the
  unit number followed by the class letters and the row number — **twelve tags, one per row, injective over
  §9.5's twelve rows** — **U2** ⇒ `U2PIM4`, `U2PIM5`, `U2PIM6`, `U2PSM4`, `U2PTP2`, `U2PTP3`, `U2PTP4` (7); **U3** ⇒
  `U3PIM7`, `U3PIM8`, `U3PIM9`, `U3PSM5`, `U3PSM6` (5) (each a 6–7-byte ASCII tag, well inside the `u64`).
  **The U2/U3 tag set is DISJOINT from the landed `PIM1`…`PTP1` set** — they share neither a tag value nor a
  row stream — so every row of §9.5 gets its own stream and per-row held/broken reporting stays meaningful
  (the tag constants are appended to the harness's tag list, not substituted for the landed ones). **No two
  rows share a stream** (the pre-remand list had nine tags for ten rows and mapped U2's and U3's `P-IM-5`
  both to `PIM5` — superseded; the second remand's scheme supersedes the interim `P<CLASS><N>` form as well).
  No wall-clock, no thread-order, no `rand::thread_rng` input may enter a row.
- **Attempt caps (corrected arithmetic — F2; scope pinned — the bookkeeping pass, 2026-09-16.)** **≤ 100
  generated cases per row**; **≤ 400 total cases per unit's property layer — the ≤400 is a PER-UNIT cap (one
  unit's register = one layer), never a per-binary budget.** The cap's authority is decision
  **`PBT-GATE-MANDATORY`** (`docs/decisions.md:68`: the ≤100/row and ≤400 total belong to the *per-unit*
  register artifacts — *"Per unit three artifacts are required"*, artifact #2 being that unit's executed
  layer), as applied to each unit's own rows here. **The pre-U1 clause's "one register file = one layer,
  therefore the cap is applied to U2 + U3 together" reading is SUPERSEDED in place**: correct in the
  pre-U1 state it described (U2's and U3's rows did not yet exist in one file), it became **wrong once
  this file carried two units' registers as two layers** — on that reading the file's single layer would
  be `310 + 400 = 710 > 400` (the U2 layer as landed), contradicting the per-row caps the property binary actually carries
  (`tests/props_gnosis_server.rs:16-40`, each row ≤ 100), and it is **not**
  how the repo's registers read the cap (`docs/specs/7-2-wire-property-register.md:14`,
  `docs/specs/4-1-store-property-register.md:17`, `4-3-facts:59`, `4-4:28`, `4-5:14`, `6-f6-eval:25`:
  "across the unit's whole property layer"). **Each unit's layer therefore satisfies the ≤400 cap on its
  own**: the **landed `p2` layer is 310** (`tests/props_gnosis_server.rs:16-24` — the seven per-row caps
  `60/40/60/40/30/40/40` for §11's seven landed rows, header comment "sum = 310 ≤ 400"; row call sites
  `:595`/`:647`/`:697`/`:769`/`:822`/`:873`/`:931`), and the **U2 layer, as landed, is exactly 400** — the
  per-row caps `48/100/70/80/23/41/38` (`tests/props_gnosis_server.rs:26-40`, `:209-215`; the row arithmetic is
  itself asserted by `u2_layer_budget_discipline`, `:4115-4149` — the fn is `u2_layer_budget_discipline`, `:4115`; the pre-U3 citations `:4034-4069`/`:4035` and the pre-review `:3968-4003` were stale) — so the layers this binary executes are
  **310 ≤ 400 ✔** (the `p2` layer), **400 ≤ 400 ✔** (U2) and **127 ≤ 400 ✔** (U3, **as landed** — the five rows
  `25 + 26 + 26 + 25 + 25`). **The pre-landing indicative budget was `7 × 35 = 245` for U2 + `5 × 25
  = 125` for U3 = 370; the U2 adversarial-audit coverage pass spent the layer's remaining headroom and re-sized
  the seven U2 rows to the 400 above (still ≤ 400 per unit, each row ≤ 100), and U3's layer landed at its
  executed **127** (`tests/props_gnosis_server.rs:278-295` + `u3_layer_budget_discipline`, `:5689`; the
  pre-landing 125 is kept here as the indicative figure it was).**
  **The binary total is the SUM of the units' layers and is not itself capped at 400** — `310 + 400 + 127 = 837`
  is the **pre-U5 record** of the binary's state (***REMAND-2 NOTE 9, 2026-09-22: this bullet carried no U5 annotation, so the `837` read as the current total; the U5 layer (`340`) was authored in §9.5.5 afterwards and the **current** total is `310 + 400 + 127 + 340 = 1177` (the same arithmetic §9.5.5's execution plan and §11's U5-update note state). The `837` sum stands as the pre-U5 record, not as a violation***) — ***POST-RED-PHASE AMENDMENT, 2026-09-22: the U5 layer this bullet names is now **355** (per-row caps `60/50/40/45/45/30/45/40`, each ≤ 100; measured **314 executed / 355 caps**), so the **current** total is `310 + 400 + 127 + 355 = 1192`; the `340` layer figure and the `1177` total above are **superseded records** kept in place, and §9.5.5's post-red-phase register amendment is the authority.***), **not a violation** (the layer, i.e. one unit's register, is the scoped unit of
  work; a per-binary cap would forbid a second unit's register from landing here at all). The pre-remand budget (**7 × 40 = 280 + 5 × 25 = 125 = 405 > 400**) is
  **superseded** — it claimed to be within cap while exceeding the cap **of its own single pre-U1 layer**
  by 5 (U2's own 7 rows at 40 would have been 280, and 280 + 125 = 405 was the single layer the 2026-09-10
  register carried). Any split satisfying `7a + 5b ≤ 400` with `a, b ≤ 100` **within the pre-U1
  single-layer reading** was admissible; **post-U1 the per-unit reading applies instead**, i.e. U2's seven
  rows sum ≤ 400 (**`48/100/70/80/23/41/38 = 400` as landed ✔**; the pre-landing indicative split was
  `7 × 35 = 245`) and U3's five rows sum ≤ 400 (**`25/26/26/25/25 = 127` as landed ✔**; the pre-landing
  indicative split was `5 × 25 = 125`) — **the two units' sums
  are never added together for cap purposes** — and the chosen split additionally keeps the U3 rows'
  boundary-heavy shape at ~25 each.
- **Stop-after-5.** A row **aborts on its 5th distinct counterexample** and reports **at most 5** distinct
  minimal counterexamples — never an unbounded dump, never a panic-without-`cex` (the house harness's
  `BROKEN: {cexes:?}` shape).
- **Held/broken reporting.** Each row prints/records exactly one line of the form
  `[<Property-id>][<Strategy-id>] HELD` or `[<Property-id>][<Strategy-id>] BROKEN: <≤5 counterexamples>`,
  plus its generated-case count (`[<Property-id>] generated cases: N HELD=true|false` — the house form).
  A broken row **fails** the test (the unit is red), and its `Strategy-id` is what the audit reads.
- **One-pass remand rule (pinned).** A **broken** row is triaged **exactly once**, into one of three
  dispositions, and the disposition is recorded in the unit's report — never re-run-and-hope, never silently
  relaxed:
  1. **host-fix** — the property is right and the **implementation** is wrong (or the flag/label is in the
     wrong state): the least code change lands, the row is re-run, and the fix is regression-pinned;
  2. **package-defect** — the row exposes a **pre-existing** engine/contract defect outside the unit's scope
     (a reserved variant reachable, an inert option, a store-side corner): it is recorded in
     `docs/defects.md` (+ `docs/HANDOFF.md` if it is upstream), **and a negative probe** is added rather
     than a weakened row;
  3. **over-strong-requiring-re-derivation** — the row asserts more than the contract pins: the **register
     row is re-derived and corrected** by the SpecWriter (the register, not the implementation, is the
     artifact at fault), with the correction noted in this section; a row may **not** be deleted, and the id
     is **not** reused for a different claim.
- **Read-only audit.** After the row layer is green, the adversarial reviewer performs the standard
  **read-only** PBT audit (per-row over-strength reasoning, generator coverage, prose counterexamples,
  negative-generator requests) against these tables; **reviewers never run generators**.
- ***(U6 note, 2026-09-22 — ADDITIVE; the bullets above stand as the U2/U3/U5 record.)*** **U6's layer is a
  separate per-unit layer and its execution plan lives in §9.5.6** (its own seed/tag set `U6PIM16`…
  `U6PTP6`, its own caps `60/50/40/45/60/45 = 300 ≤ 400`, its own `u6_layer_budget_discipline` test, and its
  own layer classification incl. the one **bin-level** row `P-IM-19` and the live row `R-L4`). ***(REMAND-1,
  2026-09-22: the six tag **constants are now pinned exactly** in §9.5.6's seed-pin bullet — the pre-remand
  "ASCII of its own name" wording was not implementable for the four 7-byte `U6PIM*` tags, whose landed-form
  values follow `U5PIM10`'s 8-byte `…_4D31_3030` spelling — and the layer's **lib-level** count stands at five
  because `P-TP-6`'s observable is §9.5.6's **second** lib seam (`PROVIDER_REQUEST_TIMEOUT` /
  `provider_client`); no cap, count, kind or strategy id moves.***)*** The rules the
  bullets above pin — the same master seed, `row_seed(tag)`, ≤100/row, ≤400 per unit, **a cap being a
  maximum rather than an expected count**, stop-after-5, the held/broken line form with the `Strategy-id`,
  and the one-pass remand rule — apply to U6 **unchanged**; the bullet above that restates the **binary
  total** is extended by U6 to `310 + 400 + 127 + 355 + 300 = 1492` (a **sum**, still not a cap), with the
  earlier `837`/`1177`/`1192` figures kept as their dated records.

#### 9.5.3.1 REMAND — the register-reviewer disposition (2026-09-16; docs-only, no row deleted)

The **register reviewer** (read-only) returned **11 MUST-FIX (F1–F11) + 4 SHOULD-FIX (F12–F15) + 1 NOTE
(F16)** on the §9.5 registers. All of them are addressed in place; nothing was deleted, both tables keep their
row counts (U2 = 7, U3 = 5, `≤ 8` each), and no **landed U1 rule** was edited (the mode rule, the canonical
§4.5.2 `filters` mapping, the token-check location, the wrongly-typed rule, the SSE 400 pin, the
POST-payload-only split, the transport decode body, the capability semantics and the frozen
`EngineSubsystems` are reused as written — this pass fixes the **rows that describe them**).

**The id-collision resolution (F1 — renumber-with-provenance, no id reused for a different claim).**

| old id (unit) | new id | claim (unchanged) | every citation site updated |
| --- | --- | --- | --- |
| `P-IM-5` (U3) | **`P-IM-7`** | flag truth under capability semantics | §9.5.2 row (id + provenance marker) + its class tally + the U3 notes bullet; §11's AUTHORED annotation; §12 item 5; this plan's tag list; F2 §9.1's cross-reference note |
| `P-IM-6` (U3) | **`P-IM-8`** | embedding-flag honesty (absent/DEGRADED provider) | same list (row, tally, notes, §11, §12, tags, F2 §9.1) |
| `P-IM-7` (U3) | **`P-IM-9`** | boot-flag honesty (the boot's *wired* vector) | same list (row, tally, notes, §11, §12, tags, F2 §9.1) + the U3 adjudication notes 1–2 |
| — | *(unchanged)* | U3's `P-SM-5`/`P-SM-6` never collided and stay | — |

Root cause recorded: §9.5 is a **continuation of the p2 register numbering** (one namespace per file), so
U3's reuse of U2's `P-IM-5`/`P-IM-6` gave two rows one id, and the tag scheme (`id` bytes → seed tag) then
mapped both to `PIM5`, while the plan listed **nine** tags for **ten** rows — breaking per-row held/broken
reporting and the adversarial audit. The three U3 IM ids were therefore renumbered **continuing after the
highest IM id claimed anywhere in this file** (p2 `P-IM-1..6`), the tag list was **re-derived injective over
all twelve rows**, and §9.5's header now states the **single-sequence** allocation rule. **Cross-file
look-alikes are untouched**: `docs/specs/7-2-wire-property-register.md`'s own `P-IM-4`/`P-SM-4`,
`docs/specs/4-3-facts-property-register.md`'s `P-SM-4`/`P-SM-5`/`P-TP-2`,
`docs/specs/4-1-store-property-register.md`'s and `docs/specs/4-4-consistency-property-register.md`'s
`P-TP-2` rows are **different units in different files** and are legitimately unchanged.

**Row-by-row disposition.**

| F# | class | where it was fixed | what changed |
| --- | --- | --- | --- |
| **F1** | MUST-FIX | §9.5 header; §9.5.2 rows/tally/notes; §9.5.3 tags; §11; §12 item 5; F2 §9.1 note | U3's `P-IM-5/6/7` ⇒ `P-IM-7/8/9`, single-sequence rule stated, tags re-derived (10 unique), all citations moved |
| **F2** | MUST-FIX | §9.5.3 attempt caps | budget restated `7×35 = 245` + `5×25 = 125` = **370 ≤ 400**, `≤100`/row kept; **the ≤400's SCOPE was pinned in the bookkeeping pass (2026-09-16, finding B4): it is a PER-UNIT cap on one unit's register layer, not a per-binary budget — the landed `p2` layer is 310 (`tests/props_gnosis_server.rs:16-24`), each ≤ 400 on its own, and the binary total is their sum and is NOT capped (decision `PBT-GATE-MANDATORY`). THE 245/370 SPLIT IS THE PRE-LANDING INDICATIVE BUDGET ONLY: the executed U2 layer landed at **400** (`48/100/70/80/23/41/38`, `:26-40`), and U3's layer landed at its executed **127** (`25/26/26/25/25`, `tests/props_gnosis_server.rs:278-295` + `:5689`) — the pre-landing 125 was indicative (§9.5.3, proofread pass 2026-09-17)** |
| **F3** | MUST-FIX | §9.5.1 `P-IM-4` | restated over two **disjoint** sets (envelope/transport ⇒ `Err`; everything else ⇒ `Ok` + §5.3 mapping); the decidable "well-formed" predicate removed and left to the owning rows |
| **F4** | MUST-FIX | §5.3 (new **14-key wrongly-typed table** + reconciled rule sentence); §9.5.1 `P-IM-5`; §9.5.1 adjudication note 4 | one per-key ruling for all 14 keys; `mode` appears **once**, as a wrong-type-⇒absent case (not an exception); row cites the table instead of enumerating exceptions |
| **F5** | MUST-FIX | §9.5.1 `P-IM-6` | casing policy pinned explicitly (three token members ASCII-case-insensitive, `state` accepting the store casing as a variant); wrongly-typed `target` member, `target` with extra members, unknown members (`nodekind`, snake_case spellings) and the serde-`target`-array shape all pinned; the `{}` and no-serde-passthrough pins kept |
| **F6** | MUST-FIX | §9.5.1 signature table + new **signature note** | the decoder now takes the SSE params (`SseParams { query, top_k, mode }`, or the param list); **`sse_params.mode` is named as the argument that carries `mode` on the SSE path**; the split entry point is the stated alternative |
| **F7** | MUST-FIX | §9.5.1 `P-IM-5` observable | domain bounded (non-empty well-formed `query`, range-validated options in range or `None`); the path quantifier restricted to the keys each path's seam carries; the excluded fail-states named as excluded |
| **F8** | MUST-FIX | §9.5.1 `P-TP-3` (split); F2 §12 (V-15.1 added) | pure half stays in the lib; the rendered-body/`Content-Type` half moves to the conformance layer, **with the creating assertion named** (`v15_request_decode_error_body_exact` in `tests/wire_conformance.rs`); the message contract restated (verbatim, **may be empty**; `UnsupportedSchemaVersion(v)` ⇒ non-empty) |
| **F9** | MUST-FIX | §9.5.1 `P-IM-4` | `InvalidEnvelope` is the single code for a non-object `payload`; `InvalidJson` is pinned unreachable on that path |
| **F10** | MUST-FIX | §9.5.1 `P-TP-2` (+ adjudication note 1) | row is purely the decode-level identity, **and** the handler pass-through is a required live assertion with named observables (α: new e2e `rag_query_unrecognized_mode_is_400_e2e`; β: the live battery's `topK`/`filters`-changed-the-response) |
| **F11** | MUST-FIX | §9.5.2 `P-IM-9` (+ U3 adjudication note 1) | a **lib-visible boot-wiring function** is required and its signature pinned (`boot_wiring(provider: BootProvider, snapshot: &DerivedIndexes) -> (EngineState, EngineSubsystems)`, `BootProvider ∈ {Absent, Unreachable, Reachable(Arc<dyn EmbeddingProvider>)}`); the `set_subsystems`-read-back assertion is forbidden; the provider-reachable outcome is **named live-battery-only with its precondition** |
| **F12** | SHOULD-FIX | §9.5.2 `P-SM-6` | row is the `health` faithfulness projection only; the frozen-shape half moves to the conformance layer as the named `health_report_shape_frozen`; **no `HealthReport` field lands in U3** (stated) |
| **F13** | SHOULD-FIX | §9.5.2 U3 notes; `docs/defects.md` (OPEN row) | one pinned sentence: `EngineState` is a **readiness** axis the flag semantics do not scope (a `{Ready, vector:false, reranker:false}` combination is legitimate), with the divergence recorded as a defects row |
| **F14** | SHOULD-FIX | **§9.5.4** (new: per-row generator-coverage notes for all 12 rows) | one ≤5-line coverage note per row, naming boundary + adversarial shapes and the excluded fail-state inputs |
| **F15** | SHOULD-FIX | §9.5.1 `P-SM-4` observable | the live-transport wiring assertions are named (`?mode=bm25` ⇒ 400 + `validation_error` frame; `?mode=HYBRID` ⇒ non-400; `?expand=`/`?compression=` ⇒ non-400) |
| **F16** | NOTE | §9.5.2 scope; §9.5 register-notes scope note; §9.5.1 `P-IM-7` | U3's mechanism pinned: the flags are **derived inside `get_engine_status`** (read-time projection of the store's state), the mutators are not wired to write a mask, and the boot has **no** flag obligation — which is what removes the "verbatim-writing hook" framing |

**Adjudication dispositions.** **Adjudication 1** (handler pass-through) and **adjudication 4** (`mode`'s
non-string reading) are handled as required: (1) is now a **named obligation** folded into `P-TP-2` + U3-note
style wording (MUST-FIX), and (4) is restated as the §5.3-table reconciliation in §9.5.1's adjudication note 4.
**Adjudications 2, 3 and 5 are ACCEPTED** and retained (2 = the bin-level shared rendering, now carrying F8's
two conditions: the rendered-body clause is dropped from `P-TP-3` and the golden that verifies it is named;
3 = no row quantifies over `nodeKind:'community'` as valid; 5 = the boot-path state set is narrower than §6's
prose).

#### 9.5.3.2 REMAND-2 — the register-verification reviewer's final disposition (2026-09-16; docs-only)

The **register verification reviewer** (read-only) returned **4 MUST-FIX + 2 BLOCKING** items against §9.5 and
its supporting text. All six are addressed in place; **no row was deleted**, both row counts are unchanged
(**U2 = 7**, **U3 = 5**, `≤ 8` each), the **U1 contract rules are untouched**, and no register row's *substance*
changed beyond what the findings require (each change below is a **restatement, a citation, a count
correction or a named home**).

| # | class | where it was fixed | what changed |
| --- | --- | --- | --- |
| **1** | MUST-FIX | `docs/specs/engine-wire-contract.md` **§7.1** (the `message` contract rule) + that file's **V-15.1** binding note | the stale "**`message` is non-empty** for every render" clause is **superseded in place** by the F8-restated `p2` §5.5 wording: the four string-carrying variants carry their string **verbatim and it MAY be empty**, `UnsupportedSchemaVersion(v)` is the one **guaranteed-non-empty** message, and TestWriters MUST NOT assert a non-empty message for the four (the register — `P-TP-3`, §11's `P-TP-3` annotation — forbids it). One clause; V-15.1's "three" corrected to **four** string-carrying variants |
| **2** | MUST-FIX | `docs/decisions.md` (**3** clauses: the U1 status paragraph, the `ENGINE-DURABLE-CORPUS-DIRECTION` clause, the `SUBSYSTEM-FLAG-CAPABILITY-SEMANTICS` row); `docs/defects.md` (**2** clauses of the hard-coded-flags OPEN row); `docs/next-steps.md` (the U0 pass recording); `docs/pending.md` (header + intro + the GR-7 clause); `docs/HANDOFF.md` (status line, the 4 GR-row go-ahead cells, the FS-13/14/15 row, the addendum); `docs/specs/gnosis-gr-inbound-review.md` (a dated post-record note); this file's **§U1 status block** | every "**U2–U5 HELD / U3 not yet authorized**" clause is **annotated in place with its historical text retained**: **SUPERSEDED (2026-09-16) — U2/U3 ARE AUTHORIZED, their registers are authored in §9.5.1/§9.5.2, and only their code is owed; U4/U5 remain NOT authorized.** Each file's own summary now reads "**U2/U3 AUTHORIZED (registers authored); U4/U5 HELD**" (the `decisions.md` "U2–U5 held" clause is corrected the same way) |
| **3** | **BLOCKING** | `docs/specs/p2-gnosis-server-live-pending-battery.md` **§3.5 (new: rows `R-L1`/`R-L2`)** + this file's `P-TP-2` row/coverage note and §9.5.2's U3 note 1 + `P-IM-9`'s coverage note | the **two live obligations** that were "homed" in a battery document with **no U2/U3 rows** now have **real rows there, each with its own precondition and pass/fail criterion**: `R-L1` = with a READY engine, `topK`/`filters`/`mode` **visibly change** the `POST /rag/query` response (the `P-TP-2` β observable); `R-L2` = a **provider-reachable boot** reports `Ready` with `embedding:true`, `vector:false`, `reranker:false` (the `P-IM-9` live half). Both are cited as those rows' **homes**; the M1–M20 rows and the battery's own revisit condition are **not disturbed** |
| **4** | **BLOCKING** | §9.5.2's U3 scope/mechanism note, `P-IM-9`'s invariant + observable cell, §9.5.2's U3 note 1, §9.5.4's `P-IM-9` coverage note | the "applies **both** outputs" contradiction is resolved against the pinned read-time projection: **the boot applies only the `EngineState`** — plus its own wiring (the snapshot swap and the provider seam) — while the returned **`EngineSubsystems` is the DERIVED vector, written nowhere**, and exists as the **assertion surface**; the derived read is what the **boot wiring produces**, so `boot_wiring(...)`'s return and an independent `get_engine_status().subsystems` read are asserted to be **equal**. No row now instructs a write of the mask, and `set_subsystems` stays forbidden as an assertion vehicle (F11/F16) |
| **5** | MUST-FIX | §9.5.3's **seed pin** (tag convention) | the pre-remand `P<CLASS><N>` tag form **collided with the tags the same property binary already uses** for §11's seven landed rows (`tests/props_gnosis_server.rs:210-216` — the pre-U3 citation was `:172-178`; call sites `:675`/`:727`/`:777`/`:849`/`:902`/`:953`/`:1011` — the pre-U3 citation was `:595`/`:647`/`:697`/`:769`/`:822`/`:873`/`:931`; same `SEED` at `:147` — the pre-U3 citation was `:109`), so the twelve §9.5 tags are re-derived **unit-discriminated and injective**: U2 ⇒ `U2PIM4`/`U2PIM5`/`U2PIM6`/`U2PSM4`/`U2PTP2`/`U2PTP3`/`U2PTP4`, U3 ⇒ `U3PIM7`/`U3PIM8`/`U3PIM9`/`U3PSM5`/`U3PSM6`, **disjoint from the landed `PIM1`…`PTP1` set**, with that file's tag constants cited as the evidence (the pre-U2 citations `:112-118` / `:507-843` are stale) |
| **6** | MUST-FIX | §11's **UPDATE** sentence + §9.5.1's **consolidation record** | the coverage-completeness claim is corrected to the table's real count: **nine** U2 obligation rows are marked AUTHORED (§11's rows at the nine `AUTHORED ⇒` annotations), **eight** of them are the register notes' owed sketches at U1 and the **ninth** — the row the first register remand **added** — is the **`QueryAuditFilters`-touching `filters`-mapping `merge` row** (§11's row annotated *merged into `P-IM-6`*; the very row §9.5.1's consolidation record names as "the one row the first register remand added"), while the pre-remand sentence **under-counted these nine as "six" and omitted three** — the **`P-IM-5` wrongly-typed** row, the **`P-TP-2` split** row and the **`P-SM-4` parity** row — each of which §11 now annotates `AUTHORED` (the wrongly-typed row landed as `P-IM-5` itself, the split row's decode-level identity half as `P-TP-2`, the parity row merged into `P-SM-4`). **All three accounts state the same composition of nine**: §9.5.1's consolidation record ("**eight** named in the register-notes sketches at U1 **plus the one row the first register remand added**" = the `QueryAuditFilters`-touching mapping), §11's UPDATE sentence (the same eight + the same ninth, with the three omissions named), and this item. The U3 count stays **one** obligation row (its four further ids are **splits** of that one row) |

#### 9.5.4 Per-row generator-coverage notes (F14 — boundary + adversarial shapes; ≤5 lines per row, **or an explicit deviation note in the row**)

Mirroring §11's and the F2 register's coverage notes, one note per §9.5 row. Each note names the generator's
**boundary + adversarial input shapes** and the **excluded fail-state inputs**; a row without a note here
MUST NOT be generated. ***(U5 addition, 2026-09-22 — ADDITIVE:** the eight U5 notes at the end of this block
are **longer than the ≤5-line guideline** this heading carried, so — per the heading's rule as restated by
the REMAND-1 pass — **each of the eight carries its own explicit deviation note** (the marker
"***Format deviation (U5, REMAND-1):***" at the start of its line), which is the heading's stated
alternative to ≤5 lines. U5's rows quantify over a *corpus* (the store's
node set), a *provider-failure* axis and a *flag-derivation* equality, so their boundaries and exclusions cannot
be named inside five lines without becoming under-specified for the TestWriter. The guideline's **purpose**
(every generated row has its boundary + adversarial shapes and its excluded inputs named) is honoured; the
**U2/U3 notes above are not touched** and stay ≤5 lines.**)***

- **`P-IM-4 — strat:query-decode-total`.** Enumerate payloads `{}`, `{"query":"x"}`, one key at a time, then random key subsets; assert `Ok` + §5.3's mapped value per key. Boundary: `schemaVersion` ∈ `{0, 1, 2, u32::MAX}` (an unknown ⇒ `UnsupportedSchemaVersion`, never a partial decode), `idFormat` ∈ `{"opaque-string-v1", "", "uuid-v4", "OPAQUE-STRING-V1"}`, and an envelope with **both** unknown ⇒ `UnsupportedSchemaVersion` first. Adversarial: **non-object payloads** `5`/`null`/`"hi"`/`[]`/`true` (⇒ `InvalidEnvelope` for each — never `InvalidJson`), a bare body, `{"query":"x","args":{}}`/`"method"`/`"requester"` (extra keys change nothing). **Excluded inputs:** nothing per-key here — the wrong-type corpus is `P-IM-5`'s, `filters` is `P-IM-6`'s, token strings are `P-SM-4`'s.
- **`P-IM-5 — strat:query-option-defaults`.** For each of §5.3's 14 keys: absent, well-typed, and the wrong-type corpus (`topK:"10"`, **`topK:-1`/`topK:10.5`** (`as_u64` does not represent them ⇒ absent ⇒ default, **no** 400 — §5.3's type-vs-range bullet), `hyde:"yes"`, `wikiId:5`, `maxHops:null`, `binaryFirstPass:1`, `subTaskDag:5`, `multiQuery:5`, `compression:null`, `expand:5`, `mode:5`/`null`/`{}`), plus the **adversarial object-member misuse** instances `{"multiQuery":{"enabled":"yes"}}`, `{"multiQuery":{"enabled":true,"n":"3"}}`, `{"subTaskDag":{"enabled":"yes"}}` ⇒ the decoded **field is `None` at layer 1** (`multi_query == None`, `sub_task_dag == None` — a wrongly-typed *documented member* belongs to §5.3's wrongly-typed rule, so the decoder never answers with a present object carrying a member-level default; **the member check is total over the declared members**, `enabled` bool + `n` u64 — ruling 2, the clause stands and the code implements it, defect `P-6` closed), and nothing else changes (§5.3's two-layer bullet, object-valued-option clause); `Ok` either way, never an `Err`. Boundary: `{"multiQuery":{}}` ⇒ `Some(MultiQueryOptions{ enabled:false, n:3 })` **and** `{"multiQuery":{"enabled":true}}` ⇒ `Some(MultiQueryOptions{ enabled:true, n:3 })` (the member-level defaults apply **only inside a well-formed object**, `n == 3`); `{"subTaskDag":{}}` ⇒ `Some(SubTaskDagOptions{ enabled:false })`; `topK:42` ⇒ `Some(42)`; `wikiId:"w1"` ⇒ `Some(WikiId("w1"))`. SSE-path sampling uses **only** `query`/`topK`/`mode` (`sse_params`; string-or-absent). **Excluded fail-state inputs:** empty/absent `query`, out-of-range `topK`/`maxHops`/`multiQuery.n`/`binaryCandidatePool` (**the RANGE half — a present `u64` only**; the **TYPE** half, `topK:-1`/`10.5`/`"10"`, is this row's own §5.3 rule and MUST NOT be counted as a 400 — §5.3's type-vs-range bullet, ruling 1, 2026-09-16), unrecognized token **strings**, malformed `filters` — each is another row's 400.
- **`P-IM-6 — strat:filters-mapping`.** Every subset of the four members (16), plus `null`, `{}`, non-objects (`[]`/`"x"`/`5`). Boundary: `{"filters":{}}` ⇒ `Some(all-None)` (present/valid); the store's own **value** casing `{"state":"Fresh"}`/`{"nodeKind":"Content"}` ⇒ the same `ReferenceState::Fresh`/`NodeKind::Content` the canonical `"FRESH"`/`"content"` yields. Adversarial — **two distinct negatives, never one**: `{"nodeKind":"community"}`/`{"edgeType":"docHead"}`/`{"edgeType":"member"}` (out-of-set token), wrongly-typed members `{"nodeKind":5}`/`{"state":5}`/`{"target":"d1"}`, a `target` that is not an object (`["d1","n1"]`) or lacks `nodeId`/`documentId` ⇒ **`Err(Validation)`** — **the store's serde shape `{"node_kind":"Content","target":["d1","n1"]}` is a SHAPE MISMATCH ⇒ `Err(Validation)`** (this is the **D3** negative: the camelCase/shape pass-through guard, per §5.3 + the row; never `Some(all-None)`); by contrast a **genuinely unknown member NAME with a valid value** `{"nodekind":"content"}` ⇒ `Ok(Some(all-None))`, and `{"bogus":1}` ⇒ `Ok(Some(all-None))`, while `{"nodeKind":"community"}` ⇒ `Err` and `{}` ⇒ `Ok(Some(all-None))`. **Excluded:** the padded-token boundary (`"content "`) — no row pins trimming for `filters` tokens, so a generator MUST NOT assert it.
- **`P-SM-4 — strat:token-resolver`.** All ASCII casings of each vocabulary member (e.g. the 16 casings of a 4-letter token) ⇒ the typed enum. Boundary: `""`, `" "`, `"\t"`, `"bm25"`, `"flat "`, `"flat\n"`, `" hybrid"`, `"flat\tflat"` ⇒ `Err(Validation)`; absent ⇒ `Ok(Flat)`. Adversarial: **non-string** values (`5`/`null`/`{}`/`[]`) ⇒ absent ⇒ default, resolver **not invoked**; a request with an unrecognized `mode` **and** `topK:0` ⇒ the token's error (step 2 before step 3). Wiring: the three live assertions (i)/(ii)/(iii) of the row's observable, plus `?expand=parent`/`?compression=nope` ⇒ non-400. **Excluded:** any SSE `expand`/`compression` token assertion (those params do not exist — N1).
- **`P-TP-2 — strat:query-options-identity`.** Generate well-formed payloads (each key present/absent/well-typed) and compare the whole `RagQueryOptions` value element-wise; assert `query` verbatim (including empty-after-trim vs whitespace-only strings — the *value* is copied, the FS-3 reject is the engine's). Boundary: every key at once; `multiQuery.n` omitted ⇒ 3; `requester` present ⇒ `None`. Adversarial: the decode-level identity is asserted for an unknown key added (`args`, `method`, `requester`). **(α)** live: `{"query":"x","mode":"bm25"}` ⇒ 400 `validation_error`; **(β)** live battery: `topK`/`filters`/`mode` change the response (the battery's **named row `R-L1`**, §3.5 — that row is this obligation's home). **Excluded:** per-option *consumption* claims (`subTaskDag` inert; `binaryFirstPass`/`binaryCandidatePool` vector-leg-only) — the row MUST NOT assert an effect.
- **`P-TP-3 — strat:transport-code-total`.** All 5 request-decode variants (each once) plus the 5 out-of-domain variants × `None`, and a random `StoreError` sample for `wire_code()` disjointness. Boundary: `UnknownMethod("")`, `UnknownMethod("bogus")`, `InvalidEnvelope("")`/`InvalidJson("")` (an **empty** carried string is legal and does not make the code empty). Adversarial: assert `request_decode_code` and `request_decode_status` are defined on **exactly** the same 5 inputs (no asymmetry). **Moved out (F8):** the rendered body/`Content-Type` bytes (the `v15_request_decode_error_body_exact` golden + V-15/V-15.1), and any "message non-empty" assertion for the string-carrying variants.
- **`P-TP-4 — strat:encode-result-checked`.** Well-formed `RagResult`s from F2's `P-TP-1` corpus across the modes, round-tripped through `encode_result_checked` → `decode_rag_result` → `validate_rag_result`. Boundary: an empty `items` list, a maximal trace, `blocked_by` with a graph trace, `parent: None`. Adversarial (the `Err` half — **exactly the two representable rejected classes**, matching the executed corpus): an `engine` not equal to `"gnosis"` — including the **empty string** `""` (the `!= "gnosis"` arm, `src/wire/decode.rs:64-66`), the optional widening the adversarial pass asked for — and a `blocked_by` present without a graph trace ⇒ `Err(EngineError)` + `server_status` 502. **A traceless result is NOT representable in Rust** (`RagResult.trace: RagTrace` is non-optional, `src/store/mod.rs:758-765`), so it is **not** a coverage gap and MUST NOT be counted as a rejected class. **Excluded:** asserting the `Err` half as a live HTTP fail-state (it is a pure U2-time totality requirement).
- **`P-IM-7 — strat:flag-truth-capability`.** Sample store states: brand-new (`vectors:None`, no provider), after `swap_snapshot` with/without vectors, after `set_embedding_provider` present/absent, and a `set_subsystems(<all-true>)` write (asserting the read is **unchanged** — the F16 write-independence probe). Boundary: all four `EngineState`s crossed with provider present/absent. Adversarial: a deliberately injected false mask MUST NOT flip any flag. **Excluded:** any state produced only by a verbatim mask write.
- **`P-IM-8 — strat:embedding-flag`.** The two pinned instances `{Degraded, provider:None}` and `{Unavailable, provider:None}` plus a wired-then-unreachable provider (`Some` present, state `Degraded`) ⇒ `embedding:true` with `last_error` carrying the reason. Boundary: provider present ⇒ `true`; provider absent in every state ⇒ `false`; the `{Degraded, None}` instance additionally pins `vector:false`/`reranker:false`. Adversarial: assert the flag tracks the **provider seam**, not the state (`Degraded` + `Some` ⇒ `true`).
- **`P-IM-9 — strat:boot-flag-honesty`.** The three `BootProvider` inputs (`Absent` / `Unreachable` / `Reachable(p)`) with the snapshot both traceless and vector-bearing; assert the returned `(state, flag-vector)` pair **and an independent derived read** of the store's own flag vector after applying **only the returned `EngineState`** plus the boot's own wiring (never a `set_subsystems` write), comparing the two side-by-side (the pair is the assertion surface; the derived read is what the store reports). Boundary: `Reachable` + `vectors:Some` (U5-time) ⇒ `embedding:true, vector:true`; the U3-time `vectors:None` ⇒ `vector:false`; `Absent` ⇒ `Unavailable`; `Unreachable` ⇒ `Degraded`. Adversarial: assert the lib half does **not** write through `set_subsystems`, and that the provider-**absent** boot makes no `embedding` claim (the e2e `engine_status_reports_no_false_embedding_claim`). **Excluded (live-only, named):** the provider-**reachable** *live server* outcome (`Ready` + `embedding:true` from `GET /engine/status`) — it needs a controlled provider and lives in the live battery's **named row `R-L2`** (`docs/specs/p2-gnosis-server-live-pending-battery.md` §3.5 — that row is this obligation's home).
- **`P-SM-5 — strat:status-pure-read`.** Repeated reads in every state/mask combination; assert element-wise equality and that `epoch()`/`journal_len()`/the snapshot `Arc` are untouched. Boundary: read after a `Ready`/`Degraded`/`Unavailable`/`Starting` transition; read twice with a wired provider. Adversarial: interleave an unrelated read (`snapshot()`, `embedding_provider()`) — no observable change. **Adversarial (as landed — the mutation-interleaved positive control the TestWriter added; not a weakening of any assertion above):** the row also reads the status, then **mutates between the two reads** — `swap_snapshot(Some(empty index))`, `set_embedding_provider`, `set_engine_state`, and a journal-appending call — and asserts (i) the two reads differ **only in the licensed positions** (the flag(s) whose capability witness the mutation changed, and `state`/`last_error` — never `version`/`schemaVersion`/`idFormat`, and never a flag whose predicate is unchanged) **and** (ii) `epoch()`/`journal_len()` **did** move *because of* the mutation — which is what makes the purity half falsifiable rather than vacuous (a pure read is proven by the mutation moving observables while the read itself moves none). **Excluded:** any assertion that a read *should* mutate state.
- ***(U6 in-place marker on the U3 row whose vector predicate U6 supersedes — REMAND-2 MUST-FIX 2, 2026-09-22 (docs-only, ADDITIVE; the §9.5.2 register cell and the §9.5.4 note keep their text as the U3-time record). The anchors are clause-name-first — the row id and the note heading — because the numbers shift; the `:2176`-class line spelling this file previously carried for `P-IM-7`'s cell is **superseded by text** and is dropped here.)***
> **The `P-IM-7` register cell (§9.5.2, the row whose `strat:` id is `strat:flag-truth-capability`).** Its `vector` half, `f.vector == s.snapshot().vectors.is_some()`, and its prose half *"`vector` is `false` exactly while the current snapshot has no vector index"* are the **pre-U6** predicate and are **superseded in place**: after U6 the equality is `f.vector == (s.snapshot().vectors.is_some() && s.snapshot().epoch == s.epoch())`, read through **one** snapshot clone before the status read (§9.5.6's surface #3 and `P-IM-16`'s observable). Its `store`/`graph`/`lexical`/`reranker`/`embedding` halves, its capability-semantics invariant and its `set_subsystems(<all-true>)` write-independence half are **unchanged** — the epoch term is the only edit.
> **The `P-IM-7` §9.5.4 coverage note (the note headed `` `P-IM-7 — strat:flag-truth-capability` `` in §9.5.4).** Its sampled-state sentence (*"brand-new (`vectors:None`, no provider), after `swap_snapshot` with/without vectors, …"*) gains the **epoch axis** the amended row now ranges over: a fixture that swaps an **index-bearing** snapshot must align its `epoch` with the store it feeds, or the row's own boundary instance (an index-bearing snapshot with `epoch != store.epoch()` ⇒ `vector:false`) is what it asserts. Its boundary/adversarial halves (all four `EngineState`s × provider present/absent; the injected false mask changing nothing) and its Excluded half **stand**. **The U3 fixtures' own alignment is verified in §9.5.6's same-unit table** (the U3 `props` fixtures run on a fresh store at epoch `0`; the one named exception is `P-SM-5`'s mutation control).
- **`P-IM-16 — strat:vector-freshness-capability`.** ***Format deviation (U6, 2026-09-22 — the six U6 notes at the end of this block follow §9.5.5's precedent: they exceed the ≤5-line guideline, so each carries this explicit deviation note, which is the heading's stated alternative; the U2/U3/U5 notes above are not touched and stay ≤5 lines.*** Sample the two axes the predicate ranges over: the snapshot's `vectors` (`None` / `Some(empty VectorIndex)` / `Some(populated)`) × the epoch pair (`snap.epoch == store.epoch()` / `<` / `>`) on stores built from `Store::new()` (epoch `0`) and from a seeded corpus (epoch `n > 0` via `create_wiki`+`create_document`+`update_document`, §9.5.5's corpus-seeding clause — all three append, so the epoch witness is derivable). Assert the `==`-element-wise equality `f == (snap.vectors.is_some() && snap.epoch == s.epoch())` **on ONE bound clone** (`let snap = s.snapshot();` **before** the status read — REMAND-1, 2026-09-22: the pre-remand note wrote `snapshot()` twice, which can compare two different snapshots under a racing swap) for each combination, with the four boundary instances named in the row (`Some`+aligned ⇒ true **including the empty index**; mismatched ⇒ false; `None` at any epoch ⇒ false; `epoch >` ⇒ false). Boundary: a `swap_snapshot` of a snapshot with a **zero-length** entries map; an epoch **equal** (the fresh case); the store's epoch moved by **one** entry (the minimal staleness). Adversarial: the write-independence probe — `set_subsystems(<all-true>)` then re-read ⇒ **unchanged**; an injected false mask MUST NOT flip any flag; and a state produced **only** by such a write is **excluded** as an input (it proves the hook, not the derivation). **Excluded:** any assertion about *why* the epoch moved (a corpus mutation and a non-corpus mutation are the **same** input to this row — the over-approximation is stated in §9.5.6's cost clause, not asserted here); and any provider-axis claim (that is `P-IM-8`'s).
- **`P-IM-17 — strat:stale-index-fail-loud`.** ***Format deviation (U6, 2026-09-22):*** this note exceeds ≤5 lines; see the block note above. The stale instance across the four shapes that bound it: a `Ready` store + a **wired** provider + an index-bearing snapshot with `snap.epoch != store.epoch()` (the primary case), the same with **no** provider (still FS-14, not FS-13), the same left **non-READY** (FS-8 wins), and `mode=hybrid` on the stale state (serves, vector leg empty, flag `false`). **Every generated instance must go through a site pinned by §9.5.6's per-site consultation clause (REMAND-1, 2026-09-22): the row's `rag_query` half is the `vector_query` site (refuse ON), and the row's shapes must also cover the direct `vector_search` site (refuse ON ⇒ the same FS-14) and the fusion site (refuse SUPPRESSED ⇒ empty leg + `Ok`) — the pre-remand note quantified the stale outcome over `rag_query` only and left `vector_search`'s stale outcome unstated.** Assert the pair (status flag `false` **and** `Err(VectorIndexUnavailable)` ⇒ `server_status == Some((503, "vector_index_unavailable"))`), the `never Ok` half, the injected provider's `embed` **call count == 0**, and the counter-witness that the **fresh** state serves `Ok`. Boundary: staleness by `+1` and by a large epoch gap; an **empty** stale index (refused like any stale index — the flag is not about entry count); a store whose snapshot is stale **and** whose state is `Degraded` (FS-8). Adversarial: the divergence probe — a state where the flag is `false` for staleness but the leg returns `Ok` is the **defect** (the row must fail on it, not accept it); and the negative probe that the row is not satisfied vacuously: a run that answers FS-14 for **every** state (including the fresh one) fails the counter-witness. **Excluded:** any assertion on a `snippet`'s content (the honest boundary of §9.5.6's `node_snippet` clause); any claim that a fresh index covers the whole corpus; and any new wire code/variant/route (none exists).
- **`P-IM-18 — strat:boot-epoch-alignment`.** ***Format deviation (U6, 2026-09-22):*** this note exceeds ≤5 lines; see the block note above. The three `BootProvider` inputs × {empty store, populated store} × {all-`Ok` provider, fail-on-*k*-th provider}: read `store.epoch()` before the build, assert the build leaves it unchanged, assert the **composed** snapshot (the boot's own composition recipe, in both branches) carries that epoch, then apply the pinned wiring and assert `subsystems.vector == snap.vectors.is_some()`. Boundary: an **empty store** (epoch `0`, so a `DerivedIndexes::default()` composition happens to align — the row must not be read as "the boot is correct because the store is empty": the populated case is the witness); a store whose epoch is `n > 0` (where the missing epoch term is **visible**: `vector:false` on a READY boot); the failed-build branch (index-free snapshot, so the epoch term is unreachable — assert it still composes with `store.epoch()` so the term is present, and that the flags are the `P-IM-15` literal). Adversarial: the **negative probe** — compose with `epoch: 0` (the `DerivedIndexes::default()` label) on a store at epoch `n > 0` ⇒ the row must FAIL (the boot's own index reading as stale); and assert the build itself never reads/stamps the epoch (the `P-IM-12`/contract-table-(2) guarantee: two builds and a snapshot composed **after** them both carry the caller's epoch, not the build's). **Excluded:** any live/ambient provider; any claim about a **rebuild** path (U6 adds none); and any assertion that a boot-time index is **complete** (that is `P-IM-11`'s corpus claim).
- **`P-IM-19 — strat:build-failure-diagnostic`.** ***Format deviation (U6, 2026-09-22):*** this note exceeds ≤5 lines; see the block note above. The failing-build axis (fail-on-*k*-th, `k ∈ {1, n}`, plus always-`Err`) × the two no-diagnostic outcomes (`Ok(None)` — no provider; `Ok(Some(_))` — all-`Ok`): assert the marker `gnosis-server: boot vector-index build failed: ` **appears exactly for the failing case** and is **absent** for the other two, **observed through the SPAWNED BIN's stderr — the row's single pinned home (REMAND-2 SHOULD-FIX 5, 2026-09-22: the layer classification's one bin-level row and this note now name the same observable; a lib-boundary observation of the same line is an explicitly non-pinning extra, never a substitute)**, and that the status surface beside it is the pinned failure-branch literal element-wise (`Degraded`, `vector:false`, `embedding:true`, `reranker:false`, core `true`, the store's own fixed `last_error` — byte-identical). Boundary: an `Err` on the **first** call and on the **last**; a build over an **empty** corpus with a supplied provider (`Ok(Some(empty))` ⇒ **no** diagnostic — the no-over-logging boundary); an `Err` whose carried text is the existing `EmbeddingUnavailable` rendering. Adversarial: the order probe — the line must be observable **without** depending on the composition's shape (i.e. the diagnostic is not nested inside the `ok().flatten()` expression or after it); and the no-reword probe — no U6 change to the `last_error` sentence (a reworded string fails this row **and** `tests/wire_conformance.rs:1109`'s golden). **Excluded:** any assertion on a **log level**, a logger name, a timestamp, a module path or an ANSI prefix (no logging crate is added and none is pinned); any claim about the diagnostic's content beyond the marker + the error's own display; and any **second** line's content.
- **`P-SM-8 — strat:freshness-pure-read`.** ***Format deviation (U6, 2026-09-22):*** this note exceeds ≤5 lines; see the block note above. Repeated status reads in every state of `P-IM-16`'s axes (aligned, stale, unbuilt, all four `EngineState`s, provider present/absent), plus the interleave of unrelated read-only calls (`snapshot()`, `embedding_provider()`, `epoch()`, `journal_len()`): assert element-wise equality of the two reads, that `epoch()`/`journal_len()` did not move, that the snapshot's `Arc` identity is stable (`Arc::ptr_eq`), and that `snapshot().vectors.is_some()` is unchanged. Boundary: a read immediately after a state transition; a read on a store whose snapshot is **stale** (the freshness term is evaluated without repairing anything — the read must **not** rebuild). Adversarial (the positive control, `P-SM-5`'s probe-4 precedent): read ⇒ **mutate** (a public-`RagStore` corpus mutation, and separately a `swap_snapshot` of a differently-epoched snapshot) ⇒ read, and assert the pair differs **only** in the licensed positions (the flags whose witness moved, `state`/`last_error`) while the observers **did** move — which is what makes the purity half falsifiable. **Excluded:** any assertion that a read *should* mutate or repair state; any wall-clock claim; and any concurrent-swap race assertion beyond the single-clone pin of §9.5.6's surface #3 (the torn-snapshot concurrency tests live in `tests/rag_query_integration.rs` and are not this row's).
- **`P-TP-6 — strat:provider-request-bound`.** ***Format deviation (U6, 2026-09-22):*** this note exceeds ≤5 lines; see the block note above. The bound is asserted over the **client construction** (no sleeping): the provider's client is built through the **lib-visible seam** of §9.5.6's surface #6 — `gnosis::provider_client(gnosis::PROVIDER_REQUEST_TIMEOUT)` (`src/bin/gnosis_server.rs:327` is the bin's only construction site) — whose pinned value is `30 s`, and the same single client serves **both** calls (the probe and `embed`) — so the assertion is (i) the seam's pinned value (`PROVIDER_REQUEST_TIMEOUT == Duration::from_secs(30)`), (ii) the duration is **falsifiably wired** (`provider_client(Duration::from_millis(50))`'s request to a controlled never-answering endpoint returns a transport `Err` with `is_timeout()` true inside the test's own bound), and (iii) the **closure** counter-witness that the bin's configuration reads are exactly three (`--port`, `GNOSIS_SERVER_OLLAMA_URL`, `GNOSIS_SERVER_OLLAMA_MODEL`). **REMAND-1, 2026-09-22: the pre-remand note asserted "the constructor's presence and its pinned value" against a `const` inside the private `[[bin]]` `OllamaProvider` — unreachable from a lib row, which is why surface #6 exists; the bin-level alternative (a real `30 s` spawn observation) is rejected there and in the row.** Boundary: a timeout **exactly** at the pinned constant (assert the constant, never a `30 s` wall-clock measurement); a probe timeout and an `embed` timeout as **two instances of one bound** (the phase consequences are `P-TP-6`'s observable-cell table: `Unreachable` / the failure branch / FS-13). **The `50 ms` seam witness is NOT a live-provider measurement** (REMAND-1, 2026-09-22): its endpoint is a test-controlled `TcpListener`/`wiremock` that accepts and never answers, and the duration is the **test's own** — no ambient provider, no `30 s` wait. Adversarial: the timeout-to-error mapping probe — a timed-out request must yield `StoreError::EmbeddingUnavailable` and `server_status == Some((503, "embedding_unavailable"))`, **never** a new variant or a 5xx the §11 map does not carry; and the **no-knob probe** — no env var or CLI flag is read for the timeout (a test that sets a hypothetical timeout env var must observe **no** change, which is the closed-set assertion). **Excluded:** any live/ambient-provider timeout **measurement** (the unit's hermeticity rule forbids a live provider in a U6 test); any claim about a **total** build budget or a node cap (the narrowed residual in §9.5.6's reconciliation table 2); and any assertion about the provider's internal behaviour beyond the bound.

- **`P-IM-10 — strat:boot-index-build`.** ***Format deviation (U5, REMAND-1):*** this note exceeds ≤5 lines; the heading's rule permits it only with this note, which is why it is here. The two provider inputs (`None` / `Some(&pr)`) crossed with the corpus shapes `{}`, `{1 node}`, `{n nodes}`, plus two boundaries from this unit's own register (`Some("")`-valued nodes; a duplicate text under two ids) and the always-`Err` provider. Assert the **three-valued** result directly: `None` ⇒ `Ok(None)` with the injected provider's `embed` **call count 0**; `Some(&pr)` + all-`Ok` ⇒ `Ok(Some(vi))` with `vi.entries.len() == embeddable_count`; `Some(&pr)` + any `Err` ⇒ `Err(EmbeddingUnavailable)`; an empty store under `Some(&pr)` ⇒ `Ok(Some(vi))` with `entries.is_empty()` — **never** `Ok(None)`. Boundary: 0/1/`n` embeddable nodes. Adversarial: assert the **`None` case is NOT an error** (an implementation answering `Err` for it fails the row) and that a provider whose `is_available` lies is never consulted (the build makes **no** probe call). **Excluded:** any live/ambient provider, any `is_available`/env-var dependency, and anything about *where* the caller installs the returned index (that is `P-IM-14`/`P-SM-7`).
- **`P-IM-11 — strat:index-corpus-full-field`.** ***Format deviation (U5, REMAND-1):*** this note exceeds ≤5 lines (see the block note above). Build over the corpus shapes `{}`, `{1 node}`, `{n nodes}`, a store with `value: None` nodes mixed in, and a store whose shards are populated out of id order; then compose the snapshot (`Some(vi)` / `DerivedIndexes::default()`) and assert `boot_wiring(p, &snap).1.vector == snap.vectors.is_some()` for each of the three `BootProvider` inputs. Boundary: `Reachable` + an **empty** store ⇒ `vector:true` (the empty index is still `Some`); `Absent`/`Unreachable` ⇒ `vector:false` **and** the provider's `embed` call count is **0**; every key's field is `Full` (a generator MUST NOT assert a `Binary` key exists — none is built). Adversarial: assert the keys carry the **source** ids (a build keyed by snippet text or by position fails), and that a node with `value: None` produces no key while `value: Some("")` produces one. **Excluded:** any wiki filter (the corpus is store-wide, wiki-scoping happens at read time — `document_wiki_id`) and any `LexicalIndex` claim (U5 builds none).
- **`P-IM-12 — strat:index-build-stability`.** ***Format deviation (U5, REMAND-1):*** this note exceeds ≤5 lines (see the block note above). Two builds over the same store state and provider: assert the returned values are **element-wise equal** (same variant; in the `Ok(Some(_))` case the same key set and the same `Vec<f32>` per key), and that a build does **not** mutate the store's observable state (`epoch()`, `journal_len()`, the `snapshot()` `Arc` identity and the wired provider unchanged across the call). Boundary: 0, 1 and `n` embeddable nodes; a store whose shards are populated out of id order. Adversarial: a provider whose `embed` returns a constant for every text ⇒ still equal across two builds (proves the key ordering, not the vectors' content); the calls must be made **without** `swap_snapshot` in between, so only the build's own behaviour is measured. **Excluded:** any assertion about the **store's** wall-clock, and any `Ok`-vs-`Err` **outcome** count claim (that is `P-IM-13`'s atomicity / `P-TP-5`'s totality territory; the `embed` **call** count is **not** excluded — it is the contract table (2)'s call-count row, asserted here as the two-build stability case).
- **`P-IM-13 — strat:index-build-atomicity`.** ***Format deviation (U5, REMAND-1):*** this note exceeds ≤5 lines (see the block note above). A provider that fails on the *k*-th `embed` call (`k ∈ {1, 2, n}`): assert `Err(EmbeddingUnavailable)` — **not** `Ok(Some(partial))` and **not** `Ok(None)` — and that **no** partially-filled index is observable (the store's `snapshot().vectors.is_none()` after the caller swaps the *unchanged* snapshot); then assert that a **successful** retry produces a **complete** index (the count witness of `P-IM-11`), so the failure is not sticky. Boundary: `k = 1` (fail on the first call), `k = n` (fail on the last), and a provider that fails **once** then succeeds. Adversarial: the partial-index negative probe — an implementation that inserted entries as it went and returned `Ok(Some(_))` would be caught by the `k`-th-failure case **and** by the count witness. **Excluded:** any new wire code, `StoreError` variant or §11 row (none exists for a boot-build failure — §9.5.5's "no invention" clause); the row asserts the **existing** `EmbeddingUnavailable` family and the no-index observable only.
- **`P-IM-15 — strat:boot-index-failure`.** ***Format deviation (U5, REMAND-1):*** this note exceeds ≤5 lines (see the block note above). The fail-on-*k*-th provider (`k ∈ {1, 2, n}`) plus the always-`Err` provider: assert `Err`, then apply the boot's documented failure outcome and read the status **element-wise** against the §9.5.5 failure-table literal (`Degraded`, `vector:false`, `embedding:true`, `reranker:false`, core `true`, the store's own `last_error` string). Boundary: `k = 1` and `k = n`; a store with a **populated** corpus (so a wrong implementation that kept a partial index visibly flips `vector`). Adversarial: assert the state is **not** `Ready` (a fabricated `Ready` fails the row) and that **no** `StoreError` outside the existing family is produced (a new variant or a §11 row would be an invention — §9.5.5's no-invention clause). **Excluded:** any claim about a *retry* mechanism (no rebuild vehicle exists in U5) and any wording change to the fixed `last_error`.
- **`P-IM-14 — strat:vector-flag-flip`.** ***Format deviation (U5, REMAND-1):*** this note exceeds ≤5 lines (see the block note above). For each boot outcome: after the pinned wiring path (`boot_wiring`'s returned `EngineState` applied, the built snapshot swapped, the provider wired **only** in the `Reachable` case, **never** `set_subsystems`), `get_engine_status().subsystems.vector == store.snapshot().vectors.is_some()` — **exactly**, and the V-8.1 fixture's `vector` value equals what the `Reachable` path derives. Boundary: an **empty** index (`VectorIndex::default()`) ⇒ `vector: true`; `Absent`/`Unreachable` ⇒ `vector: false` with the `V-8.2` fixture unchanged; `reranker` stays `false` and `embedding == (a provider was wired)` in every case (the U3 rows' own predicates are **not** re-scoped, §9.5.5's note 6). Adversarial: a **false-mask** write through `set_subsystems` before the read must change **nothing** (the `P-IM-7` write-independence probe, reused — never a vehicle for this row's claim). **Excluded:** any claim that a **READY** state implies `vector: true` (F13's readiness-vs-capability axis, §9.5.2's `P-IM-9` note 1 — a provider-reachable boot over an empty store that built an empty index is `vector:true`, while a `Reachable` boot handed a snapshot with `vectors: None` (no build attempted, or a caller-built snapshot) is `Ready` + `vector:false` by the same predicate — **REMAND-2 SHOULD-FIX 5 (2026-09-22): this parenthetical previously read "…is `Ready` + `vector:false` only if it also had no reachable provider, which is `Degraded`/`Unavailable` by §9.5.2's table", which contradicted this row's own invariant and §5.8's REMAND-1 pin — the corrected wording above is §5.8's (the `Reachable` + `vectors: None` instance), and V-8.1 is asserted **only** for `Reachable` + `vectors: Some`**); the row asserts the **derivation equality only**). **Boundary (PINNED — §9.5.5(2)'s *vector comparison / `NaN`* row, added by the post-red-phase register amendment 2026-09-22):** the row's "identical `Vec<f32>` per key" is compared as **same `len()` + every non-`NaN` element identical bit-for-bit + `NaN` asserted only in the `NaN` position** — never a bare `==` over a `NaN`-bearing vector.dentical bit-for-bit + `NaN` asserted only in the `NaN` position** (`a[i].is_nan() == b[i].is_nan()`), with **no** tolerance/`approx` comparison — so a `NaN`-bearing provider output is **not** a counterexample to this row.
- **`P-SM-7 — strat:boot-lifecycle-vector`.** ***Format deviation (U5, REMAND-1):*** this note exceeds ≤5 lines (see the block note above). The post-boot lifecycle transition: for a **`Reachable`** boot (`Ready` applied + the built index swapped + the provider wired), a `mode=vector` request succeeds (`Ok(RagResult)` with `RagTrace::Vector`, `engine == "gnosis"`, `results.len() <= top_k`, over the wiki-scoped full-field entries), and for a boot with **no** index (`Absent`/`Unreachable` — a **non-READY** store) the same request is `Err(StoreError::EngineUnavailable)` (FS-8, the pre-READY gate firing before any leg check), **not** `VectorIndexUnavailable`: FS-14's `VectorIndexUnavailable` is asserted **only** on a **READY** store with `vectors: None` (the same snapshots **plus** `set_engine_state(EngineState::Ready)`, or the explicitly named caller-built READY store) — **REMAND-2 MUST-FIX 1, 2026-09-22: the pre-remand wording of this note was `Absent`/`Unreachable` ⇒ `VectorIndexUnavailable` and is superseded in place; the row's invariant and observable above carry the corrected reading** — i.e. the flag and the leg's outcome are **consistent** rather than contradictory. Boundary: an **empty** index ⇒ `Ok` with an **empty** `results` (never an error, never a panic); `hyde: true` with a reachable provider ⇒ still `Ok`; a store whose documents own no nodes ⇒ `Ok` + empty. Adversarial: the order pin — on a **READY** store with an **unbuilt** index and **no** provider the error is `VectorIndexUnavailable`, not `EmbeddingUnavailable` (`src/store/mod.rs:4448-4454`); with a built index and **no** provider it is `EmbeddingUnavailable` (FS-13); and the **call-count half** (contract table (2)'s call-count row): on the FS-8/FS-14 non-serving paths the provider's `embed` **call count == 0** — **REMAND-2 SHOULD-FIX 8 (2026-09-22): this row is one of the rows that carry the "no `embed` call for a `value:None` node / no call on a path that never embeds" instance, so the contract table (2) call-count row's citation names it explicitly.** **Excluded (live-only, named):** the same sequence through the **running bin** — it needs a controlled provider and lives in the live battery's **new row `R-L3`** (both the serving half and, per REMAND-1, the failed-build half (v)); and any claim about `mode=hybrid` (the vector leg degrades there, §5.9 — not this row).
- **`P-TP-5 — strat:boot-index-adversarial`.** ***Format deviation (U5, REMAND-1):*** this note exceeds ≤5 lines (see the block note above). The build is **total** over the adversarial provider shapes: an `embed` returning a **different dimension** than a previous call, an **empty** `Vec<f32>`, a **zero** vector, one carrying `NaN`/`±∞`, a provider whose `embed` always errors, and one whose `is_available` lies (`true` while `embed` errors) ⇒ the build either returns `Ok(Some(vi))` whose entries are **exactly** the built corpus (no key dropped, no extra key) or `Err(StoreError::EmbeddingUnavailable)` — **never** a panic, never a partially-filled index presented as `Ok(Some(_))`, never a new error type, and never `Ok(None)` for a supplied provider. Boundary: `text` values of length 0 (a node whose `value` is `Some("")` is **embeddable** — the contract embeds it; an absent `value` is **not**), multi-codepoint texts (the build passes them through verbatim — no truncation is pinned, so a generator MUST NOT assert one). Adversarial: assert the key is `(documentId, nodeId, FieldType::Full)` with the **source** ids (a build that keyed by index position or by snippet text fails), and that a **duplicate** node text under two ids yields **two** keys (the cache/dedup question is not this row's, and no row asserts a cache). **Adversarial — the `NaN`/`±∞` shapes' comparison rule (PINNED — §9.5.5(2)'s *vector comparison / `NaN`* row, added by the post-red-phase register amendment 2026-09-22):** because this row's corpus **contains `NaN`/`±∞` by design**, its "the returned `Vec<f32>` is the provider's own value **verbatim** (same length, same elements)" claim is asserted as **same `len()` + every non-`NaN` element identical bit-for-bit + `NaN` asserted only in the `NaN` position** (`a[i].is_nan() == b[i].is_nan()` element-wise) — **never** a bare `Vec<f32> == Vec<f32>` over a `NaN`-bearing vector (IEEE-754 makes `NaN != NaN`), and **`NaN` MUST NOT be filtered out of the generator** to avoid the issue; `±∞` **is** comparable and is asserted like any other non-`NaN` element. **Excluded:** any assertion about *live* provider behaviour (timeouts, HTTP status codes, model drift) — those are §9.5.5's residual-risk items, not invariants; any multi-megabyte stress input (no size bound is pinned); and any `HyDEGenerationFailed`-style variant (**§5.9's `HyDEGenerationFailed`-unreachable clause, `:1617-1620`; re-pointed by REMAND-4 MUST-FIX 2, 2026-09-22 — the pre-REMAND-4 `§5.9: reserved, unreachable` spelling is superseded, since §5.9's reserved-variant statement is that clause**).

- ***(U6 in-place marker on the two held rows whose vector predicate U6 supersedes — REMAND-2 MUST-FIX 2, 2026-09-22 (docs-only, ADDITIVE; the two §9.5.4 notes above and the two register cells they cover keep their text as the U5-time record, and this marker is the dated annotation the same-unit table and the reconciliation table promise. The cell texts are NOT rewritten: per §9.5.3.1's no-row-deleted/no-id-reused discipline the superseded predicate is restated in place here, with the U6 reading beside it. The anchors are clause-name-first — the row ids and the note headings — because the numbers shift; the line spellings this file previously carried for them are recorded as superseded in each item.)***
> **(1) The `P-IM-14` register cell (§9.5.5, the row whose `strat:` id is `strat:vector-flag-flip`; the `:2521` and `:2523` spellings this file carried for these two cells are **superseded by text**).** Its predicate is now read **superseded in place**: the cell's equality `subsystems.vector == store.snapshot().vectors.is_some()` and its sentence *"the rule that always holds is `subsystems.vector == store.snapshot().vectors.is_some()`"* are the **pre-U6** predicate; **after U6 the equality carries the epoch term** — `derived.vector == (store.snapshot().vectors.is_some() && store.snapshot().epoch == store.epoch())`, read through **one** snapshot clone (surface #3) — while the cell's `Reachable` + `Some` scoping, its V-8.2 half, its write-independence half and its `flags.vector == snap.vectors.is_some()` half (which is `boot_wiring`'s **pinned pre-U6** surface, freshness rule 3) all **stand**. **Its V-8.1 assignment is asserted ONLY for the epoch-aligned instance** (the third precondition, §5.8's `U6's amendment to this section's `vector` predicate` clause).
> **(2) The `P-IM-14` §9.5.4 coverage note (the note headed `` `P-IM-14 — strat:vector-flag-flip` `` in §9.5.4; its own line spelling in the pre-remand layout, `:2622`, is **superseded by text**).** The same supersession applies to its sentences `get_engine_status().subsystems.vector == store.snapshot().vectors.is_some()` — **exactly** and its V-8.1 fixture half: the equality gains the epoch term, and the V-8.1 fixture's `vector` value equals what the `Reachable` path derives **only on an epoch-aligned snapshot**. Its boundary instances (the **empty** index ⇒ `vector:true`; `Absent`/`Unreachable` ⇒ `vector:false` with V-8.2 unchanged; `reranker` `false`; `embedding == (a provider was wired)`) and its adversarial/Excluded halves are **unchanged**.
> **(3) The `P-SM-7` register cell (§9.5.5, `strat:boot-lifecycle-vector`).** Its **serving half is unchanged** (a fresh index serves), and its FS-14 half **gains the STALE instance beside the unbuilt one**: a `Ready` store with an index-bearing snapshot whose `snapshot.epoch != store.epoch()` answers `Err(StoreError::VectorIndexUnavailable)` (FS-14 ⇒ 503) on the **explicit** sites and **empties the vector leg with `Ok`** on the fusion site (§9.5.6's valid/fail state 3 and its per-site consultation clause). Its `Absent`/`Unreachable` ⇒ FS-8 half, its empty-index half, its order pin and its call-count half are **unchanged**.
> **(4) The `P-SM-7` §9.5.4 coverage note (the note headed `` `P-SM-7 — strat:boot-lifecycle-vector` `` in §9.5.4 — the note this same-unit table names by id; the `:2478` spelling is **superseded by text** in both places).** The same addition applies: the note's FS-14 half now names **two** instances (unbuilt `vectors: None`, and **stale**), while its `Ready`-gated reading, its FS-8 half and its call-count half stand. **U6's own six §9.5.4 notes live in the same block** (headed by `P-IM-16`'s format-deviation note).
>
> **Derivability (what this marker is for):** a TestWriter re-running the U5 layer from the register derives the **U6** predicate at these two rows — `is_some() ∧ epoch-aligned` — and reads the `P-SM-5` probe-4 fixture of §9.5.5 (the one named exception, re-derived in §9.5.6's same-unit table) as a fixture that must align **or** assert the **stale** outcome. **No id, kind, tag, `Strategy-id`, cap, count or executed figure moves:** U5 stays **8 rows**, its caps stay `355 ≤ 400`, and the U3 rows stay **held under the amended predicate for the corpus their fixtures actually build** — with the one named `P-SM-5` exception.

### 9.5.5 U5 — the boot vector-index build (8 rows ≤ 8; **AUTHORIZED 2026-09-22**, code **OWED** — ***the "code OWED" wording is the spec-gate-time record: the unit is LANDED-GREEN and all eight gates have run; this heading's status marker is reconciled at gate 8 (2026-09-22) in this section's verification-status block below, and its rows are as landed***)

*(Pre-edit tail of this heading, kept as the record: "U5's code is now reported LANDED-GREEN — POST-GREEN SPEC AMENDMENT, 2026-09-22, see this section's verification-status block and adjudication note 1's extension: the 'code OWED' wording in this heading is the pre-green record, the unit's rows are as landed, and the failed-build branch's live criterion is PARKED".)*

**Authority.** The user's go-ahead **Q3 = "(B) U5 ONLY"** (`docs/specs/gnosis-grq-inbound-review.md` §14
POST-RECORD UPDATE 1, its Q3 row); the unit's ordered-workstream row (§8's workstream table, the **U5**
row — "the boot index build + the one-value `vector:true` flip + the V-8.1 literal edit in the same unit");
decision **`SUBSYSTEM-FLAG-CAPABILITY-SEMANTICS`** (ACTIVE) as read through §5.8; decision
**`PBT-GATE-MANDATORY`**
(the typed register below is this unit's artifact #1, its executed layer is the TestWriter's); §11's owed
row for U5 (the register-notes table, the **U5** row); §5.8's unit split; §5.9's **Interlock with U5**
bullet.

**Scope of the unit.** Make the boot produce a **vector index** so that `vector: true` becomes an honest
claim for a boot whose provider is reachable, and so `mode=vector` stops returning
`VectorIndexUnavailable` on such a boot. **Nothing else moves:** no route, no new endpoint, no new
`StoreError` variant, no new §11 row, no change to `EngineSubsystems` (frozen, six `bool`s,
`src/store/mod.rs:568-575`) or to any `HealthReport` field (`src/wire/status.rs:14-21`), no change to the
error map, and **no persistence** (§9.5.5's final clause).

**Class tally (U5):** IM ×6 (`P-IM-10`, `P-IM-11`, `P-IM-12`, `P-IM-13`, `P-IM-14`, `P-IM-15`), SM ×1
(`P-SM-7`), TP ×1 (`P-TP-5`) = **8 rows ≤ 8** ✔ (the register table follows in this section; the per-row
generator-coverage notes are in §9.5.4).

**The typed register (U5 — 8 rows, id allocation and provenance).** The ids continue the **single per-file
p2 sequence** (§9.5's header, the F1 ruling): the highest ids claimed in this file before this pass were
`P-IM-9` (§9.5.2's `P-IM-9`, after the F1 renumber), `P-SM-6` (§9.5.2) and `P-TP-4` (§9.5.1), so U5 takes
the **next free** numbers in each class — **`P-IM-10`…`P-IM-15`** (6), **`P-SM-7`** (1), **`P-TP-5`** (1).
They are **free**: a `docs/specs/*.md` sweep for `P-IM-10`…`P-IM-15`/`P-SM-7`/`P-TP-5` finds **no** prior
claim in this file or any other register file (the other files' ids are their own units' namespaces and are
not touched — §9.5.3.1's cross-file look-alike rule). **No id is reused for a different claim, no U2/U3 id
is renumbered, and no row is deleted.** The per-unit tag for each row is `U5` + its `P<CLASS><N>` form
(§9.5.3's convention, re-derived injective for this unit's eight rows).

| Property-id | Class | Invariant | Strategy-id | Observable-as-property | Code site / contract clause |
|---|---|---|---|---|---|
| `P-IM-10` | IM | **The boot index build is a total THREE-VALUED function of (store, provider), and its only error is the EXISTING `EmbeddingUnavailable` — never a new variant, never a panic.** For any store state and any provider input: **`Ok(None)`** iff no provider was supplied (nothing is to be built — the boot keeps its index-free snapshot; **this is not an error**), **`Ok(Some(vi))`** iff a provider was supplied and every `embed` call succeeded, with `vi.entries`' keys **exactly** the corpus's embeddable `(documentId, nodeId, FieldType::Full)` triples (**0 keys** for an empty corpus — an empty index, **not** `None`), and **`Err(StoreError::EmbeddingUnavailable)`** iff a provider was supplied and an `embed` call failed. The build reads the corpus from `store`, **writes nothing to it**, consults **no** live network beyond `provider` and **no** env var, and does not call `is_available` itself; the whole outcome space is a **closed three-outcome** set for both provider inputs, and the caller's snapshot's **`vectors` field** is composed from the returned `Option<VectorIndex>` alone (the rest of the snapshot is the boot's own `DerivedIndexes`, untouched). | `strat:boot-index-build` | ∀ `s: &Store`, `p: Option<&Arc<dyn EmbeddingProvider>>`: `let r = build_boot_vector_index(s, p).await` ⇒ (`p == None` ⇒ `r == Ok(None)`, with the injected provider's `embed` **call count == 0**); (`p == Some(pr)`, all `embed` calls `Ok` ⇒ `r == Ok(Some(vi))` with `vi.entries.len() == (the count of nodes reachable through the store with `value.is_some()`)` and `vi.entries.keys().all(|k| k.2 == FieldType::Full)`); (`p == Some(pr)`, some `embed` call `Err` ⇒ `r == Err(StoreError::EmbeddingUnavailable)`, **never** `Ok(None)` and never a partially-filled `Some`); **the call discipline of §9.5.5's contract table (2)'s call-count row** — for `p == Some(pr)` with all calls `Ok`, exactly `n` `embed` calls for `n` embeddable nodes, strictly sequential, **none** for a `value:None` node; across every call `s.epoch()`/`s.journal_len()`/`s.snapshot()`'s `Arc` identity are unchanged and no journal entry is appended (the build is a pure read); the call never panics for any of `P-TP-5`'s provider shapes. | §9.5.5's "What U5 adds" table (surface #1), its contract table (1) and (2) and its failure table; §5.8's pin that U5's change is *what the snapshot contains*; `src/lib.rs:69-110` (the F11 lib-seam precedent), `src/store/mod.rs:809-812` (`VectorIndex`), `:827-832` (`DerivedIndexes`), `:848-858` (`EmbeddingProvider`), `:1539-1542` (`StoreShard.docs`), `:180-183` (`Graph.nodes`), `:154` (`Node.value`), `:1690`/`:1851` (`Store::new`/`snapshot`) |
| `P-IM-11` | IM | **Reachable means built: the boot builds an index iff the provider was reachable, and the index covers the WHOLE corpus for an empty store too.** `BootProvider::Reachable(_)` ⇒ `Ok(Some(vi))` and the boot swaps an **index-bearing** snapshot even when the corpus is empty (an empty `VectorIndex` is still `Some`, so `subsystems.vector == true` and the freshly booted server's flag is honest); `BootProvider::Absent`/`Unreachable` ⇒ the build is called with **no** provider (or not called at all), so the boot swaps the **unchanged** index-free snapshot (`vectors: None`, `vector == false`, and **no** embedding call is attempted). The build is **full-field and node-level**: exactly one `FieldType::Full` entry per embeddable node, **no** `FieldType::Binary` entry in any outcome, and nodes whose `value` is `None` contribute nothing. | `strat:index-corpus-full-field` | ∀ boot outcome `p ∈ {Absent, Unreachable, Reachable(pr)}` and ∀ store state `s`: for `Reachable`, `let built = match build_boot_vector_index(s, Some(&pr)).await { Ok(Some(vi)) => vi, other => panic!("{other:?}") }` and `let snap = DerivedIndexes { vectors: Some(built), ..Default::default() }`; for `Absent`/`Unreachable`, `build_boot_vector_index(s, None).await == Ok(None)` and `snap = DerivedIndexes::default()`; then `(boot_wiring(p, &snap).1).vector == snap.vectors.is_some()`; `p == Reachable` ⇒ `snap.vectors.is_some()` **for every** store state **including an empty store** (`Store::new()` with no documents ⇒ `entries.is_empty()`) and every key's `.2 == FieldType::Full` with `.0`/`.1` equal to the **source** `(documentId, nodeId)`; `p ∈ {Absent, Unreachable}` ⇒ `snap.vectors.is_none()` and the injected provider's `embed` **call count == 0**; ∀ node with `value == None` in the corpus: no entry keyed by it; ∀ node with `value == Some("")`: exactly one entry (and §9.5.5(2)'s **corpus-key-uniqueness row** holds of the sampled corpus — one document never contributes two nodes sharing one `NodeId`, so the count is well defined); the build's **`embed` call count is exactly `n`** for `n` embeddable nodes (strictly sequential, no fan-out/retry, and **no** call for a `value:None` node — §9.5.5(2)'s call-count rule), which is the witness this cell and `P-TP-5` both read: **one key per embeddable node *and* one `embed` call per embeddable node**. | §9.5.5's contract table (1) (the three `BootProvider` outcomes: build / no-build / no-build) and (2) (the corpus + empty-store pin), its question-3 clause (full field only, binary deliberately unbuilt), its adjudication note 4; §5.9's interlock bullet; `src/store/mod.rs:5083-5093` (`vector_leg` walks `FieldType::Full` keys), `:5094-5102` (`binaryFirstPass` degrades to the full field when no `binary` entry exists), `:4270` (`value`-carrying snippet text at read time), `:4213` + `:4193-4232` (the `vector` predicate) |
| `P-IM-12` | IM | **The build is deterministic and side-effect-free on the store: same inputs ⇒ element-wise equal index, and the store's own observable state is untouched.** Two builds over the same store state with the same provider yield **element-wise equal** results (both `Ok(None)`, both `Err`, or both `Ok(Some(vi))` with identical key sets and identical `Vec<f32>` per key), with no dependence on `HashMap` iteration order, thread scheduling, wall clock or random state; and a build mutates **nothing** the store reports — `epoch()`, `journal_len()` and the `snapshot()` `Arc`'s identity/`vectors` presence are identical before and after, the wired provider is unchanged, and the build appends **no** journal entry (no `append_journal` call site). | `strat:index-build-stability` | ∀ store state `s`, ∀ injected provider `pr`: `let a = build_boot_vector_index(s, Some(&pr)).await` and `let b = build_boot_vector_index(s, Some(&pr)).await` ⇒ `a == b` (same variant; in the `Ok(Some(_))` case the entry sets are pairwise equal by key **and** by vector); and across each call `s.epoch()`/`s.journal_len()`/`s.snapshot().vectors.is_some()`/`Arc::ptr_eq(&s.snapshot(), &before)` are unchanged, while `s.embedding_provider().is_some()` is unchanged; a provider whose `embed` returns the **same** constant for every text still yields `a == b` (the key set — not the vectors' content — is what ordering would perturb). | §9.5.5's contract table (2) (the pinned-order clause: determinism required, no insertion-order claim) and its contract-table header; `src/store/mod.rs:1851-1860` (`snapshot`/`swap_snapshot` — only the caller installs, the build does not), `:1816-1821` (`epoch`/`journal_len` observers), `:1828` (`append_journal` — **not** called by the build), `:1691-1693` (`SHARD_COUNT` shards, the corpus's only carrier), and **§9.5.5's contract table (2) *vector comparison / `NaN`* row** (the semantics "pairwise equal by vector" is read under — added by the post-red-phase register amendment, 2026-09-22) |
| `P-IM-13` | IM | **The build is atomic: an embedding failure yields NO index at all — no partial index is ever observable, and a retry can be complete.** If the *k*-th `embed` call (`k` anywhere in `1..=n`) returns `Err`, the build returns `Err(StoreError::EmbeddingUnavailable)` — **not** `Ok(Some(partial))` and **not** `Ok(None)`: the value it discards is a local under construction, so no half-populated index is ever returned to the caller or reachable through the store (`snapshot().vectors.is_none()` after the boot's documented failure outcome ⇒ `vector:false`); the failure is **not sticky** (a subsequent success returns `Ok(Some(vi))` with the full corpus). The only installation vehicle is one `swap_snapshot` of a fully composed `DerivedIndexes`, so a reader can never observe a partially built index. | `strat:index-build-atomicity` | ∀ `k ∈ 1..=n` with a provider failing on its *k*-th call: `build_boot_vector_index(s, Some(&pr_k)).await == Err(StoreError::EmbeddingUnavailable)`; and after the caller applies the boot contract's failure outcome (`swap_snapshot` of the **unchanged** index-free snapshot) `s.snapshot().vectors.is_none()` and `s.get_engine_status().await.subsystems.vector == false`; with a fail-once-then-succeed provider the *same* input yields `Ok(Some(vi))` whose `entries.len()` equals the corpus's embeddable count (`P-IM-10`'s count witness); no provider behaviour yields `Ok(Some(vi))` whose `entries.len()` is **between** `1` and `n - 1` inclusive **while an `embed` call failed** (the partial-index negative probe). | §9.5.5's failure table (rows 1-3 and the "mid-build provider death / partial write" row) and its valid/fail table states 4 and 6; `src/store/mod.rs:1855-1860` (the atomic `Arc` swap — "no partial index states, no data races"), `:4294-4297` (the read-time `VectorIndexUnavailable` on `vectors: None`), `src/server.rs` (§11's 503 rendering of FS-14 — **unchanged**), **§5.9's outcome table (:1572-1583; re-read this pass)** and **§5.9's `Precedence (pinned)` bullet (:1585-1592; re-read this pass)** — its READY qualifier is what makes the FS-14 row a **READY**-store outcome — with its third clause **§5.9's `Interlock with U5` bullet (:1643-1645; re-read this pass)** (the citation this cell carried was the `testable assertions` clause, and the pre-Remand-3 `:1562-1573` spelling is the same stale class; `:1562-1573` is §5.9's heading + the ruling's opening line) (***REMAND-4 MUST-FIX 1, 2026-09-22: the `:1569-1580` table-body / `:1582-1589` bullet / `:1640-1642` interlock triple recorded by REMAND-2/3 is **superseded** — each number shifted as this file grew, and `:1569-1580` starts on the blank line before the table header while `:1640-1642` is §5.9's `Testable assertions the ruling yields` clause, **not** the interlock bullet; the previously cited numbers are kept in this annotation as the record***) |
| `P-IM-14` | IM | **The flag is the wiring: `subsystems.vector` equals `snapshot().vectors.is_some()` after the pinned boot wiring, and the V-8.1 fixture agrees with what the boot derives.** For any boot outcome, after the pinned wiring path (the returned `EngineState` applied + the snapshot swapped + the provider wired **only** in the `Reachable` case, **never** `set_subsystems`), `get_engine_status().subsystems.vector == store.snapshot().vectors.is_some()` element-wise, with `store`/`graph`/`lexical` `true`, `reranker` `false`, `embedding == (a provider was wired)` — and the **V-8.1 mask clause is scoped by the snapshot**: the amended **V-8.1** mask `{store:true, graph:true, lexical:true, vector:true, embedding:true, reranker:false}` is the mask of a `Reachable` boot **whose snapshot has `vectors: Some`** (the boot-build contract's `Reachable` outcome, an empty index included), while a `Reachable` boot handed a snapshot with `vectors: None` derives `vector:false` by the same predicate and reports `Ready` + `vector:false` (a non-boot-constructed store, or a boot that attempted no build) — **neither reading may be collapsed**: the rule that always holds is `subsystems.vector == store.snapshot().vectors.is_some()`, and V-8.1 is the `Some` instance of it. An `Absent`/`Unreachable` boot reports **V-8.2** `{…, vector:false, embedding:false, reranker:false}` unchanged. A false-mask write through `set_subsystems` changes nothing. | `strat:vector-flag-flip` | ∀ `p ∈ BootProvider`, ∀ `snap` (index-free, populated, and `Some(VectorIndex::default())`): `let (st, flags) = boot_wiring(p, &snap); let s = Store::new(); s.swap_snapshot(snap.clone()); if let Reachable(pr) = p { s.set_embedding_provider(pr) } s.set_engine_state(st);` ⇒ `s.get_engine_status().await.subsystems == flags` **and** `flags.vector == snap.vectors.is_some()`; and for `Reachable` **+ a snapshot whose `vectors` is `Some`** (the built snapshot — the **only** input pair for which V-8.1 is asserted), `flags == the V-8.1 mask of §9.5.5's flip` (i.e. `== honest_ready_subsystems()` as amended) while for `Reachable` + an index-free snapshot, `flags.vector == false` and the rest of `flags` is the V-8.1-shaped vector with `vector:false` (**not** V-8.2, whose `embedding` is `false`), and for `Absent`/`Unreachable` + the unchanged snapshot `flags == honest_degraded_subsystems()` (V-8.2, **unchanged**); the invariant that holds for **every** `(p, snap)` pair is `flags.vector == snap.vectors.is_some()` — the `Reachable` + `vectors:None` pair included; `s.set_subsystems(<all-true>)` before the read leaves every flag unchanged. | §5.8's U5 answer bullet + its capability predicate table (`vector == snapshot().vectors.is_some()`); §9.5.2's flag table and `P-IM-7` (the derivation's right-hand side, **not re-scoped**); §9.5.5's move table (the `honest_ready_subsystems()` flip and the V-8.1 literal); `src/store/mod.rs:4193-4232` (`get_engine_status`, `vector` at `:4213`), `:1868-1876` (the inert `set_subsystems`), `:568-575` (the frozen six flags); `tests/wire_conformance.rs:713-722`, `:1093-1111`, `:1192-1239` |
| `P-IM-15` | IM | **Boot-outcome honesty under a build failure: a reachable provider whose build fails degrades — it never fabricates `Ready` and never claims the index.** If the provider is reachable but the build returns `Err`, the boot applies `Degraded` (via the **existing** `boot_wiring(BootProvider::Unreachable, &unchanged_snapshot)`-shaped derived pair — no new branch, no new state), swaps the **unchanged** index-free snapshot (so `vector:false`) while the provider **is** wired (so `embedding:true`), and the status read reports that pair with the store's **fixed** `last_error` string — i.e. `vector` is never `true` without an index in the snapshot, and no flag/`last_error` value is invented for this path. | `strat:boot-index-failure` | ∀ store state `s`, ∀ fail-on-*k*-th provider `pr`: `let failed = build_boot_vector_index(s, Some(&pr)).await` is `Err(_)`; then applying the boot's documented failure outcome in **the order the contract table (4) pins** (`let snap = DerivedIndexes::default()` (the **unchanged** boot snapshot), `s.swap_snapshot(snap.clone())`, `s.set_embedding_provider(pr)` (the probe **did** succeed, so the provider **is** wired even though the build failed), `let st = boot_wiring(BootProvider::Unreachable, &snap).0`, `s.set_engine_state(st)` — and the returned flag vector of that `boot_wiring` call is **discarded**; the store's own read-time derivation is the observable) yields `s.get_engine_status().await == EngineStatus { state: Degraded, version: env!("CARGO_PKG_VERSION"), subsystems: {store:true, graph:true, lexical:true, vector:false, embedding:true, reranker:false}, last_error: Some("a non-core subsystem (embedding/reranker) is unavailable") }` — the state is **not** `Ready`, `vector` is **not** `true`, and the `last_error` string is the store's own (byte-identical), with no new wire code and no §11 row produced anywhere on this path. | §9.5.5's failure table (the "what the boot does with that `Err`" and "the wire consequence" rows) and its adjudication note 1; §9.5.2's adjudication note 2 (the boot's reachable state set) and `P-IM-8` (embedding ≠ state); §11's **closed 21-row** map + decision `RESERVED-ERRVARIANTS-DISCIPLINE` (no variant without a producing step); `src/store/mod.rs:4226-4230` (the fixed `last_error`), `:1864` (`set_engine_state`), `:1885` (`set_embedding_provider`), `src/server.rs` (the §11 rendering, unchanged) |
| `P-SM-7` | SM | **The post-boot query transition is consistent with the flag: with a built index a `mode=vector` request serves; without one the outcome is **state-gated** (***REMAND-2 MUST-FIX 1, 2026-09-22 / REMAND-3 SHOULD-FIX 4 — this opening clause read "…without one it fails with the pinned FS-14 error …" (the cell's body below carries the corrected reading in full) and is restated in place, its superseded text kept as the record here: a non-READY store ⇒ `EngineUnavailable` (FS-8); FS-14 only on a READY store with `vectors: None`.***) — fails with the pinned FS-14 error **on a READY store with `vectors: None`** — and an EMPTY index serves an empty result rather than an error.** For a `Reachable` boot (index built, provider wired, `Ready` applied): `rag_query("<q>", {mode: Vector, …})` is `Ok(RagResult)` with `trace == RagTrace::Vector(TraceDescriptor{ mode: Vector, engine: "gnosis", … })`, `results.len() <= top_k`, and a `results` set drawn **only** from the index's wiki-scoped `full` entries (empty for an empty index or a wiki with no indexed nodes — never an error, never a panic). For a boot with **no** index the outcome is **state-gated, never leg-gated** (***REMAND-2 MUST-FIX 1, 2026-09-22 — the pre-remand cell read `Absent`/`Unreachable` ⇒ `Err(VectorIndexUnavailable)` ⇒ 503 `vector_index_unavailable`, which is **unsatisfiable**: §5.9's precedence bullet puts the READY gate before every leg check for non-`graph` modes, so an `Unreachable` boot's `Degraded` state yields `Err(StoreError::EngineUnavailable)` (FS-8) → §11's **503 `engine_unavailable`**, and an `Absent` boot's `Unavailable` state likewise; the pre-remand reading's superseded text is kept here as its record.***): an `Absent`/`Unreachable` boot leaves the store **non-READY** ⇒ the same request is `Err(StoreError::EngineUnavailable)` → §11's **503 `engine_unavailable`** (FS-8) — **the FS-14 error is NOT what a non-READY store produces**, whatever its snapshot holds; FS-14 `Err(StoreError::VectorIndexUnavailable)` → **503 `vector_index_unavailable`** is reachable **only on a READY store with `vectors: None`** (a `Reachable` boot over an index-free snapshot whose `EngineState` is `Ready`, or an explicitly named caller-built READY store). The index-before-provider precedence is pinned **within `mode=vector` on a READY store**. | `strat:boot-lifecycle-vector` | ∀ boot outcome: `let mut o = RagQueryOptions::default(); o.mode = Some(QueryMode::Vector); o.wiki_id = Some(w); o.top_k = Some(k);` (plus the provider wired only in the `Reachable` case and the built snapshot swapped) ⇒ for `Reachable`: `s.rag_query("q", &o).await` is `Ok(r)` with `matches!(r.trace, RagTrace::Vector(_))`, `r.engine == "gnosis"`, `r.results.len() <= k`, and every `r.results[i].document_id/node_id` present as a `(doc, node, Full)` key of the built index; for `Absent`/`Unreachable` (a non-READY store — the returned `EngineState` applied, exactly as `set_engine_state(boot_wiring(p, &snap).0)` produces it): `== Err(StoreError::EngineUnavailable)` with `server_status(&EngineUnavailable) == Some((503, "engine_unavailable"))` (FS-8 — the pre-READY gate of `src/store/mod.rs:4078-4080` fires **before** any leg check), and the injected provider's `embed` **call count == 0** (no query text is embedded on this path); the FS-14 instance is a **READY** store only: a `READY` store with `vectors: None` (the same `Absent`/`Unreachable` snapshots **plus** `set_engine_state(EngineState::Ready)` — or the equivalent explicitly named caller-built READY store) ⇒ `== Err(StoreError::VectorIndexUnavailable)` with `server_status(&VectorIndexUnavailable) == Some((503, "vector_index_unavailable"))`, and the injected provider's `embed` **call count == 0** (the index check precedes the provider, `src/store/mod.rs:4448-4454`); the **precedence** instance (both on that READY store): `vectors: None` + no provider ⇒ `VectorIndexUnavailable` (not `EmbeddingUnavailable`), `vectors: Some(_)` + no provider ⇒ `EmbeddingUnavailable`; and the empty-index instance: an empty index + a reachable provider ⇒ `Ok(r)` with `r.results.is_empty()`. | §5.9's outcome table (`:1562-1573`; the pre-REMAND-2 citation `:1549-1558` pointed at §5.8's clause end + §5.9's heading — **stale after the §5.8 insertion**), its **precedence** bullet (`:1575-1578`; the pre-REMAND-2 citation was `:1562-1563` — it named §5.8/§5.9 body text, not the bullet; the bullet text reads "the READY gate precedes every leg check for non-`graph` modes … an unbuilt index can only surface as `VectorIndexUnavailable` on a **READY** engine") and its **Interlock with U5** bullet (`:1629-1631`; the pre-REMAND-2 citation `:1616-` was stale for the same reason)ith U5** bullet (`:1616-1618`); §9.5.5's valid/fail table states 1, 2, 6 and 7; `src/store/mod.rs:4448-4454` (index checked before provider), `:4452-4454` (`EmbeddingUnavailable`), `:4526-4556` (hybrid degradation — **excluded** here), `:4477-4482` (the vector trace), `:5072-5131` (`vector_leg` over `Full` entries), `tests/rag_query_integration.rs:689-716` (the still-reachable FS-14 store-level instance) |
| `P-TP-5` | TP | **The build is total over adversarial provider shapes and never mislabels its output:** any `embed` behaviour (dimension change between calls, an empty vector, a zero vector, `NaN`/`±∞`, a lie in `is_available`, an always-`Err` provider) leaves the outcome in `{Ok(None), Ok(Some(index over exactly the corpus)), Err(EmbeddingUnavailable)}` — never a panic, never a partially built index presented as `Ok(Some(_))`, never a new error type, never a key that is not the source `(documentId, nodeId, FieldType::Full)` triple, and never a dropped/duplicated corpus entry. | `strat:boot-index-adversarial` | ∀ adversarial provider `pr` and ∀ store state `s`: `build_boot_vector_index(s, Some(&pr)).await` terminates with either `Ok(Some(vi))` (⇒ `vi.entries.len() == embeddable_count(s)`, every key `== (source doc, source node, FieldType::Full)`, and **no** `FieldType::Binary`/`Other(_)` key) or `Err(StoreError::EmbeddingUnavailable)` (⇒ **no** index at all); the call never panics (asserted by construction: a total provider impl and an absence of `unwrap`-on-failure paths), and the returned `Err` is **never** a `ValidationError`/`EngineError`/new variant; a provider whose `is_available` lies (`true` while `embed` errors) changes **nothing** about the build's outcome (the seam is never consulted by the build); a **duplicate** node text under two distinct ids yields **two** distinct keys (no text-keyed dedup; the corpus this row quantifies over is the **valid** one — contract table (2)'s **corpus-key-uniqueness row**, cited clause-name-first — i.e. no two nodes of one document share a `NodeId`, and a generator MUST NOT create that shape: the index pins **one** key per embeddable node); a node with `value: None` yields **no** key while `value: Some("")` yields one; the returned `Vec<f32>` is the provider's own value **verbatim** (same length, same elements — no normalisation, no truncation, no re-ordering). | §9.5.5's question-3 clause (dimension not pinned; the provider's length is authoritative) and its contract table (2) (the corpus/text rules); the `VectorIndex`/`FieldType` shapes (`src/store/mod.rs:798-812`); `cosine`'s dimension tolerance (`:5333-5335`); **§5.9's `HyDEGenerationFailed`-unreachable clause (:1617-1620; re-read this pass)** — the `reserved-variant rule` spelling this cell carried, and the tail numbers it gave (which resolved to §5.9's `Precedence (pinned)` bullet instead), are superseded by ***REMAND-4 MUST-FIX 2, 2026-09-22*** — as the `HyDEGenerationFailed` rule §5.9 pins unreachable `:1586-1589`) as the discipline this row inherits (no invented fail-variant) |

**REMAND-2 (2026-09-22) corrections to the eight rows above — recorded here because the cells' own tails are long (no row id, strategy id, count or cap changed; every superseded clause is kept in place, annotated).**
This block is an **in-place dated annotation** of the register table it follows; it changes **no** row's claim beyond the corrections named below, **no** row id, **no** `Strategy-id`, and **no** arithmetic (its own `340 ≤ 400` reading is **superseded by the post-red-phase register amendment below**, under which `355 ≤ 400` governs — the superseded figure is kept here as the record). The corrections, with the line numbers **re-read this pass**:

1. **MUST-FIX 1 — `P-SM-7`'s fail half is READY-gated, not leg-gated.** The row's invariant, its
   `strat:boot-lifecycle-vector` observable and its §9.5.4 coverage note (the `P-SM-7 — strat:boot-lifecycle-vector`
   note, **re-read this pass at line 2419 — REMAND-3 MUST-FIX 2, 2026-09-22: this anchor read `2412`, which is the
   *`P-SM-5 — strat:status-pure-read`* note; the `P-SM-7` §9.5.4 note is at `:2419`, where the corrected text already
   stands verbatim (no cell edit needed). The `2412` reading is superseded by text here.***) now read: `Absent`/
   `Unreachable` boot ⇒ the returned `EngineState` applied (`set_engine_state(boot_wiring(p, &snap).0)`)
   leaves the store **non-READY** ⇒ `mode=vector` ⇒ `Err(StoreError::EngineUnavailable)` (FS-8) → §11's **503
   `engine_unavailable`**; `Err(StoreError::VectorIndexUnavailable)` (FS-14) → **503
   `vector_index_unavailable`** is asserted **only on a READY store with `vectors: None`** (the same snapshots
   **plus** `set_engine_state(EngineState::Ready)`, or the explicitly named caller-built READY store that
   `tests/rag_query_integration.rs:689-716` constructs — the FS-14 integration test calls `ready(&store)` at
   `:701`). The pre-remand text is retained in each of those three places as its record.
2. **SHOULD-FIX 7 — the §5.9 anchors in this row's Code-site cell are re-pointed by text, with the re-read
   line numbers.** §5.9's **outcome table** is at `:1572-1583`; §5.9's **`Precedence (pinned)` bullet** is at
   `:1585-1592`; §5.9's **`Interlock with U5`** bullet is at `:1643-1645` — ***all three re-read this pass
   (REMAND-4 MUST-FIX 1, 2026-09-22).*** The three citations the row carried
   (`:1549-1558`, `:1562-1563` and an `:1616-…` spelling of the third) were **stale** — that line is the
   `testable assertions` clause, not the bullet — **and the first named §5.8's clause end + §5.9's heading
   rather than the table, while the third's `:1616-…` tail is wrong**: the correct citation for the third
   clause is `:1643-1645`. ***(REMAND-3 MUST-FIX 1, 2026-09-22 — the triple that round pinned was the outcome-table body
   `:1569-1580`, the precedence bullet `:1582-1589` and the Interlock bullet `:1640-1642`, and **REMAND-4
   MUST-FIX 1 (2026-09-22) re-points all three once more**: those numbers are **superseded** (kept here as the
   record) because each drifted as the file grew — the outcome table's header is now at `:1572` and its last
   row at `:1583`, the precedence bullet at `:1585-1592`, and the Interlock bullet at `:1643-1645`; the
   `:1633-1635`
   reading this item previously carried is **superseded by text** — `:1633-1635` is §5.9's frozen-`HybridTrace`
   per-leg-availability note ("the frozen `HybridTrace` has no per-leg availability field … out of U1's scope"),
   an unrelated clause, so the one pointer a TestWriter was told to follow landed on the wrong text. The block's
   pre-Remand-3 `:1562-1573`/`:1575-1582` spellings were the same off-by-seven class: the outcome table's body
   now begins at `:1572` and the precedence bullet at `:1585`, both re-read this pass.***) The
   same re-pointing applies to **`P-IM-13`**'s cell (`§5.9's table` ⇒ `§5.9's outcome table (:1572-1583)` plus
   §5.9's `Precedence (pinned)` bullet `:1585-1592` and §5.9's `Interlock with U5` bullet `:1643-1645` — the
   REMAND-2/3 spellings `:1569-1580`/`:1582-1589`/`:1640-1642` are **superseded**, REMAND-4 MUST-FIX 1,
   2026-09-22). **No claim moved** — the anchors name the same three clauses.
3. **SHOULD-FIX 8 — the call-count citation and the rows agree.** Contract table (2)'s call-count row now
   names **`P-IM-10`/`P-IM-11`/`P-IM-13`/`P-SM-7`** (the pre-remand list named `P-IM-13`/`P-SM-7`, which did
   **not** carry the instance); `P-SM-7`'s observable and its §9.5.4 note now carry the "**no `embed` call on a
   path that never embeds**" instance explicitly (the injected provider's call count is **0** on both the FS-8
   and the FS-14 paths), so the citation's claim is true of every row it names.
4. **MUST-FIX 3 / SHOULD-FIX 4 — the "must move in the same unit" table's probe set is unchanged in scope, and
   its input rule is singular.** What moves is **probe (1)** alone: its input at `tests/wire_conformance.rs:1195`
   → `boot_snapshot(true)`, its assertion at `:1204`, and the two prose texts at `:1205`/`:1191` (SHOULD-FIX 4);
   probes **(2)** at `:1209` and **(3)** at `:1218` **stay** on `boot_snapshot(false)` (their fixture is
   `honest_degraded_subsystems()`), and probe **(4)**'s input at `:1231` is **already** `boot_snapshot(true)`.
   The same correction now sits in `docs/specs/engine-wire-contract.md` §12's U5 note (MUST-FIX 3). **The
   `:1192-1239` block citation in the move table's last row names the whole probe set, not a plural move rule.**
5. **POST-RED-PHASE AMENDMENT (2026-09-22) — the two rows' *comparison semantics* are now contractual, and
   the arithmetic in this block's header is superseded.** `P-IM-12`'s invariant/observable ("element-wise
   equal … identical `Vec<f32>` per key") and `P-TP-5`'s invariant/observable ("the provider's own value
   **verbatim** (same length, same elements)") are read **under contract table (2)'s *vector comparison /
   `NaN`* row**: same `len()`, every **non-`NaN`** element identical bit-for-bit, `NaN` asserted only in the
   `NaN` position (`a[i].is_nan() == b[i].is_nan()` element-wise), and **no** tolerance-based comparison.
   The pre-amendment cells are silent on `NaN` rather than wrong — `P-TP-5`'s own adversarial corpus
   generates `NaN`/`±∞`, and IEEE-754 makes `NaN != NaN`, so the reading "same length + non-`NaN` elements
   verbatim, with `NaN` asserted only in the `NaN` position" (the reading the U5 TestWriter implemented and
   flagged) is hereby **the contract**, not a test-side choice. **Nothing else in either cell moves**: both
   rows keep their ids, kinds, tags, strategy ids and every other clause, and the alternative disposition
   (excluding `NaN` from the generated corpus) is **rejected explicitly** — the `NaN`/`±∞` shapes are
   `P-TP-5`'s adversarial value (no other row's corpus is affected), so the comparison rule is the pinned
   remedy and it is enforced **at the comparison, not at the generator**: a TestWriter MUST NOT filter `NaN`
   out of `P-TP-5`'s corpus to make the row easier. The `340 ≤ 400` figure this block's header carried is
   **superseded** by the post-red-phase amendment's `355 ≤ 400` (per-row caps `60/50/40/45/45/30/45/40`; the
   measured layer is **314 executed / 355 caps**); no row id, strategy id, count, tag or claim moved with it.

**Class tally (U5):** IM ×6 (`P-IM-10`…`P-IM-15`), SM ×1 (`P-SM-7`), TP ×1 (`P-TP-5`) = **8 rows ≤ 8** ✔.

**What U5 adds (the API surface a TestWriter derives from — names are the unit's; the behaviour, the
signature and the return shape are pinned).**

| # | surface (name is U5's) | signature | return shape / contract |
| --- | --- | --- | --- |
| 1 | the boot index build, **lib-visible** (the F11 precedent: `boot_wiring` lives in the lib precisely so a `[[bin]]`-only path stays assertable, `src/lib.rs:69-110`) | `async fn build_boot_vector_index(store: &Store, provider: Option<&Arc<dyn crate::store::EmbeddingProvider>>) -> Result<Option<VectorIndex>, crate::store::StoreError>` | a **total, three-valued** function on its two inputs — **`Ok(None)`** = *no index is to be built* (provider absent ⇒ the boot keeps its index-free snapshot), **`Ok(Some(vi))`** = *the index to install* (`vi.entries` may be **empty** for an empty corpus — an empty index is still an index), **`Err(StoreError::EmbeddingUnavailable)`** = *a provider was present but an `embed` call failed* ⇒ the boot installs nothing. It reads the corpus from `store`, writes **nothing** to it, consults **no** env var and makes **no** availability probe of its own (the boot's probe already produced the `BootProvider`, `src/bin/gnosis_server.rs:391-402`), and returns the index **alone** — the snapshot is composed by the caller, so the seam has no `DerivedIndexes` plumbing to get wrong. **Where the body lives (pinned, REMAND-1):** the function's **name is re-exported from `src/lib.rs`** (lib-visible, like `boot_wiring`), while its **body is written inside `src/store/`'s own module**, because the corpus walk reads `Store.shards: Box<[RwLock<StoreShard>]>` (`:1552`) and `StoreShard.docs: HashMap<DocumentId, Arc<Document>>` (`:1539-1542`), both **private to that module**; the read is therefore in-module and **no new store accessor, `pub` field or test hook is added** (surface #5 states the same pin from the reader's side) |
| 2 | the existing boot-wiring seam (**U3's, unchanged by U5**) | `fn boot_wiring(provider: BootProvider, snapshot: &DerivedIndexes) -> (EngineState, EngineSubsystems)` (`src/lib.rs:90-110`) | **unchanged**: `Absent` ⇒ `(Unavailable, …)`, `Unreachable` ⇒ `(Degraded, …)`, `Reachable(_)` ⇒ `(Ready, …)`; in all three `vector == snapshot.vectors.is_some()`, `store`/`graph`/`lexical` `true`, `reranker` `false`, `embedding == (a provider was wired)` |
| 3 | the enum the boot probes with (**U3's, unchanged**) | `enum BootProvider { Absent, Unreachable, Reachable(Arc<dyn EmbeddingProvider>) }` (`src/lib.rs:70-77`) | the three boot inputs the contract's table quantifies over |
| 4 | the derived snapshot + its index (frozen store types, unchanged) | `struct DerivedIndexes { lexical: Option<LexicalIndex>, vectors: Option<VectorIndex>, epoch: u64 }` (`src/store/mod.rs:827-832`); `struct VectorIndex { entries: HashMap<(DocumentId, NodeId, FieldType), Vec<f32>> }` (`:809-812`); `enum FieldType { Full, Binary, Other(String) }` (`:798-803`) | the index's **only** carrier; both types are `Default` + `Clone` |
| 5 | the store seams the row observables read (**unchanged**) | `Store::new()` (`:1690`), `snapshot() -> Arc<DerivedIndexes>` (`:1851`), `swap_snapshot(DerivedIndexes)` (`:1858`), `set_embedding_provider(Arc<dyn EmbeddingProvider>)` (`:1885`), `embedding_provider() -> Option<Arc<dyn EmbeddingProvider>>` (`:1893`), `set_engine_state(EngineState)` (`:1864`), `get_engine_status() -> EngineStatus` (`:4193`), `epoch()` (`:1816`), `journal_len()` (`:1821`), `rag_query(query: &str, options: &RagQueryOptions) -> Result<RagResult, StoreError>` (`:4065`) | the observables named in the register's `strat:` cells — **no new _store accessor_ is required by any row** (a row MUST NOT assert through a hook U5 does not add). **The one read that is not an accessor is the build's own corpus walk:** `Store.shards: Box<[RwLock<StoreShard>]>` (`:1552`) and `StoreShard.docs: HashMap<DocumentId, Arc<Document>>` (`:1539-1542`) are **module-private**, so the build's **body lives inside `src/store/`'s own module** (where those privates are visible) and only its **name** is re-exported from `src/lib.rs` (surface #1) — no accessor, no `pub` field and no test-visible corpus hook is added. **Corpus seeding (the populated states of `P-IM-10`/`P-IM-11`/`P-IM-13`/`P-IM-15`/`P-SM-7`/`P-TP-5`) uses the public `RagStore` surface only — surface #7.** |
| 6 | the provider trait (**unchanged**) | `trait EmbeddingProvider { fn embed(&self, text: &str) -> Pin<Box<dyn Future<Output = Result<Vec<f32>, StoreError>> + Send + '_>>; fn is_available(&self) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> }` (`:848-858`) | the seam every U5 row injects a **deterministic in-memory** impl through |
| 7 | **the corpus-seeding calls (NEW, pinned so no row's populated corpus needs an invented API)** | `impl RagStore for Store`'s `async fn create_wiki(&self, name: &str) -> Result<Wiki, StoreError>` (`:2806`; trait decl `:1130`), `async fn create_document(&self, wiki_id: &WikiId, request: CreateDocumentRequest) -> Result<Document, StoreError>` (`:2381-2385`; trait decl `:1071-1075`), `async fn get_document(&self, document_id: &DocumentId) -> Result<Document, StoreError>` (`:2426`; trait decl `:1078`), `async fn update_document(&self, document_id: &DocumentId, request: UpdateDocumentRequest) -> Result<Document, StoreError>` (`:2435`; trait decl `:1086-1090`) — all four are **trait** methods, so the test needs `use gnosis::RagStore;` in scope (the trait is re-exported, `src/lib.rs:120-133`) | the exact recipe for every populated store state this unit's rows quantify over is pinned in §9.5.5's **"corpus seeding"** clause below (a wiki, a document, then one `update_document` carrying the nodes); **no row may seed a corpus through a private field, a serde round-trip or a hand-built `Document`** |

**The contract, pinned as tables (not prose).**

**(1) When does the boot build an index? — the three `BootProvider` inputs (entry question 1).**

| boot input (`main()`'s probe, `src/bin/gnosis_server.rs:391-402`) | index build | `snapshot().vectors` after the boot | `EngineState` (`boot_wiring`) | `subsystems.vector` | `embedding` |
| --- | --- | --- | --- | --- | --- |
| `BootProvider::Reachable(p)` — a provider is configured and its `is_available()` probe succeeded | **BUILT** — always, over the store's current corpus, **even when the corpus is empty** | `Some(VectorIndex { entries })` (`entries` may be **empty**) | `Ready` | **`true`** | `true` |
| `BootProvider::Unreachable` — configured, probe failed | **NOT built** — **no** embedding call is attempted at all | `None` (the boot swaps the **unchanged** snapshot) | `Degraded` | `false` | `false` |
| `BootProvider::Absent` — no provider configured (no env var) | **NOT built** | `None` | `Unavailable` (the construction value, `:1713`) | `false` | `false` |

- **Why this is the honest reading of the flag (not a convention invented here).** §5.8/F2 §9.1 define a
  flag as *"this subsystem's full query-time capability is wired and functional for the current store"*.
  With **no** provider, `mode=vector` cannot embed a query, so an index would be dead weight and
  `vector: true` would be a false claim — the error would simply move from `VectorIndexUnavailable` to
  `EmbeddingUnavailable` (§5.9's table, the FS-14 row). With a **reachable** provider the capability
  genuinely is wired: the leg runs end to end, and an empty store yields an **empty** result, not an error.
  The landed U3 rows already settle that `Some(EMPTY VectorIndex)` ⇒ `vector:true` (`tests/props_gnosis_server.rs:4747-4762`,
  "the flag is the WIRING, not the entry count") — U5 reuses that reading and does **not** invent one.
- **`vector` remains a derived value.** U5 adds **no** flag write: `get_engine_status` stays the only
  producer (`src/store/mod.rs:4193-4232`, `vector: self.snapshot().vectors.is_some()` at `:4213`) and the
  boot keeps writing **no** mask (`set_subsystems` stays inert, `:1868-1876`). U5's whole flag effect is
  therefore **what the snapshot contains**.
- **Ordering (pinned — expressed with the surface table's `Option`-returning build; REMAND-1 makes the
  failure branch's order explicit too, so the two branches cannot be read as one).** The snapshot the boot
  hands to `boot_wiring` **is** the snapshot the build's result composes into:
  `let built = build_boot_vector_index(&store, provider_ref).await;` → compose
  `let snap = DerivedIndexes { vectors: built.ok().flatten(), ..boot_snapshot };` (a **failed** build leaves
  `vectors: None`, per the failure table below) → `boot_wiring(provider, &snap)` →
  `swap_snapshot(snap)` → `set_engine_state(state)` (+ `set_embedding_provider` in the `Reachable` case
  only).
- **Ordering (b) — the failure branch (pinned verbatim by REMAND-1).** **The failed-`Reachable`-build
  branch writes the SAME steps in the SAME order** — build → compose → `swap_snapshot` of the unchanged
  snapshot → **`set_embedding_provider(provider_ref)`** →
  `set_engine_state(boot_wiring(BootProvider::Unreachable, &snap).0)`, with the returned flag vector
  discarded. Its **only** behavioural differences from the `Unreachable` branch are (a) the provider is
  wired (the probe succeeded ⇒ the store's derived status reports `embedding:true`) and (b) the reason the
  index is absent is a failed build rather than a failed probe — the observable outcome is identical
  otherwise (`Degraded`, `vector:false`, the store's fixed `last_error`). **A failing boot MUST NOT skip
  the provider wiring:** skipping it would make `P-IM-15`'s `embedding:true` unachievable, and wiring it
  after `set_engine_state` would make no observable difference (the flags are derived at read time) but is
  **not** the pinned order. The returned `EngineSubsystems` remains the **assertion surface**, never an
  applied mask (BLOCKING
  item 4 of §9.5.3.2). Whether that composition lives in `main()` or in a further lib helper is the
  Implementer's choice; the **observable** (the table above) is not.

**(2) What corpus is embedded? (entry question 2)** — the **boot snapshot's own store state**, i.e. every
node of every document held by the store at boot:

| corpus rule | pinned value |
| --- | --- |
| the unit of the index | a **node**, keyed `(documentId, nodeId, FieldType::Full)` — the key shape `vector_leg` walks (`src/store/mod.rs:5083-5093`) |
| which nodes | **every** node of **every** document in **every** shard (`StoreShard.docs: HashMap<DocumentId, Arc<Document>>`, `:1539-1542`; `Document.graph.nodes: Vec<Node>`, `:180-183`) — **all wikis**, no wiki filter (the leg wiki-scopes at read time, `document_wiki_id`, `:5058-5065`; **the corpus-scope pin is stated once below**) |
| **the corpus is key-unique within a document (pinned — REMAND-4 NOTE 4, 2026-09-22)** | **a node is identified by `(document_id, node_id)` *within its document*: two nodes of one `Document.graph.nodes` sharing one `NodeId` are a corpus that is NOT valid for this register, and a TestWriter's generator MUST NOT produce one.** Because the key shape is `(documentId, nodeId, FieldType::Full)` (`VectorIndex.entries` is a keyed `HashMap`, `:798-812`), such a pair would land on **one** key, so `entries.len() == n` would fail on an input the contract never certified — the exclusion is what keeps `n` well defined (the adversarial boundary this table pins is **a duplicate text under two *distinct* ids** ⇒ two keys, which is *not* this case). **The index pins one key per embeddable node** — not two — and no row, cap or strategy id changes: the row that would otherwise quantify over a duplicate-`NodeId` document is `P-IM-10`/`P-IM-11`/`P-TP-5`, and all three cite this row |
| the text embedded | the node's authored `value` (`Node.value: Option<String>`, `:154`) — **exactly one `embed` call per embeddable node**, no prefixing, no truncation, no title/tag/factKey concatenation |
| non-embeddable nodes | a node whose `value` is `None` contributes **no** entry (there is no text to embed) — **it is not an error** |
| **empty store** | the build **still runs** and produces `VectorIndex::default()` — `vectors.is_some()` is **`true` over zero vectors**, so a freshly booted server with a reachable provider reports **`vector:true`** and `mode=vector` returns **200 with an empty `results` array** (never `VectorIndexUnavailable`). If the store has nodes but **none** is embeddable, the outcome is identical |
| the `epoch` field | **unchanged** by the build (the build is not the epoch-driven rebuild vehicle; `DerivedIndexes.epoch` stays at the input's value, `0` for a boot's `DerivedIndexes::default()`) |
| `lexical` | **unchanged** (`None` at boot): the lexical leg is a live shard scan and needs no prebuilt index (§5.8's capability row, `lexical_rank`, `:4852`) — U5 builds **no** `LexicalIndex` |
| iteration **order** | deterministic and **pinned only up to determinism**: the build must visit the corpus in a **stable, input-derived order** that does not depend on `HashMap` iteration order, thread scheduling, wall clock or random state — the `entries` **key set** and each key's vector are what a row asserts (`strat:index-build-stability`), and a TestWriter MUST NOT assert a particular insertion order inside the `HashMap` |
| **vector comparison / `NaN` (PINNED by the post-red-phase register amendment, 2026-09-22 — the semantics `P-IM-12`'s "element-wise equal" and `P-TP-5`'s "verbatim" are read under)** | **The provider's returned `Vec<f32>` is compared as: (i) the two vectors have the same `len()`; (ii) for every index whose element is not `NaN`, the elements are identical (`a[i] == b[i]`, bit-for-bit, no tolerance, no `approx`/epsilon comparison, no normalisation, no re-ordering); and (iii) `NaN` is asserted only *in the `NaN` position*: `a[i].is_nan() == b[i].is_nan()` element-wise — i.e. a `NaN` at index *i* in the expected (provider-returned) vector requires `a[i].is_nan()` at that same index and **no other** assertion is made about that element, since IEEE-754 makes `NaN != NaN`.** The rule exists because this unit's adversarial corpus **deliberately contains `NaN`/`±∞`** (`P-TP-5`'s provider shapes), so a bare `Vec<f32> == Vec<f32>` could never hold of a legitimate `Ok(Some(_))` outcome; a row MUST NOT therefore assert plain `==` over a `NaN`-bearing vector, and MUST NOT treat a `NaN` element as evidence of a defect, a dropped key or an error. `±∞` **is** comparable and is compared by (ii) like any other non-`NaN` element. This rule covers the **build's own output** (the stored `Vec<f32>` vs the provider's return, the two-build equality of `P-IM-12`, the corpus-exactness of `P-TP-5`) and **not** any query-time scoring: no row asserts a `cosine`/`hamming` score over a `NaN`-bearing vector, and `cosine`'s dimension-tolerant shared-prefix reading (`:5333-5335`) is unchanged. **Excluded / no reading wider than this:** no row may assert a `NaN`'s sign, payload or bit pattern (Rust does not preserve them by contract here), and none may assert that a vector is `NaN`-free (no row pins that the *provider* returns finite values). |
| **`embed` call count and call discipline (moved into the contract by REMAND-1 — it is an invariant, not only a cost note)** | for a supplied provider the build makes **exactly *n* `embed` calls for *n* embeddable nodes** — **one call per embeddable node**, **strictly sequential** (no fan-out, no concurrency, no retry, no batching), **no call at all for a node whose `value` is `None`**, and **no call for any other text** (no titles/tags/`factKey`s/queries — ***and, pinned by REMAND-4 NOTE 5 (2026-09-22), **two nodes carrying the same `value` text are two embeddable nodes ⇒ they get **two** `embed` calls and **two** keys**; the pre-REMAND-4 phrase "no re-embed on a duplicate text" is **deleted** here because it contradicted the per-node call rule and the `P-TP-5`/`P-IM-10` two-key pins***). The **key set is bijective onto the embeddable corpus** (`entries.len() == n`, each key `(documentId, nodeId, Full)` of a `value: Some(_)` node, nothing dropped and nothing extra). **Both halves are pinned and both are asserted:** *one key per embeddable node* **and** *one `embed` call per embeddable node* — `P-IM-10`/`P-IM-11`/`P-IM-13` read the key half, the provider's own call counter reads the call half, and **`P-IM-13`/`P-IM-10`/`P-IM-11`/`P-SM-7`'s failure instances read the "no call for a `value:None` node" / "no call on a path that never embeds" half** (**REMAND-2 SHOULD-FIX 8, 2026-09-22: the pre-remand citation named `P-IM-13`/`P-SM-7` only; `P-SM-7`'s observable and its §9.5.4 note now carry the call-count instance explicitly — see `P-SM-7`'s `strat:boot-lifecycle-vector` observable — so this list and the row agree**). For a **supplied** provider `Ok(None)` is never the answer; for `p == None` the count is **0** |

**(3) Which index type / dimension, and what `vector` means for that choice (entry question 3).**

- **U5 builds the `full` field only** (`FieldType::Full`) — **not** the `binary` field. The multiple-vector
  fields principle (canonical `docs/specs/gnosis.md` §4.5.3a: a node/chunk may carry a `full` + a `binary`
  field for a fast first-pass ANN; §8's "Multiple vector fields" principle) is a **permission**, not an
  obligation: the coarse-to-fine first pass is already **degrading** by contract — `binaryFirstPass` with no
  `binary` field falls back to the full-field search **without error**
  (`src/store/mod.rs:5094-5102`, and §4.5.3a.4 as read at `:4287-4288`). **What a `binary` entry would
  actually be, pinned so no one invents an embedding step for it:** the landed first pass feeds the node's
  **binary field's `Vec<f32>` straight into `hamming`, which sign-binarizes it on the fly**
  (`binary_code(v: &[f32]) -> Vec<u8>`, `:5380-5382`; `hamming(a: &[u8], b: &[f32])`, `:5386-5397`) — i.e. a
  `Binary` entry is **the same node vector**, only stored a second time under a second key; it needs **no
  extra `embed` call** but it **doubles the index's per-node footprint**, and it changes what
  `binary_candidate_pool` can select from (`:5094-5102`, `:5015-5043`). Both facts make it a **deliberate
  design choice with a corpus-size cost**, which U5 does **not** take: the unit builds `Full` only and lets
  the documented fallback serve, so the honest meaning of `vector: true` under U5 is **"the full-field dense
  leg is wired for the current store and can be queried"** — with `binaryFirstPass:true` degrading to the
  full field, **not** reporting a missing capability. A consumer that wants the coarse first pass is asking
  for a **new** unit.
- **Dimension is not pinned by U5.** `VectorIndex.entries` holds a bare `Vec<f32>` (`:811`) with no
  dimension field, and `cosine` is already dimension-tolerant by contract — it compares the **shared prefix**
  (`:5333-5335`, the doc comment names the index-vs-query dimension difference from test providers). A
  TestWriter therefore MUST NOT assert a dimension, a model name, or a normalisation; **the provider's
  returned length is authoritative**.

**(4) Cost / latency and failure (entry question 4).**

| condition | pinned outcome | observable |
| --- | --- | --- |
| the store has *n* embeddable nodes | **exactly *n* `embed` calls**, strictly sequential (no fan-out, no concurrency, no retry) — **the rule itself lives in the contract table (2)'s call-count row**; this row records only its cost reading, and **`n` is store-wide** (the whole store, all wikis — REMAND-4 NOTE 6, 2026-09-22; `build_boot_vector_index` takes **no** wiki parameter, so no row's corpus is per-wiki) | the count witness in `P-IM-10`/`P-IM-11`/`P-IM-13` (all **store-wide** counts); the provider's own call count in a test impl |
| **the cost consequence of the scope (pinned — REMAND-4 NOTE 6, 2026-09-22)** | for an `Absent`/`Unreachable` boot **no** `embed` call is made (count `0`), so the scope's cost is paid only on a `Reachable` boot — but the build is still **attempted on the store-wide corpus** there, so the pre-bind cost scales with **the total corpus, all wikis**, and the cost is **not** reducible by the request's wiki: the build receives no wiki parameter and cannot be scoped by one. No timeout, node cap or deadline is pinned (§9.5.5's residual (i), unbudgeted boot latency) | `P-IM-11`'s call-count `0` instance for `Absent`/`Unreachable`; the residual list's item (i) |
| an `embed` call fails (any `Err`, incl. `EmbeddingUnavailable`) | **the build aborts and the index is dropped whole** — `Err(StoreError::EmbeddingUnavailable)`; a partial index is **never** installed or observed | `P-IM-13`; the returned `Err` and `snapshot().vectors.is_none()` |
| what the boot does with that `Err` (**the failure branch, pinned verbatim by REMAND-1 — order of calls, not just outcome**) | **no index**: the failed build's `Err` is discarded with the whole local index under construction, and the boot then (1) `swap_snapshot(snap)` where `snap = DerivedIndexes { vectors: None, ..boot_snapshot }` — the **unchanged, index-free** snapshot (`built.ok().flatten()` ⇒ `None`), (2) **`set_embedding_provider(pr)`** — the provider **is** wired, because the provider probe **succeeded** (this is the one behavioural difference from the `Unreachable` branch), (3) `set_engine_state(st)` with `(st, _) = boot_wiring(BootProvider::Unreachable, &snap)` — the **existing** degraded state (`Degraded`), i.e. the *derived state* of the `Unreachable` branch and **not** a fourth branch; the flag vector that call returns is **discarded** (the store derives its own at read time, so the failure branch writes **no** mask). The resulting derived status is `state: Degraded`, `vector:false`, `embedding:true`, `reranker:false`, core `true`, with the store's **fixed** `last_error` — the same call order as the `Unreachable` branch, whose only other difference is that it wires no provider (`embedding:false`) | the pinned call order above; `boot_wiring(BootProvider::Unreachable, &snapshot)`'s own `(Degraded, …)` **state** (its flag vector is not applied anywhere in any branch); the store's read-time derivation (`src/store/mod.rs:4204-4231`) is what the assertions read (`P-IM-15`) |
| the **wire** consequence | **none new:** `GET /engine/status` still answers **200** always with the honest vector; `mode=vector` returns `VectorIndexUnavailable` → **503 `vector_index_unavailable`** (FS-14) **on a READY store with `vectors: None`** — on the failed-build branch itself the applied `Degraded` state means a non-`graph` query returns **FS-8 `EngineUnavailable` → 503 `engine_unavailable`** (the READY gate precedes every leg check, §5.9's `Precedence (pinned)` bullet (:1585-1592; re-read this pass); ***REMAND-2 MUST-FIX 1/2, 2026-09-22 — this is what the live battery's `R-L3` (v) asserts***); the `Degraded` state carries the store's **existing fixed** `last_error` string (`src/store/mod.rs:4226-4230`), which is **not reworded** by U5 (see the residual list's item (v)) | §5.8's always-200 row; §5.9's FS-14 row; §11's 21-row map (unchanged) |
| boot latency | the build is **not** separately budgeted by U5 — no timeout, no node cap, no deadline; the only bound is whatever the provider itself imposes (the bin's probe already carries a per-request timeout, `src/bin/gnosis_server.rs:364-375`) | **residual risk, recorded** (§9.5.5's residual list and the open question below): a slow or hanging provider can delay the loopback bind because the build runs before `bind` |
| mid-build provider death / partial write | impossible by construction: the index is built into a **local** `VectorIndex` and only ever installed through one `swap_snapshot` of a fully built `DerivedIndexes` (the IMMUTABLE-DERIVED-SNAPSHOT rule, `:1849-1860`) | `P-IM-13` |

**(5) Determinism / hermeticity — and whether U5 needs a new knob (entry question 5).**

- **No new env var and no new CLI flag.** U5 adds **no** config surface: the boot build is unconditional for
  a reachable provider and has no user-tunable parameter. The bin's complete configuration surface **today**
  is `--port` (`src/bin/gnosis_server.rs:46-56`, default `8080`), `GNOSIS_SERVER_OLLAMA_URL` (no default —
  unset ⇒ `Absent`) and `GNOSIS_SERVER_OLLAMA_MODEL` (default `nomic-embed-text`) — the only three
  `std::env`/`args` reads in the bin (`:47`, `:321`, `:322`; a whole-`src/` sweep finds no others outside
  `gnosis_eval`'s four, `src/bin/gnosis_eval.rs:80-82`, `:311-312`), and U5 changes none of them.
- **Register obligation recorded (pinned, because nothing asserts it):** **no test or spec in this repo
  asserts the set of env vars or CLI flags the `gnosis-server` bin reads.** Should a later unit want a knob
  (a boot-build timeout, a node cap, a "skip the build" switch), that knob is a **new register obligation**:
  the unit must pin its name, its default and its no-set behaviour **and** an assertion that the documented
  set is closed (the natural home is §5.6/§5.7-style bounds discipline or a §11 `P-IM-*` row). **U5 proposes
  no knob and therefore owes none** ✓.
- **Hermetic behaviour (must be pinned, since the suite is green with and without the provider env var).**
  The boot build is reachable **only** through `BootProvider::Reachable`, which requires a provider that the
  probe found reachable. So every hermetic path is deterministic: the existing spawn helpers strip the
  provider (`spawn_server_without_provider`, `tests/gnosis_server_e2e.rs:1284-1310`, which
  `env_remove`s **both** `GNOSIS_SERVER_OLLAMA_URL` and `GNOSIS_SERVER_OLLAMA_MODEL`) ⇒ `Absent` ⇒ **no
  build, no embedding call, `vector:false`** — while the ambient-env spawner (`:59-86`) may boot `Ready` and
  therefore **may** build an index (over an empty store: an empty index, `vector:true`). **A U5 test MUST
  inject a deterministic in-memory `EmbeddingProvider` through the lib seam and MUST NOT depend on a live
  provider or on the ambient environment** — the same discipline as U3's `P-IM-9` (a pure/injected seam) and
  the house hermeticity rule (the U3 landing record: green with and without `GNOSIS_SERVER_OLLAMA_URL`,
  §9.5.2's landed bullet). The provider-**reachable** case is therefore exercised at **lib level** (injected
  provider) and its **live** half is the battery's new row `R-L3` — now carrying that row's PASS criteria
  (i)/(ii)/(iv) **and (v), the failed-build outcome** (REMAND-1). ***(POST-GREEN SPEC AMENDMENT, 2026-09-22 —
  superseded in place: (v) is **PARKED**, not executable live, so the clause's executable set is
  (i)/(ii)/(iv) alone. See §9.5.5's adjudication note 1 extension and the battery's U5 amendment item 5 for the
  park reason, the lib-level home and the un-park trigger; the (v) text above stands as the pre-green
  record.)***

**(6) Interaction with U4 (HELD) and the durability direction (unauthorized) — entry question 6.**

- U5 **MUST NOT** touch: the **change cursor** (accessor, `epoch()`/`journal_len()` semantics,
  `JournalEntry`'s shape), **any route** (the 14-row table and `route_bijection()` are U4's amendment
  surface, §5.2's amendment rule), the **paged reads**, `GET /changes`, the **journal** (the build appends
  **no** journal entry and does not move `epoch()`), any **persistence/durability** surface (the store is
  in-memory; `ENGINE-DURABLE-CORPUS-DIRECTION` is **DIRECTION ONLY / NOT ACTIVE**, and the durability design
  unit is **not authorized** — `docs/specs/gnosis-grq-inbound-review.md` §14 Q2 = "(B) NOT before …"), and
  the derived-index **rebuild** vehicle (the epoch-driven rebuild is a §4.5/§4.2 concern with no
  implementation in scope here — U5 builds **once, at boot**).
- U5 **MUST NOT** edit `EngineSubsystems`, `HealthReport`, `src/store/mod.rs`'s construction literal
  (`:1718-1725`, dead value after U3), the §11 map (21 rows), the `StoreError` taxonomy (21 variants), the
  canonical `docs/specs/gnosis.md`, `Cargo.toml`, or `docs/decisions.md`/`docs/defects.md`/
  `docs/pending.md`/`docs/next-steps.md`/`docs/HANDOFF.md` (trackers are the supervisor's).

**The corpus seeding clause (pinned by REMAND-1, so no row needs an invented API).** Every populated state this
register quantifies over is produced by the **public `RagStore` surface** (surface #7), never by a private
field, a serde round-trip or a hand-built `Document`. The recipe, verbatim, with the real signatures
(`src/lib.rs:120-133` re-exports every type named here):

1. `let store = Store::new();` (`src/store/mod.rs:1690`).
2. `let wiki: Wiki = store.create_wiki("w").await.expect("wiki");` then `let w = wiki.wiki_id;`
   (`RagStore::create_wiki(&self, name: &str) -> Result<Wiki, StoreError>`, impl `:2806`) — **one** wiki is
   enough for every row (no row asserts a cross-wiki distinction; the corpus is store-wide and the *leg*
   wiki-scopes at read time).
3. `let doc: Document = store.create_document(&w, CreateDocumentRequest { title: "t".to_string(), tags:
   None, author: None }).await.expect("doc");`
   (`RagStore::create_document(&self, wiki_id: &WikiId, request: CreateDocumentRequest) -> Result<Document,
   StoreError>`, impl `:2381-2385`) — **each** created document starts with an **empty** graph
   (`Graph { nodes: vec![], edges: vec![] }`, `:2406-2409`), so it contributes **no** node and therefore
   **no** index entry until step 4 seeds one.
4. `let cur = store.get_document(&doc.document_id).await.expect("current");` (impl `:2426`) then
   `let cur_rev = cur.revision;` (`0` on a freshly created document) and
   `store.update_document(&doc.document_id, UpdateDocumentRequest { base_revision: cur_rev, graph,
   title: None, tags: None }).await.expect("graph");` (impl `:2435-2449`) where **`graph` is exactly the
   corpus the row wants**:
   `Graph { nodes, edges }` with `nodes: Vec<Node>` — each seeded node
   `Node { document_id: doc.document_id.clone(), node_id: NodeId("n1".to_string()), kind: NodeKind::Content,
   value: Some("<text>".to_string()), fact_key: None, target: None }` (`Node` `:149-159`: **`value:
   Some(_)` makes it embeddable**, `value: None` makes it non-embeddable) — and
   `edges` carrying **exactly one `EdgeKind::DocHead` edge and one `EdgeKind::DocEnd` edge** (each
   `Edge { source: (doc.document_id.clone(), NodeId(…)), target: (doc.document_id.clone(), NodeId(…)),
   kind: EdgeKind::DocHead | EdgeKind::DocEnd, state: None, cross_wiki: false, relation_type: None }`,
   `Edge` `:163-176`). **Why the two edges are mandatory:** `update_document` validates the graph with
   `valid_provident_graph` (`:5449-5461`), which requires **exactly one** `DocHead` **and one** `DocEnd`
   edge — a node-only graph is rejected with `StoreError::ValidationError`, and no U5 row may assert an
   `Err` from its own seeding. The existing integration fixture shows the same shape
   (`tests/rag_query_integration.rs:124-144` — `wellformed_graph`; `:146-178` — `new_wiki`/`new_doc`/
   `apply_graph`; the node builders at `:72-94`).

**The corpus a row may then reason about (pinned):** the store's whole corpus is the **sum over its
documents** of their `graph.nodes` (all wikis, all shards); a freshly created document adds none, and no
U5-reachable call adds a node that did not come through step 4 — so for a TestWriter's store the embeddable
count is **exactly the number of seeded nodes whose `value` is `Some(_)`** (a `fact`/`content` node with
`Some("")` counts; a node with `None` does not), and that is the `n` in `P-IM-10`/`P-IM-11`'s count witness
and in §9.5.5(2)'s call-count rule. "Shards populated out of id order" is produced by **seeding more than
one document** (each `create_document` mints `doc-<n>` and `Store::shard_for` hashes the id into one of the
**64** shards, `SHARD_COUNT` `:1534-1535`, `shard_for` `:1930`) — the row asserts the **key set**, never a
shard order. **No row may assert the number of documents, the document ids, or the shard a document lands
in.** **The corpus scope is pinned (REMAND-4 NOTE 6, 2026-09-22): the corpus is the whole store, all wikis —
`build_boot_vector_index` takes no wiki parameter, and no row's corpus is per-wiki.** Only the *leg* scopes by
wiki, and it does so at **read** time (`document_wiki_id`, `:5058-5065`): for an `Absent`/`Unreachable` boot the
build is still attempted-or-skipped on that same store-wide corpus, so the pre-bind cost scales with the total
corpus and every call-count witness in this register is stated **store-wide** (see §9.5.5(4)'s cost row). A row
that wants the **empty** store (`P-IM-11`'s empty-store pin, `P-IM-13`'s zero-node instance)
stops after step 1.

**Valid/happy + fail states (TestWriter assertion guide, U5).**


| # | state | input | outcome |
| --- | --- | --- | --- |
| 1 | happy — provider reachable, populated store | `Reachable(p)`, a store with *n* embeddable nodes | `build_boot_vector_index(s, Some(&p)).await == Ok(Some(vi))` with exactly the *n* `(doc, node, Full)` keys; the boot composes `snap.vectors = Some(vi)`; `boot_wiring(Reachable, &snap).1.vector == true`; `subsystems` = `{store:true, graph:true, lexical:true, vector:true, embedding:true, reranker:false}`, state `Ready` |
| 2 | happy — provider reachable, **empty store** | `Reachable(p)`, a store with no documents | `Ok(Some(VectorIndex::default()))` (`entries` empty — **not** `Ok(None)`); `snap.vectors.is_some()` ⇒ `vector:true`; `mode=vector` ⇒ `Ok(RagResult)` with an **empty** `results` |
| 3 | happy — non-embeddable nodes only | nodes whose `value` is `None` (and/or `Some("")`) | `None`-valued nodes contribute **no** key; `Some("")` **does** contribute one; the index is still `Some` (so `vector:true`) — `Ok(None)` is **never** the answer for a supplied provider |
| 4 | fail — mid-build embedding error | a provider whose *k*-th `embed` returns `Err(EmbeddingUnavailable)` | `Err(EmbeddingUnavailable)`; **no** index installed (the boot swaps the unchanged snapshot ⇒ `vectors.is_none()`, `vector:false`, state `Degraded`, `last_error` = the store's fixed string) |
| 5 | fail — no provider / unreachable provider | `Absent` / `Unreachable` | `build_boot_vector_index(s, None).await == Ok(None)` (**not** an error) / **no** build is attempted, `vectors` stays `None`, `vector:false`; and the boot's own `EngineState` pair (`Unavailable` for `Absent`, `Degraded` for `Unreachable`) leaves the store **non-READY**, so `mode=vector` ⇒ `Err(StoreError::EngineUnavailable)` (FS-8) → **503 `engine_unavailable`** — **NOT** `VectorIndexUnavailable` (***REMAND-2 MUST-FIX 1, 2026-09-22 — this cell previously read "…`mode=vector` ⇒ `VectorIndexUnavailable` → **503** (FS-14) — the pre-U5 behaviour, unchanged for these two outcomes", which is unsatisfiable on a non-READY store: §5.9's precedence bullet puts the READY gate before every leg check for non-`graph` modes, so FS-14 needs a **READY** store with `vectors: None`; that instance is state 6 below.***). FS-8 here is the **pre-U5 behaviour for these two outcomes, unchanged** |
| 6 | fail — an unbuilt index is asked to serve | a store whose snapshot has `vectors: None` (any caller-built store, incl. the U5 unit's own negative probe) | `StoreError::VectorIndexUnavailable` (FS-14) — **still reachable**, because the store-level API accepts a snapshot with no index (`tests/rag_query_integration.rs:689-716` constructs exactly that store); U5 does **not** retire the variant or the §11 row |
| 7 | fail — built index, provider gone/unreachable at query time | `vectors: Some(_)` but no wired provider | `StoreError::EmbeddingUnavailable` (FS-13) → 503 (§5.9's order pin: the index is checked **before** the provider, `src/store/mod.rs:4448-4454`) |

**Numeric / census claims made by this section (each verified this pass).** the three `BootProvider`
inputs (`src/lib.rs:70-77`); the **six** flags and **21** `StoreError` variants/§11 rows (unchanged);
the **14**-row route table (unchanged); the **23** `EngineSubsystems` struct literals across `tests/` that
the frozen-type rule protects (re-counted this pass — SHOULD-FIX 6; the pre-remand "25" was an over-count.
The per-file enumeration is published **once**, in §5.8 above and in F2 §9.1's F7 census —
`docs/specs/engine-wire-contract.md`'s `EngineSubsystems`-is-FROZEN note, which carries the same **23**:
`tests/wire_conformance.rs` 5, `tests/props_wire.rs` 5, `tests/props_gnosis_server.rs` 10,
`tests/blind_u3_status_honesty_greens.rs` 3, `tests/rag_query_integration.rs` 0); U5's
**8** register rows and its executed-layer cap **355 ≤ 400** (per-row caps
`60/50/40/45/45/30/45/40`, each ≤ 100; measured layer **314 executed / 355 caps** — the pre-amendment
`340 ≤ 400` and the pre-amendment `45/50/40/45/45/30/45/40` split are **superseded** records, kept in the
execution plan above); the bin's **three** configuration reads
(`src/bin/gnosis_server.rs:47`, `:321`, `:322`); **`n`** embed calls for **`n`** embeddable nodes. ***(POST-GREEN
SPEC AMENDMENT, 2026-09-22 — VERIFIED against the landed layer by this pass; no figure in this block needed
changing.)*** The U5 layer figures in this block were re-read this pass against the landed suite's own constants:
the caps at `tests/props_gnosis_server.rs:361-368` are exactly `B_U5_IM10 = 60`, `B_U5_IM11 = 50`,
`B_U5_IM12 = 40`, `B_U5_IM13 = 45`, `B_U5_IM14 = 45`, `B_U5_IM15 = 30`, `B_U5_SM7 = 45`, `B_U5_TP5 = 40`
(Σ = **355** ≤ 400, every row ≤ 100) and the executed layer at `:375-382`/`:387` is
`P-IM-10` **55** / `P-IM-11` 40 / `P-IM-12` 40 / `P-IM-13` 42 / `P-IM-14` 25 / `P-IM-15` **27** / `P-SM-7` 45 /
`P-TP-5` 40 with `U5_EXECUTED_TOTAL = 314` (Σ = **314 ≤ 355 ≤ 400**) — so the caps **355**, the executed total
**314**, the per-row `55/60 · 40/50 · 40/40 · 42/45 · 25/45 · 27/30 · 45/45 · 40/40` reading and the binary total
`310 + 400 + 127 + 355 = 1192` (§9.5.3's per-unit scope pin) all agree with the layer the spec pins, and the
"a cap is a MAXIMUM, not an expected count" rule is what makes `55 < 60` and `25 < 45` compliant rather than a
contradiction.

**What U5 must move in the same unit (with `file:line`) — the golden-literal discipline (§5.8; F2 §12).**

| artifact | current (pre-U5) | what U5 makes it | why it must move in this unit |
| --- | --- | --- | --- |
| `tests/wire_conformance.rs:718` — `honest_ready_subsystems()` (**gate-8 re-read, 2026-09-22 — the moved anchors: the `fn` is at **`:718`**, its body `:719-726`, its post-U5 doc comment `:708-717` (which already states "`vector` **true** — the post-U5 value" and the V-8.1 scoping); the `:713-723` / `:713-722` spellings this row and REMAND-1 carried were U3-time and are superseded — `:713` is a doc-comment line and `:722` is `lexical: true`*) | `vector: false` (its doc comment at `:707-712` still says "`vector` **false** at U3-time … U5 flips this one value") | **`vector: true`** (the comment reconciled in the same unit: the fixture now projects the post-U5 READY mask of §9.5.5's contract table (1)) | it is the **fixture** the V-8.1 literal is the projection of (the literal is at `:1109`, its message at `:1110-1111` — **gate-8 re-read: the `:1095`/`:1096` anchors this row carried are U3-time and stale**); leaving it false would make V-8.1 disagree with the honest READY mask U5 produces |
| `tests/wire_conformance.rs:1093-1097` — the V-8.1 byte-exact literal's assertion (`v8_health_reports_exact`, the `fn` at `:1084`; the **amended-U3** golden's full extent is `:1084-1116`, V-8.2's assertion at `:1107-1111`) | V-8.1's JSON at `:1095` has `"vector":false` and its message at `:1096` says "vector false until U5" | **V-8.1 flips to `"vector":true`** (one value at `:1095` + its message at `:1096`, which becomes the post-U5 READY-mask wording — **REMAND-2 SHOULD-FIX 4**); **V-8.2 is unchanged** (`vector:false`, `embedding:false`, `reranker:false`, same `lastError`, `:1109`) | F2 §12's V-8.1 clause names U5 as the unit that flips that one value (`docs/specs/engine-wire-contract.md` §12's V-8.1 stage clause and its U5 note); V-8.2's `vector:false` stays honest because a degraded (unreachable-provider) boot attempts **no** build |
| `tests/wire_conformance.rs:1208-1223` — the `boot_wiring_couples_to_the_derived_read` probe **(1)** ("V-8.1's producer") **— this is the probe that breaks when the fixture flips; gate-8 re-read, 2026-09-22: the landed anchors are the comment `:1208`, the input `boot_snapshot(true)` at **`:1213`**, the `assert_eq!(reached, honest_ready_subsystems())` at `:1214-1218` and the hard-coded `reached.embedding && reached.vector && !reached.reranker` at **`:1222`** with its message at **`:1223`**; the `:1191-1206` / `:1195` / `:1198-1202` / `:1203-1206` / `:1204` / `:1205` / `:1191` spellings this row and REMAND-1/REMAND-2 carried are U3-time and are superseded (that block shifted by the REMAND-driven edits and the comment insertions) — kept here as the pre-move record** | `boot_snapshot(false)` at **`:1195`** (a traceless snapshot) **and** the hard-coded `assert!(reached.embedding && !reached.vector && !reached.reranker, …)` at **`:1203-1206`** (whose `!reached.vector` goes red the moment the fixture says `vector:true`); **plus** the two prose texts the same edit falsifies — the assertion **message** at **`:1205`** ("the honest READY vector is `embedding:true` with no index and no reranker: …") and the comment at **`:1191`** ("a REACHABLE provider is wired ⇒ `Ready`, no index") | **exactly two** *value* **edits (both landed):** `:1213`'s input is `boot_snapshot(true)` (an **index-bearing** snapshot, so the probe exercises the post-U5 boot shape) **and** `:1222` asserts `reached.vector` (the assertion keeps its `embedding`/`reranker` halves); the `assert_eq!(reached, honest_ready_subsystems())` at `:1214-1218` is **unchanged** — it is the line that makes the probe the amended fixture's producer, and it now matches because `honest_ready_subsystems().vector` is `true` (`:723`). **The same unit also reconciled the two prose texts (SHOULD-FIX 4, landed):** `:1223`'s message is the post-U5 READY-mask wording ("the honest READY vector is the post-U5 `embedding:true` with an index and no …") and the comment at `:1208` is restated as the index-bearing input ("a REACHABLE provider is wired ⇒ `Ready` with an index").** | the probe asserts the store's derived read **equals** the fixture (now `vector:true`); with a traceless snapshot it would correctly derive `vector:false` and fail against the amended fixture, and with `!reached.vector` left in place it would fail even **after** the input flip — the fixture, the probe input, the hard-coded assertion **and the two prose texts that describe them** are **one** obligation |
| `tests/wire_conformance.rs:1228-1242` — probes **(2)** (provider `Absent`, **`:1228`**) and **(3)** (provider `Unreachable`, **`:1237`**) — *gate-8 re-read, 2026-09-22: the `:1208-1224` / `:1209` / `:1218` / `:1210-1214` / `:1219-1223` spellings this row and REMAND-1/REMAND-2 carried are U3-time and stale (the block shifted with the probe-(1) edits); kept as the pre-move record* | each passes `boot_snapshot(false)` and asserts equality with `honest_degraded_subsystems()` (**`:1229-1233`**, **`:1238-1242`**) | **unchanged — these two stay on `false`.** Their fixture is `honest_degraded_subsystems()` (`:740`, `vector:false`) and their wiring wires **no** provider (the `wired` argument is `None`), so an index-bearing snapshot would make `boot_wiring`/the derived read return `vector:true` and **falsify a correct** degraded assertion | REMAND-1's correction: the pre-remand move-table text ("each passes `boot_snapshot(false)` → pass an index-bearing snapshot") read as if **all** the probes moved; only probe (1) does. A TestWriter who flipped `:1228`/`:1237` would break the V-8.2 producer couple. **REMAND-2 MUST-FIX 3 re-points the same correction into `docs/specs/engine-wire-contract.md` §12's U5 note (landed)**. ***REMAND-4 SHOULD-FIX 3, 2026-09-22 — TestWriter obligation recorded here (this pass edits no test file):*** the fixture's own doc comment at `tests/wire_conformance.rs:724-725` (**gate-8 re-read, 2026-09-22 — MOVED and DISCHARGED: the degraded fixture's doc comment is now `:729-739`, whose scope note already states exactly what this obligation asked for — "this fixture is the `Absent`/`Unreachable` instance only. U5's failed-`Reachable`-build branch is ALSO `Degraded` while it **wires** the provider … and that mask has no golden of its own (it is asserted at lib level by U5's `P-IM-15`)". The `:724-725` spelling this cell carried is stale; the obligation is satisfied by the landed test edit, and the cell is kept as the record*** — "§9.1 / §12 V-8.2 (U3) — the HONEST flag vector a DEGRADED engine (provider unreachable) reports: no `embedding` claim, no index, no reranker") must be **scoped in the same unit's test edit** to the provider-**`Absent`/`Unreachable`** boot: after U5 the comment's universal reading is over-claimed, because U5's failed-`Reachable`-build branch is also `Degraded` while **wiring** a provider (`vector:false`, `embedding:true`, `reranker:false`, the same fixed `lastError`) — and that mask has **no golden of its own** (`docs/specs/engine-wire-contract.md` §12's V-8.2 clause; `P-IM-15`; the live row `R-L3`'s PASS (v)). **No value, key or assertion in the fixture moves — only that two-line prose** |
| `tests/wire_conformance.rs:1244-1256` — probe **(4)** "the U5-time shape" — *gate-8 re-read, 2026-09-22: the landed anchors are the comment `:1244-1246`, the input `boot_snapshot(true)` at **`:1250`**, `let u5_expected = honest_ready_subsystems();` at **`:1253`** and the assertion `:1254-1256`; the `:1225-1239` / `:1231` / `:1234` / `:1235` / `:1228-1239` spellings this row carried are U3-time and stale — kept as the record* | `u5_expected = honest_ready_subsystems();` + the redundant `u5_expected.vector = true;` | the **assertions stay** (they now pin the same shape probe (1) exercises), and the redundant `u5_expected.vector = true;` line **is gone** — **gate-8 verified, 2026-09-22: no such assignment exists in the landed block** (after the fixture flip `honest_ready_subsystems().vector` **is** `true`, so the pinned form was to drop it, and **no** assertion changed) | it is the forward pin §15.4.6 named as "must move in the same unit as a change"; U5 is the unit it was waiting for. Leaving the pre-U5 move-table text ("unchanged assertions (kept)") would have left a redundant write next to an amended fixture |
| `tests/wire_conformance.rs:1288-1299` / `:1319-1330` — the frozen key sets (`health_report_shape_frozen`) | six flag keys + six top-level keys | **unchanged** (assert-only) | the §15.4.6 trip-wire: U5 adds **no** key anywhere, so these stay the proof that it did not |
| `tests/gnosis_server_e2e.rs:1386-1391` — `flags["vector"] == false` (its comment at `:1386` says "the boot leaves `vectors:None` ⇒ `vector:false` until the U5 index build") | asserted on a `spawn_server_without_provider` boot (the helper at `:1284-1310`; provider **Absent** ⇒ no build) | **unchanged assertion** (the value and both `serde_json::Value::Bool(false)` comparisons stay), comment reconciled in this unit (the reason becomes "provider absent ⇒ no index is built", not "U3-time") | it stays **green by construction** under the pinned contract (entry question 1); a U5 that built an index for an absent provider would falsify it — the trip-wire working as intended |
| `docs/specs/p2-gnosis-server-live-pending-battery.md:125` — `R-L2`'s PASS criterion | `subsystems.vector == false` for a provider-Reachable boot (plus its FAIL clause, which lists `vector:true` as a U3-time failure) | **`vector: true`** **in the post-U5 tree** (the dated note already sits in the cell, `:125`); the row's 2026-09-17 executed-live record at **`:186-208`** (***re-read this pass — the pre-pass citation `:182-204` is superseded: the POST-GREEN SPEC AMENDMENT of 2026-09-22 added item 5 to the U5 amendment block and shifted the battery's tail; the record's heading `R-L2` executed live — PASS (2026-09-17 …)` is now at `:186` and its paragraph ends at `:208`, while `docs/specs/p2-gnosis-server-live-pending-battery.md:126` (`R-L3`) and its `:102-126` section span below the tables are **unchanged***) **stands as the U3-time observation** and is not rewritten (+ the battery's new **`R-L3`** row, `:126`, for the build itself and its failure branch — **which is now PARKED, see §9.5.5's adjudication note 1 extension**) | `R-L2`'s criterion is a **live** expectation; left unamended it reports a false FAIL on the first post-U5 live run (§9.5.5's live classification). Until a runner re-executes it post-U5, the row reads "PASS at U3-time; re-run under U5's tree for the amended cell" |
| `docs/specs/engine-wire-contract.md` **§9.1's `vector` flag row** (`:1078`) + **§12's V-8.1 stage clause** (its U5 note, `:1367-1386`) and **§12's V-8.1/V-8.2 U5 notes** (incl. the moved `tests/wire_conformance.rs` anchors it cites: fixture `:713-723`, literal `:1093-1097`, probes `:1192-1239` — REMAND-1 re-read them against the current file, and **REMAND-2 MUST-FIX 3 re-points the probes clause to probe (1) alone**) | "the boot leaves `vectors: None` … until U5" / "U5 owns the post-boot-index value" | a **U5-labelled dated note** on each, additive only | §9.1 is the **contract home** of the flag semantics and §12 is the golden home; the notes record the outcome of the question they explicitly leave to U5 — no type, key set or map is touched |
| **`src/store/mod.rs:4211-4212` — the comment at the `vector` flag's derivation site (`get_engine_status`), *"the boot leaves `DerivedIndexes::default()` ⇒ `false` until U5's boot index build"* (IMPLEMENTER obligation, in the same unit; added by the post-red-phase register amendment 2026-09-22)** | the comment states the pre-U5 reason for `vector: false` at boot (the boot swaps an index-free `DerivedIndexes::default()`) | **the comment reconciled in the same unit as U5's code** — once the boot builds an index for a `Reachable` provider the stated reason no longer holds (the boot then installs an index-bearing snapshot and the flag is the derived `snapshot().vectors.is_some()`), so the comment is restated against the post-U5 predicate (the same `Reachable`-builds / `Absent`-`Unreachable`-keep-it-empty split this section's contract table (1) pins, with the failed-`Reachable`-build branch of `P-IM-15` leaving the snapshot index-free while `embedding:true`) | it is a **comment-only** obligation (no value, key, assertion or behaviour moves — the derivation is already correct and read-time, §9.5.5's contract table (1) and adjudication note 2) but it is **stale the moment U5 lands**; the same-unit rule that binds the `tests/wire_conformance.rs` fixture/comment pair and §5.8's `vector` row binds this site too. **This spec pass does NOT edit `src/`**: the obligation is recorded here so the Implementer's U5 diff carries it and the documentation reviewer can check it |

**U5 adjudication notes (pinned so the rows are not read wider than they are).**

1. **The build is a lib seam, so every row is lib-level; exactly one row is live-only.** The boot's `main()`
   is a `[[bin]]` target and unreachable from `tests/` (§9.5.2's U3 note 1) — U5 therefore keeps the F11
   pattern and puts the build **in the lib** (row #1 of the surface table), so rows `P-IM-10`…`P-IM-15`,
   `P-SM-7` and `P-TP-5` are all assertable with `#[tokio::test]` + a real `Store` + an injected provider,
   **without** a live server. The **live-only** obligation is the end-to-end consequence on a booted bin
   with a **controlled provider**, and it is **homed** in the battery's new row **`R-L3`** (following the
   `R-L2` precedent, `docs/specs/p2-gnosis-server-live-pending-battery.md:102-126`); **no row of this
   register is left without a home**. **REMAND-1 addition (the failure branch's home, so `P-IM-15` is not
   lib-only by accident):** the battery's `R-L3` row carries the failed-build boot as its PASS criterion
   (v) — a controlled provider that answers `/api/tags` (so the probe **succeeds**) but **errors** on
   `/api/embed` ⇒ the live bin reports `state:"Degraded"`, `subsystems.vector == false`,
   `subsystems.embedding == true`, `reranker:false`, core `true`, the fixed `lastError`, and its
   `mode=vector` probe is **503 `engine_unavailable`** (FS-8 — the `Degraded` state of the failed-build branch is **non-READY**, so the pre-READY gate fires before the leg check; ***REMAND-2 MUST-FIX 2, 2026-09-22: this clause read `503 vector_index_unavailable` and is superseded in place — the superseded reading would have made the live row report a FALSE FAIL against correct code, the class `p2 §5.9's Interlock with U5` bullet (`docs/specs/p2-gnosis-server.md:1643-1645`; re-read this pass — `:1640-1642` was this file's REMAND-2/3 spelling and is superseded by REMAND-4 MUST-FIX 1, 2026-09-22, that line being §5.9's `Testable assertions the ruling yields` clause) and §6 forbid***), never `Ready`, never `vector:true`. So the
   failure branch is assertable **twice**: lib-level (`P-IM-15`, per the order pinned in §9.5.5's contract
   table (4)) and live (`R-L3` (v)). **No third home is created and no lib test replays a `[[bin]]` path.**
   ***POST-GREEN SPEC AMENDMENT (2026-09-22) — ADDITIVE; the clause above is the pre-green record, and this
   note's extension (a)–(d) is the authority for the failure branch's status.*** **(a) The
   failed-`Reachable`-build criterion's** ***live*** **home is PARKED, and the park reason is structural.** With
   the bin as it stands, the boot store is **in-memory and empty at boot** (§9.5.5(2)'s *empty store* corpus row
   + §9.5.5(6)'s no-persistence clause: `ENGINE-DURABLE-CORPUS-DIRECTION` is DIRECTION ONLY / NOT ACTIVE), so a
   **reachable** provider's build makes **zero** `embed` calls — *n* = 0 embeddable nodes ⇒
   `Ok(Some(VectorIndex::default()))`, an **empty** index, **not** an `Err` — and the build has **no** path to
   fail on a fresh bin: the failure branch is **structurally non-exercisable live**. A live probe of a stub
   answering `/api/tags` **200** and `/api/embed` **500** (as reported by the Implementer; **not** a new live
   claim by this pass) therefore yields `Ready` + `vector:true` over that empty index, and the READY process's
   `mode=vector` request fails only at **query time**, inside the query's own embedding — **503
   `embedding_unavailable`** (**FS-13**, the fail-state table's item 7), **never** `Degraded`. **(b) Its lib-level
   home — the only home it has today.** `P-IM-15` (`strat:boot-index-failure`; contract table (1) row 3 and
   contract table (4)'s pinned failure order) plus the conformance layer's **state 4** (the mid-build-error
   state: `tests/u5_boot_vector_index_conformance.rs`'s `u5_state_4_mid_build_error_installs_no_index_and_degrades`,
   with the boundary *k* = 1 / *k* = *n*, and the call order re-pinned by
   `u5_failure_branch_call_order_is_the_pinned_order`) — a lib-level store **can** carry a corpus, so it **can**
   drive a failing `embed`; the live bin cannot. So the branch is asserted **lib-level only** and is **NOT
   live-verified**: the battery's `R-L3` PASS **(v)** is **PARKED — not deleted, not failed** (`docs/specs/p2-gnosis-server-live-pending-battery.md` §3.5 plus that file's U5 amendment **item 5**, added by this pass,
   which is the authority for that criterion's status, while items 3(b)/4(a) there and the row's own fifth-probe
   precondition text stand as the pre-green record). **(c) The un-park trigger.** The park lifts when a boot path
   carries a corpus **before** the bind — a **pre-bind/seed-file or durable-boot corpus** (the **durability
   unit**, `ENGINE-DURABLE-CORPUS-DIRECTION`, **NOT ACTIVE**) or **any** boot path that carries documents before
   the index build — at which point PASS (v) reverts to its live reading (`Degraded` + `vector:false` +
   `embedding:true` + `reranker:false` + core `true` + the fixed `lastError` + `mode=vector` ⇒ **503
   `engine_unavailable`**, FS-8). **(d) What does not move.** This extension adds no row, deletes none, changes
   no row id / `Strategy-id` / kind / tag, and claims no new authority; the lib-level obligation `P-IM-15` is
   unchanged.
2. **`mode=vector` stops returning `VectorIndexUnavailable` on a *reachable* boot — and may still return it
   on any boot without a reachable provider, or on a caller-built **READY** store with `vectors: None`.**
   §5.9's `Interlock with U5` bullet (`:1643-1645`; re-read this pass — the `:1583-1585` spelling this note
   carried is superseded by REMAND-4 MUST-FIX 1, 2026-09-22: that line is §5.9's `the narrowing this ruling
   pins` clause, not the interlock bullet) is the pre-U5 record;
   U5's end state is: the explicit-leg error becomes **reachable-but-rare** (`Absent`/`Unreachable` boots,
   or a caller-built store), and a freshly booted **reachable** server serves `mode=vector` **200**. The
   error map is **unchanged** (§5.9's table, §11's 21 rows).
3. **`Ready` still does not imply every flag is `true` (F13 stands).** `{Ready, vector:true,
   reranker:false}` is the U5-time mask; `{Ready, vector:false, reranker:false}` remains legitimate whenever
   no index was built (e.g. a provider-reachable probe followed by a build failure ⇒ `Degraded`, or a
   non-boot-constructed store). No row asserts "READY ⇒ vector:true".
4. **The `binary` field is deliberately not built, and that is pinned as a capability statement (§5.8/F2
   §9.1's semantics applied honestly):** `vector:true` under U5 means the **full-field** dense leg is wired
   (`mode=vector` serves), while `binaryFirstPass` **degrades to the full field without error**
   (`src/store/mod.rs:5094-5102`). A consumer that wants a fast first pass is asking for a **new** unit;
   nothing here invents one.
5. **The build is post-boot-loss-agnostic, and the pre-existing defect it exposes is NOT U5's.** A
   provider-reachable boot **whose build succeeded** builds the index once; if the provider then becomes
   unreachable, `vector` stays
   `true` while `mode=vector` returns `EmbeddingUnavailable` (FS-13) — consistent with `P-IM-8`'s pin ("the
   flag is the wired **capability**, not live reachability") and with the **OPEN** `docs/defects.md` row
   **P-8** (post-boot provider loss is invisible to `/engine/status`). U5 does **not** fix, hide or widen
   that defect, and no U5 row asserts provider re-evaluation.
6. **The U3 rows are not re-scoped.** `P-IM-7`/`P-IM-8`/`P-IM-9`/`P-SM-5`/`P-SM-6` keep their claims and
   their executed layer (127 cases, HELD); U5's rows quantify over **the built snapshot and its flag
   consequence**, not over a new flag mechanism. `P-IM-14` is the **U5-time instance** of `P-IM-7`'s
   `vector == snapshot().vectors.is_some()` equality, scoped to the boot's own wiring path — a
   **consequence** row, not a restatement (it adds the fixture-coupling half that `P-IM-7` does not carry).

- ***(U6 in-place marker on the two held rows whose vector predicate U6 supersedes — REMAND-2 MUST-FIX 2, 2026-09-22 (docs-only, ADDITIVE; the two §9.5.4 notes above and the two register cells above keep their text as the U5-time record, and this marker is the dated annotation the same-unit table and the reconciliation table promise). The cell texts are NOT rewritten: per §9.5.3.1's no-row-deleted/no-id-reused discipline the superseded predicate is restated in place here, with the U6 reading beside it, and the anchors are clause-name-first (the row ids and the note names) because the numbers shift.)***
> **(1) The `P-IM-14` register cell (§9.5.5, the row whose `strat:` id is `strat:vector-flag-flip` — `docs/specs/p2-gnosis-server.md`, the U5 typed register in §9.5.5; the REMAND-1/REMAND-2 line spellings this file carried for it, e.g. `:2521`, are **superseded by text**).** Its predicate is now read **superseded in place**: the cell's equality `subsystems.vector == store.snapshot().vectors.is_some()` and its sentence *"the rule that always holds is `subsystems.vector == store.snapshot().vectors.is_some()`"* are the **pre-U6** predicate; **after U6 the equality carries the epoch term** — `derived.vector == (store.snapshot().vectors.is_some() && store.snapshot().epoch == store.epoch())`, read through **one** snapshot clone (surface #3) — while the cell's `Reachable` + `Some` scoping, its V-8.2 half, its write-independence half and its `flags.vector == snap.vectors.is_some()` half (which is `boot_wiring`'s **pinned pre-U6** surface, freshness rule 3) all **stand**. **Its V-8.1 assignment is asserted ONLY for the epoch-aligned instance** (the third precondition, §5.8's `U6's amendment to this section's `vector` predicate` clause).
> **(2) The `P-IM-14` §9.5.4 coverage note (the note headed `` `P-IM-14 — strat:vector-flag-flip` `` in §9.5.4 — its own line spelling as given in the U5 register block, `:2622` in the pre-remand layout, is **superseded by text**).** The same supersession applies to its sentences `get_engine_status().subsystems.vector == store.snapshot().vectors.is_some()` — **exactly** and its V-8.1 fixture half: the equality gains the epoch term, and the V-8.1 fixture's `vector` value equals what the `Reachable` path derives **only on an epoch-aligned snapshot**. Its boundary instances (the **empty** index ⇒ `vector:true`; `Absent`/`Unreachable` ⇒ `vector:false` with V-8.2 unchanged; `reranker` `false`; `embedding == (a provider was wired)`) and its adversarial/Excluded halves are **unchanged**.
> **(3) The `P-SM-7` register cell (§9.5.5, `strat:boot-lifecycle-vector` — the `:2523` spelling this file carried is **superseded by text**).** Its **serving half is unchanged** (a fresh index serves), and its FS-14 half **gains the STALE instance beside the unbuilt one**: a `Ready` store with an index-bearing snapshot whose `snapshot.epoch != store.epoch()` answers `Err(StoreError::VectorIndexUnavailable)` (FS-14 ⇒ 503) on the **explicit** sites and **empties the vector leg with `Ok`** on the fusion site (§9.5.6's valid/fail state 3 and its per-site consultation clause). Its `Absent`/`Unreachable` ⇒ FS-8 half, its empty-index half, its order pin and its call-count half are **unchanged**.
> **(4) The `P-SM-7` §9.5.4 coverage note (the note headed `` `P-SM-7 — strat:boot-lifecycle-vector` `` in §9.5.4 — its own line spelling in the pre-remand layout — `:2623` in the pre-remand block layout, as the same-unit table's `:2478` spelling is **superseded by text** in both places — is The same addition applies: the note's FS-14 half now names **two** instances (unbuilt `vectors: None`, and **stale**), and its `Ready`-gated reading, its FS-8 half and its call-count half stand. **REMAND-2's own §9.5.4 additions for U6 live in the same block** (the six U6 notes, headed by `P-IM-16`'s format-deviation note).
>
> **Derivability (what this marker is for):** a TestWriter re-running the U5 layer from the register derives the **U6** predicate at these two rows — `is_some() ∧ epoch-aligned` — and therefore reads the `P-SM-5` probe-4 fixture of §9.5.5 (the one named exception, re-derived in this section's same-unit table) as a fixture that must align or that must assert the **stale** outcome. **No id, kind, tag, `Strategy-id`, cap, count or executed figure moves**: U5 stays **8 rows**, its caps stay `355 ≤ 400`, the U3 rows stay **held under the amended predicate for the corpus their fixtures actually build** (with that one named exception).

**The property register's execution plan (this unit's layer; §9.5.3's established form, applied to U5).**

- **Command.** `cargo test` (the whole suite; the unit's property layer is never run in isolation from the
  conformance layer in the final trio). The U5 layer needs a tokio runtime + a real `Store` + an injected
  deterministic `EmbeddingProvider` (never a live provider); the sandbox-local cargo home precedent applies
  (`CARGO_HOME=$PWD/.cargo-home cargo test`, §9.5.3).
- **Seed pin.** The **same** master seed as the rest of the file's property binary —
  `SEED = 0x9E37_79B9_7F4A_7C15` (`tests/props_gnosis_server.rs:149`), mixed per row through
  `row_seed(tag)` = `splitmix64(SEED ^ tag)` (`:207-208`) — and the **unit-discriminated, injective** tag
  convention (the F1/REMAND-2 ruling): **U5** ⇒ `U5PIM10`, `U5PIM11`, `U5PIM12`, `U5PIM13`, `U5PIM14`,
  `U5PIM15`, `U5PSM7`, `U5PTP5` (8 tags, 8 rows, one stream each; each tag constant is the ASCII of its own
  name as a `u64`, following the landed `U3PIM7 = 0x5533_5049_4D37` form at `:234-284`). **These eight tags
  are disjoint from the landed `PIM1`…`PTP1`, `U2PIM*`/`U2PSM4`/`U2PTP*` and `U3PIM*`/`U3PSM*` sets**
  (`:677`-`:5606`), so no two rows share a stream and per-row held/broken reporting stays meaningful. No
  wall clock, no thread order, no `thread_rng` input may enter a row.
- **Attempt caps (per-unit reading, §9.5.3's pinned scope; AMENDED POST-RED-PHASE 2026-09-22 — see the
  post-red-phase register amendment block below).** **≤ 100 generated cases per row** and **≤ 400
  cases for U5's whole layer**: this unit's split is `P-IM-10` **60** / `P-IM-11` 50 / `P-IM-12` 40 /
  `P-IM-13` 45 / `P-IM-14` 45 / `P-IM-15` 30 / `P-SM-7` 45 / `P-TP-5` 40 = **355 ≤ 400** ✔ (every row
  ≤ 100). *The pre-amendment split was `45/50/40/45/45/30/45/40 = 340`; it is **superseded** (record kept
  here and in the amendment block below) because the landed `P-IM-10` corpus requires **55** generated
  cases — 9 corpus variants × 5 provider shapes + 10 `p == None` cases (`§9.5.5(2)`'s corpus table,
  `P-IM-10`'s observable) — so a cap of 45 was unachievable as written while the cap is also a guard
  failure; the amendment raises the cap rather than cutting the corpus (no corpus variant and no provider
  shape is dropped).* **The layer is U5's alone** and is **not** added to U2's 400 or U3's 127 for cap purposes; the
  binary total becomes `310 + 400 + 127 + 355 = 1192`, which is a sum and **not** a violation (§9.5.3).
  The caps are recorded as constants (`const B_U5_IM10: u32 = 60;` …) with a `u5_layer_budget_discipline`
  test asserting `Σ caps ≤ 400` and per-row `cases ≤ cap` — never `cases == cap` (see the maximum-not-expected
  rule in the next bullet; the landed precedent: `:16-40`, `:280-284`, `:5689`).
- **A cap is a MAXIMUM, not an expected count (pinned with the amendment — so the amended arithmetic and
  the measured layer cannot be read as a contradiction).** Each per-row figure above is an **upper bound**
  on the cases a row may generate; a row that executes **fewer** cases than its cap is compliant, and the
  budget-discipline test asserts `cases ≤ cap` (never `cases == cap`). The **measured** U5 layer is
  **314 executed / 355 caps**: `P-IM-10` **55**/60 · `P-IM-11` 40/50 · `P-IM-12` 40/40 · `P-IM-13`
  42/45 · `P-IM-14` 25/45 · `P-IM-15` 27/30 · `P-SM-7` 45/45 · `P-TP-5` 40/40. **No executed count may
  exceed its row's cap**, and `314 ≤ 355 ≤ 400` ✔. (A TestWriter MAY raise a row's executed count up to its
  cap — `P-IM-10`'s own cap was raised 45 ⇒ 60 for exactly that reason — but **never past it**, and never
  past the unit's 400.)
- **Stop-after-5.** A row aborts on its **5th distinct counterexample** and reports **at most 5** minimal
  counterexamples — never an unbounded dump, never a panic-without-`cex` (`BROKEN: {cexes:?}`).
- **Held/broken reporting.** One line per row: `[<Property-id>][<Strategy-id>] HELD` or
  `[<Property-id>][<Strategy-id>] BROKEN: <≤5 counterexamples>`, plus
  `[<Property-id>] generated cases: N HELD=true|false`. A broken row fails the test (the unit is red) and its
  `Strategy-id` is what the audit reads.
- **Held/broken reporting — the exact required output form (pinned verbatim by the POST-GREEN SPEC AMENDMENT,
  2026-09-22, so the landed rows can be aligned by one reporting pass; this spec pass edits no test).** Each of
  U5's **8** rows MUST emit, at run time, **both** of these lines, with no substitution:
  `[<Property-id>][<Strategy-id>] HELD` — the `Strategy-id` **is** the row's own `strat:…` token verbatim
  (`strat:boot-index-build` for `P-IM-10`, `strat:index-corpus-full-field` for `P-IM-11`, … the eight tokens in
  the register table above), printed when the row is held; the same line with `BROKEN: <≤5 counterexamples>` in
  place of `HELD` when it is broken; **and** `[<Property-id>] generated cases: N HELD=true|false` as the second
  line's count form (the house form §9.5.3 pinned, kept **as well** — not instead). **The `Strategy-id` belongs
  in the HELD verdict line itself and MUST NOT appear only in failure messages**: a HELD run's output set is
  exactly `{"[<Property-id>][<Strategy-id>] HELD", "[<Property-id>] generated cases: N HELD=true"}` per row, plus
  the layer line. **The observed gap this clause closes (reported by the Implementer and re-read by this pass as
  the landed form):** the landed suite currently prints only `[P-IM-10] generated cases: {cases} HELD={}` per row
  (the row verdicts at `tests/props_gnosis_server.rs:6580-6582`, `:6905-6907`, `:7098-7100`, `:7276-7278`,
  `:7533-7535`, `:7757-7759`, `:8111-8113`, `:8437-8439` — each a two-line `println!`; the U5 layer line at
  `:8510-8512`) and carries its
  `Strategy-id` inside the failure message instead (`[P-IM-10][strat:boot-index-build] BROKEN: {cexes:?}`,
  `:6577-6579`) — a **reporting-form gap only** (the verdict and the count are correct; no row claim, cap, tag or
  HELD value changes, and the strategy id is not a new authority). The gap is **TestWriter-owned and is NOT fixed
  by this spec pass**; the required form above is what a later reporting pass aligns to, and it is stated here so
  that the alignment needs no re-derivation.
- **One-pass remand rule.** Unchanged from §9.5.3: a broken row is triaged **exactly once** into
  **host-fix** / **package-defect** (a `docs/defects.md` row + a **negative probe**, never a weakened row) /
  **over-strong-requiring-re-derivation** (the SpecWriter re-derives the row; no row is deleted and no id is
  reused for a different claim).
- **Layer classification.** **Lib-level (all 8 rows):** `P-IM-10`…`P-IM-15`, `P-SM-7`, `P-TP-5` — each
  asserts through the lib seam (#1 of the surface table) against a real `Store` and an injected
  deterministic provider. **Live-only (0 rows of this register, 1 named obligation):** the end-to-end
  consequence on a booted bin with a controlled provider is the battery's new row **`R-L3`**
  (`docs/specs/p2-gnosis-server-live-pending-battery.md` §3.5), named as the home of that obligation — the
  `R-L2` precedent — and (per REMAND-1) the home of the **failed-build boot** too: that row's PASS **(v)**
  runs the `Degraded` + `vector:false` + `embedding:true` + `mode=vector` ⇒ 503 outcome against the bin, so
  the failure branch is asserted live as well as at lib level (`P-IM-15`). **A U5 test MUST NOT reach a live
  provider** (entry question 5's hermeticity rule).
- **Read-only audit.** After the layer is green the adversarial reviewer performs the standard read-only
  PBT audit against this table (per-row over-strength reasoning, generator coverage, prose
  counterexamples, negative-generator requests); reviewers never run generators.

**Register notes for this unit (so no clause above is read wider than it is).**

- **The V-8.1 literal is a conformance obligation, not a row** (§9.5.2's U3 note, applied verbatim). The
  byte-exact literal and the fixture move **in this unit** (the move table above); rows `P-IM-14`/`P-IM-15`
  quantify over **status values**, and the fixture/literal edit is asserted by the conformance suite.
- **U5 is code-bearing, so the zero-row exemption does NOT apply** (`PBT-GATE-MANDATORY` reserves it for
  docs-only units). U5 has invariants that a property layer can falsify — the build's total function/return
  shape, the corpus key set, the flag's derivation equality, the atomic failure outcome, the post-boot
  query reachability — so the register is **8 rows, not 0**; a zero-row claim here would be a gate finding.
- **Row count stated:** **8 ≤ 8 ✔** (IM ×6, SM ×1, TP ×1). The unit's §11 obligation row is **one**
  obligation split across these rows (like U3's), and **no** id from U2/U3 is reused or renumbered.
- **Residual risks U5 accepts and does NOT absorb** (named so the adversarial pass reads them as residuals,
  not as undiscovered defects): (i) **unbudgeted boot latency** — the build runs before the loopback bind,
  so a slow/hanging provider delays the bind and no timeout is pinned (the provider's own request timeout is
  the only bound); (ii) **no rebuild after boot** — the index reflects the corpus **at boot**, so documents
  created later are not in it until a rebuild vehicle exists (the epoch-driven rebuild is **not** in U5's
  scope; today the boot snapshot is the only writer); (iii) **no `binary` field** (adjudication note 4);
  (iv) **post-boot provider loss** stays invisible to `/engine/status` (the **OPEN** `docs/defects.md` row
  **P-8**, adjudication note 5). Items (i)–(iii) are **not** filed as defects by this pass (they are
  accepted residuals of an authorized unit, and the tracker rows are the supervisor's). **(v) NEW — the
  first `Degraded` + `embedding:true` status blames a subsystem it reports as wired (REMAND-2 NOTE 11,
  2026-09-22; recorded here as a residual, NOT filed as a tracker row by this pass).** U5's failed-`Reachable`-build
  branch is the **first** reachable state that carries `state:"Degraded"` **and** `subsystems.embedding == true`
  (`§9.5.5`'s failure-table row 3 / `P-IM-15`), while the store's `last_error` for `Degraded` is the **fixed**
  string `"a non-core subsystem (embedding/reranker) is unavailable"` (`src/store/mod.rs:4226-4230`). The
  status surface therefore names `embedding` as the unavailable subsystem in a state that simultaneously
  reports `embedding:true` — a self-contradicting pair a consumer (the Astrographer status pane) can read. The
  sentence is **not reworded** (`P-IM-15`/`P-SM-7` forbid rewording it, §9.5.5's "no invention" clause, and
  the string is byte-pinned by V-8.2's golden literal at `tests/wire_conformance.rs:1109`), so the honest
  disposition is a **residual**: no U5 row asserts a `last_error` value other than the store's own, and this
  pass files **no** `docs/defects.md` row (trackers are the supervisor's). **The supervisor may prefer to file
  it as a defects row** (owner: the transport/status unit that owns §5.5's rendering discipline / the status
  surface — the same owner `P-8` carries).

**POST-RED-PHASE REGISTER AMENDMENT (2026-09-22 — the U5 red stage has run; docs-only, ADDITIVE, no row id /
`Strategy-id` / kind / tag / claim changed).** The spec gate returned **EMPTY (round 5)** and the red set has
landed (`tests/u5_boot_vector_index_conformance.rs` — 15 tests — plus 9 new cases in
`tests/props_gnosis_server.rs`: the eight register rows' `#[test]`s and a budget-discipline test). This
amendment reconciles the register with what that corpus actually requires and pins one rule the rows left
implicit. **It adds no row, deletes no row, renumbers nothing, changes no strategy id and claims no new
authority.** The four items:
**(A) the caps are amended and reconciled** (the execution plan above): per-row caps
`60/50/40/45/45/30/45/40 = 355 ≤ 400`, each row ≤ 100, **a cap being a maximum rather than an expected
count**, with the measured layer **314 executed / 355 caps**; the pre-amendment `340` and the pre-amendment
binary total `1177` are **superseded records**, kept in place and annotated wherever this file restated them
(this section's execution plan and numeric-claims block, §9.5.4's numeric-claims block, §U1's binary-total
bullet, §11's U5-update note and API note, §12's REMAND-1/2/4 summaries, §9.5.5's own REMAND-2 correction
block). The rule set is otherwise **unchanged and intact**: ≤ 100 per row, ≤ 400 per unit, the pinned seed +
`row_seed(tag)` tags, stop-after-5, held/broken reporting with the `Strategy-id`, and the one-pass remand
rule.
**(B) the vector-comparison semantics the rows use are now contractual** (contract table (2)'s new
*vector comparison / `NaN`* row below): *same length* + *every non-`NaN` element identical bit-for-bit* +
*`NaN` asserted only in the `NaN` position*, with **no tolerance-based comparison** — the reading `P-IM-12`'s
element-wise equality and `P-TP-5`'s "verbatim" now carry by contract rather than by a test-side choice
(the rows' observables are extended to it by the dated annotation in the REMAND-2 correction block below the
register). This is required because `P-TP-5`'s adversarial corpus **deliberately contains `NaN`/`±∞`**, so a
bare `==` on `Vec<f32>` could never hold of it.
**(C) the move table's row count is corrected**: it is **11 lines** — a header line, a separator line and
**9** data-row lines — not "12 rows" (a **count slip only**; an independent audit re-read every cell and
found every artifact, anchor and prose text intact, and nothing is missing). The dated "12 rows" annotation
is **superseded in place** in REMAND ROUND 4's disclosure bullet; this amendment adds **one** further
data-row line for **(D)**, so the table now stands at **12 lines = the header + the separator + 10 data
rows**.
**Line-number disclosure (read this before following any `p2` anchor into §10–§13).** This amendment
**inserts lines** inside §9.5.5 (the amended caps bullet, the cap-is-a-maximum bullet, contract table (2)'s
new comparison row, item 5 of the REMAND-2 correction block, the two §9.5.4 coverage annotations and this
block itself), so every `p2`-internal line citation **below** the insertion points shifts. The file's
standing rule applies — **the clause name is the anchor and the number is the convenience** — and the
shifts are recorded here rather than by renumbering citations this pass could not re-measure: the in-file
anchors §5.3–§5.9 (incl. §5.9's outcome table, its `Precedence (pinned)` bullet, its `Interlock with U5`
bullet and its `HyDEGenerationFailed`-unreachable clause), §9.5.4's row notes, §11's register table, §10's
fail-state table and §12/§13's bullets all remain valid **by name**; this pass's citations into **other
files** (`tests/wire_conformance.rs`, `src/store/mod.rs`, `docs/specs/engine-wire-contract.md`, the live
battery) are line-valid — those files are **not edited by this pass**, so their numbers do not move — while
§9.5's own bare `:NNNN` citations into `src/store/mod.rs` keep their meaning but read as stale `p2`-suffix
numbers and should be re-derived from the clause name. A later pass that needs a
`p2`-internal number follows the clause name and re-reads it. **No clause was deleted, reordered or
retargeted by the insertions.**
**(D) the `src/store/mod.rs` comment at the derivation site is recorded as an IMPLEMENTER obligation in the
same unit** (it is **not** in the move table's test-artifact set, so it is pinned here so it cannot be lost):
the comment at `src/store/mod.rs:4211-4212` — *"(the boot leaves `DerivedIndexes::default()` ⇒ `false` until
U5's boot index build)"* — becomes **stale the moment U5's code lands**, because the boot then builds an
index for a `Reachable` provider and the flag is `true`; the Implementer reconciles that comment **in the
same unit** (worded to the post-U5 predicate, e.g. the `Reachable` boot installs an index-bearing snapshot
and the flag is the derived `snapshot().vectors.is_some()`), with the same-unit rule that applies to §5.8's
`vector` row and `tests/wire_conformance.rs`'s fixture/comment pair. **The spec pass of this amendment does
NOT edit `src/`** (no implementation is authored here and the comment is untouched by this pass); the
obligation is recorded, not discharged.

**§9.5.5 verification status.** **AUTHORIZED 2026-09-22; its red stage has run; code OWED.** This section is
the spec gate's artifact: the contract + the typed register + the execution plan + the coverage notes + the
move table, as amended by the post-red-phase register amendment above (caps, comparison semantics, the
move-table count and the Implementer's comment obligation — **no row added, deleted or renumbered**).
**No row of this register is HELD or BROKEN yet** (the unit's code does not exist), and nothing here is a
green claim. The **same-unit** edits this section names (the `tests/wire_conformance.rs` literals/fixture/probes,
the F2 notes, the live battery's `R-L2`/`R-L3`) are **delegable together with the code**.
***POST-GREEN SPEC AMENDMENT — VERIFICATION STATUS (2026-09-22; docs-only, ADDITIVE; the paragraph above is the
pre-green record).*** **U5's code is reported LANDED-GREEN:** the Implementer's report for this unit is
**`cargo test` 628 passed / 0 failed**, with the boot build (`src/store/mod.rs`'s `build_boot_vector_index`), the
`src/lib.rs` re-export and the bin's boot wiring (including the failed-`Reachable`-build branch ⇒ `Degraded` +
`vector:false` + `embedding:true`, the `P-IM-15` outcome) landed, and the U5 rows HELD (the executed layer
**314 / 355 caps**, per the numeric-claims block above). **This spec pass is docs-only and did NOT verify the
implementation** — it read the two spec files it owns (plus the landed suite's own reporting lines and budget
constants, for the census and reporting-form checks) and nothing else; the code-landed claim is **the
Implementer's report, carried as reported, not as a new authority of this pass**. **What this pass decided, in
three items:** (1) the numeric census was re-read and **already agreed** with the landed layer (caps `355`,
executed `314`, the per-row split, `Σ caps ≤ 400`, the binary total `1192`; the `340`/`1177` pair is annotated as
a dated superseded record everywhere this file restates it, and **two "unchanged" assertions** that still called
the `340 ≤ 400` arithmetic / `1177` total unchanged are annotated in the §12 REMAND-2 and REMAND ROUND 4 bullets);
(2) the **failed-`Reachable`-build branch's live criterion is PARKED** — see the extension of adjudication note 1
above and the battery's U5 amendment item 5, which carry the identical park statement, the lib-level home
(`P-IM-15` + the conformance layer's **state 4**) and the un-park trigger (a pre-bind/seed-file or durable-boot
corpus, i.e. the **NOT ACTIVE** durability direction); (3) the **exact held/broken output form** is pinned in the
execution plan above so the landed rows' reporting-form gap can be aligned by one later TestWriter pass. **No row
id / kind / tag / `Strategy-id` changed, no row added or deleted, no cap or HELD value changed, and no new
authority is claimed.***

### 9.5.6 U6 — the vector-index freshness / honesty unit (6 rows ≤ 8; **AUTHORIZED 2026-09-22**, code **OWED**)

**Authority.** The user's go-ahead of **2026-09-22** — *"open a follow-up honesty/freshness unit now"* (the
follow-up the U5 landing records pointed at: `docs/next-steps.md`'s U5 DONE row and the §DONE table's
follow-up clause — *"a follow-up **'U5 honesty/freshness'** unit is being opened for `U5-ADV-1`"*; the same
pointer in `docs/pending.md` and `docs/HANDOFF.md`); the three OPEN `docs/defects.md` §OPEN rows this unit's
charter is exactly: **`U5-ADV-1`** (`src/store/mod.rs:4216`'s read-time `vector == snapshot().vectors.is_some()`
+ the boot's composition at `src/bin/gnosis_server.rs:414-419` + the fact that **no call site in `src/` ever
rebuilds** the index; live-CONFIRMED at gate 6), **`U5-ADV-4`** (`built.ok().flatten()` at
`src/bin/gnosis_server.rs:416-419`, the `Err` reduced to the boolean `build_ok` at `:415`) and **`U5-ADV-2`**
(`reqwest::Client::new()` with no timeout at `src/bin/gnosis_server.rs:327`, the build before the bind); the
**ACTIVE** decision `SUBSYSTEM-FLAG-CAPABILITY-SEMANTICS` (`docs/decisions.md`) as read through **§5.8**
(§5.8:1552-1553: *"a flag means this subsystem's full query-time capability is wired and functional for the
current store"*); decision `PBT-GATE-MANDATORY` (this section's typed register is the unit's artifact #1);
§9.5.5's **residual (ii)** (*"no rebuild after boot"* — the clause U6 reconciles, §9.5.6's reconciliation
table) and its residual (i) (*"unbudgeted boot latency"* — the clause U6 bounds); §9.5's header (the
single-per-file id sequence) and §9.5.3's execution-plan form.

**Scope of the unit.** Make the honest signal **complete**: `subsystems.vector` must be `true` only while the
index the store holds actually covers the store's current state, so that a corpus that moved after the boot
build yields a **loud** error (FS-14 `vector_index_unavailable` ⇒ **503**) instead of a silent 200 with
missing hits, and the vector leg must **agree** with the flag it is derived beside. Plus the two small
fold-ins: a stderr diagnostic at the boot when the build fails (**`U5-ADV-4`**) and a **real** bound on the
provider HTTP client (**`U5-ADV-2`**). **Nothing else moves:** no route, no new endpoint, no new `StoreError`
variant, no new wire code, no new §11 row, no new `HealthReport`/`EngineSubsystems` field, no new store
accessor, no new env var, no new CLI flag, no rebuild vehicle, and no persistence.
***(REMAND-1, 2026-09-22 — "no new store accessor" is exact: U6 adds **two lib-visible items** and neither is
a store accessor: (i) the freshness predicate (surface #1) and (ii) the provider-client construction seam
`PROVIDER_REQUEST_TIMEOUT` / `provider_client` (surface #6, needed because `P-TP-6`'s constant was otherwise
unnamable inside the `[[bin]]`). Both are F11-precedent lib seams for `[[bin]]`-only paths (`boot_wiring`
precedent, `src/lib.rs:79-110`); see §13's U6 bullet for the same declaration.***)***

**Explicitly OUT OF SCOPE, with the reason (pinned so the reviewer reads them as declared, not missed).**

| row | why U6 does not carry it |
| --- | --- |
| **`U5-ADV-3`** — the failed-`Reachable`-build status contradicts itself (`Degraded` + `embedding:true` + a `last_error` blaming `embedding`, `src/store/mod.rs:4229-4233`, literal at `:4230`) | a **spec/consumer decision** whose remedy (a distinct `last_error` for the build-failure branch, or the accepted generic sentence) **moves a §11/V-8.2 golden literal** (`tests/wire_conformance.rs:1109` asserts the byte-exact string) and therefore needs its own spec'd unit. **U6 does not reword the string, does not add a `last_error` variant, and no U6 row asserts a `last_error` value other than the store's own fixed one.** The row stays **OPEN** in `docs/defects.md` (supervisor-owned tracker) |
| **`U5-ADV-5`** — the `shard.read().unwrap()` lock-poison panic class now sitting on a pre-bind path (U5's site `src/store/mod.rs:5386`) | a **store-wide** decision (`unwrap_or_else(|e| e.into_inner())`, or an existing error variant across the whole class) that belongs in **its own unit** ("do **not** special-case U5" — the defect row's own disposition). U6 adds **no** lock site and changes **no** existing one |
| **`P-9`** — the non-finite-score panic (`src/wire/codecs.rs:94-99`'s `expect("RagResult is Serialize")`, reached via `encode_result_checked` at `src/wire/query.rs:369-371`) | **package/foundation — the F2 wire foundation's or the transport unit's** (the defect row's disposition); its trigger fired *via* U5's verbatim-vector storage, and **U5's verbatim-vector clause MUST NOT be patched** (`P-TP-5` pins it, and §9.5.5(2)'s *vector comparison / `NaN`* row builds on it). U6 touches **no** encoder, **no** `cosine`, **no** non-finite score path, and adds no negative probe for it — the panic stays the recorded state (`docs/defects.md`'s own note that the TestWriter's `T6` probe is EXPECTED to panic today). **`docs/decisions.md`, `docs/defects.md`, `docs/pending.md`, `docs/next-steps.md` and `docs/HANDOFF.md` are NOT edited by this pass** (trackers are the supervisor's): U6's out-of-scope list and its open questions are recorded **here** for the supervisor to land as tracker dispositions |

**The honesty predicate (pinned — the whole unit is this one expression plus its two consumers).**

> **`vector_index_is_fresh(snapshot, store) ≜ snapshot.vectors.is_some() ∧ snapshot.epoch == store.epoch()`**

and therefore, at read time:

> **`subsystems.vector == snapshot.vectors.is_some() ∧ snapshot.epoch == store.epoch()`**
> (`src/store/mod.rs:4216`, the read-time derivation inside `get_engine_status` — **the only producer of the
> flags**, §9.5.2's F16 pin; **the freshness term is ADDED beside the existing `is_some()` term, and no stored
> mask is introduced**).

**Where the predicate lives, and why it is a DERIVED value (pinned).** The predicate has **two consumers that
must not disagree** — the status derivation (`src/store/mod.rs:4204-4224`, `vector:` at `:4216`) and **all
three** vector read sites (`vector_search`, `src/store/mod.rs:4297-4300`; `vector_query`, `:4451-4454`; the
`hybrid` vector leg's read at `:4533`) — so it is pinned as **one lib-visible predicate** that both call,
never as three hand-rolled `Vec`-checks or two independent copies of the comparison. The **name is the
unit's**; the **behaviour, the term and the sharing are pinned**:

| # | surface (name is U6's) | signature | contract |
| --- | --- | --- | --- |
| 1 | **the freshness predicate, lib-visible** (the `boot_wiring`/`build_boot_vector_index` F11 precedent: `src/lib.rs:69-110`, `:120-126` — a lib seam so a `[[bin]]`-only path stays assertable) | `fn vector_index_is_fresh(snapshot: &DerivedIndexes, store_epoch: u64) -> bool`, or equivalently a pure predicate over the two inputs; the **implementer's choice of spelling is free** (a free fn, or an inherent method reading `self.snapshot()`/`self.epoch()` in-module) | **total and pure**: `true` **iff** `snapshot.vectors.is_some() ∧ snapshot.epoch == store_epoch`; no network, no lock beyond the caller's own read, no mutation. A `vectors: None` snapshot is **never** fresh (the `is_some()` term short-circuits), so the epoch term is only reachable for index-bearing snapshots |
| 2 | **the consumers (pinned to call it — this is the anti-divergence clause; REMAND-1, 2026-09-22: this cell was rewritten PER SITE — it previously quantified the stale outcome over all three read sites, which contradicted the `hybrid` state-6 cell of the valid/fail table)** | `get_engine_status`'s `vector:` value; and each of the three read sites above | the status derivation and the leg must consult the **same** predicate: the flag is `true` for a snapshot **iff** the **explicit** vector leg serves from it. **The refuse is per-site, and every site's outcome is pinned in the clause immediately below this table** (the per-site consultation clause): the two **explicit** sites (`vector_search`, `vector_query`) take `Err(StoreError::VectorIndexUnavailable)` (the **existing** FS-14 variant — no new variant, no new code, §11 unchanged) on **exactly** the inputs where the predicate is `false`, staleness included; the **fusion** site (the `hybrid` vector leg) consults the **same** predicate with its **refuse SUPPRESSED** — a stale index, like a `vectors: None` snapshot, yields an **empty leg and `Ok`, never an `Err`** (the pinned fusion resilience, §5.9's `fusion` row, and this table's state 6). **No site may substitute a second, hand-rolled `is_some()`-only check** (that is the divergence class this table exists to prevent), and **no** site may serve hits from a snapshot the predicate rejects: the pin is **one predicate, four consultation sites, two pinned outcomes** |
| 3 | **the one read of the snapshot per site (pinned to avoid a lock-free race)** | — | each site reads `self.snapshot()` **once** (one `Arc` clone) and compares **that** clone's `epoch`; a site that re-read the snapshot for the epoch could compare an epoch that belongs to a *different* snapshot than the vectors it serves from. The pin is the **observable**: the pair (index served from, epoch compared) is a single clone |
| 4 | **the diagnostics (U5-ADV-4)** | at the boot composition, **before** the snapshot is composed | `src/bin/gnosis_server.rs:414-419`: the build's `Err` must reach **stderr** as **one** operator-read line beginning with the literal marker **`gnosis-server: boot vector-index build failed: `**, followed by the error's own display (the existing `StoreError::EmbeddingUnavailable` rendering). **Order pinned:** the line is written **before** `:416-419`'s composition (`let snapshot = DerivedIndexes { vectors: built.ok().flatten(), ..boot_snapshot };`), so an operator sees the reason even when the composition and the wiring below it are unchanged. **Nothing else may change:** no status string, status code, `last_error`, flag value, key set or wire code moves — the pinned status surface stays **byte-identical**. The diagnostic fires **only** on a build `Err` (never on `Ok(None)`, never on `Ok(Some(_))`). *No logging crate: the bin's own precedent is `eprintln!` (`src/bin/gnosis_eval.rs:322`, `:329`, `:337`, `:344`, `:353`) and there is **no** logging use in `gnosis_server.rs` today* |
| 5 | **the provider-transport bound (U5-ADV-2)** | `src/bin/gnosis_server.rs:311-330` (`OllamaProvider`, `client: reqwest::Client` at `:314`, built at `:327`) | the `reqwest::Client` is built **with a configured request timeout**, `reqwest::Client::builder().timeout(...).build()` — **pinned default `30 s`**, named as a `const` (e.g. `PROVIDER_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);`). **Not configurable in U6** (see the open question below): **no** env var and **no** CLI flag is added, and the set of configuration reads the bin performs stays exactly **three** (`--port` at `:47`, `GNOSIS_SERVER_OLLAMA_URL` at `:321`, `GNOSIS_SERVER_OLLAMA_MODEL` at `:322` — U5's question-5 clause, unchanged). The bound covers **both** provider calls on that client — the boot probe (`is_available` ⇒ `GET /api/tags`, `:364-375`) **and** every build/query-time `embed` (`POST /api/embed`, `:333-363`) — because there is **one** client. **How a timeout renders (pinned to the EXISTING §11 outcome):** `reqwest`'s timeout surfaces as a transport `Err`, already mapped to `StoreError::EmbeddingUnavailable` (`:347`) ⇒ **FS-13 `embedding_unavailable` ⇒ 503**; **no new mapping, no new variant, no new row.** Consequence by phase: **probe timeout** ⇒ `BootProvider::Unreachable` ⇒ `Degraded` + `vector:false` + `embedding:false` (the boot attempts no build), the pin being §9.5.2's `P-IM-9`/`P-IM-8` unchanged; **build timeout** ⇒ the build returns `Err(EmbeddingUnavailable)` ⇒ the **pinned failure branch** (`Degraded` + `vector:false` + `embedding:true` + the store's fixed `last_error`, the `P-IM-15` shape) ⇒ `mode=vector` on that non-READY store is **FS-8 `engine_unavailable` ⇒ 503**; **query-time timeout** ⇒ `Err(EmbeddingUnavailable)` ⇒ **503 `embedding_unavailable`** (FS-13, the §9.5.5 valid/fail state 7 shape). **`EngineState`/`last_error` consequence: exactly the existing ones — no new state, no reworded string**. ***(REMAND-1 amendment, 2026-09-22): the client is built through surface #6's lib-visible seam — `provider_client(PROVIDER_REQUEST_TIMEOUT)` — because the constant must be lib-namable for `P-TP-6` to have an observable at all (this row's `const` was unnamable inside the `[[bin]]`'s private `OllamaProvider`). Nothing else in this row moves: still one client, still both calls, still the same three configuration reads.*** |
| 6 | **the provider-client construction seam, lib-visible (REMAND-1, 2026-09-22 — `P-TP-6`'s assertion surface; the `boot_wiring`/`build_boot_vector_index` F11 precedent again, `src/lib.rs:79-110`, `:120-126`)** | crate-root nameable in the lib: `pub const PROVIDER_REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);` and `pub fn provider_client(timeout: std::time::Duration) -> reqwest::Client` (≡ `reqwest::Client::builder().timeout(timeout).build()`); the defining module is the implementer's choice, the **crate-root re-export is pinned** (the `src/lib.rs:119-134` `pub use` precedent) | **Why it must exist:** surface #5's bound lives in a **private** `OllamaProvider` inside the `[[bin]]` (`src/bin/gnosis_server.rs:311-330`), so no lib test can name the constant, and observing a real `30 s` timeout costs `30 s` of wall clock — which this unit's hermeticity rule forbids. The seam therefore carries the observable: the bin's **single** construction site (`src/bin/gnosis_server.rs:327`) becomes `client: provider_client(PROVIDER_REQUEST_TIMEOUT)`, so the pinned value **is** the bin's value, and the duration is **falsifiably** wired (a builder call that dropped it hangs against a never-answering endpoint instead of returning a timeout `Err` — surface #6 is what makes that observable cheap and hermetic). **Cost and boundary, stated plainly:** this is U6's **second** lib-visible item (the predicate is the first) and an F11-precedent seam for a `[[bin]]`-only path: it adds **no** store accessor, **no** route, **no** wire code, **no** `StoreError` variant, **no** §11 row, **no** env var, **no** CLI flag, **no** new dependency (`reqwest` is already a lib dependency, `Cargo.toml:17`), **no** new configuration read (the bin's three reads are unchanged) and **no** frozen-type field. **The bin-level alternative is REJECTED, with the reason:** homing `P-TP-6` at the spawned-bin layer (the `P-IM-19` shape) would leave the row's only observable a **real 30 s timeout** — exactly what the row's own excluded list forbids — so the seam is the pinned remedy and `P-TP-6` stays a **lib-level** row |

**The predicate's per-site consultation (pinned — REMAND-1, 2026-09-22: this clause resolves the refuse-vs-serve question that surface #2's pre-remand wording left underdetermined).**

| site | how it consults the shared predicate | stale (`snapshot.epoch != store.epoch()`, index-bearing) | `vectors: None` (unbuilt) |
| --- | --- | --- | --- |
| the status derivation (`src/store/mod.rs:4204-4224`, `vector:` at **`:4216`** — the only flags producer, §9.5.2's F16 pin) | `vector = vector_index_is_fresh(&snap, self.epoch())` on the **one** clone read | flag **`false`** | flag **`false`** |
| `vector_search` (`src/store/mod.rs:4297-4300`; the **direct** store API the retrieval fixtures call — it carries **no** READY gate, which lives on the `rag_query` path at `:4078-4080`) | refuse **ON**: the predicate replaces the landed `is_some()` match arm | **`Err(StoreError::VectorIndexUnavailable)`** (FS-14 ⇒ 503) — **pinned in this clause** (the pre-remand text left this site's stale outcome unnamed) | `Err(VectorIndexUnavailable)` (FS-14, unchanged) |
| `vector_query` (the `mode=vector` leg, `:4451-4454`) | refuse **ON**, and **before** the provider check | `Err(VectorIndexUnavailable)` (FS-14); the provider check is never reached, so the injected provider's `embed` call count stays `0` | `Err(VectorIndexUnavailable)` (FS-14, unchanged) |
| the `hybrid` vector leg (`:4533`, inside `hybrid_query`; the leg at `:4529-4554`) | the **same** predicate with the refuse **SUPPRESSED** — `if vector_index_is_fresh(...) { run the leg } else { leg stays empty }` | leg **empty**, query **`Ok`** (200) — no stale hit, no `Err` | leg **empty**, query **`Ok`** (200, unchanged) |

- **What this clause forbids, and what it does not change.** (i) A second `is_some()`-only check at any of these four sites is a **violation** (surface #2's anti-divergence pin); (ii) the `hybrid` site may **not** be read as "the fusion path is exempt from the predicate" — it consults the **same** predicate, only the **refuse** is suppressed, so a stale index cannot serve hits through fusion either (that is the state-6 cell's "the flag's `false` and the leg's emptiness **agree**"); (iii) the two explicit sites' **stale** and **unbuilt** outcomes collapse onto the **same existing** variant, so the pre-U6 FS-14 reachability set is unchanged and §11 stays **21 rows**; (iv) the `rag_query`/`rag_stream` READY gate (`:4078-4080`) precedes **every** row of this table, so a non-READY store answers FS-8 `engine_unavailable` at **any** of the four sites reached through those entry points (valid/fail state 7); (v) the SSE path (`rag_stream`) carries the same per-site outcomes — its pre-stream `StoreError` is rendered as the `error` frame under the §11-mapped status (§10's `GET /rag/stream` row), and the **live** home of the whole-clause obligation is the battery's new row **`R-L4`** (a `POST /rag/query` criterion; §9.5.6's same-unit table).

**Why this shape, and the deliberate choice between the three forks (pinned, with the cost stated).**

- **Chosen: (a) the flag flips — the loud FS-14 is restored — PLUS the leg refuses on the same predicate.**
  The reason is that the defect being fixed is *a loud error replaced by a silent wrong answer*
  (`docs/defects.md` `U5-ADV-1`). Flipping the flag alone leaves the **leg** serving stale hits from a
  snapshot the status simultaneously declares unusable — a **new** divergence between the status surface and
  the read it describes, i.e. exactly the honesty class `SUBSYSTEM-FLAG-CAPABILITY-SEMANTICS` exists to
  prevent. Refusing on the **same** predicate is what makes the flag and the leg one honest statement.
  ***(REMAND-1, 2026-09-22: "the leg" here means the **explicit** leg — `vector_search` and `vector_query`;
  the **fusion** (`hybrid`) leg consults the same predicate with the refuse **suppressed** (empty leg, `Ok`),
  which the per-site consultation clause under the surface table pins. The two are one statement because a
  stale index can neither be served explicitly nor contribute hits through fusion.***)***
- **Rejected: (b) the read re-derives / rebuilds the index.** A **status read must stay pure** — `P-SM-5`
  pins it as a deterministic side-effect-free projection (and `P-SM-8` below re-pins it under the amended
  predicate) — so re-deriving inside `get_engine_status` would break a held row; and a rebuild-on-read would
  additionally need a provider (absent on a degraded boot), would pay `n` `embed` calls **per request**, and
  would turn the flag's value into a function of a **write** the read performed. It is the `WRITER-ACTOR`
  rebuild vehicle the repo deliberately holds off (`docs/decisions.md` `WRITER-ACTOR-JOURNAL`; the parked
  GR-3 maintenance half in `docs/pending.md`) — **not** authorized here.
- **Rejected: (c) pin the residual for the consumer.** That is what §9.5.5 residual (ii) already did, and the
  adversarial pass escalated exactly what that acceptance costs: a consumer reading `vector:true` cannot
  distinguish "index covers the corpus" from "index covers a corpus that used to exist", and the wire answer
  is a 200 that silently omits the missing nodes. Keeping an *accepted* silent wrong answer in a unit whose
  charter is honesty would contradict the unit's reason to exist.
- **The cost of the chosen shape, stated plainly (this is the price of honesty).** A booted server **stops
  serving `mode=vector` after any mutation** to its store. U6 adds **no** rebuild vehicle (see the
  interaction clauses), so there is **no in-process recovery**: the index stays stale until a process
  restart with a reachable provider, and `mode=vector`/`hybrid`'s vector leg are unavailable in between.
  **The store's read/write paths are otherwise untouched** (flat/graph/lexical modes are unaffected: no
  freshness term is added to any other leg). **Second-order cost, in the other direction:** the predicate's
  epoch term is **conservative** — `epoch()` advances by **one per committed journal entry**
  (`Store::append_journal`, `src/store/mod.rs:1828-1839`), so mutations that do **not** change the corpus
  (`create_wiki` at `:2822`, `add_triple` at `:3036`, `create_fact` at `:3614`, the community/fact mutators at
  `:3227`, `:3269`, `:3434`, `:3528`, `:3757`, `:3840`, `:4008`, `:4030`) also mark the index stale. That
  false-positive direction is **accepted deliberately**: an over-wide staleness term yields the honest *loud*
  error, never a wrong answer, and the finer-grained alternative (a corpus-scoped revision) is a **new
  store-epoch semantics U6 is not authorized to introduce** (see the open questions) — the honest
  disposition is to state the over-approximation rather than to hide it.
  **The unstated premise this clause rests on is now PINNED as an explicit assumption (REMAND-2 NOTE 11,
  2026-09-22).** *The claim "the honest loud error, never a wrong answer" is a **conditional**, and its
  antecedent is:* **every corpus-changing mutation advances `Store::epoch()`** — i.e. no path can change the
  corpus the vector index is derived from while leaving `epoch()` unchanged. **Verified this pass as a
  standing *assumption of this predicate*, not as a new pin:** `epoch()` advances by **one per committed
  journal entry** (`Store::append_journal`, `src/store/mod.rs:1828-1839`), and the **six** corpus-writing
  sites — every `guard.docs.insert(…)` / `guard.docs.remove(…)` in the store — are each **paired** with an
  `append_journal` call (`:2280` with `:2296`; `:2354` with `:2368`; `:2421` with `:2422`
  (`create_document`); `:2539` (`update_document`, whose own propagation appends may fire more than once);
  `:2591` with `:2594` (`delete_document`); `:3035` with `:3036` (`add_triple`); `:3359` with `:3370`
  (`set_reference_state`)), so the antecedent holds in the landed tree. **What follows if it ever stops
  holding:** an **epoch-preserving corpus write** (a mutation that changes documents/nodes without an
  `append_journal`) would make `snapshot.epoch == store.epoch()` **true on a genuinely stale index** — the
  flag would read `vector:true` and the leg would serve **stale hits silently**, which is exactly the
  `U5-ADV-1` silent-wrong-answer class this unit exists to remove and **is a defect of THIS predicate**, not
  a fixture or test problem. Such a change therefore **owes its own unit**: a new `docs/defects.md` row (the
  supervisor's tracker) plus a **negative probe** asserting the pairing for the new site — *never* a weakened
  `P-IM-16`/`P-IM-17`. **No U6 row asserts the pairing** (no row may read `append_journal`'s call sites as
  its observable); the assumption is stated here so the predicate's honesty claim is read as **conditional on
  it**, and so the reviewer can see the premise rather than infer it.

**The freshness condition, defined over the store/snapshot states it compares.** The two states compared are
`snapshot().epoch` (the epoch of the **derived snapshot** the store currently holds —
`DerivedIndexes.epoch`, `src/store/mod.rs:827-832`, whose documented role is a **label on the snapshot the
caller composed**: `:823-826`'s *"a writer rebuilds and swaps on the epoch feed"*) and **`Store::epoch()`**
(`:1814-1818`: *"the current journal epoch — the latest committed mutation's `seq` (0 when the store is
fresh)"*). **`swap_snapshot` (`:1855-1860`) does NOT stamp the snapshot** — it stores the value it is given —
so the label is the **caller's** to set, and U6 pins the two rules that make the comparison meaningful:

1. **`build_boot_vector_index` does NOT set, move or read the epoch** — it returns the index **alone** and the
   snapshot is composed by the caller (§9.5.5's surface #1/#4 and its contract table (2)'s *the `epoch`
   field* row). That clause is **UNCHANGED** by U6 (it is pinned by `P-IM-12`'s store-side-effect freedom and
   by the blind layer's *"`DerivedIndexes.epoch` stays at the INPUT's value"* assertion).
2. **The boot's composition sets the composed snapshot's `epoch` to `store.epoch()`** — the epoch of the
   **unchanged** store the build just read. This is a **new pinned term** in the boot's snapshot composition
   (`src/bin/gnosis_server.rs:416-419`), i.e. a **fifth** field of that composition beside
   `vectors: built.ok().flatten()` and the `..boot_snapshot` base. **Both branches** (a successful build and
   the failed-build branch) compose with that epoch, because both install a snapshot the boot itself built or
   deliberately left index-free. Reason, stated as the invariant rather than as a convention: the build reads
   the store and writes nothing to it, so `store.epoch()` is the same before and after the build, and the
   composed snapshot genuinely **is** the snapshot of the store at that epoch; without the term the boot's own
   index would read as **stale-by-construction** (`DerivedIndexes::default()` carries `epoch: 0`, `:828-832`)
   on any store whose epoch is non-zero.
3. **`boot_wiring`'s returned flags keep the PRE-U6 predicate — deliberately, by pin (REMAND-1, 2026-09-22).**
   `boot_wiring(provider, snapshot)` takes **no** store epoch (`src/lib.rs:90-93`) and its `vector` element is
   `snapshot.vectors.is_some()` (`:105`; doc comment `:88`, comment block `:59-67`), so it **cannot** evaluate
   the freshness term. It is therefore **not** the derivation and is **pinned unchanged** (it stays the boot's
   *wired-capability assertion surface*, §9.5.2's `P-IM-9`). **Consequence, pinned as a scope rule:** the
   equality `derived == flags` (U5's `P-IM-14` derivation-equality probe) is asserted **only for
   epoch-aligned snapshots** — on an epoch-mismatched snapshot the pinned, intended relation is
   `derived.vector == false` **while** `flags.vector == true`, and that two-sided difference is the assertion
   (never a defect, never something a fixture edit can remove). **No row of any layer may assert
   `derived == flags` — or `flags.vector == true` — for a snapshot whose `epoch` differs from the store's.**

| what is compared | "stale" means | the pinned outcome |
| --- | --- | --- |
| `snapshot.epoch == store.epoch()`, snapshot index-bearing | **not** stale — the snapshot was composed against the store's current state | `vectors` fresh ⇒ `vector:true`; the leg serves |
| `snapshot.epoch != store.epoch()` (the store moved), snapshot index-bearing | **stale** — at least one journal entry was committed after the snapshot was composed, so the snapshot's index does not certifiably cover the current corpus | `vector:false`; the leg refuses with `VectorIndexUnavailable` (FS-14) — **the two EXPLICIT sites (`vector_search`, `vector_query`); the `hybrid` leg empties instead, per the per-site consultation clause under the surface table (REMAND-1, 2026-09-22)** |
| snapshot `vectors: None` (any epoch) | **not applicable** — nothing is carried | `vector:false`; the leg refuses with `VectorIndexUnavailable` (FS-14, the existing outcome) — **the explicit sites; the `hybrid` leg empties (the pre-existing fusion behaviour, unchanged — REMAND-1, 2026-09-22)** |

- **After a create/update/delete (the `U5-ADV-1` instance).** `create_document` (`src/store/mod.rs:2422`),
  `update_document` (`:2539`), `delete_document` (`:2594`) each `append_journal`, so `Store::epoch()`
  advances by one and the boot-composed snapshot's epoch no longer matches: on a **READY** booted server the
  next `GET /engine/status` reports `subsystems.vector == false`, and the next `mode=vector` request is
  `VectorIndexUnavailable` ⇒ **503 `vector_index_unavailable`** — **never** a 200 with missing hits.
  (`update_document` can advance the epoch **more than once** through its internal `propagate_*` appends,
  `:2296`/`:2368` — irrelevant to the predicate, which is an **equality**, not a step count.)
- **After a provider change.** The freshness predicate does **not** compare providers: wiring a **different**
  provider via `set_embedding_provider` (`:1885`) does **not** move `epoch()` (`:1885-1887` writes only the
  provider slot) and therefore does **not** make the snapshot stale — so an index built with provider A keeps
  `vector:true` while queries embed with provider B. That is **out of U6's scope and NOT absorbed**: it is not
  `U5-ADV-1` (no silent *missing* hits — the hits come from a real index of the current corpus), it is the
  same **capability-vs-liveness** family as the OPEN `P-8` (the flag reports the **wired capability**, §9.5.2's
  `P-IM-8`), and pinning a provider-identity term would move `P-IM-7`'s predicate for a second time in one
  unit. **Recorded as an open question** for the supervisor (§9.5.6's open questions), with no U6 row and no
  U6 obligation.
- **Across a boot.** Each boot composes a **fresh** snapshot at the store's then-current epoch, and the boot
  store is in-memory and empty (`Store::new()`, `src/bin/gnosis_server.rs:381`; there is no persistence —
  §9.5.5(6)'s no-persistence clause, `ENGINE-DURABLE-CORPUS-DIRECTION` **DIRECTION ONLY / NOT ACTIVE**), so a
  freshly booted process's snapshot is fresh **by construction** (rule 2 above) and its `vector` value is
  exactly the post-U5 value `R-L2` re-verified live — **this is why U6 does not change a single boot-time
  flag value**.

**Valid states and documented fail-states (the TestWriter's guide; every cell stays inside the closed 21-row
§11 map — no new `StoreError` variant, no new wire code, no new route, no new accessor).**

| # | state | input (on a store) | `subsystems.vector` | `mode=vector` outcome ⇒ §11 rendering |
| --- | --- | --- | --- | --- |
| 1 | **valid — fresh boot** (unchanged by U6) | `Reachable` boot: index composed **at `store.epoch()`** + provider wired + `Ready` (or an empty index — an empty index is still fresh) | **`true`** | `Ok(RagResult)` with `RagTrace::Vector`; empty `results` for an empty index — **200** |
| 2 | **valid — a caller-built fresh snapshot whose epoch matches** (in-process rebuild shape) | an index-bearing snapshot whose `epoch == store.epoch()`, provider wired, `Ready` | **`true`** | `Ok(RagResult)` — **200** (the U5 integration fixtures' shape once their epoch is aligned) |
| 3 | **fail — the index is STALE (the unit's primary case)** | `Ready`, provider wired, `vectors: Some(_)` but `snapshot.epoch != store.epoch()` — `U5-ADV-1`'s post-boot-write instance | **`false`** | `Err(StoreError::VectorIndexUnavailable)` (**FS-14**) ⇒ **503 `vector_index_unavailable`** — **not** a 200, and **no** `embed` call (the index check precedes the provider check, `src/store/mod.rs:4451-4457`) |
| 4 | **fail — READY, no index at all** (U5's valid/fail state 6, unchanged) | `Ready`, `vectors: None` | **`false`** | `Err(VectorIndexUnavailable)` (FS-14) ⇒ **503 `vector_index_unavailable`** — the epoch term is unreachable here (`is_some()` short-circuits) |
| 5 | **fail — stale index AND no provider** | `Ready`, `vectors: Some(_)`, epoch mismatch, **no** wired provider | **`false`** | **still FS-14** `vector_index_unavailable` ⇒ 503 — the freshness check is **before** the provider check, so the order pin (`:4451-4457`) is unchanged and the answer is not FS-13 |
| 6 | **fail-adjacent — `hybrid` with a stale index** | `Ready`, provider wired, epoch mismatch | **`false`** | hybrid **serves** (graph + lexical legs, the vector leg degrades to empty, `:4529-4554`) ⇒ **200** — the pinned hybrid resilience: the flag's `false` and the leg's emptiness **agree** (no stale hits are served). **REMAND-1, 2026-09-22: this cell is the one the pre-remand surface #2 contradicted — the fusion site consults the SAME predicate with its refuse suppressed (per-site consultation clause); a stale index may not contribute a hit here either, and a second `is_some()`-only check at this site is a violation** |
| 7 | **fail — a non-READY store** (U5 states 4/5, unchanged) | `Absent`/`Unreachable`/failed build — `Unavailable` or `Degraded` | **`false`** | `Err(StoreError::EngineUnavailable)` (**FS-8**) ⇒ **503 `engine_unavailable`** — the pre-READY gate (`:4078-4080`) fires **before** any freshness or leg check, whatever the snapshot holds; **FS-14 is unreachable on a non-READY store** |
| 8 | **fail — fresh index, provider gone at query time** (U5 state 7, unchanged) | `Ready`, index fresh, **no** provider | **`true`** (the wired capability; §9.5.5 adjudication note 5 / the OPEN `P-8`) | `Err(EmbeddingUnavailable)` (**FS-13**) ⇒ **503 `embedding_unavailable`** |
| 9 | **`GET /engine/status` in every one of the above** | any state | the honest value per the predicate | **always 200** (§5.8's single status surface) with the honest vector — **no** fail-state on the status route |

- **`node_snippet` is NOT the observable (pinned).** The silent-wrong-answer mechanism is that
  `node_snippet` (`src/store/mod.rs:4961`) reads an **unindexed** node as `""`, so a partial result is not
  distinguishable from a complete one by its shape. U6 therefore **never** asserts on a snippet's content:
  the fix's observable is the **outcome** (FS-14 ⇒ 503, no `Ok` at all), and the honest boundary
  "a fresh index may return **fewer** hits than the corpus's node count" is stated so a TestWriter does not
  invent a completeness claim no row pins (a fresh index serves the nodes it covers; only the stale case is
  refused).
- **The flag and the leg agree on the same inputs (the single-sourcing obligation, restated as an observable).**
  For every state above: `get_engine_status().subsystems.vector == false` **because of staleness** ⇒ the
  `mode=vector` request is FS-14; `== true` ⇒ the request is `Ok`. A test that finds a state where the flag is
  `false` for staleness **and** the leg serves from that same snapshot has found the defect this unit exists
  to prevent (that is `P-IM-17`'s falsifiable half).

**Numeric / census claims made by this section (each re-read this pass).** the predicate has **two terms** and
**three leg consumers** (`src/store/mod.rs:4216`, `:4297-4300`, `:4451-4454`, `:4533`; the status derivation
site `:4204-4224` is the **only** flags producer, §9.5.2 F16); the freshness comparison is over **two** values
(`:827-832` and `:1814-1818`); the boot's composed-snapshot block is **four** lines today (`:416-419`, with
the build call at `:414` and `build_ok` at `:415`) and gains the epoch term; the provider client is **one**
`reqwest::Client` (`:314`, built `:327`) covering **two** calls (`:364-375`, `:333-363`); the bin's
configuration reads stay **three** (`:47`, `:321`, `:322`); the §11 map stays **21 rows** and the `StoreError`
taxonomy **21 variants**; `EngineSubsystems` keeps **six** `bool`s; the route table stays **14 rows**; U6's
register is **6 rows ≤ 8** with a cap sum **300 ≤ 400** (per-row `60/50/40/45/60/45`, each ≤ 100 — the
execution plan below); the binary's property-layer total becomes `310 + 400 + 127 + 355 + 300 = 1192 + 300 =
1492` (**a SUM, not a cap** — §9.5.3's per-unit scope pin; the `1177`/`1192` spellings elsewhere in this
file are the **pre-/post-U5 records**, both kept in place and none of them amended by this pass).
***(REMAND-1 re-count, 2026-09-22 — every figure below was re-read against the tree in this pass; the pre-remand
sentences above stand as the U6-authoring record.)*** The predicate has **two terms** and **three leg
consumers**, i.e. **four consultation sites** in total (the status derivation + the three leg sites); of those,
**two** carry the refuse (`vector_search`, `vector_query`) and **one** suppresses it (the `hybrid` leg) — the
per-site consultation clause below the surface table. The surface table has **six** rows (five at authoring:
U6's lib-visible items are the predicate **and** the provider seam `PROVIDER_REQUEST_TIMEOUT` /
`provider_client`). `boot_wiring` stays **one** unchanged seam with the **pre-U6** `is_some()` reading
(`src/lib.rs:105`), which is why `derived == flags` is scoped to **epoch-aligned** snapshots. The remaining
figures stand as written: **two** compared values, **four** lines in the composed-snapshot block, **one**
`reqwest::Client` covering **two** calls, **three** configuration reads, **21** §11 rows, **21** `StoreError`
variants, **six** `EngineSubsystems` `bool`s, **14** route rows, **6** register rows ≤ 8, caps
`60/50/40/45/60/45 = 300 ≤ 400`, binary total `310 + 400 + 127 + 355 + 300 = 1492`.

#### The U6 typed register (6 rows — `P-IM-16`…`P-IM-19`, `P-SM-8`, `P-TP-6`; **≤ 8** ✔)

**Id allocation and provenance (the §9.5 header's single-per-file sequence, §9.5.3.1's F1 ruling).** The
highest ids claimed anywhere in this file before this pass are `P-IM-15` (§9.5.5), `P-SM-7` (§9.5.5) and
`P-TP-5` (§9.5.5), so U6 takes the **next free** numbers in each class: **`P-IM-16`, `P-IM-17`, `P-IM-18`,
`P-IM-19`** (4), **`P-SM-8`** (1), **`P-TP-6`** (1). They are **free** — a `docs/specs/*.md` sweep this pass
for `P-IM-16`…`P-IM-19` / `P-SM-8` / `P-TP-6` finds **no** prior claim in this file or in any other register
file (the other files' ids are their own units' namespaces and are not touched — §9.5.3.1's cross-file
look-alike rule; this pass edited **no** other register file). **No id is reused for a different claim, no U2/U3/U5
id is renumbered, no row is deleted, and no `F-`/FS-n row is added** (fail-states appear only inside a
*total-function/error-shape* clause, per §9.5's authoring rule). The per-unit **tag** for each row is `U6` +
its `P<CLASS><N>` form (§9.5.3's injective convention): **`U6PIM16`, `U6PIM17`, `U6PIM18`, `U6PIM19`,
`U6PSM8`, `U6PTP6`** (their **exact `u64` values are pinned** in the execution plan's seed-pin bullet —
REMAND-1, 2026-09-22: the pre-remand form named the tags but not their values, which were not derivable).

**Class tally (U6):** IM ×4 (`P-IM-16`, `P-IM-17`, `P-IM-18`, `P-IM-19`), SM ×1 (`P-SM-8`), TP ×1
(`P-TP-6`) = **6 rows ≤ 8** ✔. The register's per-row generator-coverage notes are in **§9.5.4** (U6's six
notes appended there).

| Property-id | Class | Invariant | Strategy-id | Observable-as-property | Code site / contract clause |
|---|---|---|---|---|---|
| `P-IM-16` | IM | **The vector capability is `true` exactly when the CURRENT snapshot carries an index that is NOT STALE — and the predicate is DERIVED at read time, never stored, never written by a mutator.** For **any** store state: `subsystems.vector == snapshot().vectors.is_some() ∧ snapshot().epoch == epoch()`, element-wise, where the two terms are the **one** pinned predicate. Stale (the store's epoch moved after the snapshot was composed) and unbuilt (`vectors: None`) are **both** `false` and are **not** distinguished on the flag (the consumer's honest signal is the flag; the distinguishing detail is the leg's error family, `P-IM-17`). The freshness term is an **equality**, so a snapshot whose epoch is *ahead* of the store's is likewise not fresh (no "later than" reading). The value is a **projection**: `set_subsystems` (`:1874-1876`, write-only and inert after U3) changes nothing, and no mutator writes a mask | `strat:vector-freshness-capability` | ∀ state `s` (built from `Store::new()` + `swap_snapshot` + `set_embedding_provider` + `set_engine_state` + the public `RagStore` mutation calls, **never** by a `set_subsystems` write): **`let snap = s.snapshot(); let f = s.get_engine_status().await.subsystems.vector;` ⇒ `f == (snap.vectors.is_some() && snap.epoch == s.epoch())` — ONE bound clone** (REMAND-1, 2026-09-22: the pre-remand form called `s.snapshot()` twice, so under a concurrent `swap_snapshot` the vectors and the epoch could come from **different** snapshots and the assertion could fail spuriously; the single-clone form is surface #3's pin and the row's own property); in particular: an index-bearing snapshot composed at `s.epoch()` ⇒ `true` (**an empty `VectorIndex` included**); an index-bearing snapshot with `epoch != s.epoch()` ⇒ `false`; a `vectors: None` snapshot at any epoch ⇒ `false`; a snapshot whose `epoch > s.epoch()` ⇒ `false`; and `s.set_subsystems(<all-true>)` followed by a re-read leaves `f` unchanged (the write is not the producer). No flag is `true` without **both** witness terms | §5.8 (`:1552-1553`, the capability semantics; ACTIVE decision `SUBSYSTEM-FLAG-CAPABILITY-SEMANTICS`); the derivation site `src/store/mod.rs:4204-4224` (`vector:` at **`:4216`** — the read-time `is_some()` term U6 extends); `:1814-1818` (`epoch()`); `:827-832` (`DerivedIndexes.epoch`); `:1849-1860` (`snapshot`/`swap_snapshot`, the only installation vehicle); `:1868-1876` (the inert `set_subsystems` hook); §9.5.5's `P-IM-14` (the `is_some()`-only equality U6 supersedes **in place**, §9.5.6's reconciliation table) |
| `P-IM-17` | IM | **A mutated corpus makes the signal honest AND the read loud: the flag flips, and the leg refuses on the SAME predicate — the silent 200 with missing hits is unreachable.** For **any** store whose current snapshot carries an index that is stale (`snapshot.epoch != epoch()`), a **`mode=vector` `rag_query`** is `Err(StoreError::VectorIndexUnavailable)` (**exactly** on the stale inputs) ⇒ §11's **503 `vector_index_unavailable`** (FS-14) — the **existing** variant and the **existing** row, no new variant/code/route; **the row's quantifier is `rag_query` — REMAND-1 scope note, 2026-09-22 (the pre-remand text quantified `rag_query`/`rag_stream` together and left the fusion and `vector_search` outcomes unstated, which is why the per-site consultation clause now pins all four sites)**: the `mode=vector` `rag_stream`/wire half carries the **same** per-site outcome (its pre-stream `StoreError` renders as the `error` frame under the §11-mapped status, §10's `GET /rag/stream` row) and its **live** home is the battery's row **`R-L4`**; and the status read for the same state reports `vector: false`. **The divergence this row forbids:** a state in which the status says the vector capability is unusable **while** the leg serves hits from that same snapshot (flag `false` + `Ok` = broken). The FS-14-vs-FS-13 order is unchanged (the freshness/index check precedes the provider check) and FS-8 still wins on a non-READY store (the pre-READY gate precedes every leg check) | `strat:stale-index-fail-loud` | ∀ state `s` with `Ready` + provider wired + a stale snapshot (`swap_snapshot(DerivedIndexes { vectors: Some(vi), epoch: e, .. })` where `e != s.epoch()`): `s.get_engine_status().await.subsystems.vector == false` **and** `s.rag_query(q, {mode: Vector, ..})` is `Err(StoreError::VectorIndexUnavailable)` with `server_status(&err) == Some((503, "vector_index_unavailable"))` and **never** `Ok`; the injected provider's `embed` **call count == 0** on that path (the index check precedes the provider, `:4451-4457`); the same state with **no** wired provider is **still** FS-14 (not FS-13); the same state left non-READY (`Absent`/`Unreachable`/failed build) is FS-8 `engine_unavailable`; `mode=hybrid` on the stale state is `Ok` with the vector leg empty (`:4529-4554`) and the flag `false` (**the fusion site consults the SAME predicate with its refuse SUPPRESSED — the per-site consultation clause; a second `is_some()`-only check at that site is a violation, and no stale hit may be served through fusion — REMAND-1, 2026-09-22**); **`vector_search` on the same stale state is `Err(StoreError::VectorIndexUnavailable)`** — the direct-API twin of this row's `rag_query` instance, pinned by the same clause (REMAND-1, 2026-09-22: `vector_search`'s stale outcome was unnamed before this remand); and **the counter-witness**: for the fresh state (state 2 of the valid/fail table) the same request is `Ok(RagResult)`, so the row is not satisfiable by refusing everything | the defect row this row closes: `docs/defects.md` **`U5-ADV-1`** (the silent-200 escalation); `src/store/mod.rs:4216` (the status half), `:4297-4300` (`vector_search`), `:4451-4457` (`vector_query` — index **before** provider), `:4529-4554` (the hybrid leg's degrade), `:4078-4080` (the READY gate); §11's map (`src/server.rs:34` → 503; `src/wire/error.rs:34` → the code) |
| `P-IM-18` | IM | **The boot's composed snapshot is NOT stale by construction: the boot sets the composed snapshot's epoch to the store's epoch, so a freshly booted reachable server's index is genuinely fresh and its flag value is exactly the post-U5 value.** The build itself remains epoch-agnostic (`build_boot_vector_index` never reads, sets or moves the epoch — §9.5.5's *the `epoch` field* row, unchanged); the **caller's composition** carries the epoch term. Both boot branches compose with `store.epoch()` — the successful build (index-bearing) and the failed build (the unchanged index-free snapshot) — because the build writes nothing to the store, so the epoch is identical before and after it | `strat:boot-epoch-alignment` | ∀ boot outcome and ∀ store state `s` (empty and populated): with `let e = s.epoch();` **before** the build call, `let built = build_boot_vector_index(&s, wired.as_ref()).await;` ⇒ `s.epoch() == e` (the build moves nothing — the `P-IM-12` guarantee, reused) and the composed snapshot satisfies `snap.epoch == e`; then after the pinned wiring (`swap_snapshot(snap)` + `set_embedding_provider` in the `Reachable` case + `set_engine_state`), `s.get_engine_status().await.subsystems.vector == snap.vectors.is_some()` — i.e. for a **`Reachable`** boot (index-bearing, empty index included) the flag is **`true`**, and for `Absent`/`Unreachable`/failed-build it is **`false`**; and the **negative probe** this row forbids: a boot composition that leaves the snapshot at `DerivedIndexes::default()`'s `epoch: 0` (`:828-832`) on a store whose epoch is non-zero yields `vector:false` on a freshly booted READY server — the boot's own index reading as stale, which this row fails | the same-unit boot obligation: `src/bin/gnosis_server.rs:414-419` (the build call `:414`, `build_ok` `:415`, the composed snapshot `:416-419`) against `:381` (`Store::new()`), `:433-437` (the pinned wiring order); §9.5.5's surface #1/#4 and its contract table (2)'s *the `epoch` field* and *ordering* rows (both **unchanged**); `src/store/mod.rs:5369-5373` (the build's signature — no epoch in it), `:827-832`, `:1814-1818` |
| `P-IM-19` | IM | **A failed boot build reaches stderr, before the snapshot is composed, and changes NOTHING else — the pinned status surface stays byte-identical.** On a build `Err` the boot writes **one** stderr line whose payload begins with the literal marker **`gnosis-server: boot vector-index build failed: `** followed by the error's display; the write happens **before** the composition that consumes `built`, so the diagnostics do not depend on the composition's shape. The **status surface is untouched by the diagnostic**: the state, every flag, `version`, and the `Degraded` `last_error` string (`src/store/mod.rs:4229-4233`, literal at `:4230`) are **byte-identical** to the no-diagnostic contract, and no **new** wire code, status, `last_error` sentence or field appears anywhere. The line fires **only** on `Err` — never for `Ok(None)` (no provider: nothing was to be built) and never for `Ok(Some(_))` | `strat:build-failure-diagnostic` | ∀ `Err` build outcome (the fail-on-*k*-th provider, `k ∈ {1, n}`, and the always-`Err` provider) with the boot's pinned failure branch applied: stderr carries **≥ 1** line whose payload begins with `gnosis-server: boot vector-index build failed: ` **and** contains the error's own display (`EmbeddingUnavailable`'s rendering) — **the row's home is the BIN and only the bin (REMAND-2 SHOULD-FIX 5, 2026-09-22: the pre-remand cell offered the lib boundary as an alternative, which forked the observable; the layer classification's one bin-level row is now the single home, and the lib variant is a non-pinning note, not an alternative this row can be satisfied by)** — so the assertion is made against a **spawned** `gnosis-server` whose provider is a controlled endpoint answering `/api/tags` but failing `/api/embed`: **the spawned process's stderr** carries the line, while the status surface is read over HTTP beside it (`GET /engine/status` for the failure-branch literal below). ***Non-pinning note (explicitly NOT an alternative observable, REMAND-2 SHOULD-FIX 5):** if the implementer additionally homes the write beside the build seam, a lib-level test may observe the same line too — but that lib observation is **not** what satisfies this row, no test writer may substitute it for the spawned-bin assertion, and the implementation MUST NOT be moved lib-side to make the row cheaper (the pinned `P-IM-19` home is the `[[bin]]` path, following the `R-L2` precedent).** **the counter-witness (no over-logging):** **the counter-witness (no over-logging):** for `Ok(None)` and for a successful build the marker is **absent**; and for the failing case the derived status is **element-wise** the pinned failure-branch literal (`Degraded`, `vector:false`, `embedding:true`, `reranker:false`, core `true`, the store's own fixed `last_error`) — the diagnostic added no observable besides the stderr line | `docs/defects.md` **`U5-ADV-4`** (the discarded `Err`); the composition site `src/bin/gnosis_server.rs:414-419` (`built.ok().flatten()` at `:417`, `build_ok` at `:415` — the two lines the diagnostic is inserted **before**); the unchanged status surface: `src/store/mod.rs:4225-4234` (`last_error` at `:4229-4233`) and §9.5.5's failure table (4)'s *what the boot does with that `Err`* row; the bin's `eprintln!` precedent `src/bin/gnosis_eval.rs:322`/`:337`/`:353` |
| `P-SM-8` | SM | **The status read stays a deterministic, side-effect-free projection — now including the epoch comparison — and the comparison itself is stable.** For **any** store state, repeated `get_engine_status()` calls with no intervening mutation return **element-wise identical** `EngineStatus` values (the freshness term is a pure read of two values, so it cannot oscillate), and the call mutates nothing observable: neither `epoch()` nor `journal_len()` nor the snapshot's `Arc` identity changes, and **no** `swap_snapshot`, `append_journal`, provider write or mask write happens on the read path. The comparison is over **one** snapshot clone per read (the pair `(index, epoch)` is a single `Arc` clone), so a concurrent rebuild swap cannot make a single read compare one snapshot's vectors with another's epoch | `strat:freshness-pure-read` | ∀ `s`: `let a = s.get_engine_status().await; let b = s.get_engine_status().await;` ⇒ `a == b` (state, version, every flag incl. `vector`, `last_error`); `s.epoch()` and `s.journal_len()` are identical before and after **and** `Arc::ptr_eq(&s.snapshot(), &before)` holds **and** `s.snapshot().vectors.is_some()` is unchanged; **and the positive control (the `P-SM-5` precedent, reused rather than weakened):** reading the status, then mutating (a corpus mutation via the public `RagStore` API, or a `swap_snapshot` of a differently-epoched snapshot), then re-reading ⇒ the two reads differ **only** in the licensed positions (the flags whose witness predicate moved, and `state`/`last_error`) while `epoch()`/`journal_len()` **did** move — which is what makes the purity half falsifiable rather than vacuous | §9.5.2's `P-SM-5` (the U3 purity row this one **extends** under the amended predicate — not re-scoped, not renumbered); `src/store/mod.rs:4193-4234` (`get_engine_status` — two lock reads + a clone, no write), `:1814-1823` (`epoch`/`journal_len` observers), `:1849-1853` (`snapshot`'s `Arc` clone), `:1855-1860` (`swap_snapshot` — the only writer, never called by a read) |
| `P-TP-6` | TP | **Every provider request is bounded: the provider client carries a real timeout, and a timeout renders as the EXISTING §11 outcome — the `U5-ADV-2` residual's claimed bound now exists.** The `reqwest::Client` the bin's provider uses is built through **surface #6's lib-visible seam** — `provider_client(PROVIDER_REQUEST_TIMEOUT)` ≡ `reqwest::Client::builder().timeout(PROVIDER_REQUEST_TIMEOUT).build()`, `src/bin/gnosis_server.rs:327` being the bin's **only** construction site — pinned default **`30 s`**, and because there is **one** client the bound covers **both** provider calls: (**REMAND-1, 2026-09-22: the pre-remand text pinned a `const` that lives inside the `[[bin]]`'s **private** `OllamaProvider` (`src/bin/gnosis_server.rs:311-330`), so it was unnamable from a lib test and the row had **no** lib observable; the seam is now the pinned observable, and the bin-level alternative is rejected in surface #6 because its only observable would be a **real `30 s`** timeout — exactly what this row's excluded list forbids.**) the probe (`is_available` ⇒ `GET /api/tags`) and every build/query-time `embed` (`POST /api/embed`). A timed-out request surfaces as the transport `Err` the client already maps to `StoreError::EmbeddingUnavailable` ⇒ **503 `embedding_unavailable`** (FS-13) — **no** new variant, code, status or mapping. The knob set stays closed: **no** env var and **no** CLI flag is added (`--port` + `GNOSIS_SERVER_OLLAMA_URL` + `GNOSIS_SERVER_OLLAMA_MODEL` remain the bin's only three configuration reads) | `strat:provider-request-bound` | ∀ provider call made by the bin (probe, build `embed`, query-time `embed`): the request is issued through a client whose timeout is **configured** (the property is over the **client construction**, so it is asserted through **surface #6's seam** and **never** by sleeping `30 s` in a test — **(i)** `gnosis::PROVIDER_REQUEST_TIMEOUT == Duration::from_secs(30)`; **(ii)** the duration is **falsifiably wired**: `gnosis::provider_client(Duration::from_millis(50))`'s POST to a controlled endpoint that accepts and never answers returns a `reqwest` transport `Err` with `is_timeout() == true` inside the test's own bounded join (a builder that dropped the duration hangs and fails that bound — which is what makes the "builder call's presence" claim falsifiable rather than a code-shape prayer); **(iii)** the bin's construction is **one** site, calling the seam with the constant (REMAND-1, 2026-09-22)); ∀ timeout outcome: the error is `StoreError::EmbeddingUnavailable` and `server_status(&err) == Some((503, "embedding_unavailable"))`, and the **phase consequence** is the pinned existing one — probe timeout ⇒ `BootProvider::Unreachable` (no build attempted, `Degraded` + `vector:false` + `embedding:false`), build timeout ⇒ the `P-IM-15` failure branch (`Degraded` + `vector:false` + `embedding:true` + the fixed `last_error`), query-time timeout ⇒ FS-13 ⇒ 503; **the closure counter-witness:** the bin's configuration reads are **exactly three** (`--port`, `GNOSIS_SERVER_OLLAMA_URL`, `GNOSIS_SERVER_OLLAMA_MODEL`) — no timeout knob exists to read | `docs/defects.md` **`U5-ADV-2`** (the nonexistent bound); `src/bin/gnosis_server.rs:311-330` (`OllamaProvider`: `client: reqwest::Client` `:314`, `reqwest::Client::new()` at **`:327`** — the construction the bound replaces), `:333-363` (`embed`), `:364-375` (`is_available`, the probe), `:347` (the existing transport-error mapping); `src/server.rs:33`/`src/wire/error.rs` (EmbeddingUnavailable ⇒ 503 `embedding_unavailable`, FS-13); §9.5.5's question-5 clause (the bin's three configuration reads, **unchanged**) |

#### The U6 execution plan (§9.5.3's established form, applied to this unit)

- **Command.** `cargo test` (the whole suite; the unit's property layer is never run in isolation from the
  conformance layer in the final trio). The U6 layer needs a tokio runtime + a real `Store` + an injected
  deterministic `EmbeddingProvider` and **must not** reach a live provider or the ambient environment
  (§9.5.5's hermeticity rule, reused); the sandbox-local cargo home precedent applies
  (`CARGO_HOME=$PWD/.cargo-home cargo test`, §9.5.3).
- **Seed pin.** The **same** master seed as the rest of the file's property binary —
  `SEED = 0x9E37_79B9_7F4A_7C15` (`tests/props_gnosis_server.rs:149`), mixed per row through
  `row_seed(tag)` = `splitmix64(SEED ^ tag)` (`:207-208`) — with the **unit-discriminated, injective** tag
  convention (§9.5.3's F1/REMAND-2 ruling, as §9.5.5 applied it): **U6** ⇒ `U6PIM16`, `U6PIM17`, `U6PIM18`,
  `U6PIM19`, `U6PSM8`, `U6PTP6` (**6 tags, 6 rows, one stream each**). ***(REMAND-1, 2026-09-22: the
  pre-remand clause said only "the ASCII of its own name as a `u64`, following the landed `U3PIM7 =
  0x5533_5049_4D37` form", which was **not implementable** — the landed 7-byte tags do not use the bare ASCII
  reading, and a 7-byte name has two plausible spellings, so a TestWriter could not derive the constants. The
  **rule as landed** (re-read this pass against `tests/props_gnosis_server.rs`): the tag's ASCII bytes
  big-endian in the `u64`; a **≤ 6-byte** name is zero-extended on the left (`U3PIM7 = 0x5533_5049_4D37`,
  `U5PSM7 = 0x5535_5053_4D37`), while a **7-byte** name is written as an **8-byte pattern whose final byte
  repeats the name's final byte** (`U5PIM10 = 0x5535_5049_4D31_3030` = `U5PIM1` + `00`; `U5PIM15 =
  0x5535_5049_4D31_3535`; the same for `U5PIM11`/`U5PIM12`/`U5PIM13`/`U5PIM14`, `:343-350`). **The bare
  7-byte big-endian spelling is NOT the pin** — for `U6PIM16` that alternative would be
  `0x0055_3650_494D_3136` and for `U6PIM19` `0x0055_3650_494D_3139`, and using it would silently put the row
  in a **different** stream. **The six constants are pinned EXACTLY as follows** (use them verbatim; all six
  are pairwise distinct, and disjoint from `PIM1`…`PTP1` `:252-258`, the `U2*` `:262-268`, the `U3*`
  `:274-278` and the `U5*` `:343-350` prefixes `0x50…`/`0x5532…`/`0x5533…`/`0x5535…` — U6's prefix is
  `0x5536…`):***

  ```
  const U6PIM16: u64 = 0x5536_5049_4D31_3636; // "U6PIM1" + "66"
  const U6PIM17: u64 = 0x5536_5049_4D31_3737; // "U6PIM1" + "77"
  const U6PIM18: u64 = 0x5536_5049_4D31_3838; // "U6PIM1" + "88"
  const U6PIM19: u64 = 0x5536_5049_4D31_3939; // "U6PIM1" + "99"
  const U6PSM8:  u64 = 0x5536_5053_4D38;       // "U6PSM8"  (6 bytes)
  const U6PTP6:  u64 = 0x5536_5054_5036;       // "U6PTP6"  (6 bytes)
  ```

  Each is mixed through `row_seed` unchanged (`splitmix64(SEED ^ tag)`), so the six streams stay pairwise
  disjoint (XOR with one pinned `SEED` is injective). **The six tags are
  disjoint from every landed set** (`PIM1`…`PTP1` at `:210-216`, the `U2*`/`U3*` sets, and U5's `U5PIM10`…
  `U5PTP5`), so no two rows share a stream and per-row held/broken reporting stays meaningful. No wall clock,
  no thread order, no `thread_rng` input may enter a row.
- **Attempt caps (per-unit reading, §9.5.3's pinned scope).** **≤ 100 generated cases per row** and **≤ 400
  cases for U6's whole layer**: this unit's split is `P-IM-16` **60** / `P-IM-17` 50 / `P-IM-18` 40 /
  `P-IM-19` 45 / `P-SM-8` 60 / `P-TP-6` 45 = **300 ≤ 400** ✔ (every row ≤ 100). The caps are recorded as
  constants (`const B_U6_IM16: u32 = 60;` …) with a `u6_layer_budget_discipline` test asserting `Σ caps ≤ 400`
  and per-row `cases ≤ cap` — **never** `cases == cap` (**a cap is a MAXIMUM, not an expected count** —
  §9.5.5's rule, reused: a row executing fewer cases than its cap is compliant). **The layer is U6's alone**
  and is **not** added to U2's 400, U3's 127 or U5's 355 for cap purposes; the binary total becomes
  `310 + 400 + 127 + 355 + 300 = 1492`, which is a **sum and not a violation** (§9.5.3's per-unit scope pin).
  The earlier totals (`837`, `1177`, `1192`) stand where they are as the dated records they already are and
  are **not** amended by this pass.
- **Stop-after-5.** A row aborts on its **5th distinct counterexample** and reports **at most 5** minimal
  counterexamples — never an unbounded dump, never a panic-without-`cex` (`BROKEN: {cexes:?}`).
- **Held/broken reporting.** One line per row in the **exact required form** §9.5.5 pinned verbatim:
  `[<Property-id>][<Strategy-id>] HELD` (the `Strategy-id` is the row's own `strat:…` token verbatim, and it
  belongs **in the verdict line**, not only in failure messages) or the same line with
  `BROKEN: <≤5 counterexamples>`; **plus** the count line `[<Property-id>] generated cases: N HELD=true|false`.
  A broken row fails the test (the unit is red) and its `Strategy-id` is what the audit reads.
- **One-pass remand rule.** Unchanged from §9.5.3: a broken row is triaged **exactly once** into **host-fix** /
  **package-defect** (a `docs/defects.md` row the **supervisor** lands, + a **negative probe**, never a
  weakened row) / **over-strong-requiring-re-derivation** (the SpecWriter re-derives the row; no row is
  deleted and no id is reused for a different claim).
- **Layer classification.** **Lib-level (5 rows):** `P-IM-16`, `P-IM-17`, `P-IM-18`, `P-SM-8`, `P-TP-6` — each
  asserts a lib-level observable against a real `Store` and an injected deterministic provider (the `Store`
  seams are the existing ones: `snapshot`/`swap_snapshot`/`epoch`/`get_engine_status`/`rag_query`, plus the
  public `RagStore` mutations; **no new accessor is required by any row**). **`P-IM-16`'s observable includes
  surface #1's lib-visible item** — `gnosis::vector_index_is_fresh` is part of that row's assertion surface
  (`REMAND-2 SHOULD-FIX 6, 2026-09-22`: the row asserts the item **and** the derived equivalence, so surface
  #1 is asserted by a row; the lib-level count stays **five** because the item rides `P-IM-16`, not a new row).
  **`P-TP-6`'s observable is NOT a
  `Store` seam but surface #6's lib-visible construction seam** (`PROVIDER_REQUEST_TIMEOUT` /
  `provider_client`) — REMAND-1, 2026-09-22: that is what keeps this row **lib-level**; without the seam its
  only observable would be a real `30 s` bin timeout, which would re-home the row bin-level (rejected in
  surface #6, reason given there). **Bin-level (1 row):**
  `P-IM-19`'s diagnostic is asserted against the **`[[bin]]`** path (a spawned server with a controlled
  provider whose `/api/embed` fails) — following the §9.5.3.2-item-3/`R-L2` precedent that a `[[bin]]`-only
  observable is homed **live**, with the bin's stderr as the observable — and **the home is now PINNED to the
  bin with no alternative (REMAND-2 SHOULD-FIX 5, 2026-09-22): the row's observable cell demotes its
  lib-boundary variant to an explicitly **non-pinning** note there, so this classification (one bin-level row)
  and the observable name one home and one observable (the **spawned process's stderr**).** **Live battery (1 named criterion):** the post-mutation honesty
  end-to-end on a booted bin is the battery's **new row `R-L4`**, named as the home of that obligation
  (`docs/specs/p2-gnosis-server-live-pending-battery.md` §3.5; the `R-L1`/`R-L2`/`R-L3` precedent) — see
  §9.5.6's same-unit table.
- **Read-only audit.** After the layer is green the adversarial reviewer performs the standard **read-only**
  PBT audit against this table (per-row over-strength reasoning, generator coverage, prose counterexamples,
  negative-generator requests); **reviewers never run generators**.

**What U6 must move in the same unit (with `file:line` — the golden-literal + held-row discipline).**
**This table is a TEST-OBLIGATION audit; this pass edits no `src/`, no `tests/` and no other spec file.**
**The governing rule (pinned here so the count is derivable):** *a fixture that composes an index-bearing
snapshot and then reads the derived status (or a vector leg) on a store whose `epoch()` differs from the
snapshot's must align the snapshot's epoch — `epoch: store.epoch()` — or the fixture is asserting the
pre-U6 predicate.* Fixtures on a **fresh** store (`Store::new()`, epoch `0`) whose snapshot is
`DerivedIndexes::default()` (epoch `0`, `:828-832`) are **already aligned and need no edit**, and a fixture
that composes **no** index-bearing snapshot (the U5 conformance corpus's state 3) has no epoch obligation at
all — which is why §9.5.5's conformance corpus state **3** does not move, and of the three index-bearing snapshots
of states 1/2 and contract table (1) only **state 2** (`:517-520`) and **contract table (1)** (`:900-903`) do:
**state 1** (`:459-462`) is consumed only by a store-less `boot_wiring` call and by the fresh-store helper, so it
stays aligned (per this table's row below — REMAND-2 MUST-FIX 1 re-derived by consumer; the pre-remand claims
that "all of 1/2/3 were clear" **and** that "all three move" are both superseded by that row).
***(REMAND-2, 2026-09-22 — the "already aligned" clause is REWRITTEN: the pre-remand reading below justified alignment from a *reader's own store* instead of from the **store that consumes the fixture**, and it is superseded in place (kept as the record). One reading governs, and it is per site: **a fixture label is aligned iff it equals the epoch of the very store that consumes that label**, and a fixture consumed by two different stores must be **split** rather than declared aligned.** The count is derivable from the tree: (i) a fixture on a **fresh** store (`Store::new()`, epoch `0`) consumed only by a fresh-store reader — including `wired_status`'s internal `Store::new()` (`tests/u5_boot_vector_index_conformance.rs:386`) — needs **no** edit; (ii) a fixture consumed by a **corpus-seeded** store (or by **any** store whose epoch is non-zero) must carry `epoch: store.epoch()`; (iii) a single fixture value that feeds **both** kinds of consumer (the state-2 `:517-520` and contract-table-(1) `:900-903` snapshots, per this table's row above — **not** state 1's `:459-462`, whose only consumers are the store-less `boot_wiring` call and the fresh-store helper) must be split into an epoch-`0` label for the fresh-store helper and a `store.epoch()` label for the corpus-seeded store. `seed_store` (`:230`) seeds `1` wiki + `1` document + `1` `update_document` per seeded document (`create_wiki` `:232`, `create_document` `:237`, `update_document` `:297`), i.e. epoch **`2`** for `seed_store(&[])` and **`3`** for a one-document corpus (`(1)` + `(1)` + `(1)`); the pre-remand `≥ 2`/`≥ 3` arithmetic is the same count stated loosely and stands as the record. **State 7** — which reads the **corpus-seeded** store directly (`:842-879`, `:850-853`) — is **not** aligned, and `P-SM-5`'s mutation control (`tests/props_gnosis_server.rs`) is the other named exception. The same per-site test decides every row below.***)***
***(REMAND-1, 2026-09-22 — SUPERSEDED BY TEXT (recorded, not deleted); its "state 1/2 and contract table (1) are aligned" reading was justified only by the helper's own store and is FALSE because each of those snapshots also reaches a corpus-seeded vector leg / status read (REMAND-2 MUST-FIX 1, the row below). What survives from it is the per-site *test* it states — "align against the store that performs the action" — and the `wired_status` fact it cites (that helper builds its own `Store::new()` at `:386`, so the labels it receives are the ones a fresh store consumes). REMAND-2's **consumer-anchored** re-derivation replaces the blanket reading: state 1's `:459-462` is aligned (as that row shows), while `:517-520` and `:900-903` are not.***

| artifact | current (pre-U6) | what U6 makes it | why it must move in this unit |
| --- | --- | --- | --- |
| `tests/wire_conformance.rs:1175-1190` — the `boot_snapshot(ready_vectors)` helper | `DerivedIndexes::default()` (epoch `0`) with `vectors` set for `ready_vectors` | **unchanged** (assert-only): its consumers run on `Store::new()` (epoch `0`, `derived_read` at `:1193-1206`'s `:1199`), so `snapshot.epoch == epoch()` holds and probe (1)'s `vector:true` stays true | it is the V-8.1 producer's input; **the epoch term must not silently falsify it** — this row records that U6 verified it and therefore must **not** touch it |
| `tests/wire_conformance.rs:1208-1225` — `boot_wiring_couples_to_the_derived_read` probe **(1)** (`assert_eq!(reached, honest_ready_subsystems())` at `:1216-1220`, `:1221-1225`'s `reached.embedding && reached.vector && !reached.reranker`) | passes `boot_snapshot(true)` into `derived_read`, which swaps it into `Store::new()` | **assertions unchanged and still green** (epoch alignment holds: `0 == 0`) — but the pin becomes **explicit**: the probe's `derived_read` must keep composing/swapping an **epoch-aligned** snapshot (if a later edit changes it to a `swap_snapshot` of a hand-built snapshot with a non-zero epoch, this probe **fails**, which is the intended trip-wire) | it claims "V-8.1's literal equals what the pinned wiring path actually derives"; under U6 that claim is **only** true for an epoch-aligned snapshot — the claim's **three** preconditions (`Reachable` + `vectors: Some` + **epoch-aligned**, the third **required at this producer site too**, per REMAND-2 MUST-FIX 4), and the record belongs in this unit |
| `tests/wire_conformance.rs:1228-1242` — probes **(2)**/**(3)** (`Absent`/`Unreachable`, `boot_snapshot(false)`) | assert equality with `honest_degraded_subsystems()` | **unchanged** (assert-only) | their snapshot is index-free, so the epoch term is unreachable (`is_some()` short-circuits) — the V-8.2 producer couple is untouched |
| `tests/wire_conformance.rs:708-717` — `honest_ready_subsystems()`'s **doc comment** (the fixture's own value `vector:true` at `:723` and the V-8.1 literal at `:1109` **do not move**) | the scope note names two preconditions: the boot-build contract's `Reachable` **and** a snapshot whose `vectors` is `Some` | the **comment** gains U6's third precondition — **epoch-aligned** (`snapshot.epoch == store.epoch()`) — and the sentence "the rule that always holds is `subsystems.vector == snapshot().vectors.is_some()`" (the pre-U6 predicate) is annotated as **the pre-U6 record** | §5.8's rule sentence and §9.5.5's `P-IM-14` predicate-marker are reconciled in the same unit (§9.5.6's reconciliation table); a fixture comment that still states the superseded predicate is exactly the doc-drift class this table exists to prevent. **No value, literal byte, assertion or message moves** |
| `tests/u5_boot_vector_index_conformance.rs:459-462`, `:517-520`, `:900-903` (the composed boot snapshots of states 1/2 and contract table (1)) | `DerivedIndexes { vectors: …, ..DerivedIndexes::default() }` — epoch `0` | **Re-derived by RULE, per site, naming each site's status/leg consumer and that consumer's store — REMAND-2 MUST-FIX 1, 2026-09-22: the "ALIGNED as written … `epoch: store.epoch()` is NOT required" reading this cell carried (justified only by `wired_status`'s own `Store::new()`) is superseded, and it is superseded by *consumer-anchored* text, because the three sites are NOT the same case — TWO of them are STALE and one is aligned.** **The rule applied: a label is aligned iff it equals the epoch of the very store that consumes it; a fixture whose single value reaches two stores must be SPLIT rather than "aligned". Store epochs from `seed_store` (`:230`): `seed_store(&[])` = `1` `create_wiki` (`:232`) + `1` `create_document` (`:237`) = epoch **`2`** (its corpus slice is empty, so no `update_document`); `seed_store(&mixed_corpus())` = `+1` `update_document` (`:297`) = epoch **`3`**. `wired_status` (`:377-393`) is NOT the corpus-seeded store: it builds its own `Store::new()` at `:386` (epoch **`0`**).** **Site by site (each consumer named with its store's epoch, each red assertion named):** <br>**(a) `:459-462` — state 1 — ALIGNED AS WRITTEN; NO EDIT (this site is the one the pre-remand cell got right, for a reason the pre-remand text did not state).** Its consumers are `boot_wiring(Reachable, &snap)` at `:463` — which reads **no** store and no epoch (its `flags.vector` is `is_some()`-only, freshness rule 3) — and `wired_status(..., snap)` at `:470`, which swaps the label into its **own fresh `Store::new()`** (epoch `0`), so `0 == 0` holds and `:472-483`'s post-U5 READY-mask assertion (with `vector:true`) **passes**. The corpus-seeded store of that test (epoch `3`) is consumed by the **build** assertions (`:431-455`) and by the `boot_wiring` flag assertion — **never** by this snapshot's status read. **A TestWriter MUST NOT "align" this site to `store.epoch()` (=`3`): that would make the helper's read stale and turn a passing assertion red.** (The `mode=Vector` `rag_query` at `:541-551` the reviewer's note associates with this range belongs to **state 2's** snapshot, site (b) below — state 1 never swaps into its corpus store.) <br>**(b) `:517-520` — state 2 (`seed_store(&[])`, epoch `2`) — STALE; MOVED TO THE EPOCH-ALIGNMENT OBLIGATION.** Consumers: (i) `wired_status(..., snap.clone())` at `:521-525` — fresh store, epoch `0` (aligned, `:526-530`'s `status.subsystems.vector == true` passes); **and** (ii) the **same label** swapped **directly into the corpus-seeded store** at `:536` (epoch `2`), whose `mode=Vector` `rag_query` at `:541-551` then answers `Err(VectorIndexUnavailable)` (FS-14) so `.expect("an empty index serves an empty result")` at `:551` **panics** — the flag read on that store would likewise be `vector:false`. **Pinned edit: SPLIT the fixture** — an `epoch: store.epoch()` label for the swap at `:536` (the store is in scope and does not move afterwards: `swap_snapshot` appends no journal entry) and the epoch-`0` label for the `wired_status` call at `:521-525`. <br>**(c) `:900-903` — contract table (1) (`seed_store(&mixed_corpus())`, epoch `3`) — STALE; MOVED TO THE EPOCH-ALIGNMENT OBLIGATION.** Consumers: (i) `wired_status(..., built_snap)` at `:904-905` — fresh store, epoch `0` (aligned); **and** (ii) the same label fed to the built-snapshot **query path** on the corpus-seeded store (state 2's `:536` swap + `mode=Vector` `rag_query` body, `:541-551`), where the flag is `false` and the query is FS-14 (**RED** at `:907`'s `assert!(reachable.subsystems.vector && …)` for the store-consumed instance). **Pinned edit: SPLIT as in (b).** **The per-site rule this row fixes (and why the helper's own store cannot settle the sites): the helper's `Store::new()` settles ONLY the labels that helper receives, and the two `seed_store`/`u5_seed_store` recipes give epochs `2`/`3`, so a label the corpus-seeded store consumes must equal `store.epoch()`. A fixture value that reaches both stores is SPLIT; it is never declared aligned on the strength of one of them.** **The corpus-seeded U5 sites that move as single-label fixtures remain the state-7 row below and the `props_gnosis_server.rs` rows further down** | the pre-remand cell was wrong in the same direction REMAND-1 was fixing (it justified alignment from a *reader's* store instead of the *consuming* store), and only one of its three sites is actually aligned: (a) is aligned for a reason the cell did not state (its only consumer reads a fresh store), while (b) and (c) are stale because the same value reaches a corpus-seeded leg/status read. A TestWriter who treats the three as one case either "aligns" (a) into a red helper read or leaves (b)/(c) asserting the pre-U6 predicate — both are the failure this row now prevents. **No fixture may be aligned to two stores at once, hence the split** |
| `tests/u5_boot_vector_index_conformance.rs:850-853` (valid/fail **state 7**, `:842-879`) | `swap_snapshot(DerivedIndexes { vectors: Some(vi), ..default })` on the seeded store | **`epoch: store.epoch()`** — and then the test still asserts `status.subsystems.vector` (`:857`) and FS-13 (`:874`) | it asserts a **fresh** index with no provider ⇒ FS-13; with a stale index the freshness check precedes the provider check and the answer would be FS-14 — the fixture must keep the case it names |
| `tests/u5_boot_vector_index_conformance.rs:1070-1098` (contract table (6), the journal/epoch guards) | asserts the **build** moves no epoch and swaps no snapshot (`Arc::ptr_eq` at `:1092`, `vectors.is_none()` at `:1096`) | **unchanged** (assert-only) | it is the `P-IM-12`/`P-IM-18` guarantee the U6 predicate **depends on**: the build must stay epoch-agnostic for the caller's composition to be the only stamp |
| **new** `tests/u5_boot_vector_index_conformance.rs` — **the HOME IS PINNED (REMAND-2 NOTE 10, 2026-09-22: the pre-remand cell left the home open — "or the U6 conformance file, the TestWriter's call" — which made the one **new** case unfindable by clause; the U5 conformance file is the pin because every named state of §9.5.5 lives there and the case is state 3's conformance twin. A TestWriter who prefers a separate U6 file may put **the same case** there **only** by recording that move in this table's row: no `file:line` claim in this section depends on the choice, because the case is new and carries no pre-existing anchor)** | — | **a stale-index conformance case**: a `Ready` store + wiring + provider, an index-bearing snapshot with `epoch != store.epoch()` ⇒ flag `false` + FS-14 (503) + no `embed` call; and the aligned control ⇒ flag `true` + `Ok` | §9.5.5's valid/fail state 6 is the **unbuilt** case only; the **stale** case is a **new** documented fail-state (state 3 of §9.5.6's table) and needs its own case, not a reinterpretation of state 6 |
| `tests/rag_query_integration.rs:272-282` — `seed_vector_index(store, entries)` | `epoch: 1` hard-coded | **`epoch: store.epoch()`** (the helper takes the store) | it is the shared read-side fixture of the vector/hybrid integration tests, and **all three** of its call sites (`:607`, `:662`, `:751`) run on a store that has seeded a wiki **and** documents (epoch `≥ 3`), so the hard-coded `1` is a **stale** snapshot there. **The obligation is wider than "every vector-leg expectation would become FS-14" (REMAND-1, 2026-09-22 — the pre-remand cell under-stated it, because two of the three sites would not surface FS-14 at all):** <br>(a) `:607`'s `rag_query_vector_mode_returns_vector_leg_hits` (`:591-643`; `mode=Vector`, `.unwrap()` at `:625`) **would** answer FS-14 and panic; <br>(b) `:662`'s FS-13 test `rag_query_vector_mode_unavailable_provider_is_embedding_unavailable` (`:649-682`, `assert_eq!(err, StoreError::EmbeddingUnavailable)` at `:681`) would answer **FS-14 `VectorIndexUnavailable`, not `EmbeddingUnavailable`** — i.e. the staleness check precedes the provider check (the per-site consultation clause), so **this site's meaning moves**, not merely its outcome; <br>(c) `:751`'s `rag_query_hybrid_mode_merges_three_distinct_legs` (`:726-802`) would **not** error at all — the stale index makes the fused vector leg degrade to **empty** (the refuse-suppressed fusion site), so the query still returns `Ok` and the row's **non-degeneracy assertion at `:790-795`** (`"vector leg must contribute (a distinct, non-degenerate leg)"`) is what fails. **All three are named assertion sites that must be re-derived/aligned with the fixture edit**; the fixture edit alone is not the whole obligation |
| `tests/rag_query_integration.rs:689-716` — `rag_query_vector_mode_index_not_built_is_vector_index_unavailable` (FS-14) | an **unbuilt** (`vectors: None`) READY store ⇒ FS-14 | **unchanged** (assert-only) — its precondition is now **one of two** FS-14 routes | it stays valid, but after U6 it no longer *uniquely* pins "index not built": the *stale* route reaches the same variant. The stale route needs the **new** case above; **this test must not be edited to pretend it covers both** |
| `tests/rag_query_integration.rs:1757-1847` — **`concurrent_rag_query_reads_race_a_concurrent_index_rebuild`** (the initial swap `:1783-1787`, the concurrent writer's `epoch: 1..=10` swaps `:1829-1833`, the **8 readers** `:1792-1820`) | a store built by `Store::new()` + `new_wiki` (`:1762`) + `new_doc` (`:1768`) + `apply_graph` (`:1769-1777`) — **epoch ≥ 3** — with an index-bearing snapshot labelled `epoch: 1`, and a writer that re-swaps `epoch: 1..=10` while the readers run | **NOT unchanged (REMAND-1, 2026-09-22 — the pre-remand cell declared this site unchanged and was FALSE for it):** every snapshot the writer installs (and the initial one) has an epoch `≠ store.epoch()`, so under the pinned predicate each is **stale** ⇒ each of the **8 concurrent readers** — whose `:1799-1810` request is `mode: Some(QueryMode::Vector)` and whose `.expect("concurrent vector rag_query must return Ok(RagResult)")` sits at `:1810` — takes `Err(StoreError::VectorIndexUnavailable)` and the `.expect` **panics**. **Required edit: compose every swap at the store's epoch** (`epoch: store.epoch()` / `epoch: wstore.epoch()` at `:1783-1787` and `:1829-1833`; the epoch does **not** move during this test, since `swap_snapshot` appends no journal entry, `src/store/mod.rs:1855-1860`, so all swaps carry the same aligned epoch) — the test's claim is reader/vector coherence under a concurrent swap, **not** an epoch-varying read, so the aligned form preserves its purpose and its assertions (`:1813-1818`'s vector-leg hit, `:1843-1846`'s joins) | this is the one concurrency site a vector leg reaches. **A later unit that wants epoch-varying snapshots read through a vector leg must name its own expectation** (the aligned form is the pin here, and the deliberate mismatch is *not* this test's claim) |
| `tests/rag_query_integration.rs:1853-1927` — **`derived_snapshot_swap_is_atomic_under_concurrent_readers`** (the epoch-1/epoch-2 pair at `:1859-1863` and `:1907-1919`) | snapshots labelled `epoch: 1`/`epoch: 2`, swapped repeatedly under 8 reader threads | **unchanged — this test does not read `get_engine_status` and does not call a vector leg**; its assertions are over `snapshot()` (`:1876-1895`, the `snap.epoch` ↔ `snap.vectors` match) | the U6 pin for this site is a **negative** one: it is the `swap_snapshot`-labeling precedent (a caller labels the snapshot it installs), and it must not be "fixed" into alignment — the deliberate mismatch there *is* the torn-snapshot test. **If a later edit makes it read a derived status or call a vector leg, it must revisit this row** (which is exactly what the sibling row above was corrected for) |
| `tests/retrieval_stack_integration.rs:131-141` — **this file's own** `seed_vectors(store, entries)` helper (distinct from `rag_query_integration.rs`'s `seed_vector_index`) | `epoch: 1` hard-coded (at `:139`) | **`epoch: store.epoch()`** (the helper takes the store, `:131`) | **REMAND-1 MUST-FIX, 2026-09-22 (this row is NEW; the pre-remand table asserted of the vector-leg sites that "these tests do not call a vector leg" — that is FALSE for every site in this file).** The helper's call sites — `:329`, `:357`, `:435`, `:579`, `:614`, `:640`, `:669` — all run **after** `new_content_doc`/`new_doc` corpus mutations (`:327-328`, `:356`, `:434`, `:578`, `:613`, `:639`, `:667-668`), so the store's epoch is `≥ 3` and the `epoch: 1` snapshot is **stale** at every one of them. `vector_search`'s refuse is **ON** under the pinned predicate (the per-site consultation clause), so each of those sites answers `Err(StoreError::VectorIndexUnavailable)` instead of its named expectation: `vector_search_returns_top_k_by_cosine_similarity` (`:324-350`), `vector_search_embedding_provider_unreachable_is_unavailable` (`:353-369` — it names **FS-13** and would answer **FS-14**, the staleness check preceding the provider check), `vector_search_binary_first_pass_degrades_to_full_when_binary_unbuilt` (`:429-…`), `vector_search_binary_candidate_pool_zero_is_validation_error` (`:575-…` — FS-3 is validated *before* the index read, `src/store/mod.rs:4292-4296`, so this one survives the predicate and must stay as it is), the two wiremock-seam reads at `:614`/`:640` (`wiremock_http_embedding_provider_serves_the_vector_seam` `:593-626`, `.unwrap()` at `:624`; and `wiremock_http_embedding_provider_404_is_embedding_unavailable` `:630-650`, whose `assert_eq!(err, StoreError::EmbeddingUnavailable)` at `:649` would answer FS-14), and `vector_search_does_not_leak_vectors_across_wikis` (`:662-…`). **Required edit: `epoch: store.epoch()` at `:139`**; the named sites above are the ones that would otherwise red, and each keeps its own expectation afterwards. **The `QueryMode::Vector`/`Hybrid` `rag_query` sites at `:532`, `:916`, `:937` are NOT fed by this helper** (they use their own seeding) and stay out of this row |
| `tests/retrieval_stack_integration.rs:407-411`, `:516-520`, `:883-887` — **this file's three INLINE index-bearing snapshots** (each `swap_snapshot(DerivedIndexes { lexical: None, vectors: Some(vi), epoch: 1 })`), the sites the `seed_vectors` row above did not enumerate | `epoch: 1` hard-coded at each of the three (labels at `:410`, `:519`, `:886`), on stores the file has already put past epoch `1` | **`epoch: store.epoch()` at all three — REMAND-2 MUST-FIX 3, 2026-09-22 (this row is NEW; the `seed_vectors` row above enumerates the helper and its seven call sites, and the file carries these three more, each with a vector-mode consumer the predicate falsifies). Every site is named with its consumer, its store's epoch as the tree shows it, and the pre-U6 assertion that breaks:** <br>**(a) `:407-411`** — the coarse-to-fine test `vector_search_binary_first_pass_narrows_pool_then_full_cosine` (`:391-424`; the `:429` test is its `binary`-unbuilt sibling): the store has `new_content_doc` × 2 (`:396-397` ⇒ `1` `create_wiki` + `2` `create_document` + `2` `update_document` ⇒ epoch **`5`**), and the snapshot is consumed by **`store.vector_search(&w, "q", 2, &MockProvider::ok(), &fp).unwrap()`** at **`:417-420`** ⇒ under U6 the direct `vector_search` site's refuse is **ON**, so the call is `Err(StoreError::VectorIndexUnavailable)` and the `.unwrap()` **panics** (its `assert_eq!(hits.len(), 2)` at `:421` is unreachable). <br>**(b) `:516-520`** — the candidate-pool test `binary_candidate_pool_none_default_caps_pool_at_tenx_topk` (`:459-570`): the store has `1` `create_wiki` + `DEFAULT_CAP + WORSE = 22` `new_content_doc` calls (the first seeding loop `:475-490`, the `best_hit` loop `:493-514`) above `:516` ⇒ epoch **`45`**, and after the swap the file wires the provider (`:521`) and the state (`:522`) and reads through **`rag_query(mode: Vector)`** at **`rag_query(...)`'s `.unwrap()` at `:540-543`** (the `opts(false, None)` control) plus its two siblings' `.unwrap()`s at `:549-552` and `:560-563` (`opts(true, Some(u64::MAX))` and `opts(true, None)`, whose own control/assertion pairs at `:553-556`/`:564-569` never run) ⇒ all three are `Err(VectorIndexUnavailable)` (FS-14) and the three `.unwrap()`s **panic**. <br>**(c) `:883-887`** — the HyDE routing test `hyde_opt_in_routes_hypothetical_through_vector_leg` (`:855-…`): the store has `1` `create_wiki` + `4` `new_content_doc` calls (`:864-870`, each `create_document` + `update_document`) ⇒ epoch **`9`**, and the snapshot is consumed by **two** `mode=Vector` `rag_query(...).unwrap()` sites — `hyde: true` at **`:910-922`** and `hyde: false` at **`:931-943`** — plus the hit-presence assertion at **`:923-927`** (`hyde.results.iter().any(on_hypo)`), so both `.unwrap()`s are FS-14 and the row's HyDE-routing claim never runs. **Required edit at all three: `epoch: store.epoch()`** (the store is in scope at each swap; none of the three tests mutates the corpus after the swap, so one aligned label per site is stable — `swap_snapshot` appends no journal entry, `src/store/mod.rs::swap_snapshot`). **The enumeration rule that produced these three (and that a TestWriter can re-run) is the sweep clause in the `seed_vectors` row above:** every `swap_snapshot(DerivedIndexes { … })` in the file, then its vector-leg/status consumers — one helper + three inline fixtures here, twelve consumers in total | the same-unit rule of this table (a fixture that feeds a status read or a vector leg must be labelled at the consuming store's epoch) applies to an **inline** fixture exactly as it does to a helper-produced one; leaving these three out made this file's red set **larger** than the table said, which is the class of error this table exists to prevent (the REMAND-1 note above named the file's **helper** only) | the file was named **NOWHERE** in this section before this remand | **`epoch: store.epoch()`** at `:254` | **REMAND-1 MUST-FIX, 2026-09-22 (this row is NEW; the file's absence was the largest single unexplained red set).** Call sites `:694`, `:1005`, `:1119`, `:1242`, `:1336` all follow corpus seeding (`:687-692`, `:1003-1007`, `:1116-1118`, `:1327-1332`), i.e. epoch `≥ 3`, so every one is a **stale** snapshot. Named read sites whose meaning moves: <br>**(i) vector mode:** `:701` (inside the all-modes leak loop `:698-704`, whose `.unwrap_or_else(|e| panic!("mode {mode:?} must be Ok on a wired store: {e:?}"))` at **`:716`** **panics** under U6), `:1050` (`P-SM-3` — `p_sm_3_repeat_query_and_stream_are_element_wise_stable`), `:1342` (the count-bound loop's `QueryMode::Vector`); <br>**(ii) hybrid mode** (which does **not** error — the refuse is suppressed at the fusion site): `:702`, `:1137`, `:1252`, `:1343` keep returning `Ok` with an **empty vector leg**, so the assertions that require a vector contribution are what break — e.g. `:1258-1269` (`P-SM-4`'s "dedup of 3 legs → 1" and its non-vacuity check "it really was in all three legs"), and any count-bound row whose expectation assumed the vector leg. **Required edit: `epoch: store.epoch()` at `:254`**, with those named sites re-derived (the hybrid ones by their own contribution assertions, not by an FS-14 expectation) |
| `tests/props_gnosis_server.rs:4420-4442` (`u3_snapshot_for`, `epoch: 0` for probe 1) and `:5139-5164` (`u3_boot_snapshot`, `u3_boot_snapshot_with_empty_index`, `epoch: 0`) | U3's snapshot-axis fixtures | **unchanged** (assert-only): every U3 consumer (`u3_store`/`run_boot_case`, `:5183-5259`'s `let store = Store::new()` at `:5238`) runs on a **fresh** store (epoch `0`), so `0 == 0` | the U3 rows must stay HELD under U6; this row records the verification that they do, and forbids a TestWriter from "aligning" them into a change they do not need |
| `tests/props_gnosis_server.rs:5367-5414` — `P-SM-5`'s **mutation-interleaved positive control** (probe 4) | `swap_snapshot(DerivedIndexes { vectors: Some(VectorIndex::default()), epoch: 1 })` (`:5373-5377`) then a journal-appending call (`create_wiki`, `:5380`) ⇒ `epoch_after = 2`, and the row asserts `after.subsystems.vector` is `true` (`:5410-5415`) | **the assertion's meaning is re-derived**: under U6 the snapshot is **stale** at `epoch_after` (`1 ≠ 2`), so the licensed-change set for that mutation now includes `vector` flipping `true ⇒ false`; the row must either (i) align the snapshot's epoch (`epoch: epoch_after` after the mutation) so the probe keeps testing "the flag follows the wired snapshot", **or** (ii) keep `epoch: 1` and assert the **stale** outcome (`vector == false`) as the licensed change, with the mutation-moved observers unchanged | this is the one **held U3 row** whose assertion the amended predicate falsifies as written. §9.5.2's `P-SM-5` claim (a pure read changes nothing) is **not** re-scoped; only this probe's fixture/expectation pair is (disposition (i) is the pinned preference: it preserves the probe's purpose, and §9.5.6's `P-SM-8` carries U6's own version of the same control) |
| `tests/props_gnosis_server.rs:7322-7361` — `P-IM-14`'s snapshot corpus (the `"empty-index-with-epoch"` variant at `:7352-7359`, `epoch: 7`) and its four probe blocks (`:7388-7396`, `:7397-7401`, `:7404-7410`, `:7411-7418`), plus the V-8.1 block (`:7435-7459`) and the harness helper `u5_expected_flags` (`:6273-6282`) | `flags.vector != snap.vectors.is_some()` / `derived.vector != store.snapshot().vectors.is_some()` / `derived != flags` / `derived != want` asserted for **every** snapshot × `Store::new()`, with `u5_expected_flags` also using `vector: snapshot.vectors.is_some()` (`:6278`) | **the corpus KEEPS its mismatched variant and the probes are re-derived — this is the chosen remedy (REMAND-1, 2026-09-22).** The pre-remand cell offered "align the variant to `0`" as an equal alternative and left the third equality's satisfiability open; that is now **closed**, because `boot_wiring` is **pinned unchanged** (`src/lib.rs:90-93`, its `vector` = `snapshot.vectors.is_some()` at `:105`, rule 3 above) and **no fixture edit can make `derived == flags` hold for the `epoch: 7` variant** — `derived.vector` is `false` (stale) while `flags.vector` is `true`. **The pinned re-derivation, per probe:** <br>(1) `:7388-7396` (`flags.vector == snap.vectors.is_some()`) — **unchanged** (it is `boot_wiring`'s own pinned pre-U6 surface, and it stays true for every variant); <br>(2) `:7397-7401` (`derived.vector == store.snapshot().vectors.is_some()`) — becomes the **epoch-aware derivation equality** `derived.vector == (store.snapshot().vectors.is_some() && store.snapshot().epoch == store.epoch())`, read through **one** snapshot clone (surface #3); <br>(3) `:7404-7410` (`derived == flags`) — the invariant is now **scoped**: it is asserted **only for the epoch-aligned variants**, and the mismatched variant gets the explicit two-sided assertion `derived.vector == false ∧ flags.vector == true` (so the probe stays falsifiable rather than being deleted); <br>(4) `:7411-7418` (`derived != want`) — `u5_expected_flags` (`:6273-6282`) must gain the store's epoch as an input and apply the same epoch-aware rule (or be applied only to the aligned variants — the implementer/TestWriter's spelling, the reading is pinned); <br>(5) the V-8.1 block `:7435-7459` (`derived != v81`, `v81.vector = index_bearing`) is asserted **only for the epoch-aligned `Some` instances** — its own comment ("V-8.1 is the `vectors: Some` instance of the rule") gains the third precondition, exactly as §5.8's REMAND-1 clarification and `tests/wire_conformance.rs`'s comment row pin | the row is U5's `P-IM-14`; U6 **supersedes its predicate in place** (§9.5.6's reconciliation table). The probes must not keep asserting the pre-U6 equality, and the `epoch: 7` variant must not be left silently asserting `true` **nor** be quietly aligned away — it is the row's only falsifiable stale instance |
| `tests/props_gnosis_server.rs:7823-7826`, `:7846-7849` (`P-IM-15`'s post-boot index-bearing swaps, read at `:7831`) and `:8069-8072` (`P-SM-7`'s READY empty-index swap) — all on `u5_seed_store` stores | `DerivedIndexes { vectors: Some(vi), ..DerivedIndexes::default() }` (epoch `0`) swapped into a **corpus-seeded** store and then read through `get_engine_status` / a `mode=Vector` query | **`epoch: store.epoch()`** at each of the three sites — **CONFIRMED moving (REMAND-1, 2026-09-22: the pre-remand cell said "reviewed per site", which is not actionable)**. `u5_seed_store` (`:6078-6140`) seeds `1` wiki + `2` journal appends per document (the `seed_store` recipe above), so its stores are at epoch `≥ 2` and an epoch-`0` label is **stale**; the status read at `:7831` (`!store.get_engine_status().await.subsystems.vector` ⇒ a counterexample) and the query at `:7858-7859` (`mode: Some(QueryMode::Vector)`, expecting `Ok`) both fail under the pinned predicate | these are the rows whose **observable** is the derived status; the sites are named by line now (the failing red set is corroboration, not the list). **The index-free failure-branch sites keep `DerivedIndexes::default()`** — the epoch term is unreachable there (`is_some()` short-circuits), so they need no edit |
| `tests/props_gnosis_server.rs:361-387` (the landed cap constants + `U5_EXECUTED_TOTAL`) and `:8460-8512` (`u5_layer_budget_discipline`) | `B_U5_*` … = `355`, `U5_EXECUTED_TOTAL = 314` | **unchanged**, plus **six new** `B_U6_*` constants (`60/50/40/45/60/45 = 300`) and a **`u6_layer_budget_discipline`** test asserting `Σ caps ≤ 400` and `cases ≤ cap` per row (`:16-40`'s form) | the per-unit cap discipline §9.5.3/§9.5.5 pin; the U5 figures must not be moved to make room for U6's layer |
| **`P-IM-16`'s observable cell + its §9.5.4 note** — **surface #1** (`vector_index_is_fresh`, the **lib-visible predicate** at the crate root) — **REMAND-2 SHOULD-FIX 6, 2026-09-22 (this row is NEW: surface #1 pins the predicate as a lib-visible item "so a `[[bin]]`-only path stays assertable", but none of the six rows' observables named the item, so the pin was asserted by no row and no TestWriter could find its assertion surface)** | surface #1's contract cell states the predicate is **lib-visible** and crate-root nameable; the six observables call the *behaviour* (`f == (snap.vectors.is_some() && snap.epoch == s.epoch())`, one bound clone) without naming the item | **the pinned remedy (chosen: name it, do not drop it): `P-IM-16`'s observable cell and its §9.5.4 coverage note are read as carrying a THIRD assertion half — the lib-visible item's own assertion surface:** (i) the predicate item is **nameable from `tests/`** at the crate root (`gnosis::vector_index_is_fresh(...)`, the F11-seam precedent of `boot_wiring`/`build_boot_vector_index`), (ii) on the row's own sampled pairs the item returns **exactly** what the row's equivalence asserts — `vector_index_is_fresh(&snap, s.epoch()) == (snap.vectors.is_some() && snap.epoch == s.epoch())` for every generated pair (so the row fails if the lib item and the derived flag ever disagree), and (iii) its totality/purity half as surface #1 pins it (`None` ⇒ `false` short-circuit; no mutation, no lock beyond the caller's read). **The row's existing half is unchanged:** the equivalence `f == (snap.vectors.is_some() && snap.epoch == s.epoch())` on **one bound clone**, the four boundary instances, the write-independence probe and the Excluded list all stand. **Nothing else moves:** the item is **not** a store accessor, it stays one of U6's **two** lib-visible items (surface #1 and surface #6), the layer classification's lib-level count stays **five**, and no cap, tag, id or count changes | the lib-visibility of surface #1 is a **pin**, and a pin with no row asserting it is exactly the class of gap this table audits (surface #5's observable was fixed the same way by REMAND-1's surface #6); the alternative the review offered — **dropping** the lib-visibility requirement — is **rejected** because a `[[bin]]`-only path must stay assertable hermetically (the F11 reason surface #1 states), so naming the item as `P-IM-16`'s assertion surface is the remedy that keeps both the pin and the row count |
| **`tests/props_gnosis_server.rs:7437-7460`** — the **V-8.1-shaped block** of the `P-IM-14` probe (`:7437`'s loop over the corpus, `index_bearing` at `:7439`, `store = Store::new()` + `swap_snapshot` at `:7441-7442`, the `v81` mask with `vector: index_bearing` at `:7446-7453`, the `derived != v81` assertion at `:7454-7459`, plus its `!index_bearing && derived.vector` negative at `:7475-7481`) — **the per-probe re-derivation's (5)** | asserts `derived == v81` with `v81.vector = index_bearing` (`:7450`) for **every** variant of the corpus on a **fresh** `Store::new()` (epoch `0`) | **SCOPED BY THE SAME EPOCH-ALIGNED FILTER AS PROBE (3) — REMAND-2 MUST-FIX 4, 2026-09-22 (this row is NEW; the row above named the block in its artifact list but its per-probe re-derivation never addressed it, so the `"empty-index-with-epoch"` variant — `epoch: 7`, `:7352-7359` — was left as an unaddressed counterexample: it is index-bearing and therefore **stale** on the epoch-`0` store, so `derived.vector == false` while `v81.vector == true` and the block **fails as written**).** **The pinned remedy is the scoping, never a corpus edit:** `v81.vector = index_bearing` is asserted **only for the epoch-aligned variants** (here every variant **except** `"empty-index-with-epoch"`), and that variant gets the explicit **two-sided** assertion `derived.vector == false ∧ v81.vector == true` — the probe-(3) shape. The block's other halves **stand unchanged**: `derived.reranker == false` (`:7461-7466`), the core-legs check (`:7467-7472`), and the `!index_bearing ⇒ !derived.vector` negative (`:7475-7481` — an index-free snapshot is never fresh, so it is untouched by the epoch term). **A sweep of `tests/props_gnosis_server.rs` this pass finds no other V-8.1-shaped assertion** (no second `v81`/V-8.1 mask construction anywhere in the file), so this row completes the file's V-8.1 obligation. **The V-8.1 *producer* row above is reconciled in the same direction:** its claim "V-8.1's literal equals what the pinned wiring path actually derives" now names **three** preconditions — `Reachable` **and** `vectors: Some` **and** **epoch-aligned** — with the third **required at that site too** (not only in `tests/wire_conformance.rs`'s `honest_ready_subsystems()` comment the row already pinned), which is what makes the `:7450` assignment derivable here rather than accidental |
| `tests/blind_u3_status_honesty_greens.rs:130-159` (`snapshot_with_empty_vector_index` `:134-140`, epoch `0`; **`snapshot_with_populated_vector_index` `:144-159`, epoch `1` — its `epoch` is at `:157`**) and the blind U5 set's `boot_snapshot` (`tests/blind_u5_boot_vector_index_greens.rs:979-983`, `epoch: 7`) | blind-test fixtures (a **TestWriter's** files — **not edited by this pass**) | **named obligations, not edits — and one of them is a RED site (REMAND-1, 2026-09-22; the pre-remand cell claimed both U3 fixtures feed only `Store::new()`-based probes and are therefore correct, which is FALSE for the populated one):** <br>**(a) the populated variant is stale.** `:479` swaps `snapshot_with_populated_vector_index()` (epoch **`1`**) into the `Store::new()` store opened at `:451` (epoch **`0`**), so under the pinned predicate the read at `:480` yields `vector == false` while the assertion at `:481-483` requires `true` ("U3-3 … a populated index is `vector:true` too"). **The blind U3 set must re-derive this scenario: align the fixture's epoch (`epoch: store.epoch()` at `:157`) or split it into the aligned instance (`true`) plus the mismatched instance (`false`)** — the empty variant (epoch `0`, also used at `:471`, `:650`, `:697`, `:749`, `:857`) is aligned and stays; <br>**(b) the blind predicate-checker itself is pre-U6.** `assert_flags_match_predicates` (`:192-202`) asserts equality against `expected_from_store(s)` and states the rule in its message as `vector == snapshot().vectors.is_some()` (`:198`); both the helper and that sentence must carry the epoch term, or every scenario it guards is asserting the superseded predicate; <br>**(c)** the U5 blind fixture `:979-983`'s `epoch: 7` never feeds a status read, so it stays out of this obligation (a blind U6 set that feeds it to a status read inherits (a)) | the blind layer is a separate verification layer; this table's job is to **name** the sites whose alignment a blind U6 writer must re-check — including the one that is already red — not to assert them |
| `tests/blind_u5_boot_vector_index_greens.rs:1358-1470` (`u5_15_the_flag_is_the_wiring_over_every_boot_and_snapshot_pair`, whose header comment `:1352-1355` states the rule as `subsystems.vector == snapshot().vectors.is_some()` "for EVERY pair") and `:1219-1227`, `:1547-1560`, `:1582-1652` (the failure/FS-8/FS-13 blind probes) | blind U5 assertions on the **pre-U6** predicate | **named obligations, and the loop itself is still GREEN — this is a REMAND-1 re-derivation of the pre-remand cell (2026-09-22), which had asserted the opposite.** Verified this pass: every pair the loop walks is built from `DerivedIndexes::default()`-based snapshots (`:1372-1382`, all epoch `0`) applied to a **fresh** `Store::new()` (`:1393`), so <br>(i) `:1428-1433`'s `flags.vector == snap.vectors.is_some()` is **unchanged and still true** — `flags` is `boot_wiring`'s return, whose pre-U6 surface rule 3 **pins**; <br>(ii) `:1396-1401`'s `derived == flags` **still holds** because every pair is epoch-aligned — but the invariant is now **scoped** (rule 3): a U6 blind set that adds a **mismatched** pair must **not** extend this loop's equality to it, and the header comment `:1352-1353`'s "for EVERY pair" must gain the alignment precondition (the third precondition, beside `Reachable` + `vectors: Some` — the same sentence §5.8's REMAND-1 clarification and the `tests/wire_conformance.rs` comment row carry). The FS-8/FS-13 probes are **unchanged** (`EngineUnavailable` precedes freshness; FS-13 needs a **fresh** index) | the adversarial/blind layers must read a **U6-relative** register, not a superseded predicate — the same discipline §9.5.1's "TRUE of the unit's GREEN implementation — checked, not assumed" paragraph pins. **The one blind site that is actually red is the U3 populated-index scenario in the row above** |
| **`docs/specs/p2-gnosis-server-live-pending-battery.md` §3.5** — the new row **`R-L4`** — **a SUPERVISOR same-unit edit of a file this pass does NOT own (REMAND-1, 2026-09-22: the pre-remand row promised the row but, unlike this table's other out-of-file rows — §5.8/§9.1/V-8 — it did not mark who lands it, so the red set had no single authority).** Its home in the battery is §3.5 (`:102-…`), whose heading currently names `R-L1`/`R-L2`/`R-L3` and **carries no `R-L4`** — verified by a read of the file this pass; **this pass edits neither the battery nor its §8 park note** | — | **the post-mutation honesty criterion (U6's live home):** boot the bin with a **controlled provider** answering `/api/tags` (so the boot is `Reachable` + `READY` and builds an index — an **empty** index over the boot's empty store, which is the fresh case) ⇒ (i) `GET /engine/status` reports `state:"Ready"`, `vector:true`; (ii) then **mutate the corpus through the bin's own CRUD routes** (`POST /wikis`, `POST /documents`, or `POST /documents/:id/update`) ⇒ (iii) `GET /engine/status` reports **`vector:false`** (the honest flip; **no process restart**) and (iv) a `mode=vector` `POST /rag/query` is **503 `vector_index_unavailable`** (FS-14 — the store is **READY**, so the pre-READY gate does **not** fire) — **never** 200-with-missing-hits; **FAIL:** `vector` stays `true` after the mutation, or the query returns 200, or the status/query disagree | this criterion is **executable against the running bin alone** (unlike `R-L3` (iii)/(v), it needs **no** boot-time corpus: it mutates **after** the boot through the bin's own CRUD surface) — the U6 flip is exactly what makes it observable. The row is a **live** obligation and does **not** wait on A1 or the shell (the `R-L1`/`R-L2`/`R-L3` precedent); it is **not** an M1–M20 parity row and does not disturb the battery's revisit condition. **The supervisor's same-unit edit is TWO items, and §U1's U6 bullet is their single authority:** **(1)** add the `R-L4` row (above) to §3.5; **(2)** annotate §8's **item 4** (`:395-405`, the FS-14 READY-index-free mapping's park) to record that its **un-park trigger** — the parked clause's own words, *"a future boot path that can be READY without an index (… or a rebuild/durability vehicle)"* — is met by the **third route** U6 supplies (a **stale** index on a READY store, no rebuild vehicle needed), so the park's **reason** is superseded while **`R-L3` (iii)/(v) stay PARKED unchanged**. **The battery's row index note (§3.5's "executable against the running bin alone") already covers a row of this shape** — no scope rule changes |
| `src/store/mod.rs:4210-4215` — the `vector` flag's derivation-site comment (the U5 reconciliation, landed) | the comment states the post-U5 reason (`Reachable` builds; `Absent`/`Unreachable`/failed build leave the snapshot index-free) | **the comment gains the freshness term** — the flag is `snapshot().vectors.is_some() && snapshot.epoch == self.epoch()`, and a post-boot mutation makes it `false` — **in the same unit as U6's code** | it is a **comment-only** IMPLEMENTER obligation (no value or behaviour moves) but it becomes stale the moment the predicate changes; the same-unit rule that binds §5.8's `vector` sentence, §9.5.5's `P-IM-14` marker and `tests/wire_conformance.rs`'s fixture comment binds this site too |
| `src/lib.rs:50-67` (the `boot_wiring` contract comment block) and `:79-89` (its doc comment, incl. `vector == snapshot.vectors.is_some()` at `:88`), **plus the new seam's own docs** | the comment sites state the pre-U6 `vector` predicate as if it were the boot's derivation | **reconciled in the OPPOSITE direction from the rest of the unit, deliberately (REMAND-1, 2026-09-22 — this row is NEW; the pre-remand table named no `src/lib.rs` comment site):** `boot_wiring`'s returned flags are **kept** at `snapshot.vectors.is_some()` (freshness rule 3), so those comments must **keep** the pre-U6 reading and gain the one clarifying sentence — the returned flags are the **wired-capability assertion surface, not the derivation**, which is `get_engine_status`'s (`src/store/mod.rs:4216`, whose own comment moves with U6). The **new** seam (`PROVIDER_REQUEST_TIMEOUT` / `provider_client`) gets its own doc comment naming the bin's single construction site and the "not configurable in U6 — the bin's configuration reads stay three" pin | this row is what makes the scoping of `derived == flags` **discoverable where the seam lives**: rewording `boot_wiring`'s comments to the epoch-aware predicate instead would make them **false of the code they document** (`:105`), and the third equality's scope would then be unstated at the one site a TestWriter reads first. **No value, literal or signature moves here** — comment-only, plus the seam's docs |
| `src/bin/gnosis_server.rs:416-419` — the boot's composed snapshot **and** `:403-413`'s comment block that introduces the build | four lines: `let snapshot = DerivedIndexes { vectors: built.ok().flatten(), ..boot_snapshot };` (the `built.ok().flatten()` discarding the `Err`) | **the fifth term `epoch: store.epoch()`**, plus (a) the **stderr diagnostic before the composition** (`P-IM-19`) and (b) the comment reconciled | §9.5.5 pinned the comment's content; U6 changes the composition's terms and the `Err`'s fate, so the block and its comment move **in this unit** (the golden-literal/comment discipline, §5.8/F2 §12) |
| `src/bin/gnosis_server.rs:327` — `client: reqwest::Client::new()` | no timeout | **`client: provider_client(PROVIDER_REQUEST_TIMEOUT)`** — surface #6's lib-visible seam, with the pinned `30 s` constant, and the `from_env` comment updated; **the seam itself is new lib surface: `src/lib.rs` gains the `pub const PROVIDER_REQUEST_TIMEOUT` + `pub fn provider_client` and the crate-root re-export** (the `src/lib.rs:119-134` `pub use` precedent) | `P-TP-6`'s construction pin; the single-client fact (`:314`) is why one bound covers probe + embed. **REMAND-1, 2026-09-22: the pre-remand pin was a bare builder call with a `const` living inside the private `OllamaProvider` (`:311-330`) — unnamable from `tests/`, so the row had no observable and the seam is now the pinned remedy (surface #6; the bin-level alternative is rejected there). This is the unit's one IMPLEMENTER edit outside the boot's composition, its diagnostic and the derivation-site comment** |
| `src/bin/gnosis_server.rs:347` — the **provider transport-error mapping comment/site** (`.map_err(|_| StoreError::EmbeddingUnavailable)?` in `OllamaProvider::embed`; the same shape at `:351`) — **REMAND-2 NOTE 9, 2026-09-22 (this row is NEW: it was not in the edit list)** | the mapping is landed and **not reworded** by U6; what changes is that it becomes **contractual**: the timeout contract of surface #5 and `P-TP-6` hangs on **this** mapping (a timed-out request is a transport `Err`, so it renders as `EmbeddingUnavailable` ⇒ FS-13 `embedding_unavailable` ⇒ 503), and the bin's client construction at `:327` now carries the pinned `30 s` timeout | **(a) NO code change** — the mapping and its behaviour stay exactly as landed (**no new variant, no new code, no new branch**); **(b) a comment-only same-unit edit:** the site gains the one sentence that makes the pin legible — *a request timeout surfaces here as a transport `Err` and therefore renders as the existing `StoreError::EmbeddingUnavailable` ⇒ FS-13 ⇒ 503 `embedding_unavailable`; no timeout-specific variant or mapping exists* — because after U6 the timeout is a **pinned** producer of that line and the comment is the only place the mapping's role is stated. The **lib**-side equivalent (`PROVIDER_REQUEST_TIMEOUT`/`provider_client`'s doc comment) is the surface-#6 row above | the same golden/literal discipline as the other comment rows: the predicate/behaviour does not move, but the comment that explains **why** the timeout maps to FS-13 must exist where the mapping lives; leaving it unstated makes the pinned chain (timeout ⇒ transport `Err` ⇒ `EmbeddingUnavailable` ⇒ FS-13 ⇒ 503) discoverable only by inference. **No value, literal byte, assertion or message moves; the §11 map and the `StoreError` taxonomy stay at 21** |
| `docs/specs/engine-wire-contract.md` **§9.1's `vector` flag row** (`:1124`) + **§12's V-8.1/V-8.2 notes** (`:1417`, `:1438`) | state the post-U5 predicate (`snapshot().vectors.is_some()`, "U5 changes only *what the boot puts in the snapshot*") and V-8.1's scope | a **U6-labelled dated note** on each: the predicate now carries the freshness term, V-8.1's scope gains the epoch-aligned precondition, and **the values do not move** | §9.1 is the contract **home** of the flag semantics and §12 the golden home; U6's supersession must be recorded where the superseded sentence lives. **This pass does not edit that file** (it is not one of this pass's artifacts): the obligation is named here for the supervisor's same-unit landing, exactly as §9.5.5 recorded its two F2 notes |
| §5.8's **`U5's answer to this section's open question`** clause — the quoted sentence *"The `vector` predicate `snapshot().vectors.is_some()` is **unchanged** by U5"* sits at **`:1654-1655`**, inside that clause's REMAND-1 clarification (`:1648-1654`) — and §5.8's **`U6's amendment to this section's `vector` predicate`** clause (**`:1663-1678`**), plus §5.9's **`Interlock with U5`** bullet (**`:1761-1763`**). **REMAND-2 SHOULD-FIX 7 anchor correction, 2026-09-22: the quoted sentence is at `:1681-1682`, inside that clause's own body — the REMAND-1 spelling `:1654-1655` (and the `:1648-1654` clarification range that accompanied it) is WRONG: `:1654-1655` is §5.8's `src/bin/gnosis_server.rs:390-404` citation text, i.e. the same wrong-target class REMAND-1 was fixing, and it is superseded here by the clause name + the re-read number. §5.8's `U6's amendment to this section's `vector` predicate` clause is at **`:1690-1716`** (the REMAND-1 `:1663-1678` spelling is likewise superseded by text — and §5.9's heading is at **`:1718`**, so the earlier "§5.9 opens at `:1680`" note is superseded too), and §5.9's `Interlock with U5` bullet is at **`:1799-1801`** (the `:1761-1763` spelling is superseded by text — that range is §5.9's FS-13/`hyde` narrowing clause, not the bullet; the bullet's own U6 marker is the block immediately after it). REMAND-1 also recorded, and this pass confirms: the pre-remand cell cited §5.8 `:1608-1616` for the quoted sentence and §5.9 at `:1643-1645` (which is §5.8's **`Unit split (pinned)`** clause). All four §5.8/§5.9 anchors are now clause-name-first with the numbers re-read this pass; every one of the four line spellings is kept in place as its dated record** | §5.8: the quoted U5-time sentence; §5.9: the U5-time reachability reading of FS-14 | **annotated in place with dated U6 markers** (this pass does edit this file): the predicate is `is_some() ∧ epoch-aligned` **after U6**, §5.8's quoted sentence stands as the **U5-time record**, **§5.9's `Interlock with U5` bullet gains the U6 marker the reconciliation table promised** (it previously had **none** — REMAND-1 landed it: the FS-14 reachability set gains the **stale** instance beside the unbuilt/caller-built ones, the `R-L4` live home is named, and the sentence *"Until U5, `mode=vector` is unusable on a booted server"* stands as its pre-U5 record), and §5.8's marker distinguishes the flag's **value** from its **type** (the freeze is a *shape* freeze) | §5.8 is the governing clause `SUBSYSTEM-FLAG-CAPABILITY-SEMANTICS` is read through; leaving it stating the pre-U6 predicate would leave the file with two accounts of the flag — the drift class §5.8's own field-vs-literal note already records. §5.9 is the contract home of FS-13/14/15 reachability, so a U6 reachability addendum that lives **only** in §9.5.6 leaves the governing table stale |
| §6's U5-amended boot sequence (`:1709-1724`) and §9.5.3's cap bullet / §11's register notes & UPDATE sentence / §12 item 5 / §U1's status block | the U5-shaped sequence and the U5-only register accounting | **each gains a short dated U6 annotation** (this pass): the boot sequence's composed-snapshot step gains the **`epoch: store.epoch()`** term; §9.5.3's binary-total bullet gains "U6's layer is 300, total 1492"; §11's UPDATE sentence and the register notes name U6's six rows; §12 item 5 and §U1's status block record U6's authorization, its spec gate and its six rows | these are the file's own bookkeeping surfaces: every earlier unit kept them current in its own pass, and a stale "U5 is the newest unit" reading is exactly what the next reviewer would trip on |
| the **held U5 register rows whose predicates touch the flag** — `P-IM-14` (`strat:vector-flag-flip`) and `P-SM-7` (`strat:boot-lifecycle-vector`), plus their §9.5.4 coverage notes (each named by its row + `strat:` id — the note headings `` `P-IM-14 — strat:vector-flag-flip` `` / `` `P-SM-7 — strat:boot-lifecycle-vector` `` — **and cited that way deliberately: the `:2521`/`:2523`/`:2477`/`:2478` line spellings this table carried are superseded by text, REMAND-2 NOTE 8, 2026-09-22: `:2477`/`:2478` fall in the §9.5.3 execution-plan bullets, not at those two notes**) and `P-IM-7`/`P-IM-8`/`P-IM-9`/`P-SM-5`'s **status** (cited by id: the `:2176`/`:2177`/`:2178`/`:2179` spellings are the `P-IM-5` renumbering discussion of §9.5.3.1 and are likewise **superseded by text**) | the **`is_some()`-only** predicate (`flags.vector == snap.vectors.is_some()`, `derived.vector == store.snapshot().vectors.is_some()`), and `P-SM-5`'s mutation-interleaved control | **annotated in place with dated U6 markers and superseded-in-place predicates — no row is deleted, renumbered, re-tagged or re-capped**: (i) `P-IM-14`'s equality becomes `… && snap.epoch == store.epoch()` (its `Reachable` + `Some` scoping is **unchanged**, and V-8.1 is asserted only for the epoch-aligned instance); (ii) `P-SM-7`'s serving half is **unchanged** (a fresh index serves) while its FS-14 half gains the **stale** instance (§9.5.6's fail-state 3) beside the unbuilt one; (iii) `P-IM-7`'s `f.vector == s.snapshot().vectors.is_some()` **gains the epoch term** (its corpus is built by `swap_snapshot`, so its fixtures must align — `props_gnosis_server.rs`'s U3 fixtures are already aligned, the mutation control is the one exception in the table above); (iv) `P-IM-8`/`P-IM-9`/`P-SM-6` are **untouched** (no vector predicate); (v) `P-SM-5`'s claim stands and its probe-4 fixture is re-derived per the row above | these are the register rows a TestWriter re-running the U5 layer would find **red** under the new predicate; the amendment must be **recorded where the rows live** (the U5 rows' own §9.5.5 block, per §9.5.3.1's "no row deleted, no id reused" discipline), not left to be discovered as a regression |
| the blind set `tests/blind_u5_boot_vector_index_greens.rs` and `docs/specs/u5-boot-vector-index-greens.md` | 27 GREEN / 0 RED / 6 NOT-VERIFIED, all derived from the U5 predicate | **named obligations (a TestWriter's artifacts — this pass edits neither):** the U6 red set must state which of those 27 scenarios re-derive under the amended predicate (the rows named in this table's blind bullets are the candidate set), and the U5 blind-greens doc's verdict table reads **U5-relative** thereafter | the blind layer's verdicts are per-unit claims; leaving "27/27 GREEN" as the newest reading after a predicate change would misreport the suite's state |
| the live battery's **`R-L3`** row (`docs/specs/p2-gnosis-server-live-pending-battery.md:126`) and its **§8** executed-run record (the `:360+` section) | `(i)`/`(ii)`/`(iv)` **PASS live**; `(iii)`/`(v)` **PARKED** with reasons; `R-L2`'s amended cell re-verified live | **no criterion changes and no park lifts** — **except** the FS-14 **state-6 mapping's** park, which **partially lifts**: §8 item 4 parks it as *"structurally non-reachable through the bin post-U5"* because the only READY boot builds an index. **Under U6 the mapping becomes reachable again through the bin** — a READY boot whose store is **mutated post-boot** is exactly a READY store that must not serve its index (FS-14, `R-L4` (iv)) | the parked clause's own un-park trigger reads *"a future boot path that can be READY without an index (… or a rebuild/durability vehicle)"*; U6 supplies the third route (a **stale** index on a READY store) without any rebuild vehicle — so the park's **reason** is superseded and `R-L4` carries the live criterion. **`R-L3` (iii)/(v) stay PARKED unchanged** (their reasons — an empty boot corpus and no boot-time seeding — are untouched by U6) |

**The interaction statements (pinned).**

- **With U4 (HELD).** U6 **MUST NOT** touch: the **change cursor** (any accessor, the `epoch()`/`journal_len()`
  **semantics** as the cursor's basis, `JournalEntry`'s shape), **any route** (the 14-row table and
  `route_bijection()` are U4's amendment surface, §5.2's amendment rule), the **paged reads**, `GET /changes`,
  the **journal** (U6 appends **no** entry and does **not** move `epoch()`), or any **persistence/durability**
  surface. U6 **reads** `epoch()` (the existing accessor, `src/store/mod.rs:1814-1818`) and **adds no**
  accessor to the cursor or the journal. **Interaction in the other direction:** U6's staleness term uses the
  cursor's own clock, so if U4 ever changes what `epoch()` counts, U6's predicate must be re-read in U4's
  unit — recorded as a note, not as an amendment.
- **With the durability direction (unauthorized).** The durability design unit is **not authorized**
  (`ENGINE-DURABLE-CORPUS-DIRECTION` is **DIRECTION ONLY / NOT ACTIVE**; `docs/specs/gnosis-grq-inbound-review.md`
  §14 Q2 = "(B) NOT before …") and U4 is **HELD** for it. **No rebuild vehicle may be assumed beyond what U6
  pins, and U6 pins none:** the index's **only** installation vehicle stays the boot-time `swap_snapshot`
  (`src/store/mod.rs:1855-1860`; the boot's call at `src/bin/gnosis_server.rs:433` and the U5 test/eval call
  sites), U6 adds **no** rebuild path, **no** epoch-driven rebuild loop, **no** writer actor, **no**
  persistence, and **no** background task — so a stale index is **not** repaired in-process. The honest
  consequence is stated in §9.5.6's cost clause: `mode=vector` is unavailable until a restart with a
  reachable provider. **The un-park trigger this implies:** if the durability unit (or any future boot path)
  carries a corpus **before** the bind, then a boot's snapshot is composed over a non-empty corpus and the
  freshness term is exercised on the **live** boot path — the same trigger the battery's `R-L3` (v) park
  names.
- **With the consumer (the Astrographer shell — cross-repo, unverifiable here).** The engine-side statement,
  which is all this spec can pin: `mode=vector` **means** "serve from an index that covers the store's
  current state"; the honest signal a consumer must read is **`subsystems.vector`** on `GET /engine/status`
  (**always 200**), and the wire answer for `mode=vector` is **503 `vector_index_unavailable`** whenever that
  flag is `false` **because the index is stale or unbuilt** (**not** a 200 with missing hits). A consumer that
  wants vector results after a write must (a) re-check the flag, and (b) restart a reachable-provider boot,
  because U6 adds no rebuild. **No consumer-side claim is made beyond this** (the shell's behaviour is
  **cross-repo and unverifiable from this repo** — the shell's status pane and cache are `SHELL-2`'s concern,
  `docs/specs/gnosis-grq-inbound-review.md` §14, and the battery's A1-dependent rows stay parked).

**The reconciliation of the superseded clauses — annotated in place, never rewritten (dated markers).**
**This pass reconciles the following in this file; every superseded sentence is KEPT and marked, per §9.5.3.1's
discipline.**

| # | clause superseded | its pre-U6 text (kept) | the U6 reading (this pass's dated marker) |
| --- | --- | --- | --- |
| 1 | §9.5.5's **residual (ii)** (*"no rebuild after boot"*, the residual list in §9.5.5's register notes) | *"the index reflects the corpus **at boot**, so documents created later are not in it until a rebuild vehicle exists … today the boot snapshot is the only writer"* | **SUPERSEDED IN PART (U6, 2026-09-22):** the *observation* stands and is now the unit's premise — **the boot snapshot remains the only writer and U6 adds no rebuild vehicle** — but the *disposition* ("not in it until a rebuild") is superseded: a later-created document no longer leaves a **silent** 200 with missing hits; the corpus's movement makes the flag honest (`vector:false`) and the read **loud** (FS-14 ⇒ 503). **Residual (ii) is therefore closed as an honesty gap and re-stated as an availability gap** (the index is still not rebuilt; the consumer loses `mode=vector` rather than getting a wrong answer) |
| 2 | §9.5.5's **residual (i)** (*"unbudgeted boot latency … the provider's own request timeout is the only bound"*) | the residual list's item (i), and its cost-table row *"no timeout, node cap or deadline is pinned"* | **SUPERSEDED (U6, 2026-09-22):** the claimed bound now **exists** — the provider client carries a request timeout (pinned `30 s`, `P-TP-6`) covering both the probe and every `embed`. **The residual is NARROWED, NOT CLOSED:** there is still **no total build budget and no node cap**, so the worst-case pre-bind delay is **`n × 30 s`** for `n` embeddable nodes (plus the `n`-text corpus materialisation the `U5-ADV-2` row names). Closing it needs a node cap / total-budget knob, which is a new configuration surface and therefore a **new register obligation** (§9.5.5's question-5 clause) — **recorded as an open question below**, not taken |
| 3 | §9.5.5's `P-IM-14` predicate reading (`flags.vector == snap.vectors.is_some()`, `derived.vector == store.snapshot().vectors.is_some()`, `:2521`) and its §9.5.4 note (`:2477`) | the `is_some()`-only equality, `Reachable` + `Some`-scoped, with V-8.1 as its `Some` instance | **SUPERSEDED IN PLACE (U6, 2026-09-22):** the equality gains the **epoch term** (`&& snap.epoch == store.epoch()`); the `Reachable` + `Some` scoping, the V-8.1 assignment (asserted only for the epoch-aligned instance) and every other clause of the row stand. **No id, kind, tag, `Strategy-id` or cap moves; the row is annotated, not rewritten** (§9.5.5's REMAND discipline). **LANDED, not delegated (REMAND-2 MUST-FIX 2, 2026-09-22): the dated U6 marker now sits AT the row** — the in-place marker block at the foot of §9.5.4's note block (its items (1)/(2), headed by the `P-IM-14 — strat:vector-flag-flip` note), which restates the superseded predicate beside the U6 reading. **This row's `:2521`/`:2477` line spellings are superseded by text** (the row id and the note heading are the anchors; the numbers shift) |
| 4 | §9.5.2's `P-IM-7` flag predicate (`f.vector == s.snapshot().vectors.is_some()`, `:2176`) | the same `is_some()`-only equality, under the U3 capability semantics | **SUPERSEDED IN PLACE (U6, 2026-09-22):** the `vector` half gains the epoch term (the row's other predicates and its write-independence half are untouched). U3's five rows stay **held** under the amended predicate for the corpus their fixtures actually build (verified: the U3 `props` fixtures run on a fresh store, `epoch 0`) — **with the one named exception** of `P-SM-5`'s mutation-interleaved control, re-derived in the same-unit table above. **LANDED, not delegated (REMAND-2 MUST-FIX 2, 2026-09-22): the dated U6 marker now sits AT the row** — the in-place marker block at the foot of §9.5.4's note block (its items (2)/(4), headed by the `P-IM-7 — strat:flag-truth-capability` and `P-SM-7 — strat:boot-lifecycle-vector` notes; the `P-IM-7` cell is §9.5.2's, the `P-SM-7` cell §9.5.5's), restating each superseded predicate beside its U6 reading. **The `:2176`/`:2478` line spellings are superseded by text** (the row id and the note heading are the anchors) |
| 5 | §5.8's *"The `vector` predicate `snapshot().vectors.is_some()` is **unchanged** by U5"* (**`:1681-1682`** — REMAND-2 SHOULD-FIX 7, 2026-09-22: the REMAND-1 `:1654-1655` spelling was **still wrong** — it is the clause's `src/bin/gnosis_server.rs:390-404` citation, not the quoted sentence — and the pre-remand `:1608` spelling pointed at the same wrong class; the quoted sentence is at **`:1681-1682`**, re-read this pass, and the clause name `U5's answer to this section's open question` is the anchor with the number as the convenience) and §9.5.5's `P-IM-14`-echoing adjudication-note wording (the **U5 adjudication notes block** of §9.5.5, cited clause-name-first; the `:2904-2915` spelling this table carried is **superseded by text** — that range is §9.5.5's corpus-seeding recipe, not the adjudication notes, REMAND-2 NOTE 8) | the U5-time reading of the predicate and of `mode=vector`'s reachability | **U5-TIME RECORD (U6, 2026-09-22):** for **U5** the sentence is true and stands as written; **after U6** the predicate is `is_some() ∧ epoch-aligned`. §5.8's `U6's amendment…` clause is at **`:1690-1716`** and §5.9's `Interlock with U5` bullet at **`:1799-1801`** (REMAND-2 NOTE 8: the `:1663-1678` and `:1761-1763` spellings are **superseded by text** — the latter range is §5.9's FS-13/`hyde` narrowing clause) gains its **U6 marker** (the stale instance beside the unbuilt/caller-built ones, the `R-L4` live home, and the `hybrid` degradation unchanged; the marker is the block immediately after the bullet). **Neither clause is rewritten; both carry a dated marker** |
| 6 | §9.5.6's **own** first reading (this section's `P-IM-16` predicate) as a store-wide epoch semantics | — | **a note this pass adds so it cannot be misread:** U6 gives the **freshness comparison** for the **vector index** only. It does **not** redefine `DerivedIndexes.epoch` store-wide (its documented role stays the caller's label on the snapshot it composed, `src/store/mod.rs:823-826`), it does **not** make `swap_snapshot` stamp the snapshot, and it does **not** apply the freshness term to `LexicalIndex` or to any other leg. A future unit that wants epoch-stamping inside `swap_snapshot` is a **store-API change** with its own register row (§9.5.6's open questions) |
| 7 | the U5 `P-IM-14` **derivation-equality invariant** (`derived == flags`, and with it `boot_wiring`'s returned flag vector as "the derivation") read **store-wide** | the pre-U6 reading: `boot_wiring`'s `vector` element and the store's own derived `vector` are asserted **equal for every snapshot** | **SCOPED, NOT SUPERSEDED (U6, 2026-09-22; REMAND-1 recorded it here because the invariant had no home the TestWriter could find — MUST-FIX 5):** `boot_wiring` is **pinned unchanged** and keeps the **pre-U6** `is_some()` surface (`src/lib.rs:90-110`, `:105`; freshness rule 3), so the equality is asserted **only for epoch-aligned snapshots**; on an epoch-mismatched snapshot the **pinned** relation is `derived.vector == false` **with** `flags.vector == true`, asserted two-sidedly rather than dropped. The **remedy chosen** for the affected U5 corpus (`tests/props_gnosis_server.rs:7322-7361`) is **re-derivation with the variant retained** (probes re-derived per site; the corpus is **not** restricted to aligned epochs, because the mismatched variant is the row's only falsifiable stale instance) — the same-unit table's `P-IM-14` row carries the per-probe obligation |

**Open questions for the supervisor and the user (design forks — the safest option is pinned; these are NOT
silent choices).**

1. **Provider identity is not part of the freshness term.** Wiring a **different** provider leaves the index
   `fresh` (the epoch did not move), so an index built with provider A keeps `vector:true` while queries embed
   with B — a *model-drift* class of staleness (the same shape as the OPEN `P-8`'s capability-vs-liveness
   family) that U6 **does not** absorb. **Options:** (a) **pinned default** — no provider term (minimal,
   matches the `U5-ADV-1` fix shape, keeps `P-IM-7`'s predicate touched exactly once); (b) add a
   provider-identity/epoch term (a per-provider keyed index, or a provider generation counter) — a **new store
   semantics** and a second move of the same predicate. **Supervisor action requested:** either accept (a) as
   the unit's residual (a `docs/defects.md` row is the supervisor's to file) or open a follow-up unit for (b).
2. **The 30 s provider timeout is not configurable.** **Options:** (a) **pinned default** — a fixed constant
   (`P-TP-6`), keeping the bin's configuration reads at **three**; (b) a new env var (e.g.
   `GNOSIS_SERVER_OLLAMA_TIMEOUT_MS`) — which is a **new configuration surface** and therefore triggers
   §9.5.5's question-5 register obligation (*pin the name, its default, its no-set behaviour, **and** an
   assertion that the documented set is closed*). **Supervisor action requested:** confirm (a) (the pinned
   default) or authorize the knob with its register obligation.
3. **No total build budget / node cap** (the narrowed `U5-ADV-2` residual above): the per-request bound exists,
   the `n × 30 s` worst case does not. **Options:** (a) **pinned default** — restate the residual honestly
   (this pass does); (b) a build budget (a deadline or a node cap) — again a **new configuration surface** with
   the same register obligation, and a design question (what happens to the index when the budget is hit:
   partial ⇒ forbidden by `P-IM-13`'s atomicity, so it must be all-or-`Err`). **Supervisor action requested:**
   accept (a) for U6 or schedule (b) as its own unit.
4. **The staleness over-approximation** (non-corpus mutations advance `epoch()` and therefore mark the index
   stale — §9.5.6's cost clause). **Options:** (a) **pinned default** — accept the conservative reading (a
   *loud* error, never a wrong answer); (b) a corpus-scoped revision (a new store counter incremented only by
   corpus-affecting mutations) — a **store-epoch semantics change** touching `append_journal` and every
   `epoch()` consumer including U4's cursor, which is why it is not in U6. **Supervisor action requested:**
   confirm (a).
5. **`U5-ADV-3`'s disposition** (§9.5.6's out-of-scope table): the `Degraded` + `embedding:true` +
   blame-`embedding` `last_error` contradiction stays **OPEN** and is **not** absorbed here. **Supervisor
   action requested:** file the tracker disposition (accept the generic sentence, or open the unit that moves
   the §11/V-8.2 golden).

**U6's status (this section's own record).** **AUTHORIZED 2026-09-22** (the user's go-ahead, quoted in the
Authority clause) **and this section is its SPEC GATE artifact**: the contract (the honesty predicate, its two
consumers, the valid/fail states, the diagnostics clause, the bound clause), the typed register (6 rows), the
execution plan, the same-unit obligation table and the reconciliation of the superseded clauses. **The
reviewer loop on this section is OPEN at authoring time** — the unit becomes delegable to a TestWriter only
once that loop returns **EMPTY** (the AGENTS.md §2 spec gate); **its code is OWED** and **no row of this
register is HELD or BROKEN** (nothing here is a green claim). **ONE-PASS REMAND round 1 (2026-09-22; docs-only,
ADDITIVE — every superseded sentence is kept and marked, nothing deleted, no id/kind/tag/strategy-id/cap
moved).** The section reviewer's first pass returned **17 findings — 9 MUST-FIX + 2 SHOULD-FIX + 6 NOTE — and
all 17 are dispositioned in this pass** (fixed, or explicitly adjudicated with the reason recorded here and at
the site — the **per-finding disposition table is at the foot of this block**, so the next round can verify
each finding without re-deriving it). **The loop therefore stays OPEN:
the unit is still not delegable to a TestWriter until the next round returns EMPTY** (the AGENTS.md §2 spec
gate). **What the remand changed, by class:** (a) **per-site pinning of the predicate's consumers** — the
freshness-condition block gained a **per-site consultation clause** (surface #2's contract cell rewritten;
`vector_search`'s and the `hybrid` leg's stale outcomes are now pinned, and a second hand-rolled `is_some()`
check is forbidden); (b) **the six tag constants are pinned EXACTLY** (the landed 8-byte `U5PIM10`-form
spelling; the pre-remand "ASCII of its own name" wording was not implementable); (c) **two whole test files
entered the same-unit table** (`tests/retrieval_stack_integration.rs`, `tests/props_retrieval.rs` — both
hard-code `epoch: 1`) and **four** under-stated claims were corrected (the `rag_query_integration.rs` fixture's real
red sites, the split of the concurrent-rebuild row from the immutability row, the `u5_boot_vector_index_conformance.rs`
state-1/2 sites that do **not** move, and the `seed_store` epoch arithmetic: `1` wiki + `2` journal appends per
document ⇒ `≥ 2`, `≥ 3` for one document); (d) **`derived == flags` is SCOPED to epoch-aligned snapshots** with
`boot_wiring`'s returned flags **pinned at the pre-U6 `is_some()` surface** (freshness rule 3 + reconciliation
row 7), and the `P-IM-14` corpus remedy is the **re-derivation with the mismatched variant retained**; (e)
**`P-TP-6` gained a real observable** — a **second lib-visible seam** (surface #6: `PROVIDER_REQUEST_TIMEOUT` +
`provider_client`), the bin-level alternative being rejected because a real `30 s` timeout is what that row's
excluded list forbids; (f) **the §5.9 `Interlock with U5` U6 marker was LANDED** (it had been promised but never
written) and **two wrong `p2`-internal anchors were corrected** (the §5.8 quoted sentence is at `:1654-1655`,
not `:1608`; the `:1643-1645` citation was §5.8's `Unit split (pinned)` clause, not §5.9 — §5.9's `Interlock
with U5` bullet is at **`:1761-1763`**, and **the same stale `:1643-1645` spelling appears in this file's
U5-time records** (U5's REMAND notes in §9.5.5, §9.5's header note on the §5.x anchors, and §12's items):
per this file's standing rule those citations resolve by **clause name** — §5.9's `Interlock with U5` bullet —
and this marker is the authoritative correction, so no U5-time clause is rewritten to chase a number); (g) the battery's
`R-L4` row is now marked as a **supervisor** same-unit edit with its second item (§8 item 4's park annotation),
so the red set has one authority; (h) the blind U3 populated-index scenario (`:479-484`, fixture `:157`) is
named as the **one blind site that is actually RED**, and the blind U5 "for EVERY pair" row was re-derived the
other way (that loop is still green — `flags` is `boot_wiring`'s pinned surface and its pairs are epoch-aligned);
(i) **NOTE 12/NOTE 17:** the `EngineSubsystems` freeze is now stated as a **shape** freeze, not a value freeze,
inside §5.8's U6 marker, §13's U6 bullet and this section.

**Per-finding disposition (the round-1 review, 17 findings — recorded here so the next round can verify each one without re-deriving it).** *MF = MUST-FIX, SF = SHOULD-FIX, N = NOTE.*

| # | class | disposition | where it landed (clause name first) |
| --- | --- | --- | --- |
| 1 | MF | **fixed** | same-unit table: **two NEW rows** — `tests/retrieval_stack_integration.rs`'s `seed_vectors` (sites `:329`/`:357`/`:435`/`:579`/`:614`/`:640`/`:669`) and `tests/props_retrieval.rs`'s `seed_vectors` (`:694`/`:1005`/`:1119`/`:1242`/`:1336`, reads `:701`/`:1050`/`:1342`, the `:716` panic) |
| 2 | MF | **fixed** (the "already aligned" reading kept, the contradictory row corrected) — **SUPERSEDED IN PART by round 2's MUST-FIX 1 (kept as the record): the "they are **aligned**" reading held for `:459-462` only; `:517-520` and `:900-903` move (see the round-2 table below)** | the same-unit table's governing rule + the `u5_boot_vector_index_conformance.rs:459-462`/`:517-520`/`:900-903` row (they are **aligned**: `wired_status` builds its own `Store::new()`, `:386`); state 7 (`:850-853`) is the moving one |
| 3 | MF | **fixed** | the `rag_query_integration.rs:272-282` row now names the hybrid non-degeneracy assertion (`:790-795`) and the FS-13 site (`:649-682`) |
| 4 | MF | **fixed** (row split in two) | the same-unit table: `concurrent_rag_query_reads_race_a_concurrent_index_rebuild` (`:1757-1847`, readers `:1799-1810`, `.expect` `:1810`) needs epoch alignment; `derived_snapshot_swap_is_atomic_under_concurrent_readers` (`:1853-1927`) stays **unchanged** |
| 5 | MF | **fixed** (scope + the remedy chosen: re-derivation with the mismatched variant retained) | freshness **rule 3** + reconciliation **row 7** + the `P-IM-14` same-unit row (per-probe: 1 unchanged, 2 epoch-aware, 3 scoped + two-sided stale assertion, 4 `u5_expected_flags` re-derived, plus the V-8.1 block `:7435-7459`) |
| 6 | MF | **fixed** | the execution plan's **Seed pin** bullet: the six constants pinned exactly, with the landed 8-byte `U5PIM10`-form rule and the rejected bare-7-byte spelling |
| 7 | MF | **fixed** | the **per-site consultation clause** under the surface table (+ surface #2 rewritten, + P-IM-17's invariant/observable, + both §9.5.4 notes) |
| 8 | MF | **fixed** | **surface #6** (the lib-visible seam) + `P-TP-6`'s invariant/observable + the layer-classification bullet + §U1/§12/§13 |
| 9 | MF | **fixed** | the §5.8/§5.9 same-unit row's anchor correction + reconciliation row 5 + the **§5.9 `Interlock with U5` U6 marker, landed** |
| 10 | SF | **fixed** | the governing rule + the `u5_boot_vector_index_conformance.rs` row: `seed_store` = **epoch ≥ 2** (`1` wiki + `2` journal appends per document, `≥ 3` for one document) |
| 11 | SF | **fixed** | the battery `R-L4` row is marked a **supervisor** same-unit edit with both of its items, and §U1's U6 bullet is named as the red set's authority |
| 12 | N | **fixed** | §5.8's U6 marker + §13's U6 bullet + this status block: the freeze is a **shape** freeze, not a value freeze |
| 13 | N | **fixed** | `P-IM-16`'s observable cell + its §9.5.4 note: **one bound clone** before the status read |
| 14 | N | **fixed** | the same-unit table's blind row: `tests/blind_u3_status_honesty_greens.rs:479-484` with the `epoch: 1` fixture at `:157` (+ the pre-U6 predicate sentence at `:198`) is named as the **red** blind site |
| 15 | N | **fixed** | a **NEW** same-unit row for `src/lib.rs:50-67`/`:79-89` (comments kept at the pre-U6 predicate, with the "assertion surface, not the derivation" clarification) + the seam's docs |
| 16 | N | **fixed** | `P-IM-17`'s invariant is scoped to **`rag_query`**, with the `rag_stream`/wire half's outcome pinned and its live home cited as the battery's `R-L4` |
| 17 | N | **fixed** | same site as 12 (the `EngineSubsystems`/freeze wording is reconciled with the derivation change in §5.8, §13 and here) |

**⚠️ REMAND round 2 (2026-09-22; docs-only, ADDITIVE — every superseded sentence is kept and marked, nothing deleted, no id/kind/tag/strategy-id/cap/constant/arithmetic moved).** The section reviewer's **second** pass returned **11 findings — 4 MUST-FIX + 3 SHOULD-FIX + 4 NOTE — and all 11 are dispositioned in this pass**, with the per-finding row at the foot of this block. **The loop therefore stays OPEN: the unit is still not delegable to a TestWriter until the next round returns EMPTY** (AGENTS.md §2's spec gate). **What round 2 changed, by class:** (a) **the same-unit table's "already aligned" reading was false for two of the three U5 conformance snapshots** and all three are now re-derived **by rule, per site, naming each consumer and its store's epoch** (state 2 `:517-520` and contract table (1) `:900-903` **do** move — split into `epoch: store.epoch()` for the corpus-seeded consumer and epoch `0` for the fresh-store helper — while state 1 `:459-462` stays **aligned with no edit**, its only consumers being a store-less `boot_wiring` call and the fresh-store helper); (b) **the held `P-IM-14`/`P-IM-7`/`P-SM-7` register cells and their four §9.5.4 notes now carry LANDED dated U6 markers in this file** (in-place marker blocks at the foot of §9.5.4's note block, each naming its superseded predicate beside the U6 reading — **not** recorded as a supervisor obligation, since §9.5.3.1 permits the annotation in place and the red set needs one authority); (c) **`tests/retrieval_stack_integration.rs`'s three INLINE index-bearing fixtures entered the same-unit table** with their consumers, and the **per-file enumeration rule** that produces the count is now stated so it is re-derivable; (d) the `P-IM-14` per-probe re-derivation **gained probe (5): the V-8.1 block `:7437-7460`**, re-derived under the **same epoch-aligned filter as probe (3)**, with the V-8.1 *producer* row's **three preconditions** reconciled; (e) **`P-IM-19`'s home is PINNED to the spawned bin** (its lib-boundary alternative demoted to a non-pinning note) so the layer classification and the observable name one home; (f) **surface #1's lib-visible predicate is now asserted by a row** — `P-IM-16`'s observable carries the `gnosis::vector_index_is_fresh` assertion half, and the lib-level count stays **five**; (g) the **§5.8 quoted-sentence anchor was corrected a second time** (REMAND-1's `:1654-1655` was still wrong; the sentence is at `:1671-1672`, and the §5.9 `Interlock with U5` bullet is at `:1789-1791`, not `:1761-1763`); (h) the **per-row line anchors were replaced by ids/clause names** (`P-IM-16`…`P-IM-19`, `P-SM-7`, `P-IM-7`, plus the renumber-table and budget-arithmetic spellings); (i) the **bin's transport-error mapping comment** (`src/bin/gnosis_server.rs:347`) entered the same-unit table as a comment-only Implementer obligation; (j) the **new stale-index conformance case's home is PINNED** to `tests/u5_boot_vector_index_conformance.rs`; (k) **the cost clause's unstated premise is now an explicit assumption** — every corpus-changing mutation advances `epoch()` (verified: the six `guard.docs.insert/remove` sites are each paired with an `append_journal` call) — with the defect class and the owed tracker row stated for the case where it stops holding.

**Per-finding disposition (the round-2 review, 11 findings — recorded here so the next round can verify each one without re-deriving it).** *MF = MUST-FIX, SF = SHOULD-FIX, N = NOTE.*

| # | class | disposition | where it landed (clause name first) |
| --- | --- | --- | --- |
| 1 | MF | **fixed — consumer-anchored; TWO of the three sites move, one is aligned** | §9.5.6's **governing rule** clause rewritten (per-site rule: a label is aligned against **the store that consumes it**; a fixture feeding two stores must be **split**) + the same-unit table's `u5_boot_vector_index_conformance.rs:459-462`/`:517-520`/`:900-903` row, re-derived site by site with each consumer and its store epoch — **(a) `:459-462` ALIGNED, NO EDIT** (consumers: `boot_wiring` `:463`, which reads no store; fresh-store `wired_status` `:470` — aligning it to the corpus store's epoch `3` would make the helper's read stale and turn a passing assertion red); **(b) `:517-520` MOVED** (store epoch `2`; the same label swapped into the corpus-seeded store at `:536` ⇒ `mode=Vector` `rag_query` `:541-551` FS-14, `.expect` at `:551` panics); **(c) `:900-903` MOVED** (store epoch `3`; the same label on the state-2 query path ⇒ FS-14) — the pinned edit being a **split** label (epoch-`0` for the helper, `store.epoch()` for the corpus-seeded store) + §U1's same-unit list |
| 2 | MF | **fixed — LANDED, not delegated** (the markers are in this file) | in-place **dated U6 marker blocks at the foot of §9.5.4's note block**: the `P-IM-7 — strat:flag-truth-capability` marker (its §9.5.2 cell predicate + its §9.5.4 note) and the `P-IM-14`/`P-SM-7` marker (both §9.5.5 cells + both §9.5.4 notes), each naming the superseded predicate beside the U6 reading; the same-unit table's row and reconciliation rows **3** and **4** now state **LANDED** explicitly; the `:2176`/`:2477`/`:2478`/`:2521`/`:2523` spellings are marked superseded by text |
| 3 | MF | **fixed** | the same-unit table: a **NEW row** for `tests/retrieval_stack_integration.rs`'s **three inline** snapshots (`:407-411` ⇒ `store.vector_search(...).unwrap()` `:417-420`; `:516-520` ⇒ `rag_query(mode=Vector).unwrap()` `:540-543`; `:883-887` ⇒ `rag_query(...).unwrap()` `:910-922` **and** `:931-943` **and** the hit-presence assertion `:923-927`), plus the **per-file enumeration rule** in the file's `seed_vectors` row (every `swap_snapshot(DerivedIndexes{…})` site → its vector-leg/status consumers), plus §U1's red-set list |
| 4 | MF | **fixed** | the same-unit table: a **NEW row** for `tests/props_gnosis_server.rs:7437-7460` (the V-8.1 block: `index_bearing` `:7439`, `v81.vector = index_bearing` `:7450`, the assertion `:7454-7459`, the `!index_bearing` negative `:7475-7481`) re-derived under the **epoch-aligned filter** exactly as probe (3), with the `"empty-index-with-epoch"` variant (`epoch: 7`) getting the two-sided stale assertion; the **V-8.1 producer row** now names **three** preconditions with the third required there too; a sweep confirms no other V-8.1-shaped assertion in the file |
| 5 | SF | **fixed** | `P-IM-19`'s **observable cell** (home **PINNED to the spawned bin's stderr**; the lib-boundary variant demoted to an explicitly **non-pinning** note) + its §9.5.4 note + the execution plan's **layer classification** bullet, all three now naming one home |
| 6 | SF | **fixed** (naming chosen over dropping) | the same-unit table: a **NEW row** for **surface #1** naming `P-IM-16`'s observable cell + its §9.5.4 note as the lib-visible predicate's assertion surface (`vector_index_is_fresh` nameable at the crate root, returning exactly what the row's equivalence asserts) + the **layer classification** bullet (`P-IM-16`'s observable includes surface #1; count stays five); the "drop the lib-visibility requirement" alternative is **rejected with the reason** |
| 7 | SF | **fixed** | the §5.8/§5.9 same-unit row (anchor corrected to **`:1681-1682`**, clause-name-first, with REMAND-1's `:1654-1655` marked superseded) + reconciliation **row 5** (quoted sentence `:1681-1682`; the U6 clause `:1690-1716`; §5.9 bullet `:1799-1801`; the `:2904-2915` adjudication-note spelling superseded) |
| 8 | N | **fixed** | the same-unit table's held-rows row **and** reconciliation rows 3/4: `P-IM-14`, `P-SM-7`, `P-IM-7`, `P-IM-8`, `P-IM-9`, `P-SM-5` and the two `P-*` §9.5.4 notes are now cited **by id/`strat:` id/note heading**, with the `:2176`-class line spellings marked **superseded by text** |
| 9 | N | **fixed** | a **NEW same-unit row** for `src/bin/gnosis_server.rs:347` (the provider transport-error mapping the timeout contract hangs on): **no code change**, a comment-only same-unit edit stating the timeout ⇒ transport `Err` ⇒ `EmbeddingUnavailable` ⇒ FS-13 ⇒ 503 chain; §U1's `src/` list names it |
| 10 | N | **fixed** | the same-unit table's **new** stale-index conformance row: the home is **PINNED** to `tests/u5_boot_vector_index_conformance.rs` (the "or the U6 conformance file" alternative removed; a TestWriter moving it must record the move at that row; **no `file:line` claim depends on it**) + §U1's list |
| 11 | N | **fixed** | the **cost clause**: the unstated premise is pinned as an explicit **assumption** that **every corpus-changing mutation advances `epoch()`** (verified: the six `guard.docs.insert/remove` sites are each paired with an `append_journal` call), with the **epoch-preserving corpus write** class named as a defect **of this predicate** owing its own `docs/defects.md` row + negative probe, and no U6 row asserting the pairing |

**One item is left to the supervisor's judgement and
is flagged as such:** the **second lib seam** (surface #6) is the pinned remedy for `P-TP-6`; if the supervisor
refuses a lib-visible item, the recorded fallback is to re-home that row bin-level at the cost of a real `30 s`
observation — the spec pins the seam and states that trade-off rather than leaving the row unobservable. **This
pass is docs-only:** it edits **only**
`docs/specs/p2-gnosis-server.md` (this section + the dated markers named in the same-unit table's "this file"
rows) and **no** `src/`, `tests/`, `Cargo.toml`, tracker file, canonical `docs/specs/gnosis.md` or other spec
file; it runs **no** cargo command and does **not** re-verify the pre-existing baseline (the trio figures in
this file's other status blocks are **carried as reported**, not re-measured). **Line-number disclosure for
the `p2`-internal anchors:** this pass inserts lines (this section, the §9.5.4 notes, and the dated markers in
§U1/§5.8/§6/§9.5.3/§10/§11/§12/§13), so every bare `p2`-internal `:NNNN` citation below the first insertion
shifts; the file's standing rule applies — **the clause name is the anchor and the number is the
convenience**. ***(REMAND-1, 2026-09-22: this remand pass inserts further lines in §9.5.6, §9.5.4, §5.8, §5.9,
§9.5.3's U6 note, §U1, §11 and §13, so every bare `p2`-internal `:NNNN` citation written before it shifts
again; the numbers cited **inside** this remand's markers are the ones **re-read this pass** (pre-insertion),
and the citations into other files (`src/…`, `tests/…`, the battery, the wire contract) remain line-valid
because no other file is edited.***)*** — and the citations into **other** files (`src/…`, `tests/…`, `docs/specs/engine-wire-contract.md`,
the battery) are **line-valid as written**, because **only this file is edited by this pass**.


## 10. Valid/happy + fail states per endpoint (TestWriter assertion guide)

| endpoint | valid/happy | fail-state |
| --- | --- | --- |
| `POST /rag/query` | request envelope → `RagResult` response envelope (payload = the **bare** `RagResult` body, §5.3) | transport **400** `invalid_json`/`invalid_envelope` (+ `unsupported_schema_version`/`unknown_id_format` §5.5; the **non-object-payload** row is **U2-time**, F13, and its code is **`invalid_envelope` alone** — F9, `P-IM-4`); `ValidationError`→**400** `validation_error` (unrecognized `mode`/`expand`/`compression` token or out-of-range option value, empty query — §5.3/§5.4); a **wrongly-typed** optional value ⇒ that option's default, **not** a 400 (§5.3's **per-key table**); `EngineUnavailable`→503, `EngineError`→502, `TraceUnavailable`→502, `VectorIndexUnavailable`→503 / `EmbeddingUnavailable`→503 (**explicit-leg only**, §5.9), other §11-mapped errors. ***(U6 note, 2026-09-22 — ADDITIVE: the `VectorIndexUnavailable`→503 cell above gains a SECOND instance, no new code and no new row: a `mode=vector` request on a **READY** store whose snapshot carries an index that is **STALE** (`snapshot().epoch != epoch()`, §9.5.6's `P-IM-16`/`P-IM-17` and its valid/fail state 3) is FS-14 ⇒ **503 `vector_index_unavailable`** — **never** a 200 with silently missing hits. The unbuilt (`vectors: None`) instance, the FS-13 instance (a **fresh** index with no provider), and the FS-8 instance (a non-READY store: the pre-READY gate precedes every leg check) are **unchanged**; `mode=hybrid` with a stale index still serves (its vector leg degrades to empty).***) |
| `GET /rag/stream` | SSE frames (`result`/`done`/`error`); a **pre-stream** `StoreError` is rendered as an **`error` frame under its §11-mapped status** (a not-READY engine ⇒ **503**); **every** response carries `Content-Type: text/event-stream` exactly (§5.4's F-2 pin); the SSE surface reads **exactly `query`/`topK`/`mode`** (§5.4) — any other param (incl. `expand`/`compression`) is ignored | `EngineUnavailable`→503, `EngineError`→502 (rendered as an `error` SSE frame under that status, `src/bin/gnosis_server.rs:272-281`, landed; the pre-U3 citation was `:264-281`, pre-U2 `:211-223`); an unrecognized **`mode`** token ⇒ **HTTP 400** with the **single `error` frame body** `{"type":"error","code":"validation_error","message":"<detail>"}` and the stream closed (**F4 — the status is pinned, not just the frame**, §5.4). **N1:** the 400/`error`-frame outcome belongs to `mode` **alone** — `expand`/`compression` are POST-payload-only keys with **no SSE param**, so `?expand=`/`?compression=` is **ignored ⇒ that option's default, never a 400** |
| `GET /engine/status` | `HealthReport` JSON (`state`, `version`, `subsystems`, `lastError`) | none (always 200). **Recorded ambiguity (not a pin):** this contract pins the **status** and the **body shape** but **not the response's `Content-Type`** — the landed handler returns it through axum's `Json` responder (`src/bin/gnosis_server.rs:285-289`, i.e. `application/json`), and no register row, golden or e2e assertion pins the header ([`docs/specs/u3-status-honesty-greens.md`](u3-status-honesty-greens.md) §"Contract ambiguities" item 5 records it as an ambiguity the U3 blind set could therefore not assert). A consumer that keys on the media type (the Astrographer shell's status pane) needs the pin — its home is the transport/status unit that owns §5.5's rendering discipline (the same unit that owns `docs/defects.md` **P-5**), where it lands with an e2e header assertion rather than being asserted by a property row |
| `GET /changes` **(U4 — this route does NOT exist at U1; pinned so U4 lands it as specified, §5.6)** | SSE: a **cursor-first** frame then ordered `change` frames | none on the subscribe path (no §11-mapped fail-state; no READY gate). Frames use the F2 §4.4 framing with the `cursor`/`change` event family (F2 §4.7) |
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

**Cross-cutting fail-states (U1 additions — the query surface, §5.3–§5.5):**
- **A bare (non-envelope) body on `POST /rag/query` → 400**, never accepted (§5.3).
- **The transport decode body is JSON** `{"code","message"}` with `Content-Type: application/json`
  (§5.5): `invalid_json`/`invalid_envelope`/`unsupported_schema_version`/`unknown_id_format` → 400;
  `unknown_method` → 422 (**CRUD only** — unreachable on the query surface).
- **A transport code is never a §11 code**: `StoreError::from_wire(code, Some(msg)) == None` for each
  of the five (§5.5).
- **N1 — the token fail-state split (pinned).** **An unrecognized `mode` token → 400 `validation_error`**
  on POST, and **HTTP 400** + the **`error` SSE frame body**
  `{"type":"error","code":"validation_error","message":"<detail>"}`
  on `/rag/stream` (F4: the SSE status is pinned, not only the frame) — never a silent default and never
  a 422 (§5.4). **An unrecognized `expand`/`compression` token → 400 `validation_error` on POST ONLY:**
  the SSE surface has **no** `expand`/`compression` query param, so a `?expand=`/`?compression=` param is
  **ignored and yields that option's default — never a 400 and never an `error` frame** (§5.4; the
  pre-remand "both paths" clause is superseded). **`expand` is a new state created by this amendment**
  (canonical FS-3 lists `mode`/`compression`, not `expand`; the upstream ask requests the addition —
  §5.4).
- **A wrongly-typed optional value is ABSENT, not a fail-state** (F5; **the wrong TYPE half** — its
  complement, a present **u64** out of range, is the FS-3 400 half, §5.3's type-vs-range bullet): `topK:"10"`,
  `topK:-1`/`topK:10.5`, `hyde:"yes"`,
  `wikiId:5`, `maxHops:null`, `binaryFirstPass:1`, `multiQuery:{enabled:true}` with `n` omitted,
  **`expand:5`**, **`compression:null`**, `mode:5` ⇒ that option's documented default (`mode` ⇒ `Flat`;
  `expand`/`compression` ⇒ `'none'`; `multiQuery.n` ⇒ **3**) and **no** 400 —
  the same policy the SSE path already applies to an unparseable `topK` (§5.3). **Read that outcome at the
  two layers of §5.3's "THE TWO-LAYER READING OF THE `absent ⇒` COLUMN" bullet:** the **decoded**
  `RagQueryOptions.<field>` is `None` (`mode == None`, never `Some(Flat)`; only a well-formed vocabulary
  string yields `Some(token)`) and the default is the **engine's query-time** reading — this list states the
  *effective* outcome, not a value the decoder writes. **The same field-level reading governs the two
  object-valued keys:** an object whose documented member is wrongly typed (`{"multiQuery":{"enabled":"yes"}}`,
  `{"multiQuery":{"enabled":true,"n":"3"}}`, `{"subTaskDag":{"enabled":"yes"}}`) ⇒ the **field is `None`** at
  layer 1 (never a present object with a member-level default) — §5.3's two-layer bullet, object-valued-option
  clause; `{"multiQuery":{"enabled":true}}` with `n` omitted is a **well-formed** object and stays
  `Some({enabled:true,n:3})`. Only a **string**
  reaches the token check (N2, §5.3). `query` is an exception on **both** of its halves — **absent** ⇒ `""`
  at layer 1 ⇒ 400 (the engine's FS-3 reading), a **present non-string** ⇒ the decoder's own
  `Err(Validation)` at layer 1 ⇒ 400 — and
  **`filters`** (a non-object / wrongly-typed member / unrecognized token ⇒ 400 `validation_error`;
  `null` and `{}` are valid, and `nodeKind:"community"` is 400 — N5) are the **two exceptions**.
  **The canonical corpus is §5.3's** (the token bullets + the wrongly-typed bullet); this list is a
  restatement, not a second corpus (N2).
- **The numeric inputs split at the type/range boundary (ruling 1, 2026-09-16 — pinned so this list is not
  read as one claim).** A **present value that is not a `u64`** — `topK:-1`, `topK:10.5`, `maxHops:-2`,
  `binaryCandidatePool:7.5`, a numeric **string**, `null` — is the **wrong-TYPE** half ⇒ absent ⇒ the
  documented default and **never** a 400; a **present `u64` out of its pinned range** (`topK:0`/`51`,
  `maxHops:0`/`6`, `binaryCandidatePool:0` with `binaryFirstPass:true`, `multiQuery:{enabled:true,n:0}`) is
  the **RANGE** half ⇒ FS-3 **400 `validation_error`**. **`multiQuery.n`** carries the same two halves with
  the member-level type rule of ruling 2: a non-u64 `n` makes the **whole option `None`** (⇒ disabled, no
  400), while a u64 `n == 0` under `enabled:true` is the 400 (full statement: §5.3's type-vs-range bullet).
- **Absent `mode` ⇒ `Flat`** and unknown payload keys are tolerated (§5.3/§5.4) — so the existing e2e
  POSTs with an extra `args` key still return **503** while not READY.
- **Precedence**: transport decode → **wire token resolver** (`mode` on both paths; `expand`/
  `compression` on the POST payload only — N1) →
  `validate_rag_options` (range/consistency) → READY gate → leg availability (§5.3).
- **The pre-U4 SSE surface reads exactly `query`/`topK`/`mode`** (§5.4); the remaining canonical
  `ragStream` params (`docs/specs/gnosis.md:873`) are **not part of it** and their wire treatment is
  **U4's** to pin — an unknown SSE param is ignored ⇒ default, never a 400 (N1).
- **The §11 map is unchanged (21 rows).** No `stale_revision`, no `decode_failed`; `GET
  /engine/status` is still always-200 and still the single status surface (§5.8).
- **The engine-side retrieval-semantics ruling (§5.9)**: explicit-leg errors (**503**), fusion
  degrades a failing leg to **empty** (**200**) — so `VectorIndexUnavailable`/`EmbeddingUnavailable`
  are never fusion outcomes. **F3:** the narrowing covers the canonical §4.6.1 `ragQuery` throw column
  (`docs/specs/gnosis.md:872`) and FS-13's `hyde: true` clause (`:1037`) as well: `mode=hybrid` +
  `hyde: true` + no/unreachable provider ⇒ **200** (leg empty), not 503.

---

## 11. The §5.x Property register (MANDATORY — PBT gate)

This register is the contract for the **property-based-testing gate** on the P2 server unit.
The TestWriter's executed layer runs under `cargo test` with a **deterministic pinned seed**,
**≤100 generated cases per register row**, **≤400 total cases** across the unit's whole property
layer (**a PER-UNIT cap, one unit's register = one layer — not a per-binary budget**: this file's §11
layer is 310 (`tests/props_gnosis_server.rs:16-24`) and §9.5's U2 layer is **400 as landed**
(`48/100/70/80/23/41/38`, `:26-40`; the pre-landing indicative budget was 245, §9.5.3), each ≤ 400 on its
own; the binary total is their sum and is not itself capped — scope pinned in
§9.5.3; authority decision `PBT-GATE-MANDATORY`, `docs/decisions.md:68`), **stop-after-5** (report ≤5 distinct held/broken counterexamples per row), and records
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
| `P-IM-3` | IM | **Endpoint routing is a bijection — RESTATED by U1 as a GROWTH INVARIANT (no fixed count).** The routing table maps every `"<VERB> <path>"` row to **exactly one** handler, and every handler is reachable by **exactly one** row; the rows are **pairwise distinct**; the **11 document-CRUD rows equal the P1a `ENGINE_ENDPOINTS` rows verbatim**; and the **retrieval trio is present** (`POST /rag/query`, `GET /rag/stream`, `GET /engine/status`). **No fixed row count is part of this invariant** — the pre-U1 "the table has exactly 14 rows" clause is **DELETED** (§U1/§5.2), because a fixed count would make every future route a contract violation instead of an amendment. | `strat:route-bijection` | ∀ distinct rows `(row_a, handler_a)`, `(row_b, handler_b)` in the routing table: `row_a != row_b`; `handler_a != handler_b`; the 11 CRUD rows' `"<VERB> <path>"` values are **equal, element-wise and in order, to the P1a `ENGINE_ENDPOINTS` rows**; the three retrieval rows are present. **No `len() == N` assertion** (the table's current length — 14 at U1 — is state, not invariant). |
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
- **`P-IM-3 — strat:route-bijection`.** Enumerate the current routing-table rows; assert pairwise-distinct
  rows and pairwise-distinct handlers; assert the 11 CRUD rows equal the P1a `ENGINE_ENDPOINTS` rows
  verbatim; assert the retrieval trio is present. **Do NOT assert a fixed row count** (U1 deleted the
  "exactly 14 rows" clause; growth is permitted by the §5.2 amendment rule). The pre-U1 fixed-count
  assertions remain green today (the table is still 14 rows) but are **superseded**: U4 supersedes the
  count assertions (`tests/gnosis_server_conformance.rs:305`, `tests/props_gnosis_server.rs:702`,
  `tests/blind_p2_gnosis_server_greens.rs:218`) in the same unit that adds `GET /changes` and the paged
  read routes.
- **`P-IM-2 — strat:decode-outcome-total` (U1 note).** The 5-variant domain is unchanged; the
  **transport code** mapping of §5.5 has the *identical* domain (a code is defined exactly where
  `request_decode_status` is defined, `None` exactly where it is `None`). That mapping is a **new
  function**, so its invariant row is **owed by U2** (see the register notes below) — U1 adds no row.
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

## Register notes — rows owed by the code-bearing units (U1 adds NO row)

**Zero new register rows land in U1** (see the §U1 status block for the justification: a docs-only
unit, and each code-bearing unit owes its **own** typed §5.x rows in its own spec). U1 **restates**
`P-IM-3` in place (above) and nothing else. The following rows are **owed** — named here so the unit's
SpecWriter authors them **together with its code**. **This is a statement of obligations, not a
register**: a row is an `owed` row until its own unit authors it in its own spec with its own
`Property-id` and `Strategy-id`, so **U1's register above is still exactly 7 rows and U1 adds none**. The
U2 rows the remand added below (the F1 `filters`-mapping rows, the F2 token-resolver row, the F5
wrongly-typed row, the F4 SSE-status row) are likewise **owed by U2**, not landed by U1.

**UPDATE — U2 and U3 are AUTHORIZED and their rows are now AUTHORED (this pass, docs-only).** The user
authorized the two code-bearing units, and this SpecWriter pass authored their typed registers in
**§9.5** of this file: **U2 ⇒ §9.5.1** (`P-IM-4`, `P-IM-5`, `P-IM-6`, `P-SM-4`, `P-TP-2`, `P-TP-3`,
`P-TP-4` — 7 rows) and **U3 ⇒ §9.5.2** (`P-IM-7`, `P-IM-8`, `P-IM-9`, `P-SM-5`, `P-SM-6` — 5 rows), with
the property-layer execution plan in **§9.5.3**. The table below therefore marks **nine** U2 obligation
rows (`AUTHORED ⇒` annotations — `P-IM-4`, `P-IM-6`, the second `P-IM-6` **merge** row, `P-SM-4`, the second
`P-SM-4` **merge** row, `P-IM-5`, `P-TP-3`, the `P-TP-2` split row and the `P-SM-4` parity row: **eight** were
already the register notes' owed sketches and the **ninth** is the row the first register remand **added** (the
`QueryAuditFilters`-touching mapping; the pre-remand sentence under-counted these as "six" and **omitted three**
of them — the `P-IM-5` wrongly-typed row, the `P-TP-2` split row and the `P-SM-4` parity row) and the one U3
obligation row (`P-IM-7`; its four sibling ids `P-IM-8`/`P-IM-9`/`P-SM-5`/`P-SM-6` are **splits** of that one
obligation, not four further obligation rows) **AUTHORED** (each naming the `Property-id` that carries it, or the id
it merged into); their obligation text is **kept verbatim** as the provenance record. **The U4/U5 rows
stay `owed` and `id-less`** — those two units remain **NOT authorized** and **no `Property-id` is claimed
for them by this pass**. Note that the ids are **allocated from a single per-file p2 sequence** (F1's ruling;
§9.5's header): cross-**file** look-alikes are the repo's convention (the `p2` register and the F2 register
`docs/specs/7-2-wire-property-register.md` each contain a `P-IM-1` legitimately, executed by
`tests/props_gnosis_server.rs` and `tests/props_wire.rs` respectively), but **within this file** the twelve §9.5
rows have **twelve distinct ids** — U3's three IM rows were renumbered to `P-IM-7`/`P-IM-8`/`P-IM-9` by the
register remand precisely because a within-file duplicate id would break per-row tag/seed reporting
(§9.5.3.1). The table below has **no `Property-id` column** — it never had
one (an owed row is id-less by construction), which is why the authored ids are recorded in the
`owed by` cell's annotation rather than by adding a column.

***(U6 UPDATE, 2026-09-22 — ADDITIVE; the UPDATE sentence above stands as its record.)*** **U6 is
AUTHORIZED** (the user's go-ahead of **2026-09-22**: *"open a follow-up honesty/freshness unit now"*) and its
typed register is **§9.5.6** — **6 rows**: `P-IM-16` (the freshness/capability predicate), `P-IM-17` (the
stale-index loud FS-14 + the flag/leg agreement), `P-IM-18` (the boot snapshot's epoch alignment), `P-IM-19`
(the stderr build diagnostic — `U5-ADV-4`), `P-SM-8` (the status read's purity under the amended predicate),
`P-TP-6` (the provider request bound — `U5-ADV-2`) — with the execution plan in **§9.5.6** and the six
per-row coverage notes in **§9.5.4**. **U6's obligation is the `U5-ADV-1` defect row itself** (a post-boot
write leaving the index frozen while the flag reads `true`), split across those six rows; U6 does **not**
re-open U5's obligation row (its `AUTHORED` annotation above stands) and claims **no** id from U2/U3/U5 (the
single per-file sequence continues at `P-IM-16`/`P-SM-8`/`P-TP-6`). **The §11 property table above is still 7
landed `P2` rows and gains none**; the U6 ids are new claims in the §9.5 namespace only, and
`P-IM-1`…`P-TP-1` with the 21-row §11 map are **unchanged** by U6 (no route, no variant, no status, no wire
code). **U4's row stays owed and id-less** (U4 remains **HELD** — §12 item 6 stands). ***(REMAND-1, 2026-09-22 —
docs-only, additive: the six ids/kinds/strategy ids/caps above are unchanged. What this remand added is the
**per-site pinning** of the predicate's consumers (the fusion site consults the predicate with its refuse
**suppressed**; the two explicit sites refuse) and the **second lib-visible seam** (`PROVIDER_REQUEST_TIMEOUT` /
`provider_client`) that makes `P-TP-6` assertable. **No §11 row is added, no wire code, no status and no
variant moves** — the §11 map still holds **21 rows** and the `StoreError` taxonomy **21 variants**.***)***

| owed by | invariant the row must pin | observable-as-property sketch |
| --- | --- | --- |
| **U2** — **AUTHORED** ⇒ `P-IM-4` (§9.5.1) | **The shared query decoder is envelope-strict and tolerant of unknown payload keys.** For any well-formed `ragQuery` payload, decoding yields the `query` + the documented camelCase options; an unknown/extra key is ignored; a bad `schemaVersion`/`idFormat` is a transport decode failure, never a partial decode. | ∀ payload: `decode(payload).is_ok()` with the documented fields mapped; ∀ foreign key `k`: `decode(payload ∪ {k})` yields the same options as `decode(payload)`; `schemaVersion != 1 || idFormat != "opaque-string-v1"` ⇒ `Err`. |
| **U2** — **AUTHORED** ⇒ `P-IM-6` (§9.5.1) | **The `filters` decoding is the canonical §4.5.2 token mapping and is NEVER a raw serde pass-through (F1).** From any `filters` object, the four members map to the frozen store types per §5.3's table (`content|fact|reference`; `link|embed|crosslink`; `{documentId,nodeId}`; `FRESH|RESOLVED|STALE|BROKEN`); a member outside its pinned token set (incl. `nodeKind:"community"` and any non-referencing `edgeType`) ⇒ `ValidationError`; and the decoded `filters` **reaches the engine** (it is not an all-`None` `QueryAuditFilters`). | ∀ payload with `filters`: the mapped `RagQueryOptions.filters` equals the expected store-typed filter (never `Some(all-None)` for a non-empty payload); ∀ out-of-set token: `decode` ⇒ `Err(ValidationError(_))`; a payload **without** `filters` ⇒ `None`. **Negative property (the GR-2 defect class):** `decode({"filters":{…camelCase…}}).filters != Some(all-None)` — quantified over **non-empty** payloads, since the caller-sent `{}` legitimately **is** `Some(all-None)` (N5; §5.3 pins its expectation). |
| **U2** — **AUTHORED** ⇒ merged into `P-IM-6` (§9.5.1; the mapping and its negative form share one exact-equality observable) | **The `filters` mapping is the only decoder path that touches `QueryAuditFilters` (F1).** `serde_json::from_value::<QueryAuditFilters>` is never the decoder (that type has no camelCase rename and no `deny_unknown_fields`, so a pass-through yields `Some(all-None)`). | ∀ payload: the decoded filter's members are non-`None` exactly where the payload named a recognized member — the decoder's mapping, not serde's field binding, produces them. |
| **U2** — **AUTHORED** ⇒ `P-SM-4` (§9.5.1) | **The mode rule is total, single-sourced and case-insensitive.** For any input string, the resolver returns either one of the four modes (ASCII-case-insensitively matched) or a `ValidationError`; absent ⇒ `Flat`; the POST path and the SSE path call the *same* resolver (observable as: the two paths' outcomes agree for every input). | ∀ `s`: `resolve(Some(s))` ∈ {`Ok(Flat/Graph/Vector/Hybrid)`, `Err(ValidationError(_))`}; `resolve(None) == Ok(Flat)`; ∀ casing variant `v` of a token: `resolve(v) == resolve(token)`; POST/SSE route outcomes agree. |
| **U2** — **AUTHORED** ⇒ merged into `P-SM-4` (§9.5.1; one rule, three vocabularies, one path-aware domain) | **The token resolver is total over all three enum-valued option tokens and lives at the wire layer (F2).** For `mode`, `expand` (`none|parent`) and `compression` (`none|filter|extract|graph`), each resolver returns the typed enum or a `ValidationError`; no unrecognized token can reach `validate_rag_options` (which is not edited); the resolver precedes it in the precedence order. **N1:** the domain is **string values on the path that reads the key** — `mode` is read by POST **and** SSE, `expand`/`compression` by the **POST payload only** (the SSE surface has no such params, so the row MUST NOT quantify over an SSE `expand`/`compression` input); **N2:** a non-string value is outside the resolver's domain (⇒ absent ⇒ default, pinned by the wrongly-typed row below). | ∀ token `t` in the union of the three vocabularies (any ASCII casing) **on a path that reads its key**: the corresponding option decodes to the typed enum; ∀ non-token **string**: `Err(ValidationError(_))`; ∀ non-string value: `None` ⇒ default (never a token error); ∀ request with an unrecognized token **and** an out-of-range `topK`, the error is the token's (step 2 before step 3). |
| **U2** — **AUTHORED** ⇒ `P-IM-5` (§9.5.1; the per-key ruling is now §5.3's **14-key table**, cited by the row) | **A wrongly-typed optional value decodes as ABSENT, with `query` and `filters` the two `Err` exceptions.** For any documented key whose wrongly-typed outcome is "absent/default" (the §5.3 per-key table), a present value of the wrong JSON type ⇒ that option's documented default (no error) — read **at the field (layer 1)**, so a **wrongly-typed documented member** of an object-valued option (`{"multiQuery":{"enabled":"yes"}}`, `{"multiQuery":{"enabled":true,"n":"3"}}`, `{"subTaskDag":{"enabled":"yes"}}`) ⇒ the **field is `None`**, never a present object with a defaulted member (§5.3's two-layer bullet, object-valued-option clause); `multiQuery` with `n` omitted **inside a well-formed object** ⇒ `Some({enabled:true, n:3})` (`n == 3`); **`mode` that is not a string ⇒ absent ⇒ `Flat`** (inside that rule, **not** an exception — only a **string** reaches the token resolver). **N2:** the corpus MUST include the two enum-valued keys (`expand`/`compression`), so their wrongly-typed reading is pinned identically to their token reading; **N4:** an "absent ⇒ default" row is an *acceptance* claim only and MUST NOT be presented as evidence that the option is *honored* (see the reachability note in §5.3); **N5:** the `filters: {}` corner has the pinned expectation below. | ∀ key/value pairs from the wrong-type corpus (**§5.3's per-key table is canonical**: `topK:"10"`, `hyde:"yes"`, `wikiId:5`, `maxHops:null`, `binaryFirstPass:1`, `subTaskDag:5`, `mode:5`, **`expand:5`**, **`compression:null`**, and the **object-member-misuse instances `{"multiQuery":{"enabled":"yes"}}`, `{"multiQuery":{"enabled":true,"n":"3"}}`, `{"subTaskDag":{"enabled":"yes"}}`**): the decoded field is `None` (`multi_query`/`sub_task_dag` included — a wrongly-typed documented **member** is a field-level `None`, §5.3's two-layer bullet) (and `mode` ⇒ `Flat`, `expand`/`compression` ⇒ `'none'` **at the engine's query-time layer only** — §5.3's two-layer bullet: `Some(Flat)` is never a decoder output); `{"multiQuery":{"enabled":true}}` ⇒ `n == 3` **with the field `Some(…)`**; `{"multiQuery":{"enabled":"yes"}}` / `{"multiQuery":{"enabled":true,"n":"3"}}` / `{"subTaskDag":{"enabled":"yes"}}` ⇒ the field is **`None`** (field-level reading — the members `enabled`/`n` are **not** "keys": §5.3's two-layer bullet, object-valued-option clause, so a wrongly-typed member never yields a present object whose member was defaulted); `filters:[]` / `{"nodeKind":5}` / `{"nodeKind":"community"}` ⇒ `Err(ValidationError(_))`; **`filters:{}` ⇒ `Some(QueryAuditFilters{ node_kind: None, edge_type: None, target: None, state: None })`** (present, valid, honored — `{}` is the caller's identity filter, and it is *not* the defect instance the F1 negative property targets). |
| **U2** — **AUTHORED** ⇒ `P-TP-3` (§9.5.1; the **pure** half — the rendered-body half moved to the conformance golden, F8) | **The transport decode code mapping is total over the request-decode domain and disjoint from §11.** Every request-decode `DecodeError` has exactly one code; the code set is pairwise disjoint from the 21 §11 codes; `DecodeError` variants outside the domain have no code. | ∀ `e` in the 5: `request_decode_code(e).is_some()` and `StoreError::from_wire(code, Some("m")).is_none()`; ∀ `e` outside: `request_decode_code(e).is_none()`; `request_decode_status` defined on exactly the same 5. **The rendered body/`Content-Type` bytes are NOT this row** (the bin-private renderer is unreachable from `tests/`) — they are the named conformance golden `v15_request_decode_error_body_exact` (F2 §12 V-15/**V-15.1**), with the message contract restated: verbatim, **may be empty**; `UnsupportedSchemaVersion(v)` ⇒ non-empty. |
| **U2** — **AUTHORED (split)** ⇒ the decode-level identity half is `P-TP-2` (`P-IM-4` holds the key→field mapping; §9.5.1) | **The query POST path honors the options surface.** For any decoded options, the `RagQueryOptions` handed to `rag_query`/`rag_stream` equals the wire options element-wise (so `mode`/`topK`/`wikiId`/`filters` are no longer inert). **N4 (honesty qualifier):** this row is a **pass-through** claim, not a per-option **consumption** claim — `subTaskDag` is inert on the engine side (`src/store/mod.rs:664`/`:1022`, no read site) and `binaryFirstPass`/`binaryCandidatePool` are consumed by the vector leg only, so the row's observable MUST be "the options object handed over equals the decoded options", never "the option changed a leg's behavior". | ∀ payload: the options passed to the engine `==` the options the wire payload names (with the documented defaults for absent keys) — compared **as the options value**, on the modes §5.3's reachability note names. |
| **U2** — **AUTHORED** ⇒ merged into `P-SM-4` (§9.5.1; the parity half is the two-path agreement clause, whose status/body half is live-transport) | **SSE/POST parity includes the fail-state status (F4) — and is scoped to `mode` (N1).** For an unrecognized **`mode`** token on either path the outcome is `ValidationError`; on the SSE path the response status is **400** with the single `error` frame body, identical to the POST path's 400 `validation_error`. **`expand`/`compression` have NO SSE outcome**: the SSE surface does not read those params, so `?expand=`/`?compression=` is ignored ⇒ the option's default and the row MUST NOT assert a 400 for it. | ∀ unrecognized **`mode`** token: POST ⇒ 400 `validation_error`; SSE ⇒ status 400 + one `{"type":"error","code":"validation_error",…}` frame then close. ∀ unrecognized `expand`/`compression` **string** on the POST payload: 400 `validation_error`; ∀ `?expand=`/`?compression=` **SSE param**: ignored, option at default, status not 400 (no `error` frame). |
| **U3** — **AUTHORED ⇒ LANDED-GREEN (2026-09-17)** ⇒ `P-IM-7` (§9.5.2; the DEGRADED-boot and boot-wiring halves land as `P-IM-8`/`P-IM-9` and the projection halves as `P-SM-5`/`P-SM-6`) | **A subsystem flag is `true` iff that subsystem's full query-time capability is wired and functional for the current store** (the exact predicate U3 implements: e.g. `vector` ⇒ an index is present in the current derived snapshot; `reranker` ⇒ a reranker is wired — `false` today). | ∀ store states in the sampled corpus: `get_engine_status().subsystems.<flag>` is `true` iff the flag's capability predicate holds; `reranker` is `false` in every reachable state at U3. **As landed:** all five §9.5.2 rows HELD (127 executed ≤ 400; `docs/specs/p2-gnosis-server.md` §9.5.2's landed bullet). |
| **U4** | **The change cursor advances by exactly 1 per committed journal entry and is process-lifetime monotonic**; and the change feed emits the cursor-first frame before any change frame, with strictly increasing cursors in commit order. | ∀ committed entries: `cursor_{n+1} == cursor_n + 1`; `cursor` never decreases; the first emitted frame is `cursor`; the change frames' cursors are strictly increasing. |
| **U4** | **Paginated reads are bounded.** For any read request, `page_size` outside `1..=100` (or `page < 1`) ⇒ `ValidationError`; no response exceeds the cap; the response carries the `DocumentList`-style bounds metadata. | ∀ generated page params: over-cap/under-1 ⇒ `Err(ValidationError)`; `response.items.len() <= 100`; `total`/`page`/`page_size` present and consistent. |
| **U5** — **AUTHORED 2026-09-22 ⇒ 8 rows in §9.5.5** (`P-IM-10`, `P-IM-11`, `P-IM-12`, `P-IM-13`, `P-IM-14`, `P-IM-15`, `P-SM-7`, `P-TP-5`; the unit is **AUTHORIZED** — `docs/specs/gnosis-grq-inbound-review.md` §14 POST-RECORD UPDATE 1, Q3 = "(B) U5 ONLY" — and its **code is owed**; the register table, the execution plan and the per-row coverage notes are in **§9.5.5**/**§9.5.4**) | **A READY boot builds the vector index** (so `vector: true` becomes an honest claim and `mode=vector` stops returning `VectorIndexUnavailable` on a freshly booted server). | after boot: the derived snapshot has `vectors.is_some()`; `engine_status().subsystems.vector == true`; `mode=vector` returns a result (given a provider). **As authored:** the one obligation is split across §9.5.5's eight rows — `P-IM-10` (the build's total function + its closed two-outcome error set), `P-IM-11` (reachable ⇔ built; the whole corpus; full field only; the empty-store pin), `P-IM-12` (determinism + store-side-effect freedom), `P-IM-13` (atomic failure — no partial index), `P-IM-14` (the flag/wiring equality + the V-8.1 fixture, **scoped to `Reachable` + a snapshot whose `vectors` is `Some`**), `P-IM-15` (the build-failure boot outcome: `Degraded` + `vector:false` + **`embedding:true`** via the pinned failure order — the provider **is** wired — never `Ready`), `P-SM-7` (the post-boot query transition, incl. `Ok` over an empty index and FS-14 without one) and `P-TP-5` (adversarial provider shapes). **REMAND-1 (2026-09-22, docs-only):** the 8 rows stand — the pass pinned the failed-build call order, the V-8.1 `Some` scoping, the corpus-seeding calls (§9.5.5's surface #7 + its "corpus seeding" clause), the build body's module placement, the `embed`-call-count rule (in the contract table (2)) and the exact `tests/wire_conformance.rs` lines that move; **no row added or deleted**. |

**P-IM-1/P-IM-2/P-SM-1/P-SM-2/P-SM-3/P-TP-1 are unchanged by U1** — the §11 map still has 21 rows and
their domains are closed sets that U1 does not alter.

**(U2/U3 update — U2 LANDED 2026-09-17, U3 LANDED-GREEN 2026-09-17.)** The same holds for **U2** and **U3**: this pass **adds no row to the table above** and
edits none of it. The seven P2 ids remain exactly as landed, `P-IM-3` stays the growth invariant (and the
route table is still **14 rows** — neither unit adds a route), and the 5-variant request-decode domain of
`P-IM-2`/`P-SM-2` is untouched (U2's §5.5 work adds a **code** mapping **beside** `request_decode_status`,
not a status and not a `DecodeError` variant). The U2/U3 rows are the **new §9.5 registers**, which
quantify over disjoint concerns (the `ragQuery` decode/token/transport-code surfaces and the status-flag
derivation), so there is no double-claim between the two register sections. **Both units' rows have now
landed and HELD** (U2: 7 rows, 400 executed; U3: 5 rows, 127 executed — §9.5.2's landed bullet), so the
obligation rows in the table above (marked `AUTHORED`) are carried by landed rows; **U4's rows stay owed
and id-less.** **(U5 update, 2026-09-22.)** U5 is **AUTHORIZED** and its obligation row is now
**`AUTHORED`** (the §11 row above, annotated `AUTHORED 2026-09-22 ⇒ 8 rows in §9.5.5`): its typed register
is **§9.5.5**'s table (`P-IM-10`…`P-IM-15`, `P-SM-7`, `P-TP-5` — **8 ≤ 8**), with the execution plan and
the per-row coverage notes in that section and §9.5.4. So **only U4's row remains owed and id-less** — U4
is still **HELD** and **no `Property-id` is claimed for it by this pass**. The §11 property table above is
still **7 landed `P2` rows** and gains none; the U5 ids are new claims in the §9.5 namespace only.
`P-IM-1`/`P-IM-2`/`P-IM-3`/`P-SM-1`/`P-SM-2`/`P-SM-3`/`P-TP-1` and the 21-row §11 map are **unchanged** by
U5 (it adds no route, no variant and no status).

**(U5 update — the TestWriter's surface, 2026-09-22.)** U5's rows are **lib-level**: they need a tokio
runtime + a real `Store` + a **deterministic injected** `EmbeddingProvider`, exactly like U3's status rows
— never a live provider and never the ambient environment. U5's one **live** obligation (the built index
observed through the running `gnosis-server` bin with a controlled provider, and `mode=vector` serving on
it) is homed in the live battery's new row **`R-L3`** (`docs/specs/p2-gnosis-server-live-pending-battery.md`
§3.5), following `R-L2`'s precedent. The **U5 layer's cap is 355 ≤ 400** (`60/50/40/45/45/30/45/40`), each
row ≤ 100, and **a cap is a maximum rather than an expected count** (the measured layer is **314 executed /
355 caps**; a row executing fewer cases than its cap is compliant — §9.5.5's execution plan); the binary
total is now `310 + 400 + 127 + 355 = 1192`, a sum and **not** a violation (§9.5.3's
per-unit scope pin). *The pre-amendment figures this note carried — the cap `340 ≤ 400`
(`45/50/40/45/45/30/45/40`) and the binary total `310 + 400 + 127 + 340 = 1177` — are **superseded**
records: the landed `P-IM-10` corpus needs **55** generated cases, so the 45 cap was unachievable as
written. §9.5.5's post-red-phase register amendment is the authority for the amended set.*

## API notes for the TestWriter

- **Pure/synchronous surface.** The status-mapping (`server_status`), request-decode-outcome
  (`request_decode_status`), and routing-table (`route_bijection`) fns are pure synchronous fns
  over values — `#[test]`, no tokio runtime, no engine instance. Build `StoreError`/`DecodeError`
  values directly from the public `src/store/mod.rs` + `src/wire/` types.
  **(U1 addition — U2-time.)** The shared query decoder, the mode resolver and the transport
  decode-code mapping (§5.3–§5.5) are pure synchronous fns too, reachable from `gnosis::wire::…`
  / `gnosis::server_…`; the mode rule is testable without a runtime and without an engine instance,
  and the POST/SSE agreement is testable by driving both paths' decoders with the same inputs **for
  `mode`** — the SSE inputs arrive as the decoder's **`sse_params` argument** (§9.5.1's signature note,
  F6), and `expand`/`compression` have no SSE input to drive (N1, §5.4).
  **(U2/U3 remand additions.)** The three surfaces the registers now name but a pure test cannot
  cover are homed explicitly: the **rendered decode-error body** (the `v15_request_decode_error_body_exact`
  golden in `tests/wire_conformance.rs`, F8), the **SSE handler's wiring** (the `?mode=` live assertions of
  `P-SM-4`, F15) and U3's **`boot_wiring`** function (F11, lib-level, with its provider-reachable case in the
  live battery). U3's status rows need a tokio runtime + a `Store` (as today's integration tests do), not a
  live server.
- **Live-transport surface.** The end-to-end transport test (§9) runs against the real server
  (a tokio runtime + the bound `127.0.0.1:<port>`), exercising the `EngineUnavailable`/
  `EngineError` split and the document-CRUD round-trips over the wire. **(U2/U3 additions.)**
  `tests/gnosis_server_e2e.rs` also carries `rag_query_unrecognized_mode_is_400_e2e` (`P-TP-2`'s (α), F10) and
  `engine_status_reports_no_false_embedding_claim` (`P-IM-9`'s provider-absent transport half, F11);
  `?mode=bm25` / `?mode=HYBRID` / `?expand=` on `/rag/stream` are the `P-SM-4` wiring assertions (F15).
- **Determinism/seeding.** Use one deterministic pinned seed per binary, ≤100 cases per row,
  ≤400 total across §9.5's property layer **— a PER-UNIT cap: §9.5's executed U2 layer (400 as landed,
  `tests/props_gnosis_server.rs:26-40`; the pre-landing indicative budget was U2's 245 + U3's 125 = 370,
  §9.5.3) is ≤ 400 on its own, the executed U3 layer is **127** (as landed, `:26-40`/`:278-295`), and the 310 of the landed `p2` layer
  (`tests/props_gnosis_server.rs:16-24`) is ≤ 400 on its own; the binary's `310 + 400 + 127 = 837` (**the pre-U5 record** — the post-U5 total is `310 + 400 + 127 + 355 = 1192`, §9.5.5's execution plan as amended by its post-red-phase register amendment, 2026-09-22; the `340`/`1177` pair this note carried is superseded — **REMAND-2 NOTE 9, 2026-09-22, as amended**) is their sum and is NOT
  capped (scope pinned in §9.5.3; authority `PBT-GATE-MANDATORY`)**; stop-after-5;
  report each row held/broken with its `Strategy-id`.

---

## 12. Cross-references and ownership hand-off

- **Contract authority:** `docs/specs/gnosis.md` §4.1.3, §4.1.4, §4.4.3, §4.4.5, §4.6.1, §6.
- **U1 amendment authority (gate-1):** `docs/specs/gnosis-gr-inbound-review.md` — §Per-GR verdict table
  (GR-1/GR-2/GR-3/GR-4/GR-5), §The four rulings, §Ordered workstream (U0/U1/U2/U3/U4/U5 +
  SHELL-1/SHELL-2), §Blast radius + reconcile order, §Regression watch, §Residual risk register,
  §Appendix A (the change cursor), §Appendix B (the FS-13/14/15 reconcile), §Go-ahead record.
  Decisions `GNOSIS-CHANGE-CURSOR` + `SUBSYSTEM-FLAG-CAPABILITY-SEMANTICS` (`docs/decisions.md`).
- **U1 additions in this file:** §U1 (status block, **incl. the 2026-09-16 REMAND-2 entry**), §2 (3 new
  bullets), §5.2 (restated routing
  invariant), §5.3–§5.9 (REMAND-2 adds the per-option reachability note + the `filters:{}`/
  `nodeKind` pins to §5.3 and the token/SSE split + the named pre-U4 SSE surface to §5.4),
  §7.1/§7.2, §10, §11 (`P-IM-3` restated + register notes), §12/§13.
  **(U2 red-phase ambiguity pin, 2026-09-16:** §5.3's **two-layer reading** bullet (the decoder's
  `Option`-preserving `None` vs the engine's query-time default) + its §5.4 cross-reference and the
  boundary-state parenthetical — annotation only, **no rule, row or reference changed**; the companion F2
  item is the **V-15.1** binding note in `docs/specs/engine-wire-contract.md` §12.**
  **(U2 object-valued-option field-level pin, 2026-09-16:** §5.3's two object cells (`multiQuery`,
  `subTaskDag`) read at **layer 1** + the two-layer bullet's **object-valued-option clause** and its
  `.multi_query == None` enumeration entry + the `P-IM-5` row/coverage-note alignment + the four reconciled
  clauses (§5.3's wrongly-typed corpus, §10's cross-cutting list, §11's `P-IM-5` sketch, this §12 list) —
  the companion F2 item is the object-member sentence in `docs/specs/engine-wire-contract.md` §4.5.**))**
- **U2/U3 register additions in this file (2026-09-16, two passes + the bookkeeping pass):** §U1's status
  block (the U2/U3 bullet + the REMAND bullet + the **bookkeeping-pass bullet recording the closing
  verification's 4 findings, B1–B4**), **§9.5** (U2's rows §9.5.1, U3's rows §9.5.2, the execution plan §9.5.3 — **incl. the per-unit ≤400 scope pin, B4** —, the
  reviewer dispositions §9.5.3.1 **+ §9.5.3.2 (REMAND-2)**, the per-row coverage notes §9.5.4), §5.3 (the **per-key wrongly-typed
  table** added by the register remand, F4), §5.5 (the restated `message` contract, F8), §10 (the
  non-object-payload code narrowed to `invalid_envelope`, F9), §11 (the AUTHORED annotations + the id
  allocation note + **the per-unit ≤400 restatement, B4**), and §7.x/§12 (citations). **No row was deleted and no landed U1 rule was edited**; the
  bookkeeping pass changed no rule, row substance or row count — it annotated authorization status
  (B1/B2 in the F2 and gate-1 files) and pinned the cap's scope and the nine-row composition (B3/B4 here).
- **U5 spec-gate REMAND-1 corrections in this file (2026-09-22, the one-pass remand on §9.5.5 — docs-only,
  NO row added or deleted, the 8-row count and every strategy id kept):** a **correction pass** that makes the
  U5 contract unambiguous for a TestWriter who reads the spec only. It (i) pins the **failed-Reachable-build
  branch's call order** in §9.5.5's contract table (failure row 3 and ordered bullet 3d) and restates
  `P-IM-15`'s observable to that order (the provider is wired **even though the build failed**, because the
  probe succeeded — hence `embedding:true`); (ii) restricts the **V-8.1 mask clause** to `Reachable` **+ a
  snapshot whose `vectors` is `Some`** in `P-IM-14`'s invariant/observable and its §9.5.4 coverage note
  (the unconditional rule stays `flags.vector == snapshot().vectors.is_some()`) and records the same
  clarification in §5.8 above; (iii) pins the **populated-corpus seeding calls** (the `RagStore` methods +
  their exact signatures + the corpus shape) in §9.5.5's surface table (new surface #7) and in its new
  "corpus seeding" clause, so `P-IM-10`/`P-IM-11`/`P-SM-7`'s populated states need no invented API;
  (iv) states where the build's body lives (**inside `src/store/`'s module**, on the module-private
  `Store.shards`/`StoreShard.docs`, re-exported from `src/lib.rs`) so the "no new accessor" claim is not read
  against the corpus read; (v) moves the **one-`embed`-call-per-embeddable-node / strictly-sequential /
  no-call-for-a-`value:None`-node** rule into the contract table (its new corpus table's last row) and cites
  it from `P-IM-10`/`P-IM-11`/`P-SM-7`/`P-TP-5` and their coverage notes; (vi) restates the §9.5.4 heading's
  format rule as "≤5 lines per row, or an explicit deviation note in the row" and marks the eight U5 notes
  with that deviation note; and (vii) states the **exact `tests/wire_conformance.rs` lines that move** in the
  move table (only `:1195`, `:1204` and `:1235` — probes (2)/(3) and probe (4)'s assertions stay; ***REMAND-2
  NOTE 6 / REMAND-3 NOTE 6, 2026-09-22: read this list as **the lines U5 touches**, not three edits of one class
  — the pinned scope is **exactly two** *value* edits (`:1195`, `:1204`), while `:1235` is the **redundant-line
  drop** (and the two prose reconciliations at `:1205`/`:1191` sit alongside them), §9.5.5's move table***). It
  **adds no rule**: no new §11 row, no new `StoreError` variant, no new wire code, no new route, no new
  store accessor, and no change to the frozen `EngineSubsystems`/`HealthReport` shape.
- **U5 spec-gate additions in this file (2026-09-22, docs-only):** §U1's status block (the **U5 AUTHORIZED +
  spec gate** bullet + the same-unit-edit bullet), §5.8 (the **U5 answer** bullet to its own open question),
  **§9.5.5** (U5's contract + its typed register — 8 rows — + the execution plan + the "must move in the same
  unit" table + the adjudication notes + the residual list), **§9.5.4** (U5's eight per-row coverage notes),
  §11 (the U5 obligation row annotated `AUTHORED` + the U5-update notes), §12 item 5 (U5's authorization and
  its owed same-unit edits) and §13 (U5's constraint bullets). **No pinned rule, no register row's substance and
  no row count changed for U2/U3/U4**; the p2 §11 property table is still **7 landed rows** and the §11 map
  still **21**.
- **U5 spec-gate REMAND-2 (2026-09-22, the second one-pass remand on §9.5.5 — docs-only, ADDITIVE; the spec
  gate stays closed until the reviewer loop returns empty).** The re-review confirmed **all 13 REMAND-1
  findings resolved** and returned **3 MUST-FIX + 5 SHOULD-FIX + 3 NOTE**; every one is addressed in place here,
  in `docs/specs/engine-wire-contract.md` and in `docs/specs/p2-gnosis-server-live-pending-battery.md`, with the
  superseded clauses kept and annotated (**the finding-by-finding disposition is reported with this remand** —
  §12's REMAND-2 bullet records the substance). The **only** rule-level corrections this round makes are the
  two **error-outcome** ones: **(1)** a **non-READY** store's `mode=vector` request is **FS-8
  `EngineUnavailable` → 503 `engine_unavailable`** (the READY gate precedes every leg check), so FS-14
  `VectorIndexUnavailable` is a **READY-store-only** outcome (MUST-FIX 1, echoed live in the battery's `R-L3`
  **(v)** — ***(POST-GREEN SPEC AMENDMENT, 2026-09-22: that live echo is **PARKED**, because an empty boot
  corpus cannot drive a build failure live; see §9.5.5's adjudication note 1 extension and the battery's U5
  amendment item 5. The rule this MUST-FIX pins — FS-8 for a non-READY store, FS-14 a READY-store-only outcome —
  is **unchanged**; only the live *home* of the failed-build criterion is parked.)*** (v) — MUST-FIX 2); and
  **(2)** the failed-`Reachable`-build's live query probe therefore asserts
  `engine_unavailable`, never `vector_index_unavailable`. Everything else **restates, re-points, re-measures or
  annotates**: the `EngineSubsystems` census (**23**, re-counted), §5.9's citations, the same-unit prose texts
  in `tests/wire_conformance.rs`, the battery's provider precondition, the pre-U5 binary total (`837` ⇒ the
  recorded pre-U5 figure against the post-U5 `1177`) and a **new residual** (the `Degraded` + `embedding:true`
  status's `last_error` wording). **No row added or deleted, no row id / `Strategy-id` changed, the 8-row count,
  the `340 ≤ 400` arithmetic, the §11 map (21 rows), the `StoreError` taxonomy, the route table and the frozen
  `EngineSubsystems`/`HealthReport` shapes all unchanged.** *The `1177` and `340 ≤ 400` figures in this
  bullet are REMAND-2's arithmetic as it stood then; both are **superseded** by §9.5.5's post-red-phase
  register amendment (2026-09-22) — the U5 layer is now `355 ≤ 400` and the binary total `1192` — and are
  kept here as that round's dated record.* ***(POST-GREEN SPEC AMENDMENT, 2026-09-22 — scoping the "unchanged"
  claims in this bullet, so a reader does not re-derive them: the 8-row count, the §11 map (21 rows), the
  `StoreError` taxonomy, the 14-row route table and the frozen `EngineSubsystems`/`HealthReport` shapes ARE
  still unchanged, while the `340 ≤ 400` arithmetic is NOT (it is `355 ≤ 400`, measured `314 executed / 355
  caps`) and the binary total is `1192` — §9.5.5's post-red-phase register amendment is the authority. No row
  id, `Strategy-id`, row count, tag or HELD value moved.)***
- **U5 spec-gate REMAND-1 corrections in this file (2026-09-22, same-day one-pass remand; docs-only):**
  §9.5.5 (the contract table (1)'s failed-`Reachable`-build branch + the ordering clause's new failure-branch
  bullet; the contract table (2)'s new **call-count** row; the contract table (4)'s rewired failure row; the
  surface table's row #1 placement pin, row #5's "no new **store accessor**" restatement and the new row #7
  (the corpus-seeding calls); the new **"corpus seeding"** clause; the "must move in the same unit" table's
  per-line probes; the `P-IM-14`/`P-IM-15` rows; adjudication note 1's `R-L3` (v) addition and the
  layer-classification block), §9.5.4 (the heading's format rule + the eight U5 notes' deviation markers +
  the `P-IM-10`/`P-IM-11`/`P-SM-7` citation clauses), §6 (the boot-sequence step), §5.8 (the V-8.1 `Some`
  scoping note), §12 (this bullet) and §13 (the accessor bullet); plus the battery's `R-L3` PASS (iii)
  restate-and-park + new PASS (v) and its amendment item 3. **No row added or deleted, no row id or strategy
  id changed, the 8-row count and the `340 ≤ 400` arithmetic unchanged, and no new authority claimed.** *The
  `340 ≤ 400` figure in this bullet is that round's arithmetic as it stood then and is **superseded** by
  §9.5.5's post-red-phase register amendment (2026-09-22), under which the U5 layer is `355 ≤ 400` and the
  binary total `1192`; kept here as the round's dated record.*
- **U5 spec-gate REMAND-2 corrections in this file (2026-09-22, the second one-pass remand on §9.5.5 — docs-only;
  REMAND ROUND 2).** The re-review verified all 13 REMAND-1 findings and returned **11 new findings** (3
  MUST-FIX / 5 SHOULD-FIX / 3 NOTE); all are addressed in place, with the superseded text kept and annotated.
  The substantive changes: **(MUST-FIX 1)** the **READY-gate precedence** correction — `Absent`/`Unreachable`
  boots (and the failed-`Reachable`-build branch) are **non-READY**, so `mode=vector` yields **FS-8
  `EngineUnavailable` → 503 `engine_unavailable`**, and FS-14 `VectorIndexUnavailable` is asserted **only** on a
  **READY** store with `vectors: None` — applied in `P-SM-7`'s invariant/observable, its §9.5.4 note, the U5
  fail-state table's item 5, §5.9's FS-14 row + precedence bullet, and the fresh §9.5.5 REMAND-2 correction
  block below the register; **(MUST-FIX 2)** the live battery's `R-L3` PASS (v) probe re-pointed to **503
  `engine_unavailable`** (its amendment item 3(b) + a new item 4) and this file's adjudication note 1
  reconciled; **(MUST-FIX 3)** F2 §12's U5 note re-pointed to **probe (1) alone** (`:1195`), probes (2)/(3) at
  `:1228`/`:1237` staying on `boot_snapshot(false)` *(gate-8 re-read, 2026-09-22: the landed probe lines; the `:1209`/`:1218` spelling is the U3-time record)*, probe (4)'s `:1250` input already index-bearing *(gate-8 re-read; the `:1231` spelling is the U3-time record)*;
  **(SHOULD-FIX 4)** the same-unit texts that the two value edits falsify are now named (the assertion message
  at `tests/wire_conformance.rs:1205` and the comment at `:1191`) in §9.5.5's move table and in F2 §12;
  **(SHOULD-FIX 5)** §9.5.4's `P-IM-14` "Excluded" parenthetical replaced by §5.8's wording (`Reachable` +
  `vectors: None` ⇒ `Ready` + `vector:false`; V-8.1 asserted only for `Reachable` + `vectors: Some`);
  **(SHOULD-FIX 6)** the `EngineSubsystems`-literal census **re-measured and republished as one figure — 23**
  (5 + 5 + 10 + 3 + 0 over `wire_conformance`/`props_wire`/`props_gnosis_server`/`blind_u3`/
  `rag_query_integration`), in §5.8, §9.5.5's numeric-claims block and F2 §9.1's F7 note (which now agree);
  **(SHOULD-FIX 7)** the stale §5.9 anchors re-pointed by text with re-read line numbers (§5.9's **outcome table**
  `:1572-1583`, §5.9's **`Precedence (pinned)` bullet** `:1585-1592`, §5.9's **`Interlock with U5` bullet**
  `:1643-1645` — all three **re-read this pass**; the REMAND-2/3 triple `:1569-1580` / `:1582-1589` /
  `:1640-1642` is **superseded**, and with it the `:1633-1635` reading REMAND-3 already killed — REMAND-4
  MUST-FIX 1, 2026-09-22: the numbers shifted again as this file grew, and the `:1640-1642` target was
  §5.9's `Testable assertions the ruling yields` clause, not the interlock bullet) for `P-SM-7`
  and `P-IM-13`; **(SHOULD-FIX 8)** the contract table (2) call-count citation and the rows now agree —
  `P-SM-7`'s observable and §9.5.4 note carry the "no `embed` call on a path that never embeds" instance;
  **(NOTE 9)** the pre-U5 binary-total bullet (`310 + 400 + 127 = 837`) is annotated as the pre-U5 record and
  points at the post-U5 `1177` (***post-red-phase register amendment, 2026-09-22: the figure that bullet now
  points at is **`1192`** — the U5 layer was amended `340 ⇒ 355` — so read the `1177` in this NOTE as the
  round's dated record***); **(NOTE 10)** the battery's `R-L3` precondition now states what the controlled
  provider must return (a body making the parse fail; the status alone is not what maps); **(NOTE 11)** the new
  **`Degraded` + `embedding:true`** status, whose fixed `last_error` blames a subsystem it reports as wired, is
  recorded as a **residual** in §9.5.5's residual list (with `src/store/mod.rs:4226-4230`) and **not** filed as
  a tracker row. **No row added or deleted, no row id / `Strategy-id` changed, the 8-row count, the `340 ≤ 400`
  arithmetic and the `1177` binary total unchanged, every earlier clause kept and annotated, and no new
  authority claimed** (no new §11 row, `StoreError` variant, wire code, route, accessor or change to the frozen
  `EngineSubsystems`/`HealthReport` shapes). *The `340 ≤ 400` and `1177` figures in this bullet are
  REMAND ROUND 4's arithmetic as it stood then and are **superseded** by §9.5.5's post-red-phase register
  amendment (2026-09-22) — the U5 layer is now `355 ≤ 400` and the binary total `1192`; kept here as that
  round's dated record.*
- **U5 spec-gate REMAND ROUND 4 corrections in this file (2026-09-22, the third one-pass remand on §9.5.5 —
  docs-only, REMAND-4).** The re-review verified round 3's six fixes and returned **6 findings (2 MUST-FIX /
  1 SHOULD-FIX / 3 NOTE)**; all six are addressed in place, with **clause names carried ahead of every
  re-read line number** so a later shift degrades the numbers but never the target. **(MUST-FIX 1)** the
  §5.9 anchor triple is **re-pointed again by text** — §5.9's **outcome table** `:1572-1583`, §5.9's
  **`Precedence (pinned)` bullet** `:1585-1592`, §5.9's **`Interlock with U5` bullet** `:1643-1645` (all
  re-read this pass; the REMAND-2/3 numbers `:1569-1580`/`:1582-1589`/`:1640-1642` are kept as the
  superseded record) — in the REMAND-2/3 correction block below the register, the `P-IM-13` and `P-TP-5`
  cells, the §9.5.5 residual-list note's interlock citation, this §12 summary, and
  `docs/specs/engine-wire-contract.md` §16's rules 1 and 3 (each citing `p2` §5.9's `Precedence (pinned)`
  bullet); **(MUST-FIX 2)** `P-TP-5`'s code-site citation is re-pointed from §5.9's `Precedence (pinned)`
  bullet to §5.9's **`HyDEGenerationFailed`-unreachable clause** `:1617-1620` (and its §9.5.4 coverage
  note's `§5.9: reserved, unreachable` spelling likewise); **(SHOULD-FIX 3)** in
  `docs/specs/engine-wire-contract.md` §12, V-8.2's gloss is **scoped** to the provider-
  `Absent`/`Unreachable` boot (the fixtures whose producer probes are `tests/wire_conformance.rs:1228`/
  `:1237` — *gate-8 re-read, 2026-09-22: the landed lines; the `:1209`/`:1218` spelling is the U3-time record* — verified by text), and the failed-`Reachable`-build `Degraded` mask (`embedding:true`) is stated
  to have **no golden of its own** — asserted only by `P-IM-15` and the live row `R-L3`'s PASS (v);
  **(NOTE 4)** the corpus table gains a **key-uniqueness row**: a corpus with two nodes sharing one
  `NodeId` inside one document is **NOT a valid corpus**, excluded from the generated corpus, with the
  index pinning **one** key per embeddable node — cited from `P-TP-5`; **(NOTE 5)** the call-count row's
  **"no re-embed on a duplicate text"** phrase is **deleted**: **two nodes carrying the same text are two
  embeddable nodes ⇒ two `embed` calls and two keys** (per-node calls bind the provider's counter), so the
  call-count clauses can no longer contradict `P-TP-5`/`P-IM-10`; **(NOTE 6)** the corpus scope is **pinned
  explicitly** — the whole store, all wikis, `build_boot_vector_index` takes **no** wiki parameter, no row's
  corpus is per-wiki — and §9.5.5(4) gains a **cost-consequence row** stating that the pre-bind cost scales
  with the total corpus and that only the *leg* wiki-scopes, at read time. **No row added or deleted, no row
  id / `Strategy-id` changed, the 8-row count, every row cap, the `340 ≤ 400` arithmetic, the `1177` binary
  total, the seed/`row_seed`/stop-after-5/held-broken/one-pass-remand rules and every earlier clause are
  unchanged, and no new authority is claimed** (no new §11 row, `StoreError` variant, wire code, route,
  accessor or change to the frozen `EngineSubsystems`/`HealthReport` shapes). ***(POST-GREEN SPEC AMENDMENT,
  2026-09-22 — the enumeration of "unchanged" items in this bullet is scoped so it is not re-derived: the 8-row
  count, the ≤ 100/row rule, the ≤ 400/unit rule, the seed/`row_seed`/stop-after-5/held-broken/one-pass-remand
  rules, every earlier clause, the §11 map and the frozen types ARE unchanged, while the per-row cap VALUES are
  NOT (raised by §9.5.5's post-red-phase register amendment: caps `60/50/40/45/45/30/45/40 = 355 ≤ 400`,
  measured `314 executed / 355 caps`) and neither is the `340 ≤ 400` arithmetic or the `1177` binary total (they
  are `355 ≤ 400` and `1192`). No row id, `Strategy-id`, row count, tag or HELD value moved.)*** *The `340 ≤ 400` arithmetic and
  the `1177` binary total named in this bullet are REMAND ROUND 4's figures as it stood then and are
  **superseded** by §9.5.5's post-red-phase register amendment (2026-09-22) — `355 ≤ 400` and `1192`
  govern — while every *rule* this bullet names as unchanged (the 8-row count, the seed/`row_seed`/
  stop-after-5/held-broken/one-pass-remand set) remains exactly as stated and is confirmed intact by that
  amendment.* ***Disclosure (REMAND-4
  in-pass bookkeeping):*** while re-pointing `P-IM-13`'s code-site cell, and again while re-pointing
  `P-TP-5`'s, a literal-match probe **over-reached and consumed citation tails** and left one duplicated
  cell fragment in the §9.5.5 move table. Both cells were **restored and re-pointed in this pass**: the
  `P-IM-13` tail was rebuilt from this same pass's read of §5.9, of §9.5.5's **failure table** and of the
  REMAND-2 note (item 2 of the block below), the `P-TP-5` cell's `question-3`/`contract table (2)`/
  `VectorIndex`-shape citations were re-inserted verbatim from the pre-edit read, and the **duplicated
  fragment was deleted whole** (the move table is **back to its pre-over-reach size with no stray row**: it is
  **11 lines — a header line + a separator line + 9 data-row lines** — and ***the `12 rows` count this
  disclosure carried is CORRECTED HERE by the post-red-phase register amendment (2026-09-22): it was a
  **count slip only**, and an independent audit of the table re-read every cell and found every artifact,
  anchor and prose text intact — nothing is missing. With that amendment's added Implementer-obligation row
  the table now stands at **12 lines = the header + the separator + 10 data rows**.***). **No rule, row id,
  cap, count or pin changed and no row of the U5 register was touched**; the restoration is recorded here
  rather than left silent, and a proofreader should diff the two cells against this bullet.
- **U5 POST-RED-PHASE REGISTER AMENDMENT (2026-09-22, docs-only, ADDITIVE — a small register amendment after
  U5's red stage; REMAND-5/AMEND-1 in this file's series).** The spec gate returned **EMPTY (round 5)** and
  the red set landed; two spec-level items the TestWriter surfaced are fixed **in the spec**. **(A) caps
  amended and reconciled:** §9.5.5's execution plan now pins per-row `60/50/40/45/45/30/45/40 = 355 ≤ 400`
  (each ≤ 100) with **a cap as a MAXIMUM, not an expected count**, and the **measured layer 314 executed /
  355 caps** (`P-IM-10` 55/60 · `P-IM-11` 40/50 · `P-IM-12` 40/40 · `P-IM-13` 42/45 · `P-IM-14` 25/45 ·
  `P-IM-15` 27/30 · `P-SM-7` 45/45 · `P-TP-5` 40/40); the reason is the landed `P-IM-10` corpus (**55** cases
  = 9 corpus variants × 5 provider shapes + 10 `p == None` cases), against which the pinned 45 was
  unachievable while exceeding a cap is a guard failure. **The corpus is NOT reduced and no input class is
  dropped** — all **9** corpus variants (the `{}` / `{1 node}` / `{n nodes}` shapes, the `Some("")`-valued
  nodes, the duplicate text under two ids, the `value: None` mix, and the out-of-id-order store) crossed with
  the **5** provider shapes, plus the **10** `p == None` cases (`P-IM-10`'s §9.5.4 coverage note, `P-IM-11`'s
  corpus, `P-TP-5`'s adversarial set) stay as pinned: the cap is amended **upward to fit the landed corpus**
  rather than the corpus being cut to fit the cap. The pre-amendment `340` layer and `1177` binary
  total are **annotated superseded records** everywhere this file restated them — §9.5.5's execution plan +
  numeric-claims block + REMAND-2 correction block, §U1's binary-total bullet, §11's U5-update note and its
  API note, and §12's REMAND-1/2 bullets (the numeric-claims block that carries the `340 ⇒ 355` reading is
  §9.5.5's, not §9.5.4's — §9.5.4 holds the per-row coverage notes) — and the amended binary total is
  `310 + 400 + 127 + 355 = 1192`. **(B) the floating-point comparison rule is now contractual:** contract
  table (2) gains the *vector comparison / `NaN`* row (same `len()` + every non-`NaN` element identical
  bit-for-bit + `NaN` asserted only in the `NaN` position, `a[i].is_nan() == b[i].is_nan()`, **no** tolerance
  comparison), which is the reading `P-IM-12`'s "element-wise equal" and `P-TP-5`'s "verbatim" are asserted
  under; the alternative disposition (**excluding `NaN` from the generated corpus**) is **rejected
  explicitly**, because the `NaN`/`±∞` shapes are `P-TP-5`'s adversarial value. **(C) bookkeeping:** the move
  table's "**12 rows**" annotation is corrected in place to **11 lines = a header + a separator + 9 data
  rows** (a count slip only; an audit found every cell intact) and now stands at **10 data rows** after the
  new row below. **(D) an Implementer obligation in the same unit is recorded** (it is not in the move
  table's test-artifact set): the comment at `src/store/mod.rs:4211-4212` — *"the boot leaves
  `DerivedIndexes::default()` ⇒ `false` until U5's boot index build"* — becomes stale once U5's boot builds
  an index and must be reconciled **with U5's code**; the obligation is pinned as a row of §9.5.5's
  "must move in the same unit" table, and **this pass edits no `src/` file**. **No row id, kind,
  `Strategy-id`, tag or claim changed; the 8-row count, the ≤ 100/row rule, the ≤ 400/unit rule, the pinned
  seed/tags, stop-after-5, held/broken reporting and the one-pass remand rule are all intact; no new
  authority is claimed** (no new §11 row, `StoreError` variant, wire code, route, accessor or frozen
  `EngineSubsystems`/`HealthReport` change), and **no other file** (`docs/specs/engine-wire-contract.md`,
  the live battery, the trackers, `docs/specs/gnosis.md`, `src/**`, `tests/**`, `Cargo.toml`) is touched.
- **Bookkeeping-pass companions outside this file (2026-09-16):**
  `docs/specs/engine-wire-contract.md` **:29** (the status-block clause + the in-kind parenthetical at
  **:124**) — the dated U2/U3-authorization supersession marker, mirroring this file's §U1 marker; and
  `docs/specs/gnosis-gr-inbound-review.md` **Appendix B's status footer** — the same dated marker, so the
  record's own post-record note (`:18-22`) holds. Both historical clauses are kept, annotated, not deleted.
- **U3-landing additions in this file (2026-09-17, the U3 documentation pass — docs-only):** §U1's status
  block (the **U3 LANDED-GREEN** bullet), §5.8 (the heading + the LANDED note on the legacy literal), §9.5
  (the register-notes scope note, §9.5.2's **landed** bullet + the U3 notes/rows' re-pointed citations, §9.5.3's
  caps/seed/tag blocks, **§9.5.4's `P-SM-5` coverage note** — the mutation-interleaved positive control the
  TestWriter added is now named there), §11 (the U3 obligation row marked `AUTHORED ⇒ LANDED-GREEN`), §12 item 5
  (U3's landing), and the U3-drifted `file:line` citations throughout §5.3–§5.7/§10. **No pinned rule, no register
  row's substance and no row count was changed** — the pass records the landed unit, re-points citations and adds
  the two OPEN defect rows (`docs/defects.md` **P-8**/**P-9**, cross-filed in `docs/HANDOFF.md`).
- **Register-remand companions outside this file:** `docs/specs/engine-wire-contract.md` **§9.1** (the
  renumbered U3 cross-reference, F1) and **§12** (the new **V-15.1** query-path decode-error golden, F8) +
  its §14 conformance note; `docs/defects.md` (the OPEN `Ready`-vs-capability-axis row, F13).
- **Register-remand-2 (REMAND-2) companions outside this file:** `docs/specs/engine-wire-contract.md` **§7.1**
  (the `message` contract rule restated — verbatim, **may be empty**; `UnsupportedSchemaVersion(v)` the one
  guaranteed-non-empty render) + its V-15.1 binding note; `docs/specs/p2-gnosis-server-live-pending-battery.md`
  **§3.5** (the two named live rows **`R-L1`**/**`R-L2`** that are `P-TP-2`'s (β) and `P-IM-9`'s live half's
  homes) + its row-index note; the tracker/decision annotations of the U2/U3 authorization
  (`docs/decisions.md`, `docs/defects.md`, `docs/next-steps.md`, `docs/pending.md`, `docs/HANDOFF.md`) and the
  dated post-record note in `docs/specs/gnosis-gr-inbound-review.md`.
- **Companion F2 amendment:** `docs/specs/engine-wire-contract.md` §4.5 (`ragQuery` request/response
  wire), §4.6 (the change cursor), §4.7 (`GET /changes` SSE), §7.1 (the transport decode-error body),
  §9.1 (subsystem-flag capability semantics), §11 (the map stays 21 rows), §12 (the amended V-8
  vectors), §16 (the FS-3 vs FS-13/14/15 ruling).
- **Boundary C4 (as-of-N reconstruction):**
  `docs/research/astrographer-engine-shell-boundary.md:86` — ENGINE-owned, a journal-retention concern,
  **not** a query route.
- **Base wire layer:** `docs/specs/engine-wire-contract.md` (F2 — the `Envelope`, the error
  codec, the §11 map, decode-then-validate, the golden vectors V-1..V-9, the SSE framing).
- **CRUD wire layer:** `docs/specs/p1a-document-crud-wire.md` (P1a — the 11 §4.1 wire shapes,
  the `ENGINE_ENDPOINTS`/`ENDPOINT_*` constants, the §6.2 request-decode outcome table, the RBAC
  `caller` shape, the golden vectors V-10..V-14). **U1 does not change this file's frozen shapes.**
- **Register format precedent:** `docs/specs/7-2-wire-property-register.md` (the F2 register this
  unit's §5.x register mirrors). **U1 adds no row there either** (see §9.1 of F2).
- **PBT:** the §5.x register (authored HERE) + the TestWriter's executed property layer.
- **Conformance tests (TestWriter):** `tests/gnosis_server_conformance.rs` (or an extension of
  the existing conformance suites) + the e2e transport test.

**Owned by a later unit (not P2, per §2):**
1. The shell-side client (A1) — consumes the same paths + shapes.
2. The graph/fact/consistency/RAG-companion endpoints (P1b–P1e — deferred follow-ons).
3. The RBAC **enforcement** semantics (the engine is the enforcer; P2 threads the `caller` only) —
   RECONCILED 2026-09-22 (`RBAC-DOC-DRIFT`): the engine pins the credential **SHAPE** + its
   decode-layer **presence check** only; the authority mapping and the deny are **SHELL**-side — see
   `GNOSIS-RBAC-EDIT-ENFORCEMENT` / `docs/HANDOFF.md` §8-RBAC. The historical clause stands as written.
4. **(U1)** **U2** — the query POST contract: the shared envelope-strict query decoder, the mode/`expand`/
   `compression` token resolver (wire-layer, one per token family), the canonical-§4.5.2 `filters` mapping, the transport JSON
   decode-error body, and the SSE fail-state status pin (and its own typed §5.x rows).
   **(U2 AUTHORIZED 2026-09-16; its typed rows are AUTHORED in §9.5.1 — `P-IM-4`/`P-IM-5`/`P-IM-6`/`P-SM-4`/
   `P-TP-2`/`P-TP-3`/`P-TP-4`, 7 rows — with the execution plan in §9.5.3. **UPDATE (2026-09-17): U2's code has
   LANDED REALIZED-GREEN** — `src/wire/query.rs` + the `src/wire/mod.rs`/`src/lib.rs` re-exports + the bin
   wiring; `cargo test` 582/0, all 7 rows HELD with the layer's 400 executed cases, the blind set 24/24 and the
   live row `R-L1` PASSED. The historical clause read "The code is still owed: this spec pass is docs-only.")**
5. **(U1)** **U3/U5** — status honesty (`reranker: false`, `vector: false` until U5, flags derived from
   real state; any additive capability-vs-index signal lands on **`HealthReport`**, **never** on
   `EngineSubsystems` — §5.8/F7) + the boot vector-index build.
   **(U3 AUTHORIZED 2026-09-16; its typed rows are AUTHORED in §9.5.2 — `P-IM-7`/`P-IM-8`/`P-IM-9`/`P-SM-5`/
   `P-SM-6`, 5 rows (the three IM ids were renumbered from `P-IM-5/6/7` by the register remand's F1 ruling —
   §9.5.3.1) — with the V-8.1/V-8.2 literal edit owed in the same unit as the flag change, §5.8/F2
   §12. **UPDATE (2026-09-17): U3's code has LANDED-GREEN** — the read-time derivation in `get_engine_status`
   (`src/store/mod.rs:4193-4232`), the lib seam `boot_wiring` (`src/lib.rs:69-110`) and the bin boot
   (`src/bin/gnosis_server.rs:378-408`), with the V-8.1/V-8.2 literals amended in the same unit
   (`tests/wire_conformance.rs:1061-1112`); all five rows HELD (127 executed ≤ 400) — **the "literal edit owed in the same unit" clause above is discharged, not outstanding** — the blind set 13/13 and the
   live battery 9/9 (incl. `R-L2`). **U5 remains NOT authorized**: its owed row in §11 stays id-less — the boot
   index build, and therefore the `vector:true` live value, is still unbuilt.) — ***U3-time record; superseded for U5 below.***
   **UPDATE (2026-09-22): U5 IS AUTHORIZED and its spec gate has landed — the U5 clause above is the
   U3-time record.** The user's go-ahead (`docs/specs/gnosis-grq-inbound-review.md` §14 POST-RECORD
   UPDATE 1, **Q3 = "(B) U5 ONLY"**) plus this pass's contract + typed register in **§9.5.5** (8 rows:
   `P-IM-10`, `P-IM-11`, `P-IM-12`, `P-IM-13`, `P-IM-14`, `P-IM-15`, `P-SM-7`, `P-TP-5`; execution plan in
   §9.5.5, per-row coverage notes in §9.5.4, the "must move in the same unit" table in §9.5.5). Its
   **code is OWED**, together with the same-unit edits §9.5.5 names: the V-8.1 `"vector"` literal + the
   `honest_ready_subsystems()` fixture + the `boot_wiring_couples_to_the_derived_read` probe inputs
   (`tests/wire_conformance.rs:713-723`, `:1093-1097`, probe (1) at `:1191-1206` — the pre-REMAND-2
   `:713-722`/`:1093-1111`/`:1192-1239` spelling is the literal-body/probe-block reading this pass
   re-measured; **gate-8 re-read, 2026-09-22 — the landed anchors are the fixture `:718` (doc comment `:708-717`), the V-8.1 literal `:1109` (message `:1110-1111`), probe (1) `:1208-1223`, probes (2)/(3) `:1228`/`:1237`, probe (4) `:1244-1256`; the citations above stand as the U3-time records**), the two U5-labelled notes in
   `docs/specs/engine-wire-contract.md` (§9.1 + §12), and the live battery's `R-L2` criterion correction +
   new `R-L3` row (`docs/specs/p2-gnosis-server-live-pending-battery.md` §3.5). A TestWriter may now derive
   U5's red set from §9.5.5 alone. **POST-GREENS GATE-8 RECONCILE (2026-09-22, docs-only — additive; the "code is OWED" clause above is the pre-green record).** U5's code has **LANDED-GREEN** (build seam `build_boot_vector_index`, body `src/store/mod.rs:5369-5414`, name re-exported `src/lib.rs:126`; the bin's boot wiring `src/bin/gnosis_server.rs:381-437`, build call at **`:414`** — the `:404` spelling in the battery's `R-L3` cell was stale and is re-pointed there — composed snapshot `:416-419`, pinned `boot_wiring` order `:425-432`, wiring `:433-437`; the derivation-site comment `src/store/mod.rs:4210-4215` reconciled), and its whole gate chain has run: **`cargo test` 655 passed / 0 failed** (serial; baseline 604 → +24 U5 in-crate → +27 blind), green **with and without** `GNOSIS_SERVER_OLLAMA_URL`; the U5 property layer **314 executed / 355 caps ≤ 400** (8 rows HELD); the blind set **27/27 GREEN / 0 RED / 6 NOT-VERIFIED**; the live battery's `R-L3` **(i)/(ii)/(iv) PASSED live** with (iii)/(v) parked and `R-L2`'s amended `vector:true` cell re-verified live; `fmt`/`clippy`/`build` clean. **Nothing pinned moved** (the 7+5+8 §9.5 rows, the tags, the 21-row §11 map, the 23-literal census and the 14-row route bijection are intact); **the failed-`Reachable`-build branch is lib-level only — its live criterion is PARKED, not live-verified**; `U5-ADV-1`…`U5-ADV-5` and `P-9`'s fired trigger stay **OPEN** in `docs/defects.md`, and the PBT audit's **T1–T10** list is an undis-charged TestWriter obligation. Review record: `archive/reviews/2026-09-22-u5-boot-vector-index-doc-review.md`. **U4 remains HELD** (it needs `SHELL-2`) and its §11 row stays owed and
   id-less.)**
6. **(U1)** **U4** — the change cursor **accessor** + the `append_journal`/`JournalEntry` `kind` + ids
   extension + `GET /changes` + the paginated wiki-scoped read routes + the `route_bijection()`/
   `P-IM-3` extension (all in U4's own unit, per §5.2's amendment rule), plus `SHELL-2`'s bounded
   consumer cache.
   ***(U6 note, 2026-09-22 — ADDITIVE; U4 is still HELD and its owed §11 row is still owed and id-less.)***
   The **U5 honesty/freshness unit is now open as U6** (§9.5.6 — **AUTHORIZED 2026-09-22**, its spec gate
   authored there, code OWED): it makes the `vector` flag honest (`snapshot().vectors.is_some() ∧
   snapshot().epoch == epoch()`) so a post-boot write yields FS-14 ⇒ 503 instead of a silent 200, adds the
   boot's stderr build diagnostic (`U5-ADV-4`) and a real provider request timeout (`U5-ADV-2` — whose
   **observable is U6's second lib seam**, `PROVIDER_REQUEST_TIMEOUT` / `provider_client`, §9.5.6's surface #6;
   REMAND-1, 2026-09-22). **U6 adds no
   route, so U4's amendment surface (§5.2's growth invariant, the 14-row table, `route_bijection()`) is
   untouched**; U6 **reads** `epoch()` and changes **no** cursor/journal semantics. **`U5-ADV-3`**
   (the `Degraded` + `embedding:true` + blame-`embedding` `last_error` contradiction) and **`U5-ADV-5`**
   (the lock-`unwrap` panic class) are **explicitly out of U6's scope** and stay OPEN in `docs/defects.md`;
   **`P-9`** (the non-finite-score panic) stays the F2 wire foundation's, with U5's verbatim-vector clause
   **not** patched (§9.5.6's out-of-scope table).***
7. Bind-loopback + auth/TLS policy (recorded shell-owned, as in F2/P1a).

---

## 13. What the spec does NOT do (constraints honored)

- This is a **TDD unit spec** — it includes the §5.x Property register (PBT gate) but does
  **NOT** author the property tests (the TestWriter does) and does **NOT** author implementation.
- It does **NOT** touch `src/` or `tests/` — it writes **only** the spec file.
- It does **NOT** change the F2 error codec or the §11 map (21 rows) — it reuses them verbatim
  and qualifies the "no new statuses" claim to the `StoreError` taxonomy (NEW-2).
- It does **NOT** add new `StoreError` variants or new §11 rows.
- **(U1)** It does **NOT** add a new route, a new endpoint or a new property row: the 14-row table is
  the **current state at U1**, and the routing contract is now a **growth invariant** (§5.2) whose
  future growth is owned by the amendment unit that adds the route (U4 for `GET /changes`).
- **(U1)** It does **NOT** refuse-and-replace anything in the §11 map: the transport decode codes
  (§5.5) are a **transport-level** vocabulary outside the map, disjoint from `wire_code()`.
- **(U1)** It does **NOT** change `docs/specs/gnosis.md` (the canonical contract) — the FS-3 vs
  FS-13/14/15 wording, **§4.5.1** and **§4.6.1's `ragQuery` throw column including its `hyde` clause**
  (`docs/specs/gnosis.md:872`, `:1037`, `:1052-1055`), the canonical **§4.5.2 `filters` shape**
  (`:692-694`) and the `ragQuery` request/response body are reconciled **upstream** via the
  `docs/HANDOFF.md` rows, while the engine-side resolution is pinned here and in F2. **U1's own
  engine-side pins that are new states rather than canonical restatements are labelled as such**
  (the `expand` 400, §5.4; the `changes` surface, §5.6) and each carries its own upstream ask.
- **(U1)** It does **NOT** author the U2/U3/U4/U5 specs and does **NOT** change `src/`, `tests/` or
  `Cargo.toml`; the F2 §9.1 status amendment deliberately **precedes** its code (U3), with the V-8
  literal edit landing in the same unit as the flag change.
- It does **NOT** change the engine lib's dependency set — the server framework (axum/hyper/
  tower) is scoped to the bin only.
- **(U5, 2026-09-22)** It does **NOT** add a route, an endpoint, a `StoreError` variant, a §11 row, a wire
  code, a `HealthReport` field or an `EngineSubsystems` field; it does **NOT** change the error map, the
  frozen canonical types, the route table (14 rows) or the §11 map (21 rows). U5's contract is **additive**
  to §5.8's semantics: it pins **what the boot puts in the derived snapshot**, and its flag effect is the
  consequence of the **unchanged** read-time derivation (`src/store/mod.rs:4213`).
- **(U5, 2026-09-22)** It does **NOT** touch the change cursor, any route, the paged reads, `GET /changes`,
  the journal or any persistence/durability surface (U4 is **HELD**; the durability design unit is **not
  authorized** — `docs/specs/gnosis-grq-inbound-review.md` §14 **Q2**), and it does **NOT** authorize a
  rebuild vehicle (U5 builds **once, at boot**).
- **(U5, 2026-09-22)** It does **NOT** add an env var or a CLI flag, and it does **NOT** assert the set of
  configuration reads the bin performs — a future knob is a **register obligation** (§9.5.5's question-5
  clause), and U5 owes none.
- **(U5, 2026-09-22)** It does **NOT** edit `docs/specs/gnosis.md` (canonical), the trackers
  (`docs/decisions.md`/`docs/defects.md`/`docs/pending.md`/`docs/next-steps.md`/`docs/HANDOFF.md`),
  `src/**`, `tests/**` or `Cargo.toml`; and it does **NOT** run the trio (the TestWriter/Implementer own the
  red→green and the gate).
- **(U5, REMAND-1, 2026-09-22)** It does **NOT** add a store **accessor**, a `pub` field or a test-visible
  corpus hook — the build's only non-accessor read is its **own module's** private `Store.shards`/
  `StoreShard.docs` (its body lives in `src/store/`, its name is re-exported from `src/lib.rs`), and every
  corpus in §9.5.5's rows is **seeded through the public `RagStore` surface** (§9.5.5's "corpus seeding"
  clause). The correction pass changes **no** row's claim, adds **no** route, variant, wire code, §11 row or
  frozen-type field, and the parked live criterion (`R-L3` PASS (iii)) is **parked with its reason recorded**
  rather than replaced by an assertion the bin cannot satisfy.
- **(U6, 2026-09-22)** It does **NOT** add a route, an endpoint, a `StoreError` variant, a §11 row, a wire
  code, a `HealthReport`/`EngineSubsystems` field, a store accessor, an env var or a CLI flag; it does **NOT**
  add, assume or authorize a **rebuild vehicle** (the boot-time `swap_snapshot` stays the only installation
  vehicle), and it does **NOT** touch the change cursor, the journal, the paged reads, `GET /changes` or any
  persistence/durability surface (U4 HELD; the durability direction NOT ACTIVE). Its flag effect is the
  **derived predicate** `snapshot().vectors.is_some() ∧ snapshot().epoch == self.epoch()` read at the
  **unchanged** derivation site (`src/store/mod.rs`), and its failures stay inside the closed 21-row §11 map
  (FS-14 `vector_index_unavailable` for a stale/unbuilt index on a READY store; FS-8 for a non-READY store;
  FS-13 for a fresh index with no provider). **It does NOT reconcile `U5-ADV-3`, does NOT touch the
  `U5-ADV-5` lock class, does NOT patch U5's verbatim-vector clause or any `P-9` path, and it does**
  **NOT** edit the trackers, the canonical `docs/specs/gnosis.md`, `src/**`, `tests/**` or `Cargo.toml` —
  its out-of-scope list and its five open questions are recorded in **§9.5.6** for the supervisor (§9.5.6's
  out-of-scope table and open-questions block).
  ***(REMAND-1 additions, 2026-09-22.)*** **The freeze wording vs the derivation change (NOTE 12/NOTE 17):**
  "frozen" here and in §5.8's `Where the additive signal may land (F7; pinned)` clause (`:1623-1642`) is a freeze
  of the type's **SHAPE**, not of the flags' **values** — U6 moves the `vector` flag's **value** and nothing
  else: six `bool`s stay, no field is added/lost/re-typed, and `EngineSubsystems { … }`'s **twenty-three** struct
  literals in `tests/` (the §5.8 census) are **not** broken by U6. **Two lib-visible items are added and are
  declared here so "no new accessor" cannot be misread:** (i) the freshness predicate `vector_index_is_fresh`
  (**surface #1**) and (ii) the provider-client construction seam `PROVIDER_REQUEST_TIMEOUT` + `provider_client`
  (**surface #6**, which `P-TP-6` needs because its constant was otherwise unnamable inside the `[[bin]]`). Both
  follow the F11 lib-seam precedent (`boot_wiring`, `src/lib.rs:79-110`) for a `[[bin]]`-only path; **neither**
  adds a store accessor, a `pub` field, a test-visible corpus hook, a route, a wire code, a `StoreError` variant,
  a §11 row, a dependency (`reqwest` is already a lib dependency, `Cargo.toml:17`) or a configuration read.***

