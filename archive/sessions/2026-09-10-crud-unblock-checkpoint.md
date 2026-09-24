# CHECKPOINT — 2026-09-10 — CRUD-unblock roadmap: P1a + P2 LANDED, A1 in progress (subagent reliability issue)

> **Purpose:** the prior session was corrupted. This doc is the authoritative handover for a
> fresh supervisor to pick up the work cleanly. Read this FIRST, then the referenced files.
> The work spans TWO repos: **Gnosis** (the Rust engine, the session workspace) and
> **Astrographer** (the Electron shell). Both are under `/media/ryanr/Shared Files/Projects/`.

---

## 1. The objective

Unblock the deferred Gnosis document-CRUD + graph-operations endpoints through the Astrographer
MCP/UI (D4 parity), per the approved roadmap. **The MVP is document-CRUD only** (Binding decision
`GNOSIS-CRUD-MVP-SCOPE`): the units are **P1a → P2 → A1 → A2**. A3/A4/A5 (graph/fact/consistency
wiring) and P1b–P1e (graph/fact/consistency/RAG-companion wire) are deferred follow-ons, NOT gates.

## 2. The gate topology (enforce in this order)

proposal → [validity ∥ critique] → architecture_review → change_analysis → **spec gate** →
**TDD gate** (test_writer red → implementer green) → **adversarial** (incl. read-only PBT audit) →
**blind-greens** → **live-scenario** → **proofreader** → **doc-review** → **trio** → **trackers** → DONE.

Key rules:
- **Spec gate:** a code unit is delegable only once its `docs/specs/*.md` contract exists AND the
  reviewer loop returned empty.
- **TDD gate:** NEVER "implement X and add tests". First test_writer (red set from the spec ONLY),
  then implementer (least code to green). Record the red set in the DONE row.
- **PBT gate (MANDATORY):** every code-bearing unit needs the typed §5.x Property register
  (P-IM/P-SM/P-TP rows, ≤8, never F-rows/§6/FS-n), the executed property layer (deterministic
  pinned seed, ≤100/row, ≤400 total, stop-after-5), and the read-only PBT audit.
- **Adversarial:** host findings fixed here + regression-tested; package findings → `defects.md` +
  `HANDOFF.md` (never patch the package).
- **Live-scenario:** parked scenarios are NOT a failure (write a pending battery).

## 3. COMPLETED and LANDED

### 3.1 The roadmap (Astrographer)
- **File:** `/media/ryanr/Shared Files/Projects/Astrographer/docs/specs/unblock-gnosis-remaining-endpoints.md`
- Proposal gate **PROCEED-WITH-AMENDMENTS**. 7 amendments applied + 3 binding decisions recorded
  in `/media/ryanr/Shared Files/Projects/Astrographer/docs/decisions.md`:
  - `GNOSIS-SUPPLANTS-DOCUMENT-STORE` (H1 — Gnosis supplants the document store; the A2 screens are
    the document surface; the local `createJsonRagStore` is the D2 fallback).
  - `GNOSIS-CRUD-MVP-SCOPE` (document-CRUD-only MVP).
  - `GNOSIS-RBAC-EDIT-ENFORCEMENT` (H3 — Gnosis accepts a credential to enforce RBAC edit access;
    Astrographer stores user authority).
- The roadmap is the authoritative CRUD-deferral baseline.

### 3.2 P1a — document-CRUD wire-contract unit (Gnosis) — **LANDED**
- Spec: `docs/specs/p1a-document-crud-wire.md` (+ §5.x register, 8 rows)
- Impl: `src/wire/crud.rs`, `src/wire/decode.rs` (`DecodeError::UnknownMethod`), `src/wire/mod.rs`, `src/lib.rs`
- Tests: `tests/crud_wire_conformance.rs` (35), `tests/props_crud_wire.rs` (8), `tests/blind_p1a_crud_wire_greens.rs` (21)
- Greens: `docs/specs/p1a-document-crud-wire-greens.md` (19/19 PASS)
- Live battery: `docs/specs/p1a-document-crud-wire-live-pending-battery.md` (PENDING on A1)
- Doc-review: `archive/reviews/2026-09-10-p1a-doc-review.md`
- Adversarial: 4 HOST-MINOR fixed; no PACKAGE. Trio green (full suite 523 pass / 0 fail).
- Decision: `P1A-DOCUMENT-CRUD-WIRE-LANDED` in `docs/decisions.md`.

### 3.3 P2 — `gnosis-server` binary crate (Gnosis) — **LANDED**
- Spec: `docs/specs/p2-gnosis-server.md` (+ §5.x register, 7 rows)
- Impl: `src/bin/gnosis_server.rs`, `src/server.rs` (`server_status`/`request_decode_status`/`route_bijection`), `src/lib.rs`, `Cargo.toml`
- Tests: `tests/gnosis_server_conformance.rs` (25), `tests/props_gnosis_server.rs` (7), `tests/gnosis_server_e2e.rs` (16), `tests/blind_p2_gnosis_server_greens.rs` (15)
- Greens: `docs/specs/p2-gnosis-server-greens.md` (15/15 PASS)
- Live battery: `docs/specs/p2-gnosis-server-live-pending-battery.md` (20 MCP/UI parity scenarios M1–M20, PARKED on A1)
- Doc-review: `archive/reviews/2026-09-10-p2-doc-review.md`
- Adversarial: 2 HOST-MAJOR + 4 HOST-MINOR fixed; **2 PACKAGE recorded** in `docs/defects.md` + `docs/HANDOFF.md`:
  - **RBAC caller threading** — the `RagStore` mutating methods take no `caller` param; the engine
    must gain one (or §8 stays decode-layer-only). The P2 server validates `caller` presence on
    mutating requests (400 if missing) and threads it through the decode layer only.
  - **axum-in-lib-deps** — Cargo can't scope a dep to a bin; the spec §3 note documents that the
    lib code uses no axum.
