# Inbound Astrographer feature-request set (GR-1..GR-9) — PROPOSAL-REVIEW record

- **Unit:** the inbound Astrographer feature-request set **GR-1..GR-9** (the
  consolidated engine feature requests filed by the Astrographer shell).
- **Gate:** proposal-review gate — **validity ∥ critique → architecture review →
  change-analysis** (four read-only passes, each `file:line`-grounded against the
  working tree).
- **Date:** 2026-09-16.
- **Verdict:** **PROCEED-WITH-AMENDMENTS on a re-shaped subset — review PASSED.**
  The re-shaped subset stands as reviewed (U0–U3 as amended, U4/U5 gated, GR-6/GR-7/GR-8
  parked behind named triggers, GR-9 **refused as an engine deliverable** — already
  answered by the C5/SHELL ruling). **U0 LANDED + U1 LANDED (2026-09-16)**; the U1
  spec-gate reviewer loop returned **EMPTY**, so **the spec gate is OPEN**;
  **U2–U5 remain HELD** pending the user's next go-ahead — **U2** (query POST contract) and
  **U3** (status honesty) are the recommended next scope, **U4** (change cursor + paged reads
  + `GET /changes`) additionally needs **SHELL-2** on the consumer side, and **U5** (boot
  vector-index build) follows **U3**. See §Go-ahead record.
  **POST-RECORD UPDATE (2026-09-16): the user then authorized U2 and U3** — their typed §5.x registers are
  **authored** in `docs/specs/p2-gnosis-server.md` §9.5.1 (U2 = 7 rows) / §9.5.2 (U3 = 5 rows), so for U2/U3
  only the **code** is owed; **U4/U5 remain NOT authorized**. The review record itself is unchanged — every
  "U2–U5 HELD / not yet authorized" clause in this file is the **2026-09-16 gate-time** state, superseded only
  as to U2/U3's authorization by that later user decision. **POST-RECORD UPDATE 2 (2026-09-17, proofread pass):
  U2's code has LANDED REALIZED-GREEN** (`cargo test` 582 / 0; its property layer 400 cases HELD; the blind set
  `tests/blind_u2_query_post_greens.rs` 24/24; the live battery's `R-L1` PASSED live — the record is
  `docs/specs/p2-gnosis-server.md` §U1's U2-landed bullet), so "the code is owed" now applies to **U3 only**;
  **U4/U5 remain NOT authorized**. **POST-RECORD UPDATE 3 (2026-09-17, the U3 documentation pass): U3's code has
  LANDED-GREEN** (`cargo test` 604 / 0 serial; its five §9.5.2 rows HELD with 127 executed ≤ 400; the blind set
  `tests/blind_u3_status_honesty_greens.rs` 13/13; the live battery 9/9 rows PASS live incl. `R-L2`), so **no
  U2/U3 code is owed any longer** and **U4/U5 are the only units still NOT authorized**.
