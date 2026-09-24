# Gnosis — Work Queue

> **SESSION CHECKPOINT (2026-09-10, archived):** the prior session was corrupted. Its handover doc
> (`archive/sessions/2026-09-10-crud-unblock-checkpoint.md`, a gitignored archival record) has been
> **fully absorbed into the CURRENT WORK + DONE rows below** and the active trackers (A1/A2 LANDED,
> RBAC rescoped to the shell, live-scenario parks). The checkpoint itself is **obsolete** and lives
> only as archival provenance; this file is now the sole authoritative handover. **RESOLVED
> (2026-09-10):** the A1 subagent-reliability hazard is closed — all A1 files are verified on disk
> and the A1 unit is LANDED-GREEN (see the CURRENT WORK block below).

Maintained by the document-archival loop. Open work on top; finished items move
to the tracker rows they produced. This is Gnosis's local next-steps.

Gnosis is the **production graph/vector engine** of the Auspicion Suite — a
Rust backend that owns the document store, the knowledge graph, fact/citation
tracking, consistency enforcement, and the RAG/agent-memory retrieval stack. It
is the production replacement for the archived Incanter prototype. The canonical
contract is `docs/specs/gnosis.md`.

## CURRENT WORK / handover-state

**THE `GRQ-1..GRQ-11` PROPOSAL-REVIEW GATE — PASSED (PROCEED-WITH-AMENDMENTS), THE GO-AHEAD IS GIVEN, AND **U5
IS NOW AUTHORIZED** (2026-09-22).** The Astrographer shell re-filed its engine asks as one set
(`<Astrographer repo>/docs/feature-requests/gnosis-engine-prerequisites.md`, 11 requests in 5 groups, dated
2026-09-21). All four read-only gate stages ran (validity ∥ critique → architecture review → change-analysis) and
the engine-side record is **`docs/specs/gnosis-grq-inbound-review.md`** (landed by the supervisor, since the
review stages are read-only tools; the change-analysis stage is its **§15**). **Verdict: PROCEED-WITH-AMENDMENTS —
the review PASSES; NOTHING in the set is authorized** (11 conditions, §15.5). **Counts:** 11 requests mapped —
**VALID-AMENDED 7** (GRQ-1/2/3/5/6/8/9) · **DUPLICATE-ALREADY-RULED 2** (GRQ-4, GRQ-10) · **OVERSTATED 2**
(GRQ-7, GRQ-11) · **UNSUPPORTED 0**; **21 findings** (12 must-fix, 6 a-big, 1 trigger-drift, + 4 correction
findings in §15.6); **7 of 11 are re-filings** of already-adjudicated items, and **no GRQ supplies new evidence**
for disturbing a standing ruling. **What moves where:** GRQ-1/2/11 + §6(c) → the **durability design unit** (scope
only, **not authorized**); GRQ-3 → a **rider** inside it (DERIVED `durability` on `HealthReport` only); GRQ-5/6 →
**PARKED unchanged** on GR-6a's trigger; GRQ-10 + GRQ-4's bulk half → **U4** (HELD, needs `SHELL-2`); GRQ-7 →
**already fixed (U2)** with its live repro **stale**; GRQ-9 → a **docs appendix** (its decode-body half already
landed) + the engine-side stale-RBAC-clause reconcile; GRQ-8 → a **documented refusal**. **Tracker truth landed in
this pass:** the engine-side index rows + the **durable-authority fork** ask (`docs/HANDOFF.md`), the re-filing
annotations + trigger-text reconciliation on the parked rows (`docs/pending.md`), a dated pointer on the
direction-only line (`docs/decisions.md`), the new docs-layer row **`RBAC-DOC-DRIFT`** (`docs/defects.md`), and
this block. **THE GATE'S FIVE QUESTIONS — ANSWERED (2026-09-22, recorded in the record's §14 POST-RECORD UPDATE 1):**
**Q1 = (A) the engine becomes the durable authority** (the shell becomes a consumer; the consumer's O-8
`SINGLE-WRITER-STORE` amendment is the precondition — `ENGINE-DURABLE-CORPUS-DIRECTION` stands, no reversal);
**Q2 = (B) the durability design unit is NOT authorized** until that O-8 amendment is recorded consumer-side (so
**D-D1 and the GRQ-3 rider stay unauthorized**); **Q3 = (B) U5 ONLY is authorized** (the boot vector-index
build — **U4 remains HELD**, it needs `SHELL-2`); **Q4 = YES to all three docs-only asks** (the GRQ-8 refusal
paragraph, the GRQ-9 per-route posture appendix, and the `RBAC-DOC-DRIFT` reconcile — owner: SpecWriter,
docs-only, zero-row exemption, no red set); **Q5 = REFUSED** (no §6(c) "half-satisfied trigger marker").
**Trio re-run this pass (docs-only change): `cargo test` 604 passed / 0 failed
(serial), `cargo fmt --check` exit 0, `cargo clippy --all-targets` 0 warnings, `cargo build` clean.** No `src/`,
`tests/` or `Cargo.toml` byte changed. **NEXT (the authorized work): U5's spec gate** — a contract + its typed
§5.x property register must be **authored and its reviewer loop returned EMPTY** before any TestWriter derives
U5's red set (the p2 §12 obligation row for U5 exists at `docs/specs/p2-gnosis-server.md`'s U5 row, but **U5's
typed register rows are still owed** — the spec carries U2's §9.5.1 and U3's §9.5.2 only).

**U5 (the only authorized unit) — THE SPEC GATE IS OPEN-AND-IN-REMAND (2026-09-22).** U5's contract + its
**typed §5.x register** (8 rows: `P-IM-10`..`P-IM-15`, `P-SM-7`, `P-TP-5`; caps 340 ≤ 400) are **authored** in
`docs/specs/p2-gnosis-server.md` §9.5.5, with the same-unit obligations named (the V-8.1 `"vector"` literal, the
`honest_ready_subsystems` helper, the boot-wiring coupling probes), a U5 note in
`docs/specs/engine-wire-contract.md` §9.1/§12, and the live obligation homed in the battery's new **`R-L3`**
(with `R-L2`'s `vector` cell re-pointed). **The spec-gate reviewer loop is NOT yet EMPTY:** the first review
returned **13 findings (4 MUST-FIX, 5 SHOULD-FIX, 2 NOTE)** — an imprecise same-unit move table, a `P-IM-14`
observable/invariant contradiction, an unpinned bin build-failure wiring, §6's lifecycle missing the index-build
step, an un-named corpus-seeding API, the private-shard read vs "no new accessor", an unpinned `embed`-call-count
witness, two unobservable `R-L3` criteria and a register-format deviation — and a **one-pass remand is running**
(owner: SpecWriter). **THE LOOP IS NOW EMPTY: round 1 = 13 findings, round 2 = 11, round 3 = 6, round 4 = 6, round 5 = EMPTY (spec gate OPEN, 2026-09-22).** The rounds fixed, in order: the same-unit move table's exact lines; a `P-IM-14` observable/invariant contradiction; the bin's failed-build wiring; §6's missing index-build step; the corpus-seeding surface; the private-shard/no-new-accessor contradiction; the `embed`-call-count witness; unobservable `R-L3` criteria; the **FS-8-vs-FS-14 READY-gate** bug (a non-READY boot ⇒ `engine_unavailable`, FS-14 only on a READY store with `vectors: None`) in `P-SM-7`, §5.9, F2 §16 and `R-L3` (v); the `EngineSubsystems` literal census (**23**, per-file enumeration); clause-name-first §5.9 anchors; the corpus key-uniqueness rule (two nodes sharing a `NodeId` in one document = **not a valid corpus**); the duplicate-text call-count contradiction (deleted in favour of per-node calls/keys); and the store-wide corpus-scope pin. **U5's red stage is now running** (TestWriter: the new integration/conformance file + the 8 register rows' property layer + the move table's test-side reconciliations per the spec, incl. the `honest_degraded_subsystems()` doc comment). **Two non-blocking carry-overs for the U5 unit:** a dated annotation in §9.5.5 says the move table is "back to its 12 rows" while it is 11 lines/9 data rows (count slip only), and `src/store/mod.rs:4211-4212`'s comment ("the boot leaves `DerivedIndexes::default()` ⇒ `false` until U5") **must be reconciled by the Implementer in the same unit** — it is not in the move table.

**U5 — REALIZED-GREEN, WITH THE ADVERSARIAL GATE'S FINDINGS FILED AND FOUR GATES STILL OUTSTANDING (2026-09-22).**
TDD ran as spec'd: **red 24** (15 in the new `tests/u5_boot_vector_index_conformance.rs` + 8 register rows + the
budget test in `tests/props_gnosis_server.rs`, with 2 test targets compile-failing on the absent symbol) →
**green 628 passed / 0 failed** (baseline **604** + 24), green **with and without** `GNOSIS_SERVER_OLLAMA_URL`;
`cargo fmt --check` exit 0, `cargo clippy --all-targets` **0 warnings**, `cargo build` clean. **U5's property layer:
all 8 rows HELD — `P-IM-10` 55/60 · `P-IM-11` 40/50 · `P-IM-12` 40/40 · `P-IM-13` 42/45 · `P-IM-14` 25/45 ·
`P-IM-15` 27/30 · `P-SM-7` 45/45 · `P-TP-5` 40/40 = 314 executed / 355 caps ≤ 400** (seed `0x9E37_79B9_7F4A_7C15`
+ `row_seed(tag)`; `Σcaps ≤ 400`, cap = maximum). Code: `build_boot_vector_index` in `src/store/mod.rs` (body
in-module; name re-exported from `src/lib.rs`), the bin's boot wiring (`Reachable` ⇒ build + composed snapshot;
failed `Reachable` ⇒ the pinned order with the flags discarded ⇒ `Degraded` + `vector:false` + `embedding:true`),
a comment-only reconciliation at the flag-derivation site; **no new `StoreError` variant, §11 row, route, accessor
or frozen-shape change**. **The adversarial pass (gate 4, read-only, incl. the PBT audit) found NO U5 regression
requiring a host code fix in this pass, and filed:** **`U5-ADV-1`** (the primary escalation — the boot index is
frozen with no rebuild vehicle, so post-boot writes make `mode=vector` return **200 with silently missing hits**
while `vector` still reports `true`, where pre-U5 the same request was a loud FS-14 503 → **a follow-up
"U5 honesty/freshness" unit**, its own spec amendment + red set, or an explicit residual), **`U5-ADV-2`**
(unbounded + **un-timed** pre-bind build: residual (i)'s stated provider-timeout bound does not exist),
**`U5-ADV-3`** (`Degraded` + `embedding:true` + a `last_error` blaming embedding — a spec/consumer decision),
**`U5-ADV-4`** (the build's `Err` is discarded with no diagnostics), **`U5-ADV-5`** (the lock-`unwrap` poison
class now sitting on a pre-bind path), and **`P-9`'s TRIGGER FIRED** (a non-finite provider vector now reaches
`encode_result`'s `expect` on the request path — **package/foundation**, not a U5 patch). All are recorded in
`docs/defects.md` §OPEN. **The PBT audit's row verdicts:** `P-IM-13`/`P-IM-14`/`P-IM-15`/`P-TP-5` sound (with a
tautological and a wording clause flagged), `P-IM-10`/`P-IM-11`/`P-IM-12`/`P-SM-7` **under-defended** — with a
**10-item negative-generator task list (T1–T10)** for the TestWriter (cross-document duplicate `NodeId`;
perturbed-`HashMap` order; `NaN`/`±inf` bit-comparison; interior-`k` mid-build death; the bin's failed-build
branch; the P-9 non-finite probe; cross-wiki scoping; `hyde:true`; un-wiring the two vacuous counter probes; the
`p == None` observable). **STILL OUTSTANDING FOR U5 (the gates that have NOT run): blind-greens (gate 5), the live
battery (gate 6 — `R-L3` exists and its clause (iii) is parked with the recorded reason; clause (v) is **lib-level
only**; `R-L2`'s `vector` cell is re-pointed), the proofreader (gate 7) and the documentation review (gate 8).**
The spec's post-green amendments (caps `355`/executed `314`/binary total `1192`, the `NaN` comparison rule, the
`R-L3` (v) park, the Implementer's `src/store/mod.rs` comment obligation) are landed in
`docs/specs/p2-gnosis-server.md` §9.5.5/§U1/§12 and the battery file.

**U6 — THE VECTOR-INDEX FRESHNESS / HONESTY UNIT — SPEC AUTHORED, SPEC GATE IN REMAND ROUND 1 (2026-09-22).**
Charter (user-authorized): **`U5-ADV-1`** (primary — the boot index is frozen, so a post-boot write makes
`mode=vector` a silent 200 with missing hits while `vector` stays `true`; live-confirmed at gate 6) + **`U5-ADV-4`**
(the discarded build `Err` with no diagnostics) + **`U5-ADV-2`** (a real provider-request bound); **out of scope:**
`U5-ADV-3`, `U5-ADV-5`, `P-9`. **Authored:** `docs/specs/p2-gnosis-server.md` **§9.5.6** (the honesty predicate
`vector_index_is_fresh(snapshot, store) ≜ snapshot.vectors.is_some() ∧ snapshot.epoch == store.epoch()`,
evaluated at the existing read-time derivation with **both** the flag and the vector leg refusing on it so a mutated
corpus ⇒ READY store ⇒ FS-14 `vector_index_unavailable` ⇒ 503; a stderr diagnostic before the composition with a
byte-identical status surface; one `reqwest::Client` with a **fixed 30 s** `PROVIDER_REQUEST_TIMEOUT` covering probe
+ every `embed`, timeout ⇒ FS-13 503, probe timeout ⇒ `Unreachable`, build timeout ⇒ the `P-IM-15` degraded branch;
**6 typed rows** `P-IM-16`…`P-IM-19`/`P-SM-8`/`P-TP-6`, cap sum **300 ≤ 400**, binary total **1492**; a 26-row
same-unit table; 9 valid/fail states; 6 reconciliation clauses; 5 open questions) plus U6 coverage notes in §9.5.4
and U6 markers in §U1/§5.8/§6/§9.5/§9.5.3/§10/§11/§12/§13 and `engine-wire-contract.md`. **THE SPEC GATE RETURNED
17 FINDINGS (NOT EMPTY) AND A ONE-PASS REMAND IS OWED** — 9 MUST-FIX, 2 SHOULD-FIX, 6 NOTE. **The remand list (give
it to the SpecWriter verbatim):** (1) two vector-leg fixture files are missing from the same-unit table —
`tests/retrieval_stack_integration.rs`'s `seed_vectors` (`:131-141`, `epoch: 1`) with its call sites, and
`tests/props_retrieval.rs`'s `seed_vectors` (`:246-256`) with `:701`/`:1050`/`:1342` (both seed **after** corpus
mutations, so they become FS-14 / a degraded hybrid leg and their `unwrap_or_else` panics); (2) an internal
contradiction in the table (`:3498-3500` says the U5 states 1/2/3 are already aligned, `:3508` says exactly those
sites are `epoch 0` and must move); (3) the `rag_query_integration.rs` rows under-state their own obligation (the
hybrid non-degeneracy assertion at `:790-795` and the FS-13 test at `:649-682`); (4) the "unchanged" concurrent
fixtures row is **false** for the rebuild-vs-reader test (`:1757-1847`'s 8 readers at `:1799-1810` all take
`Err(VectorIndexUnavailable)` — split the row); (5) the `derived == flags` invariant has **no satisfiable remedy** on
a mismatched-epoch corpus because `boot_wiring`'s signature is pinned unchanged — state the invariant's scope
(aligned epochs only) and which of the two remedies is chosen, and say the returned flags keep the pre-U6
`is_some()` surface; (6) the **tag constants are not implementable as pinned** (7-char tags overflow the landed
≤6-char/8-byte convention — pin exact constants or 6-char names); (7) the **hybrid/predicate consumer rule is
underdetermined** (pin per site whether the shared predicate is consulted with the refuse suppressed for `hybrid`,
and `vector_search`'s stale outcome); (8) **`P-TP-6` is declared lib-level but its observable is bin-private**
(`PROVIDER_REQUEST_TIMEOUT` in the `[[bin]]`) — home it bin-level or pin a lib-visible seam; (9) two wrong internal
anchors + a promised §5.9 `Interlock with U5` marker that was never landed; (10) the `seed_store` epoch arithmetic
("≥ 2" ⇒ "1 wiki + 2 appends per document"); (11–17 NOTEs: the frozen-type-vs-derivation sentence, the
double-`snapshot()` observable, the battery's `R-L4` needing an explicit supervisor-obligation label, the blind U3
populated-index fixture in the predicate re-derivation list, the `boot_wiring` doc-comment sites, the `rag_stream`
half of one row's invariant). **Next after the remand loop is EMPTY: the red set (TestWriter) — which should also
discharge U5's **T1–T10** negative-generator list — then the Implementer, the adversarial pass, blind-greens, the
live battery (`R-L4`), the doc-review and the trio.**

**U6 SPEC GATE — REMAND ROUND 1 LANDED; ROUND 2 RETURNED 11 FINDINGS (4 MUST-FIX, 3 SHOULD-FIX, 4 NOTE) — NOT
EMPTY (2026-09-22).** Round 1's 17 findings were all fixed (the two missing vector-leg fixture files were added to
the same-unit table; the table's internal contradiction corrected; the **six tag constants pinned exactly** —
`U6PIM16 = 0x5536_5049_4D31_3636`, `…3737`, `…3838`, `…3939`, `U6PSM8 = 0x5536_5053_4D38`,
`U6PTP6 = 0x5536_5054_5036`, the landed ≤6-char zero-extended / 8-byte padded form, verified against
`tests/props_gnosis_server.rs`; a **per-site consultation table** pinning which read site consults the shared
predicate and where the refuse is suppressed for `hybrid`; a new **lib-visible surface #6** re-homing `P-TP-6`;
the `derived == flags` invariant scoped to epoch-aligned snapshots; the §5.9 `Interlock with U5` marker landed).
**Round 2's remand list (give it to the SpecWriter verbatim):** **(MUST-FIX 1)** the "already aligned" claim for
`tests/u5_boot_vector_index_conformance.rs:459-462`/`:517-520`/`:900-903` is **false** — those snapshots are also
swapped into the corpus-seeded store (`seed_store` ⇒ epoch ≥ 1) and read by `rag_query(mode=Vector)`
(`.expect("an empty index serves an empty result")`), so all three are stale under the pinned predicate: re-derive
the row by **rule** (each site's status/leg consumer and its store's epoch), not by the helper's own store;
**(MUST-FIX 2)** the same-unit + reconciliation rows claim the held `P-IM-7`/`P-IM-14`/`P-SM-7` cells and their
§9.5.4 notes are "annotated in place with dated U6 markers" — **no such marker exists at those cells**, so a
TestWriter re-running the U5/U3 layer derives the superseded predicate: land the markers or record them as explicit
supervisor obligations instead of marking them fixed; **(MUST-FIX 3)** the `retrieval_stack_integration.rs` row
misses **three inline fixtures** (`:407-411` + its `vector_search` at `:417-420`; `:516-520` + the `rag_query`
`mode=Vector` `.unwrap()` at `:540-543`; `:883-887` + the two vector-mode `.unwrap()` sites and the HyDE-positive
assertion) — each answers FS-14 / panics under U6; **(MUST-FIX 4)** the `P-IM-14` re-derivation names probes (1)–(4)
but leaves the V-8.1 block at `tests/props_gnosis_server.rs:7437-7460` (asserting `derived == v81` with
`v81.vector = index_bearing` for **every** variant, incl. the `epoch: 7` one) unaddressed — name it with the
epoch-aligned filter. **(SHOULD-FIX)** `P-IM-19`'s observable forks its layer (bin vs lib) while the plan pins
bin-level — pin one home; surface #1's lib-visible predicate is asserted by **no** row — give it `P-IM-16`'s
observable or drop the lib-visibility requirement; the corrected §5.8 anchor is still wrong (`:1671-1672` is the
sentence, not `:1654-1655`). **(NOTEs)** the per-row anchors `:2176-2179`/`:2521-2523`/`:2477-2478` land in the wrong
clauses (cite by id); the `src/bin/gnosis_server.rs` `:347`-adjacent comment is unnamed; the new stale-index case's
file is left to the TestWriter; and the cost clause's premise ("an epoch-preserving corpus write is a defect of this
predicate") is unstated — pin it (the coupling holds today: the six `guard.docs.insert/remove` sites are all paired
with `append_journal`). **Round 2 verified as holding:** the tag constants + arithmetic, caps `300 ≤ 400` / binary
total `1492`, id-freeness, the per-site table's agreement with the freshness table and states 3/5/6/7, surface #6's
non-collision with frozen artifacts, `boot_wiring`'s genuinely pre-U6 pins, the predicate's soundness against the
code it names, U3's fixture alignment, and the blind U3 populated-index site being the one correctly named RED
blind site.

