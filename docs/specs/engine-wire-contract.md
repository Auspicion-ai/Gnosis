# §7.2 F2 — Engine wire contract (mechanism-agnostic codecs + encoding layer)

- **Unit:** §7.2 F2 — engine transport/API reconciliation (the last OPEN unit in
  `docs/next-steps.md`).
- **Status:** **CONTRACT (REALIZED-GREEN — landed 2026-09-09)** — this is the behavior
  contract the F2 unit's TestWriter derives its red set from, and the wire schema the
  Astrographer shell will implement identically. It pins the codec + validation
  + SSE + health + envelope layer **only**. It does **not** build a server, does
  **not** add runtime dependencies, and does **not** mutate the frozen §4.1/§4.5
  types. The F2 unit shipped green (**360 total tests**) matching §1–§13.
- **Gate:** proposal-review **PASSED** — Architecture A1 (`7-2-f2-review.md`,
  decision `F2-WIRE-CONTRACT-A1`). Delegation is complete; the unit is implemented.
- **Contract cross-refs:** `docs/specs/gnosis.md` §4.6.1 (the retrieval trio),
  §4.1.5 (`RagStore` persistence seam — the wire covers the **retrieval trio**,
  not CRUD), §4.1.4/FS-4 (`ConflictError` = HTTP 409), §4.3.4 (audit), §6
  (FS-1..FS-26 for the code map); `docs/specs/7-2-f2-review.md` (the verdict +
  deliverable set + scope guardrails); `docs/decisions.md` `F2-WIRE-CONTRACT-A1`.
- **Date:** 2026-09-09. **Author-role:** spec_writer.
- **Scope:** `src/wire/` + the re-export surface in `src/lib.rs`. Companion PBT
  register: `docs/specs/7-2-wire-property-register.md`.
- **U1 status note (2026-09-16):** amended by **U1** of the inbound GR-1..GR-9 set (the P2/F2
  contract amendment, authorized by the gate-1 go-ahead in `docs/specs/gnosis-gr-inbound-review.md`
  §Go-ahead record) — the additions are **§2** (U1-pinned, code-bearing units named), **§4.5** (the
  `ragQuery` request/response wire), **§4.6** (the one wire-visible change cursor), **§4.7** (the
  `GET /changes` SSE feed), **§5** (the §11 map stays 21 rows), **§7.1** (the structured transport
  decode-error body), **§9.1** (subsystem-flag capability semantics), **§12** (the amended V-8 + the
  new V-15..V-19), **§13/§14/§15** (assertion guide, cross-refs, red-set notes) and **§16** (the
  FS-3 vs FS-13/14/15 retrieval-semantics ruling). **Contract-only**: no `src/`, no `tests/`, no
  cargo; the implementing units **U2/U3/U4/U5 are NOT authorized** and each code-bearing statement
  names its unit. **Zero new register rows** land in U1 (see §9.1). ***SUPERSEDED (2026-09-16) as to
  the two code-bearing units: U2/U3 AUTHORIZED (2026-09-16; registers authored in `p2` §9.5.1/§9.5.2);
  U4/U5 NOT authorized; **(2026-09-17) BOTH LANDED — U2 code LANDED, U3 LANDED-GREEN (the §U3 status note
  below).*** The status clause now reads **U2/U3 AUTHORIZED (2026-09-16; registers
  authored in `p2` §9.5.1/§9.5.2); U4/U5 NOT authorized** — U2's and U3's code has since
  **landed** (U2 2026-09-17; **U3 LANDED-GREEN 2026-09-17**); U4
  (change cursor + paged reads + `GET /changes`) and U5 (boot vector-index build) stay **NOT
  **LANDED-GREEN 2026-09-22 (the §U5 status note below); U4 NOT authorized** — U2's and U3's code has since
   **landed** (U2 2026-09-17; **U3 LANDED-GREEN 2026-09-17**); U4 (change cursor + paged reads +
   `GET /changes`) remains **NOT authorized** (it needs the consumer-side `SHELL-2`), and no `Property-id` is
   claimed for it. ***(U5 update, gate 8, 2026-09-22 — the clause above that also named U5 is the
   historical record: U5 was authorized on 2026-09-22, its typed register landed in `p2` §9.5.5 and the
   unit is LANDED-GREEN; only U4 remains unauthorized.)*** The superseded clause above stands as the
  historical record of what *the U1 amendment itself* authorized; the same dated marker, with the same
  reading, is carried by this file's §2 guardrails block (`:187-198`) and by `p2` §U1's status
  block (`docs/specs/p2-gnosis-server.md` §U1). *(**Doc-review 2026-09-17:** the clause here cited "this
  file's §14 note (`:1415-1416`, `:1428-1432`)" — those lines are now §12's V-16/V-17 framing and carry no
  authorization marker, so the reference was stale and is re-pointed to the §2 block that actually carries
  the marker; the pre-edit line numbers are kept here as the drift record.)* **Zero new register rows** land in U1 (see §9.1).
- **U2 status note (proofread pass, 2026-09-17; annotation only — no pinned content changed).** **U2's code has
  LANDED** (`src/wire/query.rs`, `src/wire/mod.rs`/`src/lib.rs` re-exports, `src/bin/gnosis_server.rs`'s
  shared decode-error renderer, the SSE pre-stream `error` frame and the checked POST handler): verified
  **`cargo test` 582 passed / 0 failed**, the U2 property layer 400 cases HELD, the blind set
  `tests/blind_u2_query_post_greens.rs` **24/24**, and the live battery's `R-L1` PASSED live. So this file's
  §4.5/§7.1 **"code lands in U2" / "U2-time"** markers now read **LANDED** (their historical wording is kept),
  the moved code-line citations have been re-pointed where they describe the landed state, **F2 §4.4 now pins
  the SSE media type (`Content-Type: text/event-stream`, F-2)**, and **§13's `validate_rag_result` cell names
  its two landed arms** (a traceless result is not representable — `MissingTrace` is `decode_rag_result`'s, not
  the validator's). See `docs/specs/p2-gnosis-server.md` §U1's U2-landed bullet for the full record, and
  `docs/defects.md` **P-5** (the pre-existing `text/plain; charset=utf-8` SSE error-frame divergence, **OPEN**) and
  **P-6** (the reported wrongly-typed-`n` divergence between §4.5/§13's clause and the landed decoder —
  **CLOSED 2026-09-16**: the supervisor ruled the clause correct and the decoder was implemented to it,
  `object_option`, `src/wire/query.rs:188-202`; the companion wording defect **P-7** is CLOSED docs-side).
  **POST-GREENS DOC-REVIEW (2026-09-16, docs-only; nothing pinned changed):** the documentation reviewer
  reconciled this file's U2-bearing sections (§4.4/§4.5/§7.1/§9.1/§12/§13/§14) against the landed tree — the
  stale `src/wire/query.rs:155` citation (⇒ **`:157`**, §4.5), the two headings' "code lands in U2" markers
  (now **LANDED**), the pre-U2 "today"-descriptions of the non-object-payload row (§7.1/§13, now the landed
  `invalid_envelope` outcome), §7.1's `request_decode_code` marker (now named and landed, `:339-350`) and
  §14's V-15..V-19 landing state (V-15/V-15.1/V-18 landed with U2; V-16/V-17/V-19 remain U4's;
  `health_report_shape_frozen` — **LANDED with U3** (2026-09-17; `tests/wire_conformance.rs:1263`), not
  merely owed) — **no vector literal, rule or row was changed.** Review record:
  `archive/reviews/2026-09-16-u2-query-post-contract-doc-review.md` (gitignored provenance).
- **U3 status note (proofread pass, 2026-09-17; annotation only — no pinned content changed).** **U3's code has
  LANDED-GREEN.** The six `EngineSubsystems` flags are now **derived at read time** in `get_engine_status`
  (`src/store/mod.rs:4193-4232`), the lib-visible boot seam `boot_wiring(provider: BootProvider, snapshot:
  &DerivedIndexes) -> (EngineState, EngineSubsystems)` is landed (`src/lib.rs:69-110`), the bin's boot applies
  **only** the returned `EngineState` plus its own wiring — never a flag mask (`src/bin/gnosis_server.rs:378-408`
  at U3; **after U5 the boot block is `:381-437`** — the build at `:414` and the wiring at `:433-437`; gate 8,
  2026-09-22),
  the V-8.1/V-8.2 golden literals are amended in the same unit (`tests/wire_conformance.rs:1061-1112`, with
  `boot_wiring_couples_to_the_derived_read` at `:1129`) and the conformance assertion `health_report_shape_frozen`
  is landed (`:1263`). Verified: **`cargo test` 604 passed / 0 failed** (serial), green **with and without**
  `GNOSIS_SERVER_OLLAMA_URL` set; the five U3 property rows HELD (`P-IM-7` 25 / `P-IM-8` 26 / `P-IM-9` 26 /
  `P-SM-5` 25 / `P-SM-6` 25 = **127 executed ≤ 400**); blind set **13/13**; live battery **9/9** (incl. `R-L2`);
  `cargo fmt --check` exit 0, `cargo clippy --all-targets` 0 warnings, `cargo build` clean. So this file's
  §9.1 and §12's **"U3 must derive it" / "the literals are pre-U1 masks" / "U3 makes the live mask honest"**
  markers now read **LANDED** (their historical wording is kept in place), and the code-line citations that
  drifted when U3's in-tree comment insertions shifted them have been re-pointed (§9.1's table, §12's V-8.2
  reachability bullet). Two U3-time findings are filed as **OPEN** rows in `docs/defects.md` — **post-boot
  provider loss is invisible to `/engine/status`**, and **`encode_result`'s `expect` panic path** — and neither is
  a U3 regression (every pinned U3 row passes).
- **U2 red-phase ambiguity pin (2026-09-16; docs-only, ADDITIVE — no pinned rule rewritten, no register row
  added).** This file's two annotations, both cross-referencing the pin stated **once** in
  `docs/specs/p2-gnosis-server.md` §5.3 ("**THE TWO-LAYER READING OF THE `absent ⇒` COLUMN**"): **(a)**
  §4.5's "Absent `mode` ⇒ `Flat`" and "wrongly-typed ⇒ ABSENT ⇒ documented default" bullets now state the
  layers explicitly — the **decoder's** output for an absent/wrongly-typed value is the store's own
  `Option` field `== None` (`Some(Flat)` is never a decoder output; only a **well-formed vocabulary
  string** yields `Some(token)`), while `Flat`/`10`/`'none'` are the **engine's query-time** readings
  (`mode.unwrap_or(QueryMode::Flat)`, `src/store/mod.rs:4060`, `:4168`); **(b)** §7.1's new
  "which part of the body is pinned" note + §12's **V-15.1** binding note state that the
  **code/status/header/keyset** are the byte-binding pins and the **`message` value is the implemented
  renderer's own output** (the golden asserts the structure **plus** the derived string, not a
  hand-written literal). §7.1's `message` contract rule (the four string-carrying variants
  verbatim/may-be-empty; `UnsupportedSchemaVersion(v)` non-empty) and **V-15's own literals are
  unchanged**; `p2` §5.5's rule is unchanged and its companion record is `p2` §U1's ambiguity-pin bullet.
- **U1 status note — REMAND (2026-09-16):** the spec-gate reviewer returned **4 MUST-FIX + 7 SHOULD-FIX
  + 6 NOTES** on U1's first pass; **all 17 are addressed in this file and in
  `docs/specs/p2-gnosis-server.md` in place** (the finding-by-finding disposition is reported with the
  remand). This file's substantive remand changes: **(F1)** §4.5's `filters` casing rule now pins the
  **canonical §4.5.2 shape** (`docs/specs/gnosis.md:692-694`) instead of the store's serde casing, with
  the store casing labelled the **response/audit** shape and `filters` **not** a
  `serde_json::from_value::<QueryAuditFilters>` pass-through (a silent all-`None` — the GR-2 defect
  class); **(F2)** the `mode`/`expand`/`compression` token check is located in the **U2 wire resolver
  only** (the fields are typed enums, so it cannot be a `validate_rag_options` arm), and the `expand`
  400 is annotated a **new state created by this amendment** whose addition is requested upstream;
  **(F3)** §16 extends the narrowing to **§4.6.1's `ragQuery` throw column (`:872`) and FS-13's
  `hyde: true` clause (`:1037`)**; **(F4)** the SSE mode fail-state's status is pinned **400** (§13);
  **(F6)** V-16/V-17 are labelled **codec-level vectors over a synthetic `JournalEntry`**; **(F7)** the
  additive status signal is scoped to **`HealthReport`**, and frozen canonical `EngineSubsystems`
  **MUST NOT** gain/lose/re-type a field; **(F9)** the refused cursor semantics are narrowed to **no
  request-side as-of-N / point-in-time** semantics, with a U4 page token defined as a **resume token**;
  **(F10)** §4.4's closed `<type>` set is annotated as the **retrieval** family; **(F11)** §4.7's `kind`
  gains unknown-token tolerance + the rename binding; plus the NOTES **F12** (`UnknownMethod` arm),
  **F13** (the non-object-payload row is U2-time), **F14** (the cursor is not a
  coherence/validation token), **F15** (`Content-Type` exactly `application/json`), **F16** (§2's two
  superseded guardrails) and **F17(ii)/(iii)** (the key count; the `P-SM-3` citation). **No pinned
  content that is still true was deleted** — superseded statements are marked in place.
- **U1 status note — REMAND-2 (2026-09-16):** the verification reviewer confirmed **F1–F17** (F5
  PARTIAL) and returned **one MUST-FIX (N1) + two SHOULD-FIX (N2, N3) + three NOTES (N4–N6)**; **all six
  are addressed in this file and in `docs/specs/p2-gnosis-server.md`**. This file's changes: **(N1)**
  §4.5's `expand`/`compression` paragraph no longer claims the SSE `error`-frame outcome — **only `mode`
  has a POST *and* an SSE outcome**, the two others are **POST-payload-only** and the SSE params
  `?expand=`/`?compression=` are **ignored ⇒ the option's default, never a 400** (the same split is
  pinned in `p2` §5.4, and §13's token row + SSE-parity state are aligned to the mode-only claim);
  **(N2)** §13's wrongly-typed corpus gains `expand:5`/`compression:null` (with `p2` §5.3 as the
  canonical corpus) and the "only a **string** reaches the token check" rule is stated; **(N6)** §9.1's
  F7 blast-radius citation is corrected to **nine** `EngineSubsystems { … }` literals (the pre-remand
  "ten" and its `tests/props_wire.rs:282` citation were wrong — that line is the
  `fn all_true_subsystems()` **signature**); **(N3)** the `docs/HANDOFF.md` addendum now states row 51's
  relationship explicitly (that file's change). **(N4)/(N5)** are `p2` §5.3/§10/§11 pins (option
  reachability; `filters:{}` is present/valid/honored, and the wire surface exposes no
  `nodeKind: 'community'`). **Zero new register rows**: the §9.1
  row relationship and this file's register claim are unchanged, and no pinned content still true was
  deleted.