- **Status:** the rulings below are **adopted at the gate**, and the two gate decisions
  are **ACTIVE** in `docs/decisions.md` — **`GNOSIS-CHANGE-CURSOR`** and
  **`SUBSYSTEM-FLAG-CAPABILITY-SEMANTICS`** (landed on the user's 2026-09-16 instruction
  "clean the JS files and then proceed", which accepted the gate's recommendations by
  proceeding — the user did not select the alternatives option-by-option). **U1 (the
  P2/F2 contract amendment) is now LANDED** (2026-09-16, docs-only): its contract pins live in
  `docs/specs/p2-gnosis-server.md` (§U1 + §5.3–§5.9) and `docs/specs/engine-wire-contract.md`
  (§4.5–§4.7, §9.1, §16), and the upstream reconcile ask is the `docs/HANDOFF.md` addendum.
  **The spec gate is therefore OPEN** — but **U2/U3/U4/U5 remain HELD** (the approved scope was
  **U0+U1, re-gated at U1's spec review**), so no code unit may start until the user gives the
  next go-ahead. See §Go-ahead record.
  **SUPERSEDED (2026-09-16 → 2026-09-17, post-greens doc-review): the user then AUTHORIZED U2 and U3 (2026-09-16;
  their typed registers are authored in `docs/specs/p2-gnosis-server.md` §9.5.1/§9.5.2), and U2's code has since
  LANDED REALIZED-GREEN (2026-09-17: `cargo test` 582/0, all 7 §9.5.1 rows HELD with the layer's 400 executed
  cases, the blind set 24/24, the live row `R-L1` PASSED live — `docs/next-steps.md`'s U2 DONE row) — so only
  U3's code was owed. U4/U5 remain NOT authorized.** **EXTENDED (2026-09-17, the U3 documentation pass): U3's
  code has LANDED-GREEN** (the read-time flag derivation in `get_engine_status`, the lib seam `boot_wiring`, the
  bin boot that writes no mask, and the amended V-8.1/V-8.2 literals; `cargo test` 604/0, five rows HELD with 127
  executed, blind set 13/13, live battery 9/9 incl. `R-L2`) — **so no U2/U3 code is owed any longer**. The clause above stands as the 2026-09-16 gate-time record;
  the same reading is carried by the **POST-RECORD UPDATE** notes at the head of this file (`:18-26`) and by the
  §Go-ahead record's POST-RECORD UPDATE 3 bullet below.

## The inbound artifact

- **Consumer doc:** `<Astrographer repo>/docs/feature-requests/gnosis-engine-feature-requests.md`
  — the consolidated set **GR-1..GR-9**, each with live repro evidence, a proposed
  wire shape and testable acceptance criteria. **Read-only for this repo.**
- **Engine-side record:** the **INBOUND from the Astrographer shell (2026-09-16)**
  note at the top of `docs/HANDOFF.md` (already present; this review is its gate-1
  continuation, cross-referenced from that note).
- **What the set asks, in aggregate:** make the shell's query path work
  end-to-end against the engine's own contract (GR-1 envelope, GR-2 options
  surface, GR-3 vector index), then let the **engine own the store** instead of the
  consumer reaching into the store interface (GR-4 bulk projection read, GR-5
  store-change notification), followed by three parked destination items (GR-6 bulk
  markdown ingestion, GR-7 server-side persistence, GR-8 enrichment/traversal routes
  + a text-generation seam) and one authorization request (GR-9).
- **Boundary rule honored throughout:** **this repo never patches the consumer's
  source.** Every consumer-side obligation is recorded as a *shell* unit
  (SHELL-1/SHELL-2) or a HANDOFF/spec-reconcile request — never as an edit to the
  Astrographer tree.

## Baseline (re-verified this session against the actual tree)

| Check | Result |
| --- | --- |
| `cargo test` | **538 passed / 0 failed** |
| `cargo fmt --check` | **exit 0** |
| `cargo clippy --all-targets` | **0 warnings** |
| `cargo build` | **clean** |

- **Tree state:** the working tree is **not committed** — it holds (a) an
  uncommitted **fmt/clippy hygiene fix** and (b) the **HANDOFF inbound note**. Both
  are tree state only; neither is a code-behavior change. The baseline above is a
  **re-run this session**, not a carried-forward claim (per the ruling pinned in
  `docs/decisions.md` row `P1A-P2-HYGIENE-TRIO-FIX`: a trio claim must be
  re-verified against the actual tree, never carried forward).
- All four gate-1 passes were **read-only** and `file:line`-grounded; no pass wrote
  to `src/`, `tests/`, or any frozen spec.

## Per-GR verdict table

| GR | request (short) | verdict | reason | owning unit / precondition |
| --- | --- | --- | --- | --- |
| **GR-1** | `POST /rag/query` must accept the F2 envelope | PROCEED-WITH-AMENDMENTS (engine half only) | The envelope requirement is **intentional and final** (the route already parses through `Envelope::from_json`, `src/bin/gnosis_server.rs:157-161`), so the **shell's bare POST body is the consumer's defect**; the engine's own residual obligation is only that a decode failure returns a **structured JSON** body instead of today's literal `"request decode failed"` plain text (`src/bin/gnosis_server.rs:57-66`). | **U2** (engine half); **SHELL-1** (consumer client + its own spec drift at `<Astrographer repo>/src/specs/unit-gn-engine-integration.md:452-458`) must land first for any user-visible value; **U1** defines the decode-body shape. Bare-body tolerance is **REFUSED**. |
| **GR-2** | POST query must honour `mode`/`topK` | PROCEED-WITH-AMENDMENTS | **CONFIRMED real defect:** `rag_query_handler` reads only `payload.get("query")` and calls `store.rag_query(&query, &RagQueryOptions::default())` (`src/bin/gnosis_server.rs:162-168`), so the caller's whole options surface is silently inert while the SSE route reads both (`:193-203`) — and the SSE route matches mode **case-sensitively** and silently coerces an unknown mode to Flat, which FS-3 forbids. | **U2**; **U1** pins the request/response bodies + the single shared mode rule. **Zero consumer work.** |
| **GR-3** | build the vector index; make `mode=vector` usable | PROCEED-WITH-AMENDMENTS (split) | **CONFIRMED and broader than reported:** `swap_snapshot(DerivedIndexes::default())` leaves `vectors: None` (`src/bin/gnosis_server.rs:329`), and **all six** `EngineSubsystems` flags are hard-coded `true` at construction (`src/store/mod.rs:1714-1725`; the gate-time citation was `:1710-1717`) — including `reranker: true` although no reranker exists anywhere in `src/`. So `/engine/status` makes **false claims** today. *(**Post-record note, 2026-09-17:** the status-honesty half is **FIXED (U3)** — the flags are now derived at read time in `get_engine_status` (`src/store/mod.rs:4193-4232`), the two citations above are the gate-time ones, and the boot's snapshot swap is `:390-404`; the **boot index build** half remains **U5**'s, so `vector:false` and the `mode=vector` 503 are still the honest landed state.)* | Status-honesty half = **U3**; boot index build = **U5**; "maintain on store changes" = **PARKED** (needs the deliberately held-off `WRITER-ACTOR-JOURNAL` writer). |
| **GR-4** | a bulk projection-snapshot read route | PARK-WITH-TRIGGER | The *bulk read* is legitimate engine work (no `/nodes`/`/edges`/`/adjacency`/`/snapshot` route exists; no `list_nodes`/`list_edges` on the trait; `Store.shards` is private, `src/store/mod.rs:1552`), but the requested **revisioned projection** (`/snapshot?revision=`, `stale_revision`) is **REFUSED**: the engine owns **no consumer-visible store-wide revision** (only a per-process `epoch` that bumps on propagation/state-annotation mutations too, `src/store/mod.rs:2284,2356,3348-3357`), a consumer-held projection keyed on it is a **second source of truth** (§4.2.7.2 + `GRAPH-OWNS-RELATION-AND-MERGE`), and `SHARDED-RWLOCK-STORE` means **no point-in-time read across shards**. | Re-shaped as **U4** (paginated, wiki-scoped, cursor-tagged reads); trigger = **U1's cursor contract + `SHELL-2`'s bounded consumer cache**. |
| **GR-5** | a store-change notification route | PROCEED-WITH-AMENDMENTS (re-shaped) | **CONFIRMED absent**, and the correct primitive — but only as an **opaque change cursor**, **never as a revision**: `JournalEntry` is `{seq, op, base_revision, timestamp}` (`src/store/mod.rs:943-953`) with **no ids/kind**, and the "two writes in one commit batch" acceptance presumes a **batching primitive that does not exist**. | Folded into **U4** (cursor accessor + journal kind/ids + `GET /changes` SSE + route-table amendment). **No `stale_revision` wire code.** |
| **GR-6** | bulk markdown ingestion (atomicity/cap/progress/cancel + document path) | PARK-WITH-TRIGGER; the `{files:[paths]}` variant is **REJECTED** | **CONFIRMED absent** (no parser, no route, no batch primitive, `Document` has no path field), but the requested contract is **self-contradictory** ("the whole corpus commits as one journal entry" vs "chunked commits or an SSE progress stream"), and `{files:[paths]}` would grant the engine a **filesystem-read capability** on a loopback-only server whose accepted residual risk is "any process that can reach the loopback port may call the mutating methods". The document `path`/segment half collides with the **FROZEN** P1a `Document` body (`docs/specs/p1a-document-crud-wire.md:253-256`, `P-IM-2`). | Trigger: an accepted spec resolving **one-commit-vs-chunked** + a **cap** + a §11 fail-state allocation, plus a **P1a wire-version decision** for `path`, plus an **explicit user decision** on the filesystem capability. |
| **GR-7** | server-side persistence for the engine store | PARK-WITH-TRIGGER | **CONFIRMED in-memory** (`let store = Arc::new(Store::new())`, only `--port`), and persistence+revisioning **ARE** engine-owned work (`docs/specs/gnosis.md:82,190`; boundary A1) — but the request **raises the decisive questions it does not answer** ("what a revision means across a restore", crash-mid-commit), would re-open `SHARDED-RWLOCK-STORE`/`IMMUTABLE-DERIVED-SNAPSHOT`, and collides with the consumer's own parked `SINGLE-WRITER-STORE` (O-8) amendment. The request's "the store trait's persistence seam is a trait with no disk implementation" is a **misstatement**: there is **no persistence abstraction in the crate at all**. | Trigger: the consumer's **O-8 amendment recorded** + a **durability design** (format, atomic commit, recovery, store-scope revision semantics) + a scheduled consumer unit. |
| **GR-8** | enrichment/traversal routes + a text-generation seam | PARK-WITH-TRIGGER | **CONFIRMED absent** (the four trait methods `declare_community`/`update_community_summary`/`resolve_entities`/`merge_facts` are implemented but **unrouted**; no text-gen seam exists), but the text-generation seam is **already ruled OUT** (`docs/specs/f4-llm-enrichment-integration.md:139-149`; `docs/pending.md` F4-LLM = SPECULATIVE), and under §4.2.8 / `RESOLVE-ENTITIES-AUTHORITATIVE-OVERWRITE` an LLM host as caller would be **authoritative** and could silently supersede a human declaration while `AUTHORSHIP-SOURCE-PROPERTY` (the only mechanism recording who won) is **unlanded**. | Trigger: `AUTHORSHIP-SOURCE-PROPERTY` landed **code-bearing** + an accepted text-gen-seam design + a consumer freeze (F4-LLM un-parked). |
| **GR-9** | mutating-surface authorization for a machine caller | **REJECT** as an engine deliverable (already answered) | **REFUTED as an engine gap.** The observed `caller has no edit authority` is **shell-side fail-closed** from an empty authority mapping, not an engine refusal; the engine has **no** denial code, discards `caller` after the decode layer (`src/bin/gnosis_server.rs:73-123`), and its `RagStore` mutating methods take **no `caller` param**. C5 + decision `GNOSIS-RBAC-EDIT-ENFORCEMENT` place the authorization gate in the **SHELL**. | The engine owes only the **existing HANDOFF reconcile row** naming the SHELL as authority owner (already present; cross-referenced). **Do NOT author an engine authority contract.** |

## Validity findings

**CONFIRMED (the request is right, with evidence).**

1. **GR-2 is a real, silent-defect surface.** The POST handler's options surface is
   inert — `payload.get("query")` → `RagQueryOptions::default()`
   (`src/bin/gnosis_server.rs:162-168`) — while the SSE route reads `topK` and
   `mode` (`:193-203`). The asymmetry is the defect, and the SSE route's
   **case-sensitive** match with a silent fall-through to `None` (`_ => None`)
   silently degrades an unknown mode instead of raising FS-3.
2. **GR-3's status-honesty half is real and broader than filed.** `vectors: None`
   at boot (`src/bin/gnosis_server.rs:329`) **plus** six hard-coded `true` flags
   (`src/store/mod.rs:1714-1725`; the gate-time citation was `:1710-1717`), one of which (`reranker`) names a subsystem with
   no implementation anywhere in `src/`. `/engine/status` therefore asserts
   capabilities the engine does not have. *(**Post-record note, 2026-09-17:** this finding's status half is
   **FIXED (U3)** — the flags are derived at read time (`src/store/mod.rs:4193-4232`) and the legacy literal is
   dead value (it now sits at **`:1718-1725`**, under its U3 comment block `:1714-1717`; the
   `:1714-1725`/`:1710-1717` spellings above are the post-U3/gate-time ones, §5.8 of `p2`); the boot's
   snapshot swap is now `:390-404`. The finding text stands as the gate-time record.)*
3. **GR-4's bulk-read half is real.** The server exposes **no**
   `/nodes`/`/edges`/`/adjacency`/`/snapshot` route; the `RagStore` trait has no
   `list_nodes`/`list_edges`; and the shard map is private (`Store.shards`,
   `src/store/mod.rs:1552`) — so a consumer genuinely cannot bulk-read a projection
   today.
4. **GR-5's primitive is real and correctly motivated.** No store-change route
   exists; the journal is the right substrate (`src/store/mod.rs:943-953`) but is
   **ids-blind and kind-blind**, so it cannot drive a consumer cache as filed.
5. **GR-6/GR-7/GR-8's absences are real** (no ingest parser/route/batch primitive;
   `Store::new()` only, in-memory; the four enrichment trait methods
   implemented-but-unrouted and no text-generation seam).
6. **GR-1's engine-side residual is real but tiny**: the decode failure path is the
   literal plain-text body `"request decode failed"`
   (`src/bin/gnosis_server.rs:57-66`), which the masking `decodeEnvelope` consumer
   client cannot surface usefully.

**REFUTED / not-confirmed as stated.**

7. **GR-9 is not an engine gap.** The engine has **no** denial code at all, discards
   `caller` after the request-decode layer (`src/bin/gnosis_server.rs:73-123`), and
   its mutating `RagStore` methods take no `caller` param. The observed shell error
   is **consumer-side fail-closed** from an empty authority mapping. The engine-side
   answer already exists as a HANDOFF reconcile row (SHELL owns the authorization
   gate, per C5 + `GNOSIS-RBAC-EDIT-ENFORCEMENT`).
8. **GR-1's framing is inverted.** The requirement it asks to relax is the engine's
   **own landed contract** (`Envelope::from_json` at `src/bin/gnosis_server.rs:157-161`,
   the F2 envelope, the P1a CRUD precedent) — so the bare POST body is the
   **consumer's** defect, and the engine's obligation is limited to a structured
   decode-error body. **Bare-body tolerance is REFUSED** (it would make the query
   surface envelope-optional while CRUD stays envelope-strict).