**U6 SPEC GATE — REMAND ROUND 2 LANDED; ROUND 3 RETURNED 7 FINDINGS (2 MUST-FIX, 1 SHOULD-FIX, 4 NOTE) — NOT
EMPTY (2026-09-22). THE NEXT REMAND LIST (verbatim):** **(MUST-FIX 1 — a WRONG RED SET)** the same-unit row /
governing rule / §U1 list / the REMAND-2 summaries declare `tests/u5_boot_vector_index_conformance.rs:900-903`
(`built_snap`) as MOVED ("store epoch 3; the same label on the state-2 query path ⇒ FS-14"), but the tree shows its
**only** consumer is `wired_status(…)` at `:905`, whose own store is a fresh `Store::new()` (epoch 0) — it is
aligned exactly like `:459-462`, and applying `epoch: store.epoch()` (=3) makes `:907`'s
`reachable.subsystems.vector` assertion **fail (a correct test turned red)**. So of the three U5 conformance
snapshots **exactly one (`:517-520`) moves**; restate (c) as ALIGNED/NO EDIT and correct the same claim in the
governing rule, §U1 and the round-2 disposition records. **(MUST-FIX 2 — an UNSATISFIABLE remedy + a false
outcome)** the named "one held U3 row whose assertion the amended predicate falsifies" (`P-SM-5`'s probe 4,
`tests/props_gnosis_server.rs:5367-5414`) is **not red**: the store is a fresh `Store::new()` (epoch 0),
`swap_snapshot` moves no epoch, one `create_wiki` appends once ⇒ `epoch_after == 1` **==** the snapshot label `1`
⇒ fresh ⇒ `:5410`'s `vector: true` passes; the stated `epoch_after = 2` is wrong, remedy (ii) would assert a false
outcome and remedy (i) reads the literal after the mutation it must precede. Correct the premise to
`epoch_after = 1` (**no edit needed**), drop `P-SM-5`'s probe 4 from the "one named exception" list (row,
reconciliation row 4, the two marker blocks, §U1) and state that U6's own control is `P-SM-8`. **(SHOULD-FIX 3)**
`seed_store(&[])` is stated as `1 create_wiki + 1 create_document = epoch 2` — self-contradictory and wrong: the
`for values in docs` loop never runs for `&[]` ⇒ **epoch 1** (only `create_wiki`); the red set is unaffected
(0 ≠ 1) but a TestWriter hard-coding `2` still produces a stale label — state `1` and keep the remedy pinned as
`epoch: store.epoch()` (never a literal). **(NOTEs)** the lib-visible predicate `gnosis::vector_index_is_fresh`
is named only by cross-reference — add it to `P-IM-16`'s observable cell **and** its §9.5.4 note (the two surfaces
a TestWriter reads row-by-row); `P-TP-6`'s item (iii) demands a bin-configuration closure witness of a **lib-level**
row with no pinned mechanism — pin the mechanism (e.g. a source-text witness over the bin's three reads) or drop
it; the seed-pin bullet's "as landed" citations are stale (`SEED` is at `:189`, `row_seed` at `:247-249`, the tag
form at `:252-258`) — re-point or drop the numbers; and "a fifth **field**" of the boot composition should read
"a fifth **line** (the third field term)" (the composition has three fields). **Round 3 VERIFIED clean:** the
`:459-462` adjudication (the spec's consumer-anchored reading was right; the round-2 reviewer's reading was wrong),
the marker blocks as LANDED, the three inline `retrieval_stack_integration.rs` fixtures + the enumeration rule
(4 `swap_snapshot` sites), the V-8.1 re-derivation + three preconditions, `P-IM-19`'s bin home, the corrected
§5.8/§5.9 anchors, the id/`strat:`-id citations, the bin `:347` row, the pinned new-case file, the epoch-advance
assumption clause, and the whole gate-core set (ids free, kinds IM×4/SM×1/TP×1, six strategy ids, caps
`300 ≤ 400`, binary total `1492`, tags/`SEED` against the landed constants, the predicate against its four cited
code sites, and the out-of-scope boundary). **The docs-only
appendix accepted under Q4 has LANDED:** the GRQ-8 refusal (`engine-wire-contract.md` §14.1), the GRQ-9 per-route
GET-with-body posture table (§14.2, recording the decode-error half as **already satisfied** by U2), and the
`RBAC-DOC-DRIFT` reconcile (**8 stale clauses annotated in place**; the ledger row is now **FIXED (docs-only)**,
with the count corrected to 9 clauses of which 1 was already correct); a new parked row records the follow-on ask
about the four body-carrying read routes (`docs/pending.md`).

**U5 — ALL EIGHT GATES HAVE NOW RUN; THE UNIT IS DONE AND ITS DONE ROW IS WRITTEN BELOW (2026-09-22, gate 8).**
The four gates outstanding at the block above have all closed: **gate 5 (blind-greens)** —
`tests/blind_u5_boot_vector_index_greens.rs`, **27 scenarios: 27 GREEN / 0 RED / 6 NOT-VERIFIED** (2 live-HTTP +
25 lib-level), report `docs/specs/u5-boot-vector-index-greens.md` (its four disclosed derivation corrections
re-read against §9.5.5 this pass and **reconciled** — each is the doc pin, not an implementation finding; the
report's arithmetic — `628` was the interim post-green/pre-blind count, so **604 + 24 U5 in-crate = 628, then
+ 27 blind = 655** — and its `27/27` claim agree with the tree (its "628 baseline" label was corrected in the
report itself this pass); **gate 6
(live battery)** — `R-L3` **(i)/(ii)/(iv) PASSED live** on a controlled provider, **(iii)** and **(v)**
**PARKED with their reasons re-confirmed live**, the **FS-14 READY-index-free mapping** newly parked as
structurally non-reachable through the bin post-U5, `R-L2`'s amended `vector:true` cell **re-verified live**,
and one **doc-drift finding F1** (the `R-L3` (iv)/(v) cells' 503 code — FS-14 vs FS-8 — fixed in place with the
superseded wording dated); **gate 7 (proofreader)** — no pinned rule/row/count moved; **gate 8 (this pass)** —
the review record is `archive/reviews/2026-09-22-u5-boot-vector-index-doc-review.md` and the reconciled files are
`docs/specs/p2-gnosis-server.md` (§U1's verification-status block + §9.5.5's heading + §12 item 5), `docs/specs/engine-wire-contract.md` (§U5/§9.1/§12 status notes), `docs/specs/p2-gnosis-server-live-pending-battery.md` (§3.5's `R-L3` cells + §4 + the new §8), `docs/specs/u5-boot-vector-index-greens.md` (its gate-8 reconciliation note), `docs/defects.md` (the `U5-ADV-*` rows' `file:line`), `docs/decisions.md`, `docs/pending.md`, `docs/HANDOFF.md` and this file. **Verified state:**
**`cargo test` 655 passed / 0 failed** (serial; 604 baseline → +24 U5 in-crate → +27 blind), hermetic with and
without `GNOSIS_SERVER_OLLAMA_URL`; property layer **314 executed / 355 caps ≤ 400**; `cargo fmt --check` exit 0 ·
`cargo clippy --all-targets` 0 warnings · `cargo build` clean. **What stays OPEN:** the `U5-ADV-1`…`U5-ADV-5`
rows plus **`P-9`'s fired trigger** in `docs/defects.md` (a follow-up **"U5 honesty/freshness"** unit is being
opened for `U5-ADV-1`) and the PBT audit's **T1–T10** negative-generator list as an undis-charged **TestWriter**
obligation. **The failed-`Reachable`-build branch is lib-level only — not live-verified.** **U4 remains HELD**
(it needs the consumer-side `SHELL-2`).

