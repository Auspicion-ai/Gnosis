# §7.2 P2 **U2** — the `POST /rag/query` contract — **BLIND-GREENS SET**

- **Author-role:** blind_test_writer (documentation-only; the implementation was **not** read).
- **Sources (docs ONLY):** `docs/specs/p2-gnosis-server.md` — §5.2 (routing), **§5.3**
  (the `POST /rag/query` request/response contract, the 14-key table, the two-layer
  absent/wrongly-typed reading, the `query` cell's two halves, the canonical §4.5.2
  `filters` mapping), **§5.4** (the single case-insensitive token rule, the POST/SSE
  split, the SSE 400 frame), **§5.5** (the structured transport decode-error body),
  **§10** (per-endpoint fail-states + precedence), **§9.5.1** (U2's 7 property rows),
  **§9.5.4** (coverage notes); `docs/specs/engine-wire-contract.md` — §4.4 (SSE
  framing), §4.5 (the `ragQuery` wire), §7.1 (the transport decode body), §12
  (V-15/V-15.1), §13 (per-fn valid/fail states); `docs/specs/p1a-document-crud-wire.md`
  (the envelope/`ENGINE_ENDPOINTS` precedent only). **§5.6/§5.7 (the change cursor and
  `GET /changes`) and §9.5.2/§9.5.3 (U3/U4/U5) are out of scope for this set.**
- **Executed by:** `tests/blind_u2_query_post_greens.rs` (new; the blind set lives in
  its own test file, mirroring `tests/blind_p2_gnosis_server_greens.rs`): the live
  `gnosis-server` bin (`CARGO_BIN_EXE_gnosis-server`) for the HTTP/SSE legs and the
  engine lib's public API for the pure decode-level legs.
- **Blind discipline:** no `src/` file was read for expectations and **no existing test
  file was read for expectations** (only public signatures for compilation, plus the
  docs). Every expectation below quotes its clause; where a clause does not pin a value,
  that is listed in §Contract ambiguities rather than invented.
- **Run:** `CARGO_HOME="$PWD/.cargo-home" cargo test --test blind_u2_query_post_greens`
  — **23 PASS / 1 FAIL (24 scenarios)**; the failure is a live contradiction of a
  documented expectation (§Run result / §Failures below). **RESOLVED (2026-09-17, proofread-pass
  annotation — the historical run record is kept):** the `U2-13` defect was **fixed** (the decoder
  answers `None` at layer 1 for an object whose documented member is wrongly typed, `src/wire/query.rs`
  `enabled_object`) — **the pre-generalization name: after the `P-6` fix the landed function is
  `object_option` (`src/wire/query.rs:188-202`), total over every declared member** — and the spec hole it
  exposed was closed by `p2` §5.3's object-valued-option field-level pin; the set is now **24 PASS / 0 FAIL
  (24 scenarios)** and the then-verified suite was **`cargo test` 582 passed / 0 failed**. **RE-RUN (2026-09-17, after the post-clarification `U2-12`
  assertion correction — serial, `--test-threads=1`):** the set is **24 PASS / 0 FAIL (24 scenarios)**
  and the suite is **582 passed / 0 failed** over 31 binaries, exit 0 (see §Run result). No
  ephemeral-port collision occurred in this run. *(**Baseline note (2026-09-17, the U3 documentation pass):**
  `582 / 31 binaries` is **this set's own U2-time baseline**; the current tree's baseline is **604 passed / 0
  failed over 32 binaries** after U3 (U3's own binary — the blind-u3 set's 13 scenarios — plus **nine
  in-crate** additions: `props_gnosis_server` 15→21, `wire_conformance` 58→59, `gnosis_server_e2e` 27→28;
  `docs/specs/u3-status-honesty-greens.md` §Run result + `docs/next-steps.md`'s U3 DONE row). The 582
  figures in this file are the dated record and are not restated.)* A *transient* same-class drift in the **U2
  property-layer** artifact was observed mid-pass and resolved by that artifact's owner during this
  same pass — recorded, not edited here (§Run result, `U2-PROP-DRIFT`).
- **Date:** 2026-09-17.
- **Post-greens documentation review (2026-09-16; docs-only — no scenario, expectation or count changed).**
  This set was reconciled against the landed tree: **24/24** re-verified by counting the file's 24 test fns
  (**13 `#[tokio::test]` + 11 `#[test]`**, matching the live/pure split below), the `U2-13` finding history and
  the `U2-12` correction note kept intact, the `enabled_object` ⇒ **`object_option`** rename (the `P-6` fix) and
  its moved line numbers corrected, and the **layer statement** (pure-lib vs live-HTTP vs the battery's
  `R-L1`/`R-L2`) added. Review record:
  `archive/reviews/2026-09-16-u2-query-post-contract-doc-review.md`.

**Reading convention.** "non-400" is the contract's own quantifier on the token checks
(§5.4: a member token in any ASCII casing is recognized; §10's N1 split: an unread SSE
param is ignored). Where the documented outcome is an engine fail-state rather than a
shape, the scenario asserts the pinned status (`503` on a not-READY boot, §5.3's
fail-state table + §10).

---

## Scenario table

| # | id | clause | request | expectation | result |
| --- | --- | --- | --- | --- | --- |
| 1 | `U2-1` | §5.3 "Response body — the bare `RagResult` (NOT the SSE chunk wrapper)"; F2 §4.5 + §12 V-18 | pure: `encode_result_checked(&well_formed_RagResult)` (the 200 body the contract describes) | the §4.1 envelope (`schemaVersion:1`, `idFormat:"opaque-string-v1"`) whose `payload` is the **bare** serde `RagResult` body — `query`/`results`/`engine`/`citations`/`trace`/`blocked_by`, snake_case, `engine=="gnosis"`, `trace` externally tagged, `blocked_by:null` — and **not** `{"type":"result","result":…}` | PASS |
| 1b | `U2-1b` | §5.3 + §10 (`POST /rag/query` happy path) | live: `POST /rag/query` `{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"query":"hello","mode":"flat","topK":10,"filters":{"nodeKind":"fact"}}}` | well-formed ⇒ either 200 with the U2-1 bare-`RagResult` payload, or the READY gate 503 `engine_unavailable` — **never** 400/422 | PASS (live observed **503** `{"code":"engine_unavailable","message":"engine is not ready"}`; the 200 half is unreachable on a fresh boot — pure half is U2-1) |
| 2 | `U2-2` | §5.4 (last row + "never a silent default"), §5.3's fail-state table, §9.5.4 `P-SM-4` | live POST `payload {"query":"x","mode":T}` for `T ∈ {"", " ", "\t", "bm25", "flat ", "flat\n", " hybrid", "flat\tflat", "FlatX", "vectors", "none", "parent"}` | **400** + the §11 error envelope (`{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"code":"validation_error","message":…}}`) via `encode_error` | PASS |
| 3 | `U2-3` | §5.3 precedence (2 before 3 and before 4); §5.4 | live POST `payload {"query":"x","mode":"bm25","topK":0}` | 400 `validation_error` (the **mode** error, not the range error, not 503) | PASS |
| 4 | `U2-4` | §5.4 (`flat|graph|vector|hybrid`, any ASCII casing); §9.5.1 `P-SM-4` (ii) | live POST `payload {"query":"x","mode":T}` for all 16 casings of the four member tokens | **non-400** (200 or the READY gate 503) | PASS |
| 5 | `U2-5a` | §5.3 "Value tokens (casing)"; §5.4 | live POST `expand ∈ {parent, PARENT, none, None}`, `compression ∈ {graph, GRAPH, filter, extract, ExTrAcT}` | **non-400** | PASS |
| 5 | `U2-5b` | §5.3 "Value tokens (casing)" ("an unrecognized token ⇒ 400 … never a silent default"); §5.4 | pure: `expand ∈ {"", " ", "bogus", "parent ", "parent\n"}`, `compression ∈ {"", " ", "nope", "graph ", "graphs"}` | `Err(QueryDecodeError::Validation(StoreError::ValidationError(_)))` — never a transport code, never a 422 | PASS |
| 6 | `U2-6` | §5.5 (code table + `Content-Type` **EXACTLY** `application/json`, no `charset`), F2 §12 V-15/V-15.1 | live POST `schemaVersion ∈ {0,2,99,u32::MAX}`; `idFormat ∈ {"uuid-v4","","OPAQUE-STRING-V1","opaque-string-v2"}`; both unknown; non-object `payload ∈ {5,null,"hi",[],true}`; bare `{"query":"x"}`; unparseable `{"schemaVersion":1,`; `payload {"query":"x","method":"bogus"}` | status 400; `Content-Type` byte-exact `application/json`; body exactly `{"code":…,"message":…}` (2 keys, that order, **not** envelope-wrapped): `unsupported_schema_version` / `unknown_id_format` / `invalid_envelope` / `invalid_json`; version **before** idFormat; `UnsupportedSchemaVersion(99)` ⇒ `"unsupported schemaVersion: 99"`; `method` is a tolerated extra key ⇒ never 422/400 | PASS |
| 7 | `U2-7` | §5.3 (`P-IM-4`'s two disjoint outcome sets), §5.5 | pure: unknown `schemaVersion`/`idFormat`/both; non-object `payload` ×5; all 5 request-decode variants | `Transport(UnsupportedSchemaVersion(v))` / `Transport(UnknownIdFormat(_))` / `Transport(InvalidEnvelope(_))` (`InvalidJson` unreachable here); `request_decode_status` is `Some(400|422)` on exactly that 5-variant domain | PASS |
| 8 | `U2-8` | §5.4 N1/F4, §10 (`GET /rag/stream` row), §9.5.1 `P-SM-4` (i), F2 §4.4 | live `GET /rag/stream?query=x&mode=T` for `T ∈ {bm25, "", %20}` | **HTTP 400** + **exactly one** frame: `event: error`, `data: {"type":"error","code":"validation_error","message":…}` (never a 200, never a `result` frame) | PASS |
| 9 | `U2-9` | §5.4 ("one rule, both readings"); §9.5.1 `P-SM-4` (ii) | live `GET /rag/stream?query=x&mode=T` for `T ∈ {HYBRID, Hybrid, hYbRiD, flat, Graph, VECTOR}` | **non-400** (the token went through the shared resolver) | PASS |
| 10 | `U2-10` | §5.4 N1 (only `expand`/`compression` split), §10 N1, §9.5.1 `P-SM-4` (iii) | live `GET /rag/stream?query=x&expand=parent&compression=nope` (and `?expand=bogus`, `?compression=bogus`, `?expand=&compression=`, `?wikiId=w1&maxHops=2&hyde=true`) | **non-400** and **no** `validation_error` frame (the params are not read) | PASS |
| 11 | `U2-11` | §5.3 unknown-key tolerance ("load-bearing"), §5.4 "Absent `mode` ⇒ `Flat` … still reach the READY gate and still return 503" | live POST payload `{"query":"x","args":{}}`, `{"query":"x","args":{"anything":1},"method":"bogus"}`, `{"query":"x","requester":"user:alice"}`, `{"query":"x","nodeKind":"content","extra":[1,2]}`, `{"query":"x"}` (no `mode`, no `topK`) | never 400, never 422; on a READY engine 200, else 503 `engine_unavailable` (the documented defaults are the engine's query-time reading) | PASS |
| 12 | `U2-12` | §5.3's per-key wrongly-typed table + the **two-layer** bullet; §9.5.1 `P-IM-5`/`P-TP-2` | pure POST decode of one payload carrying every wrong-type: `mode:5`, `topK:"10"`, `wikiId:5`, `maxHops:null`, `expand:5`, `maxParentContext:"5"`, `multiQuery:5`, `compression:null`, `hyde:"yes"`, `binaryFirstPass:1`, `binaryCandidatePool:"10"`, `subTaskDag:5`; plus well-typed controls; plus the `query` halves | every wrongly-typed field `== None` (never `Some(default)`), no `Err`; `{multiQuery:{enabled:true}}` ⇒ `Some(MultiQueryOptions{enabled:true,n:3})`; well-typed ⇒ `Some(mapped)`; absent `query` ⇒ `""` (**not** an `Err`), present non-string `query` ⇒ `Err(Validation(ValidationError(_)))`; `requester == None` | PASS (one assertion **CORRECTED 2026-09-17** after the contract clarification — see §Correction notes, `U2-12`) |
| 13 | `U2-13` | §5.3 `multiQuery`/`subTaskDag` cells ("an **object** with a non-boolean `enabled` ⇒ absent ⇒ disabled") + §9.5.4 `P-IM-5` (object-member misuse ⇒ the documented default) | pure POST decode `{"query":"x","multiQuery":{"enabled":"yes"},"subTaskDag":{"enabled":"yes"}}` | `multi_query == None` **and** `sub_task_dag == None` (layer 1: "the decoder NEVER substitutes a typed default value … and writes none into the options") | **FAIL — implementation defect** (see §Failures) — **RESOLVED: PASS after the fix (2026-09-17)** |
| 13b | `U2-13b` | §5.3 ("not a decode failure and not a 400") | live POST `{"query":"x","multiQuery":{"enabled":"yes"}}`, `{"query":"x","subTaskDag":{"enabled":"yes"}}`, `{"query":"x","multiQuery":{"enabled":true,"n":"3"}}`, `{"query":"x","multiQuery":{"enabled":false,"n":"0"}}` | never 400; the documented default's effective outcome (200 on a READY engine, else the 503 READY gate) | PASS |
| 14 | `U2-14` | §5.3/§9.5.1 `P-TP-2`/`P-IM-4` (unknown keys change nothing) | pure POST decode of the same payload with and without `args`/`method`/`requester` | identical `query` and element-wise identical `RagQueryOptions`; `requester == None` | PASS |
| 15 | `U2-15` | §5.3 canonical §4.5.2 mapping table + N5; §9.5.1 `P-IM-6` | pure POST decode of the fully-populated canonical `filters` (`nodeKind:"fact"`, `edgeType:"embed"`, `target:{documentId,nodeId}`, `state:"FRESH"`), a canonical subset, the casing variants (`"Fresh"`, `"CONTENT"`, …), `{}`, `null`, a `target` extra member, an unknown member NAME | `Some(QueryAuditFilters{node_kind:Some(Fact), edge_type:Some(Embed), target:Some((DocumentId("d1"),NodeId("n1"))), state:Some(Fresh)})` — never all-`None`; subset ⇒ exactly the named members; `{}` ⇒ `Some(all-None)` (present/valid/honored); `null` ⇒ `None`; a `target` extra ⇒ the two named members; an unknown member name ⇒ `Ok(Some(all-None))` | PASS |
| 16 | `U2-16` | §5.3 `filters` exception bullet (non-object / wrongly-typed member / unrecognized token ⇒ 400), §9.5.4 `P-IM-6` (the store-serde shape is a SHAPE MISMATCH ⇒ `Err`) | pure POST decode `filters ∈ {[], "x", 5, {nodeKind:5}, {state:5}, {edgeType:[]}, {target:"d1"}, {target:["d1","n1"]}, {target:{documentId:"d1"}}, {target:{nodeId:"n1"}}, {target:{documentId:1,nodeId:"n1"}}, {nodeKind:"community"}, {edgeType:"docHead"}, {edgeType:"member"}, {state:"frozen"}, {node_kind:"Content",target:["d1","n1"]}}` | `Err(Validation(ValidationError(_)))` for every one — **never** `Ok(Some(all-None))` | PASS |
| 17 | `U2-17` | §5.3 fail-state table (FS-3 ranges), §10 | live POST `{"query":"x","topK":0}`, `{"query":"x","topK":51}`, `{"query":"x","maxHops":0}`, `{"query":"x","maxHops":6}`, `{"query":"x","multiQuery":{"enabled":true,"n":0}}`, `{"query":"x","binaryFirstPass":true,"binaryCandidatePool":0}`, `{"query":"x","filters":{"nodeKind":"community"}}`, `{"query":"x","filters":"nope"}` | **400** + §11 error envelope `validation_error` | PASS |
| 18 | `U2-18` | §5.3 `query` cell (both halves) | live POST `payload {}`, `{"query":""}`, `{"query":5}`, `{"query":null}` | **400** `validation_error` (absent ⇒ `""` ⇒ FS-3; present non-string ⇒ the decoder's own `Err`) | PASS |
| 19 | `U2-19` | §5.5 code table + "Disjointness (testable)"; §9.5.1 `P-TP-3` | pure: the 5 request-decode variants, 4 out-of-domain variants, three `StoreError`s | 5 codes exactly (`invalid_json`/`invalid_envelope`/`unsupported_schema_version`/`unknown_id_format`/`unknown_method`), statuses 400/400/400/400/422 (never 502), `None` outside the domain for **both** fns, every code absent from the §11 map (`from_wire(code, Some("m")) == None`), §11 `wire_code()`s disjoint from the transport set, `server_status(EngineError) == Some((502,"engine_error"))` | PASS |
| 20 | `U2-20` | §5.3 "encoder must never emit a body the result validator rejects"; §9.5.1 `P-TP-4` | pure: a well-formed `RagResult`; `engine ∈ {"", "not-gnosis"}`; `blocked_by:Some(..)` with a `Flat` trace | well-formed ⇒ `Ok(Envelope)`; each rejected class ⇒ `Err(StoreError::EngineError)` (⇒ 502, never a 200) | PASS |
| 21 | `U2-21` | §5.4 "one rule, both readings" + §9.5.1 `P-SM-4` (single-sourcing / POST≡SSE classification) | pure: the same `mode` input through `decode_query_request(Post, …)` and `decode_query_request(Sse, …)` | identical classification: same `Ok(Some(enum))` for member strings (all casings), same `Err(Validation(_))` family for `""`/`" "`/`"bm25"`/`"flat "` | PASS |

**Scenario count: 24 (`U2-1`, `U2-1b`, `U2-2`..`U2-21`).** Each row is one `#[test]`;
the live rows are `U2-1b`, `U2-2`, `U2-3`, `U2-4`, `U2-5a`, `U2-6`, `U2-8`, `U2-9`,
`U2-10`, `U2-11`, `U2-13b`, `U2-17`, `U2-18`.

**Layer statement (which rows run where — re-verified against the tree by the post-greens
documentation review, 2026-09-16).** The set is **24 scenarios = 13 live-HTTP + 11 pure-lib**, and the
split is *by test attribute* in `tests/blind_u2_query_post_greens.rs`, not by claim:

- **13 live-HTTP rows** (`#[tokio::test]`; they spawn/step the compiled `gnosis-server` bin over loopback
  and assert the real wire): **`U2-1b`, `U2-2`, `U2-3`, `U2-4`, `U2-5a`, `U2-6`, `U2-8`, `U2-9`, `U2-10`,
  `U2-11`, `U2-13b`, `U2-17`, `U2-18`.** On a fresh (not-READY) boot the well-formed POSTs stop at the READY
  gate — **503 `engine_unavailable`** — which is the documented precedence-(4) outcome (§5.3/§5.4), and why
  the set branches on `GET /engine/status` for the 200 half.
- **11 pure-lib rows** (`#[test]`; they assert the library surface the U2 seam exports —
  `decode_query_request` / `resolve_*` / `request_decode_code` / `encode_result_checked`): **`U2-1`,
  `U2-5b`, `U2-7`, `U2-12`, `U2-13`, `U2-14`, `U2-15`, `U2-16`, `U2-19`, `U2-20`, `U2-21`.**

**The live *battery* rows are NOT part of this set (layer boundary, stated so nothing is credited twice).**
`docs/specs/p2-gnosis-server-live-pending-battery.md` §3.5 carries the two live rows the U2/U3 registers home
there: **`R-L1`** (`P-TP-2`'s (β)) **PASSED live** on a **READY** boot with a real provider — the wire
`topK`/`filters` visibly changed the response and an unrecognized `mode` returned **400 `validation_error`** —
so U2's live obligation is discharged; **`R-L2`** is **`P-IM-9`'s provider-reachable boot half and belongs to
U3**: it needs a **controlled** provider and U3's flag derivation, so it is **out of this set's and U2's
scope and is not covered here**.

---

## Correction notes (assertion alignments *after* a contract clarification)

### `U2-12` — one assertion corrected **2026-09-17** (post-clarification; credited to this set's own `U2-13` finding)

- **What changed.** In `u2_post_wrongly_typed_values_decode_as_absent_layer1`
  (`tests/blind_u2_query_post_greens.rs`, the `multiQuery.n` half), the payload
  `{"query":"x","multiQuery":{"enabled":true,"n":"3"}}` is now asserted to decode as
  `multi_query == None`. **As authored** it asserted `Some(MultiQueryOptions{enabled:true,n:3})`
  ("the documented `n` default") — the **pre-clarification**, member-level reading.
- **Why (a clarification, not a rule change).** `p2` §5.3's object-valued-option clause (the
  "U2 object-valued-option field-level pin", 2026-09-16, ADDITIVE) and §5.3's two object cells
  now pin that **EVERY documented member of an object-valued option is field-level validated** —
  `enabled` a **bool**, **`n` a `u64`** for `multiQuery` — so a documented member that is
  **present-and-wrongly-typed makes the whole option absent (`None`)**, while a member
  **omitted inside a well-formed object** keeps its documented default. §5.3's `multiQuery` cell
  states the corrected reading literally: "a present non-u64 `n` is a wrongly-typed documented
  member, so `{"enabled":true,"n":"3"}` ⇒ `None`" (**ruling 2, 2026-09-16**); §9.5.1's `P-IM-5`
  domain and §9.5.4's `P-IM-5` note carry the same three-instance adversarial corpus with
  `{"multiQuery":{"enabled":true,"n":"3"}}` named in it.
- **Attribution.** The clarification was **forced by this set's own `U2-13` finding** (kept verbatim
  in §Failures below): the blind set proved the decoder was answering with a *present* object
  carrying decoder-supplied member defaults (`enabled:false`, `n:3`) instead of field-level `None`.
  The fix and the pin **generalize that finding from `enabled` to every documented member**,
  `n` included — so this correction is downstream of a finding this set made, not of a review that
  overruled it.
- **Not smoothed over.** This is an **assertion alignment to a *clarified* rule — not a greens
  failure being hidden**: the `U2-13` failure record and its resolution history are kept intact,
  **no scenario was deleted**, and no expectation was edited to match an implementation. The
  correction is recorded here, dated, at the scenario it touches, and the pre-correction
  expectation is quoted above so the read is auditable.
- **Unchanged boundary readings (still correct under the clarified rule).** `{"multiQuery":{}}` ⇒
  `Some({enabled:false,n:3})`; `{"multiQuery":{"enabled":true}}` ⇒ `Some({enabled:true,n:3})`
  (omitted member ⇒ documented default); `{"subTaskDag":{}}` ⇒ `Some({enabled:false})`; the misuse
  cases `{"multiQuery":{"enabled":"yes"}}` / `{"subTaskDag":{"enabled":"yes"}}` ⇒ `None`.
- **Live scenarios untouched, expected outcomes unchanged.** `U2-13b` still asserts non-400 for
  `{"multiQuery":{"enabled":"yes"}}`, `{"subTaskDag":{"enabled":"yes"}}`,
  `{"multiQuery":{"enabled":true,"n":"3"}}` and `{"multiQuery":{"enabled":false,"n":"0"}}`;
  `U2-17` still asserts **400** for `{"multiQuery":{"enabled":true,"n":0}}` (`Some` ⇒ the engine's
  range validator). Under the clarified rule the wrongly-typed-`n` payload is now `None` at layer 1
  and so reaches the option's documented default — which is non-400 either way, exactly as `U2-13b`
  already pinned.

---

## Run result

**Historical run (as authored — kept as the gate record).**

```
$ CARGO_HOME="$PWD/.cargo-home" cargo test --test blind_u2_query_post_greens
running 24 tests
… 23 ok …
test result: FAILED. 23 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.19s
```

**Current state (2026-09-17, proofread-pass annotation).** After the `U2-13` fix the set is
**24 passed / 0 failed (24 scenarios)** and the suite total is **582 passed / 0 failed** (the historical
`581 passed / 1 failed` below is that run's record; the per-binary counts in it are unchanged and still
match the tree — `props_gnosis_server` 15, `gnosis_server_e2e` 27, `wire_conformance` 58, this set 24).
The historical `558 + 23 = 581` arithmetic therefore reads `558 + 24 = 582` today.

**RE-RUN after the `U2-12` correction (2026-09-17, blind_test_writer, serial).** The set is
**24 passed / 0 failed (24 scenarios)** — no scenario deleted, 24 still present:

```
$ CARGO_HOME="$PWD/.cargo-home" cargo test --test blind_u2_query_post_greens -- --test-threads=1
running 24 tests
… 24 ok …
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.92s

$ CARGO_HOME="$PWD/.cargo-home" cargo test --lib --tests --no-fail-fast -- --test-threads=1
… 31 test binaries …
(aggregate) 582 passed / 0 failed          ⇒ exit 0
(0 occurrences of "Address already in use")
```

**Transient same-class drift observed mid-pass in the property layer (`U2-PROP-DRIFT` — found by this
pass, resolved by that artifact's owner during it).** An intermediate serial suite run in this same
pass (04:44, before the property artifact's owner landed their edit) returned **581 passed / 1 failed**
with exactly one failure — `props_gnosis_server::u2_p_im_5_query_option_defaults`, whose expectation
helper `absorbed_expectation` (and its doc comment at `:1330-1334`) still read
`{"enabled":true,"n":"3"}` ⇒ `Some({enabled:true, n:3})` ("the documented `n` default"), i.e. a
*present* wrongly-typed documented member defaulted inside a kept-`Some` object — while the decoder now
(correctly, per the clarified §5.3) answers `multi_query == None`. The property's own panic text already
quoted the clarified rule ("a wrongly-typed member is layer-1 None, never a fabricated default
object"), so it was the property **helper** that was stale w.r.t. the pin, not the implementation.

- **Not this set's artifact and never edited here:** `props_gnosis_server.rs` is the property-layer
  deliverable (a separate artifact; see the closing section), so it was **reported** as a finding for
  its owner rather than changed under a blind-greens pass — the same discipline `U2-13` was held to.
  The owner's edit landed during this pass and the target is now **15 passed / 0 failed**; the final
  full-suite re-run above is **582 / 0**.
- It was **not** caused by this blind set (separate test binaries, no shared state) and **not** an
  ephemeral-port collision (0 occurrences of `Address already in use`); every run in this pass was
  serial (`--test-threads=1`) precisely to exclude that harness effect.
- The `U2-12` assertion correction and the property drift are **independent instances of one drift
  class** — a pre-clarification member-level reading of a present wrongly-typed documented member.
  This set's own count was never affected by the property failure: **24/24** throughout.

Commands run (all from the repo root, `CARGO_HOME` pointed at the writable in-repo
cargo home because the sandbox blocks `~/.cargo`):

1. `cargo fmt -- --check` ⇒ exit 0
2. `cargo test --test blind_u2_query_post_greens` ⇒ **23/24** (the one failure below)
3. `cargo test --lib --tests --no-fail-fast` ⇒ **581 passed / 1 failed** over 31 test
   binaries; every pre-existing binary is green (**558 pre-existing tests still pass**:
   `agent_memory_integration` 6, `blind_p1a_crud_wire_greens` 21,
   `blind_p2_gnosis_server_greens` 15, `community_context_integration` 15,
   `consistency_integration` 22, `crud_wire_conformance` 35, `eval_harness` 11,
   `facts_integration` 36, `gnosis_server_conformance` 25, `gnosis_server_e2e` 27,
   `graph_integration` 66, `integration` 1, `props_community_context` 9,
   `props_consistency` 12, `props_crud_wire` 8, `props_eval` 8, `props_facts` 14,
   `props_gnosis_server` 15, `props_graph` 21, `props_retrieval` 8, `props_store` 12,
   `props_wire` 8, `rag_query_integration` 36, `retrieval_stack_integration` 19,
   `store_integration` 50, `wire_conformance` 58 = **558**
   + this set's 23 = 581)
4. `cargo clippy --all-targets` ⇒ exit 0, **0 warnings**

**Census precision (post-greens doc-review, 2026-09-16).** The **558** in command 3 is the **post-U2
per-binary census** — it *includes* U2's own contribution to those binaries (`props_gnosis_server` 15 = the 7
landed P2 rows + the 7 U2 rows (`U2PIM4`…`U2PTP4`) + `u2_layer_budget_discipline`; `gnosis_server_e2e` 27 = the
pre-U2 16 + U2's 11 live cases; `wire_conformance` 58 includes `v15_request_decode_error_body_exact`), so
"pre-existing" here means **"binaries other than this blind set"**, not "unaffected by U2". The pre-U2 baseline
those figures grew from was **538** (`docs/next-steps.md`), and with this set's 24 the suite is **582**.

**Live observation for U2-1b (recorded by the scenario itself).** On a fresh boot
(`gnosis-server --port <ephemeral>`) the engine is **not READY**, so the well-formed
POST took the documented READY-gate half and returned:

```
HTTP 503
{"schemaVersion":1,"idFormat":"opaque-string-v1","payload":{"code":"engine_unavailable","message":"engine is not ready"}}
```

i.e. the request passed the envelope check, the token resolver and the range validator
(no 400) and was stopped by the READY gate — which is exactly what §5.3/§5.4's
"absent `mode` still reaches the READY gate" and precedence (4) describe. The **200**
half of §5.3 is therefore not observable on a fresh boot in this environment; its
body shape is verified structurally by `U2-1` (the encoder output is the bare
`RagResult` payload, not the SSE wrapper). The set branches on the live readiness
surface (`GET /engine/status`, §6/§10) so the 200 half is asserted whenever a READY
boot is provided.

---

## Failures (triage)

**STATUS (2026-09-17, proofread-pass annotation): the one failure below was FIXED and the spec hole it
exposed was closed** — the decoder now returns field-level `None` for an object whose documented member is
wrongly typed (`object_option` — the function the finding's fix generalized; it was named `enabled_object` when
this record was written, `src/wire/query.rs:183-190` ⇒ now **`:188-202`**), and `p2` §5.3's new object-valued-option
field-level pin (`p2` §U1's "U2 object-valued-option field-level pin" bullet) makes the reading the only one
the contract permits. The record below stands as the gate's finding.

### `U2-13` — `u2_post_object_member_misuse_is_absent_layer1` — **implementation defect (live contradiction)**

- **Request (pure):** `decode_query_request(QueryPath::Post, &envelope({"query":"x",
  "multiQuery":{"enabled":"yes"},"subTaskDag":{"enabled":"yes"}}), SseParams::default())`.
- **Expected (docs):** layer 1 of §5.3's "THE TWO-LAYER READING OF THE `absent ⇒`
  COLUMN" bullet: the wrongly-typed member decodes as **absent**, i.e.
  `options.multi_query == None` and `options.sub_task_dag == None`. §5.3's per-key
  table: "`multiQuery` … an **object** with a non-boolean `enabled` ⇒ absent ⇒
  disabled"; "`subTaskDag` … an object with a non-boolean `enabled` ⇒ that member
  absent ⇒ disabled". The two-layer bullet is explicit that the decoder "NEVER
  substitutes a typed default value — it applies no documented default of its own and
  writes none into the options", and §9.5.1's `P-IM-5`/`P-TP-2` assert the decoder's
  output element-wise (`== None`), with §9.5.4's `P-IM-5` note naming precisely this
  "object-member misuse" corpus.
- **Observed:** `Ok((_, RagQueryOptions { multi_query: Some(MultiQueryOptions { enabled:
  false, n: 3 }), sub_task_dag: Some(SubTaskDagOptions { enabled: false }), … }))`.
  Exact evidence (each run isolates one half; the assertion order was swapped between
  runs to report both):
  - `left: Some(SubTaskDagOptions { enabled: false })` / `right: None`
    (panicked at the `sub_task_dag` assertion, `tests/blind_u2_query_post_greens.rs`)
  - `left: Some(MultiQueryOptions { enabled: false, n: 3 })` / `right: None`
    (panicked at the `multi_query` assertion)
- **Diagnosis: (ii) implementation defect.** The decoder normalizes the two
  object-valued options with `unwrap_or(false)`/`unwrap_or(3)` on the **members** while
  keeping the container `Some(…)`, i.e. it writes decoder-supplied defaults
  (`enabled:false`, `n:3`) into the options — the behavior §5.3's layer-1 clause and
  `P-IM-5`/`P-TP-2` forbid. It is **not** a blind-set defect: the expectation is quoted
  verbatim from §5.3's two-layer bullet + the `multiQuery`/`subTaskDag` cells of §5.3's
  per-key table, and it is not a contract ambiguity (both cells name "absent ⇒
  disabled" and the two-layer bullet pins which layer the row asserts).
- **Blast radius (measured, not assumed):** the **wire outcome is unchanged** —
  `U2-13b` (live) is green: every object-member-misuse payload is non-400 and reaches
  the documented default's effective outcome, because the engine reads
  `multi_query.enabled` (false either way) and never reads `sub_task_dag` (§5.3's
  reachability note). So this is a **decoder-identity / layer-1 regression**, invisible
  on the wire today, and it is exactly the class `P-IM-5`/`P-TP-2` exist to catch.
- **Not smoothed over:** the scenario was kept as authored (the assertion text and
  expectation are unchanged); it was only split out of the broader `U2-12` wrongly-typed
  scenario into its own test so the failing claim reports on its own id. No expectation
  was edited to match the implementation.

### Raw wire evidence used by this report (fresh boot, `curl`)

```
POST /rag/query  {"schemaVersion":1,"idFormat":"opaque-string-v1","payload":5}
  400  content-type: application/json
  {"code":"invalid_envelope","message":"envelope payload must be a JSON object"}

POST /rag/query  {"query":"x"}                     # a bare, non-envelope body
  400  content-type: application/json
  {"code":"invalid_envelope","message":"missing or non-u32 \"schemaVersion\""}

GET /rag/stream?query=x&mode=bm25
  400  content-type: text/event-stream
  event: error
  data: {"type":"error","code":"validation_error","message":"mode must be flat, graph, vector, or hybrid (got \"bm25\")"}
  <blank line>
```

(The non-object-payload byte-exact body is F2 §12's V-15.1; note that its V-15.1 literal
`"envelope payload must be a JSON object"` happened to match, but the doc pins only the
structure + derived string — see ambiguity 1.)

---

## Contract ambiguities (cannot be blind-verified)

These are **not** failures; they are the places where the documentation does not pin the
observable, so this set either does not assert it or asserts only the pinned half.

1. **The `message` text of a transport decode body is implementation-derived.** §5.5/F8
   + F2 §12's V-15.1 binding note: only `code` + status + the header + the
   `code`/`message` key set/order are byte-binding; the `"envelope payload must be a
   JSON object"` literal of V-15.1 is explicitly **"a SUGGESTED string, not a pin"**. The
   set therefore asserts the structure plus `code`, and asserts a byte-exact `message`
   only for the one guaranteed-non-empty render (`UnsupportedSchemaVersion(99)` ⇒
   `"unsupported schemaVersion: 99"`). Serde's own parse-error text for `InvalidJson` is
   never asserted (§5.5).
2. **The `Content-Type` of the SSE 400 `error`-frame response is not pinned.** §5.4's F4
   pins the SSE **status** (400), the **frame bytes** and "then the stream closes", and
   §5.5 pins `Content-Type: application/json` for the *transport decode body* only; no
   clause stated the header for the SSE error body. The set asserts status + frame and
   leaves the header unasserted. (Observed in this environment:
   `text/event-stream`.) **SUPERSEDED IN PART (2026-09-17, proofread pass):** the header **is now
   pinned** — `p2` §5.4's F-2 bullet, §5.6 and §10, and F2 §4.4, require `Content-Type:
   text/event-stream` (exactly, no `charset`) on **every** SSE response. This scenario's own
   unasserted-header status is a blind-set choice and is unchanged; the pre-existing divergence of the
   post-stream `StoreError` branch (`text/plain; charset=utf-8`) is `docs/defects.md` **P-5**.
3. **The exact transport code for a bare (non-envelope) but well-formed JSON body is not
   pinned per-input.** §5.5's table maps `InvalidEnvelope(String)`/`InvalidJson(String)`
   to `invalid_envelope`/`invalid_json`, but the assignment for one specific degenerate
   body (`{"query":"x"}` — valid JSON, no envelope shape) is not stated; only the
   outcome class is ("a bare (non-envelope) body is REJECTED" ⇒ transport 400). The set
   asserts 400 + `invalid_envelope`; the observed code matched, but a different
   implementation carrying `InvalidJson` for that input would still satisfy the pinned
   text. (Same shape for §10's "unparseable JSON" row, where the code is pinned but the
   message is not.)
4. **The `filters` token whitespace boundary is deliberately unpinned.** §5.4's
   no-trim rule is pinned for `mode` only, and §9.5.4's `P-IM-6` note explicitly says the
   padded-`filters`-token state (`{"nodeKind":"content "}`) "is asserted by **no** row
   here (a generator MUST NOT invent that state)". The set does not assert it.
5. **`filters.nodeKind`/`edgeType` extra members and unknown member names are only pinned
   in the register's observable cells** (§9.5.1 `P-IM-6`, §9.5.4), not in §5.3's prose;
   the set asserts the register's reading (tolerated ⇒ `Ok`, contributing no option).
6. **The decoder's `None`-vs-`Some(false)` normalization for a wrongly-typed
   *container member* is pinned at layer 1 by §5.3's two-layer bullet but the wire
   outcome is identical at layer 2** — so no live scenario can distinguish the two
   readings. This is what makes `U2-13` a pure-level claim (and why its failure is a
   finding at the register layer, not a consumer-visible 4xx/5xx change).
7. **The 200 response's own `Content-Type` is not pinned** for `POST /rag/query` (no
   clause states the media type of a successful response envelope), so only the body
   shape is asserted.
8. **The READY precondition of the 200 happy path**: no clause pins how a READY engine is
   booted, so the set cannot force the 200 half and branches on
   `GET /engine/status` instead (see §Run result).

---

## What this set does NOT cover (out of U2 scope)

- `GET /changes` and the change cursor (§5.6), the paginated-read bounds (§5.7) and the
  boot vector-index build (§5.8/U5) — U4/U5 contracts, explicitly not part of U2.
- The U3 status-honesty rows (`P-IM-7/8/9`, `P-SM-5/6`) and the boot/provider wiring.
- §9.5.3's property-layer generators (`props_*`) — this is the blind **scenario** set; the
  register's property layer is a separate artifact.