9. **GR-4's revisioned projection is refused, not merely parked.** The engine owns no
   consumer-visible store-wide revision; the only counter is a per-process `epoch`
   that also bumps on propagation and state-annotation mutations
   (`src/store/mod.rs:2284,2356,3348-3357`). A consumer-held projection keyed on it
   would be a second source of truth (§4.2.7.2, `GRAPH-OWNS-RELATION-AND-MERGE`) and
   `SHARDED-RWLOCK-STORE` forecloses a point-in-time read across shards.
10. **GR-5's "two writes in one commit batch" acceptance presumes a primitive the
    engine does not have** — there is no commit-batching primitive; each mutation is
    its own journal append inside its own critical section.
11. **GR-7's premise is a misstatement.** There is **no persistence abstraction** in
    the crate — no trait with "no disk implementation", no seam to fill in. The
    request would be authoring the abstraction, not implementing one.
12. **GR-6's `{files:[paths]}` variant is rejected on capability grounds**, and its
    contract is self-contradictory (one-journal-entry atomicity vs chunked
    commits/progress SSE cannot both hold).

**Corrected request-document claims (validity pass).**

- **Route enumeration is verb-insensitive and incomplete.** The request's route list
  counts paths, not `(verb, path)` pairs. The server actually exposes **14
  method-aware routes** (`src/bin/gnosis_server.rs:234-248` + `src/server.rs:68-87`):
  three paths carry two verbs each (`/documents`, `/wikis`, plus the shared
  `DELETE`/`GET` on `/documents/:id`), which the verb-insensitive enumeration
  undercounts.
- **GR-4's "the architectural ruling chose exactly this boundary" cites a
  *consumer-side* ruling, not an engine ruling.** No engine-side decision or spec row
  chooses a revisioned projection boundary; the citation does not transfer.
- **The cited byte-identical hashes/scores are not independently confirmable from
  this repo.** They are consistent with the code as read here, but no artifact in
  this tree pins them; the claims are **unverifiable here** (not disputed).
- **Stale vendored copies of the shell client sat in this Rust tree.** Verified as
  actually present at gate time: `engine-rag-store.ts` (repo root),
  `src/main/engine-rag-store.ts`, `engine-crud-rag-store.ts` (repo root),
  `src/main/engine-crud-rag-store.ts` — all four existed. They were vendored copies of
  the consumer's TypeScript client and drifted from the live consumer client. **The
  user answered open question 5 with DELETE, and the copies were removed by the
  supervisor on 2026-09-16** (`git rm`, plus the then-empty `src/main/` directory); the
  `docs/defects.md` row is **FIXED**, and it retains the record of *why* they were
  a defect (drift from the live consumer client — the class of risk behind GR-1's
  bare-body report). **This is now fully cleaned, in two passes:** the four client
  copies on 2026-09-16, then the fifth stray copy `retrieval.ts` (repo root) plus the two
  orphaned vendored TypeScript test files (`tests/unit-a1-crud-routing-proxy.test.ts`,
  `tests/props-a1-crud-routing-proxy.test.ts`) on the user's "clean the JS files"
  instruction in this pass — every stray TS/JS file was deleted and a follow-up sweep
  finds **none remaining** anywhere in the tree outside `target/`/`archive/`/`.cargo-home/`
  (`.cargo-home/` is a pre-existing, gitignored, intentional sandbox-local cargo home, not
  JS). All three `docs/defects.md` rows are **FIXED** and the Rust baseline was unaffected
  (`cargo test` 538 pass / 0 fail).

## Boundary table

| GR | ENGINE (this repo) | SHELL (consumer repo) | SEAM (shared contract) | PARKED |
| --- | --- | --- | --- | --- |
| **GR-1** | structured JSON decode-error body on the query surface (U2) | **SHELL-1** — the bare POST body fix + its own spec drift at `<Astrographer repo>/src/specs/unit-gn-engine-integration.md:452-458` | the envelope + the decode-body shape (U1) | — |
| **GR-2** | POST options decoder + shared mode resolver (U2) | none (zero consumer work) | request/response bodies + the single mode rule (U1) | — |
| **GR-3** | flag honesty (U3); boot index build (U5) | none | subsystem-flag semantics + V-8 golden (U1) | "maintain on store changes" (needs `WRITER-ACTOR-JOURNAL` writer) |
| **GR-4** | paginated, wiki-scoped, cursor-tagged reads (U4) | **SHELL-2** — the bounded local cache behind the six **synchronous** `RagStore` members + a cursor-resync consumer | the cursor contract + the page/cursor shapes (U1/U4) | revisioned projection (`/snapshot?revision=`, `stale_revision`) — **refused** |
| **GR-5** | cursor accessor + journal kind/ids + `GET /changes` SSE (U4) | **SHELL-2** — the resync consumer | route-table + P-IM-3 amendment in U4 | route growth beyond U4 |
| **GR-6** | — | — | — | both halves (ingest contract; document `path`/segments) |
| **GR-7** | — | consumer's own `SINGLE-WRITER-STORE` (O-8) amendment is a precondition | a future durability design | persistence (parked with trigger) |
| **GR-8** | — | the LLM host remains a caller (F4-LLM) | an accepted text-gen-seam design | enrichment routes + text-gen seam |
| **GR-9** | **no deliverable** (existing HANDOFF row only) | the authorization gate + the authority mapping | C5 + `GNOSIS-RBAC-EDIT-ENFORCEMENT` (recorded) | — |

## The four rulings

### 1. The revision question

The engine owns **no consumer-visible store-wide revision**. Instead it exposes
exactly one wire-visible **change cursor**: an opaque string equal to the committed
journal `seq`, advancing by **exactly 1 per committed journal entry**,
**process-lifetime monotonic** (may repeat across restarts until a durable store
exists), explicitly **not point-in-time** and **not valid for cache validation,
optimistic concurrency, or state reconstruction** — only for cache invalidation,
resync and de-dup. Per-document `revision` stays the concurrency token. The cursor
gets its **own accessor and documentation**, distinct from the derived-index `epoch`
(which legitimately bumps on propagation/state-annotation mutations). It **refines**
`IMMUTABLE-DERIVED-SNAPSHOT` + `WRITER-ACTOR-JOURNAL` and **supersedes nothing**.
`/snapshot?revision=` and a `stale_revision` error code are **refused**.

### 2. The route-surface question

The pinned 14-row `route_bijection`/`P-IM-3` becomes a **growth invariant, not a
fixed count**: pairwise-distinct `(verb, path)` rows → pairwise-distinct handlers,
the **11 P1a CRUD paths verbatim**, the **retrieval trio present**; the "exactly 14
rows" clause is **deleted**. New routes may be added **only** by an amendment unit
that extends `route_bijection()` **and** its conformance/property rows **in the same
unit**. The **§11 map stays 21 rows** — no new `StoreError` variant, no
`stale_revision`/`decode_failed` wire code; a decode failure stays the **NEW-2
transport** outcome (**400/422, never 502, no §11 row**).

### 3. The `/rag/query` contract

- **Envelope-strict exactly like CRUD** (shared decoder; a bare body stays rejected —
  **no back-compat tolerance**).
- Request `payload` = the camelCase `ragQuery` options object (`query`, `mode`,
  `topK`, `wikiId`, …).
- Response **keeps the bare `RagResult` body** (**NOT** the SSE
  `{"type":"result",…}` chunk); `encode_result` must never emit a body the result
  validator rejects.
- **One** mode rule shared by POST and SSE — **case-insensitive** over
  `flat|graph|vector|hybrid`, **absent → Flat**, **unknown → `ValidationError` →
  400 / `validation_error`** — **never silent degradation** and **never a decode-level
  422**.
- **FS-13/14/15 are reconciled as explicit-leg-only errors** (`mode=vector` with no
  index → `VectorIndexUnavailable` → **503**) while the **hybrid fusion contract
  keeps degrading a failing leg to empty** (this **closes the long-open HANDOFF
  FS-13/14/15 row**, `docs/HANDOFF.md:29`).

### 4. Status honesty

A subsystem flag means **"this subsystem's full query-time capability is wired and
functional for the current store"**. The six hard-coded `true` flags are therefore a
**defect** (`reranker` is a false claim; `vector` is false until the index is built;
`lexical: true` is honest under capability semantics). **No new status endpoint**;
the F2 §9 change is **additive** and requires the **F2 amendment + the V-8 golden
update in the same unit as the code**.

## Ordered workstream (U0–U5 + the two shell units)

Every unit owes the repo's full chain: **proposal-review record → spec with the typed
§5.x property register → red test set → green implementation → adversarial pass incl.
the read-only PBT audit → blind-greens → live battery → doc-review → trio.** Doc-only
units take the **justified zero-row exemption**.