**U2 — THE QUERY POST CONTRACT — REALIZED-GREEN / LANDED, AND ITS DONE ROW IS NOW WRITTEN (see the U2 DONE row
below; doc-review reconciled 2026-09-16).** U2's code landed (`src/wire/query.rs`, the
`src/wire/mod.rs`/`src/lib.rs` re-exports, and `src/bin/gnosis_server.rs`'s shared JSON decode-error renderer +
SSE pre-stream `error` frame + checked POST handler). Verified: **`cargo test` 582 passed / 0 failed** ·
`cargo fmt --check` exit 0 · `cargo clippy --all-targets` 0 warnings · `cargo build` clean; U2's property layer
**400 cases HELD** (the seven rows `U2PIM4`…`U2PTP4` — the landed layer is 400 of the **per-unit** ≤400 cap); the
blind set `tests/blind_u2_query_post_greens.rs` **24/24** (13 live-HTTP + 11 pure-lib; its `U2-13` defect fixed
+ the spec hole closed); the live battery's **`R-L1` PASSED live** on a READY boot with a real provider (Ollama
`embeddinggemma`), with **`R-L2`** being **U3's** provider-reachable boot half and out of U2's scope. U2's
adversarial pass, its read-only PBT audit and the doc audit are recorded in `docs/specs/p2-gnosis-server.md`
§U1 (the U2-landed bullet + the U2 post-greens doc-review bullet + the proofread-pass corrections),
`docs/specs/engine-wire-contract.md`'s U2 status note, `docs/specs/u2-query-post-contract-greens.md`,
`docs/defects.md` (**P-1..P-5** OPEN; **P-6**/**P-7** fixed/closed) and `docs/HANDOFF.md` §"U2 adversarial
pass". **CURRENT QUEUE: U3 (status honesty) IS LANDED-GREEN (2026-09-17) — its DONE row is written below (the
documentation-review gate reconciled it in the same pass,
`archive/reviews/2026-09-17-u3-status-honesty-doc-review.md`).** U3's code landed (the six flags derived at read time inside `get_engine_status`
(`src/store/mod.rs:4193-4232`), the lib-visible seam `boot_wiring` (`src/lib.rs:69-110`), the bin boot that
applies only the returned `EngineState` plus its own wiring and writes **no** flag mask
(`src/bin/gnosis_server.rs:378-408`), and the V-8.1/V-8.2 golden literals amended in the same unit
(`tests/wire_conformance.rs:1061-1112`)); `set_subsystems` is observationally inert, `EngineSubsystems` is
still exactly six `bool`s and `HealthReport` gained **no** field. Verified: **`cargo test` 604 passed / 0
failed** (serial `-- --test-threads=1`), green **with and without** `GNOSIS_SERVER_OLLAMA_URL` set (the suite
is hermetic); `cargo fmt --check` exit 0 · `cargo clippy --all-targets` 0 warnings · `cargo build` clean;
U3's property layer **127 executed ≤ 400** (the five rows HELD — `P-IM-7` 25 / `P-IM-8` 26 / `P-IM-9` 26 /
`P-SM-5` 25 / `P-SM-6` 25); the blind set `tests/blind_u3_status_honesty_greens.rs` **13/13** (3 live-HTTP +
10 pure-lib); a **nine-in-crate** test contribution (582 → 591) plus the 13 blind scenarios; and the live
battery **9/9 rows PASS live**, including **`R-L2`** (provider-reachable boot ⇒ `state:"Ready"`,
`embedding:true`, `vector:false`, `reranker:false`, core true — with `mode=vector` in the same session ⇒ 503
`vector_index_unavailable`, so status and query agree) and the configured-but-unreachable boot ⇒ `Degraded` +
`embedding:false` + the pinned `lastError`. U3's two open findings (post-boot provider loss is invisible to
`/engine/status`; `encode_result`'s `expect` panic path) are filed as `docs/defects.md` **P-8**/**P-9** — both
**OPEN**, neither a U3 regression, both cross-filed in `docs/HANDOFF.md` §"U3 documentation pass (2026-09-17)".
The register (`docs/specs/p2-gnosis-server.md` §9.5.2: `P-IM-7`/`P-IM-8`/`P-IM-9`/`P-SM-5`/`P-SM-6`) and the
execution plan (§9.5.3) are unchanged in substance; its live row is **`R-L2`** (now PASSED). U3's
post-greens documentation review (gate 8, 2026-09-17) reconciled the U3-bearing specs and these trackers
in the same pass — `archive/reviews/2026-09-17-u3-status-honesty-doc-review.md` (gitignored provenance);
the U3 DONE row is now written in the DONE table below. **NEXT: U4 (change cursor + paged reads + `GET /changes`) and U5 (boot
vector-index build) remain NOT
authorized** (U4 additionally needs the consumer-side `SHELL-2`; U5 follows U3 and owns the one-value
`vector:true` flip). **SUPERSEDED (2026-09-22, gate 8): U5 WAS AUTHORIZED and has since LANDED-GREEN with all
eight gates run (see the U5 CURRENT WORK blocks + the U5 DONE row above), so only `U4` remains NOT authorized
(it alone needs the consumer-side `SHELL-2`); the clause above stands as the 2026-09-17 record.** The durability design unit
(`ENGINE-DURABLE-CORPUS-DIRECTION`) stays **roadmap-direction-only / NOT ACTIVE / not authorized**. The parked
items keep their measured triggers exactly as recorded in `docs/pending.md` (GR-3's maintenance half, the GR-4
revised-projection refusal, GR-6a/GR-6b, GR-7's durability direction, GR-8, route growth beyond U4). **The U2/U3
status clauses elsewhere in this file and in the other trackers now read "U2 LANDED / U3 LANDED-GREEN
(2026-09-17); U4/U5 HELD"** — the pre-U3 "U2/U3 AUTHORIZED … only their code is owed" wording is annotated in
place wherever it survives (the U2 clauses were reconciled by the previous pass, the U3 clauses by this one).
The numbers in the historical blocks below are each unit's own dated pass record; the current tree's
baseline is **655 / 0** (U5's landing, 2026-09-22 — the pre-U5 reading `604 / 0` was this file's line until the
U5 pass; both are dated records, and the **655** figure is the one the U5 DONE row and the gate-8 review
verified).

**Handover state (historical, 2026-09-10 — kept as the P2/CRUD-MVP record): the core engine, the wire-contract
seam, the F6
RAG-evaluation dev/QA gate, the F4 community-retrieval unit, the §7.2 P1a
document-CRUD wire-contract unit, AND the §7.2 P2 `gnosis-server` binary crate
are fully implemented, adversarially hardened, and trio-green.** §4.1–§4.6 (the
core engine), the §7.2/F2 engine wire contract, the §7.8/F6 RAG-evaluation
harness, the §7.5/F4 community-retrieval accessor, the §7.2/P1a document-CRUD
wire shapes, and the §7.2/P2 `gnosis-server` bin are all DONE (see the DONE rows
below) — **523 green tests** (baseline 454 + the P2 set 63:
`gnosis_server_conformance.rs` 25 + `props_gnosis_server.rs` 7, all 7 property
rows HELD + `gnosis_server_e2e.rs` 16; the P2 blind-greens
`tests/blind_p2_gnosis_server_greens.rs` 15 = 15/15 scenarios PASS are a separate
verification layer), with `build`/`clippy`/`fmt` clean.
**All previously-unparked items in this repo's scope are now complete**:
`resolve_entities` authoritative-overwrite, the UUID-v4 id decision (RESOLVED as
`ID-SCHEME-RECONCILE-MONOTONIC`), F6, F4, P1a, and P2. **The Astrographer-side
CRUD routing client (A1) is LANDED (2026-09-10)** — the shell-side client that
consumes the same paths + shapes the `gnosis-server` bin serves (the 11 §4.1
document-CRUD wire client over the frozen P1a wire; see the A1 DONE row in the
Astrographer `docs/next-steps.md` + the ENGINE-CRUD-WIRE-CLIENT / ENCODE+DECODE /
P4-IDEMPOTENCY-RETRY decision rows in the Astrographer `docs/decisions.md`). **The
Astrographer-side A2 document-CRUD D4 wiring is LANDED (2026-09-10)** — the final
MVP unit: the 11 `gnosis.document.*`/`gnosis.wiki.*` MCP tools + the `gnosis-edit`
group + the extended `handleGnosisTool` + the `AuthorityStore` + the
`IdempotencyRegistry` + the GUI document-editor/wiki screens over the LANDED A1
proxy (see the A2 DONE row in the Astrographer `docs/next-steps.md` + the
GNOSIS-CRUD-EDIT-GROUP / GNOSIS-DOCUMENT-TOOLS / GNOSIS-RBAC-CALLER-STORE /
GNOSIS-409-UX / GNOSIS-IDEMPOTENCY-DEDUP decision rows in the Astrographer
`docs/decisions.md`). **THE DOCUMENT-CRUD MVP IS COMPLETE (P1a + P2 + A1 + A2).**
The P1a/P2/A1/A2 live batteries un-park on a running `gnosis-server` + a running
app (the A2 live battery is PARKED — the Gnosis engine is absent; revisit
condition: the engine running on loopback + the `gnosis-edit` group enabled).
**RBAC caller threading RESOLVED as re-scoped to the shell (2026-09-10, proposal
gate — `docs/specs/rbac-caller-review.md`):** the engine-side RBAC enforcement is
deliberately NOT implemented (the engine has no authority-mapping source; the C5
boundary + the pure-backend framing place the authorization gate at the SHELL;
the shell's A2 caller-side deny is the enforcement). Decision
`GNOSIS-RBAC-EDIT-ENFORCEMENT` amended to align with C5 (RBAC → SHELL); the
`defects.md` + `HANDOFF.md` OPEN rows closed as re-scoped to the shell; a
canonical-contract reconcile request recorded in `docs/HANDOFF.md`. A
fresh supervisor picks up the **shell-integration unit** next (SSE client +
bind/auth/TLS + full `RagStore` CRUD routing + boot→READY lifecycle +
HTTP-status rendering + D2-absent shell behavior). The **suite-side DeepEval Python harness**
(faithfulness/answer-relevancy, answer generation, user-facing retrieval-quality
surface), the **F4-LLM integration** (a suite tool with a harnessed LLM drives
Gnosis's manual-override-authoritative enrichment surfaces), and the
**authorship-source code-bearing unit** (decision AUTHORSHIP-SOURCE-PROPERTY)
remain deferred/out-of-repo and are recorded in `docs/specs/7-2-f2-review.md`
(§"Deferred / OUT of F2" + §"What a later shell-integration unit must own"),
`docs/specs/6-f6-eval-review.md`, `docs/specs/f4-llm-enrichment-integration.md`,
`docs/specs/authorship-source-review.md`, `docs/pending.md`, and
`docs/decisions.md` (`F2-WIRE-CONTRACT-A1`, `F6-EVAL-RE-SCOPED`,
`F4-COMMUNITY-RETRIEVAL`, `AUTHORSHIP-SOURCE-PROPERTY`). The dated entries below
are the historical scaffold→implemented record.

**P1a/P2 HYGIENE FIX (2026-09-10, host-side lint/format pass):** the P1a/P2 landing's
trio claim ("clippy 0 warnings · fmt clean") was stale — `cargo fmt --check` exited 1
(48 diff sites across the 9 P1a/P2 files) and `cargo clippy --all-targets` emitted ~50
warnings (41× `clone` on `Copy` `CrudMethod`, 4× `clippy::doc_lazy_continuation`,
3 dead reference-enumerator fns, 2 `clippy::zombie_processes`). Fully resolved and
re-verified: `fmt --check` exit 0 · `clippy --all-targets` 0 warnings · `build` clean ·
`cargo test` **538 pass / 0 fail**. Pure formatting + lint hygiene (justified `#[allow]`
for the spec-citation enumerators and the `ServerGuard`-reaped server children); no
behavioral change. Records: `docs/defects.md` OPEN row + `docs/decisions.md`.

**ARCHIVAL-CLEANUP PASS (2026-09-10):** moved the obsolete session handover
`CHECKPOINT-2026-09-10.md` to a gitignored archival record
`archive/sessions/2026-09-10-crud-unblock-checkpoint.md` (its content was fully absorbed
into the CURRENT WORK + DONE rows and this file is now the sole authoritative handover)
and the transient vitest run-cache `.vite/vitest/results.json` to
`archive/test-data/2026-09-10-vitest-results-cache.json`, adding `.vite/` to `.gitignore`
(vitest cache is not source). No dangling references remain (verified by sweep). All other
candidates (the two dated `2026-09-08-*` research-run dirs, the live-pending batteries) are
**live cited sources and were kept**.

**U1 — CONTRACT AMENDMENT — LANDED (2026-09-16, docs-only) ⇒ THE SPEC GATE IS NOW OPEN (U0 + U1 landed;
U2–U5 still HELD pending the user's next go-ahead):** the P2/F2 **contract amendment** written by the SpecWriter and
driven to **EMPTY** by the spec-gate reviewer loop is **landed** into `docs/specs/p2-gnosis-server.md` (the
`U1 — contract amendment (2026-09-16)` status block + §5.3–§5.9) and `docs/specs/engine-wire-contract.md`
(§4.5–§4.7, §9.1, §16) plus the upstream reconcile ask in `docs/HANDOFF.md` (the addendum's six items +
row 51's query-body half). **The amendment's content:** the `route_bijection`/`P-IM-3` **growth invariant**
with the "**exactly 14 rows**" clause **deleted**; the `POST /rag/query` **request/response contract**
(envelope-strict, camelCase `ragQuery` options, bare-`RagResult` response); the **single mode rule**
(case-insensitive over `flat|graph|vector|hybrid`, absent → Flat, unknown → `ValidationError` → 400 — one rule
shared by POST **and** SSE); the **structured transport decode-error body** (the NEW-2 400/422 body, never 502,
**no §11 row**); the **change cursor** (opaque `seq`, not a revision) + the `GET /changes` feed; the **bounds
discipline** for U4's paged reads; the **subsystem-flag capability semantics** + the V-8 golden restatement;
and the **FS-3 / FS-13/14/15 ruling** (explicit-leg errors → 503, fusion degrades to empty). **The reviewer
loop:** remand 1 returned **4 must-fix + 7 should-fix + 6 notes** (F1–F17), remand 2 returned **1 must-fix +
2 should-fix + 4 notes** (N1–N6) — F1 the `filters` shape had to be the canonical §4.5.2 camelCase shape with
an explicit mapping and **never** a `serde_json::from_value::<QueryAuditFilters>` pass-through; F2 the
`mode`/`expand`/`compression` **token** check lives **only** in the U2 wire resolver (the options are typed
enums, so `validate_rag_options` structurally cannot check them) and `expand`'s 400 is a **new state created by
the amendment**; F3 the reconcile had to name `gnosis.md:872`'s `ragQuery` throw column and FS-13's
`hyde: true` clause (with `mode=hybrid`/`flat` + `hyde:true` degrading, and `flat`/`graph` + `hyde` **inert**);
F4 the SSE mode fail-state had to pin **HTTP 400** + one `event: error` frame body; N1 `mode` is the **only**
token with a POST **and** SSE outcome (`expand`/`compression` are POST-payload-only and `?expand=`/
`?compression=` on `/rag/stream` are **ignored** ⇒ default, never a 400 — the pre-U4 SSE surface reads exactly
`query`/`topK`/`mode`); N2 `expand:5`/`compression:null` joined the wrongly-typed corpus and "only a JSON
string reaches the token check" was pinned; N3 row 51 was marked superseded-by-the-addendum for its
canonical-ask half while its query-body half lives on. **Final verification: `VERDICT: EMPTY — no must-fix
findings; the spec gate may open for U1's dependent units.`** The one non-blocking stale cross-reference it
found (the addendum's "item 5's request/response shape") was corrected by the supervisor in the same pass to
"row 51's part (b)". **Discipline held across the unit:** docs-only; `docs/specs/gnosis.md` (canonical) and
`docs/specs/p1a-document-crud-wire.md` (frozen shapes) **untouched** — the upstream reconciliation is asked via
HANDOFF instead; **zero new property rows** (the p2 §11 register still shows exactly the **7** landed ids
`P-IM-1/2/3`, `P-SM-1/2/3`, `P-TP-1`; the rows the code-bearing units owe are recorded as **id-less owed
sketches** naming their unit); no U2/U3/U4/U5 spec was written and no implementation began. **The next
supervisor's queue is the user's go-ahead for the code-bearing units — U2 (the query POST contract: shared
payload decoder, envelope strictness, mode resolver + unknown-mode rejection on POST *and* SSE, structured
JSON decode-error body) and U3 (status honesty: derive the six subsystem flags from real state; `vector`
false until U5; F2 §9 + V-8 goldens in the same unit)**; **U4 (change cursor + paged reads + `GET /changes`)
and U5 (boot vector-index build) remain HELD, and U4 additionally needs SHELL-2 on the consumer side** (six of
the consumer's `RagStore` members are synchronous). **Verified baseline re-verified against the actual tree:
`cargo test` 538 pass / 0 fail · `cargo fmt --check` exit 0 · `cargo clippy --all-targets` 0 warnings ·
`cargo build` clean.** **The spec gate is now OPEN, so U2 is delegable on the user's word.**

**INBOUND GR-1..GR-9 GATE-1 REVIEW — U0 LANDED, U1 WAS AUTHORIZED IN THE U0 PASS (2026-09-16, docs-only;
record kept as history — U1 has since LANDED, see the U1 block above):** the Astrographer shell's consolidated
engine feature-request set (`GR-1..GR-9`, filed in the consumer repo, with the engine-side inbound note at
the top of `docs/HANDOFF.md`) was reviewed at the **proposal-review gate** — validity ∥ critique →
architecture review → change-analysis, all four passes read-only and `file:line`-grounded — and the review
**PASSED**; its record **landed**: **`docs/specs/gnosis-gr-inbound-review.md`** (including the answered open
questions, **Appendix A** (Q2 change-cursor elaboration) + **Appendix B** (Q3 FS-13/14/15 reconcile
elaboration), each now carrying its own accepted-ruling status footer). Verdict:
**PROCEED-WITH-AMENDMENTS on a re-shaped subset** (U0–U3 as amended, U4/U5 gated,
**GR-6/GR-7/GR-8 parked behind named triggers**, **GR-9 refused as an engine deliverable** — already answered
by the C5/SHELL `GNOSIS-RBAC-EDIT-ENFORCEMENT` ruling: the shell's empty authority mapping fails closed with
an engine-shaped message; the engine has no denial code and discards `caller` after the decode layer).
**The user's instruction "clean the JS files and then proceed" (2026-09-16) closed both halves:** the
**JS cleanup is DONE** (the supervisor `git rm`'d the three remaining stray files — `retrieval.ts` at the
repo root plus the two orphaned vendored TypeScript test files
`tests/unit-a1-crud-routing-proxy.test.ts` / `tests/props-a1-crud-routing-proxy.test.ts` — so the tree is
now **free of TypeScript/JavaScript**: a sweep finds no `.ts`/`.js`/`.mjs`/`.tsx`/`.jsx` file anywhere
outside `target/`/`archive/`/`.cargo-home/`, where `.cargo-home/` is a pre-existing, gitignored, intentional
sandbox-local cargo home and is **not** JS to clean; the Rust baseline is **unchanged: `cargo test` 538
pass / 0 fail**, `fmt --check` exit 0), and **"then proceed" = the gate-1 go-ahead for U1**.
**The two follow-up calls from the previous pass are therefore CLOSED** (both defect rows — `retrieval.ts`
and the orphaned vendored test files — are now **FIXED (2026-09-16)** in `docs/defects.md`, alongside the
four vendored client copies fixed in the first pass), and **Q2/Q3 are ANSWERED**: the user accepted the
recommended rulings as elaborated in Appendices A and B — **Q2 = Option A** (the opaque change cursor; no
consumer-visible store-wide revision, `/snapshot?revision=` and `stale_revision` refused) and **Q3 = Option
A** (explicit-leg errors, fusion degrades: `mode=vector` with no index → `VectorIndexUnavailable` → 503; the
hybrid fusion path keeps degrading a failing leg to empty). Both were **accepted by proceeding, not by an
explicit option-by-option selection**. Consequently the two proposed decisions are now **ACTIVE rows in the
`docs/decisions.md` Gnosis-local table: `GNOSIS-CHANGE-CURSOR` + `SUBSYSTEM-FLAG-CAPABILITY-SEMANTICS`
(2026-09-16)**. **Q4 = YES** — the engine will eventually own a durable corpus, so a **durability DESIGN
unit is on the roadmap** (scope: format / atomic commit / crash recovery / store-scope revision semantics),
**gated and NOT authorized to start** (consumer `SINGLE-WRITER-STORE`/O-8 amendment + a scheduled consumer
unit; see the GR-7 row in `docs/pending.md` + the `ENGINE-DURABLE-CORPUS-DIRECTION` **DIRECTION ONLY /
NOT ACTIVE** line in `docs/decisions.md`); **Q5 = DELETE — landed in two passes** (the four vendored
consumer-client TS copies on 2026-09-16, then `retrieval.ts` + the two orphaned test files in this pass),
all three `docs/defects.md` rows **FIXED**. Verified baseline **re-verified 2026-09-16 against the actual
tree**: `cargo test` **538 pass / 0 fail** · `cargo fmt --check` **exit 0** · `cargo clippy --all-targets`
**0 warnings** · `cargo build` **clean** — the tree also carries an **uncommitted fmt/clippy hygiene fix**
and the HANDOFF inbound note (tree state only). GR-1's user-visible value is entirely **consumer-side**
(**SHELL-1** is the actual P0 on the query path); **U4 delivers no value until SHELL-2** (six of the
consumer's `RagStore` members are synchronous). **The queue at that point was U1's SPEC-GATE work — the
P2/F2 contract amendment** (route bijection as a growth invariant + P-IM-3 restated; the query
request/response bodies; the single case-insensitive mode rule; the structured NEW-2 decode body on the
query surface; the subsystem-flag semantics + the V-8 golden; and the FS-13/14/15 explicit-leg reconcile,
which closes the long-open HANDOFF row). **That work is now DONE** — U1 landed and its spec review came back
**EMPTY** (see the U1 block above), so the **current** queue is the user's go-ahead for **U2/U3**.
In the U0 pass U1 was **authorized** and landed **alone** (contract-only)
so U2/U3/U4 each have authority to code against; **U2 (query POST contract), U3 (status honesty), U4 (change
cursor + paged reads + change feed) and U5 (boot index build) remained HELD pending U1's spec review** — that
spec review is now **EMPTY** and U1 is **LANDED** (see the U1 block above), so the gate they were held behind is
**open** and only the user's next go-ahead is outstanding. U0
parks/rows landed: `docs/pending.md` (the gate-1 PARKED rows + measured triggers — header now reads
"U0 + U1 landed (2026-09-16); U2/U3 AUTHORIZED (registers authored); U4/U5 HELD; spec gate OPEN" — **now updated in place to "U2 LANDED (2026-09-17); U3 LANDED-GREEN (2026-09-17); U4/U5 HELD"**; the GR-7 row re-stated as the user-answered roadmap
direction), `docs/defects.md` (the six hard-coded `EngineSubsystems` flags → `/engine/status` false claims,
**OPEN** — its row, written in the U0 pass, records the pre-U1 state ("U1 authorized / U3 not yet authorized") and
its U1/U3 clauses are **superseded by U1's landing**, though U3 itself is still not authorized; **SUPERSEDED (2026-09-16): U2/U3 ARE AUTHORIZED — their typed registers are authored in `docs/specs/p2-gnosis-server.md` §9.5.1/§9.5.2, so only the U3 *code* is owed; U4/U5 remain NOT authorized. The historical clause read "...though U3 itself is still not authorized".** **RESOLVED (2026-09-17, the U3 documentation pass): that row is now marked `FIXED (U3)` in `docs/defects.md`** — the six flags are derived at read time (`src/store/mod.rs:4193-4232`) and no flag mask is written anywhere in `src/`. **EXTENDED (2026-09-17, the U3 post-greens doc-review): "only the U3 *code* is owed" is discharged too — U3's code LANDED-GREEN, so no U2/U3 code is owed and U4/U5 are the only not-authorized code-bearing units (see the U3 block + DONE row above).** the stale vendored
consumer-client TS copies, **FIXED**; `retrieval.ts` + the two orphaned test files, **FIXED (2026-09-16)**),
`docs/HANDOFF.md` (the gate-1 record pointer, the FS-13/14/15 U1 annotation, the GR-6/GR-7/GR-8
parked-request rows, and the GR-9 cross-reference on the RBAC row — their go-ahead clauses now read
"U0 + U1 landed (2026-09-16); U2/U3 AUTHORIZED (registers authored); U4/U5 HELD; spec gate OPEN" — **now "U2 LANDED (2026-09-17); U3 LANDED-GREEN (2026-09-17); U4/U5 HELD"**), the **two ACTIVE** decision rows
(`GNOSIS-CHANGE-CURSOR` + `SUBSYSTEM-FLAG-CAPABILITY-SEMANTICS`) + the `ENGINE-DURABLE-CORPUS-DIRECTION`
direction-only line in `docs/decisions.md`, and this file's gate-answer recording.

**§4.5-DEFERRED REVISIT (2026-09-09, change-analysis pass):** the four
`ENGINE-INTERNAL DEFERRED` rows previously framed as "revisit with §4.5" were
re-derived against the actual crate now that §4.5 has landed:
**audit-log recording sink — RESOLVED** (landed in §4.5: `rag_query`/`rag_stream`
append real `QueryAuditEntry`s; §4.3.4 returns real entries, pinned by two tests);
**fact-store sharding — KEEP DEFERRED** (re-scoped to a **measured** contention/throughput signal; no §4.5 load path justifies it yet); **fact `node_id` coherence — CLOSED** as a spec-wording tension (`factKey` is the canonical identity; engine will not materialize fact nodes); **cross-field consistency gate — CLOSED** as a documented non-goal (deterministic fail-closed gate; reconcile spec §4.3.2a.1 as aspirational/offline). All recorded in `docs/pending.md` + `docs/defects.md`; the two closures carry spec-reconcile notes in `docs/HANDOFF.md`. None of the four requires new engine work.

**DOC-REVIEW QUICK-PIN PASS (2026-09-09, DONE):** the doc-review gaps that are
implementable/checkable now were pinned by a test pass — `author` round-trip was
already **CLOSED by the PBT retrofit** (`props_store.rs` P-TP-2); **ISO-8601 UTC
format** pinned via `assert_iso8601_utc` in store/facts/graph suites;
**`MultiQueryExpansionFailed` (FS-19) reachability** pinned via an empty-wiki seed
in `rag_query_integration.rs` (+ stream error-chunk mirror). Engine ids not
RFC-4122 UUID v4 remains **deferred to the F2 seam** (an on-the-wire contract
decision, not a test pin). Test count 293 → **295** (see the DONE row).

**PBT-GATE RETROFIT (2026-09-09, decision PBT-GATE-MANDATORY):** the mandatory
property-based-testing gate now applies to every code-bearing unit. Five typed
property registers (`docs/specs/4-*-property-register.md`, P-IM/P-SM/P-TP ≤8 rows
each), five executed property layers (`tests/props_*.rs`, each a deterministic
hand-rolled SplitMix64/Xoshiro PRNG with a pinned seed, ≤100 generated
cases/row, ≤400 total, stop-after-5, HELD/BROKEN + strategy-id), and five read-only
PBT audits landed. Register rows are invariants-only (never §6/FS-n or §7/F-gap
rows), each TRUE of the green implementation. Result: **39 of 40 property rows
HELD**; the gate surfaced and host-fixed a **genuine defect** — `rrf_fuse`'s f64
RRF accumulation was non-associative so exact-tied keys landed 1 ULP apart by
input-list order, breaking §4.5.3 exact-merge determinism (register P-IM-2);
fixed (canonical sorted order of per-key contributions) + regression test in
`tests/retrieval_stack_integration.rs`. Negative-generator probes added from the
audits across all 5 units; the §4.3 register's stale `[PENDING]` tags were
re-tagged GREEN (those defects were already fixed). Two engine-internal
non-goals/behaviours recorded: the §4.4 `publish_document` concurrent-atomicity
TOCTOU (defects.md + HANDOFF.md) and the §4.2 `resolve_entities` residual-alias
behaviour (defects.md + pending.md).

**Scaffold (2026-09-09): the project is scaffolded.** The folder structure, the
canonical spec (`docs/specs/gnosis.md`, copied from the Auspicion Suite), the
relevant research notes (`docs/research/`), the integration-surface docs
(`docs/integrations/`), the Rust crate skeleton (`Cargo.toml`, `src/`), and the
trackers are in place. No implementation code has been written yet.

**Housekeeping (2026-09-09):** `archive/` is now gitignored (decision
ARCHIVE-GITIGNORED) and `Cargo.lock` is committed (decision LOCKFILE-COMMITTED)
so build reproducibility is pinned. First archival-loop pass run — nothing to
archive yet (no obsolete docs or findings reports exist at the scaffold stage);
all citations verified against current files.

**Trio green at scaffold (2026-09-09):** `cargo build`, `cargo test`
(1 scaffold test), `cargo clippy --all-targets`, and `cargo fmt --check` are all
clean after installing the host build prerequisites. **Linux build/dep
prerequisites:** a C linker (`gcc`/`cc`), `pkg-config`, and `libssl-dev` (for
`reqwest`'s native-tls TLS). If a CI/consumer host lacks them, `reqwest`'s TLS
backend can be switched to `rustls` at the cost of a `cmake`-building provider —
currently the native-tls default is kept.

**Data-structure & concurrency plan (2026-09-09):** the pre-implementation design
is `docs/research/gnosis-data-structures-concurrency-plan.md` — answers live-
update/overlay, speed-at-volume under concurrent requests, and Safe-Rust
mutable+shared tension via ownership splitting. Decisions pinned:
SHARDED-RWLOCK-STORE, IMMUTABLE-DERIVED-SNAPSHOT, WRITER-ACTOR-JOURNAL,
LAYERED-OVERLAY, ARC-SHARED-ENGINE (`docs/decisions.md`). The core-unit TestWriter
red sets (§4.1–§4.4) must now exercise the concurrency / optimistic-concurrency /
overlay states that this plan pins.

**§4.1 document store DONE (2026-09-09).** TDD red→green to green with the
adversarial findings fixed; full details in the DONE row below. The next unit to
delegate is **§4.2 knowledge graph** (nodes/edges/properties, subject-relation
model, entity resolution, manual overrides), which attaches to the store's new
journal/epoch feed.

**§4.2 knowledge graph DONE (2026-09-09).** Red→green; the §4.2 adversarial gate
rejected the first green (sidecar second-source-of-truth) and drove a decided
**type evolution** (GRAPH-OWNS-RELATION-AND-MERGE) so triples/merges/resolutions
are graph-owned; fixed + regression-pinned. Details in the DONE row.

**§4.3 fact/citation DONE (2026-09-09).** Red→green; the §4.3 adversarial gate
found dangling-citation / whitespace-value / wiki-consistency defects, all fixed +
regression-pinned. Cross-field consistency declared deferred to §4.5. Details in
the DONE row.

**§4.4 consistency enforcement DONE (2026-09-09).** Red→green; the §4.4
adversarial gate found a HIGH AB-BA deadlock (eliminated by the uniform lock-order
decision LOCK-ORDER-REF-SHARD-SIDECAR) + propagation/report defects, all fixed +
regression-pinned. Details in the DONE row.

**§4.5 RAG/agent-memory retrieval DONE (2026-09-09) — after an adversarial REJECT + rework.**
The initial green was correctly rejected by the §4.5 adversarial gate (vector/hybrid
ran the lexical leg, retrieval-stack fail-states were dead code, several greens
vacuous). A real rework wired the vector/hybrid legs, made the retrieval-stack
fail-states reachable, wiki-scoped the vector leg, fixed deterministic graph-leg
ordering, and de-vacuated the weak tests; a second adversarial re-audit confirmed
the blockers resolved (parked/reserved items documented honestly, not faked). **The
core engine (§4.1–§4.6) is now fully implemented and adversarially hardened.**

**§4.5 RE-AUDIT FIX PASS (2026-09-09):** the §4.5 re-audit left 2 genuine RED tests
+ several documentary items, all now closed:
**MEDIUM-A** (graph-leg determinism — sort walk roots + resolved results by
`(DocumentId, NodeId)` ascending before `top_k`/RRF in `graph_query` + `hybrid_graph_leg`)
**FIXED** → `graph_hybrid_ordering_is_deterministic` GREEN (stable); **LOW-E** (HyDE
embedding errors surface as `EmbeddingUnavailable`) **FIXED**; **LOW-G**
(`binaryCandidatePool` default `10×topK`) **code-resolved** and its spec-conflict test
**corrected** (seed arithmetic made consistent with the default) → `retrieval_stack` 18/18.
`subTaskDag`/`CompressionFailed`/`HyDEGenerationFailed` recorded as **reserved** variants
(decisions SUB-TASK-DAG-VALIDATED-ONLY + RESERVED-ERRVARIANTS-DISCIPLINE), sub-task DAG
parked (`docs/pending.md`), FS-13/14/15 lexical-index tension in `docs/HANDOFF.md`.

**§7.2 F2 engine wire contract DONE (2026-09-09).** Red→green; the last code-unit in
this repo's scope is now implemented + adversarially hardened + trio-green. Contract:
`docs/specs/engine-wire-contract.md` (+ property register
`docs/specs/7-2-wire-property-register.md`), proposal-review record `docs/specs/7-2-f2-review.md`,
decision `F2-WIRE-CONTRACT-A1`. Details in the DONE row. **360 green tests** (baseline 295 +
65 F2: `wire_conformance` 57 [47 conformance + 2 regressions + 8 probes] + `props_wire` 8).

## OPEN

| Unit | Status | Notes |
| --- | --- | --- |
| **`GRQ-1..GRQ-11` — the inbound P2 prerequisite set (the proposal-review gate's outcome)** | **GATE PASSED (PROCEED-WITH-AMENDMENTS, 2026-09-22); GO-AHEAD GIVEN — U5 AUTHORIZED; D-D1 / the GRQ-3 rider / U4 still NOT authorized** | Record: **`docs/specs/gnosis-grq-inbound-review.md`** (§4 verdicts, §7 rulings, §13 the five questions, **§14 POST-RECORD UPDATE 1 = the four answers**, §15 change-analysis). **The fork is answered (A) the engine becomes the durable authority** with the consumer's O-8 amendment as the precondition; **the docs-only asks are accepted** (owner: SpecWriter) and **the §6(c) marker is refused**. **Dispositions:** GRQ-1/2/11 + §6(c) → the durability design unit (scope only, NOT authorized); GRQ-3 → a rider inside it (DERIVED field on `HealthReport`); GRQ-4 → refusal standing (DUPLICATE); GRQ-5/6 → PARKED unchanged; GRQ-7 → already fixed (U2), repro stale; GRQ-8 → documented refusal (accepted); GRQ-9 → docs appendix (accepted) + `RBAC-DOC-DRIFT`; GRQ-10 → U4 (HELD, needs `SHELL-2`). **No new `docs/defects.md` row for any GRQ** (handoffs); **no ACTIVE decision row added**; the canonical `docs/specs/gnosis.md` reconciles go upstream via `docs/HANDOFF.md` once the O-8 amendment lands. |
| **U5 — the boot vector-index build (the only unit the GRQ gate authorized)** | **DONE — LANDED-GREEN + ALL EIGHT GATES RUN (2026-09-22); its DONE row is the first data row of the §DONE table below. SCOPE NOTE (read this before treating U5 as closed): the *unit* is done, but SIX OPEN follow-on rows and one undis-charged test obligation remain (`U5-ADV-1`…`U5-ADV-5`, `P-9`'s fired trigger, and the T1–T10 negative generators) — they are the follow-up unit's and the TestWriter's work, not U5's, and they are deliberately NOT closed by this row.** | Authorized by the user's go-ahead (`docs/specs/gnosis-grq-inbound-review.md` §14 POST-RECORD UPDATE 1, **Q3 = "(B) U5 ONLY"**); contract + typed register §9.5.5 (8 rows, spec gate **EMPTY** at round 5). **Landed:** `build_boot_vector_index` (body `src/store/mod.rs:5369`, name re-exported `src/lib.rs:126`) + the bin's boot wiring (`src/bin/gnosis_server.rs:381-437`: `Reachable` ⇒ build + composed snapshot; failed `Reachable` ⇒ the pinned order with the flags discarded ⇒ `Degraded` + `vector:false` + `embedding:true`) + the comment-only reconciliation at `src/store/mod.rs:4210-4215`; **no new `StoreError` variant, §11 row, route, accessor or frozen-shape change**. **Verified:** `cargo test` **655/0** serial (604 → +24 in-crate → +27 blind), hermetic both ways; property layer **314/355 ≤ 400**; blind set **27/27**; live battery `R-L3` **(i)/(ii)/(iv) PASS live** with (iii)/(v) parked and `R-L2`'s amended cell re-verified; trio clean. **What remains open is NOT U5 work:** the six OPEN rows `U5-ADV-1`…`U5-ADV-5` + `P-9`'s fired trigger (`docs/defects.md`) — a follow-up **"U5 honesty/freshness" unit** is being opened for **`U5-ADV-1`** (the boot index is frozen: a post-boot write makes `mode=vector` a silent-200 with missing hits while `vector` stays `true`; live-confirmed at gate 6) — and the PBT audit's **T1–T10** negative-generator list, which is the **TestWriter's** obligation. The failed-`Reachable`-build branch is **lib-level only (not live-verified)**. **U4 remains HELD** (it needs the consumer-side `SHELL-2`). |
| **_(shell-integration unit + suite-side DeepEval harness + F4-LLM integration + authorship-source code-bearing unit)_** | **shell-integration: RE-SCOPED (2026-09-10, proposal gate — `docs/specs/shell-integration-review.md` in the Astrographer repo); the rest DEFERRED / out of this repo's current scope** | The shell-integration unit's **7-item scope is largely LANDED** (P2 server host + READY lifecycle + §11 rendering; GN SSE client + auth/TLS options; A1/A2 document-CRUD routing; GN-MCP-UI + A2 D4 panes). The proposal gate **re-scoped it to Option B** — the five genuine remaining deliverables, owned by an Astrographer-shell unit: (1) TLS application (a shared `engine-transport.ts` https-agent-backed `fetch`); (2) a fetch-based SSE client (sends the Bearer header, no auto-reconnect); (3) a concrete bind/auth/TLS policy record; (4) the D2-fallback clarification (the document-CRUD surface surfaces `EngineUnavailable`; the local `rag.*` surface is the parallel D2 fallback); (5) the e2e transport test / live-battery revisit. Item 4 (full `RagStore` CRUD routing) is **dropped** — the 11-method A1/A2 routing is landed; the ~45-method surface stays deferred per `GNOSIS-CRUD-MVP-SCOPE`. The **suite-side DeepEval Python harness** (faithfulness/answer-relevancy, answer generation, user-facing retrieval-quality surface) is **NOT Gnosis** — recorded in `docs/specs/6-f6-eval-review.md` + `docs/decisions.md` `F6-EVAL-RE-SCOPED`. The **F4-LLM integration** (a suite tool with a harnessed LLM drives Gnosis's manual-override-authoritative enrichment surfaces) is **NOT a Gnosis-repo code unit** — recorded in `docs/specs/f4-llm-enrichment-integration.md` + `docs/pending.md`. The **authorship-source code-bearing unit** (decision AUTHORSHIP-SOURCE-PROPERTY) is **doc/design-only now** — recorded in `docs/specs/authorship-source-review.md` + `docs/decisions.md`. The **UUID-v4 id decision** is **RESOLVED** as `ID-SCHEME-RECONCILE-MONOTONIC` (ids stay monotonic opaque strings; RFC-4122 not taken up; reconcile request in `docs/HANDOFF.md`). |

## DONE

| Unit | Red set | Green | Adversarial findings | Blind-greens | Doc-review | Trio |
| --- | --- | --- | --- | --- | --- | --- |
| **U5 — the boot vector-index build** (`build_boot_vector_index` — body in `src/store/mod.rs:5369`, name re-exported from `src/lib.rs:126` — + the bin's boot wiring in `src/bin/gnosis_server.rs:381-437` + the comment-only reconciliation at the flag-derivation site `src/store/mod.rs:4210-4215`; spec `docs/specs/p2-gnosis-server.md` §5.8 + the §9.5.5 register; blind-greens `docs/specs/u5-boot-vector-index-greens.md`; live battery `docs/specs/p2-gnosis-server-live-pending-battery.md` §3.5 row `R-L3` + §8) | **The 24-item red set reported before the implementation** (15 in the new `tests/u5_boot_vector_index_conformance.rs` + the eight §9.5.5 register rows' `#[test]`s + `u5_layer_budget_discipline` in `tests/props_gnosis_server.rs`, with **2 test targets compile-failing on the absent `build_boot_vector_index` symbol** at the stubs stage); the spec gate itself was driven to **EMPTY (round 5; rounds 1–4 returned 13 / 11 / 6 / 6 findings)** and the TestWriter's red-phase triage fed back two spec-level items which landed as the **post-red-phase register amendment** (the `355` cap set + the `NaN` comparison rule) | **`cargo test` 655 passed / 0 failed** (serial `-- --test-threads=1`; baseline **604** → **+24 U5 in-crate** → **+27 blind**), green **with and without** `GNOSIS_SERVER_OLLAMA_URL` (hermetic); `cargo fmt --check` exit 0 · `cargo clippy --all-targets` 0 warnings · `cargo build` clean. **All 8 §9.5.5 rows HELD** — `P-IM-10` 55/60 · `P-IM-11` 40/50 · `P-IM-12` 40/40 · `P-IM-13` 42/45 · `P-IM-14` 25/45 · `P-IM-15` 27/30 · `P-SM-7` 45/45 · `P-TP-5` 40/40 = **314 executed / 355 caps ≤ 400** | **NO U5 regression required a host code fix in this pass; five adversarial rows filed OPEN + one pre-existing trigger fired** (gate 4, read-only, incl. the PBT audit): **`U5-ADV-1`** — the boot index is frozen with no rebuild vehicle, so a **post-boot write makes `mode=vector` return 200 with silently missing hits while `vector` still reports `true`** (pre-U5 the same request was a loud FS-14 503) — live-confirmed by gate 6; **`U5-ADV-2`** — the pre-bind build is unbounded **and** un-timed (residual (i)'s stated provider-timeout bound does not exist); **`U5-ADV-3`** — `Degraded` + `embedding:true` + a `lastError` blaming embedding; **`U5-ADV-4`** — the build's `Err` is discarded with no diagnostics; **`U5-ADV-5`** — the lock-`unwrap` panic class now sitting on a pre-bind path; **`P-9`'s TRIGGER FIRED** — a non-finite provider vector reaches `encode_result`'s `expect` on the request path (package/foundation, not a U5 patch). All six are **OPEN** in `docs/defects.md`; a follow-up **"U5 honesty/freshness"** unit is being opened for `U5-ADV-1`. PBT audit verdicts: `P-IM-13`/`P-IM-14`/`P-IM-15`/`P-TP-5` sound; `P-IM-10`/`P-IM-11`/`P-IM-12`/`P-SM-7` under-defended, with a **10-item negative-generator list (T1–T10)** left as a **TestWriter obligation (NOT discharged)** | **27 GREEN / 0 RED / 6 NOT-VERIFIED** (`tests/blind_u5_boot_vector_index_greens.rs`; 2 live-HTTP + 25 lib-level; report `docs/specs/u5-boot-vector-index-greens.md`; the report's four disclosed derivation corrections were re-read against §9.5.5 by gate 8 and reconcile **to the docs**, each being a doc pin rather than an implementation finding; its `628 + 27 = 655` arithmetic matches the tree) | **DONE (gate 8, 2026-09-22)** — reconciled in one pass: `docs/specs/p2-gnosis-server.md` (§U1's new U5 post-greens verification-status block + §12 item 5's owed clause), `docs/specs/engine-wire-contract.md` (§U5 status note + §9.1's `vector` row + the two authorization markers), `docs/specs/p2-gnosis-server-live-pending-battery.md` (§3.5's `R-L3` cells/FAIL set + the `:414` build-call citation + §4's scope note + the new **§7** run/park record), `docs/next-steps.md` (this row + the CURRENT WORK block + the OPEN row), `docs/pending.md` (the header status marker + GR-3's trigger), `docs/HANDOFF.md` (§U4/U5 status clause + a U5 gate-chain block), `docs/defects.md` (the `U5-ADV-*`/`P-9` rows re-verified against the tree — no stale `file:line` found). Record: `archive/reviews/2026-09-22-u5-boot-vector-index-doc-review.md` | `cargo test` **655 pass / 0 fail** (serial) · `build` clean · `clippy --all-targets` 0 warnings · `fmt --check` exit 0 |
| **U3 — status honesty** (`src/store/mod.rs`'s read-time flag derivation in `get_engine_status` + the `src/lib.rs` boot-wiring seam `boot_wiring` + `src/bin/gnosis_server.rs`'s boot that writes **no** flag mask + the amended V-8.1/V-8.2 golden literals; spec `docs/specs/p2-gnosis-server.md` §5.8 + the §9.5.2 register; blind-greens `docs/specs/u3-status-honesty-greens.md`; live battery `docs/specs/p2-gnosis-server-live-pending-battery.md` §3.5 row `R-L2`) | **The 4-item red set reported before the implementation: 3 property rows BROKEN** (§9.5.2 `P-IM-7`/`P-IM-8`/`P-IM-9`, tags `U3PIM7`…`U3PIM9` — against the hard-coded all-`true` construction mask and the boot that never updated it; `P-IM-9` also red against the TestWriter's deliberately non-conforming `gnosis::boot_wiring` stub, the pinned seam compiling but deriving nothing) **+ the live-status e2e** (`tests/gnosis_server_e2e.rs::engine_status_reports_no_false_embedding_claim`, then RED because a provider-absent boot still reported `embedding:true`); `P-SM-5`/`P-SM-6` held from the start as the unit's blast-radius guards | **`cargo test` 604 passed / 0 failed** (serial `-- --test-threads=1`; pre-U3 baseline 582 — U3's contribution is **nine in-crate tests**: the five executed U3 property rows + `u3_layer_budget_discipline` (`props_gnosis_server` 15→21), `boot_wiring_couples_to_the_derived_read` (`wire_conformance` 58→59) and `engine_status_reports_no_false_embedding_claim` (`gnosis_server_e2e` 27→28) — **plus the 13-scenario blind set** as a separate verification layer), green **both with and without** `GNOSIS_SERVER_OLLAMA_URL` set (the suite is hermetic; the P2 blind set's ambient-provider non-hermeticity was fixed in the same unit — `s9_rag_query_not_ready_503` now spawns provider-free); `cargo fmt --check` exit 0 · `cargo clippy --all-targets` 0 warnings · `cargo build` clean. **All five §9.5.2 rows HELD** (`U3PIM7`…`U3PSM6`) with the layer's **127 executed cases against the ≤400 per-unit cap** (`P-IM-7` 25 / `P-IM-8` 26 / `P-IM-9` 26 / `P-SM-5` 25 / `P-SM-6` 25 = 127, `tests/props_gnosis_server.rs:59-76` + `:278-295`; arithmetic pinned by `u3_layer_budget_discipline`, `:5689`); the boot-wiring seam is landed at `src/lib.rs:69-110` and the derivation at `src/store/mod.rs:4193-4232`. **PBT = the typed register §9.5.2 authored (5 rows; twelve injective tags over §9.5 = 7 + 5, disjoint from the landed `PIM1`…`PTP1` set) + the executed property layer + the read-only PBT audit** | **The 6 host/test must-fixes (all fixed in the unit) — H1 the inert-hook documentation, H2 a test passing for the wrong reason, H4 the non-hermetic e2e, H5/H6 the count/guard hygiene — + the 7 added negative generators** (incl. the `Some(empty VectorIndex)` boundary, the inverted reachability probe, the mutation-interleaved purity control and the `boot_wiring`↔derived-read coupling assertion `boot_wiring_couples_to_the_derived_read`, `tests/wire_conformance.rs:1129`) **+ the package/defect rows filed** (`P-8` post-boot provider loss is invisible to `/engine/status`; `P-9` `encode_result`'s `expect` panic path — both **OPEN**, neither a U3 regression, cross-filed in `docs/HANDOFF.md` §"U3 documentation pass (2026-09-17)") | **13/13 PASS** — 3 live-HTTP (`U3-1`, `U3-2`, `U3-12`) + 10 pure-lib (`U3-3`…`U3-11`, `U3-13`) scenarios (`tests/blind_u3_status_honesty_greens.rs`; the set's `U3-13` asserts the U3-time V-8.1-stage and V-8.2 vectors field by field); **plus the P2 blind set's hermeticity fix** (its `s9_rag_query_not_ready_503` row now spawns with `spawn_server_without_provider()`) | **RECONCILED 2026-09-17 (gate 8)** — the U3-bearing sections of `docs/specs/p2-gnosis-server.md` (§5.8, §6, §9.5.2/§9.5.3/§9.5.4, §10–§12, §U1), `docs/specs/engine-wire-contract.md` (§9/§9.1, §12 V-8/V-8.1/V-8.2, §13/§14) and `docs/specs/u3-status-honesty-greens.md` were checked against the landed tree, and the trackers (`docs/next-steps.md`, `docs/decisions.md`, `docs/defects.md` `P-1`…`P-9`, `docs/pending.md`, `docs/HANDOFF.md`, `docs/specs/gnosis-gr-inbound-review.md`, `docs/specs/p2-gnosis-server-live-pending-battery.md` §3.5/§4) re-pointed where they had drifted; findings fixed in place (notably the `EngineSubsystems` literal ⇒ `:1718-1725` and the stale renderer citations ⇒ `:62-75`/`:100-115`). Review record: `archive/reviews/2026-09-17-u3-status-honesty-doc-review.md` (gitignored provenance) | **`cargo test` 604 passed / 0 failed** (serial) · `cargo fmt --check` exit 0 · `cargo clippy --all-targets` 0 warnings · `cargo build` clean — green **with and without** `GNOSIS_SERVER_OLLAMA_URL`; **U3's property layer 127 executed ≤ 400** (per-unit cap) and the binary's total `310 + 400 + 127` is the binary's state, not a capped quantity |
| **U2 — the query POST contract** (`src/wire/query.rs` + the `src/wire/mod.rs`/`src/lib.rs` re-exports + `src/bin/gnosis_server.rs`'s shared JSON decode-error renderer, SSE pre-stream `error` frame and `encode_result_checked` POST handler; spec `docs/specs/p2-gnosis-server.md` §5.3–§5.5 + the §9.5.1 register; blind-greens `docs/specs/u2-query-post-contract-greens.md`; live battery `docs/specs/p2-gnosis-server-live-pending-battery.md` §3.5) | **The 14-item red set reported by the TestWriter before the implementation: 7 property rows** (§9.5.1 `P-IM-4`/`P-IM-5`/`P-IM-6`/`P-SM-4`/`P-TP-2`/`P-TP-3`/`P-TP-4`, tags `U2PIM4`…`U2PTP4`) **+ 6 e2e** (`tests/gnosis_server_e2e.rs`) **+ 1 golden** (`v15_request_decode_error_body_exact`, `tests/wire_conformance.rs:1385`) | **`cargo test` 582 passed / 0 failed** (pre-U2 baseline 538; U2's contribution: the 7 property rows + `u2_layer_budget_discipline` + 11 e2e (`gnosis_server_e2e` 16→27) + the V-15/V-15.1 golden (`wire_conformance` 58) + the 24-scenario blind set; `props_gnosis_server` 7→15). **All 7 rows HELD** (`U2PIM4`…`U2PTP4`) with the layer's **400 executed cases against the ≤400 per-unit cap** (`48/100/70/80/23/41/38`, `tests/props_gnosis_server.rs:26-40`, `:209-215`; arithmetic pinned by `u2_layer_budget_discipline`, `:4034-4069`; the `p2` layer stays 310 of its own cap). **PBT = the typed register §9.5.1 authored (7 rows; twelve injective tags over §9.5) + the executed property layer + the read-only PBT audit** | **The 4 host must-fixes (all fixed in the unit) + the blind-greens-driven rule fix** (the object-valued-option member-total rule — `object_option` generalized to every declared member, `src/wire/query.rs:188-202`, closing `P-6`) **+ the proofread divergences** (`P-6`/`P-7` CLOSED; `P-5` OPEN) **+ the package rows filed** (`P-1`/`P-2`/`P-3` OPEN in `docs/defects.md`, cross-filed as this repo's own deferred work in `docs/HANDOFF.md` §"U2 adversarial pass") | **24/24 PASS** — 13 live-HTTP + 11 pure-lib scenarios (`tests/blind_u2_query_post_greens.rs`), **including the `U2-13` catch** (an object with a wrongly-typed documented member decoding to a *present* object with decoder-supplied member defaults), whose fix generalized the rule to every declared member | **THIS PASS — post-greens doc-review (2026-09-16, docs-only):** reconciled `p2` §5.2–§5.5/§9.5/§10–§12, F2 §4.4/§4.5/§7.1/§12/§13/§14, the greens set and all five trackers (stale `file:line` citations, the remaining "code lands in U2"/"U2-time … today" markers now LANDED, the non-object-`payload` row to `invalid_envelope` alone, the greens' layer statement); record `archive/reviews/2026-09-16-u2-query-post-contract-doc-review.md`. **LIVE: 7/8 battery rows PASS live, incl. `R-L1`** on a READY boot with a real provider (Ollama `embeddinggemma`) — the wire `topK`/`filters` visibly changed the response and an unrecognized `mode` returned **400 `validation_error`**; **`R-L2` is U3's** provider-reachable boot half and is **out of U2's scope** | **`cargo test` 582 pass / 0 fail** · `cargo fmt --check` exit 0 · `cargo clippy --all-targets` 0 warnings · `cargo build` clean |
| **U1 — the inbound GR-1..GR-9 contract amendment** (docs-only: `docs/specs/p2-gnosis-server.md` + `docs/specs/engine-wire-contract.md` + the `docs/HANDOFF.md` upstream reconcile ask) | **N/A (contract-only) — no test set owed.** No TestWriter red set exists or is owed: U1 writes contracts, not behavior. | **N/A** — the verified baseline is **unchanged: `cargo test` 538 pass / 0 fail** (no code changed; re-verified against the actual tree). **PBT = ZERO ROWS — explicit justified exemption** (contract-only unit: it **restates** the existing `P-IM-3` row in place and **adds none**; the register rows the code-bearing units owe are recorded as **id-less owed sketches** naming U2/U3/U4/U5, per decision `PBT-GATE-MANDATORY`). | **The spec-gate reviewer loop, driven to EMPTY:** remand 1 returned **4 must-fix + 7 should-fix + 6 notes** (F1–F17), remand 2 returned **1 must-fix + 2 should-fix + 4 notes** (N1–N6) — **all fixed and re-verified; final verification verdict `EMPTY — no must-fix findings; the spec gate may open for U1's dependent units`** (the single non-blocking stale cross-reference it found — the HANDOFF addendum's "item 5's request/response shape" → "row 51's part (b)" — was corrected by the supervisor in the same pass) | **N/A (contract-only; the blind-greens layer belongs to the code-bearing units)** | **The three amended artifacts** (`docs/specs/p2-gnosis-server.md`, `docs/specs/engine-wire-contract.md`, `docs/HANDOFF.md` upstream ask) **+ the U1 REMAND / REMAND-2 status lines**; the record's header/status lines, §Go-ahead record and the two appendix status footers were reconciled by the supervisor to the landed state. | **N/A (docs-only, no trio owed)** — **baseline unchanged: 538 pass / 0 fail · `fmt --check` exit 0 · clippy 0 warnings · build clean** |
| **U0 — inbound GR-1..GR-9 gate-1 landing** (docs-only: `docs/specs/gnosis-gr-inbound-review.md` new + `docs/pending.md`, `docs/defects.md`, `docs/HANDOFF.md`, `docs/decisions.md`, this file) | **ZERO PBT ROWS — explicit justified exemption:** doc-only/tracker-only unit, no code and no behavior contract (per decision `PBT-GATE-MANDATORY`, which permits a recorded zero-row exemption for doc/config-only units). No TestWriter red set owed. | **N/A (docs-only)** — the verified baseline is unchanged: `cargo test` **538 pass / 0 fail** (re-verified 2026-09-16 against the actual tree, including the uncommitted fmt/clippy hygiene fix). | N/A (no code unit; the four gate-1 passes — validity ∥ critique → architecture → change-analysis — were read-only and `file:line`-grounded, and their findings are recorded as verdicts/parks/defect rows rather than code findings) | N/A (no UI scenario; the gate record's `file:line` citations are the verification layer) | **DONE (first pass):** gate-1 record (`docs/specs/gnosis-gr-inbound-review.md`); `docs/pending.md` gate-1 PARKED rows + triggers; `docs/defects.md` two OPEN rows (six hard-coded `EngineSubsystems` flags; four stale vendored TS client copies); `docs/HANDOFF.md` gate-1 pointer + FS-13/14/15 U1 annotation + GR-6/GR-7/GR-8 parked-request rows + GR-9 cross-reference; `docs/decisions.md` proposed-pending note only (no ACTIVE rows); this CURRENT WORK + DONE row. **DONE (second pass — the gate-answer recording, 2026-09-16):** the review record's **open-question answers** (Q1 U0 only / hold; Q2 + Q3 **elaboration requested — still pending**; Q4 **YES — durability design unit scheduled, not authorized**; Q5 **DELETE — landed**) + the corrected header verdict/status lines + the corrected vendored-copy claim in §Validity findings + the rewritten §Go-ahead record (no code unit may start until U1 is authorized; U1 additionally requires Q2 + Q3) + **Appendix A** (the Q2 change-cursor elaboration) + **Appendix B** (the Q3 FS-13/14/15 reconcile elaboration) appended. **DONE (third pass — the "clean the JS files and then proceed" landing, 2026-09-16, docs-only):** the supervisor's **`git rm` cleanup of the three remaining stray JS/TS files** (`retrieval.ts` at the repo root + the two orphaned vendored test files `tests/unit-a1-crud-routing-proxy.test.ts` / `tests/props-a1-crud-routing-proxy.test.ts`; a sweep then found **no `.ts`/`.js`/`.mjs`/`.tsx`/`.jsx` file** anywhere outside `target/`/`archive/`/`.cargo-home/`, and the baseline stayed **538/0** with `fmt`/`clippy`/`build` clean); the review record's **header verdict/status lines** (review PASSED; go-ahead = **U0 landed + U1 authorized**; the two decisions now **ACTIVE**; U2–U5 held), **§Open questions** (Q1 superseded by "proceed"; Q2 = **Option A accepted**; Q3 = **Option A accepted**; Q4 unchanged; Q5 = **delete, two passes**), the fully-cleaned **vendored-copy claim** in §Validity findings, the rewritten **§Go-ahead record** (U1 authorized + may proceed to the spec gate; U2–U5 held), the **accepted-ruling status footers on Appendix A + Appendix B**, and the **CLOSED contract-drift residual-risk row**; **two new ACTIVE decision rows** in `docs/decisions.md` (`GNOSIS-CHANGE-CURSOR` + `SUBSYSTEM-FLAG-CAPABILITY-SEMANTICS`, in the ACTIVE table — the `ENGINE-DURABLE-CORPUS-DIRECTION` direction-only line retained alongside them); `docs/defects.md` (the `retrieval.ts` row **and** the orphaned-vendored-test-files row moved to **FIXED**, kept as two separate rows, with the pre-existing vendored-copies FIXED row intact; the status-honesty OPEN row annotated **U1 authorized / U3 not yet**); `docs/pending.md` (the gate-1 PARKED section header → "go-ahead = U0 landed + U1 authorized; U2–U5 HELD", every park row + trigger unchanged); `docs/HANDOFF.md` (the GR-6/GR-7/GR-8 parked-request rows + the FS-13/14/15 row go-ahead clause → "go-ahead = U0 landed + U1 authorized; U2–U5 HELD"); and this CURRENT WORK block. **Docs/tracker-only — no code, no cargo run, no pinned-contract edit; U1 is the next unit and U2–U5 stay HELD.** **Appendix A** (the Q2 change-cursor elaboration) + **Appendix B** (the Q3 FS-13/14/15 reconcile elaboration) appended; `docs/defects.md` vendored-copies row **CLOSED as FIXED (2026-09-16)** + **one new OPEN row** for the fifth stray copy `retrieval.ts`; `docs/pending.md` GR-7 row re-stated as the user-answered roadmap direction (durability **design** unit on the roadmap, not authorized to start); `docs/decisions.md` `ENGINE-DURABLE-CORPUS-DIRECTION` **DIRECTION ONLY / NOT ACTIVE** line; this CURRENT WORK update. **Landed outside the docs pass (performed by the supervisor):** the **`git rm` deletion of the four vendored consumer-client copies** (`engine-rag-store.ts`, `engine-crud-rag-store.ts`, `src/main/engine-rag-store.ts`, `src/main/engine-crud-rag-store.ts` + the then-empty `src/main/`) — no `src/`/`tests/`/cargo change came from this U0 pass | **N/A (docs-only, no trio owed) — baseline unchanged 538/0** — baseline re-verified 2026-09-16: `cargo test` **538 pass / 0 fail** · `fmt --check` exit 0 · `clippy --all-targets` 0 warnings · `build` clean |
| **§7.2 P1a document-CRUD wire contract** (`src/wire/crud.rs` + `DecodeError::UnknownMethod` in `src/wire/decode.rs` + re-export surface in `src/lib.rs`; spec `docs/specs/p1a-document-crud-wire.md`, PBT register §5.x in the spec, blind-greens `docs/specs/p1a-document-crud-wire-greens.md`, live-pending battery `docs/specs/p1a-document-crud-wire-live-pending-battery.md`) | TestWriter red: **missing `src/wire/crud.rs`** (the 11 `CrudMethod` variants + `CrudRequestArgs`/`CrudResult`/`CrudResponseError`/`CrudValidationFailure` + the codecs + `ENGINE_ENDPOINTS`/`ENDPOINT_*`), **`DecodeError::UnknownMethod`** (new variant in `src/wire/decode.rs`), and the re-export surface in `src/lib.rs` | **64 P1a tests** (35 conformance in `tests/crud_wire_conformance.rs` + 8 property rows in `tests/props_crud_wire.rs` + 21 blind-greens in `tests/blind_p1a_crud_wire_greens.rs`); **full suite 454 pass / 0 fail** (baseline 411 + 43: conformance 35 + props 8; the 21 blind-greens are a separate verification layer) | Adversarial gate: **4 HOST-MINOR findings, all fixed + regression-tested; no PACKAGE** (host findings fixed in the unit's own `tests/`; no defect row warranted) | **19/19 scenarios PASS** (`docs/specs/p1a-document-crud-wire-greens.md` S1–S19, executed via `tests/blind_p1a_crud_wire_greens.rs` 21 `#[test]` = S1..S19 + 1 extra S8 endpoint-constant assertion + 1 supporting envelope-constant assertion) | **LIVE-SCENARIO PENDING** (parked on **A1** — the Astrographer CRUD client did not exist yet; the `gnosis-server` bin (P2) now exists and serves the CRUD endpoints, so the P1a battery resumes once A1 lands; battery `docs/specs/p1a-document-crud-wire-live-pending-battery.md` L1–L19 written); **A1 LANDED 2026-09-10** — the battery un-parks on A1 + a running app (see the CURRENT WORK block); doc-review reconciled spec/greens/battery + trackers | `cargo test` **454 pass, 0 fail** · `build` clean · `clippy` 0 warnings · `fmt` clean |
| **§7.2 P2 `gnosis-server` binary crate** (`src/bin/gnosis_server.rs` + `src/server.rs` + re-export surface in `src/lib.rs` + `[[bin]]`/axum in `Cargo.toml`; spec `docs/specs/p2-gnosis-server.md`, PBT register §5.x in the spec, blind-greens `docs/specs/p2-gnosis-server-greens.md`, live-pending battery `docs/specs/p2-gnosis-server-live-pending-battery.md`) | TestWriter red: **missing `src/bin/gnosis_server.rs`** (the loopback bind, the 14 REST/SSE endpoints, the READY boot lifecycle, the §11 status rendering + NEW-2 request-decode outcome, the RBAC `caller` threading), **`src/server.rs`** (the pure `server_status`/`request_decode_status`/`route_bijection` fns), and the re-export surface in `src/lib.rs` | **63 P2 tests** (25 conformance in `tests/gnosis_server_conformance.rs` + 7 property rows in `tests/props_gnosis_server.rs` + 16 e2e in `tests/gnosis_server_e2e.rs` (**27 after U2's live cases landed, 2026-09-17**) + 15 blind-greens in `tests/blind_p2_gnosis_server_greens.rs`); **full suite 523 pass / 0 fail** (baseline 454 + 48: conformance 25 + props 7 + e2e 16; the 15 blind-greens are a separate verification layer); **all 7 property rows HELD** (P-IM-1..3, P-SM-1..3, P-TP-1) | Adversarial gate: **2 HOST-MAJOR + 4 HOST-MINOR findings, all fixed + regression-tested; 2 PACKAGE findings recorded** in `docs/defects.md` + `docs/HANDOFF.md` | **15/15 scenarios PASS** (`docs/specs/p2-gnosis-server-greens.md` S1–S15, executed via `tests/blind_p2_gnosis_server_greens.rs` 15 `#[test]`) | **LIVE-SCENARIO PARTIAL** — **15/15 server-endpoint scenarios PASS live** (S1–S15 against the running `gnosis-server` bin + the compiled blind-greens 15/15 + e2e **27/27** binaries — 16/16 at the P2 landing); the **20 MCP/UI parity scenarios M1–M20 are PARKED on A1** (the Astrographer CRUD routing client did not exist yet; battery `docs/specs/p2-gnosis-server-live-pending-battery.md` M1–M20 written); **A1 LANDED 2026-09-10** — the battery un-parks on A1 + a running app (see the CURRENT WORK block); doc-review reconciled spec/greens/battery + trackers | `cargo test` **523 pass, 0 fail** · `build` clean · `clippy` 0 warnings · `fmt` clean |
| **§7.5 F4 community retrieval** (`src/store/mod.rs` `get_community_context` + `CommunityContext`; spec `docs/specs/f4-community-context-spec.md`, PBT register `docs/specs/4-2-graph-property-register.md` §7.5 F4 section, proposal-review `docs/specs/f4-community-summaries-review.md`) | TestWriter red: **19** (13 happy/fail in `tests/community_context_integration.rs` — `h1_declared_community_returns_matching_context_fresh`, `h2_update_document_rewriting_member_marks_stale`, `h3_re_derive_clears_stale_keeps_summary_members`, `h4_update_community_summary_reflects_new_manual_summary`, `h5_members_spanning_two_documents`, `h6_single_member_community`, `h7_member_that_is_a_fact_location`, `h8_never_triggered_reports_fresh`, `h9_two_consecutive_reads_are_equal`, `h10_membership_completeness`, `h11_manual_summary_survives_member_change_and_re_derive`, `f1_unknown_community_is_community_not_found`, `f2_wiki_not_found_cannot_fire` — + 6 PBT rows in `tests/props_community_context.rs` — `p_im1_context_determinism`, `p_im2_membership_completeness`, `p_sm1_read_side_effect_free`, `p_sm2_state_reflects_staleness`, `p_tp1_manual_summary_authority`, `p_tp2_faithful_projection`) | **411/411 total** (baseline 387 + 24 F4: `community_context_integration.rs` 15 [13 happy/fail + 2 negative probes `neg4_unknown_community_not_found_with_others_present`, `neg5_ghost_wiki_never_wiki_not_found`] + `props_community_context.rs` 9 [6 PBT rows + 3 negative probes `neg1_fact_incorporation_isolation`, `neg2_member_node_removal_stays_fresh`, `neg3_stale_fresh_stale_cycle`]) | Adversarial gate: **2 host findings, all fixed** — **HOST-1** the non-member-touch probe had an **id-collision** (`fresh_doc` built a NEW store whose first doc is `doc-0` — the same id as the outer `doc_a` — so the old probe rewrote the MEMBER's document, not a separate non-member doc) → fixed by creating a genuinely non-member doc on the SAME store + a regression assert; **P-SM-1** the **revisions leg** was hardened (the register's side-effect-free observable now also covers "no document revision bump" — LOW-1). **5 negative probes** added (`neg1`/`neg2`/`neg3` in props + `neg4`/`neg5` in integration). **PBT audit: 6/6 rows HELD** (P-IM-1, P-IM-2, P-SM-1, P-SM-2, P-TP-1, P-TP-2) | **33 GREEN / 0 RED / 2 NOT-VERIFIED** (`docs/greens/4-2-community-context-greens.md`: signature/return-shape, H-1..H-11, F-1/F-2, boundary shapes, addTriple non-trigger, 6 PBT rows; NOT-VERIFIED = the two unconstructible empty-set cases — empty member set + empty summary, both rejected by `declare_community` with `ValidationError`) | doc-review reconciled `docs/next-steps.md` (F4 DONE, 411), `docs/pending.md` (F4 LANDED), `docs/decisions.md` (F4-COMMUNITY-RETRIEVAL LANDED), `docs/defects.md` (no new defect row — adversarial findings were host-fixed), `docs/specs/f4-community-context-spec.md` (status + line refs), `docs/specs/4-2-graph-property-register.md` (line refs), `docs/specs/f4-community-summaries-review.md` (status) | `cargo test` **411 pass, 0 fail** · `build` clean · `clippy` 0 warnings · `fmt` clean |
| **§7.8 F6 RAG evaluation harness** (`src/retrieval/eval.rs` + `src/bin/gnosis_eval.rs`; spec `docs/specs/6-f6-eval-spec.md`, property register `docs/specs/6-f6-eval-property-register.md`, proposal-review `docs/specs/6-f6-eval-review.md`) | TestWriter red: **17** (9 red-set in `tests/eval_harness.rs` — `contextual_precision_reference_values`, `contextual_recall_reference_values`, `ndcg_at_k_reference_values`, `mrr_at_k_reference_values`, `empty_set_conventions`, `k_boundaries`, `duplicate_key_handling`, `no_panic_no_nan_bounded`, `corpus_parsing_and_metric_path` — + 8 property rows in `tests/props_eval.rs`); `query_mode_serde_pascal_case` was already-green (PascalCase serde, no stub) | **387/387 total** (baseline 368 + 19 F6: `eval_harness.rs` 11 [9 red-set + `query_mode_serde_pascal_case` + adversarial-fix bin smoke `bin_smoke_report_shape`] + `props_eval.rs` 8 [*p_im_1_metric_bounds*, *p_im_2_metric_repeat*, *p_im_3_empty_results*, *p_im_4_empty_relevant*, *p_sm_1_rank_improve*, *p_sm_2_k_monotone*, *p_tp_1_perfect_retrieval*, *p_tp_2_no_relevant*]) | Adversarial gate: **4 host findings, all fixed** — (1) the `--live` path was a **fake no-op** → replaced with a **real Ollama `LiveProvider`** (`reqwest` to `/api/embed`, `GNOSIS_EVAL_OLLAMA_URL`/`GNOSIS_EVAL_OLLAMA_MODEL`); (2) `map_mode` **panicked** on an unknown mode → now returns `Result` + graceful non-zero exit; (3) latent **`wellformed_graph` empty-nodes panic** → guarded (returns a well-formed empty graph); (4) the §10 **bin smoke test was missing** → added `bin_smoke_report_shape` (runs the built bin against the fixture, asserts the §7.3 report shape). **PBT audit: 8/8 rows HELD** (P-IM-1..4, P-SM-1..2, P-TP-1..2) | **35 GREEN / 0 RED / 3 NOT-VERIFIED** (`docs/greens/6-f6-eval-greens.md`: metric happy/fail-states, partial rankings, boundaries, empty-set conventions, bin report shape, mode-serde note; NOT-VERIFIED = `--live` real-provider path, per-case `rag_query` error branch, non-zero exit on hard failure) | doc-review reconciled `docs/next-steps.md` (387, F6 DONE, F4 next), `docs/pending.md` (F6 LANDED), `docs/decisions.md` (F6-EVAL-RE-SCOPED LANDED), spec + review status headers; **no new defect row** (adversarial findings host-fixed, not defects) | `cargo test` **387 pass, 0 fail** · `build` clean · `clippy` 0 warnings · `fmt` clean |
| **§4.2 `resolve_entities` authoritative-overwrite** (decision RESOLVE-ENTITIES-AUTHORITATIVE-OVERWRITE; unparked from `docs/pending.md` §ENGINE-INTERNAL DEFERRED) | TestWriter red: **3 red** (updated `p_tp2_resolve_entities_overlap_and_convergence` to assert convergence + new `p_tp2_resolve_entities_overlap_flip_no_residual` + `p_tp2_resolve_entities_chain_flatten`) vs the additive impl | **368/368 total** (baseline 360 + 8: 3 red-set tests + 6 adversarial probes − 1 updated overlap test; `props_graph.rs` 15→21) | Adversarial gate: **no host defects** (flatten algorithm correct — acyclic + flat + full convergence incl. chains/flips; contract/concurrency/regression clean). **PBT audit P-TP-2 HELD**; tightened the register observable (multiple roots legal, scoped to requested set + path-compressed aliases) + **6 negative probes** (`p_tp2_resolve_entities_three_hop_chain_flatten`, `_flip_root_with_aliases`, `_duplicate_ids`, `_multiple_roots_acyclic`, `_canonical_was_alias`, `_reaffirm_noop_idempotent`) | **7/7 GREEN, 0 RED** (`docs/greens/4-2-resolve-entities-greens.md`: basic, idempotence, overlap-flip, chain-flatten, convergence-back, fail-states, return shape) | doc-review reconciled `docs/pending.md` (RESOLVED), `docs/defects.md` (FIXED), `docs/decisions.md` (decision row), `docs/specs/4-2-graph-property-register.md` P-TP-2 | `cargo test` **368 pass, 0 fail** · `build` clean · `clippy` 0 warnings · `fmt` clean |
| **§7.2 F2 engine wire contract** (`src/wire/`; spec §4.6.1/§5.1 seam; contract `docs/specs/engine-wire-contract.md`, property register `docs/specs/7-2-wire-property-register.md`) | TestWriter red: **55** (47 conformance in `tests/wire_conformance.rs` + 8 property rows in `tests/props_wire.rs`), then the adversarial fix pass added **2 regression tests + 8 probes** → conformance 57; props wire 8/8 **HELD** | **360/360 total** (baseline 295 + 65 F2: `wire_conformance.rs` 57 [47 conformance + 2 regressions *`envelope_from_json_rejects_oversized_and_negative_schema_version`*, *`decode_done_with_extra_keys_is_malformed`* + 8 `probe_*` adversarial probes] + `props_wire.rs` 8 [*p_im_1_chunk_roundtrip*, *p_im_2_result_bijective*, *p_im_3_code_unique*, *p_im_4_sse_roundtrip*, *p_sm_1_envelope_stable*, *p_sm_2_validation_msg*, *p_sm_3_health_determinism*, *p_tp_1_encode_validates*]) | Adversarial gate on the wire unit: **A1** done-chunk exactness (`decode_chunk_payload` accepts `RagChunk::Done` only for exactly `{"type":"done"}` — extra keys malformed, regression *`decode_done_with_extra_keys_is_malformed`*); **A2** `schemaVersion` truncation (`from_json` rejects an out-of-`u32`-range `schemaVersion` rather than wrapping it — regression *`envelope_from_json_rejects_oversized_and_negative_schema_version`*); **A4** `code_table` display doc (the `ValidationError` row carries the fixed §5 label `"validation failed"`, never the dynamic wire `message` — documented in `src/wire/error.rs`; no code defect); plus PBT test fixes (P-IM-4/A3 SSE framing, P-IM-3 code-uniqueness) and **8 `probe_*`** adversarial probes (*probe_blocked_by_without_graph_across_flat_and_hybrid*, *probe_trace_mode_mismatch_is_transparent_not_a_failure*, *probe_unknown_schema_version_and_id_format_are_errors*, *probe_health_faithful_on_contradictory_input*, *probe_sse_event_data_mismatch_all_ordered_pairs*, *probe_error_codec_missing_message_and_unknown_code*, *probe_validation_error_message_with_code_message_data_survives*, *probe_done_chunk_strictness_via_sse_path*). **8/8 `props_wire` PBT rows HELD.** | **50 GREEN / 1 RED / 3 NOT-VERIFIED** (`docs/greens/7-2-wire-greens.md`). The single RED — `decode_rag_result` precedence for a body that is *both* malformed and trace-less (`{"query":123}` → `MissingTrace`, not `InvalidJson`) — was **resolved as a contract-precedence DOC-fix, not a code defect**: the contract now pins the precedence in §7 + §12 V-9 (trace-`key`-presence is checked **first**, so a malformed-and-traceless body stays FS-10 `TraceUnavailable`; both codes map to HTTP 502 so rendered status is identical), and `decode_rag_result` (`src/wire/decode.rs`) is verified to check trace-presence before structural deserialization. The empty-object `{}`-payload `UnknownType` nuance was folded into the same reconciliation (GREEN-with-note; net `EngineError` outcome unchanged). | This documentation-review pass reconciled the F2 spec + greens + trackers against the landed crate; **360** is the statically-enumerated `#[test]`/`#[tokio::test]` total (see `archive/reviews/2026-09-09-f2-wire-doc-review.md`). `docs/decisions.md` `F2-WIRE-CONTRACT-A1` confirmed accurate + landed; `docs/pending.md` F2 row updated to reflect the landed wire contract + remaining transport deferral; the blind-greens RED row annotated as resolved; no code defect row added (resolved as doc-fix). | `cargo test` **360 pass, 0 fail** (store 50 + graph 66 + facts 36 + consistency 22 + rag_query 36 + retrieval_stack 19 + agent_memory 6 + `integration.rs` `scaffold_compiles` 1 + props store 12/graph 13/facts 14/consistency 12/retrieval 8/wire 8 + wire_conformance 57) · `build` clean · `clippy` 0 warnings · `fmt` clean |
| **DOC-REVIEW QUICK-PIN pass** (2 open pins from `docs/pending.md` §DOC-REVIEW GAPS) | TestWriter red: **PIN B `MultiQueryExpansionFailed` (FS-19) reachable** — empty-wiki + `multiQuery:{enabled,n:2}` seed genuinely triggers it (RED→green, no seam); **PIN A ISO-8601 UTC format** — a new format assertion, RED→green (existing tests only asserted non-empty) | **295/295** (baseline 293 + 2: `rag_query_multi_query_empty_wiki_is_expansion_failed`, `rag_stream_multi_query_empty_wiki_emits_error_chunk`); `assert_iso8601_utc` wired into store/facts/graph create/update/merge assertions | N/A (correctness pins against already-green engine; no new behavior, no adversarial findings) | N/A (no UI scenario; unit-level test pins) | proofread + reconciled `docs/pending.md` (PIN A/B closed, `author` closed by prior PBT retrofit), `docs/next-steps.md` (293→295) | `cargo test` **295 pass, 0 fail** · `build` clean · `clippy` 0 warnings · `fmt` clean |
| **PBT-gate retrofit** (all code-bearing units, decision PBT-GATE-MANDATORY) | TestWriter red: §4.1–§4.4 + §4.5 property layers all **HELD** (40 rows); **§4.5 P-IM-2 genuinely BROKEN** (`rrf_fuse` f64 RRF accumulation non-associative → exact ties land 1 ULP apart by input order, defeating the id-ascending §4.5.3 tie-break) → host-fixed | **40/40 property rows HELD** after the `rrf_fuse` host fix (5 suites: props_store 12, props_graph 13, props_facts 14, props_consistency 12, props_retrieval 8) + negative-generator probes + 1 `rrf_fuse` regression (retrieval_stack 18→19) | 5 read-only PBT audits. **§4.5 P-IM-2 = genuine host defect (fixed + regression-pinned);** register re-scopes (store P-IM-1/P-SM-1/P-SM-2 monotonic-id + annotation-reconcile; graph P-TP-2 same-call idempotence + residual-alias; consistency P-IM-3/P-TP-2/P-SM-1/P-SM-3 scoping); §4.3 stale `[PENDING]` tags re-tagged GREEN (already-fixed defects); negative-generator probes added; **2 engine-internal items recorded**: §4.4 `publish_document` concurrent-atomicity TOCTOU non-goal (defects.md + HANDOFF.md), §4.2 `resolve_entities` residual-alias behaviour (defects.md + pending.md) | N/A (property-layer gates, not UI scenario) | proofreader applied all register/tracker corrections; test count reconciled to **293** (the archived doc-review's 234 overcount was itself wrong — actual baseline was 233) | `cargo test` **293 pass, 0 fail** · `build` clean · `clippy` 0 warnings · `fmt` clean |
| **§4.1 document store** (spec §4.1) | TestWriter red: **41 red + 1 green** at the compile-with-stubs stage; after the red-set fixture corrections, the implementer landed **42/42**, then the adversarial regression set added **8 tests** (5 red) → **50/50** | 50/50 (`tests/store_integration.rs`) + placeholder 1 | **HIGH** out-of-range pagination panic (fixed: clamp/overflow-safe); **MEDIUM** delete TOCTOU dangling-reference (fixed: store-wide `reference_integrity` `RwLock`); **MEDIUM** crosslink `Broken`/`Stale` not gated on publish (fixed); **MEDIUM** fabricated reference state trusted (fixed: derive state from target existence, skip cross-wiki); **MEDIUM** WRITER-ACTOR-JOURNAL not honored (fixed: minimal mutation journal + epoch feed); concurrency tests single-threaded false security (fixed: multi-threaded + Barrier); plus low/`unwrap`/ordering notes | pending (documentation gates) | pending (documentation gates) | `cargo test` 51 pass (store 50 + placeholder 1) · `build` clean · `clippy` clean · `fmt` clean |
| **§4.2 knowledge graph** (spec §4.2) | TestWriter red: **55 red** at the stubs stage; after the §4.2 adversarial gate, **10 regression tests** pinned the CRITICAL/HIGH findings (rejected first green) → **66/66** after the graph-owned type evolution + a `max_hops` spec-conflict rebase | 66/66 (`tests/graph_integration.rs`) + store 50 + placeholder 1 | §4.2 adversarial rejected the first green. **CRITICAL (all fixed + regression-pinned):** triple `relationType` lived in a sidecar, not the graph (§4.2.7.2 — fixed via decided type evolution, `Edge.relation_type`); triple cascade only masked, leaving a `triple_store` second-source-of-truth and duplicate-on-re-add (fixed: prune relation edges on node delete + derive membership from the graph); `merge_facts` computed but never persisted (fixed: write back union citations + `updated_at` + journal + `get_fact`); `resolve_entities` had no durable effect (fixed: journaled alias→canonical `entity_resolution` + `entity_alias_canonical`). **HIGH:** `set_reference_state` whole-graph clobber + no reference-lock/optimistic compare (fixed: targeted edge-state mutation under `reference_lock` + revision-aware reconcile so concurrent updates don't lose data); `resolve_references` unbounded `max_hops` + ignored wiki (fixed: validate 1–5 → `ValidationError`, unknown wiki → `WikiNotFound`); `add_triple`/`get_triples` unknown wiki (fixed). **SPEC-CONFLICT:** three `resolve_references` tests used `max_hops:10` outside the pinned 1–5 — rebased to `2` (still ≥ chain length). **HANDOFF (upstream):** spec §4.2.9.1/§4.2.7.2/`resolveEntities` result-shape should be reconciled to match the graph-owned realization | pending (documentation gates) | pending (documentation gates) | `cargo test` 117 pass (graph 66 + store 50 + placeholder 1) · `build` clean · `clippy` clean · `fmt` clean |
| **§4.3 fact/citation tracking** (spec §4.3) | TestWriter red: **18 red + 6 green guards** at the stubs stage; after the §4.3 adversarial gate, **11 regression tests** pinned the findings → **36/36** | 36/36 (`tests/facts_integration.rs`) + graph 66 + store 50 + placeholder 1 | **HIGH:** delete gate didn't block/prune fact citations → dangling citations + a commit-time grounding TOCTOU (fixed: delete gate scans `fact_store` → `DocumentInUse`; fact commits take `reference_lock.read()` + re-verify grounding inside `fact_store.write()`). **MEDIUM:** `update_fact` accepted empty/whitespace value (fixed: trim→`ValidationError`); whitespace-only key/value passed schema conformance (fixed: trim before `is_empty` in gate + `create_fact`); inconsistent/absent unknown-wiki on the fact surface (fixed: `WikiNotFound` up front on `create_fact`/`update_fact`/`propose_candidate_fact`/`get_fact`); **cross-field consistency** doc mismatch — declared deferred to §4.5 (needs the embedding leg), test header corrected (not faked). **DEFERRED (pending):** fact-store sharding (single global `RwLock` → shard by wiki later); fact `node_id` is a store handle not a live graph node (reconcile by materializing graph `fact` nodes or dropping the claim); engine-side query **audit-log recording sink** (the `getQueryAuditLog` accessor exists; the recording feed lands in §4.5). **NIT/comment:** stale RED headers corrected | pending (documentation gates) | pending (documentation gates) | `cargo test` 153 pass (facts 36 + graph 66 + store 50 + placeholder 1) · `build` clean · `clippy` clean · `fmt` clean |
| **§4.4 consistency enforcement** (spec §4.4) | TestWriter red: **15 red** at the stubs stage (then two test-fixture defects fixed: a `"MTI"` seed typo + a read-before-join race) → **22/22** after the adversarial fix pass | 22/22 (`tests/consistency_integration.rs`) + facts 36 + graph 66 + store 50 + placeholder 1 | **HIGH (F1):** `publish_document` held a shard write while reading `fact_store`/other shards — an AB-BA deadlock vs the fact-commit paths (fixed: uniform lock order decision **LOCK-ORDER-REF-SHARD-SIDECAR**; publish validates under read locks then takes the single shard write only for the transition; fact-commits ground under `reference_lock.read()` before `fact_store.write()`). **MEDIUM:** propagation TOCTOU — embeds created after a fact-update scan could stay `FRESH` with a stale snapshot (fixed: derive embed state from snapshot-vs-canonical); `re_sync_embed` incoherent snapshot + non-fact targets (fixed); reference target-**change** (repoint) not propagated (fixed: re-stamp from new-target liveness); propagation didn't bump `revision`/journal on referencing docs (fixed). **LOW:** report fabricated state for `None`-state edges + wrong crosslink default (fixed: derive from target liveness; crosslink-with-snapshot = embed, crosslink-without = live link `RESOLVED`). **Concurrency tests:** added publish∥fact-commit∥re-sync stress + propagation/ordering regressions | pending (documentation gates) | pending (documentation gates) | `cargo test` 175 pass (consistency 22 + facts 36 + graph 66 + store 50 + placeholder 1) · `build` clean · `clippy` clean · `fmt` clean · concurrency stress stable + deadlock-free |
| **§4.5 RAG/agent-memory retrieval** (spec §4.5–§4.6) | TestWriter red: **47 red** at the stubs stage (3 files) + 4 green guards; **initially GREEN but ADVERSARIAL-REJECTED**; after the real rework + re-audit → **58/58** | 58/58 (rag_query 34 + retrieval_stack 18 + agent_memory 6) + store 50 + graph 66 + facts 36 + consistency 22 + placeholder 1 | First adversarial gate **rejected** the initial green (real, not shape-only): HIGH-1 `ragQuery(mode:vector)` ran the LEXICAL leg behind a vector trace (fixed: real dense vector leg + provider seam + `EmbeddingUnavailable`/`VectorIndexUnavailable` reachable from `rag_query`); HIGH-2 hybrid used the lexical list twice → degenerate flat (fixed: 3 distinct graph/vector/lexical legs + exact RRF); HIGH-3 retrieval fail-states were unreachable dead code (mostly fixed; `subTaskDag`/`CompressionFailed`/`HyDEGenerationFailed` recorded as **reserved** variants via decisions SUB-TASK-DAG-VALIDATED-ONLY + RESERVED-ERRVARIANTS-DISCIPLINE, sub-task DAG parked, FS-13/14/15 lexical tension → HANDOFF); HIGH-4 multi-query/compression/hyde were silently ignored (fixed: real fan-out/merge, filter compression, HyDE routing); MEDIUM-5 cross-wiki vector leak (fixed: wiki-scope); MEDIUM-6/8/9 vacuous greens (stale-parent, profile-value, non-contending concurrency) — all de-vacuated; MEDIUM-7 fabricated `Resolved` walk default (kept, ingest-derivation reachable); MEDIUM-10 `blocked_by` on non-empty (fixed). Second re-audit: MEDIUM-A **graph-leg nondeterminism in RRF/hybrid/graph** (fixed: sort by `(DocumentId,NodeId)` before `top_k`/RRF), LOW-G `binaryCandidatePool` default 10×topK (fixed + spec-conflict test corrected), LOW-E HyDE error mapping (fixed), LOW-F HyDE test de-vacuated. **233 tests total, trio clean** | pending (documentation gates) | pending (documentation gates) | `cargo test` 233 pass (rag_query 34 + retrieval 18 + agent_memory 6 + store 50 + graph 66 + facts 36 + consistency 22 + placeholder 1) · `build` clean · `clippy` clean · `fmt` clean · determinism + concurrency stable |
