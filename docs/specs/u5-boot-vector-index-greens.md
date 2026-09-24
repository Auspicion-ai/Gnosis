# §7.2 P2 **U5** — the boot vector-index build — **BLIND-GREENS SET**

- **Author-role:** blind_test_writer (documentation-only). **No `src/` file was read for
  expectations**, and **neither `tests/u5_boot_vector_index_conformance.rs` nor the U5 rows of
  `tests/props_gnosis_server.rs` was read before the scenarios were written and run** — the only
  non-doc inputs were public API *signatures* (for compilation), `Cargo.toml`, and the two
  **precedent blind sets** (`tests/blind_u2_query_post_greens.rs`,
  `tests/blind_u3_status_honesty_greens.rs`) for style, harness shape and the live/pure split.
- **Sources (docs ONLY):** `docs/specs/p2-gnosis-server.md` — **§5.8** (capability semantics + U5's
  answer bullet + the REMAND-1 V-8.1 scoping), **§5.9** (the FS-3/8/13/14 outcome table, the
  `Precedence (pinned)` bullet: the READY gate precedes every leg check for non-`graph` modes, and
  the index is checked before the provider), **§6** (the post-U5 READY-lifecycle sequence),
  **§9.5.2** (U3's `P-IM-7`/`P-IM-8`/`P-IM-9`, the capability predicate table, F13/F16),
  **§9.5.5** (the whole U5 contract: contract tables (1)–(5), the eight typed rows
  `P-IM-10`…`P-IM-15`/`P-SM-7`/`P-TP-5`, the corpus-seeding clause, the valid/fail states 1–7, the
  move table, the adjudication notes incl. the PARKED live criteria, the residual list), **§9.5.4**
  (the eight U5 coverage notes), **§10** (`/rag/query` fail-states), **§11** (the U5 obligation row +
  the TestWriter surface notes), **§13** (U5's constraint bullets);
  `docs/specs/engine-wire-contract.md` — **§9.1** (the U5 note on the `vector` flag), **§12**
  (V-8.1/V-8.2 and their U5 notes), **§16** (rules 1–6, incl. "readiness precedes leg
  availability"); `docs/specs/p2-gnosis-server-live-pending-battery.md` **§3.5** (`R-L2`'s amended
  cell and `R-L3`, whose criteria (iii) and (v) are PARKED).
- **Executed by:** `tests/blind_u5_boot_vector_index_greens.rs` (new; own test file, mirroring the
  precedent blind sets): the lib seam `build_boot_vector_index`, the pinned `boot_wiring` seam, a
  real `Store`, an injected deterministic in-memory `EmbeddingProvider`, the public `RagStore`
  corpus-seeding calls, and the live `gnosis-server` bin (`CARGO_BIN_EXE_gnosis-server`) for the two
  hermetic HTTP rows.
- **Verdict vocabulary:** **GREEN** (verified) / **RED** (contradiction — reported, never reconciled
  by editing the test) / **NOT-VERIFIED** (unconstructible, with the reason). This set carries **27
  executed scenarios, all GREEN (27 passed / 0 failed)**, and **6 NOT-VERIFIED** claims recorded in
  §"NOT-VERIFIED" below. **No RED.**
- **Run:** `CARGO_HOME="$PWD/.cargo-home" cargo test --test blind_u5_boot_vector_index_greens
  -- --test-threads=1` ⇒ **27 passed / 0 failed**; the full-suite confirmation is
  `CARGO_HOME="$PWD/.cargo-home" cargo test -- --test-threads=1` ⇒ **655 passed / 0 failed**
  (= the 628 baseline + this set's 27 scenarios).
- **Date:** 2026-09-22.

**Reading convention.** "The corpus" is a store seeded **only** through the public `RagStore`
surface (§9.5.5's corpus-seeding clause): `create_wiki` → `create_document` → `update_document`
with the doc-head/doc-end graph. The generator deliberately **never** creates two nodes of one
document sharing a `NodeId` (the corpus-key-uniqueness pin: "a TestWriter's generator MUST NOT
produce one"), and it never asserts insertion order inside `entries` (the determinism clause is
"pinned only up to determinism"). Vector equality is asserted under the pinned
*vector comparison / `NaN`* row (same `len()`, non-`NaN` elements bit-for-bit, `NaN` asserted only
in the `NaN` position, no tolerance) — `NaN` is **not** filtered out of the adversarial corpus.

---

## Scenario table

| # | id | clause (docs) | request / observation | expectation | result |
| --- | --- | --- | --- | --- | --- |
| 1 | `U5-1` | §9.5.5 (1) row 1 + (2) *empty store* + the ordering clause + §6; §5.8 | lib: `build_boot_vector_index(empty_store, Some(&p))`; compose `DerivedIndexes { vectors: built.ok().flatten(), ..default }`; apply the pinned wiring for `Reachable` | `Ok(Some(vi))` with `entries` **empty** (never `Ok(None)`); 0 `embed` calls; `snap.vectors.is_some()`; `Ready`; derived flags == **V-8.1** `{store,graph,lexical:true, vector:true, embedding:true, reranker:false}`; the store's derived read == the wiring's vector | GREEN |
| 2 | `U5-2` | §9.5.5 (1) row 2 + (4)'s call-count row | lib: the `Unreachable` boot — build called with **no** provider; the unchanged snapshot wired | `Ok(None)` (not an error); 0 `embed` calls; `Degraded`; **V-8.2** `{…, vector:false, embedding:false, reranker:false}`; `snapshot().vectors.is_none()` | GREEN |
| 3 | `U5-3` | §9.5.5 (1) row 3 | lib: the `Absent` boot | `Unavailable`; V-8.2; `vectors.is_none()`; 0 calls | GREEN |
| 4 | `U5-4` | §9.5.5 (2) (node-level unit, `value` = the embedded text, the call-count row); `P-IM-10`/`P-IM-11` | lib: a 3-node corpus (two `Some`, one `None`) | `entries` is exactly the 2 embeddable `(doc, node, Full)` keys with **source** ids; exactly **2** `embed` calls; the embedded texts are exactly the two node values (no title/tag/prefix text) | GREEN |
| 5 | `U5-5` | §9.5.5 (2) (all wikis/all shards, no wiki filter) + (4) (n is store-wide) | lib: two wikis, one document each, 3 embeddable nodes | one store-wide index over both wikis; key set exactly the 3 source keys; 3 calls; no per-wiki scoping | GREEN |
| 6 | `U5-6` | §9.5.5 (2) *non-embeddable nodes* + valid/fail state 3 | lib: a corpus whose nodes are **all** `value: None`; then a `value: Some("")` node | all-`None` ⇒ `Ok(Some(vi))` with 0 keys (never `Ok(None)`), 0 calls, and the boot's flag is still **V-8.1**; `Some("")` ⇒ exactly one key and one call with the **empty text** | GREEN |
| 7 | `U5-7` | §9.5.5 (2) *empty store* row | lib: `build_boot_vector_index(Store::new(), Some(&p))` | `Ok(Some(VectorIndex::default()))` — an **empty** index, **never** `Ok(None)`; 0 calls | GREEN |
| 8 | `U5-8` | §9.5.5 (2) (`epoch` unchanged; `lexical` unchanged; the build returns the index **alone**) + the composition clause | lib: a boot snapshot with `epoch: 7`, `vectors: None`, `lexical: None`; compose `{ vectors: built.ok().flatten(), ..boot_snapshot }` | the composed snapshot carries the built index, `epoch == 7` (the input's value), `lexical.is_none()` (U5 builds no `LexicalIndex`) | GREEN |
| 9 | `U5-9` | §9.5.5 (3) (full field only; `Binary` deliberately not built) | lib: a 2-node corpus; inspect every key | every key's `.2 == FieldType::Full`; **no** `Binary`/`Other(_)` key; no extra key for the coarse first pass | GREEN |
| 10 | `U5-10` | §9.5.5 (1)/(2)/(4); `P-IM-10` (closed three-outcome set; `None` is **not** an error) | lib: 3 store shapes (empty, populated, no-embeddable) × 3 provider inputs (`None`, all-`Ok`, always-`Err`) | every outcome ∈ `{Ok(None), Ok(Some(index over exactly the corpus)), Err(EmbeddingUnavailable)}`; `p == None` ⇒ `Ok(None)` with the injected provider's call count **0**; an all-`Ok` supplied provider ⇒ `Ok(Some)` with `n` keys and `n` calls; an always-`Err` provider over a non-empty corpus ⇒ `Err(EmbeddingUnavailable)` after **1** call (abort), and over a 0-node corpus ⇒ `Ok(Some(empty))` (0 calls ⇒ nothing can fail — §9.5.5's adjudication note 1(a)) | GREEN |
| 11 | `U5-11` | §9.5.5 (2)'s **call-count/call-discipline row** (an invariant) | lib: a 5-node corpus in 2 documents (3 embeddable, 2 `None`) | exactly 3 calls; **strictly sequential** (max in-flight == 1 ⇒ no fan-out); no call for a `value: None` node; the embedded texts are exactly the node values | GREEN |
| 12 | `U5-12` | §9.5.5 (4) (`Err` ⇒ the index is dropped whole) + `P-IM-13` + valid/fail state 4 | lib: fail-on-the-*k*-th-call for `k = 1..=n`; then the boot's documented failure outcome | every `k` ⇒ `Err(EmbeddingUnavailable)` (never `Ok(Some(partial))`, never `Ok(None)`) after exactly `k` calls; after the failure outcome `snapshot().vectors.is_none()` and `subsystems.vector == false` — no partial index is observable | GREEN |
| 13 | `U5-13` | `P-IM-13` (the failure is **not sticky**: a subsequent success is complete) | lib: a fail-once-then-succeed provider, the **same** store + provider object, two builds | build 1 ⇒ `Err(EmbeddingUnavailable)` after 1 call; build 2 ⇒ `Ok(Some(vi))` with **all 3** keys and 3 more calls (4 total) | GREEN |
| 14 | `U5-14` | §9.5.5 (4)'s "what the boot does with that `Err`" row (the pinned **order**) + `P-IM-15` | lib: `Reachable` + an always-`Err` provider ⇒ build → `swap_snapshot(DerivedIndexes::default())` → `set_embedding_provider` → `set_engine_state(boot_wiring(Unreachable, &snap).0)` | `Degraded` (never `Ready`); `{store,graph,lexical:true, vector:false, embedding:true, reranker:false}`; `last_error == Some("a non-core subsystem (embedding/reranker) is unavailable")` byte-identical; `version == CARGO_PKG_VERSION`; `vectors.is_none()`; the error family is the existing `EmbeddingUnavailable` ⇒ `Some((503, "embedding_unavailable"))` (no new code/§11 row) | GREEN |
| 15 | `U5-15` | §5.8 + §9.5.2 `P-IM-7`/`P-IM-9` + §9.5.5 `P-IM-14` + §12 V-8.1/V-8.2 | lib: `{Absent, Unreachable, Reachable}` × `{index-free, empty index, populated index}` snapshots; the pinned wiring applied; then a **false-mask** `set_subsystems(<all-true>)` write and a re-read | for **every** pair: derived read == the wiring's vector, core `true`, `reranker false`, `embedding == (a provider was wired)`, and `flags.vector == snap.vectors.is_some()`; **V-8.1 only** for `Reachable` + `vectors: Some`; `Reachable` + `vectors: None` ⇒ `Ready` + `vector:false` (**not** V-8.2's `embedding:false` shape); `Absent`/`Unreachable` + the unchanged snapshot ⇒ V-8.2 exactly (and the V-8.2-shaped vector with `vector:true` when handed an index-bearing snapshot); the mask write changes **nothing** (no stored mask is ever the producer) | GREEN |
| 16 | `U5-16` | §5.9's `Precedence (pinned)` bullet + §16 rule 6 + `P-SM-7` + valid/fail state 5 | lib: the `Absent` and `Unreachable` boots (non-READY), then `rag_query(mode=Vector)` | `Err(EngineUnavailable)` (FS-8) ⇒ `Some((503, "engine_unavailable"))` — **never** FS-14 whatever the snapshot holds; the injected provider's `embed` call count is **0** | GREEN |
| 17 | `U5-17` | `P-SM-7` + §5.9 + valid/fail state 6 | lib: a caller-built **READY** store with `vectors: None` and a provider wired, then `rag_query(mode=Vector)` | `Err(VectorIndexUnavailable)` (FS-14) ⇒ `Some((503, "vector_index_unavailable"))`; `embed` call count **0** (the index check precedes the provider) | GREEN |
| 18 | `U5-18` | §5.9's order pin (`src/store/mod.rs:4448-4454`, cited by the docs) | lib: on one READY store: (a) `vectors: None` + **no** provider; (b) `vectors: Some(_)` + **no** provider | (a) `VectorIndexUnavailable` (**not** `EmbeddingUnavailable`); (b) `EmbeddingUnavailable` (FS-13) ⇒ 503 `embedding_unavailable` | GREEN |
| 19 | `U5-19` | `P-SM-7` (the serving half) | lib: a 2-node seeded corpus → build → the pinned `Reachable` wiring (index swapped, provider wired, `Ready`) → `rag_query("<q>", {mode: Vector, wiki_id, top_k: 10})` | `Ok(RagResult)`; `RagTrace::Vector(_)`; `engine == "gnosis"`; `results.len() <= top_k`; every returned `(documentId, nodeId, Full)` is a key of the built index (observed `results.len() == 2` of 2 indexed nodes) | GREEN |
| 20 | `U5-20` | `P-SM-7` (the empty-index instance) + valid/fail state 2 | lib: a `Reachable` boot over an **empty** index, then `rag_query(mode=Vector)` | `Ok` with `results` **empty** — never an error, never a panic | GREEN |
| 21 | `U5-21` | `P-TP-5` (totality over the adversarial shapes) + §9.5.5 (2) (verbatim vectors, the `NaN` comparison row) | lib: shapes `{dimension change between calls, empty vector, zero vector, NaN/±∞, always-Err}` × `{empty store, 3-node store}` + a **lying** `is_available` (`true` while `embed` errors) | the outcome stays in the closed set; `Ok(Some)` ⇒ exactly the corpus keys with the provider's own value **verbatim** (same `len()`, non-`NaN` bit-for-bit, `NaN` only in the `NaN` position) and no `Binary` key; the empty-vector/zero-vector values are stored as-is; `is_available` is **never** consulted (`availability_calls == 0`, lying or not); no panic, no new error type, no `Ok(None)` for a supplied provider | GREEN |
| 22 | `U5-22` | §9.5.5 (2) (REMAND-4 NOTE 5: two nodes with the same `value` are two embeddable nodes) + `P-TP-5` | lib: a duplicate **text** under two distinct ids | two keys and **two** `embed` calls (no text-keyed dedup/cache) | GREEN |
| 23 | `U5-23` | `P-IM-12` (determinism + store-side-effect freedom) | lib: two builds over the same store state + provider; `epoch()`/`journal_len()`/the snapshot `Arc` identity/`vectors` presence/the wired provider captured before and after | both `Ok(Some(_))` with identical key sets and per-key vectors under the pinned comparison; `epoch()`/`journal_len()` unchanged (no journal entry appended); `Arc::ptr_eq` on the snapshot; `vectors` presence and the provider unchanged | GREEN |
| 24 | `U5-24` | §9.5.5 (5) (no env var, no ambient provider, no availability probe) | lib: with `GNOSIS_SERVER_OLLAMA_URL` pointed at a bogus URL, `build_boot_vector_index(store, None)` | `Ok(None)` (the outcome is a function of the two **arguments** only); 0 calls; the same store with the provider argument supplied ⇒ `Ok(Some(_))` — the seam consults no env var and no ambient provider | GREEN |
| 25 | `U5-25` | §13's U5 bullets ("does NOT add a route … a `StoreError` variant, a §11 row, a wire code, a `HealthReport` field or an `EngineSubsystems` field") + §9.5.5 (6) | pure: `route_bijection()`, the error map, the serialized flag shape | the route table is still **14** rows, a bijection, with the 11 CRUD rows equal to `ENGINE_ENDPOINTS` and the retrieval trio present, and **no** vector/index route; `EmbeddingUnavailable`/`VectorIndexUnavailable`/`EngineUnavailable` keep their §11 `(status, code)`; the serialized `EngineSubsystems` key set is exactly the six frozen flags (U5 adds no field) | GREEN |
| 26 | `U5-26` | §9.5.5 (1) row 3 + (5)'s hermeticity clause (`spawn_server_without_provider`) + §5.9's precedence | live (**provider env removed**): `GET /engine/status`, then `POST /rag/query {"query":"q","mode":"vector"}` | status 200, `state != "Ready"`, the flags exactly V-8.2 (no build happened); the query is **503 `engine_unavailable`** (FS-8) — never `vector_index_unavailable` | GREEN |
| 27 | `U5-27` | §9.5.5 (1) row 2 + §12 V-8.2's reachability paragraph + §5.9's precedence | live (**provider configured at a loopback port with nothing listening** ⇒ probe fails ⇒ `Unreachable`): the same two probes | status 200, `state == "Degraded"`, flags exactly V-8.2, `lastError` == the fixed string byte-identical; the query is **503 `engine_unavailable`** (FS-8) — the unreachable boot attempts **no** embedding call | GREEN |

**Scenario count: 27 (`U5-1` … `U5-27`).** Attribute split: **26 `#[tokio::test]` + 1 `#[test]`**
(`U5-25`, which is pure/synchronous). The split that matters is the **surface** split:

### Layer statement (which rows run where — stated so nothing is credited twice)

- **2 live-HTTP rows** — `U5-26`, `U5-27`. Both spawn the compiled `gnosis-server` bin on an
  ephemeral loopback port with the boot's provider configuration either **removed**
  (`GNOSIS_SERVER_OLLAMA_URL`/`GNOSIS_SERVER_OLLAMA_MODEL` ⇒ `Absent`) or **pointed at a loopback
  port with nothing listening** (⇒ the probe fails ⇒ `Unreachable`). No real network is touched and
  no provider is assumed.
- **25 lib-level rows** — `U5-1`…`U5-25`. They need no server and no provider: every state is built
  through the documented non-mask mutators (`swap_snapshot`, `set_embedding_provider`,
  `set_engine_state`), the corpus through the public `RagStore` surface, the build through the lib
  seam, and the flags through the store's own read-time derivation plus the `boot_wiring` assertion
  surface. `set_subsystems` appears **only** as `U5-15`'s adversarial write (never as an expectation
  source).
- **The live battery's `R-L3` is NOT covered here (layer boundary).** `R-L3` (i)/(ii)/(iv) need a
  **controlled provider** (an Ollama-compatible endpoint answering `/api/tags` **and** `/api/embed`)
  — a precondition a node test may not assume, and one §9.5.5's hermeticity clause explicitly
  forbids a U5 test from depending on. `R-L3` (iii) and (v) are **PARKED** by the spec itself. All
  four are therefore recorded as **NOT-VERIFIED** below, with their lib-level equivalents named.

### Hermeticity evidence (run with a live provider present)

An Ollama **is** reachable at `http://127.0.0.1:11434` in this environment, so the set was run both
ways: `GNOSIS_SERVER_OLLAMA_URL=http://127.0.0.1:11434 cargo test --test
blind_u5_boot_vector_index_greens -- --test-threads=1` ⇒ **27 passed / 0 failed**, identical to the
plain run. The two live rows pin their own boot configuration, so the ambient environment cannot
select a different branch (§9.5.5 (5): the suite is green with and without the provider env var).

---

## NOT-VERIFIED (unconstructible, with the reason)

These are **not** failures and **not** scenarios that were dropped to make the set pass: each is a
claim the docs name but that no hermetic node test can drive. Every one has a **lib-level
equivalent that is GREEN** in the table above, and none of them is counted in the 27.

| # | claim the docs name | why it is NOT-VERIFIED | where the pinned half IS asserted |
| --- | --- | --- | --- |
| `NV-1` | `R-L3` (i)/(ii): a provider-**`Reachable`** live boot reports `state:"Ready"` + `subsystems.vector:true` + `embedding:true` + `reranker:false` + core `true`, and its `mode=vector` probe returns **HTTP 200** | needs a **controlled provider** answering `/api/tags` **and** `/api/embed`; §9.5.5's hermeticity clause: "A U5 test MUST inject a deterministic in-memory `EmbeddingProvider` through the lib seam and MUST NOT depend on a live provider or on the ambient environment"; the row's home is `R-L3` in the live battery | `U5-1`, `U5-15`, `U5-19`, `U5-20` (lib: the built snapshot ⇒ V-8.1, the derived flag, and `mode=vector` serving `Ok`) |
| `NV-2` | `R-L3` (v): the live **failed-`Reachable`-build** branch (`Degraded` + `vector:false` + `embedding:true` + the fixed `lastError`) | **PARKED by the spec**: §9.5.5's adjudication note 1 extension (a) — with the bin as it stands the boot store is in-memory and **empty** at boot, so a reachable provider's build makes **zero** `embed` calls (`n = 0` ⇒ `Ok(Some(empty index))`, **not** an `Err`) and the branch is "structurally non-exercisable live". The un-park trigger is a pre-bind/durable corpus (the **NOT ACTIVE** durability direction) | `U5-14` (lib: the pinned failure order and the element-wise status literal) + `U5-12` |
| `NV-3` | `R-L3` (iii): on a **seeded** live corpus the returned results are drawn from that corpus | **PARKED by the spec** (REMAND-1 item 3(a)): the bin hard-codes `Store::new()` and builds at boot, the only seeding surface the running bin exposes (its CRUD routes) necessarily runs **after** the build, and U5 has **no rebuild vehicle** (boot-only build) | `U5-19`, `U5-4`, `U5-5`, `U5-11` (lib: the corpus is seeded before the build and the served results are checked against the built index's keys) |
| `NV-4` | the byte-exact **V-8.1** golden literal + the `honest_ready_subsystems()` fixture + the `boot_wiring_couples_to_the_derived_read` probe edits | a **conformance** obligation, not a U5 register row (§9.5.5's register notes: "the fixture/literal edit is asserted by the conformance suite"), and its home is `tests/wire_conformance.rs` — outside this gate's blind scope (reading that file for expectations is not part of a docs-only derivation) | `U5-15` asserts the **mask** the literal projects (`Reachable` + `vectors: Some` ⇒ exactly `{store,graph,lexical:true, vector:true, embedding:true, reranker:false}`) |
| `NV-5` | "U5 adds no env var and no CLI flag" as a **closed** configuration set | §9.5.5 (5) records it as a **register obligation that nothing asserts**: "no test or spec in this repo asserts the set of env vars or CLI flags the `gnosis-server` bin reads … **U5 proposes no knob and therefore owes none** ✓" — a closed-set assertion would be a new obligation the unit does not owe | `U5-26`/`U5-27` (partial: the bin boots and serves on `--port` alone with the documented provider env pair, so U5 introduced **no new required knob**) and `U5-24` (the build consults no env var at all) |
| `NV-6` | "the `StoreError` taxonomy is still **21** variants / the §11 map still **21** rows" as a *count* | the count is not observable through the public API this set may use (no enumerator for the variant set or the map size); §13's U5 bullet states the no-invention claim, but a census cannot be asserted docs-only | `U5-25` (the three relevant codes keep their §11 `(status, code)` entries), `U5-10`/`U5-14`/`U5-21` (every failure outcome is the **existing** `EmbeddingUnavailable`; never a `ValidationError`/`EngineError`/new variant) |

---

## Run result

```
$ CARGO_HOME="$PWD/.cargo-home" cargo test --test blind_u5_boot_vector_index_greens -- --test-threads=1
running 27 tests
... (27 lines) ...
test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
```

**27/27 GREEN (27 passed / 0 failed; 2 live-HTTP + 25 lib-level).**

Hermeticity re-run (a live Ollama **is** reachable at `127.0.0.1:11434` in this environment):

```
$ CARGO_HOME="$PWD/.cargo-home" GNOSIS_SERVER_OLLAMA_URL=http://127.0.0.1:11434 \
    cargo test --test blind_u5_boot_vector_index_greens -- --test-threads=1
test result: ok. 27 passed; 0 failed; ... finished in 0.17s
```

Full suite (serial) and the trio re-check:

```
$ CARGO_HOME="$PWD/.cargo-home" cargo test -- --test-threads=1
    ⇒ 655 passed / 0 failed   (the 628 baseline + this set's 27 scenarios)   exit 0
$ CARGO_HOME="$PWD/.cargo-home" cargo fmt --check            ⇒ exit 0
$ CARGO_HOME="$PWD/.cargo-home" cargo clippy --all-targets   ⇒ 0 warnings
$ CARGO_HOME="$PWD/.cargo-home" cargo build                  ⇒ exit 0
```

**RED: none.** No scenario was edited to match the implementation and no assertion was weakened. The
one live provider in the environment is never reached by any scenario (the live rows pin their own
boot configuration; the lib rows inject an in-memory provider).

### Derivation corrections made while authoring (disclosed, none an implementation finding)

Four scenarios were **first written over-general and corrected to the docs' actual pins after their
first run**; in each case the landed behaviour matched the doc clause and the correction was on the
test side. They are recorded so the corrections cannot be mistaken for post-hoc weakening:

1. **`U5-10` / `U5-21` — an always-`Err` provider over a 0-node corpus.** I first required
   `Err(EmbeddingUnavailable)` for it. §9.5.5's `Ok(Some(vi))` arm is conditioned on "*a provider was
   supplied and every `embed` call succeeded*" — vacuously true when `n = 0`, and adjudication note
   1(a) pins the outcome exactly ("*n* = 0 embeddable nodes ⇒ `Ok(Some(VectorIndex::default()))`, an
   **empty** index, **not** an `Err`"). Corrected to that pin, with the call count asserted **0**
   (nothing can fail).
2. **`U5-13` — "not sticky" needs a *subsequent* build.** I first called the build once and expected
   the fail-once provider to succeed anyway. `P-IM-13` pins "*a subsequent success* returns
   `Ok(Some(vi))` with the full corpus". Corrected to two builds over the **same** store and provider
   object: the first returns the atomic `Err` after 1 call, the second returns the **complete** index
   (3 calls more). This is a **stronger** scenario than the first version.
3. **`U5-15` — V-8.2 is scoped to the *unchanged* snapshot.** I first asserted V-8.2 for every
   `Absent`/`Unreachable` pair, including index-bearing snapshots. §9.5.5's `P-IM-14` observable pins
   V-8.2 for "`Absent`/`Unreachable` + **the unchanged snapshot**", while the rule that holds for
   **every** `(p, snap)` pair is `flags.vector == snap.vectors.is_some()`. Corrected to the pair-wise
   literal (and the V-8.2-shaped vector with `vector:true` for an index-bearing snapshot), with the
   universal invariant still asserted in the same scenario.
4. **The `u5_13` call-count expectation** moved with correction 2 (1 failing + 3 succeeding = 4).

---

## Contract ambiguities (cannot be blind-verified; recorded, not promoted to pins)

Nothing below is asserted by this set; each is a place where the cited clauses do not pin an
observable, so the set asserts only the pinned half. **None of them is a failure and none is a
finding against U5.**

1. **`U5-19`'s result *count* is not pinned.** `P-SM-7` pins `Ok`, the trace, `engine == "gnosis"`
   and `results.len() <= top_k`, and that the results are drawn from the index's wiki-scoped
   full-field entries — it does **not** pin non-emptiness for a non-empty index (only the
   empty-index ⇒ empty-results instance is pinned). The set therefore asserts membership +
   `<= top_k` and records the observation (`results.len() == 2` over the 2-node corpus) as an
   observation, not a pin.
2. **The score/ranking of the served results is not pinned.** No U5 clause pins a similarity metric
   over the built entries (`cosine`'s dimension-tolerant shared-prefix reading is explicitly
   unchanged and out of this unit), so no scenario asserts an order or a score.
3. **The `version` *value* is not pinned** — every clause only spells it `env!("CARGO_PKG_VERSION")`
   (`P-IM-15`'s literal). `U5-14` asserts that mirroring, never a hard-coded version string.
4. **`is_available`'s own return value is not observable through the build.** The docs pin only that
   the build **does not consult** the seam (asserted by a call counter == 0), not what a provider
   would answer.
5. **Insertion order inside `VectorIndex.entries` is explicitly not assertable** ("a TestWriter MUST
   NOT assert a particular insertion order inside the `HashMap`"). The set asserts the key set and
   per-key vectors only.
6. **No assertion is made about live provider behaviour** (timeouts, HTTP status codes, model
   drift) — those are §9.5.5's residual-risk items, not invariants; the residual
   `Degraded` + `embedding:true` `lastError` self-contradiction (residual (v)) is likewise **not**
   asserted either way, because the fixed string is byte-pinned and U5 must not reword it.

---

## What this set does NOT cover (out of scope)

- **The U5 property layer** (`tests/props_gnosis_server.rs`, tags `U5PIM10`…`U5TP5`, the 314/355
  executed layer) — the register's generated layer is a separate artifact with its own
  seed/tag/budget discipline.
- **The conformance layer** (`tests/u5_boot_vector_index_conformance.rs`'s 15 tests and the
  `tests/wire_conformance.rs` fixture/literal/probe edits) — read by neither this set nor its author
  (`NV-4`).
- **The live battery's `R-L2`/`R-L3`** — `R-L2` is U3's live row; `R-L3`'s four criteria are named
  as `NV-1`/`NV-2`/`NV-3` (plus its provider precondition, which this set cannot assume).
- **U4** — the change cursor, `GET /changes` and the paged reads (still HELD); `U5-25` asserts only
  that U5 added no route/variant/field.

---

## Gate-8 reconciliation of this report against the landed spec (2026-09-22)

**Scope of this note:** the documentation reviewer (gate 8) re-read this report's claims against
`docs/specs/p2-gnosis-server.md` **§9.5.5 as landed**, its contract tables (1)–(5), the failure table and the
adjudication notes, and against the battery's `R-L3` row. **No scenario row, count, verdict or NOT-VERIFIED
entry was changed** — the corrections below are arithmetic/provenance labels on this report's own header and
run-result lines, made in place (the superseded wording is kept).

1. **The four disclosed derivation corrections all reconcile to the docs, not to a weakened test.** Each was
   re-checked against the clause it cites: the always-`Err`-provider-over-a-0-node-corpus outcome against
   §9.5.5's `Ok(Some(vi))` arm + **adjudication note 1(a)** ("*n* = 0 embeddable nodes ⇒
   `Ok(Some(VectorIndex::default()))`, an **empty** index, **not** an `Err`") — `U5-10`/`U5-21`; the
   not-sticky reading against **`P-IM-13`** ("*a subsequent success* returns `Ok(Some(vi))` with the full
   corpus") — `U5-13`; the V-8.2 scoping against **`P-IM-14`**'s pair-wise literal ("`Absent`/`Unreachable`
   + **the unchanged snapshot**") with the universal invariant
   `flags.vector == snap.vectors.is_some()` still asserted in the same scenario — `U5-15`; and the
   `u5_13` call-count move (1 + 3 = 4) follows from the second. **None is an implementation finding and none
   was a post-hoc weakening**; the landed code agrees (the build returns `Ok(Some(VectorIndex{ entries }))`
   over a zero-length corpus without calling `embed`, `src/store/mod.rs:5405-5413`).
2. **Baseline arithmetic — corrected wording.** The header block and the run-result block spell the full-suite
   confirmation as "the **628** baseline + this set's 27 scenarios". That reading is **superseded**: the U5
   landing's own green count is **628 = 604 (the pre-U5 baseline) + U5's 24 in-crate tests**, and the blind set
   adds its **27** on top ⇒ **655 passed / 0 failed**, which is what the tree reports. The figure `655` and the
   `27/27` verdict are **current and correct**; only the "628 baseline" label was loose (628 was itself an
   interim, post-green, pre-blind count). Gate-8 re-read: `docs/next-steps.md`'s U5 DONE row and the U5
   verification-status block in `docs/specs/p2-gnosis-server.md` §U1 both record the same arithmetic.
3. **The set's layer claims agree with §9.5.5's own classification.** 2 live-HTTP rows (`U5-26`/`U5-27`) + 25
   lib-level rows, with `U5-25` the single pure `#[test]`, match the report's layer statement; the set's
   `U5-16`/`U5-17`/`U5-18` FS-8-vs-FS-14 split matches §9.5.5's valid/fail **state 5** / **state 6** pairing
   exactly (FS-8 on a non-READY store; FS-14 only on a READY store with `vectors: None`) — the same reading the
   live battery's `R-L3` (iv)/(v) cells were corrected to in the same pass (**finding F1**).
4. **`NV-1`/`NV-2`/`NV-3` status after the gate-6 live pass (no change to this report's verdicts).** `NV-1` (the
   provider-`Reachable` live boot) is **now live-verified** by the battery's `R-L3` (i)/(ii) and by `R-L2`'s
   amended cell — recorded in the battery, not retro-fitted into this report's table (the set's verdict column
   is that author's run record). `NV-2` (the failed-`Reachable`-build branch) is **PARKED, not live-verified**
   (an empty boot corpus makes zero `embed` calls ⇒ no live path to fail; §9.5.5's POST-GREEN SPEC AMENDMENT
   item 5). `NV-3` (the seeded live corpus) stays **PARKED** with its reason re-confirmed live; the **FS-14
   READY-index-free mapping** joins it as **structurally non-reachable through the bin post-U5**
   (`docs/specs/p2-gnosis-server-live-pending-battery.md` §7).
5. **No `src/`, `tests/` or other spec edit is made by this note** — the corrections are labels and provenance
   on this report only. Review record:
   `archive/reviews/2026-09-22-u5-boot-vector-index-doc-review.md` (gitignored provenance).