| # | Unit | Scope | Blocking dep | Size |
| --- | --- | --- | --- | --- |
| **U0** | this landing: gate-1 review record + tracker truth (parks, defects, HANDOFF rows, CURRENT WORK) | — | gate-1 review | S |
| **U1** | the P2/F2 contract amendment: route bijection as growth invariant + P-IM-3 restated; query request/response bodies; the single mode rule; the structured NEW-2 decode body on the query surface; subsystem-flag semantics + V-8 golden; FS-13/14/15 explicit-leg reconcile — **LANDED (2026-09-16, docs-only); the spec-gate reviewer loop returned EMPTY ⇒ the spec gate is OPEN** | U0 + go-ahead (given; done) | M |
| **U2** | query POST contract (GR-1 engine half + GR-2): shared `payload`→`RagQueryOptions` decoder, envelope strictness, mode resolver + unknown-mode rejection on POST **and** SSE, structured JSON decode-error body — **LANDED (2026-09-17):** code in `src/wire/query.rs` + the bin wiring, typed register §9.5.1 (7 rows, tags `U2PIM4`…`U2PTP4`, layer 400/400 cases HELD), blind-greens **24/24**, live row **`R-L1` PASSED** — `docs/next-steps.md`'s U2 DONE row | U1 | M |
| **U3** | status honesty (GR-3 first half): derive the six subsystem flags from real state; `reranker` false; `vector` false until U5; V-8 amendment applied with the code — **LANDED-GREEN (2026-09-17):** the read-time derivation in `get_engine_status` (`src/store/mod.rs:4193-4232`), the lib seam `boot_wiring` (`src/lib.rs:69-110`), the bin boot that writes **no** flag mask (`src/bin/gnosis_server.rs:378-408`) and the amended V-8.1/V-8.2 golden literals (`tests/wire_conformance.rs:1061-1112`), with the typed register §9.5.2 (5 rows, tags `U3PIM7`…`U3PSM6`, layer 127/400 executed HELD), blind-greens **13/13** and the live battery **9/9** incl. **`R-L2`** — `docs/next-steps.md`'s U3 DONE row *(the cell read "AUTHORIZED (2026-09-16), register authored in §9.5.2 (5 rows), code owed; its live row is `R-L2`" — kept as the pre-landing record)* | U1 | S–M |
| **U4** | change cursor + paged reads + change feed (GR-4 + GR-5 re-shaped): cursor accessor, journal `kind` + affected ids, `GET /changes` SSE, paginated wiki-scoped node/edge reads, route-table + P-IM-3 amendment in the same unit | U1; and `SHELL-2` for real value | L |
| **U5** | boot vector-index build (GR-3 second half) + honest `vector` flag flip | U3 (+ U1) | M–L |
| **SHELL-1** | (consumer repo) the envelope client fix + its own spec reconcile — **the actual P0** for the user-visible query path | — | S |
| **SHELL-2** | (consumer repo) a bounded local cache behind the six **synchronous** `RagStore` members + a cursor-resync consumer | U4 | L |

**Effort calibration:** P1a = 11 wire shapes + 8 register rows + 35 conformance + 8
property (411→454); P2 = 14 endpoints + 7 register rows + 25 conformance + 7 property
+ 16 e2e + 15 blind-greens (454→523); **U0–U5 band ≈ 56–80 new tests (→ ≈594–618
total)**, **XL overall**, and per **RCA-5** each unit is **delegated separately —
never as one pass**.

## Blast radius + reconcile order

| Frozen artifact | What the amendment touches | What breaks if skipped |
| --- | --- | --- |
| `route_bijection()` / `P-IM-3` (14-row fixed count) | U1 restates P-IM-3/§5.2 as a growth invariant; U4 rewrites the assertions in its own unit | U4's new routes contradict the pinned "exactly 14 rows" and the property row fails |
| V-8 subsystem-flag goldens | F2 §9 text + `tests/wire_conformance.rs` literals/fixture helpers amended **in U1**, flags changed **in U3**, golden literal edit in the **same unit as the flag change** | a flag change lands against a golden that still pins `true` → red conformance with no contract authority for the new literal |
| §11's **21 rows** | **unchanged** — decode failure stays NEW-2 transport (400/422, never 502); no `stale_revision`/`decode_failed` code | adding a §11 row (or a 502 for a decode failure) breaks the frozen 21-row map + its conformance rows |
| the F2 response **result body** | U1 pins the bare `RagResult` body (not the SSE `{"type":"result"}` chunk) + the "never emit a validator-rejected body" rule | `encode_result` and the result validator drift → a 200 body the consumer cannot validate |
| CRUD **envelope strictness** | U1 states the shared decoder precedent (**unknown payload keys tolerated** — no `deny_unknown_fields` in `src/wire/`; **absent `mode` → Flat**) | envelope-strict `/rag/query` rejects existing e2e POSTs that carry an extra `args` key |
| P1a `Document` **freeze** (`docs/specs/p1a-document-crud-wire.md:253-256`, `P-IM-2`) | GR-6's `path`/segments half needs a **P1a wire-version decision** — not authorized | amending the frozen `Document` body silently changes the CRUD wire for A1/A2 |
| the **538** tests | see §Regression watch — four named reddening paths, each with a guard | a unit lands red against the verified baseline |