- Trio green (full suite 523 pass / 0 fail). Decision: `P2-GNOSIS-SERVER-LANDED` in `docs/decisions.md`.

## 4. IN PROGRESS — A1 (Astrographer CRUD routing proxy client) — **NOT LANDING**

A1 is the next unit (queued as NEXT in `docs/next-steps.md`). It routes the 11 document-CRUD
methods over the frozen P1a wire + the live P2 server, reusing Unit GN's decode-then-validate,
the §11 map, the `EngineWireError` model, loopback + auth/TLS, READY observation, and D2
engine-absent framing. It is a **TypeScript** unit in the Astrographer repo.

**⚠️ CRITICAL — the A1 subagent work is NOT landing on disk.** The following files DO NOT exist:
- `docs/specs/unit-a1-crud-routing-proxy.md` (A1 spec)
- `tests/unit-a1-crud-routing-proxy.test.ts` (A1 conformance tests)
- `tests/props-a1-crud-routing-proxy.test.ts` (A1 property tests)
- `src/main/engine-crud-rag-store.ts` (A1 implementation)

The A1 subagents (subagent-169 through 174) are **not in the job registry** (the job list stops at
subagent-168). The A1 subagent calls returned outputs claiming success (spec written, red tests
written, implementer landing) but the files were never actually written. This is a **harness-level
reliability problem**, not a task problem.

**Do NOT trust any prior A1 subagent output.** Re-do the A1 work from scratch, verifying each file
lands on disk before advancing.

## 5. NEXT STEPS for a fresh supervisor

1. **Verify the baseline** (quick): `cargo test` in Gnosis → 523 pass / 0 fail; the P1a/P2 files
   in §3 exist. If the trio is green, the baseline is intact.
2. **A1 spec gate:** delegate `role_spec_writer` to write
   `docs/specs/unit-a1-crud-routing-proxy.md` (the A1 spec). **Verify the file lands on disk.**
   The spec must include: the 11 CRUD method surface, the endpoint constants (H4, from P1a
   `ENGINE_ENDPOINTS`), the CRUD request/response encode + decode, decode-then-validate, the §11
   status mapping + NEW-2 request-decode outcome (400/422, NOT 502), the RBAC `caller` threading
   (H3), the D2 engine-absent framing, and the §5.x Property register (8 rows, PBT gate).
   Run the reviewer loop until empty.
3. **A1 TDD gate:** delegate `role_test_writer` (red tests from the spec ONLY, incl. the property
   layer), then `role_implementer` (least code to green). **Verify each file lands.**
4. **A1 adversarial gate:** `role_adversarial_reviewer` (read-only, incl. the PBT audit). Host
   findings fixed here; package findings → `defects.md` + `HANDOFF.md`.
5. **A1 blind-greens + live-scenario + proofreader + doc-review + trio + trackers.**
6. **A2** (document CRUD D4 wiring) is the final MVP unit after A1.

## 6. Key file paths (both repos)

**Gnosis** (`/media/ryanr/Shared Files/Projects/Gnosis`):
- `docs/specs/gnosis.md` (the behavior contract), `docs/specs/engine-wire-contract.md` (F2 wire),
  `docs/specs/p1a-document-crud-wire.md`, `docs/specs/p2-gnosis-server.md`
- `src/store/mod.rs` (the `RagStore` trait, 45 methods), `src/wire/`, `src/bin/gnosis_server.rs`, `src/server.rs`
- `docs/next-steps.md`, `docs/pending.md`, `docs/decisions.md`, `docs/defects.md`, `docs/HANDOFF.md`
- `archive/reviews/` (the doc-review records)

**Astrographer** (`/media/ryanr/Shared Files/Projects/Astrographer`):
- `docs/specs/unblock-gnosis-remaining-endpoints.md` (the roadmap)
- `docs/specs/unit-gn-engine-integration.md` (Unit GN — the LANDED retrieval-trio proxy A1 reuses)
- `src/main/engine-rag-store.ts` (the LANDED proxy surface A1 extends)
- `docs/decisions.md`, `docs/pending.md`, `docs/next-steps.md`

## 7. Key decisions (all ACTIVE)

- `GNOSIS-CRUD-SURFACE-CONFIRMED` — the CRUD surface is the intended future surface (D4 parity).
- `GNOSIS-SUPPLANTS-DOCUMENT-STORE` — Gnosis supplants the document store; local store = D2 fallback.
- `GNOSIS-CRUD-MVP-SCOPE` — document-CRUD-only MVP (P1a + P2 + A1 + A2).
- `GNOSIS-RBAC-EDIT-ENFORCEMENT` — Gnosis enforces RBAC edit access (accepts a credential);
  Astrographer stores user authority.
- `P1A-DOCUMENT-CRUD-WIRE-LANDED`, `P2-GNOSIS-SERVER-LANDED` — the two landed units.

## 8. Known open items / caveats

- **A1 subagent reliability** — the A1 subagents do not land files. Verify every A1 file on disk.
- **RBAC caller threading (PACKAGE)** — the `RagStore` mutating methods take no `caller` param;
  the engine must gain one (or §8 stays decode-layer-only). Recorded in `defects.md` + `HANDOFF.md`.
- **Live-scenario parks** — P1a (on A1) and P2 (20 MCP/UI parity scenarios M1–M20, on A1) are
  parked; they un-park when A1 + a running Astrographer app exist.
- **Cargo registry** — the `~/.cargo` cache is outside the writable workspace; use a workspace-local
  `CARGO_HOME` (e.g. `.cargo-home`) to run `cargo test`/`build`, and remove it afterward.