- **U5 spec-gate REMAND ROUND 4 note (2026-09-22; docs-only, ADDITIVE — no rule, row, type or map changed).**
  The third one-pass remand on `p2` §9.5.5 returned 6 findings (2 MUST-FIX / 1 SHOULD-FIX / 3 NOTE); this
  file is touched by two of them. **(MUST-FIX 1)** §16's **rule 1** and §16's **rule 3's narrowing clause**
  (§4.6.1's `ragQuery` throw column) re-point the `p2` §5.9 anchor they cite to **§5.9's
  `Precedence (pinned)` bullet** with its **re-read line number `:1585-1592`**
  (`docs/specs/p2-gnosis-server.md`); the `:1582-1589` spelling each rule previously carried
  is kept in place as the **superseded record** (it started on two rows of §5.9's outcome table and ended
  mid-bullet). **(SHOULD-FIX 3)** §12's **V-8.2** reachability bullet and the U5 note's V-8.2 clause now
  **scope** V-8.2's gloss to the provider-`Absent`/`Unreachable` boot — the producer probes verified by text
  at `tests/wire_conformance.rs:1228`/`:1237` — *gate-8 re-read, 2026-09-22: the landed probes (2)/(3) are at
  those lines; the `:1209`/`:1218` spelling this note carried is the U3-time record* — and state that U5's failed-`Reachable`-build `Degraded` mask
  (`vector:false`, `embedding:true`, `reranker:false`, the same `lastError`) has **no golden of its own**: it
  is asserted only by `p2`'s `P-IM-15` and by the live battery row `R-L3`'s PASS (v). The **values** V-8.2
  pins are unchanged; only the universal **phrasing** was over-claimed. **No vector literal, no rule number,
  no register row, no §11 code, no type and no key set changes**, and the `tests/wire_conformance.rs`
  fixture-comment reconciliation the scoping implies is a **TestWriter obligation** (recorded in `p2`'s
  REMAND-4 status bullet), not an edit this docs pass makes.
- **U5 status note (gate 8, 2026-09-22; annotation only — no pinned content changed).** **U5's code has
  LANDED-GREEN** and its whole gate chain (adversarial, blind-greens, live battery, doc review) has run:
  the build seam is `build_boot_vector_index` (body in `src/store/mod.rs:5369`, name re-exported at
  `src/lib.rs:126`), the bin's boot wiring is `src/bin/gnosis_server.rs:381-437` (the build call at `:414`,
  the composed snapshot at `:416-419`, the pinned `boot_wiring` order at `:425-432`, the wiring at
  `:433-437`), and the derivation-site comment at `src/store/mod.rs:4210-4215` is reconciled to the
  post-U5 predicate (the Implementer obligation `p2` §9.5.5's same-unit table named). Verified: **`cargo
  test` 655 passed / 0 failed** (serial; baseline 604 → +24 U5 in-crate → +27 blind), green **with and
  without** `GNOSIS_SERVER_OLLAMA_URL`; the U5 property layer **314 executed / 355 caps ≤ 400** (8 rows
  HELD); the blind set `tests/blind_u5_boot_vector_index_greens.rs` **27/27 GREEN** (2 live-HTTP + 25
  lib-level); the live battery's **`R-L3` (i)/(ii)/(iv) PASSED live** (with (iii) and (v) parked, and
  `R-L2`'s amended `vector:true` cell re-verified live — `docs/specs/p2-gnosis-server-live-pending-battery.md`
  §7); `cargo fmt --check` exit 0, `cargo clippy --all-targets` 0 warnings, `cargo build` clean. So this
  file's **§9.1 `vector` row** and **§12's V-8.1/V-8.2 U5 notes** now read **LANDED** (their historical
  wording stands as the record), and the **§9.1/§12 "U4/U5 NOT authorized" markers** are annotated
  accordingly: **U5 is authorized and landed; U4 remains HELD** (it needs the consumer-side `SHELL-2`).
  **Nothing here is owed as a wire change:** U5 adds **no** type, key, code, route or §11 row — the flag's
  predicate is the unchanged `snapshot().vectors.is_some()` and only *what the boot puts in the snapshot*
  moved. The failed-`Reachable`-build branch is asserted at lib level only (`p2` §9.5.5's `P-IM-15`; the
  live criterion is **PARKED — not live-verified**), and U5's adversarial pass filed `U5-ADV-1`…`U5-ADV-5`
  plus **`P-9`'s fired trigger** as OPEN rows in `docs/defects.md` (one follow-up "U5 honesty/freshness"
  unit is being opened for `U5-ADV-1`).

---

## 1. What F2 asks

Pin the concrete wire serialization for the Astrographer-shell → Gnosis-engine
proxy seam (§5.1), restricted to the §4.6.1 **retrieval trio** — `ragQuery`
(`RagResult`), `ragStream` (`RagChunk`), `getEngineStatus` (`EngineStatus`,
health) — **plus** the two §4.3.4 audit entry and the SSE framing needed to carry
them. The transport mechanism (HTTP/REST + SSE vs a native IPC channel) is
**deliberately deferred** to a later shell-integration unit; F2 delivers only the
**mechanism-agnostic wire codecs + validation + SSE single-event framing + health
serialization + versioned envelope**. This makes the proxy seam testable
end-to-end (the review's named benefit) and confirms the
`EngineUnavailable`/`EngineError` split against the encoded failure modes.

F2 is **not** a server, **not** a client, and ships **no new runtime deps**
(serde / serde_json / tokio / futures already present suffice). The §4.1.5
`RagStore` persistence surface (34 methods) is out of scope; only the retrieval
trio crosses the wire.

---

## 2. Scope guardrails

**In scope (this contract):**
- `StoreError::wire_code()` — the stable, unique code string per §6 FS variant.
- JSON codecs for the two non-`Serialize` types — `RagChunk` and `StoreError` —
  encoded in `src/wire/`, **not** by mutating the frozen store enum's derive
  surface.
- `RagResult` (already `Serialize`) encoding; the wire wraps it in the envelope
  and adds the decode-then-validate step.
- **decode-then-validate** — malformed body → `EngineError` outcome; well-formed
  body missing `trace` → `TraceUnavailable` outcome (FS-9 / FS-10 realized at the
  decoder).
- **Single-event SSE** framing (`event: result|done|error`), honest to the
  single-shot `rag_stream` (`[Result, Done]` / `[Error, Done]`).
- **Health** serialization of the already-`Serialize` `EngineStatus`.
- The versioned **envelope** `{schemaVersion, idFormat, payload}` carrying the
  deferred UUID-v4 id decision.

**NOT in scope (explicit):**
- A hosted HTTP or native-IPC server (deferred to the shell-integration unit).
- Full `RagStore` CRUD routing.
- HTTP-status **rendering** (this contract documents the map as reference only;
  the shell renders it).
- Bind/auth/TLS + loopback enforcement (recorded **shell-owned**).
- Boot→READY lifecycle (the existing `set_engine_state(Ready)` fixture suffices).
- A shell-side SSE client.
- RFC-4122 id adoption (deferred to the suite/HANDOFF via the `idFormat` seam).
- Incremental multi-chunk `rag_stream` streaming (a separate `rag_stream` engine
  unit).
- Zero new runtime dependencies.

**U1-pinned here, code-bearing elsewhere (none of these units is authorized).** The U1 amendment
pins the **wire** for four surfaces whose **code** lands in a later unit. Naming them keeps F2's
"no server, no client" scope honest:

***SUPERSEDED (2026-09-16) as to U2 and U3: U2/U3 AUTHORIZED (registers authored in
`docs/specs/p2-gnosis-server.md` §9.5.1/§9.5.2); U4/U5 NOT authorized; **(2026-09-17) BOTH LANDED —
U2 LANDED and U3 LANDED-GREEN (see this file's §U2/§U3 status notes at the top).*** The status sentence above
therefore now reads **U2/U3 AUTHORIZED (2026-09-16; registers authored in `p2` §9.5.1/§9.5.2);
U4/U5 NOT authorized** — it was written before the user authorized the two code-bearing units, and it
stands as the historical record of what *the U1 amendment itself* authorized. The table's `code lands
in` column is unaffected (it names the **unit**, not an authorization state): **U2's and U3's
*code* has since LANDED (U2 2026-09-17; U3 LANDED-GREEN 2026-09-17); **U5 is likewise AUTHORIZED (2026-09-22) and LANDED-GREEN** (its typed register is `p2` §9.5.5; the §U5 status note above), so **U4 alone remains NOT authorized** and the §4.6/§4.7 rows stay owed and id-less.

| pinned in | code lands in | what F2 pins |
| --- | --- | --- |
| §4.5 | **U2** | the `ragQuery` request payload (camelCase options) + the bare-`RagResult` response body + the shared envelope-strict/mode decode |
| §4.6 | **U4** | the one wire-visible change cursor (opaque string, step-1, process-lifetime monotonic, accessor distinct from `epoch`) |
| §4.7 | **U4** | the `GET /changes` SSE event/field table, its staleness + reconnect/resync contract, and the cursor-first framing |
| §7.1 | **U2** | the structured transport request-decode error body (the NEW-2 JSON body + its code table) |
| §9.1 / §12 | **U3** (+ **U5** for the index) | the subsystem-flag **capability** semantics + the restated V-8 golden bodies (any additive signal lands on **`HealthReport`** — never on the frozen canonical `EngineSubsystems`) |
| §16 | — (documentation only) | the FS-3 vs FS-13/14/15 engine-side ruling (the engine already behaves this way; only U2's mode decode is code-bearing) |

**Superseded-by-U1 (F16) — two F2 scope guardrails, and exactly how far they fall.** F2's original
"NOT in scope" list below (written 2026-09-09, before P2 landed) contains two clauses that U1's own
surfaces now contradict. Both are **SUPERSEDED for the surfaces U1 pins, and for those only**; every
other bullet stands:

- **"A hosted HTTP or native-IPC server (deferred to the shell-integration unit)."** *Superseded for
  §7.1/REST semantics:* P2 landed the engine-side HTTP host (`src/bin/gnosis_server.rs`; the
  §11-map rendering is `src/server.rs:18-44`), and U1 pins the transport decode-error body (§7.1) and
  the `GET /changes` route (§4.7) **on that host**. F2 itself still builds **no** server: the code
  lands in **U2** (§7.1's body) and **U4** (§4.7's route).
- **"HTTP-status **rendering** (this contract documents the map as reference only; the shell renders
  it)."** *Superseded:* since P2 the **engine** renders the §11 map (§11's status line), and U1 adds
  the SSE status pins (§13; `docs/specs/p2-gnosis-server.md` §5.4/§10). F2 still ships no rendering
  code; §11's table remains the frozen authority both sides agree on.

---

## 3. Module map (`src/wire/`)

| file | concern | exported public API |
| --- | --- | --- |
| `src/wire/mod.rs` | declares + re-export the submodules | `pub mod error; pub mod codecs; pub mod decode; pub mod sse; pub mod status; pub mod envelope;` + flat re-exports (below) |
| `error.rs` | `StoreError` → stable code string + reverse lookup | `impl StoreError { pub fn wire_code(&self) -> &'static str }`; `pub fn from_wire(code: &str, message: Option<&str>) -> Option<StoreError>`; `pub fn code_table() -> &'static [WireCodeRow]` |
| `codecs.rs` | JSON codecs for `RagChunk`, `StoreError`, `RagResult` (envelope-returning) | `encode_chunk`, `decode_chunk`, `encode_error`, `decode_error`, `encode_result`, `decode_result`, `encode_event`(→ sse), `WireCodecError` |
| `decode.rs` | decode-then-validate | `DecodeError`, `ValidationFailure`, `decode_rag_result`, `validate_rag_result`, `decode_chunk_payload`, `outcome_of`. `outcome_of` maps a `DecodeError` → `RagChunk::Error(StoreError::EngineError | StoreError::TraceUnavailable)` |
| `sse.rs` | single-event SSE framing | `SseEventType`, `encode_event`, `decode_event` |
| `status.rs` | schemaVersion-aware health | `HealthReport`, `health(&EngineStatus) -> HealthReport` |
| `envelope.rs` | the versioned envelope | `Envelope`, `CURRENT_SCHEMA_VERSION`, `ID_FORMAT_OPAQUE_STRING_V1`, `current_schema_version()`, `Envelope::{to_json, from_json}` |

**Re-export surface to add in `src/lib.rs`** (the paths the TestWriter calls):

```rust
pub mod wire;
pub use self::wire::codecs;
pub use self::wire::decode;
pub use self::wire::envelope;
pub use self::wire::error;   // unconditional: crate-root `gnosis::error` (no collision exists)
pub use self::wire::sse;
pub use self::wire::status;
pub use self::wire::envelope::Envelope;
pub use self::wire::decode::{DecodeError, ValidationFailure};
pub use self::wire::status::HealthReport;
```

The TestWriter reaches the surface as `gnosis::wire::{codecs, decode,
envelope, error, sse, status}` and via the flat re-exports `gnosis::Envelope`,
`gnosis::DecodeError`, `gnosis::ValidationFailure`, `gnosis::HealthReport`,
`gnosis::error`.
`wire_code` is an inherent method on the existing `gnosis::StoreError` (no
trait/impl change to the store enum; the `impl` lives in `error.rs`).

---

## 4. Canonical wire shapes

### 4.1 The versioned envelope (`envelope.rs`)

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")] // wire keys are schemaVersion / idFormat (shell-friendly)
pub struct Envelope {
    pub schema_version: u32,
    pub id_format: String,
    pub payload: serde_json::Value,
}

pub const CURRENT_SCHEMA_VERSION: u32 = 1;
pub const ID_FORMAT_OPAQUE_STRING_V1: &str = "opaque-string-v1";
pub fn current_schema_version() -> u32 { CURRENT_SCHEMA_VERSION }

impl Envelope {
    pub fn to_json(&self) -> Result<String, WireCodecError>; // serde_json::to_string
    pub fn from_json(s: &str) -> Result<Envelope, DecodeError>; // parse + shape-check
    pub fn with_payload(payload: serde_json::Value) -> Envelope; // schema_version=1, id_format="opaque-string-v1"
}
```

The struct keeps **snake_case Rust fields** (`schema_version`, `id_format`) for the
TestWriter's field access, and the `#[serde(rename_all = "camelCase")]` attribute
makes `serde_json::to_string` emit the **camelCase wire keys** below; `Deserialize`
accepts exactly those same camelCase keys. `to_json`/`from_json` use the same
serde derive path, so the JSON example, the §10 struct, and the §12 golden vectors
all serialize identically (they cannot disagree — they are the same two functions).

Envelope JSON (canonical — equals `to_json(&Envelope{…})` exactly; no spaces after
`:`/`,` and field-order fixed by the struct):
```json
{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":<chunk | result | error json>}
```

**UUID-v4 deferral seam.** Ids (`DocumentId::0`, `NodeId::0`, `WikiId::0`) cross
the wire as **opaque strings** — the wire never parses or validates them as
UUIDs. Changing the id scheme (e.g. adopting RFC-4122 later) **never changes the
wire shape**; it only changes what opaque string the engine emits, plus
optionally a new `id_format` value. `"opaque-string-v1"` is the only implemented
`id_format`; a future `id_format` value is the **extensibility seam**, not code
in F2. RFC-4122 is **not** implemented.

### 4.2 Canonical chunk JSON (identical in `envelope.payload` and the SSE `data:` line)

`RagChunk` is **not** `Serialize`; the wire represents it as a single JSON object
with a discriminative `"type"`:

| Cargo variant | wire JSON | notes |
| --- | --- | --- |
| `RagChunk::Result(r)` | `{"type":"result","result":<RagResult body via serde>}` | `RagResult` is `Serialize`; `trace` is required (non-optional). |
| `RagChunk::Done` | `{"type":"done"}` | terminal, no payload. |
| `RagChunk::Error(e)` | `{"type":"error","code":"<wire_code>","message":"<Display text>"}` | `message` = `format!("{}", e)`; for `ValidationError(m)` this is the carried detail `m`. |

**Body shape note (serde-frozen).** The `<RagResult body via serde>` in the `Result`
row is **exactly** what `serde_json::to_string` emits for the real frozen
`RagResult`/`RagResultItem`/`RagTrace` types — the wire does **not** re-case, re-tag,
or hand-reroute it. That means: **snake_case** field keys (`document_id`, `node_id`,
`top_k`); **PascalCase** enum-unit values (`"Local"`, `"Zodiac"`, `"Flat"`,
`"Graph"`, `"Vector"`, `"Hybrid"`); `RagTrace` **externally-tagged** by variant name
(`{"Flat":{…}}`, `{"Graph":[…]}`, `{"Hybrid":{…}}`); id newtypes (`DocumentId`,
`NodeId`) serialized as their inner strings; optional fields without
`skip_serializing_if` (`parent`, `stale`, `blocked_by`) always present, `null` when
`None`. These body types are frozen (§4.1/§4.5) and immutable in F2; see §12 V-5 for
the pinned full body. Only the wire-defined types (**`Envelope`**, **`HealthReport`**,
the chunk/`code`/`message`/`type` wrapper) use camelCase at their top level.

### 4.3 Canonical error JSON (the non-chunk codec, `codecs::encode_error`)

`{"code":"<wire_code>","message":"<Display text>"}`. This is the shape used when
an error is encoded on its own (e.g. as a query's error outcome); the chunk
encoder adds the `"type":"error"` discriminator on top.

### 4.4 Canonical SSE frame (`sse.rs`)

Single event, LF line endings, terminal blank line:

```
event: <type>
data: <canonical chunk JSON, single line>
<blank line>
```

`<type>` ∈ `{result, done, error}` and **must equal** the `"type"` field of the
data JSON (a mismatch is a `DecodeError`). `data:` is the single-line JSON
(no multi-line SSE continuation in F2). No `id:`/`retry:` fields.

**Media type (F-2 — pinned by the U2 proofread pass, 2026-09-17; a header pin, no frame-byte change).**
**Every SSE response carries `Content-Type: text/event-stream` — exactly that media type, with no `charset`
parameter:** the `200` frame stream, a **pre-stream** `error`-frame response (the `p2` §5.4 mode 400 *and*
the §11-mapped pre-stream `StoreError` frames), and U4's `cursor`/`change` feed (`p2` §5.6). The JSON bodies'
media type (`application/json`, §7.1 / `p2` §5.5) never applies to an SSE response — a consumer that keys on
the media type must not have to special-case which branch produced the frame. **Landed state (recorded, not
absorbed):** the U2 pre-stream renderer emits `text/event-stream`
(`src/bin/gnosis_server.rs:108`), while the **pre-existing** post-stream `StoreError` branch returns the frame
bytes through axum's `String` responder ⇒ `text/plain; charset=utf-8` (`:264-281`) — filed as
`docs/defects.md` **P-5** (owner: the transport unit / the SSE branch; not a U2 regression).

**The `<type>` set is the RETRIEVAL family (F10; annotated by U1).** This closed set is the
`RagChunk` **retrieval** family — `SseEventType` has exactly three variants
(`src/wire/sse.rs`; `src/wire/decode.rs:10-32`) and §8's `decode_event` rejects anything else. It is
**not** a global wire-wide event vocabulary: §4.7's change family (`cursor`/`change`) is **framed
identically** by the same single-event rules but is a **different family** — decoded by U4's own
`decode_change_event`, **never** by `decode_event`, which MUST reject a `cursor`/`change` frame as
`DecodeError::UnknownType` (V-19). A TestWriter must therefore not read this line as "no other
`event:` label can exist on the wire".

### 4.5 The `ragQuery` wire — request envelope, payload, response body (U1-pinned; **code LANDED in U2 — 2026-09-17**)

**Authority.** gate-1 ruling 3 (`docs/specs/gnosis-gr-inbound-review.md`); the server-side route
semantics live in `docs/specs/p2-gnosis-server.md` §5.3/§5.4, and **this** sub-section pins the wire
shapes. **No code landed in U1** — **the U2 code has since landed (2026-09-17: `src/wire/query.rs` +
`src/bin/gnosis_server.rs`), so the "code lands in U2" markers of this sub-section's heading and its
`p2` counterparts now read LANDED; the pinned shapes are unchanged.**

**Request (canonical).** The request is the §4.1 envelope; the payload is the camelCase `ragQuery`
options object:

```json
{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"query":"…","mode":"hybrid","topK":10,"wikiId":"w1"}}
```

- **Envelope-strict.** `schemaVersion` + `idFormat` are validated exactly as the CRUD decode path does
  (`decode_crud_request`, `src/wire/crud.rs:305-310` — `schemaVersion` first, then `idFormat`); a bare
  (non-envelope) body is **rejected**. The envelope check MUST be **shared** with the CRUD path (one
  helper; U2-time), and the **mode rule MUST be shared between the POST path and the SSE path** (below)
  — U2 MUST NOT duplicate either in handler code.
- **Payload keys** (the §4.6.1 options object, `docs/specs/gnosis.md:872`): **14 keys** — `query`
  (string; the only semantically required key), `mode`, `topK`, `wikiId`, `filters`, `maxHops`,
  `expand`, `maxParentContext`, `multiQuery`, `compression`, `hyde`, `binaryFirstPass`,
  `binaryCandidatePool`, `subTaskDag` — types, defaults and fail-states are tabulated once in
  `docs/specs/p2-gnosis-server.md` §5.3 and are **not duplicated** here. `requester` is **not** part
  of the contract (it decodes as an unknown key: §4.6.1 has no such parameter, and the engine takes no
  `caller`).
- **Unknown/extra payload keys are TOLERATED** (ignored; there is no `deny_unknown_fields` anywhere in
  `src/wire/` — the CRUD decoder's precedent, `src/wire/crud.rs:311-326`). This is **load-bearing**:
  the existing e2e POSTs carry an extra `args` key (`tests/gnosis_server_e2e.rs:188-198`).
- **Absent `mode` ⇒ `Flat`** (the mode rule), **read at the two layers of
  `docs/specs/p2-gnosis-server.md` §5.3's "THE TWO-LAYER READING OF THE `absent ⇒` COLUMN" bullet**
  (2026-09-16 annotation, no rule change): the **decoder's** output for an absent or wrongly-typed
  `mode` is `RagQueryOptions.mode == None` — **`Some(Flat)` is never a decoder output**, and only a
  **well-formed vocabulary string** yields `Some(token)` — while `Flat` is the **engine's query-time**
  reading (`mode.unwrap_or(QueryMode::Flat)`, `src/store/mod.rs:4060`, `:4168`). Same for the
  `expand`/`compression`/`topK`-style defaults of the next bullet.
- **A present-but-wrongly-typed optional value decodes as ABSENT** — i.e. the option's own `Option`
  field is **`None`**, read at the **decoder** layer of that same bullet — ⇒ that option's documented
  default **at query time** (never a decode failure, never a 400; the same policy the SSE path already
  applies to an unparseable `topK`: pre-U2 the SSE handler parsed it tolerantly
  (`src/bin/gnosis_server.rs:194`), and U2 moved the parse into the shared decoder
  (`sse_params.top_k.and_then(|s| s.parse::<u64>().ok())`, `src/wire/query.rs:157`)). **`query` is an exception** (⇒ `""` ⇒ 400); a **non-string
  `mode` is not** (it decodes as absent ⇒ `Flat`, §4.5's table). **`filters` is the other exception** —
  its **type/shape IS checked** (a non-object `filters`, a wrongly-typed member, or an unrecognized
  token ⇒ **400 `validation_error`**, since FS-3 names "malformed `filters`"; `null` and `{}` are
  valid). The full corpus is in `docs/specs/p2-gnosis-server.md` §5.3 (F5). **Object-valued options are read at
  the same FIELD level (pin clarification, 2026-09-16 — the blind-greens gate's U2-13 finding; no rule change,
  the wire outcome is identical):** the rule above is quantified over the **option key**, so a value that is a
  **non-object**, **or an object whose documented member is present with the wrong type**
  (`{"multiQuery":{"enabled":"yes"}}`, `{"multiQuery":{"enabled":true,"n":"3"}}`,
  `{"subTaskDag":{"enabled":"yes"}}`), leaves the option's own `Option` field **`None`** — the decoder MUST NOT
  answer with a present object whose member was defaulted inside it. The member-level defaults
  (`enabled` omitted ⇒ `false`; `n` omitted ⇒ `3`) apply **only inside a well-formed object**, so
  `{"multiQuery":{}}` ⇒ `Some(MultiQueryOptions{ enabled:false, n:3 })` and
  `{"multiQuery":{"enabled":true}}` ⇒ `Some({enabled:true, n:3})`. **The member-type check is TOTAL over the
  members each option declares (ruling 2, 2026-09-16 — annotation only, no rule change):** `enabled` is a
  **bool** and `n` a **u64** for `multiQuery`, `enabled` a **bool** for `subTaskDag`, so a present member of
  any other type — `n` included — is a wrongly-typed documented member and makes the **whole option** `None`
  (the clause is implemented in `src/wire/query.rs:178-202` — every declared member is type-checked before the
  option is built, and the pinned `p2` clause is **not** narrowed to `enabled`). **The numeric inputs' TYPE-vs-RANGE boundary is pinned the same way (ruling 1, 2026-09-16):** a
  **present value that is not a `u64`** (`topK:-1`, `topK:10.5`, `maxHops:-2`, `binaryCandidatePool:7.5`) ⇒
  absent ⇒ the documented default and **never** a 400 (`serde_json`'s `as_u64` cannot represent it), while a
  **present `u64` out of its pinned range** (`topK:0`/`51`, `maxHops:0`/`6`, `binaryCandidatePool:0` with
  `binaryFirstPass:true`, `multiQuery:{enabled:true,n:0}`) is the engine's FS-3 **400 `validation_error`** —
  the full statement is `p2` §5.3's type-vs-range bullet, and no cell here claims a 400 for `-1`/`10.5`.
  The canonical cells are `p2` §5.3's two
  object rows plus its two-layer bullet's object-valued-option clause (the one place this reading is pinned).
- **Enum value casing (F1 — the `filters` half is CORRECTED).** `mode` values are the four lowercase
  §4.6.1 tokens (`flat|graph|vector|hybrid`), matched **case-insensitively**; `expand`
  (`none|parent`) and `compression` (`none|filter|extract|graph`) follow the same policy. **The three
  enum-valued `filters` members do NOT keep the store's serde casing** — they are the **canonical
  §4.5.2 request shape** (`docs/specs/gnosis.md:692-694`): `nodeKind` ∈
  `'content'|'fact'|'reference'`, `edgeType` ∈ `'link'|'embed'|'crosslink'`,
  `state` ∈ `'FRESH'|'RESOLVED'|'STALE'|'BROKEN'`, and `target` is the object
  `{documentId, nodeId}`. **Superseded-by-this-remand:** the pre-remand pin ("PascalCase unit values …
  `filters.target` is the serde tuple shape `[documentId, nodeId]`") was the **serde-frozen store
  casing**; the **store casing (`node_kind`, the tuple `target`, PascalCase) is the RESPONSE/AUDIT
  shape only** (§4.2's body-shape note governs the `RagResult`/`QueryAuditEntry` **response** body, not
  caller-supplied request payloads — `src/store/mod.rs:471-477`). Because FS-3 makes an unrecognized
  option value a 400, the pre-remand pin would have **rejected the canonically-shaped caller** — a
  consumer-visible narrowing of a pinned canonical shape in a unit that explicitly does not amend
  `gnosis.md`.
  - **The U2 decoder MAPS the canonical tokens to the frozen store types**; it MUST **NOT** call
    `serde_json::from_value::<QueryAuditFilters>` on the raw payload value: that type has **no**
    `#[serde(rename_all = "camelCase")]` (`src/store/mod.rs:471-477`), so camelCase keys cannot bind,
    and it has **no `deny_unknown_fields`**, so a literal pass-through silently yields an **all-`None`
    filter** — the **GR-2 defect class** (a caller-visible option that is inert).
  - **An `edgeType` outside the three validator-accepted kinds is 400 `validation_error`**
    (`validate_rag_options`: "filters.edgeType must be link, embed, or crosslink",
    `src/store/mod.rs:4311-4321`) — so the pinned `edgeType` token set and the validator-accepted set
    **coincide**, and the store's 9 `EdgeKind` variants (`:112-132`) never widen the wire tokens.
    `nodeKind`'s wire set is likewise exactly the three §4.5.2 tokens (`'community'` is not a member of
    the pinned shape ⇒ malformed ⇒ 400). The full mapping table is
    `docs/specs/p2-gnosis-server.md` §5.3.
- **Where the enum-token check lives (F2 — corrected).** The `mode`/`expand`/`compression` **token**
  check is performed by the **U2 wire resolver** (`p2` §5.4) and **only** there: `RagQueryOptions.mode`
  / `.expand` / `.compression` are **typed enums** (`Option<QueryMode>`/`Option<ExpandMode>`/
  `Option<CompressionMode>`, `src/store/mod.rs:646`, `:650`, `:656`), so an **unrecognized token is
  unrepresentable** in the parameter, and `validate_rag_options` (`:4293-4338`) contains **no**
  mode/expand/compression arm to add to. The resolver runs **before** `validate_rag_options`, which is
  what makes precedence step (2) precede step (3) in `p2` §5.3. **The `expand` 400 is a NEW state
  created by this amendment**: canonical FS-3's closed catalogue enumerates invalid `mode` and invalid
  `compression` but **not** `expand` (`docs/specs/gnosis.md:1027`), and §4.6.1's throw column omits it
  (`:872`) — the `docs/HANDOFF.md` reconcile ask requests that the canonical contract add it (or that
  the engine drop the pin). The case-insensitivity generalization is a **benign superset** and is not
  part of that ask.

**The single mode rule (wire encoding).** Stated once in `docs/specs/p2-gnosis-server.md` §5.4; its
wire encoding:

| input | wire form | resolves to |
| --- | --- | --- |
| absent | POST: no `mode` key; SSE: no `mode` query param | `QueryMode::Flat` (the engine default — `options.mode.unwrap_or(QueryMode::Flat)`, `src/store/mod.rs:4060`, `:4168`) |
| present but **not a JSON string** (`5`, `null`, `{}`) | POST payload value of the wrong type; SSE params are always strings | decodes as **absent** ⇒ `Flat` (the wrongly-typed rule in §4.5; the token rule is not invoked). **N2:** for the two POST-only tokens the same reading holds (`expand:5` ⇒ absent ⇒ `'none'`; `compression:null` ⇒ absent ⇒ `'none'`) — **only a JSON string reaches a token check** |
| `"flat"`/`"graph"`/`"vector"`/`"hybrid"`, any ASCII casing | POST: payload string; SSE: query param | `Flat`/`Graph`/`Vector`/`Hybrid` |
| any other **string** (incl. `""`, whitespace, `"bm25"`) | `mode`: either path (POST payload / SSE param); the `expand`/`compression` tokens: **POST payload only** | `StoreError::ValidationError` ⇒ **400 + wire code `validation_error`** on POST, and for **`mode`** on `/rag/stream` **HTTP 400** with the single **`error` SSE frame** `{"type":"error","code":"validation_error","message":"<detail>"}` then the stream closed (F4 — the status *and* the frame are pinned; `p2` §5.4) — **never** a silent default, **never** a decode-level 422, and **never** an `error` frame under a 200. **N1:** `expand`/`compression` have **no SSE param**, so there is no SSE outcome for them |

The same **casing/token rule** governs the `expand` (`none|parent`) and `compression`
(`none|filter|extract|graph`) tokens, each resolved by one U2 resolver family under the identical
casing policy (**an unrecognized string ⇒ `StoreError::ValidationError` ⇒ 400 `validation_error`**).
Those two are **POST-payload keys**: the SSE query surface reads **only** `query`/`topK`/`mode` today
(`src/bin/gnosis_server.rs:245-263`, landed post-U2 — the handler fills the three-param `SseParams` carrier;
pre-U2 it built the options inline at `:190-203`) and **U2 adds no new SSE params**. **N1 — the outcome is split, and
the SSE half of the pre-remand claim is deleted:** `mode` is the **only** token with a POST **and** an
SSE outcome (the 400 + single `error` frame of the table above applies to `?mode=`); for
`expand`/`compression` the SSE surface has **no such param**, so `?expand=parent`/`?compression=graph`
are **ignored and yield that option's default — NEVER a 400 and never an `error` frame** (the resolver is
never invoked there). The **rule** is single-sourced; the resolver is called by every path that **reads**
the token — and only `mode` is read by both.

**The pre-U4 SSE surface reads exactly `query`, `topK`, `mode` (N1 — named, not an unowned gap).** The
canonical `ragStream` signature also lists `wikiId?`, `filters?`, `maxHops?`, `maxParentContext?`,
`multiQuery?`, `compression?` and `hyde?` (`docs/specs/gnosis.md:873`); **none of those seven is part of
the pre-U4 SSE surface** — an SSE query param outside the three is an unknown param, **ignored, with the
option left at its default** (never a 400), the same tolerance an unknown payload key gets above.
**Their wire treatment is U4's to pin** (in U4's own spec, per `p2` §5.2's route-growth rule), **not** a
`docs/HANDOFF.md` ask: no canonical sentence is contradicted by the narrower SSE surface, so no upstream
row is owed.

The SSE path's pre-U1 case-sensitive match with a silent `None` fall-through
(`src/bin/gnosis_server.rs:195-201`) and the POST path's pre-U1 total disregard of the field
(`:162-168`) are both **superseded** by this rule.

**Response body (canonical).** The response envelope's `payload` is the **bare serde `RagResult`
body** (§4.2 / §12 V-5) — **not** the SSE `result` discriminator wrapper:

```json
{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"query":"…","results":[…],"engine":"gnosis","citations":[…],"trace":{…},"blocked_by":null}}
```

`{"type":"result","result":{…}}` (the §4.2 chunk shape) is **SSE-only**; using it as a POST response
body would produce a body the result validator rejects.

**Encoder/validator rule (pinned).** The encoder MUST never emit a body the result validator rejects:
a `RagResult` failing `validate_rag_result` (§7) is an **engine error at the encoder**
(`StoreError::EngineError` → 502, FS-9), never a silently un-decodable 200. **Reachability:** with the
current engine, every `rag_query` result passes validation (engine `"gnosis"`, trace always present —
`src/store/mod.rs:4413-4419` — the flat trace construction; the pre-U3 citations `:4124`, `:4138-4145` are stale), so this is a **U2-time totality requirement** tested at the
pure level (extending `P-TP-1`), **not** a currently-reachable HTTP fail-state.

### 4.6 The change cursor (U1-pinned; code lands in U4)

**Authority.** gate-1 **ruling 1** + Appendix A; decision **`GNOSIS-CHANGE-CURSOR`** (ACTIVE). There
is **exactly one** wire-visible change cursor. **No code lands in U1** — the accessor is U4's.

**Definition.**

- **Type:** a JSON **string** (Rust `String`). Never a number, never an object.
- **Value:** the **decimal rendering of the committed journal `seq`** (`u64`): `"0"` when no entry has
  committed (a fresh store), `"1"` after the first committed entry, and **+1 per committed journal
  entry** thereafter. The seq is produced by the single writer inside its mutation critical section
  (`append_journal`, `src/store/mod.rs:1828-1839`; the pre-U3 citation was `:1820-1831`; `JournalEntry.seq`, `:944-953`).
- **Monotonicity:** **process-lifetime monotonic**. It **may repeat across restarts** (the store is
  in-memory; GR-7 is parked and `ENGINE-DURABLE-CORPUS-DIRECTION` is DIRECTION ONLY / NOT ACTIVE).
- **Its own accessor, distinct from `epoch` (U4-time).** U4 MUST add a cursor accessor whose
  documentation states the wire semantics above, **distinct** from `epoch()`
  (`src/store/mod.rs:1814-1818`; the pre-U3 citation was `:1808-1810`; documented as the journal/derived-index counter) and `journal_len()`
  (`:1820-1823`; the pre-U3 citation was `:1813-1815`). **Value-equality caveat (pinned so it is not read as a contradiction):** today
  `epoch()` **is** the journal seq (one `AtomicU64`; "`seq` … equals the epoch at commit", `:945`,
  `:1821-1823`), so the new accessor returns the same **value** at the same instant. The distinction
  the ruling requires is **naming, documentation and scope** — a wire cursor for invalidation vs an
  internal rebuild counter — **not** a second counter. A consumer therefore gains **no** additional
  coherence from the cursor over `epoch`, which is exactly why the scope below is restricted.
- **Opaque, in this precise sense:** a consumer MUST NOT derive store semantics from it — it is **not**
  a revision, **not** point-in-time, **not a coherence/validation token**, **not** a timestamp. **F14
  (reworded):** the pre-remand phrase "**not** a count" was contradicted by the pinned value — the
  cursor **is** the committed `seq` and equals `epoch()`/`journal_len()` numerically today
  (`tests/props_consistency.rs:1293` asserts `store.epoch() == store.journal_len() as u64`), so it
  **may be read as an opaque monotone sequence** (and counting entries with it is arithmetically
  possible); what the contract withholds is any **store coherence** from that reading — counting with
  it confers **no** store coherence, no read barrier and no validation power. Its **only** sanctioned
  consumer operations are (a) **equality** comparison for de-dup and (b) **gap detection** (is this
  cursor the previous one + 1?) within one process. A consumer MUST NOT persist it across a restart and
  compare it against the next process's cursor, and MUST NOT assume the rendering (decimal, width,
  prefix) is stable across engine versions — only the **semantics** (step-1, process-lifetime
  monotonic) are contractual.
- **Documented scope (pinned):** **cache invalidation, resync and de-dup ONLY.** Explicitly **not**
  point-in-time, and **not** valid for **cache validation**, **optimistic concurrency**, or **state
  reconstruction**. **Per-document `revision` remains the concurrency token** (§4.1.4/FS-4,
  `docs/specs/gnosis.md`).
- **Refused alternatives:** `GET /snapshot?revision=…` and a `stale_revision` error are **REFUSED, not
  parked** (`docs/specs/p2-gnosis-server.md` §2/§5.6): no global store lock under
  `SHARDED-RWLOCK-STORE` ⇒ no point-in-time read; the only store-wide counter is `epoch`, which also
  advances on propagation / state-annotation journal appends (`src/store/mod.rs:2284`, `:2356`,
  `:3358`); a `stale_revision` variant's generating step cannot run
  (`RESERVED-ERRVARIANTS-DISCIPLINE`); and §11's map may not grow (§5). As-of-N reconstruction is a
  **journal-retention** concern — boundary **C4**
  (`docs/research/astrographer-engine-shell-boundary.md:86`) — inside the future durability design
  unit, **not** a query surface.
- **Where it appears on the wire (exhaustive).** (1) the `GET /changes` **cursor-first** frame
  (`{"type":"cursor","cursor":"…"}`, §4.7); (2) the `cursor` field of **every** change frame (§4.7);
  (3) **any** read payload that carries a cursor (U4's paginated reads) — and when one does, it MUST
  use this field name and this encoding. There is **no second cursor type** on the wire.
- **Refused vs permitted request-side cursor semantics (F9 — scoped).** What is **refused** is a
  request-side **as-of-N / point-in-time** cursor: no caller may ask for "the state as of cursor N",
  and no route may implement a revisioned projection (§4.6's refusal + `p2` §5.6). What is **permitted
  and owed** is a request-side **resume token** on U4's paginated read routes: a `cursor` **field** in a
  read request is a page/resume position, and it MUST be documented as a **resume token** (never as a
  snapshot selector). A resume token that is **not comparable to the current process** (stale, foreign,
  or larger than the current cursor — e.g. after a restart, since cursors may repeat) MUST be
  **tolerated but DEFINED — not merely "ignored"**: the pinned outcome is that the request **yields the
  first page** (`p2` §5.7, §5.6's reconnect rule). **U4's spec owes this pin explicitly**; U4 may pin a
  different *defined* outcome only by updating this clause and `p2` §5.7 in the same unit. A consumer
  that wants point-in-time state must re-read the affected entities through the ordinary read routes.

### 4.7 The `GET /changes` SSE change feed (U1-pinned; code lands in U4)

**Authority.** gate-1 ruling 1 + the GR-5 verdict row; the route, its staleness contract and its
resync rule are also stated in `docs/specs/p2-gnosis-server.md` §5.6. **No code lands in U1** — the
route, the journal `kind`/ids extension and the accessor are all **U4**.

**Framing (reused verbatim from §4.4; one new event family).**

- Same framing as §4.4: LF line endings, exactly one `event:` line, one single-line `data:` JSON line,
  terminal blank line, **no `id:`/`retry:` fields**, **no multi-chunk/continuation lines**.
- The `event:` label MUST equal the data JSON's `"type"` (the §4.4 rule, preserved).
- **The event-type family is extended** with `cursor` and `change`. This is a **new event family**, not
  a reuse of `result|done|error`: `done` never terminates this feed (it is a subscription) and the
  retrieval payload shapes are not reused. Consequently §8's `decode_event` — which decodes
  **`RagChunk`s only** — MUST reject a `cursor`/`change` frame as `DecodeError::UnknownType`
  (reachable today via `decode_chunk_payload`'s catch-all, `src/wire/decode.rs:114`), so the two
  families can never be silently confused. A U4-time `decode_change_event` owns the change family's
  round-trip.
- The `error` frame from §4.2 stays available for a post-head stream failure; **none is reachable on
  the subscribe path** (no fallible store operation is performed, and there is **no READY gate** — the
  cursor is a journal property).

**Frames.**

| # | `event:` | `data:` JSON | notes |
| --- | --- | --- | --- |
| 1 | `cursor` | `{"type":"cursor","cursor":"<opaque>"}` | **cursor-first frame**, emitted on **every** connection (cold start included), before any change frame |
| 2..n | `change` | `{"type":"change","cursor":"<opaque>","wikiId":<string\|null>,"kind":"<token>","nodeIds":[…],"edgeIds":[…],"timestamp":"<iso8601>"}` | exactly one frame per committed journal entry, in commit order, `cursor` strictly increasing |
| fail | `error` | `{"type":"error","code":"…","message":"…"}` | §4.2's error data shape; not reachable on the subscribe path |

**Field table.**

| field | type | nullable | semantics |
| --- | --- | --- | --- |
| `type` | string | no | `"cursor"` / `"change"` / `"error"` — equals the `event:` label |
| `cursor` | string | no | the change cursor (§4.6) of the committed entry; on the cursor frame, the **current** cursor |
| `wikiId` | string | **yes** | the affected wiki's opaque id, or `null` when the entry is not wiki-scoped. **Reachability:** all 20 `op` values appended today are wiki-scoped, so `null` is a **U4-time** possibility the contract permits rather than a live state — U4 must not invent a non-wiki-scoped entry merely to exercise it (`docs/specs/p2-gnosis-server.md` §5.6) |
| `kind` | string | no | the committed journal entry's operation token (`JournalEntry.op`) — closed set below |
| `nodeIds` | array of strings | no (may be empty) | affected `NodeId`s as opaque strings (§4.1) |
| `edgeIds` | array of strings | no (may be empty) | affected edges. **Element encoding owed by U4**: the store has **no** edge-id type (an edge is identified structurally by `Edge { source, target, kind, … }`, `src/store/mod.rs:163-176`), so U1 pins the field, its type and its nullability, and **U4's spec must define the identity string and its uniqueness** |
| `timestamp` | string | no | the committed entry's ISO-8601 UTC timestamp (`iso_now()`, `src/store/mod.rs:1829`) |

**`kind` closed set (the `op` vocabulary that exists today — call sites `src/store/mod.rs:2284-4018`):**
`create_document`, `update_document`, `delete_document`, `publish_document`, `unpublish_document`,
`archive_document`, `create_wiki`, `add_triple`, `declare_community`, `update_community_summary`,
`set_reference_state`, `resolve_entities`, `merge_facts`, `create_fact`, `update_fact`,
`propose_candidate_fact`, `re_sync_embed`, `re_derive_community`, `propagate_fact_staleness`,
`propagate_archived_target`. Any new `append_journal` call site extends this wire-visible set **and
U4's spec** in the same unit. **Note (no invention):** `JournalEntry` today carries **no** ids at all
and no kind beyond `op` (`{seq, op, base_revision, timestamp}`, `src/store/mod.rs:944-953`) — the
ids/kind extension is **U4's**.

**`kind` tolerance + rename binding (F11; pinned in `p2` §5.6).**

- **A consumer MUST tolerate an unknown `kind`**: an unrecognized token does **not** invalidate the
  frame — the consumer treats it as an **opaque change** (invalidate/resync using `wikiId` + ids; it
  MUST NOT drop the frame, MUST NOT fail the stream, and MUST NOT treat the token as an error). The
  engine may therefore add a kind without breaking an older consumer.
- **The engine MAY rename an `op` literal only via an amendment unit that updates this closed set, the
  `p2` §5.6 table, and U4's conformance rows in the same unit.** `op` is an internal
  `&'static str` at the append site (`src/store/mod.rs:1828-1839`; the pre-U3 citation was `:1820-1831`) and **no `tests/` assertion binds
  the literals** today, so without this rule a rename would silently change the consumer-visible
  vocabulary. The conformance assertion over the `op` set is **owed by U4's spec**.

**Ordering contract.** Exactly one frame per committed journal entry (no coalescing of distinct seqs),
delivered in **commit order**, with `cursor` advancing by exactly 1 between consecutive change frames
**within one process**. Propagation and state-annotation appends (`propagate_fact_staleness`,
`propagate_archived_target`, `set_reference_state`) are ordinary entries in this sequence, exactly as
the ruling states.

**Staleness contract (what a consumer may assume).**

1. The cursor is a **notification trigger, not a read barrier**: between a write and its frame, a
   concurrent read may or may not include the write.
2. The cursor is **not** point-in-time and **not** a validation token (no cache validation, no
   optimistic concurrency, no state reconstruction); per-document `revision` remains the token.
3. **Step-1 within a process** (above); frames may repeat a `kind` but never a `cursor`.
4. **Frames may be lost** — this feed is not a durable log.
5. The cursor alone does **not** say *what* changed: `kind` + `wikiId` + ids discriminate.
6. **Process-lifetime only**: after a restart, cursors restart and may repeat; the **cursor-first
   frame** is authoritative for the current process.

**Reconnect / resync rule.** There is **no retained replay log** and **no `?since=` backfill**. On
reconnect — or when the cursor-first frame is ahead of the last cursor the consumer saw, or a change
frame jumps by more than 1 — the consumer **MUST re-read** the affected state from the read routes
(U4) and then resume de-dup from the cursor-first frame. Engine-side guarantees are limited to **order
+ step-1 within one process**; the consumer's bounded cache is **SHELL-2**.

---

## 5. `error.rs` — `StoreError::wire_code` table (exhaustive)

`pub fn wire_code(&self) -> &'static str`. The full 21-variant (the current
`Store` taxonomy) one-to-one map. Each code is **stable** (frozen by this
contract), **unique** (pairwise distinct), **non-empty**, derived from the real
enum only — never invented:

| `StoreError` variant | §6 FS | wire code |
| --- | --- | --- |
| `DocumentNotFound` | FS-1 | `"not_found"` |
| `WikiNotFound` | FS-2 | `"wiki_not_found"` |
| `ValidationError(String)` | FS-3 | `"validation_error"` |
| `ConflictError` | FS-4 | `"conflict"` |
| `DocumentInUse` | FS-5 | `"doc_in_use"` |
| `InvalidState` | FS-6 | `"invalid_state"` |
| `UnresolvedReference` | FS-7 | `"unresolved_reference"` |
| `EngineUnavailable` | FS-8 | `"engine_unavailable"` |
| `EngineError` | FS-9 | `"engine_error"` |
| `TraceUnavailable` | FS-10 | `"trace_unavailable"` |
| `HopLimitExceeded` | FS-11 | `"hop_limit_exceeded"` |
| `CycleDetected` | FS-12 | `"cycle_detected"` |
| `EmbeddingUnavailable` | FS-13 | `"embedding_unavailable"` |
| `VectorIndexUnavailable` | FS-14 | `"vector_index_unavailable"` |
| `LexicalIndexUnavailable` | FS-15 | `"lexical_index_unavailable"` |
| `RerankerUnavailable` | FS-16 | `"reranker_unavailable"` |
| `CompressionFailed` | FS-17 | `"compression_failed"` |
| `HyDEGenerationFailed` | FS-18 | `"hyde_generation_failed"` |
| `MultiQueryExpansionFailed` | FS-19 | `"multi_query_expansion_failed"` |
| `CommunityNotFound` | FS-25 | `"community_not_found"` |
| `SubTaskDagFailed` | FS-26 | `"sub_task_dag_failed"` |

**(21 variants total.** This is the true count of the `pub enum StoreError` at
`src/store/mod.rs:960` (the enum opens there; `:944-953` is `JournalEntry`) — the
reviewed list's "22" was a count slip; the one-to-one code set is exactly as
enumerated above.)

**(U1 restatement: the map stays exactly 21 rows.)** **No new `StoreError` variant and no new wire code
is added** — by U1 or by any later unit of the GR-1..GR-9 workstream. Specifically:

- **No `stale_revision` code.** The revisioned-projection route is **refused** (§4.6) and a variant
  whose generating step cannot run is barred by `RESERVED-ERRVARIANTS-DISCIPLINE`.
- **No `decode_failed` code.** A request-decode failure stays the NEW-2 **transport-level** outcome —
  **400/422, never 502, never a §11 row** (`docs/specs/p2-gnosis-server.md` §5.5/§7.2; §7.1 below).
  The transport code vocabulary (§7.1) lives **outside** this map and is **disjoint** from it.
- The map is re-asserted unchanged in `docs/specs/p2-gnosis-server.md` §5.2 (the route-growth
  invariant: "the §11 map stays 21 rows") and in P1a §6.1 (the CRUD-reachable 7-variant subset).

**`ValidationError` carries its message.** The message `m` appears only in the
wire `"message"` field; the wire code stays the fixed `"validation_error"` for
every instance of the variant. `wire_code()` therefore returns `&'static str`
regardless of the carried message.

**Reverse lookup.**

```rust
pub fn from_wire(code: &str, message: Option<&str>) -> Option<StoreError>
```

Maps the canonical code → the unit variant; for `"validation_error"` uses
`message` (required) → `ValidationError(msg.into())`; returns `None` for any
non-canonical / unknown / foreign code. Bijective for the 20 unit variants and
for `validation_error` paired with a message.

```rust
pub struct WireCodeRow { pub code: &'static str, pub variant_name: &'static str, pub display: &'static str }
pub fn code_table() -> &'static [WireCodeRow]; // the 21 rows above, for docs/tests
```

**Fail-state / throw patterns for `wire_code`:** no internal failure; it is a
pure, total function over the closed enum. It never panics, never returns an
empty string, and never returns the same string for two distinct variants.

---

## 6. `codecs.rs` — JSON codecs

All codecs return/receive an `Envelope` (the wire wraps every payload). All
`decode_*` path is **decode-then-validate** (see §7).

```rust
// RagChunk — encode the chunk into an envelope (payload = canonical chunk JSON).
pub fn encode_chunk(chunk: &RagChunk) -> Envelope;
// inverse: decode the envelope back to the chunk, mapping result-body failures
// into EngineError / TraceUnavailable outcomes (decode.rs).
pub fn decode_chunk(env: &Envelope) -> Result<RagChunk, DecodeError>;

// StoreError — standalone error codec (payload = {"code","message"}).
pub fn encode_error(err: &StoreError) -> Envelope;
pub fn decode_error(env: &Envelope) -> Result<StoreError, DecodeError>;

// RagResult — the result body is already Serialize; the wire wraps it + validates.
pub fn encode_result(res: &RagResult) -> Envelope;   // payload = RagResult body
pub fn decode_result(env: &Envelope) -> Result<RagResult, DecodeError>; // decode-then-validate
```

`WireCodecError` (internal serialization error; only reachable on a
non-object payload map, which the codecs never produce — a defensive enum, not a
tested fail-path).

**Round-trip identity (the core contract):**
- `decode_chunk(&encode_chunk(c)) == Ok(c)` for every `RagChunk` (all three
  variants), with `Result` bodies round-tripping through the validation step.
- `decode_error(&encode_error(e)) == Ok(e)` for every `StoreError` (21 variants,
  incl. `ValidationError(m)` preserving `m`).
- `decode_result(&encode_result(r)) == Ok(r)` for every well-formed `RagResult`
  (query, results, engine `"gnosis"`, citations, trace matching the result,
  optional `blocked_by`).

---

## 7. `decode.rs` — decode-then-validate

This is the realization of FS-9 (malformed body → `EngineError`) and FS-10
(well-formed body, missing `trace` → `TraceUnavailable`).

```rust
pub enum DecodeError {
    InvalidJson(String),                 // serde parse/shape failure
    UnknownType(String),                 // chunk payload "type" not in {result,done,error}
    MissingTrace,                        // well-formed result body, no "trace"
    UnsupportedSchemaVersion(u32),       // envelope.schema_version != CURRENT_SCHEMA_VERSION
    UnknownIdFormat(String),             // envelope.id_format not a known value
    InvalidEnvelope(String),             // missing schema_version/id_format/payload, or SSE frame malformed
    UnknownCode(String),                 // error codec: no variant maps to this code
    UnknownMethod(String),               // added by P1a: a CRUD request "method" naming no CrudMethod (transport 422)
    EventTypeMismatch { event: String, data_type: String }, // SSE event line != data "type"
    ValidationFailed(ValidationFailure), // a valid structural body failed validation
}

**`UnknownMethod` (F12 — the arm the listing above omitted; added by P1a).** The enum has **10**
variants (`src/wire/decode.rs:10-32`; the arm is `UnknownMethod(String)` at `:25-27`, documented
"§7.2 P1a — a CRUD request `"method"` value that names no `CrudMethod` variant (well-formed JSON,
unrecognized method → transport 422)"). The pre-remand listing showed only nine, so a TestWriter
deriving the enum from this section would have missed the variant the CRUD request-decode path needs.
It is **request-decode-only**: one of the five variants in §7.1's transport code table
(`unknown_method` → **422**) and of `request_decode_status` (`src/server.rs:56`), and **unreachable on
the `ragQuery` path** (`docs/specs/p2-gnosis-server.md` §5.3).

pub enum ValidationFailure {
    MissingTrace,                        // trace absent
    WrongEngine(String),                 // RagResult.engine != "gnosis"
    BlockedByWithoutGraphTrace,          // blocked_by is Some but trace is not RagTrace::Graph
}
```

**Functions:**

```rust
// Strict decode of a RagResult body from a JSON value.
pub fn decode_rag_result(json: &serde_json::Value) -> Result<RagResult, DecodeError>;
//   - a body with NO "trace" KEY (regardless of any other structural issue)  -> Err(MissingTrace) [FS-10 route]
//     (trace-presence is checked FIRST so a well-formed-but-traceless body stays FS-10; both
//     FS-9 and FS-10 map to HTTP 502 at the shell, so this precedence only changes the wire code)
//   - otherwise structurally malformed (wrong field types, invalid enum, etc.) -> Err(InvalidJson(_)) [FS-9 route]
//   - otherwise -> Ok(valid RagResult)

// Post-decode validation of a constructed/decoded RagResult against the
// §4.6.1 / §4.3.3 invariants: engine == "gnosis"; trace present; a present
// blocked_by implies a RagTrace::Graph trace (blocked_by is only produced by the
// graph empty-result path).
pub fn validate_rag_result(res: &RagResult) -> Result<(), ValidationFailure>;

// Decode the canonical chunk JSON (envelope.payload or the SSE data line).
pub fn decode_chunk_payload(payload: &serde_json::Value) -> Result<RagChunk, DecodeError>;

// Map a DecodeError to the wire outcome chunk (FS-9 / FS-10).
pub fn outcome_of(e: DecodeError) -> RagChunk;
//   InvalidJson | UnknownType | UnsupportedSchemaVersion | UnknownIdFormat
//   | InvalidEnvelope | UnknownCode | EventTypeMismatch | ValidationFailed(_)
//        -> RagChunk::Error(StoreError::EngineError)
//   MissingTrace
//        -> RagChunk::Error(StoreError::TraceUnavailable)
```

**Contract rules:**
- A body that decodes but fails `validate_rag_result` surfaces as an `EngineError`
  **outcome** (`decode_chunk`/`decode_result` return `Err(ValidationFailed(_))`,
  and the caller maps it via `outcome_of`). `MissingTrace` alone → `TraceUnavailable`
  outcome.
- The encoder (`encode_result`) **never** emits a body that `decode_rag_result` +
  `validate_rag_result` rejects: `validate_rag_result(&decode_rag_result(&encoded)?.unwrap())`
  is `Ok` for every encoder input (the P-TP-1 row of the register).

### 7.1 The structured transport request-decode error body (U1-pinned; **code LANDED in U2 — 2026-09-17**)

**Authority.** gate-1 ruling 2 + the GR-1 verdict row; the HTTP side is
`docs/specs/p2-gnosis-server.md` §5.5 (with the worked examples). Pre-U1 the server returned the
literal plain-text body `"request decode failed"` (`decode_error_response`,
`src/bin/gnosis_server.rs:57-66` in the **pre-U1 tree**; the landed renderer is `:62-75` — the `:61-74`
spelling this section carried under U3 is stale), which the
consumer's masking client cannot surface. **The code has since landed (U2, 2026-09-17)** — the shape below is
the landed rendering.

**Shape (pinned).** `Content-Type: application/json` — **EXACTLY** that media type, with **no `charset`
parameter** (F15: V-15's byte-exact claim requires the header value to be `application/json` alone; the
pre-U1 plain-text body carried axum's `text/plain; charset=utf-8`) — and a minimal,
**non-envelope-wrapped** body — the §4.3 canonical error JSON:

```
{"code":"<transport code>","message":"<detail>"}
```

Rationale: the envelope is precisely what failed to decode, so an envelope-shaped error response would
assert an envelope contract the request did not satisfy; the body is the minimal `{"code","message"}`
pair. This body is **transport-level** — `decide_*`/`DecodeError`-derived, **not** a `StoreError`, and
**not** a §11 row (§5).

**The code table (derived from the `DecodeError` variants, `src/wire/decode.rs:10-32`):**

| `DecodeError` variant | transport code | transport status | `message` |
| --- | --- | --- | --- |
| `InvalidJson(String)` | `"invalid_json"` | **400** | the carried string, verbatim |
| `InvalidEnvelope(String)` | `"invalid_envelope"` | **400** | the carried string, verbatim |
| `UnsupportedSchemaVersion(u32)` | `"unsupported_schema_version"` | **400** | `"unsupported schemaVersion: {v}"` |
| `UnknownIdFormat(String)` | `"unknown_id_format"` | **400** | the carried string, verbatim |
| `UnknownMethod(String)` | `"unknown_method"` | **422** | the carried string, verbatim |
| `UnknownType` / `MissingTrace` / `UnknownCode` / `EventTypeMismatch` / `ValidationFailed` | — (`None`) | — | never rendered as a request-decode body (response/SSE-side variants) |

**Which part of the body is pinned, and which part is the renderer's own (2026-09-16; annotation — the
`message` rule below is unchanged).** The **code**, the **status**, the exact **header**, and the **key
set/order** (`code`, `message`) are the **byte-binding pins**; the `message` **value** is the
**implemented renderer's own output** — the carried string of the `DecodeError`, rendered verbatim under
the rule below (so for `InvalidEnvelope(…)` it is whatever string the implementation carries, which V-15.1
states explicitly). A golden therefore asserts the structure **plus** the string derived from the
implementation, never a hand-written literal presented as pinned wire text; V-15's two literals are the
exception only because their messages **are** the carried strings of their cases.

**Contract rules.**

- **Status rule.** The status is exactly the existing NEW-2 outcome
  (`request_decode_status`, `src/server.rs:51-62`): **400/422, never 502, never a §11 row.** The code
  mapping MUST be **total over the same domain** and `None` outside it, so status and code are always
  defined together (U2-time — **LANDED 2026-09-17**: the pure fn is
  `request_decode_code(e: &DecodeError) -> Option<&'static str>`, `src/wire/query.rs:339-350` — the name that
  landed; the mapping above
  is pinned).
- **`message` is the carried string verbatim, and it MAY be empty** (the pre-U1 "non-empty for every
  render" clause is **superseded** — the restated rule is `p2` §5.5, with the register rows
  `p2` §9.5.1 `P-TP-3` / §11's `P-TP-3` annotation forbidding the assertion): the four string-carrying
  variants (`InvalidJson(m)`/`InvalidEnvelope(m)`/`UnknownIdFormat(m)`/`UnknownMethod(m)`) carry their
  string **verbatim**, and the decoder substitutes **no** placeholder, so an empty carried string renders
  an empty `message` (see V-15.1's binding note below); **`UnsupportedSchemaVersion(v)` is the one variant
  whose message is guaranteed non-empty** — it renders the decimal value
  (`"unsupported schemaVersion: {v}"`). `code` + status are
  **byte-stable** test targets; `message` is byte-stable for the pinned cases (e.g.
  `UnknownMethod("bogus")` ⇒ `"bogus"`). TestWriters MUST NOT assert serde's own parse-error text for
  `InvalidJson` (an implementation string, not a contract token) and MUST NOT assert a non-empty
  `message` for the four string-carrying variants.
- **Disjoint from §11 (testable).** For each of the five codes,
  `StoreError::from_wire(code, Some(msg)) == None` (`src/wire/error.rs`) — a client must not feed these
  to the `StoreError` reverse lookup, and no §11 row exists for them.
- **Shared rendering.** One function renders the body for **every** route that can fail to decode a
  request (the CRUD handler and the query handler — `src/bin/gnosis_server.rs:191-201`, `:206-214`, landed;
  pre-U2 `:143-153`, `:157-161`), so
  the CRUD and query decode-error bodies changed together in U2. **No existing test pins the pre-U1
  plain-text body or its content type** (no `tests/` assertion on `"request decode failed"`), so the
  change is not a regression. **The SSE route's frame responses are the exception and are NOT this JSON
  body:** every SSE response carries `Content-Type: text/event-stream` (§4.4's F-2 pin); the JSON body
  applies to the SSE handler's transport-failure fallback only, where no frame can be formed. The one
  pre-existing divergence (the post-stream `StoreError` frame rendered `text/plain; charset=utf-8`) is
  `docs/defects.md` **P-5**.
- **The `None` domain is unreachable on the request-decode path** (those variants are response/SSE-side,
  §7/§8); the `.unwrap_or(400)` fallback in `decode_error_response` stays defensive only.
- **The mode rule is not a transport outcome.** An unrecognized `mode` is a
  `StoreError::ValidationError` ⇒ 400 `validation_error` (§4.5), **never** a transport code and
  **never** a 422.
- **A non-object payload was a U2-TIME change — now LANDED (F13; annotated by the post-greens doc-review,
  2026-09-16).** **Current tree (U2 landed, 2026-09-17):** the shared query decoder's object check
  (`as_object().ok_or(InvalidEnvelope(…))`, `src/wire/query.rs:83-87`) rejects a non-object `payload` as
  **`Transport(InvalidEnvelope(_))`** ⇒ 400 **`invalid_envelope`** — the single code `P-IM-4`/F9 pins. The
  paragraph below is the **pre-U2** record it supersedes: before U2, `Envelope::from_json` accepted **any**
  payload `Value` for the `payload` key
  (`src/wire/envelope.rs:65-68` — it only requires the key's presence), so on the query path a
  `"payload": 5` / `"payload": []` envelope reaches the handler, `payload.get("query")` misses, and the
  outcome is `query = ""` ⇒ **400 `validation_error`** (a `StoreError`, §11-mapped) — *not* a transport
  code, and on a not-READY engine the READY gate never runs because validation fails first. The
  **`invalid_json`/`invalid_envelope`** rendering of a non-object payload was therefore **U2-time** (and is now
  landed, see the annotation at the head of this bullet): U2's
  shared query decoder adds the object check (`as_object().ok_or(InvalidEnvelope(…))`, mirroring
  `decode_crud_request`'s CRUD rule at `src/wire/crud.rs:311-313`), and only then does the transport
  code apply. `p2` §5.3's fail-state table marks this row **U2-time** for the same reason. **Status-only
  note:** the HTTP *status* is 400 both before and after (the pre-U2 `validation_error` and the landed
  `invalid_envelope` are both 400), so no test can pin the switch by status alone — the **code** is what
  changes.

**Worked examples (byte-pinned; both bodies were U2-time and have **LANDED** — `decode_error_response`,
`src/bin/gnosis_server.rs:62-75`, landed post-U2; **the `:61-74` spelling this header carried is stale — the
U3 landing shifted the fn's opening brace to `:62`**; pre-U2 the status was right and the body was plain text).**

```
HTTP/1.1 422 Unprocessable Entity
content-type: application/json

{"code":"unknown_method","message":"bogus"}
```

```
HTTP/1.1 400 Bad Request
content-type: application/json

{"code":"unsupported_schema_version","message":"unsupported schemaVersion: 99"}
```

---

## 8. `sse.rs` — single-event SSE

```rust
pub enum SseEventType { Result, Done, Error }

pub fn event_type(chunk: &RagChunk) -> SseEventType;  // Result/Done/Error
pub fn encode_event(chunk: &RagChunk) -> String;      // §4.4 framing; data = canonical chunk JSON
pub fn decode_event(frame: &str) -> Result<RagChunk, DecodeError>;
```

**Contract rules:**
- `encode_event` emits exactly one event: `event: <type>\ndata: <json>\n\n`.
- `decode_event(&encode_event(c)) == Ok(c)` for all three variant classes
  (round-trip). On decode: `InvalidEnvelope` for a frame with no `event:`/`data:`
  lines or trailing non-blank content; `InvalidJson` for an unparseable `data:`;
  `EventTypeMismatch` when `event:` != data `"type"`; result bodies route through
  decode-then-validate.
- **Single-shot honesty:** F2 emits at most one data-carrying event then a
  `done` (or `error` then `done`). No multi-chunk/incremental framing, no
  continuation lines.

---

## 9. `status.rs` — health

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")] // wire keys: schemaVersion / idFormat / lastError
pub struct HealthReport {
    pub schema_version: u32,      // CURRENT_SCHEMA_VERSION
    pub id_format: String,        // ID_FORMAT_OPAQUE_STRING_V1
    pub state: EngineState,       // reused store enum (Serialize) → PascalCase ("Ready"/"Degraded")
    pub version: String,
    pub subsystems: EngineSubsystems, // store/graph/lexical/vector/embedding/reranker (Serialize)
    pub last_error: Option<String>,
}

pub fn health(status: &EngineStatus) -> HealthReport;
```

Like the `Envelope`, the `HealthReport` struct keeps **snake_case Rust fields** and a
`#[serde(rename_all = "camelCase")]` attribute, so it serializes
`schemaVersion`/`idFormat`/`lastError` on the wire (consistent with the envelope).
It **maps** `EngineStatus` field-by-field into its own shape (it does **not** embed
the raw `EngineStatus`), so the emitted `state` value is the real serde output of the
reused `EngineState` enum — PascalCase `"Ready"`/`"Starting"`/`"Degraded"`/
`"Unavailable"`.

**Contract rules:**
- Pure, deterministic function of `EngineStatus` (no hidden state, no I/O): equal
  `EngineStatus` → equal `HealthReport` (the P-SM-3 row of the register).
- Maps `status.state`, `status.version`, `status.subsystems`, and
  `status.last_error` verbatim (`state` emits the PascalCase `EngineState` value).
  The `subsystems` inner flags keep their snake_case single-word field names
  (`store`, `graph`, `lexical`, `vector`, `embedding`, `reranker`) — unchanged by the
  top-level rename.
- `last_error` is `Some` exactly when `status.last_error` is `Some` (the §4.6.1
  `getEngineStatus` impl populates it only in `DEGRADED` state; the report is a
  faithful projection and does **not** invent a `last_error`).

### 9.1 Subsystem-flag capability semantics (U1-pinned; **U3 LANDED-GREEN 2026-09-17**; the boot index build is **U5**'s)

**Authority.** gate-1 **ruling 4**; decision **`SUBSYSTEM-FLAG-CAPABILITY-SEMANTICS`** (ACTIVE);
`docs/specs/p2-gnosis-server.md` §5.8 is the server-side counterpart.

**The semantics (pinned).** An `EngineSubsystems` flag means:

> **"this subsystem's full query-time capability is wired and functional for the current store"**

— **not** "a provider exists". The flag is a claim about the *query path*, not about the presence of a
seam.

**Under that definition the pre-U3 code was wrong.** The six flags were hard-coded `true` at
construction (`src/store/mod.rs:1718-1725` — the LEGACY literal, under its U3 comment block `:1714-1717`,
**dead value after U3**; the pre-U3 citation was `:1710-1717` and the `:1714-1725` spelling this section
carried after the U3 landing is stale, `p2` §5.8):

| flag | honest today? | evidence |
| --- | --- | --- |
| `store` | **yes** | the sharded store is the engine |
| `graph` | **yes** | the graph/nodes/edges surface is implemented |
| `lexical` | **yes** | a **live shard-scan BM25** leg — no prebuilt index is needed (`lexical_rank`, `src/store/mod.rs:4852`; `bm25_search`, `:4261-4274` — the pre-U3 citations were `:4815`, `:4224-4234`) |
| `vector` | **NO (as emitted pre-U3)** | the boot path swaps in `DerivedIndexes::default()`, leaving `vectors: None` (`src/bin/gnosis_server.rs:390-404`, landed; the pre-U3 citation was `:385`, post-U2 `:329`) ⇒ `mode=vector` cannot work until **U5**'s boot index build (`vector` is now derived honestly at read time, `src/store/mod.rs:4213`). **(U5 note, 2026-09-22 — ADDITIVE; the pre-U5 text above stands as the record.)** **U5 AUTHORIZED, its spec gate landed, and the unit is LANDED-GREEN (2026-09-22 — gate 8: see the §U5 status note at the top of this file for the verified figures and the `file:line` of the build, the boot wiring and the reconciled derivation-site comment)** (`docs/specs/p2-gnosis-server.md` **§9.5.5**, 8 typed rows `P-IM-10`…`P-IM-15`/`P-SM-7`/`P-TP-5`): the boot **builds** the index when the provider probe was `Reachable` — over the store's whole node corpus, `FieldType::Full` only, and **an empty store still yields `Some(VectorIndex{})`** ⇒ **`vector: true`** — while `Absent`/`Unreachable` builds **no** index (`vectors: None` ⇒ `vector: false`). The flag's **predicate is unchanged** (`snapshot().vectors.is_some()`); U5 changes only *what the boot puts in the snapshot*. The V-8/V-8.x literals that project this row's value are named in §12 below and move in the **same unit**. |
| `embedding` | **NO (as emitted pre-U3)** | the flag was hard-coded `true` and the server's boot path **never** called `set_subsystems` (`src/store/mod.rs:1868-1876` — write-only and **observationally inert** after U3; the pre-U3 citation was `:1862-1863`), so a **DEGRADED** boot (provider unreachable, `src/bin/gnosis_server.rs:391-408`, landed; the pre-U3 citation was `:390-391`, post-U2 `:334-336`) reported `embedding:true` — a false claim. **U3 derived it from the wired provider (LANDED 2026-09-17, `src/store/mod.rs:4217`); the *state* was already honest (`READY`/`DEGRADED` track provider availability)** |
| `reranker` | **NO** | **no reranker implementation exists anywhere in `src/`** — a pure false claim |

**The additive amendment (permitted ONLY here — and scoped to `HealthReport`, F7).** To express the
"capability wired vs index built" distinction for `vector`/`reranker`, an **additive** change to this §9
status surface is permitted — **a new field on `HealthReport`**
(`src/wire/status.rs:14-21`, F2's **own wire type**) or **a documented value convention within the
existing six booleans** — **provided** it is (a) additive (no existing field removed or re-typed),
(b) consistent with the `HealthReport` camelCase top-level convention, and (c) landed **together with**
the V-8 golden update. Any such change is **F2-amendment-only**; U1 pins the *semantics*, **not** a
specific new field (nothing is invented here).

**`EngineSubsystems` is FROZEN — the additive signal MUST NOT land there (F7; supersedes the broader
pre-remand framing).** `EngineSubsystems` is a **canonical §4.6.1 store type**
(`docs/specs/gnosis.md:874` — `subsystems: {store, graph, lexical, vector, embedding, reranker}`;
`src/store/mod.rs:568-575`), not an F2-owned wire type. Adding a field to it is therefore **not**
"F2-amendment-only": it would be a change to the canonical contract **and** it would break every
`EngineSubsystems { … }` struct literal — **twenty-three** of them across `tests/` (***REMAND-2 SHOULD-FIX 6, 2026-09-22 — re-counted by `EngineSubsystems *\{` sweep this pass: the pre-remand "twenty-five" was wrong; its enumeration double-counted `tests/wire_conformance.rs` and cited two `tests/rag_query_integration.rs` literals that do not exist (that file has **zero** `EngineSubsystems` occurrences). The per-file, per-line enumeration below is the single published figure and `docs/specs/p2-gnosis-server.md` §5.8 carries the same count.***):
`tests/wire_conformance.rs` **5** (`:698`, `:714`, `:727`, `:770`, `:1266` — the helper's return-type-only mention at `:1180` is **not** a literal, and no `derived_read`-body literal exists); `tests/props_wire.rs` **5** (`:282`, `:689`, `:697`, `:705`, `:713` — `:282` is a literal inside `fn all_true_subsystems()`'s body, whose **signature** is `:281`); `tests/props_gnosis_server.rs` **10** (`:4231`, `:4253`, `:4591`, `:4599`, `:5000`, `:5497`, `:5505`, `:5513`, `:5522`, `:5572` — the pre-remand `:4229`/`:4251`/`:4589`/`:4597`/`:4998`/`:5495`/`:5503`/`:5511`/`:5520`/`:5570` spelling of this list was 1–2 lines stale); `tests/blind_u3_status_honesty_greens.rs` **3** (`:164`, `:916`, `:1074`); `tests/rag_query_integration.rs` **0** (**no** `EngineSubsystems` occurrence at all — the pre-remand `:1649`/`:1668` pair were phantom anchors). **N6 — corrected count/citation:** the
pre-remand text said **ten** and cited `tests/props_wire.rs:282` as a signature (it is a literal — the
**signature** is at `:281`), and the current count is **twenty-three** (5 + 5 + 10 + 3 + 0) — **the "nineteen" this F7 note and §9.1's
unit-split paragraph carried after U3 was an under-count** (it omitted the blind set), and the
**"twenty-five" carried before this remand was an over-count** (it invented the two
`rag_query_integration` literals and split `wire_conformance`'s helper mention from its count). The pre-U3 citations
`tests/wire_conformance.rs:690`/`:733`/`:1044` have themselves drifted to `:698-706`/`:714-722`/`:1086`
(fixture literals) under the U3 insertions, while the pre-U3 `tests/props_wire.rs:689`/`:697`/`:705`/`:713` are
unchanged. The frozen-type conclusion is
unaffected. **Pinned: `EngineSubsystems` MUST NOT gain,
lose, or re-type any field** in U1, U3, U5 or any other unit of this workstream; its **type** is frozen
exactly as `HealthReport`'s top-level convention is. **U3's blast radius is therefore `HealthReport` +
`src/wire/status.rs` + the V-8 golden literals only** (a new `HealthReport` field, if U3 wants one,
appears in the V-8.x literals **in the same unit**, and is additive for the consumer).

**Register relationship (pinned; no row added in U1).** The row that pins the status projection is
**`P-SM-3`** of the F2 register — **`docs/specs/7-2-wire-property-register.md:51`** (F17(iii): the
pre-remand citation `:51-52` *spanned* the row, and line `:52` is **`P-TP-1`**, not `P-SM-3` — so the
pinned row is `:51` alone) — and it is
**unchanged** by U1 — it pins `health` as a **deterministic, faithful pure projection** of
`EngineStatus` (equal input ⇒ equal report; `last_error` mirrored). Faithfulness is satisfied whatever
the flag *values* are, **and is unaffected by an additive `HealthReport` field**: `health` stays a
faithful projection of whatever `EngineStatus` supplies (`P-SM-3`'s element-wise equality already
tolerates an added field, because both sides are produced by the same `health` call). The truth of a
flag is an obligation on whoever **produces** `EngineStatus` (§4.6.1), i.e. on **U3's** derivation —
not on `health`'s projection. The new invariant U1 does *not* add is therefore **owed by U3** ("a flag
is `true` iff that subsystem's full query-time capability is wired and functional for the current
store"; `docs/specs/p2-gnosis-server.md` §11's register notes name it). **Zero new rows land in U1.** ***DISCHARGED
by U3 (2026-09-17):*** the derivation is landed in `get_engine_status` (`src/store/mod.rs:4193-4232`), the row
that carries the invariant is `p2` §9.5.2's `P-IM-7` (with `P-IM-8`/`P-IM-9`/`P-SM-5`/`P-SM-6`), and all five
rows are **HELD** (127 executed ≤ 400) — so nothing on this line remains owed.

**U3's rows are now AUTHORED — in `p2` §9.5.2, not in this file (2026-09-16 update; `P-SM-3` untouched;
row ids renumbered by the register remand, see below).**
U3 is **authorized** — and **has since LANDED-GREEN** (§U3 status note at the head of this file; the
preamble above stands as the 2026-09-16 authorization record) — and its typed register is authored as
**`docs/specs/p2-gnosis-server.md` §9.5.2** — `P-IM-7` (flag truth under capability semantics),
`P-IM-8` (embedding-flag honesty in the DEGRADED/absent-provider case), `P-IM-9` (boot-flag honesty),
`P-SM-5` (status read purity), `P-SM-6` (`HealthReport` faithfulness — and, since the register remand's
**F12**, the **frozen six-flag shape** is asserted at the **conformance layer**
(`health_report_shape_frozen` in `tests/wire_conformance.rs`, `:1263` — **LANDED with U3**, 2026-09-17), not by a property row, because a frozen-type
criterion is not falsifiable by a generator; **no additive `HealthReport` field lands in U3**, so this
sub-section's F7 additive permission stays unused — **U3 has since landed and the permission is still
unused** — `HealthReport` keeps exactly `{schemaVersion, idFormat, state, version, subsystems, lastError}`,
`src/wire/status.rs:14-21`). **They are deliberately NOT added to this
file's register**: `docs/specs/7-2-wire-property-register.md` is the **F2 unit's** register (its header
scopes it to `src/wire/` and its executed layer is `tests/props_wire.rs`), so U3 rows there would expand
the **F2** unit's property layer; the repo's per-unit row namespaces make that the wrong home. This
file's `P-SM-3` row is **not** modified, superseded or re-scoped — U3's rows quantify over flag **values**
and the flag **shape**, `P-SM-3` over the projection's determinism/faithfulness, and the two are
complementary. The same applies to U2: its rows land in **`p2` §9.5.1**, and `§12`'s V-8.1/V-8.2 golden
literals (this file) are edited **in the same unit as U3's flag change**, never as a property row.

**Register remand (2026-09-16) — the U3 row ids were renumbered (F1), and this note is the cross-reference
that moved with them.** The register reviewer found that `p2` §9.5 is a **continuation of the p2 register
numbering**, so U3's original `P-IM-5`/`P-IM-6` collided with U2's rows **in the same file** (and, because
the execution plan derives each row's seed tag from the id's bytes, both mapped to the tag `PIM5` while the
plan listed nine tags for ten rows). The ruling: U3's three IM rows were **renumbered continuing after the
highest p2 IM id** — `P-IM-5` ⇒ **`P-IM-7`** (flag truth under capability semantics), `P-IM-6` ⇒
**`P-IM-8`** (embedding-flag honesty), `P-IM-7` ⇒ **`P-IM-9`** (boot-flag honesty) — with `P-SM-5`/`P-SM-6`
unchanged (they never collided) and **no id reused for a different claim**. The full disposition
(F1–F16) is in `p2` §9.5.3.1. Two U3 rows also gained a **named non-row layer** in that remand: the
frozen-flag **shape** (F12, the conformance assertion above) and the boot's **provider-reachable** live
outcome (F11, the live-scenario battery), while U3's flag derivation itself is now pinned as a **read-time
projection inside `get_engine_status`** (F16) — which is why the boot keeps its flag obligation at zero.
**Cross-file look-alikes remain untouched:** this repo's other registers (F2's own `P-IM-4`/`P-SM-4`,
`4-3-facts-property-register.md`'s `P-SM-4`/`P-SM-5`) are different units in different files.

**No new status surface.** `GET /engine/status` remains the **single** status surface, always **200**,
and **no new endpoint** is added for capabilities or index state.

**Unit split (pinned — U3 LANDED 2026-09-17, U5 still owed).** The flag **code** change lands in **U3** (`reranker: false`; `vector: false`
until U5; both derived from real state — **now landed**: the derivation is `src/store/mod.rs:4193-4232` and
`src/lib.rs:69-110`), the **boot vector-index build** in **U5**, and the **V-8 golden
literal edit lands in the same unit as the flag change** (U3 — **landed**, `tests/wire_conformance.rs:1061-1112`) — see §12's **V-8.1/V-8.2** for the
amended literals and §12's transient-divergence note (now resolved). **U3's blast radius (F7, restated where the
split lives):** `src/wire/status.rs` (`HealthReport` — including any additive capability-vs-index field
it chooses to add; **U3 added none**), the flag **derivation** in `src/store/mod.rs`'s status producer, the **V-8.x**
golden literals + their fixture helpers in `tests/wire_conformance.rs`, and the §9.1/§12 text — **and
nothing in the canonical `EngineSubsystems` type** (`docs/specs/gnosis.md:874`), which is frozen (see
the F7 rule above: it MUST NOT gain, lose or re-type a field, or **twenty-three** `tests/` struct literals break —
(*REMAND-2 SHOULD-FIX 6, 2026-09-22: the "twenty-five" this paragraph carried, and the "nine → sixteen added"
composition behind it, are superseded — the current figure is **23** with the per-file enumeration published
in §9.1's F7 note above (which is also `docs/specs/p2-gnosis-server.md` §5.8's figure); the "nineteen" this
paragraph previously noted as an under-count is likewise historical. No U5 literal is added by the boot
build — U5 adds no `EngineSubsystems` construction — so the figure is 23 before and after U5.*)

---

## 10. `envelope.rs` — helpers (recap of §4.1)

`Envelope` is `Serialize`/`Deserialize` (the TestWriter round-trips it through
`serde_json::to_string`/`from_str` or via `to_json`/`from_json`). `to_json` never
fails for our payload types; `from_json` returns `DecodeError::InvalidJson` on a
parse error and `DecodeError::InvalidEnvelope` when any of the three required
top-level fields are absent or wrongly typed. `from_json` does **not** itself
validate `schema_version`/`id_format` against the constants (that is a
`decode_chunk`/transport concern); unknown values surface as
`UnsupportedSchemaVersion`/`UnknownIdFormat` at the decode step.

---

## 11. Documented HTTP-status map (reference-only — rendered by the shell unit, not built here)

This is a **documentation table for the Astrographer shell unit**, which owns the
HTTP transport + status rendering. F2 ships **no** HTTP code and **no** status
rendering; the mapping is frozen here so the shell (and a TestWriter's conformance
assertions, where relevant) implements it identically. `ConflictError` = **409**
is **mandated** (FS-4, `docs/specs/gnosis.md` §4.1.4).

**(U1): the map stays exactly 21 rows / no new wire codes.** This table is **unchanged** by U1 and by
every remaining unit of the GR-1..GR-9 workstream: no `StoreError` variant is added, no row is added,
and no code is renamed. In particular **no `stale_revision`** (the revisioned projection is refused —
§4.6) and **no `decode_failed`** (a request-decode failure is the NEW-2 **transport-level** outcome —
400/422, never 502, never a row here; §7.1). *Status line: since P2 landed, the **engine** renders this
map (`server_status`, `src/server.rs:18-44`; `docs/specs/p2-gnosis-server.md` §7.1) and the shell
consumes it; the table's frozen contents are what both sides must agree on. The **which-variant-fires**
ruling for the retrieval surface is §16.*

| wire code | `StoreError` | documented HTTP status | rationalization |
| --- | --- | --- | --- |
| `"not_found"` | `DocumentNotFound` | 404 | FS-1 |
| `"wiki_not_found"` | `WikiNotFound` | 404 | FS-2 |
| `"validation_error"` | `ValidationError` | 400 | FS-3 |
| `"conflict"` | `ConflictError` | **409** | FS-4 (mandated) |
| `"doc_in_use"` | `DocumentInUse` | 409 | FS-5 (referential delete conflict) |
| `"invalid_state"` | `InvalidState` | 409 | FS-6 (state-transition conflict) |
| `"unresolved_reference"` | `UnresolvedReference` | 422 | FS-7 (publish gate, unprocessable) |
| `"engine_unavailable"` | `EngineUnavailable` | 503 | FS-8 |
| `"engine_error"` | `EngineError` | 502 | FS-9 (upstream produced malformed result) |
| `"trace_unavailable"` | `TraceUnavailable` | 502 | FS-10 |
| `"hop_limit_exceeded"` | `HopLimitExceeded` | 422 | FS-11 |
| `"cycle_detected"` | `CycleDetected` | 409 | FS-12 |
| `"embedding_unavailable"` | `EmbeddingUnavailable` | 503 | FS-13 |
| `"vector_index_unavailable"` | `VectorIndexUnavailable` | 503 | FS-14 |
| `"lexical_index_unavailable"` | `LexicalIndexUnavailable` | 503 | FS-15 |
| `"reranker_unavailable"` | `RerankerUnavailable` | 503 | FS-16 |
| `"compression_failed"` | `CompressionFailed` | 500 | FS-17 |
| `"hyde_generation_failed"` | `HyDEGenerationFailed` | 500 | FS-18 |
| `"multi_query_expansion_failed"` | `MultiQueryExpansionFailed` | 500 | FS-19 |
| `"community_not_found"` | `CommunityNotFound` | 404 | FS-25 |
| `"sub_task_dag_failed"` | `SubTaskDagFailed` | 500 | FS-26 |

---

## 12. Golden conformance vectors

Exact bytes/JSON a conforming implementation (the engine F2 codecs and the later
shell) must reproduce. All envelopes use `schemaVersion:1`,
`idFormat:"opaque-string-v1"`.

**V-1 — `encode_chunk(&RagChunk::Done)`:**
```
{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"type":"done"}}
```

**V-2 — `encode_chunk(&RagChunk::Error(StoreError::ConflictError))`:**
```
{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"type":"error","code":"conflict","message":"optimistic-concurrency conflict: stale base revision"}}
```

**V-3 — `encode_chunk(&RagChunk::Error(StoreError::ValidationError("empty query".into())))`:**
```
{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"type":"error","code":"validation_error","message":"empty query"}}
```
(`"message"` carries the variant's detail; the code stays `"validation_error"`.)

**V-4 — `encode_error(&StoreError::WikiNotFound)` (standalone non-chunk codec):**
```
{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"code":"wiki_not_found","message":"wiki not found"}}
```

**V-5 — `encode_result(r)` for a minimal flat `RagResult`**
(`query:"q"`, one item, `engine:"gnosis"`, flat `TraceDescriptor` trace). This is the
**exact** byte output of `serde_json::to_string` on the frozen `RagResult`/
`RagResultItem`/`TraceDescriptor`/`Source`/`QueryMode` values below: snake_case keys,
PascalCase enum values, external-tagged `RagTrace`, id newtypes as bare strings,
`null` for absent `parent`/`stale`/`blocked_by`:
```
{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"query":"q","results":[{"document_id":"d1","node_id":"n1","score":0.9,"snippet":"s","source":"Local","parent":null,"stale":null}],"engine":"gnosis","citations":[["d1","n1"]],"trace":{"Flat":{"mode":"Flat","engine":"gnosis","top_k":10,"source":"Local"}},"blocked_by":null}}
```
(`blocked_by` is `null` when absent — the field is `#[serde(default)]`, which only
defaults **deserialization**; the field is always **serialized**.)

**V-6 — the three SSE event frames** (from `encode_event`):
- `RagChunk::Done`:
  ```
  event: done
  data: {"type":"done"}
  <blank line>
  ```
- `RagChunk::Error(ValidationError("empty query".into()))`:
  ```
  event: error
  data: {"type":"error","code":"validation_error","message":"empty query"}
  <blank line>
  ```
- `RagChunk::Result(r)` (the §12 V-5 result): `event: result` + `data: {"type":"result","result":{…V-5 payload body…}}`.

**V-7 — three representative `wire_code` samples:** `DocumentNotFound`→`"not_found"`,
`ConflictError`→`"conflict"`, `EngineUnavailable`→`"engine_unavailable"`.

**V-8 — health reports** (canonical `HealthReport` JSON — camelCase top-level per §9, `state` the real PascalCase `EngineState`; `subsystems` flags single-word, unchanged):
- Ready engine (`EngineStatus{state:Ready, version:"…", subsystems:all-true, last_error:None}`):
  ```
  {"schemaVersion":1,"idFormat":"opaque-string-v1","state":"Ready","version":"…","subsystems":{"store":true,"graph":true,"lexical":true,"vector":true,"embedding":true,"reranker":true},"lastError":null}
  ```
- Degraded engine (`state:Degraded, subsystems.embedding:false,
  last_error:Some("a non-core subsystem (embedding/reranker) is unavailable")`):
  ```
  {"schemaVersion":1,"idFormat":"opaque-string-v1","state":"Degraded","version":"…","subsystems":{"store":true,"graph":true,"lexical":true,"vector":true,"embedding":false,"reranker":true},"lastError":"a non-core subsystem (embedding/reranker) is unavailable"}
  ```

**V-8 status (U1): the literals above are the PRE-U1 masks, and two of their values are now known-false
claims.** Under §9.1's capability semantics the degraded fixture's `"vector":true` and `"reranker":true`
(and the ready fixture's `"reranker":true`) are **defects**, not contract. **U1 therefore restates the
expected literals below as the post-U3 contract.** ***LANDED: U3 (2026-09-17)*** — **the amended literals are
now the ASSERTED contract and the pre-U1 literals above are dead**: `tests/wire_conformance.rs:1061-1112`
(`v8_health_reports_exact`) asserts V-8.1's U3-stage variant and V-8.2 byte-exactly, and
`boot_wiring_couples_to_the_derived_read` (`:1129`) pins the producer↔literal coupling, so the two blocks
below are the live contract while the two JSON blocks above are the historical pre-U1 record. The **shape**
rules are unchanged: camelCase
top-level (`schemaVersion`/`idFormat`/`lastError`), PascalCase `state`, single-word `subsystems` flags,
and the DEGRADED `lastError` string **unchanged** (it is the store's fixed message,
`src/store/mod.rs:4226-4230`; the pre-U3 citation was `:4189-4193`; rewording it is not authorized).

**V-8.1 — the amended READY literal** (mask: core all true, `lexical` true, `embedding` true,
`reranker` **false**, `vector` **true** once the index is built):

```
{"schemaVersion":1,"idFormat":"opaque-string-v1","state":"Ready","version":"…","subsystems":{"store":true,"graph":true,"lexical":true,"vector":true,"embedding":true,"reranker":false},"lastError":null}
```

- `reranker:false` is **unconditional** in every reachable state (no reranker implementation exists in
  `src/`), so it is part of the U3 contract.
- `vector:true` in this literal presumes an index exists. **U3-stage variant (no boot index build yet,
  i.e. until U5 lands): the same literal with `"vector":false`** — `{"store":true,"graph":true,
  "lexical":true,"vector":false,"embedding":true,"reranker":false}`. The **U5** unit flips that one
  value to `true` **in the same unit as the boot build**, per the golden-literal discipline below.
  ***(LANDED — U5, 2026-09-22: the one-value flip is the ASSERTED contract, gate 8 re-read it this pass —
  `tests/wire_conformance.rs:1099-1112`'s `v8_health_reports_exact` asserts `"vector":true` at `:1109` with the
  post-U5 READY-mask message at `:1110-1111`, `honest_ready_subsystems()` (`:718`, its post-U5 doc comment
  `:708-717`) projects `vector: true` at `:723`, and the probe-1 input is index-bearing
  (`boot_wiring_couples_to_the_derived_read`, `:1144`/`:1208-1223`); the U3-stage paragraph above stands as the
  pre-U5 record.***

**V-8.2 — the amended DEGRADED literal** (mask: core all true, `lexical` true, `embedding` **false**,
`vector` **false** — the index is not built —, `reranker` **false**):

```
{"schemaVersion":1,"idFormat":"opaque-string-v1","state":"Degraded","version":"…","subsystems":{"store":true,"graph":true,"lexical":true,"vector":false,"embedding":false,"reranker":false},"lastError":"a non-core subsystem (embedding/reranker) is unavailable"}
```

- **Reachability (pinned, no invention).** V-8 is a **codec** vector: it is the exact projection of the
  **fixture's** `EngineStatus` mask; before U3 the fixture's mask was producible only by construction/
  `set_subsystems` (`src/store/mod.rs:1868-1876` — a write-only hook after U3; the pre-U3 citation was
  `:1862-1863`). **The live boot path now produces these masks too (U3 LANDED 2026-09-17):** the boot applies
  `boot_wiring`'s `EngineState` plus its own wiring and writes **no** flag mask (`src/bin/gnosis_server.rs:378-408`
  at U3; **the post-U5 extent is `:381-437`** — gate 8, 2026-09-22),
  so a provider-reachable boot derives the **V-8.1 U3-stage** mask (`vector:false` until U5), a
  configured-but-unreachable boot derives **V-8.2** exactly (live-verified: `state:"Degraded"`,
  `embedding:false`, `vector:false`, `reranker:false`, core `true`, with the pinned `lastError`), and the
  provider-absent boot derives the same vector with `state:"Unavailable"` and `lastError:null`. The
  pre-U3 statement — "the live boot path does **not** produce this mask yet; it hard-codes all six flags `true`
  (`:1718-1725`, the literal; the pre-U3 citation `:1710-1717`, and the `:1714-1725` spelling this sentence
  carried, are stale — the preceding lines `:1714-1717` are U3's own comment block) and never updates them, so a DEGRADED live engine reports
  `embedding:true`/`vector:true`/`reranker:true`" — is kept here as the historical record. **U5** owns the
  post-boot-index value of that fixture's `vector` flag — if the
  boot build produces an index even when the provider is unreachable, U5 flips that one value **in the
  same unit as the build**. ***LANDED — U5, 2026-09-22 (gate 8): the flip is in place and the premise is
  refined rather than altered — the boot build produces an index **iff the provider probe was `Reachable`**
  (an empty index included), so the `Unreachable`/`Absent` boots still derive **V-8.2** unchanged while the
  `Reachable` boot derives the amended V-8.1; the third U5 shape — a `Reachable` probe whose build **fails** —
  degrades with `vector:false`, `embedding:true`, `reranker:false` and has **no golden of its own** (asserted
  by `p2` §9.5.5's `P-IM-15`, lib-level only; the live criterion is PARKED, not live-verified).*** ***(REMAND-2 SHOULD-FIX 4, 2026-09-22: besides the `"vector"` value itself
  (asserted at `tests/wire_conformance.rs:1095`), the literal's **assertion message** at `:1096`
  ("V-8.1 ready report (amended U3 mask: reranker false, vector false until U5)") is one of U5's same-unit
  texts — it becomes the **post-U5 READY-mask wording** ("…vector true, reranker false"), because leaving
  "vector false until U5" in place next to an amended literal would state the superseded claim; and the
  producer probe's comment at `:1191` ("a REACHABLE provider is wired ⇒ `Ready`, no index") is reconciled in
  the same unit: after the `:1195` flip the probe feeds an **index-bearing** snapshot, so the comment's "no
  index" half is restated as the index-bearing input (its post-flip sense: the READY mask with an index and
  a wired provider). Neither text is a pinned value; both are named here so the "exactly two edits" reading
  does not leave falsified prose behind.***)

**(U5 note, 2026-09-22 — ADDITIVE; every clause above stands as the record.)** U5's answer to the question
the paragraph above leaves open is **pinned in `docs/specs/p2-gnosis-server.md` §9.5.5** (its contract table
(1)): the boot builds the index **iff** the provider probe was `Reachable`; a provider **unreachable or
absent** ⇒ **no** index is attempted ⇒ `vectors: None` ⇒ **`vector:false`**. Therefore, **in U5's unit**:
**V-8.1 flips its one value to `"vector":true`** (the mask of a provider-reachable boot, incl. an empty
store — an empty `VectorIndex` is `Some`) — **and the flip applies to the `Reachable` + `vectors: Some`
instance only: the historical clause's antecedent (a build producing an index *even when the provider is
unreachable*) is FALSE — U5 builds nothing for `Unreachable`** (***REMAND-2 NOTE 5 / REMAND-3 NOTE 5,
2026-09-22: recorded so the `if`-clause above is not read as a live condition; see `p2` §9.5.5's
entry-question table (1)***) — and **V-8.2 is unchanged** (`vector:false`, `embedding:false`,
`reranker:false`, the same `lastError` — a degraded boot **of the provider-`Absent`/`Unreachable` kind** wires no provider and builds nothing — ***REMAND-4 SHOULD-FIX 3, 2026-09-22: that gloss is **scoped** to those two boots — its producer probes are exactly `tests/wire_conformance.rs:1228` / `:1237` (verified by text **— gate-8 re-read, 2026-09-22: the landed lines; the `:1209`/`:1218` spelling this gloss carried is the U3-time record —**; probes (2)/(3) of `boot_wiring_couples_to_the_derived_read`, both wiring **no** provider) — and the **universal** phrasing the gloss previously carried ("a degraded boot wires no provider and builds nothing") is **superseded**: it no longer covers every `Degraded` state U5 creates, because U5's failed-`Reachable`-build branch yields the **same flag values** while **wiring** a provider — `Degraded`, `vector:false`, `embedding:true`, `reranker:false`, the same fixed `lastError` (`p2` §9.5.5's failure table (4)'s "what the boot does with that `Err`" row; the `P-IM-15` row). The **values never contradicted**; only the universal phrasing did. **The failed-build `Degraded` mask has no golden of its own:** no V-8 literal is edited for it, and it is asserted **only** by `p2`'s `P-IM-15` (lib level, in the order that failure row pins) and by the live battery row **`R-L3`**'s PASS **(v)**.*** The literal
edit stays what the discipline below pins: the fixture (`honest_ready_subsystems`,
`tests/wire_conformance.rs:713-723`), the byte-exact literal (`:1093-1097` — V-8.1's JSON at `:1095`, its
message at `:1096`; V-8.2's at `:1109-1110`), the producer-coupling probes
(`:1192-1239`) — **only probe (1) moves**: its input at `tests/wire_conformance.rs:1195` moves to
`boot_snapshot(true)`, while probes **(2)** at `:1209` and **(3)** at `:1218` **stay on `boot_snapshot(false)`**
(their fixture is `honest_degraded_subsystems()`, `:726-735`, and their wiring wires **no** provider, so an
index-bearing snapshot would make the derived read `vector:true` and falsify a correct degraded assertion), and
probe **(4)**'s input at `:1231` is **already** index-bearing (`boot_snapshot(true)`). ***(REMAND-2 MUST-FIX 3,
2026-09-22: this clause previously read "the producer-coupling probes (`:1192-1239`), whose `boot_snapshot`
inputs move to the index-bearing shape)" — the plural reading was superseded by `docs/specs/p2-gnosis-server.md`
§9.5.5's move table (only probe (1) moves); a TestWriter flipping probes (2)/(3) would break the V-8.2
producer couple. Probe (1)'s remaining same-unit edits are its assertion at `:1204` (`!reached.vector` ⇒
`reached.vector`) and the assertion **message** at `:1205`, whose "…`embedding:true` with no index and no
reranker…" wording becomes the post-U5 READY-mask wording — REMAND-2 SHOULD-FIX 4 — while the comment at
`:1191` ("a REACHABLE provider is wired ⇒ `Ready`, no index") is reconciled in place: with `:1195` flipped
the probe feeds an **index-bearing** snapshot, so the comment's "no index" half describes the pre-U5 input
and is restated as the index-bearing input.)*** ***(GATE-8 ANCHOR RE-READ (2026-09-22) — the landed
`tests/wire_conformance.rs` lines for everything this U5 note names, so the U3-time numbers above cannot be
mistaken for current: the fixture `honest_ready_subsystems()` at **`:718`** (body `:719-726`, post-U5 doc comment
`:708-717`); the byte-exact literal at **`:1109`** (message **`:1110-1111`**); V-8.2's at **`:1124`**;
`honest_degraded_subsystems()` at **`:740`** (doc comment `:729-739`); probe (1) at **`:1208-1223`** (input
`boot_snapshot(true)` **`:1213`**, assertion **`:1222`**, message **`:1223`**, comment **`:1208`**), probes (2)/(3)
at **`:1228`**/**:1237`**, probe (4) at **`:1244-1256`** (input **`:1250`**, `u5_expected` **`:1253`**). **Every
same-unit edit this note demands is LANDED** — the fixture and the literal carry `"vector":true`, the two prose
texts are reconciled, and the redundant `u5_expected.vector = true;` line is gone.*** — **the type rules are
untouched: `EngineSubsystems` keeps six `bool`s, `HealthReport` keeps its six top-level keys, and the §11
map stays 21 rows.**

**Golden-literal discipline (pinned).** V-8 is a **codec** vector (`health(&EngineStatus{…})` is a
faithful, deterministic projection of whatever mask the fixture supplies — §9.1's register note), so
the literals are exact projections of the fixture's mask. The **literal edit lands in the same unit as
the flag change that makes the mask honest**: `tests/wire_conformance.rs:1061-1116` (the `v8_health_reports_exact` golden, its `fn` at `:1084`,
amended to the V-8.1 U3-stage / V-8.2 literals; the pre-U3 citation was `:1024-1059`; **REMAND-2 re-read:
V-8.1's JSON at `:1095` + its message at `:1096`, V-8.2's at `:1109-1110` — the `:1093-1111` spelling this
note and §9.1 carried names the assertion block, not the whole golden**) + the fixture helpers
(`all_true_subsystems`, `:697-706`; `honest_ready_subsystems`, `:713-723`; `honest_degraded_subsystems`, `:726-735`) is edited in **U3** — **LANDED 2026-09-17**, with the producer coupling pinned in the same file
(`boot_wiring_couples_to_the_derived_read`, `:1129-1240`) (and its `vector` value again in **U5** if the boot build changes
that fixture's mask — U5 **does** flip it, per the U5 note above). **Transient divergence (RESOLVED):** between U1 and U3, the spec text carried
the amended literals while the test still asserted the pre-U1 ones — a documented contract-ahead-of-code state,
per the gate-1 blast-radius row. **U3 closed it in the same unit:** the test now asserts the amended literals
and the code emits them, so the divergence no longer exists (`boot_wiring_couples_to_the_derived_read` fails the
file if the derivation ever drifts from the literals).

**V-9 — decode-then-validate vector pair:**
- A body missing `trace` (trace-presence checked first) → `DecodeError::MissingTrace` → `outcome_of` → `RagChunk::Error(StoreError::TraceUnavailable)`.
- A truncated/`null` body, or wrong field type **with a `trace` key present** → `DecodeError::InvalidJson(_)` → `outcome_of` → `RagChunk::Error(StoreError::EngineError)`.
- **Precedence (pinned):** `decode_rag_result` checks trace-`key`-presence BEFORE structural type-deserialization, so a body that is both missing `trace` AND structurally malformed (e.g. `{"query":123}`) yields `MissingTrace` → `TraceUnavailable` (not `EngineError`). This keeps FS-10 reachable (a well-formed-but-traceless body would otherwise fail the required-`trace` serde parse and map to `EngineError`); both codes map to HTTP 502 at the shell so the rendered status is identical.

**U1-pinned vectors (V-15..V-19).** These pin the exact bytes of the U1 amendment's new surfaces. They
are **goldens for the units that land them** (U2 for V-15/V-18, U4 for V-16/V-17, codec-level for
V-19) — **no code lands in U1**, and each is labelled with its unit so a TestWriter does not expect it
early. **(2026-09-16, annotation only:** V-15.1's binding note restates that its `message` string is the
**implemented renderer's own output** — the code/status/header/keyset are the byte-binding pins and the
golden asserts the structure plus the derived string, not a hand-written literal; `p2` §5.5's `message`
rule (the four string-carrying variants verbatim/may-be-empty; `UnsupportedSchemaVersion(v)` non-empty) is
**unchanged**, and V-15's own literals remain pinned messages. The companion pin is `p2` §5.3's
two-layer reading of its "absent ⇒" column.)**

**V-15 — the transport request-decode error body (§7.1; U2).** Two byte-exact bodies, each with
`Content-Type: application/json` and **no** envelope wrapper. Body for
`DecodeError::UnknownMethod("bogus")` — HTTP **422**:

```
{"code":"unknown_method","message":"bogus"}
```

Body for `DecodeError::UnsupportedSchemaVersion(99)` — HTTP **400**:

```
{"code":"unsupported_schema_version","message":"unsupported schemaVersion: 99"}
```

**V-15.1 — the QUERY-path non-object-payload body (§7.1; U2; added by the register remand, F8/F9).** The
`p2` register's `P-IM-4` pins **one** transport code for a non-object `payload` on `POST /rag/query`
(`InvalidEnvelope` — `InvalidJson` is unreachable there), so its rendered bytes get their own vector. A
request envelope whose `payload` is a JSON scalar/array (e.g. `{"schemaVersion":1,"idFormat":
"opaque-string-v1","payload":5}`) — HTTP **400**, `Content-Type: application/json` (no `charset`), no
envelope wrapper:

```
{"code":"invalid_envelope","message":"envelope payload must be a JSON object"}
```

- **What the vector binds:** the **code**, the status, the exact header, and the **key set/order**
  (`code`, `message`) — the `message` text is the carried string and is pinned only in that it is
  **verbatim** (it may be empty for the four string-carrying variants — `p2` §5.5's restated message
  rule). **The literal above is a SUGGESTED string, not a pin (restated so it is not read as one):** the
  `message` for this case is the **implemented renderer's own output** — the string the U2 decoder carries
  in `InvalidEnvelope(…)` — so `v15_request_decode_error_body_exact` MUST assert the **structure**
  (`Content-Type: application/json`, no `charset`, no envelope wrapper, the exact status **400**, the
  `code`+`message` key set/order, `code == "invalid_envelope"`) **plus** the string **derived from
  the implementation** (the decoder's carried string, read from the same source), and MUST NOT assert the
  literal `"envelope payload must be a JSON object"` as a byte-exact contract token — a hand-written
  literal here would pin an implementation detail as if it were pinned wire text. The byte-binding pins are
  therefore **code + status + header + keyset**; the `message` value is derived, not dictated.
  (`p2` §5.5's `message` rule is **unchanged** by this note: the four string-carrying variants verbatim and
  MAY be empty; `UnsupportedSchemaVersion(v)` the one guaranteed-non-empty render.) V-15's two bodies,
  whose literals **are** the pinned messages of their cases, are unaffected, and the golden for this case is
  created/asserted by the conformance test **`v15_request_decode_error_body_exact`** in
  `tests/wire_conformance.rs` (naming convention of
  `v8_health_reports_exact`/`v9_decode_then_validate_routes_exact`), driven at the transport by the
  `malformed_request_returns_400`-family e2e cases. V-15's two bodies and V-15.1's one body are **U2
  goldens**; no code lands in U1.

**V-16 — the `GET /changes` cursor-first frame (§4.7; U4).** Framing is §4.4's single event:

```
event: cursor
data: {"type":"cursor","cursor":"7"}
<blank line>
```

**V-17 — one `GET /changes` change frame (§4.7; U4).** Field order is fixed by the frame table's order
(`type`, `cursor`, `wikiId`, `kind`, `nodeIds`, `edgeIds`, `timestamp`); `wikiId` is `null` when the
entry is not wiki-scoped; `nodeIds`/`edgeIds` are always present (possibly empty):

```
event: change
data: {"type":"change","cursor":"8","wikiId":"w1","kind":"create_document","nodeIds":["n1"],"edgeIds":[],"timestamp":"2026-09-16T00:00:00Z"}
<blank line>
```

**V-16/V-17 are CODEC-LEVEL vectors over a SYNTHETIC `JournalEntry` (F6; the label the pre-remand text
lacked).** Both frames are the **exact output of `encode`/frame-building for a hand-constructed
`JournalEntry`**, not an assertion about what a live engine emits:

- The `timestamp` literal above is **synthetic** (`"2026-09-16T00:00:00Z"` — a **seconds-precision**
  form). **`iso_now()` never emits that shape**: it renders **9-digit fractional seconds**
  (`format!("{y:04}-{mo:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}.{nanos:09}Z")`,
  `src/store/mod.rs:5496`), so a real committed entry's `timestamp` looks like
  `"2026-09-16T00:00:00.123456789Z"`. A TestWriter MUST NOT derive a seconds-precision expectation for
  a live entry from V-17.
- **What the vector binds:** the frame must reproduce **that `JournalEntry`'s own `timestamp` string
  verbatim** — the codec copies the field, it never parses/re-formats/normalizes it. The golden
  therefore pins **field order, key set, nullability and types**, and the **verbatim-copy rule**; the
  literal timestamp value is an opaque input, exactly as `wikiId`/`kind` are.
- **The live-entry expectation (also pinned):** for a live feed, `timestamp` is the committed entry's
  `iso_now()` output — 9-digit fractional seconds, UTC, `Z`-suffixed. A conformance test over a live
  commit asserts the **shape** (`YYYY-MM-DDThh:mm:ss.fffffffffZ`), not V-17's synthetic literal.
- `cursor "7"`/`"8"` are likewise synthetic inputs (the vector pins the **decimal-string** encoding and
  the +1 relation between the two frames, not those two values).

**V-18 — the `ragQuery` request envelope + the response body shape (§4.5; U2).**

Request (canonical; it shows **four** of the **14** keys §4.5 documents — `query`, `mode`, `topK`,
`wikiId` — and absent keys are simply omitted; F17(ii)):

```
{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"query":"q","mode":"hybrid","topK":10,"wikiId":"w1"}}
```

Response (HTTP 200): the §4.1 envelope whose `payload` is **exactly V-5's payload body** — the bare
`RagResult`, **not** the `{"type":"result","result":…}` chunk:

```
{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"query":"q","results":[…],"engine":"gnosis","citations":[…],"trace":{…},"blocked_by":null}}
```

**V-19 — the cross-family rejection (§4.7; U4-time route, but reachable today at the codec level).**
`decode_event` applied to a V-16/V-17 frame → `DecodeError::UnknownType("cursor")` /
`DecodeError::UnknownType("change")` (the catch-all at `src/wire/decode.rs:114`, exercised today by a
hand-built frame), i.e. the change family can never be mistaken for a `RagChunk` frame.

---

## 13. Valid/happy + fail states per function (TestWriter assertion guide)

| wire fn | valid/happy | fail-state |
| --- | --- | --- |
| `StoreError::wire_code()` | returns a stable, non-empty, unique code for all 21 variants | none (total, pure) |
| `StoreError::from_wire` | canonical code → that variant; `validation_error` + message → `ValidationError(m)` | unknown/foreign code → `None` |
| `encode_chunk` | any `RagChunk` → envelope with canonical JSON (result body via serde) | none (serialize can't fail on our payloads) |
| `decode_chunk` | round-trips all three variant classes | `InvalidJson`, `UnknownType`, `MissingTrace`, `UnsupportedSchemaVersion`, `UnknownIdFormat`, `InvalidEnvelope`, `EventTypeMismatch`, `ValidationFailed` |
| `encode_error` | 21 variants → `{"code","message"}` envelope | none |
| `decode_error` | canonical `{code,message}` → that variant | `UnknownCode` for a non-canonical code; `InvalidJson` for a bad shape |
| `encode_result` | any well-formed `RagResult` → envelope wrapping the serde body | none |
| `decode_result` | well-formed body, trace present, engine `"gnosis"` → `Ok(r)` | missing trace → `MissingTrace`; malformed → `InvalidJson`; invariant break → `ValidationFailed` |
| `validate_rag_result` | valid result → `Ok(())` | **its two landed arms**: `WrongEngine` (`engine != "gnosis"`); `BlockedByWithoutGraphTrace` — `MissingTrace` is **NOT** a `validate_rag_result` outcome (it is `decode_rag_result`'s, trace-`key`-presence-checked first, `src/wire/decode.rs:48-58`, `:63-71`; a traceless `RagResult` is not representable — `RagResult.trace` is not an `Option`). The `ValidationFailure` **enum** does carry the `MissingTrace` variant (`:36-43`), which no validator path constructs. |
| `encode_event` | any `RagChunk` → single SSE frame | none |
| `decode_event` | round-trips all three event types | frame malformed (`InvalidEnvelope`), bad JSON (`InvalidJson`), event/data type mismatch (`EventTypeMismatch`), result-body fails validation |
| `health(&EngineStatus)` | any `EngineStatus` → deterministic `HealthReport` | none (pure) |
| `Envelope::to_json`/`from_json` | round-trips `schema_version/id_format/payload` | `from_json`: `InvalidJson` (parse), `InvalidEnvelope` (missing/typed-weak field) |
| **(U1/U2)** the shared envelope check on the `ragQuery` path | a well-formed envelope with `schemaVersion:1` + `idFormat:"opaque-string-v1"` passes; `schemaVersion` is checked before `idFormat` | `UnsupportedSchemaVersion(v)` for any `v != 1` (checked first) → transport 400; `UnknownIdFormat(s)` for any other format → transport 400; a bare/non-object body → `InvalidJson`/`InvalidEnvelope` → 400 (**the non-object-payload half was U2-TIME and has LANDED, F13 — the shared decoder's object check makes it `InvalidEnvelope` ⇒ 400 `invalid_envelope`, `src/wire/query.rs:83-87`**; the pre-U2 tree yielded `query=""` ⇒ 400 `validation_error`) |
| **(U1/U2)** the shared token rule (`mode`/`expand`/`compression` ⇒ typed enums) | absent ⇒ `Flat` (and the documented defaults for the other two); `flat|graph|vector|hybrid`, `none|parent`, `none|filter|extract|graph` in any ASCII casing ⇒ the typed enum; a present-but-wrongly-typed value ⇒ **absent** ⇒ default (only a **string** reaches the token check — N2) | any other **string** value (incl. `""`, whitespace, unknown token) ⇒ `StoreError::ValidationError` → **400 `validation_error`** — on POST for all three tokens; on SSE (`?mode=` only, F4) **HTTP 400 + one `error` frame**; **N1:** `expand`/`compression` have **no SSE param**, so `?expand=`/`?compression=` is **ignored ⇒ that option's default, never a 400 and never an `error` frame**; **never** `DecodeError`, **never** 422; the check lives in the **U2 wire resolver** and NOT in `validate_rag_options` (F2) |
| **(U1/U2)** the `filters` mapping | the canonical §4.5.2 tokens map to the frozen store types, and the decoded `filters` is non-`None` and honored by the engine (never an all-`None` `QueryAuditFilters`) | an out-of-set `nodeKind`/`edgeType`/`state` token or a malformed `target` ⇒ 400 `validation_error`; a raw `serde_json::from_value::<QueryAuditFilters>` pass-through is **forbidden** (silent all-`None` — the GR-2 defect class) |
| **(U1/U2)** the transport decode-code mapping | the 5 request-decode variants ⇒ their documented code (§7.1), all non-empty; each code parses out of the JSON body | `None` for the other 5 `DecodeError` variants; **not** a `StoreError` (a store error keeps its §11 code); `from_wire(code, …) == None` for all five |
| **(U1/U2)** the `ragQuery` payload decoder | `query` (string) + the documented camelCase options (**14** keys, §4.5); unknown/extra keys tolerated (incl. `args`, `method`, `requester`); `query` absent/non-string ⇒ `""`; a wrongly-typed optional value ⇒ absent ⇒ default | no payload-level decode failure of its own: option-range failures are surfaced by the engine's `validate_rag_options` → 400 `validation_error`, the token failures by the wire resolver (§4.5) |
| **(U1/U2)** `encode_result` at the transport + `validate_rag_result` | any `rag_query` result today validates (engine `"gnosis"`, trace present) → 200 with the bare-`RagResult` payload | a validator-rejected body ⇒ `EngineError` → 502 (**not reachable today — the `Err` half is a U2-time totality requirement that LANDED 2026-09-17 at the pure level, `encode_result_checked`, `src/wire/query.rs:369-372`**) |
| **(U1/U4)** the change cursor accessor + `cursor` encoding | returns the current journal seq as a decimal **string**; `"0"` on a fresh store; +1 per committed entry; non-decreasing | none (total); it must **not** be used as a revision/validation token, and counting entries with it confers **no** store coherence (F14, §4.6) |
| **(U1/U4)** the `GET /changes` feed | a cursor-first frame, then one ordered `change` frame per committed entry; `event:` label equals the data `"type"`; `wikiId` nullable; `nodeIds`/`edgeIds` arrays (possibly empty); an **unknown `kind` is tolerated** as an opaque change (F11) | no §11-mapped fail-state on the subscribe path (no READY gate); `decode_event` (the `RagChunk` decoder) rejects a change-family frame with `UnknownType` |
| **(U1/U4)** the cursor resume token on a read route (F9) | a `cursor` field in a read request is a **resume token**; a comparable cursor resumes after it | a **not-comparable** cursor (stale/foreign/larger-than-current) is **tolerated but defined** — the request yields the **first page** (never silently ignored as absent, never a failure; U4's spec owes the pin) |

**Cross-cutting TestWriter states:**
- **Cross-wiki:** a `RagResult` whose items/citations reference documents from a
  wiki other than the requested one is a *data* concern of the engine (already
  guaranteed by §4.5); the wire codec does **not** enforce wiki scope and must
  round-trip such a result byte-for-byte (it is transparent to scope).
- **Unknown `id_format`:** decode of an envelope with `id_format:"uuid-v4"` →
  `UnknownIdFormat` → `EngineError` outcome (the RFC-4122 seam is documented, not
  implemented).
- **Unknown `schema_version`:** decode of `schema_version:99` →
  `UnsupportedSchemaVersion(99)` → `EngineError` outcome.
- **Empty `payload`:** `InvalidEnvelope` for a result/done chunk; a `done` chunk
  with a non-`{}` payload → still `Ok(Done)` only if payload is `{"type":"done"}`
  (exact), else `InvalidEnvelope`/`UnknownType`.

**Cross-cutting TestWriter states (U1 additions).**

- **Unknown-key tolerance (both tolerance rules together).** A `ragQuery` payload carrying extra keys
  (`args`, `method`, `requester`, or any other) decodes to the same options as without them; a
  `"method":"bogus"` key is **not** an unknown-method 422 on the query surface (that code is CRUD-only).
- **Two-error-vocabulary separation.** A §11-mapped failure keeps its `StoreError::wire_code()` and its
  §11 status; a transport decode failure uses §7.1's codes and the NEW-2 400/422 statuses; the two code
  sets are pairwise disjoint and the transport set has no §11 row.
- **Mode precedence.** An unrecognized `mode` token is a 400 even when the engine is not READY (the
  token rule is a **wire-resolver** outcome, before `validate_rag_options` and before the READY gate);
  absent `mode` on a not-READY engine is a 503 (`docs/specs/p2-gnosis-server.md` §5.3/§5.4). The
  resolver — not `validate_rag_options` — owns the token check (F2).
- **SSE mode parity (status included — F4; scoped to `mode` — N1).** For the **`mode`** token the same
  inputs give the same outcome on both paths: the SSE path returns **HTTP 400** with an `error` frame
  carrying `code:"validation_error"` exactly where the POST path returns 400 `validation_error` — the
  status is pinned, not only the frame. `expand`/`compression` are **POST-only** (§4.5): the SSE surface
  has no such param, so there is **no SSE outcome to compare** and a `?expand=`/`?compression=` param is
  **ignored ⇒ default, never a 400**.
- **`filters` is not inert.** A canonically-shaped `filters` payload yields a non-`None`,
  non-all-`None` filter that reaches the engine; a camelCase-keyed payload is decoded by the mapping
  (never by serde into `QueryAuditFilters`), and a store-cased payload (`"nodeKind":"Content"`,
  `target:["d1","n1"]`) is **not** the request shape (F1).
- **Wrong-type tolerance.** A wrongly-typed optional value (`topK:"10"`, **`topK:-1`/`topK:10.5`**, `hyde:"yes"`,
  `mode:5`,
  **`expand:5`**, **`compression:null`** …)
  decodes as absent ⇒ the documented default (`mode` ⇒ `Flat`; `expand`/`compression` ⇒ `'none'`), on
  **every path that reads the key** (N1: the two `expand`/`compression` keys exist on POST only) (F5);
  `query` (⇒ `""`) and `filters` (schema-checked; `{}` is a valid all-`None` filter — N5) are the two
  exceptions. **A present number that is not a `u64` belongs to this rule** (`as_u64` does not represent a
  negative or fractional number), whereas a **present `u64` outside its pinned range** is the separate FS-3
  **400 `validation_error`** state (`topK:0`/`51`, `maxHops:0`/`6`, `binaryCandidatePool:0` with
  `binaryFirstPass:true`, `multiQuery:{enabled:true,n:0}` — `p2` §5.3's type-vs-range bullet; ruling 1,
  2026-09-16 — annotation only, no rule change). **The canonical corpus is `docs/specs/p2-gnosis-server.md` §5.3** (N2). The decoded **field** is
  `None` (layer 1, §4.5's two-layer reading), and for the two **object-valued** keys the same field-level
  reading decides a **wrongly-typed documented member** (`{"multiQuery":{"enabled":"yes"}}`,
  `{"multiQuery":{"enabled":true,"n":"3"}}`, `{"subTaskDag":{"enabled":"yes"}}` ⇒ `multi_query`/
  `sub_task_dag == None`; member-level defaults apply only inside a **well-formed** object: `{}` ⇒
  `Some({enabled:false, n:3})`, `{"enabled":true}` ⇒ `Some({enabled:true, n:3})`) — the member-type check is
  **total over the declared members** (`enabled` bool, `n` u64; ruling 2, 2026-09-16, the clause stands and is
  implemented) — the pin is `p2` §5.3
  (object-valued-option clause; U2-13 clarification, no rule change).
- **Cursor de-dup vs. revision.** Two frames with the same `cursor` are the same change (de-dup key);
  a consumer must not treat a cursor as a revision, and the payload's per-document `revision` remains
  the only concurrency token (§4.6).
- **Feed/family separation.** A change-family frame is never `RagChunk`-decodable (§4.7/V-19), and the
  retrieval `done` frame never terminates the change feed.

---

## 14. Cross-references and ownership hand-off

- **Contract authority:** `docs/specs/gnosis.md` §4.6.1, §4.1.5, §4.1.4, §4.3.4, §6.
- **U1 amendment authority (gate-1):** `docs/specs/gnosis-gr-inbound-review.md` — §The four rulings,
  §Per-GR verdicts (GR-1/GR-2/GR-3/GR-5), §Ordered workstream, §Blast radius + reconcile order,
  §Regression watch, §Appendix A (the change cursor), §Appendix B (FS-13/14/15), §Go-ahead record.
  Decisions `GNOSIS-CHANGE-CURSOR` + `SUBSYSTEM-FLAG-CAPABILITY-SEMANTICS` (`docs/decisions.md`).
- **U1 additions in this file:** the §U1 status notes (**incl. the 2026-09-16 REMAND note, the
  2026-09-16 REMAND-2 note and the 2026-09-16 U2 red-phase ambiguity-pin note**); §2 (the
  U1-pinned/units table + the two superseded guardrails, F16); §4.4 (the retrieval-family annotation,
  F10); §4.5 (incl. the corrected `filters` casing, F1, the token-check location, F2, and — 2026-09-16,
  annotation only — the **two-layer reading** of the "absent `mode` ⇒ `Flat`" / "wrongly-typed ⇒ ABSENT"
  bullets, cross-referencing `p2` §5.3's pin; **plus, 2026-09-16, the same annotation's object-valued half —
  the field-level reading of a wrongly-typed documented *member* of `multiQuery`/`subTaskDag` (non-object, or
  object-with-wrong-typed-member ⇒ the field is `None`; member defaults only inside a well-formed object),
  restated in §13's wrong-type-tolerance state — annotation only, the wire outcome unchanged, no register row
  and no rule changed; the pin is `p2` §5.3, per the blind-greens gate's U2-13 finding**); §4.6 (incl.
  the refused-vs-permitted cursor split, F9, and the coherence-token reword, F14); §4.7 (incl. the
  `kind` tolerance + rename binding, F11); §5 (the 21-row restatement); §7 (the `UnknownMethod` arm,
  F12); §7.1 (incl. the exact `Content-Type`, F15, the U2-time non-object row, F13, and — 2026-09-16,
  annotation only — the **which-part-is-pinned** note naming code/status/header/keyset as the byte-binding
  pins and the `message` as the renderer's own output); §9.1 (the
  `HealthReport`-only additive scope + the frozen-`EngineSubsystems` rule, F7); §11 (the 21-row/refusal
  note); §12 (the V-8 status, V-8.1, V-8.2, V-15..V-19, incl. the V-16/V-17 synthetic-entry label, F6);
  §13 (rows + cross-cutting states); §14/§15; §16 (incl. the §4.6.1 throw-column/hyde extension, F3).
- **Companion P2 amendment:** `docs/specs/p2-gnosis-server.md` §U1, §2, §5.2–§5.9, §7.1/§7.2, §9.5
  (**§9.5.1 = U2's rows, §9.5.2 = U3's rows, §9.5.3 = the shared execution plan**), §10,
  §11 (P-IM-3 restated + register notes), §12/§13. The route semantics + the HTTP rendering live there;
  this file pins the wire. **(U2/U3 update, 2026-09-16:)** the two code-bearing units are **authorized** and
  their typed rows are **authored in §9.5** (U2 ⇒ §9.5.1, U3 ⇒ §9.5.2) — **not** in this file's register,
  whose `P-SM-3` row stays untouched (§9.1).
- **Boundary C4 (as-of-N reconstruction):**
  `docs/research/astrographer-engine-shell-boundary.md:86` — ENGINE-owned, journal-retention, **not** a
  query route.
- **Gate record + scope guardrails:** `docs/specs/7-2-f2-review.md`
  (`F2-WIRE-CONTRACT-A1`); decision row `docs/decisions.md` `F2-WIRE-CONTRACT-A1`.
- **PBT:** `docs/specs/7-2-wire-property-register.md` (+ TestWriter's
  `tests/props_wire.rs`). **U1 adds no row there** (§9.1); the code-bearing units owe their own rows
  (named in `docs/specs/p2-gnosis-server.md` §11's register notes).
- **Conformance tests (TestWriter):** `tests/wire_conformance.rs` (the V-1..V-8 literals + fixture
  helpers are edited in **U3** with the flag change — **LANDED 2026-09-17** (`v8_health_reports_exact`,
  `:1061-1112`); **V-15/V-15.1/V-18 LANDED with U2 (2026-09-17)** —
  `v15_request_decode_error_body_exact`, `:1674` (the pre-U3 citation was `:1385`), asserts V-15's two bodies and V-15.1's query-path body
  byte-exactly, and V-18's request/response shape is asserted by the blind set's `U2-1` plus the e2e
  envelope cases — while **V-16/V-17/V-19 remain U4's**). **(Register-remand
  additions, 2026-09-16.)** Two U2/U3 obligations are homed here rather than in a property row:
  **V-15.1** (the query-path non-object-payload decode-error body — created/asserted by
  `v15_request_decode_error_body_exact`, F8 — **landed**) and **`health_report_shape_frozen`** (the frozen six-flag
  shape / `HealthReport` key set, F12 — **LANDED with U3**, `:1263`; the pre-U3 sentence read "owed by U3, not
  landed"). The U3 row ids this file's §9.1 cross-reference names are the
  **renumbered** ones (`P-IM-7`/`P-IM-8`/`P-IM-9`; F1 — see §9.1's remand paragraph).

**Owned by a later shell-integration unit** (not F2, per §2):
1. The HTTP-over-native-IPC decision + the server host (axum/hyper/tower deps).
2. The shell-side SSE client consuming the §4.4 event schema.
3. Bind-loopback + auth/TLS policy (recorded shell-owned) + loopback enforcement.
4. Full `RagStore` persistence CRUD routing.
5. The engine boot→READY lifecycle + its end-to-end transport test.
6. **Rendering** the §11 HTTP-status map (incl. `ConflictError`=409).
7. D2 engine-absent degrade **behavior** at the shell (F2 pins only the reporting).

### 14.1 REFUSED — engine query-result-shape parity (inbound `GRQ-8`); the translation duty is the CONSUMER's (2026-09-22)

**Authority.** `docs/specs/gnosis-grq-inbound-review.md` §7(v) (GRQ-8 ⇒ *"a documented REFUSAL with a named owner"*), its §14 POST-RECORD UPDATE 1 **Q4 = YES**, and reconcile-order step 6 (§15.2). **The answer to `GRQ-8` is a refusal, in terms** — a `docs`-only pass, the **justified zero-row exemption** (`PBT-GATE-MANDATORY`): no rule change, no §11 row, no register row, no red set, no `src/`/`tests/` change.

> **REFUSED: the engine does NOT emit the consumer's `ranked`/`context`/`markdown`/`lineMap`/`k` field set on its query result, and adds no engine field for it.** The engine's result shape is the canonical one and is **final** — `RagResult {query, results, engine, citations, trace, blocked_by}` (`src/store/mod.rs:757-766`) with `RagResultItem {document_id, node_id, score, snippet, source, parent, stale}` (`:688-699`); a tree-wide sweep of `src/` finds **no** `lineMap`/`markdown` field and no result-carried `ranked`/`k` (the only `ranked` tokens are internal sort vectors in `src/retrieval/mod.rs:67-75` and `src/store/mod.rs:5027-5042`). The consumer already **decided deliberately** to keep the two shapes distinct (**`DECIDED: ENGINE-RAG-RESULT-TRACE`**, a **consumer-tree decision** — cross-repo and **unverifiable from this role**), so emit-parity would add a **third result vocabulary** alongside the consumer's local fan-out shape and its proxy-specific `EngineRagResult`, plus new golden/validator surface on this side, for a render path the consumer's own decision says engine queries **never feed**. **The translation duty is the CONSUMER's**, and it is discharged consumer-side by the existing proxy-specific `EngineRagResult` mapping: the engine emits `RagResult` and nothing else. **No engine behavior changes and no engine field is added**; an engine-side parity block would require a wire-version decision and a new result-shape field, **neither of which this refusal grants**. Cross-references: record **§7(v)**; §4.5 (the pinned `ragQuery` response body — `RagResult`, unchanged); §13's `decode_result`/`validate_rag_result` rows (the shape's valid/fail states, unchanged); the consumer ask `<Astrographer>/docs/feature-requests/gnosis-engine-prerequisites.md` **GRQ-8** (its own "Fix shape" cell already permits this branch: *"a one-paragraph documented refusal naming the translation owner"*).

### 14.2 Appendix — per-route transport posture over the 11 document-CRUD request paths (inbound `GRQ-9`) (2026-09-22)

**Authority.** `docs/specs/gnosis-grq-inbound-review.md` §7(v) (GRQ-9 ⇒ *"a docs APPENDIX"*), its §14 POST-RECORD UPDATE 1 **Q4 = YES** (*"the GRQ-9 per-route GET-with-body posture appendix"*), and reconcile-order step 6. **This appendix records a posture; it changes no route behavior and adds no enforcement.** The frozen CRUD wire (paths, args, bodies) lives in `docs/specs/p1a-document-crud-wire.md` §4.2/§7/§10 and is **not** amended here.

**The 11 pinned paths are P1a's (`ENGINE_ENDPOINTS`, `p1a` §7, `:469-481`), registered verbatim in the 14-row `route_bijection()` (`src/server.rs:68-87`) and mounted on 9 live `router()` rows** where two rows carry two verbs each — `/documents` (`get`+`post`) and `/documents/:id` (`get`+`delete`), `src/bin/gnosis_server.rs:291-305`. **Every** CRUD route — read and mutating alike — is served by the **one** `crud_handler` (`src/bin/gnosis_server.rs:192-202`, mounted at `:296-303`), so **the request body is the only input**: no handler reads a `:id` path segment (grep for `Path`/`:id` in the bin: the only `:id` tokens are the route patterns at `:297`/`:303`), and the method's args — including `documentId`/`wikiId` — come from the envelope's `payload.args` (`p1a` §4.2). **A body therefore is not optional on any of the 11 paths, on either verb:** an absent, empty or non-JSON body reaches `Envelope::from_json` (`src/wire/envelope.rs:41-74`) and fails as **`invalid_json`** before any method/args decision is taken.

| # | route (pinned path, verb) | body required? | served by | decode-error body (a body that fails to decode) |
| --- | --- | --- | --- | --- |
| 1 | `POST /documents` (`createDocument`) | **yes** — required envelope | `crud_handler` | **400** `{"code":"invalid_json"\|"invalid_envelope"\|"unsupported_schema_version"\|"unknown_id_format","message":"<carried>"}`, `Content-Type: application/json`; **422** `unknown_method` for a well-formed body naming no `CrudMethod` |
| 2 | **`GET /documents/:id`** (`getDocument`) | **yes** — the `documentId` is read **only** from the body | `crud_handler` | as row 1 (a body absent/empty/non-JSON ⇒ 400 `invalid_json`) |
| 3 | `POST /documents/:id/update` (`updateDocument`) | **yes** | `crud_handler` | as row 1 |
| 4 | `DELETE /documents/:id` (`deleteDocument`) | **yes** | `crud_handler` | as row 1 |
| 5 | `POST /documents/:id/publish` (`publishDocument`) | **yes** | `crud_handler` | as row 1 |
| 6 | `POST /documents/:id/unpublish` (`unpublishDocument`) | **yes** | `crud_handler` | as row 1 |
| 7 | `POST /documents/:id/archive` (`archiveDocument`) | **yes** | `crud_handler` | as row 1 |
| 8 | **`GET /documents`** (`listDocuments`) | **yes** — `wikiId` + the `ListDocumentsFilter` body live in the envelope (`p1a` §4.2; both args fields are **required**, `src/wire/crud.rs:271-276`) | `crud_handler` | as row 1 |
| 9 | `POST /wikis` (`createWiki`) | **yes** | `crud_handler` | as row 1 |
| 10 | **`GET /wikis/:id`** (`getWiki`) | **yes** — the `wikiId` is read **only** from the body | `crud_handler` | as row 1 |
| 11 | **`GET /wikis`** (`listWikis`) | **yes** — `listWikis` is the one **empty-`args`** method (`p1a` §4.2), but the **envelope** is still required | `crud_handler` | as row 1 |

**The four GET rows are rows 2, 8, 10, 11** — exactly the four **read-only** methods (`getDocument`/`listDocuments`/`getWiki`/`listWikis`, `p1a` §8). **GET-with-a-request-body is therefore the normal posture of all four**, and the engine **accepts** such a request: axum's whole-body `String` extractor (`src/bin/gnosis_server.rs:192`/`:207`) has **no verb restriction** and **no body cap** (a sweep of the bin finds **no** `DefaultBodyLimit`; the transport gap is `docs/defects.md` **P-1**, OPEN), so a GET that carries a well-formed envelope is decoded and served normally. The response-side posture is unchanged by this appendix: **`GET /engine/status` takes no body at all** (always 200, `:284-289`) and **`GET /rag/stream` is param-driven** — it reads exactly `query`/`topK`/`mode` from the query string and constructs an **empty** envelope itself (`:246-264`), so it has no GET-with-body posture to state.

**What a decode error returns (the half that is ALREADY SATISFIED).** One shared renderer serves every route that can fail to decode a request — `decode_error_response`, `src/bin/gnosis_server.rs:62-75`, pinned in **§7.1** — emitting the structured body `{"code","message"}` with `Content-Type: application/json` **exactly** (no `charset`) and the NEW-2 statuses **400/422, never 502 and never a §11 row** (`request_decode_status`, `src/server.rs:51-62`; `request_decode_code`, `src/wire/query.rs:339-350`). **The engine's side of `GRQ-9`'s ask is therefore already implemented** — this landed in **U2** (`docs/defects.md`'s U2 record; §7.1's *"code LANDED in U2 — 2026-09-17"* marker) — and this appendix only **documents** the per-route consequence. The ask's own words are discharged: a decode error is a structured JSON body, **not** `text/plain`, and the consumer's client no longer has to mask a deterministic 400 as a JSON-parse failure.

**The residual divergence: a HOST-RUNTIME property, not a wire defect — CONSUMER-side adjudication, owner = the SHELL.** The consumer's engine client is an 11-method CRUD proxy whose read methods issue a GET-with-a-body that **Node/undici rejects while Electron/Chromium permits** — that is a property of the **consumer's HTTP client runtime**, **unverifiable from this role** (cross-repo), and it is recorded consumer-side as a host confirm-in-app item, **not** a handoff. **Consequence, stated plainly: there is nothing for the engine to fix and no enforcement change is made.** The engine cannot adjudicate it — it never sees a client that refused to send — and the divergence cannot be removed by re-classifying a route: `GET /documents/:id`, `GET /documents` and `GET /wikis/:id` carry **required** body args and `GET /wikis` carries a required envelope, so the posture is the path's, not an accident. **The engine's residual obligation is bounded to three things and nothing more:** (1) keep decode errors structured and transport-level (§7.1, landed); (2) keep this posture table accurate if a route ever changes; and (3) change the wire — a **method/route amendment unit** per `p2` §5.2's amendment rule (`route_bijection()` + the live `router()` + the `P-IM-3` conformance/property assertions **and** the 5 count assertions, `p2` §5.2:654-661) + a **P1a wire-version decision** (`p1a` §7) — **only if** the consumer asks for read routes that take no body (e.g. param- or path-carried read args) and the user authorizes it. **Until then the only consumer-side routes are: send the body from a runtime that permits it (Electron/Chromium), or restate the read methods against the existing posture.** Cross-references: `p1a` §4.2/§7/§8/§10 (the frozen wire + paths + the read/mutating split); §7.1 (the decode-error body this table's last column reuses); `docs/specs/p2-gnosis-server.md` §5.2/§5.5 (the routing amendment rule + the decode-error shape); `docs/defects.md` **P-1** (the absent body cap, OPEN); record §7(v)/§15.2 step 6.

---

## 15. What the TestWriter derives (red-set readiness)

- `wire_conformance.rs`: §5 table exhaustiveness + uniqueness; §6 round-trips
  across all `RagChunk`/`StoreError`/`RagResult` variants; §7 decode-then-validate
  incl. all `DecodeError` routes; §8 SSE round-trip + all `DecodeError`/frame
  fail-states; §9 health determinism; §12 golden vectors byte-for-byte.
- **Byte-for-byte golden assertions.** V-1..V-8 pin the **exact string** each codec
  must emit — the raw `serde_json::to_string`/`encode_event` output on the given
  input value (no re-formatting). They cover the two wire-defined camelCase surfaces
  (`Envelope`, `HealthReport`) and the serde-frozen body (`RagResult`/`RagTrace`/
  `RagResultItem`/`Source`/`QueryMode` — snake_case keys, PascalCase enums,
  externally-tagged trace, id newtypes as strings, `null` for absent `parent`/`stale`/
  `blocked_by`). The red tests assert equality **against these exact bytes**.
- `props_wire.rs`: the 8 invariant rows of the PBT register (FILE 2 asserts
  decoded-value `eq` against the input, **not** byte identity — it is independent of
  the literal JSON in FILE 1 and needs no change alongside the V-5/V-8 rewrites).
- The red set is derived **from this contract alone**; the implementer lands the
  least `src/wire/` code to green.

**U1 additions (per-unit red-set readiness; U1 itself writes no test and no implementation).**

- **U2's red set (from §4.5 + §7.1 + the mode rule):** **(LANDED 2026-09-17 — this red set was realized by
  `tests/props_gnosis_server.rs`'s seven U2 rows (`U2PIM4`…`U2PTP4`, layer 400 cases HELD),
  `tests/gnosis_server_e2e.rs`, `tests/wire_conformance.rs`'s `v15_request_decode_error_body_exact` and the
  blind set `tests/blind_u2_query_post_greens.rs` 24/24; the suite is 582/0. The item list below stands as the
  red-set record.)** the envelope-strict query decode (incl. the
  `schemaVersion`-before-`idFormat` precedence); the payload→`RagQueryOptions` mapping for every
  documented camelCase key; **the `filters` canonical-token mapping incl. its two negative forms**
  (a canonically-shaped `filters` — camelCase keys `nodeKind`/`edgeType`/`target`/`state` with the
  **lowercase/uppercase §4.5.2 tokens** — is never decoded into an all-`None` `QueryAuditFilters`; a
  **store-cased** `filters` (`"nodeKind":"Content"`, `target:["d1","n1"]`) is not the request shape, so
  it is either mapped per the canonical tokens or rejected — F1); the token resolver for all three
  enum-valued fields, located at the wire layer
  and not in `validate_rag_options` (F2); the **wrongly-typed ⇒ absent** corpus (F5); unknown-key
  tolerance (`args`/`method`/`requester`); **absent `mode` ⇒ `Flat` read at `p2` §5.3's two layers** —
  the **decoded** `RagQueryOptions.mode == None` (`Some(Flat)` is not a decoder output) and `Flat` is the
  **engine's query-time** reading (`mode.unwrap_or(QueryMode::Flat)`), with `Some(token)` produced only for
  a well-formed vocabulary string;
  the case-insensitive four-token match; every unrecognized-token case → `ValidationError`;
  POST/SSE parity **including the 400 status** for every token input (F4); the transport decode-code
  table + its disjointness from §11;
  the V-15/V-18 bytes **and V-15.1's binding note — its `message` is the implemented renderer's own
  output (the golden asserts code + status + header + keyset, plus the derived string), not the suggested
  literal**; the CRUD path's new JSON decode-error body (same unit).
- **U4's red set (from §4.6 + §4.7):** the cursor's type/step-1/monotonicity/opacity discipline; the
  cursor-first frame; one ordered `change` frame per committed entry with the pinned field set;
  `kind` ∈ the closed `op` vocabulary **and the unknown-`kind` tolerance** (F11, plus the conformance
  assertion over the `op` literals); the `decode_event` `UnknownType` cross-family rejection; the
  **resume-token** behavior for a not-comparable cursor (F9); the page-cap bounds
  (`docs/specs/p2-gnosis-server.md` §5.7).
- **U3's red set (from §9.1 + §12's V-8.1/V-8.2):** the amended literals land **with** the flag change,
  in the same unit — **REALIZED and GREEN (2026-09-17):** the flag change is the read-time derivation in
  `get_engine_status` (`src/store/mod.rs:4193-4232`) and the literal edit is
  `tests/wire_conformance.rs:1061-1112` + `boot_wiring_couples_to_the_derived_read` (`:1129`), with the
  property layer `tests/props_gnosis_server.rs`'s five `U3*` rows (127 executed ≤ 400) and the blind set
  `tests/blind_u3_status_honesty_greens.rs` **13/13**. The suite is **604 passed / 0 failed** (serial).
- **Not a U1 test target:** the FS-9 encoder path (§4.5's reachability note) and the `error` frame on
  the change feed — neither is reachable today.

---

## 16. Retrieval-semantics ruling — FS-3 vs FS-13/14/15 (U1; documentation reconcile)

**Authority.** gate-1 **ruling 5** (+ Appendix B), accepted by the go-ahead. This closes the long-open
`docs/HANDOFF.md` FS-13/14/15 row **engine-side**, and the `docs/HANDOFF.md` reconcile ask (extended by
the remand) requests that the canonical contract state the same for **FS-13/14/15, §4.5.1,
**§4.6.1's `ragQuery` throw column including its `hyde` clause**, and the canonical §4.5.2 `filters`
shape**. **No code change is required by this section** — the engine already behaves this way; the
code-bearing consequence is only U2's mode decode (§4.5). The server-side rendering table is
`docs/specs/p2-gnosis-server.md` §5.9.

**The rule: explicit-leg errors, fusion degrades.**

1. A request that **names a leg explicitly** (`mode=vector`) whose leg is **unavailable** (no vector
   index built) ⇒ **`StoreError::VectorIndexUnavailable`** (FS-14) ⇒ **503** `vector_index_unavailable`
   — honest and actionable ***(REMAND-2 SHOULD-FIX 3 / REMAND-3 SHOULD-FIX 3, 2026-09-22 — the qualifier
   `p2` §5.9's **`Precedence (pinned)` bullet** carries and this rule previously omitted: **FS-14 is
   READY-store-only.** A **non-READY** store short-circuits to **FS-8 `EngineUnavailable`** ⇒ **503
   `engine_unavailable`** (`p2` §5.9's `Precedence (pinned)` bullet,
   `docs/specs/p2-gnosis-server.md:1585-1592` — re-read this pass; the `:1582-1589` spelling this rule
   carried is **superseded** (REMAND-4 MUST-FIX 1, 2026-09-22), and the previously cited numbers are kept
   here as the record;
   rule 6 below). The unqualified reading above is superseded in place here and kept as its record; the
   FS-14 instance is a **READY** store with `vectors: None`***). **With an index built but no embedding
   provider wired, the same `mode=vector` request ⇒ `EmbeddingUnavailable` (FS-13) ⇒ 503
   `embedding_unavailable`; the **index check precedes the provider check**
   (`src/store/mod.rs:4448-4454`; the pre-U3 citation was `:4411-4417`).
2. A **fusion** request (`mode=hybrid`, and the lexical `mode=flat`) with a failing or absent leg ⇒ that
   leg **degrades to empty** and the query **succeeds** — never a hard failure
   (`src/store/mod.rs:4526-4556`, `:4493-4560`; the pre-U3 citations were `:4449-4454`, `:4489-4513`; §4.5.1's graceful degradation). The trace names the
   fused legs (`HybridTrace.legs = ["graph","vector","lexical"]`, `:4546-4556`); the frozen
   `HybridTrace` has **no** per-leg availability field, so the legs list does not record which leg was
   empty (a per-leg marker would be a change to a frozen §4.5 type — out of scope here).
   **`hyde: true` does not change this (F3 — a leg-naming condition, not a mode).** FS-13's
   "`(or with hyde: true)`" clause (`docs/specs/gnosis.md:1037`) is **not** a mode condition: `hyde`
   names **what to embed** (a generated hypothetical document), not a leg to fail on. Pinned: with
   `mode=hybrid` + `hyde: true` + `READY` + **no/unreachable provider**, the vector leg
   **degrades to empty and the query returns 200** — the provider error is absorbed by the fusion path
   (`src/store/mod.rs:4526-4556`; the pre-U3 citation was `:4503-4513`), exactly as in the unqualified hybrid case. `EmbeddingUnavailable`
   stays reachable **only** via `mode=vector`, and only when the index exists (`:4448-4454`; the pre-U3 citation was `:4411-4417`); a
   `mode=vector` + `hyde: true` request with a built index and no provider is still
   `EmbeddingUnavailable` ⇒ **503** (the explicit leg fails first). With `mode=flat`/`graph` the flag is
   **inert** — those paths never call the embedding provider (`flat_query` = the lexical leg only,
   `:4393-4419`; the pre-U3 citation was `:4370-4382`), so no `EmbeddingUnavailable` can arise. `HyDEGenerationFailed` (FS-18) is
   **untouched** and stays unreachable — the hypothetical-generation step is total
   (`:5136`; the pre-U3 citation was `:4429-4434`), so a provider failure while embedding the hypothetical surfaces as the provider
   error, which the fusion path absorbs.
3. **FS-13/14/15 therefore describe the explicit-leg case only — and the narrowing is wider than FS-13/14
   (F3).** FS-14's `mode: 'vector'`/`'hybrid'` wording (`docs/specs/gnosis.md:1038`) is read engine-side
   as explicit-leg-only; hybrid never fails for an absent/failing vector leg. **Two canonical sentences
   are narrowed by the same ruling and are named here so the reconcile is complete:**
   1. **§4.6.1's `ragQuery` throw column** (`docs/specs/gnosis.md:872`) — "`EmbeddingUnavailable` if the
      embedding provider is unreachable and a vector/hybrid/**hyde** leg requires it;
      `VectorIndexUnavailable` if the vector index is not built" — carries **no mode qualifier**, so its
      `hybrid`/`hyde` clauses and its unconditional `VectorIndexUnavailable` clause are all read
      **`mode=vector`-only** — **and that clause is additionally READY-store-only** (**REMAND-2
      SHOULD-FIX 3 / REMAND-3 SHOULD-FIX 3, 2026-09-22: a non-READY store yields FS-8
      `EngineUnavailable` ⇒ 503 `engine_unavailable` *before* any leg check, per rule 1's qualifier and
      `p2` §5.9's `Precedence (pinned)` bullet (`docs/specs/p2-gnosis-server.md:1585-1592`; re-read this
       pass — the `:1582-1589` spelling this rule carried is **superseded**, REMAND-4 MUST-FIX 1,
       2026-09-22, and is kept here as the record); the unqualified
      reading stands here as the record**). Under §6's **fail-state completeness rule**
      (`docs/specs/gnosis.md:1052-1055`) the §4.6.1 operation table **is** an authority table, so this
      sentence cannot be left standing unnamed.
   2. **FS-13's `hyde: true` clause** (`docs/specs/gnosis.md:1037`) — as pinned in rule 2 above.
4. **FS-15 is unreachable on the `ragQuery` surface.** The lexical "index" is a **live shard scan**, so
   a wiki with documents always has a lexical leg; `LexicalIndexUnavailable` surfaces **only** via the
   `bm25_search` API (reachable there when the queried wiki has no documents,
   `src/store/mod.rs:4270-4272`; the pre-U3 citation was `:4233-4234`).
5. **Invalid option values remain FS-3** — a **different condition** from an unbuilt index. An
   unrecognized `mode`/`expand`/`compression` token, or a **present `u64` numeric option outside its pinned
   range**, is
   `ValidationError` ⇒ **400 `validation_error`** (§4.5; for `expand`/`compression` that 400 is
   **POST-only** — the SSE surface reads no such param, N1), never a 503 and never a "leg unavailable"
   code. **Precision (ruling 1, 2026-09-16 — annotation only):** "out-of-range" here means a **present
   `u64`** outside its pinned range (`topK:0`/`51`, `maxHops:0`/`6`, `binaryCandidatePool:0` with
   `binaryFirstPass:true`, `multiQuery:{enabled:true,n:0}`); a present numeric value that is **not a
   `u64`** (`topK:-1`, `topK:10.5`) is **not** in this fail-state class at all — it decodes as absent ⇒
   that option's documented default, **never** a 400 (`p2` §5.3's type-vs-range bullet).
6. **Readiness precedes leg availability.** For every mode except `graph`, a not-`READY` engine returns
   `EngineUnavailable` (FS-8) ⇒ 503 **before** any leg check (`src/store/mod.rs:4078-4080`; the pre-U3 citation was `:4066-4068`); so the
   explicit-leg `VectorIndexUnavailable` outcome is only observable on a **READY** engine. `mode=graph`
   is not READY-gated (a deterministic local walk on the core store).

**Observable consequences (U2/U5 TestWriter targets, not new register rows here):** an explicit-leg
request for an unbuilt leg errors with the pinned code; a fusion request never fails because one leg is
absent; adding a leg never shrinks the result set; `mode=flat` never fails for want of an index.
**Interlock with U5:** once the boot index build lands, the explicit-leg error becomes
reachable-but-rare — the honest end state.