**Reconcile order (strict):** **U0 → U1 alone → U2/U3 → U4 after SHELL-2 → U5.**
U1 lands **alone** (contract-only) so U2/U3/U4 each have authority to code against;
U4 waits on SHELL-2's bounded consumer cache so the cursor has a real consumer.
**(Executed state, 2026-09-17: U2 LANDED and U3 LANDED-GREEN — the order's next step is U4, after SHELL-2 —
with U4/U5 still NOT authorized; U5 remains U3's follower and owns the boot index build /
`vector:true` flip.)** *(Pre-doc-review reading: "U2 LANDED, so the order's next step is U3 — whose code is
owed".)*

**Shell-first statement.** **GR-1's user-visible value is entirely consumer-side** —
the shell must stop posting a bare body; the engine must **not** be credited with the
shell's fix (the engine's half is only the structured decode body, which is what makes
the *consumer's* failure legible). Likewise **U4 delivers no value until SHELL-2
exists**, because six of the consumer's `RagStore` members are **synchronous** and
cannot await a paginated/cursor-tagged route on their current shape. **SHELL-1 is the
actual P0 for the user-visible query path**; U0–U5 are the engine half of a
two-repo fix.

## Residual risk register

| Risk | Disposition | Revisit trigger |
| --- | --- | --- |
| **Loopback-only + shell-owned RBAC preserved by parking GR-7/GR-9** | **ACCEPTED** — explicitly, by parking | any engine-owned durable store, non-loopback bind, or multi-user deployment |
| **GR-9 stays a documented shell gap** (the shell's empty authority mapping fails closed with an engine-shaped message) | **ACCEPTED**, consumer-owned | if the message still misattributes after the consumer lands its mapping |
| **`mode=vector` remains unusable until U5 lands** (honest-but-degraded: the flag goes `false` in U3 **before** the index exists in U5) | **ACCEPTED until U5** | reprioritize U5 above U4 if a consumer needs vector search sooner |
| **Cursor misinterpretation as a revision** (a shell using it for validation would believe in a coherence it does not have) | **ACCEPTED**, mitigated by distinct naming/accessor + documentation | any attempt to use it as a precondition/validation token |
| **`stale_revision` absent** | **MITIGATED by design** — per-document `revision` remains the concurrency token | a consumer demonstrates a stale-read need it cannot serve |
| **Contract drift across the shell/engine split** (client envelope, masking `decodeEnvelope`, the vendored stale copies in this tree) | **CLOSED 2026-09-16**: open question 5 was answered *delete*, and the cleanup landed in **two passes** — the four client copies, then `retrieval.ts` + the two orphaned vendored test files (`git rm`); a sweep finds **no stray TS/JS left** and all three `docs/defects.md` rows are **FIXED** | any further vendored copy or a shell-side fix credited to the engine |
| **Unbounded payloads on the new list routes** | **MUST be mitigated inside U4** (page-size cap + bounds, mirroring `DocumentList{page,page_size,total}`); shipping U4 without caps is a **defect, not a residual** | U4's spec review (cap + bounds pinned in the register) |
| **Pinning the wrong invariant** (P-IM-3's fixed count, V-8's golden literals) | **MITIGATED in U1** — but only if U1 lands **before** U4/U3 | any U3/U4 landing that precedes U1's restatement |
| **Second source of truth** if a consumer treats the cursor as authority | **ACCEPTED**, guarded by §4.2.7.2 + `IMMUTABLE-DERIVED-SNAPSHOT` + the cursor's documented scope | a consumer stores a projection keyed on the cursor and treats it as authoritative |

## Parked items with triggers

| Parked item | Why parked | Measured un-park trigger |
| --- | --- | --- |
| **GR-3 "maintain on change"** (index maintenance on store changes) | needs the **deliberately held-off** `WRITER-ACTOR-JOURNAL` writer (`docs/decisions.md` row: the tokio writer-actor is held off until §4.2–§4.4 need the async rebuild/audit feed and the write-path latency is justified) | the writer-actor un-parks (a measured rebuild-latency/contention signal) **and** U5's boot build is green |
| **GR-4/GR-5 route growth beyond U4** | P-IM-3 becomes a growth invariant, so growth needs its own amendment unit | a named consumer need for an additional route **plus** an amendment unit that extends `route_bijection()` + its conformance/property rows together |
| **GR-6 ingest half (atomicity/cap/progress/cancel)** | self-contradictory contract + no batch primitive + a NEW-2/§11 fail-state allocation owed | an accepted spec resolving **one-commit-vs-chunked** + a **cap** + a §11 fail-state allocation |
| **GR-6 document `path`/segments half** | collides with the **FROZEN** P1a `Document` body (`docs/specs/p1a-document-crud-wire.md:253-256`, `P-IM-2`) | a **P1a wire-version decision** for `path` (plus the `{files:[paths]}` filesystem-capability decision, currently **REJECTED**) |
| **GR-7 server-side persistence** | raises the decisive questions it does not answer; re-opens `SHARDED-RWLOCK-STORE`/`IMMUTABLE-DERIVED-SNAPSHOT`; collides with the consumer's parked `SINGLE-WRITER-STORE` (O-8) | the consumer's **O-8 amendment recorded** + a **durability design** (format, atomic commit, recovery, store-scope revision semantics) + a scheduled consumer unit |
| **GR-8 enrichment/traversal routes + text-gen seam** | the text-generation seam is **already ruled OUT**; an LLM host as caller would be authoritative over human declarations while the authorship mechanism is unlanded | `AUTHORSHIP-SOURCE-PROPERTY` landed **code-bearing** + an accepted text-gen-seam design + a consumer freeze (F4-LLM un-parked) |

## Regression watch

The workstream can redden the **538** baseline in exactly **four** ways:

1. **Envelope-strict `/rag/query` vs the existing e2e POSTs.**
   `tests/gnosis_server_e2e.rs:188-198` and
   `tests/blind_p2_gnosis_server_greens.rs:437-447` already post **full envelopes with
   an extra `args` key** and expect **503 (not-READY)** *before* payload work. **Guard:**
   U1 must state that **unknown payload keys are tolerated** (the CRUD decoder sets
   that precedent — no `deny_unknown_fields` in `src/wire/`) and **absent `mode` →
   Flat**.
2. **The 14-row `route_bijection` assertions** (4 conformance rows, 1 property row
   `p_im_3`, 1 blind-green). **Guard:** U1 restates `P-IM-3`/§5.2 **first**; U4
   rewrites them **in its own unit** and **supersedes the old blind-green**.
3. **The V-8 subsystem-flag goldens** (`tests/wire_conformance.rs` literals + fixture
   helpers, plus flag assertions in the server suites). **Guard:** amend F2 §9 **+**
   the goldens in **U1**, change the flags in **U3**, in the **same unit as the
   literal edit**.
4. **`validate_rag_options` (`src/store/mod.rs:4293`) is shared** by
   `rag_query`/`rag_stream`/`bm25_search`. **Guard:** the mode check must be **purely
   additive** (no reordering of existing validation ⇒ **unchanged error precedence**),
   and **U2's trio must include the retrieval-stack + property suites**.

## Open questions for the user

**All five are now ANSWERED (2026-09-16).** The first answer scoped the go-ahead to
**U0 only / hold**; the follow-up instruction **"clean the JS files and then proceed"**
superseded it by extending the go-ahead to **U1**. On that instruction the two items that
had come back as an **elaboration request** (Q2, Q3) are **accepted as recommended by
proceeding** — the user did not select the options option-by-option — and Q4/Q5 had already
been answered outright (durability direction; delete the vendored copies). Each item keeps
its options and the gate's recommendation alongside the answered state. **Appendix A**
(the Q2 cursor elaboration) and **Appendix B** (the Q3 FS-13/14/15 elaboration) stand as
the accepted reasoning, each carrying its own status footer.

1. **Scope of the go-ahead.** (A) U0+U1 only, re-gated at U1's spec review; (B) U0–U5
   in one workstream; (C) U0–U3. **ANSWERED: U0 only / HOLD — then SUPERSEDED** by the
   user's follow-up instruction **"clean the JS files and then proceed"** (2026-09-16),
   which extended the go-ahead to **U1**. The passing review stands; **U1 (the P2/F2
   contract amendment) is now AUTHORIZED** and may proceed to the spec gate, while
   **U2/U3/U4/U5 remain HELD** (the approved scope is U0+U1, re-gated at U1's spec
   review). *(The gate had recommended (A); the user granted (A) in two steps.)*
2. **Cursor instead of a durable revision** (re-shapes GR-4/GR-5). (A) accept the
   opaque change cursor (no consumer-visible store-wide revision; not point-in-time;
   per-document `revision` stays the concurrency token); (B) insist on a
   durable/point-in-time revision — which re-opens `SHARDED-RWLOCK-STORE`,
   `IMMUTABLE-DERIVED-SNAPSHOT` and the held-off writer-actor. **ANSWERED (via the
   "proceed" instruction, 2026-09-16): Option A accepted — the opaque change cursor.**
   The recommendation was accepted **by proceeding, not by an explicit option-by-option
   selection**; the requested elaboration is landed as **Appendix A**, which now carries
   the accepted ruling. Consequence: the decision **`GNOSIS-CHANGE-CURSOR` is ACTIVE**
   in `docs/decisions.md`; the code-bearing unit (**U4**) is **not yet authorized**.
3. **FS-13/14/15 reconcile.** (A) explicit-leg-only errors (`mode=vector` unbuilt →
   `VectorIndexUnavailable` → 503) while hybrid fusion keeps degrading a failing leg
   to empty; (B) errors in hybrid too (needs an unreachable-error carve-out and
   contradicts §4.5.1's graceful degradation). **ANSWERED (via the "proceed"
   instruction, 2026-09-16): Option A accepted — explicit-leg errors, fusion degrades.**
   Accepted **by proceeding, not by an explicit option-by-option selection**; the
   requested elaboration is landed as **Appendix B**, which now carries the accepted
   ruling. Consequence: the reconcile was scheduled for and is now **LANDED by U1**
   (2026-09-16) as `docs/specs/engine-wire-contract.md` §16 + `docs/specs/p2-gnosis-server.md`
   §5.9 — **the long-open HANDOFF FS-13/14/15 row is closed** (its row 41 carries the
   SUPERSEDED marker and stays undeleted for provenance); the code units that touch the
   query/flag paths (**U2/U3**) remain **HELD**.
4. **Will the engine ever own a durable corpus** (GR-7 / the consumer's O-8)?
   (A) no — parked with a named trigger, the consumer owns persistence; (B) yes —
   schedule a durability design unit after the consumer's `SINGLE-WRITER-STORE`
   amendment. **ANSWERED: YES — schedule a durability design unit** (option B; **not
   yet authorized to start** — the approved scope is U0+U1, re-gated at U1's spec review,
   so the design unit begins only on a future go-ahead, and it stays gated on the
   consumer's `SINGLE-WRITER-STORE`/O-8 amendment plus a scheduled consumer unit; see
   the updated GR-7 row in `docs/pending.md` and the `ENGINE-DURABLE-CORPUS-DIRECTION`
   **DIRECTION ONLY / NOT ACTIVE** line in `docs/decisions.md`). *(The gate had
   recommended (A); the user chose (B).)*
5. **The vendored consumer-client TS copies in this engine tree.** (A) delete them in
   U0 (this repo is Rust-only; the consumer repo is the source of truth); (B) keep
   them with a defect row recording them as known-stale. **ANSWERED: DELETE — landed in
   two passes (2026-09-16).** First pass: `git rm` removed the four files named in the
   defect row — `engine-rag-store.ts`, `engine-crud-rag-store.ts`,
   `src/main/engine-rag-store.ts`, `src/main/engine-crud-rag-store.ts` — and the
   then-empty `src/main/` directory. Second pass (this one, on the user's "clean the JS
   files" instruction): `git rm` removed the fifth stray copy **`retrieval.ts`** (repo
   root) **and** the two orphaned vendored test files
   (`tests/unit-a1-crud-routing-proxy.test.ts`,
   `tests/props-a1-crud-routing-proxy.test.ts`); a follow-up sweep finds **no remaining
   `.ts`/`.js`/`.mjs`/`.tsx`/`.jsx` file** anywhere outside
   `target/`/`archive/`/`.cargo-home/`, and the Rust baseline is unchanged (538/0). All
   three `docs/defects.md` rows are **FIXED**; the deletions were actioned by the
   supervisor, not by this docs pass.

## Go-ahead record

- **U1 IS LANDED (2026-09-16).** The P2/F2 contract amendment was authored by the SpecWriter into
  `docs/specs/p2-gnosis-server.md` (§U1 + §5.3–§5.9) and `docs/specs/engine-wire-contract.md`
  (§4.5–§4.7, §9.1, §16), plus the upstream reconcile ask in `docs/HANDOFF.md`; the **spec-gate
  reviewer loop** then ran read-only and `file:line`-grounded and was driven to **EMPTY** —
  **remand 1** returned **4 must-fix + 7 should-fix + 6 notes** (F1–F17, all fixed), **remand 2**
  returned **1 must-fix + 2 should-fix + 4 notes** (N1–N6, all fixed), and the **final verification
  returned `VERDICT: EMPTY — no must-fix findings; the spec gate may open for U1's dependent
  units.`** The unit was **docs-only** (no `src/`/`tests/`/`Cargo.toml`/cargo change, zero new
  property rows, the canonical `docs/specs/gnosis.md` and the frozen
  `docs/specs/p1a-document-crud-wire.md` untouched) and the verified baseline is **unchanged:
  538 pass / 0 fail · `fmt --check` exit 0 · clippy 0 warnings · build clean**.
  **⇒ THE SPEC GATE IS OPEN.** **U2–U5 remain HELD**, pending the user's next go-ahead, with
  **U2** (the query POST contract: shared payload decoder, envelope strictness, mode resolver +
  unknown-mode rejection on POST *and* SSE, structured JSON decode-error body) and **U3** (status
  honesty: derive the six subsystem flags from real state; `vector` false until U5, F2 §9 + V-8
  goldens in the same unit) as the **recommended next scope**; **U4** (change cursor + paged reads
  + `GET /changes`) and **U5** (boot vector-index build) stay **HELD**, with **U4 additionally
  requiring SHELL-2 on the consumer side** and U5 following U3.
- **Go-ahead: GIVEN — U0 (landed) + U1 (authorized, then landed) on 2026-09-16.** The review
  **passed**. The gate's re-shaping (U0–U3 as amended, U4/U5 gated, GR-6/GR-7/GR-8
  parked, GR-9 refused) stands as reviewed. U0 — this gate-1 record, the tracker-truth
  landing, and the JS cleanup — is **DONE**; **U1 (the P2/F2 contract amendment)** was
  **AUTHORIZED** by the user's instruction **"clean the JS files and then proceed"**
  (2026-09-16), **ran through the spec gate, and is LANDED** (see the bullet above).
- **U2/U3/U4/U5 remain HELD pending the user's next go-ahead — now that U1's spec review has
  returned EMPTY, it is the GO-AHEAD alone that is outstanding.** The approved scope was **U0+U1,
  re-gated at U1's spec review**; that review is **done and EMPTY** (the spec gate is **open**), so
  U2 (query POST contract), U3 (status honesty), U4 (change cursor + paged reads + change feed) and
  U5 (boot index build) are **NOT yet authorized** — they become delegable on the user's word, and
  no consumer/code unit may start from this record alone. U1 was **contract-only** and landed
  **alone** so U2/U3/U4 each have authority to code against.
- **U1's two blocking questions were answered, and U1 has since LANDED.** U1 was the unit that pins
  the **change-cursor contract** (Q2), the **route-growth invariant** (P-IM-3/§5.2 restated) and the
  **FS-13/14/15 reconcile** (Q3); both elaborations were resolved by the "proceed" instruction —
  **Q2 = Option A (the opaque change cursor)**, **Q3 = Option A (explicit-leg errors, fusion
  degrades)** — so U1 was unblocked, ran to an **EMPTY** spec review, and is **landed**. As for all
  the units, the answers were accepted **by proceeding rather than by an explicit
  option-by-option selection**.
- **The two decisions are now ACTIVE:** `GNOSIS-CHANGE-CURSOR` +
  `SUBSYSTEM-FLAG-CAPABILITY-SEMANTICS` are **ACTIVE rows in the `docs/decisions.md`
  Gnosis-local table** (landed 2026-09-16). They governed **U1's contract amendment**, which has now
  **landed** (`docs/specs/p2-gnosis-server.md` §5.3–§5.9 + `docs/specs/engine-wire-contract.md`
  §4.5–§4.7/§9.1/§16); the
  code-bearing units that realize them (U4 for the cursor, U3 for the flag semantics)
  are **not yet authorized**. `ENGINE-DURABLE-CORPUS-DIRECTION` stays **DIRECTION ONLY /
  NOT ACTIVE** (a durability **design** unit on the roadmap, gated on the consumer's
  `SINGLE-WRITER-STORE`/O-8 amendment, not authorized to start). Still **no `src/` or `tests/`
  change** by any of these docs passes — including no amendment
  to `docs/specs/gnosis.md` or `docs/specs/p1a-document-crud-wire.md` (the two artifacts U1
  deliberately left untouched, asking for the upstream reconcile via `docs/HANDOFF.md` instead).
- **The answers received (2026-09-16), recorded:** **Q1 = U0 only / HOLD, then
  SUPERSEDED** — the follow-up instruction **"clean the JS files and then proceed"**
  extended the go-ahead to **U1** (the unit has since **LANDED**); **Q2 = ANSWERED via "proceed" — Option A accepted
  (the opaque change cursor)** (Appendix A now carries the accepted ruling); **Q3 =
  ANSWERED via "proceed" — Option A accepted (explicit-leg errors, fusion degrades)**
  (Appendix B now carries the accepted ruling); **Q4 = YES, the engine will eventually
  own a durable corpus — a durability **design** unit is now on the roadmap, not
  authorized to start, gated on the consumer's `SINGLE-WRITER-STORE`/O-8 amendment**
  (recorded in `docs/pending.md`'s GR-7 row + the `ENGINE-DURABLE-CORPUS-DIRECTION`
  **DIRECTION ONLY / NOT ACTIVE** line in `docs/decisions.md`); **Q5 = DELETE, landed in
  two passes** — the four vendored consumer-client copies + the empty `src/main/`
  directory (2026-09-16), then `retrieval.ts` + the two orphaned TypeScript test files
  (this pass), all actioned by the supervisor; a sweep finds no stray TS/JS left, and all
  three `docs/defects.md` rows are **FIXED**.
- **Until U2–U5 are authorized:** U1 landed and amended **contracts/specs only** (no `src/`/
  `tests/`/`Cargo.toml`/cargo change by that unit, and none by these docs passes); no cargo run is
  owed for the docs passes (no code changed); the baseline recorded above stays the verified one.
- **What U0 has already done (the first U0 pass, docs-only):** this gate-1 record; the
  PARKED rows + triggers in `docs/pending.md`; the two OPEN rows in `docs/defects.md`;
  the HANDOFF cross-reference/annotation/parked-request rows in `docs/HANDOFF.md`; and
  the CURRENT WORK update in `docs/next-steps.md`.
- **What the gate-answer recording pass has now done (second U0 pass, docs-only):**
  the header verdict/status lines; the answered Q1/Q2/Q3/Q4/Q5 states above; this
  §Go-ahead record; the corrected vendored-copy claim in §Validity findings; **Appendix
  A** (the Q2 change-cursor elaboration) and **Appendix B** (the Q3 FS-13/14/15
  elaboration) appended to this record; `docs/defects.md` (the vendored-copies row moved
  from OPEN to **FIXED**, plus one new OPEN row for `retrieval.ts`); `docs/pending.md`
  (the GR-7 row re-stated as the user-answered roadmap direction); `docs/decisions.md`
  (the `ENGINE-DURABLE-CORPUS-DIRECTION` direction-only line); and `docs/next-steps.md`
  (the closed-U0 CURRENT WORK block + the updated U0 DONE row).
- **What this third U0 pass has now done (docs-only, on the "proceed" instruction):**
  the JS cleanup landed (`git rm` of `retrieval.ts` + the two orphaned vendored test
  files; no stray TS/JS remains; baseline unchanged 538/0); the review record's header
  verdict/status lines, §Open questions (Q1 superseded; Q2/Q3 answered via "proceed"; Q5
  as two passes), §Validity findings (fully cleaned) and this §Go-ahead record; the
  **Appendix A + Appendix B status footers**; **two new ACTIVE decision rows**
  (`GNOSIS-CHANGE-CURSOR`, `SUBSYSTEM-FLAG-CAPABILITY-SEMANTICS`) in `docs/decisions.md`;
  `docs/defects.md` (the `retrieval.ts` + orphaned-test rows moved to **FIXED**; the
  status-honesty OPEN row noted as U1-authorized / U3 not-yet-authorized);
  `docs/pending.md` (the gate-1 PARKED section header); `docs/HANDOFF.md` (the
  GR-6/GR-7/GR-8 + FS-13/14/15 go-ahead clause); and `docs/next-steps.md` (the
  U1-authorized CURRENT WORK block + the U0 DONE row). **Docs/tracker-only — no code
  unit started, and U2–U5 stay HELD pending the user's next go-ahead (U1's spec review has since returned
  EMPTY — see the fourth-pass bullet below).**

- **What this fourth pass has now done (docs-only, the U1-landing reconciliation):** recorded the
  **U1 landing** and the **EMPTY** spec-gate verdict (header verdict/status lines; the U1 row of
  §Ordered workstream; the new U1 bullet + the held/answered clauses of this §Go-ahead record; the
  Q3 answer's "scheduled for U1" clause → **landed by U1**; the two appendix status footers
  reconciled to the landed state), and landed the tracker truth in `docs/next-steps.md` (the
  `U1 DONE` row + the U1-landed / spec-gate-OPEN CURRENT WORK block), `docs/pending.md` (the gate-1
  PARKED section header), `docs/decisions.md` (the two ACTIVE rows' Source cells now cite the landed
  U1 spec sections) and `docs/HANDOFF.md` (the FS-13/14/15 row + the addendum's landed-state note).
  **Docs/tracker-only — no code unit started; `docs/specs/p2-gnosis-server.md` and
  `docs/specs/engine-wire-contract.md` were NOT edited by this pass, and U2–U5 stay HELD pending the
  user's next go-ahead.**

- **POST-RECORD UPDATE 3 (2026-09-16, post-greens documentation review; docs-only) — the U2 landing, recorded
  against this record's gate-time HELD clauses.** The user authorized **U2/U3** on 2026-09-16 (registers authored
  in `docs/specs/p2-gnosis-server.md` §9.5.1/§9.5.2) and **U2's code has since LANDED REALIZED-GREEN (2026-09-17)**
  — `cargo test` **582 passed / 0 failed** (`fmt --check` exit 0, `clippy --all-targets` 0 warnings, `build`
  clean), all **7** §9.5.1 rows **HELD** with the layer's **400 executed cases** (≤400 per-unit cap), the blind set
  **24/24**, and the live battery's **7/8 rows PASS live** including **`R-L1`** on a READY boot with a real
  provider; **`R-L2`** is U3's provider-reachable boot half. **Only U3's code was owed; U4/U5 remain NOT
  authorized.** *(Historical clause read: "**Only U3's code was owed**" — closed by the EXTENDED note below.)* **EXTENDED (2026-09-17, the U3 documentation pass): U3's code has LANDED-GREEN** — the six flags
  are derived at read time inside `get_engine_status` (`src/store/mod.rs:4193-4232`), the lib seam is
  `boot_wiring` (`src/lib.rs:69-110`), the boot writes **no** flag mask (`src/bin/gnosis_server.rs:378-408`), and
  the V-8.1/V-8.2 golden literals were amended in the same unit (`tests/wire_conformance.rs:1061-1112`); verified
  **`cargo test` 604 passed / 0 failed** (serial), the five §9.5.2 rows HELD (`127` executed ≤ 400), the blind set
  **13/13**, and the live battery **9/9 rows PASS live** including **`R-L2`** (so **no U2/U3 code is owed any
  longer**); the hard-coded-flags defect row is marked **FIXED (U3)** in `docs/defects.md`. This update is the single dated marker for the gate-time clauses it supersedes; each stands
  above as written:
  - **§Verdict/§Status** (`:12-17`, `:35-44`) — "**U2–U5 remain HELD**" ⇒ **U2 LANDED (2026-09-17); U3
    LANDED-GREEN (2026-09-17); U4/U5 NOT authorized** (the header's POST-RECORD UPDATE notes carry the same reading).
  - **§Go-ahead record** — "**U2–U5 remain HELD**" (`:437-439`), "**U2/U3/U4/U5 remain HELD** … are **NOT yet
    authorized**" (`:450-456`), "**Until U2–U5 are authorized:**" (`:490-492`), "**U2–U5 stay HELD**"
    (`:517-519`), "**U2–U5 stay HELD**" (`:529-531`) ⇒ the same reading.
  - **§Appendix B** — the interlock's "both **HELD**" (`:674`) and the status footer's two clauses (`:653-668`)
    ⇒ **U2 LANDED (2026-09-17); U3 LANDED-GREEN (2026-09-17); U4/U5 NOT authorized** (the footer's own
    SUPERSEDED marker is extended by this update).
  - **§Blast radius** — the "**538** tests" baseline (`:294`) is the **gate-time** verified baseline; the current
    tree's baseline is **582 / 0** after U2 (below).
  - **§Ordered workstream / effort calibration** (`:280`) — the forecast band "**≈ 56–80 new tests
    (→ ≈594–618 total)**" was recorded at gate time; **U2's actual contribution is 44 new tests — 20 in-crate**
    (7 property rows + `u2_layer_budget_discipline` + 11 e2e + the V-15/V-15.1 golden: `props_gnosis_server`
    7→15, `gnosis_server_e2e` 16→27, `wire_conformance` 57→58) **plus the 24-scenario blind set** as a separate
    verification layer — i.e. **538 → 582**, with U3/U4/U5 still to come. The band's arithmetic is left as the
    forecast it was.
  - **Gate-time `src/bin/gnosis_server.rs` citations in this record's tables** (`:329`, `:334-336` in
    §Per-GR verdicts and §Validity findings; `:73-123` in the GR-9 row and the RBAC paragraph; the
    pre-U1 query/SSE handler ranges) are **pre-U2 line numbers**, kept as the gate's `file:line` evidence:
    under the landed tree they resolve to `:385` (the `swap_snapshot(DerivedIndexes::default())` boot line),
    `:390-391` (the DEGRADED branch), `:121-201` (the decode/dispatch path) and `:206-214`/`:245-263` (the
    landed POST/SSE handlers) — the behaviour each citation describes is unchanged.
  No ruling, verdict, park, trigger, residual-risk row or appendix is changed by this update — it records
  authorization/landing state and citation drift only.

- **POST-RECORD UPDATE 4 (2026-09-17, the U3 post-greens documentation review; docs-only) — the U3 landing,
  recorded against this record's remaining code-owed clauses.** U3's code is **LANDED-GREEN** (see the
  §Verdict/§Status POST-RECORD UPDATE 3 note above and this file's §Ordered workstream U3 row): the six flags
  are derived at read time in `get_engine_status` (`src/store/mod.rs:4193-4232`), the lib seam is
  `boot_wiring` (`src/lib.rs:69-110`), the boot writes **no** flag mask (`src/bin/gnosis_server.rs:378-408`),
  the V-8.1/V-8.2 golden literals were amended in the same unit (`tests/wire_conformance.rs:1061-1112`, with
  the producer↔literal coupling at `:1129` and the frozen-shape assertion `health_report_shape_frozen` at
  `:1263`); verified **`cargo test` 604 passed / 0 failed** (serial, green **with and without**
  `GNOSIS_SERVER_OLLAMA_URL`), the five §9.5.2 rows **HELD** (`127` executed ≤ 400), the blind set **13/13**,
  and the live battery **9/9 rows PASS live** including **`R-L2`**. **So no U2/U3 code is owed any longer,
  and U4/U5 are the only units still NOT authorized.** The clauses this update supersedes (each stands above
  as written):
  - **§Ordered workstream U3 row / §Reconcile order's executed-state note** — the "AUTHORIZED (2026-09-16),
    register authored in §9.5.2 (5 rows), code owed" cell and the "next step is U3 — whose code is owed"
    note ⇒ **U3 LANDED-GREEN (2026-09-17); the order's next step is U4 after SHELL-2**, both annotated in
    place in this pass.
  - **§Go-ahead record's POST-RECORD UPDATE 3 bullet** — its "Only U3's code was owed" half ⇒ the same
    reading. The two U3-time findings this record now carries are **`P-8`** (post-boot provider loss is
    invisible to `/engine/status`) and **`P-9`** (`encode_result`'s `expect` panic path) — both **OPEN** in
    `docs/defects.md`, **neither a U3 regression**; the hard-coded-flags defect row is marked **FIXED (U3)**.
  - **§Blast radius's "the 538 tests" row** — the gate-time baseline; the current tree is **604 / 0**, and the
    gate-time U0–U5 forecast band (≈594–618 total) is now bracketed by the landed U2 (+44) and U3 (+22)
    contributions — **582 → 604: nine in-crate** (the five executed U3 property rows + `u3_layer_budget_discipline`
    → `props_gnosis_server` 15→21; `boot_wiring_couples_to_the_derived_read` → `wire_conformance` 58→59;
    `engine_status_reports_no_false_embedding_claim` → `gnosis_server_e2e` 27→28) **plus the 13-scenario blind
    set** as a separate verification layer. The band's arithmetic is left as the forecast it was.
  - **§Appendix B's footer** — its own SUPERSEDED marker is extended by POST-RECORD UPDATE 3; nothing further
    is owed there.
  No ruling, verdict, park, trigger, residual-risk row or appendix is changed by this update — it records
  landing state and the two filed findings only. Full review record:
  `archive/reviews/2026-09-17-u3-status-honesty-doc-review.md` (gitignored provenance).

## Appendix A — Elaboration requested by the user: why a change cursor and not a revision (Q2)

**What GR-4 literally asks for.** `GET /snapshot?store=…&revision=…` returning the
whole store's nodes + edges + docHeads in one response, with a monotonic `u64` bumped
on every committed write; a request for an older revision returns either the newest
(with its revision) or a structured `stale_revision` error. GR-5 then rides the same
counter on a change feed.

**Why the engine cannot honour it as written.**

1. **No point-in-time read exists to back the claim.** The store is sharded with
   per-shard `RwLock`s (`SHARDED-RWLOCK-STORE`); there is no global store lock, so
   enumerating all shards cannot be atomic. "The store as of revision N" would be a
   promise the engine cannot keep — a torn read presented as versioned truth. The repo
   already refuses this class of claim elsewhere (`IMMUTABLE-DERIVED-SNAPSHOT` makes
   every derived artifact an atomically-swapped immutable `Arc`, precisely so that no
   reader ever sees a half-built view).
2. **The only store-wide counter is the derived-index `epoch`, and it is the wrong
   counter.** `epoch` advances on *every* `append_journal`, including fact-staleness
   propagation, archived-target propagation and `state_annotations` inserts
   (`src/store/mod.rs:2284,2356,3348-3357`) — mutations a consumer cannot see content
   for. A consumer whose coherence rule is "re-traverse on ANY committed change" would
   re-traverse on phantom bumps.
3. **`stale_revision` would be a variant whose generating step cannot run.** The repo's
   `RESERVED-ERRVARIANTS-DISCIPLINE` bars declaring errors nothing can produce; §11's
   21-row status map plus `P-IM-3`'s exhaustive uniqueness property would need a new
   row, and `docs/specs/p2-gnosis-server.md` explicitly puts new `StoreError`
   variants/§11 rows out of scope. A transport-level code could be invented, but that is
   a new contract, not a reconcile — and a decode/transport outcome pretending to be a
   store outcome is exactly the confusion NEW-2 was written to avoid.
4. **Authority, not convenience.** A revision the consumer treats as authoritative is a
   second source of truth for graph truth, which §4.2.7.2 and decision
   `GRAPH-OWNS-RELATION-AND-MERGE` forbid: every lookup index is a derived, rebuildable
   artifact. The consumer may hold a **cache**; it may not hold a projection it outranks
   the graph with.
5. **The consumer's own architecture ruling points elsewhere.** Its recorded ruling puts
   the store crossing at main→renderer as an **async pull** while the `RagStore`
   interface is unchanged — and six of the members it wants (`listNodes`, `listEdges`,
   `edgesFrom`, `edgesTo`, `edgesByKind`, `edgesForDocument`, `docHeadForDocument`,
   `journal`) are **synchronous**. An async HTTP route cannot implement synchronous
   members; the shell needs a bounded local cache regardless (`SHELL-2`).

**What is proposed instead (option A — the gate's ruling).** One wire-visible **change
cursor**: an opaque string equal to the committed journal `seq`, advancing by exactly 1
per committed journal entry, process-lifetime monotonic, with its own accessor and
documentation distinct from the derived-index `epoch`. Documented scope:
**invalidation, resync and de-dup only** — explicitly *not* point-in-time, *not* valid
for cache validation, optimistic concurrency or state reconstruction. Per-document
`revision` remains the concurrency token. Paired with paginated, wiki-scoped,
**authoritative** reads (so the consumer reads truth from the engine rather than
reconstructing it), plus a `GET /changes` SSE feed carrying
`{cursor, wikiId, kind, nodeIds, edgeIds, timestamp}`. **Gained:** a trigger for the
consumer's re-traversal rule and a cheap "did anything change" probe. **Given up:** the
ability to ask "give me the store as of revision N".

**Option B (if a durable revision is insisted on).** That is not a route — it is a
program: a durable journal with retention, a *persisted* store-wide revision, a global
read discipline that makes a snapshot point-in-time (or an explicitly documented
tearing window), and the writer-actor that `WRITER-ACTOR-JOURNAL` deliberately held
off. It re-opens `SHARDED-RWLOCK-STORE` and `IMMUTABLE-DERIVED-SNAPSHOT`, and it
naturally belongs inside the durability design unit the user has just put on the
roadmap (Q4). If as-of-N reconstruction is ever genuinely required, that is where it
must be designed (C4 in the boundary doc), not smuggled in as a query route.

**Cost of the ruling.** One capability is deferred (as-of-N reconstruction). Everything
the consumer needs for its re-traversal coherence rule survives on the cursor.

**Status: this appendix is the elaboration the user requested — the Q2 ruling itself is
now ACCEPTED and **LANDED as contract by U1 (2026-09-16)**; the cursor's wire contract may
be cited from the U1 specs, but **no code may implement it until U4 is authorized** (see the
status footer below).**

**STATUS (2026-09-16): ruling ACCEPTED by the user's "proceed" instruction; the cursor contract is now
LANDED by U1, and the code-bearing unit (U4 for the cursor) is still not authorized.** The acceptance came **by
proceeding** (Option A stands, the alternative was not selected option-by-option), and it is what made
`GNOSIS-CHANGE-CURSOR` an **ACTIVE** row in `docs/decisions.md`; U1 has since pinned the cursor's wire contract
(`docs/specs/engine-wire-contract.md` §4.6–§4.7 + `docs/specs/p2-gnosis-server.md` §5.6), so this appendix is no
longer the only statement of the design — but **no code may implement the cursor until U4 is authorized**.

## Appendix B — Elaboration requested by the user: the FS-13/14/15 index-not-built reconcile (Q3)

**The tension.** FS-13 (hybrid with the index not built) / FS-14 (vector) / FS-15 (BM25)
want index-not-built **errors**; §4.5.1 says a failing leg is **degraded gracefully**.
Today on the `ragQuery` surface: the lexical "index" is a live shard scan, so FS-15 is
effectively unreachable there (`LexicalIndexUnavailable` surfaces only via the
`bm25_search` API); a hybrid query with no vector index silently degrades the vector leg
to empty; but `mode=vector` alone returns `VectorIndexUnavailable` (503), because the
vector path checks the index before the provider (`src/store/mod.rs:4411-4417`). So the
engine already behaves as *explicit leg → honest error; fused legs → degrade*, it has
simply never been written down — which is why the HANDOFF row has stood open.

**Option A (the gate's ruling — recommended).** Rule the two cases separately and write
both down: **explicit-leg errors, fusion degrades.** `mode=vector` (the caller named
that leg) with no index → `VectorIndexUnavailable` → 503, honest and actionable.
`mode=hybrid`/`flat` keep degrading a failing or absent leg to empty, with the trace
naming the legs. This matches what the code already does, preserves §4.5.1, sharpens
FS-13/14/15 to mean the explicit-leg case, and closes the open HANDOFF row with an
amendment rather than a behavior change. It is testable as properties: an explicit-leg
request for an unbuilt leg errors with the pinned code; a fusion request never fails
because one leg is absent; adding a leg never shrinks the result set.

**Option B (errors in hybrid too — not recommended).** It would require declaring
"index not built" reachable inside hybrid, contradicting §4.5.1's graceful degradation
and turning a degraded-but-useful hybrid query into a hard failure — a worse outcome for
the user, for a condition (an unbuilt index) that U5 is meant to eliminate at boot
anyway. Choosing B also forces a new §11 row/error allocation and re-opens `blocked_by`
semantics.

**Interlock with U5.** Once the boot index build lands (U5), the explicit-leg error
becomes reachable-but-rare — the honest end state: the caller is told when the leg it
asked for is unavailable, the fusion surface stays useful, and `/engine/status` stops
claiming a capability the engine does not have. **Landing note (2026-09-16):** the ruling
itself is no longer prospective — **U1 landed it as contract** (`docs/specs/engine-wire-contract.md`
§16 + `docs/specs/p2-gnosis-server.md` §5.9), so what remains open is the **code** that realizes it
(U3's flag derivation; U5's boot index build; both **HELD**) — **UPDATE (2026-09-16 → 2026-09-17): this
ruling's query-path consequence (U2's mode decode, §5.4) has LANDED its code; U3 remains AUTHORIZED with its
code owed and U5 remains NOT authorized** (see §Go-ahead record's POST-RECORD UPDATE 3 — and POST-RECORD
UPDATE 4 for U3's landing, which supersedes that half). *(**Post-greens
doc-review, 2026-09-17:** the "U3 … code owed" half is superseded — **U3's flag derivation LANDED-GREEN**
(2026-09-17: `get_engine_status`'s read-time derivation, the `boot_wiring` seam, the bin boot that writes no
mask, the amended V-8.1/V-8.2 literals, five §9.5.2 rows HELD with 127/400, blind set 13/13, live battery
9/9 incl. `R-L2`), so the **only** code this ruling still awaits is **U5**'s boot index build. See §Go-ahead
record's POST-RECORD UPDATE 4.)*

**Status: this appendix is the elaboration the user requested — the Q3 ruling itself is
now ACCEPTED and **LANDED as contract by U1 (2026-09-16)**, which closes the HANDOFF
FS-13/14/15 row (see the status footer below).**

**STATUS (2026-09-16): ruling ACCEPTED by the user's "proceed" instruction and now LANDED as contract
by U1; the code-bearing units (U2/U3 for the query/flag code) are still not authorized.** The acceptance came
**by proceeding** (Option A stands, the alternative was not selected option-by-option). The reconcile itself is
pinned by **U1** — **landed 2026-09-16** in `docs/specs/engine-wire-contract.md` §16 and
`docs/specs/p2-gnosis-server.md` §5.9, which **closes the long-open `docs/HANDOFF.md` FS-13/14/15 row** (its row
41 carries the SUPERSEDED marker and is kept, not deleted) — while the units that touch the query/flag code
(**U2/U3**) remain HELD.

**SUPERSEDED (2026-09-16) as to U2 and U3 (the two code-bearing units this footer names): U2/U3 are
AUTHORIZED, and their typed registers are AUTHORED in `docs/specs/p2-gnosis-server.md`
§9.5.1 (U2 = 7 rows) / §9.5.2 (U3 = 5 rows) with the execution plan §9.5.3 — so for U2/U3 only the
**code** is owed; U4/U5 remain HELD.** This footer's authorization clauses were written at the
**2026-09-16 gate/proceed time** and are the historical record of that moment, exactly as this file's
status header says of every such clause (see the **POST-RECORD UPDATE** note at the head of this file,
`:18-22`) — with this marker the footer no longer asserts the pre-authorization state as current. Two
status clauses are annotated here, each reading now: (i) the first sentence's *"the code-bearing units
(U2/U3 for the query/flag code) are still not authorized"* ⇒ **U2/U3 AUTHORIZED (2026-09-16; registers
authored in `docs/specs/p2-gnosis-server.md` §9.5.1/§9.5.2) — and U2's code has since LANDED (2026-09-17), so
only U3's code was owed — **and U3's code too has since LANDED-GREEN (2026-09-17, the U3 documentation pass:
`cargo test` 604/0, five §9.5.2 rows HELD with 127 executed ≤ 400, blind set 13/13, live battery 9/9 incl.
`R-L2`), so no U2/U3 code is owed now**; and (ii) the
closing clause's *"the units that touch the query/flag code (**U2/U3**) remain HELD"* ⇒ **U4/U5 remain
HELD; U2/U3 are AUTHORIZED and LANDED (U2 realized-green 2026-09-17, U3 landed-green 2026-09-17)**. Both clauses stand as written above, as the historical record of what
*U1's landing* left owed — **and this footer's "only the code is owed" phrase is now discharged for U2/U3 as
well (U3 post-greens doc-review, 2026-09-17: §Go-ahead record's POST-RECORD UPDATE 4), so the footer's own
marker no longer leaves U3 outstanding.** The **ruling text itself is untouched** by this marker — §16/§5.9's
explicit-leg-errors/fusion-degrades pin and the HANDOFF FS-13/14/15 row's closure are unaffected.
