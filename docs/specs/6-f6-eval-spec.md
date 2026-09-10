# §7.8 F6 — RAG evaluation harness (deterministic contextual precision/recall) — behavior contract

- **Unit:** §7.8 F6 — RAG evaluation harness (SHOULD-HAVE dev/QA gate).
- **Status:** **LANDED + GREEN (2026-09-09)** — the F6 unit is implemented and
  trio-green (**387 tests**, clippy/fmt clean). This is the behavior contract the
  F6 unit's TestWriter derived its red set from. It pins the **pure deterministic
  metric functions** + the **`gnosis-eval` `[[bin]]`** + the **labeled-corpus
  fixture** + the **test-suite**. It is a **dev/QA gate, NOT a runtime contract
  element**: ZERO runtime change, ZERO new runtime deps, ZERO change to any
  §4.3.4/§4.5/§4.6 surface.
- **Gate:** proposal-review **PASSED — RE-SCOPED** (`docs/specs/6-f6-eval-review.md`,
  decision `F6-EVAL-RE-SCOPED`). The as-stated F6 (audit-log-fed, faithfulness/
  relevancy, DeepEval-in-Rust) is **INVALID**; the re-scoped F6 is the approved,
  delegable deliverable. **LANDED + GREEN (2026-09-09)** — see the F6 DONE row in
  `docs/next-steps.md`.
- **Contract cross-refs:** `docs/specs/gnosis.md` §7.8 (the gap), §4.5.1 (query
  modes), §4.6.1 (`ragQuery` surface + `RagResult`), §4.3.4 (audit log — the
  harness does **NOT** use it); `docs/specs/6-f6-eval-review.md` (the verdict +
  deliverable set + metric semantics + zero-runtime guardrail + suite-side split);
  `docs/decisions.md` `F6-EVAL-RE-SCOPED`.
- **Date:** 2026-09-09. **Author-role:** spec_writer.
- **Scope:** `src/retrieval/eval.rs` + the re-export surface in `src/lib.rs` +
  `src/bin/gnosis_eval.rs` + `tests/eval_harness.rs` + `tests/props_eval.rs` +
  `tests/fixtures/eval_corpus.json`. Companion PBT register:
  `docs/specs/6-f6-eval-property-register.md`.

---

## 1. What F6 asks

Pin a **deterministic, ground-truth-labeled contextual precision/recall module** —
a dev/QA gate that objectively compares retrieval changes against a hand-authored
labeled corpus (query → relevant node set). It is **not** a runtime contract
element: it does not change how the engine answers queries, and it ships **no new
runtime dependencies** (serde / serde_json / tokio / futures already present
suffice).

The module computes four **pure, deterministic** metrics over a `RagResult`'s
retrieved items and a labeled relevant-node set:

- `contextual_precision` — how precise the retrieved ranking is at the ranks
  where relevant items appear.
- `contextual_recall` — what fraction of the labeled relevant set the top-k
  retrieved.
- `ndcg_at_k` — discounted cumulative gain normalized to the ideal ranking.
- `mrr_at_k` — the reciprocal rank of the first relevant item.

The `gnosis-eval` `[[bin]]` reads a labeled corpus (JSON fixture), seeds a `Store`,
runs `rag_query` per case, computes the four metrics, and prints a per-case +
aggregate report. It supports a **deterministic fake embedding provider**
(reproducible CI) and a **real provider** (live vector/hybrid) via a flag/env.

F6 is **not** a DeepEval Python harness, **not** a runtime eval endpoint, **not**
a faithfulness/answer-relevancy judge, and **not** a change to the §4.3.4 audit
log or any §4.5/§4.6 surface. The harness **re-runs `rag_query`** against a READY
engine (live data acquisition; deterministic metric math offline) — it does **not**
read the audit log.

---

## 2. Scope guardrails

**In scope (this contract):**
- `src/retrieval/eval.rs` — the four pure deterministic metric fns
  `contextual_precision`, `contextual_recall`, `ndcg_at_k`, `mrr_at_k`, re-exported
  from `src/lib.rs`.
- `src/bin/gnosis_eval.rs` — the `[[bin]]` `gnosis-eval` (a second `[[bin]]`
  alongside the existing `gnosis`; **no new deps**).
- `tests/eval_harness.rs` — pins the metric math + harness smoke.
- `tests/props_eval.rs` — the executed PBT layer (per the register
  `docs/specs/6-f6-eval-property-register.md`).
- `tests/fixtures/eval_corpus.json` — the hand-authored labeled corpus.

**NOT in scope (explicit):**
- The §4.3.4 audit log (`QueryAuditEntry`/`get_query_audit_log`) — the harness
  re-runs `rag_query`, it does **not** read the audit log. **ZERO change** to
  `QueryAuditEntry`, `rag_query`, `rag_stream`, `get_query_audit_log`, or any
  §4.3.4/§4.5/§4.6 surface.
- Faithfulness / answer-relevancy — these need a **generated answer**, which is
  the shell/LLM's job, **suite-side**.
- A DeepEval Python harness — **suite-side**, not Gnosis.
- A runtime eval endpoint — F6 is a dev/QA tool, not shipped runtime.
- A user-facing retrieval-quality surface — D4 parity applies at the Astrographer
  shell, not the headless Gnosis.
- Real benchmark data (BEIR-style) — the labeled corpus is hand-authored ground
  truth; its quality bounds the gate's meaningfulness.
- Any change to the frozen §4.1/§4.5 types (`RagResult`, `RagResultItem`,
  `DocumentId`, `NodeId`, `WikiId`, `QueryMode`, `Source`, `RagQueryOptions`).

---

## 3. Module map

| file | concern | exported public API |
| --- | --- | --- |
| `src/retrieval/eval.rs` | the four pure deterministic metric fns | `pub fn contextual_precision(results: &[RagResultItem], relevant: &HashSet<(DocumentId, NodeId)>, k: usize) -> f64`; `pub fn contextual_recall(...) -> f64`; `pub fn ndcg_at_k(...) -> f64`; `pub fn mrr_at_k(...) -> f64` |
| `src/bin/gnosis_eval.rs` | the `[[bin]]` `gnosis-eval` | reads a labeled corpus JSON, seeds a `Store`, runs `rag_query` per case, computes the four metrics, prints a per-case + aggregate report |
| `tests/eval_harness.rs` | metric-math + harness smoke | `#[test]` fns asserting the §5/§6 hand-computed values and the bin's report shape |
| `tests/props_eval.rs` | the executed PBT layer | one `#[test]` per register row (8 rows: P-IM-1..4, P-SM-1..2, P-TP-1..2) |
| `tests/fixtures/eval_corpus.json` | the labeled corpus | `[{query, wiki_id, mode, top_k, relevant: [[doc-id,node-id],…]}]` |

**Re-export surface to add in `src/lib.rs`** (the paths the TestWriter calls):

```rust
pub use self::retrieval::{contextual_precision, contextual_recall, ndcg_at_k, mrr_at_k};
```

The TestWriter reaches the metric surface as `gnosis::contextual_precision`,
`gnosis::contextual_recall`, `gnosis::ndcg_at_k`, `gnosis::mrr_at_k`, and the
types as `gnosis::{RagResultItem, DocumentId, NodeId}` (already re-exported). The
metric fns are **pure and synchronous** — they take `&[RagResultItem]` and
`&HashSet<(DocumentId, NodeId)>` and return `f64` directly (no `Result`, no
runtime, no `async`).

---

## 4. Pinned metric conventions (apply to ALL four fns)

- **Rank convention:** 1-based. A `RagResultItem` at 0-based index `i` in
  `results` has **rank `i + 1`**.
- **Effective top-k:** `effective_top = min(k, results.len())`. All four metrics
  operate only on the first `effective_top` items of `results`; items beyond the
  effective top are **ignored** (never counted as relevant, never contribute to
  any sum).
- **Relevance membership:** an item is *relevant* iff its `(document_id, node_id)`
  key is a member of the `relevant` set. Membership is by key equality
  (`DocumentId`/`NodeId` are `PartialEq`/`Eq`/`Hash` single-field `String`
  newtypes).
- **`log2(i+1)` denominator:** the discount factor for 1-based rank `i` is
  `log2(i + 1)` (so rank 1 → `log2(2) = 1`, rank 2 → `log2(3) ≈ 1.585`, …). This
  is the **pinned** denominator; it is never `log2(i)` (which would divide by zero
  at rank 1).
- **Empty-set conventions (pinned, apply to all four fns):**

  | input condition | precision | recall | nDCG@k | MRR@k |
  |---|---|---|---|---|
  | empty `relevant` | 0.0 | **1.0** (vacuous) | 0.0 | 0.0 |
  | empty `results` (relevant non-empty) | 0.0 | 0.0 | 0.0 | 0.0 |
  | `k = 0` | 0.0 | 0.0 | 0.0 | 0.0 |

  **Precedence (pinned):** empty `relevant` takes precedence over `k = 0` for
  recall — recall is **`1.0`** (vacuous) even at `k = 0`. (For precision/nDCG/MRR
  the two conditions agree on `0.0`, so no precedence conflict arises there.)

- **Boundedness:** every metric returns a value in `[0, 1]` for **any** input
  (including empty results, empty relevant, `k = 0`, `k > len`, and duplicate
  keys). No metric ever panics and no metric ever returns `NaN`/`inf`/negative.
- **Duplicate-key handling (pinned):** the retrieval stack dedups
  `(documentId, nodeId)` keys (`P-SM-4`), so duplicates are not the normal input.
  For robustness the metrics are defined on arbitrary inputs as follows:
  - `contextual_precision` / `mrr_at_k`: each position is checked independently
    against the `relevant` set (a duplicate relevant key at two positions counts
    as relevant at both positions).
  - `contextual_recall`: the effective top-k is treated as a **set of distinct
    keys** for the intersection — duplicates do **not** inflate recall.
  - `ndcg_at_k`: a relevant key contributes to `DCG` **only at its first
    (highest-ranked) occurrence**; later duplicate occurrences of the same key
    contribute `rel = 0`. This is what keeps `nDCG ≤ 1` for arbitrary (incl.
    duplicate) inputs.

---

## 5. Function signatures + return shape + fail pattern

### 5.1 `contextual_precision`

```rust
pub fn contextual_precision(
    results: &[RagResultItem],
    relevant: &HashSet<(DocumentId, NodeId)>,
    k: usize,
) -> f64
```

**Definition.** Let `R` = the set of 1-based ranks `i ∈ {1..=effective_top}` such
that the item at rank `i` (0-based index `i-1`) is relevant. Let
`precision@i = (# relevant items in the top i) / i`. Then

```
CP = (1 / |R|) · Σ_{i ∈ R} precision@i
```

**Return shape.** A single `f64` in `[0, 1]`. `|R| = 0` (no relevant item in the
effective top) → `0.0`.

**Fail pattern.** None — the fn is total and pure. It never panics, never returns
`NaN`/`inf`, and is always in `[0, 1]`.

### 5.2 `contextual_recall`

```rust
pub fn contextual_recall(
    results: &[RagResultItem],
    relevant: &HashSet<(DocumentId, NodeId)>,
    k: usize,
) -> f64
```

**Definition.** Let `top_k` = the set of **distinct** keys among the first
`effective_top` items of `results`. Then

```
CR = |relevant ∩ top_k| / |relevant|
```

**Return shape.** A single `f64` in `[0, 1]`. Empty `relevant` → **`1.0`**
(vacuous, pinned convention). `|relevant| > 0` and empty intersection → `0.0`.

**Fail pattern.** None — total and pure. Never panics, never `NaN`/`inf`, always
in `[0, 1]`.

### 5.3 `ndcg_at_k`

```rust
pub fn ndcg_at_k(
    results: &[RagResultItem],
    relevant: &HashSet<(DocumentId, NodeId)>,
    k: usize,
) -> f64
```

**Definition.** For 1-based rank `i ∈ {1..=effective_top}`, let `rel_i = 1` if the
item at rank `i` is relevant **and** its key has not appeared at a higher rank
(first-occurrence rule, §4), else `rel_i = 0`. Then

```
DCG@k  = Σ_{i=1}^{effective_top} rel_i / log2(i + 1)
IDCG@k = Σ_{i=1}^{min(k, |relevant|)} 1 / log2(i + 1)
nDCG   = DCG@k / IDCG@k
```

**Return shape.** A single `f64` in `[0, 1]`. `IDCG@k = 0` (empty `relevant`) or
`k = 0` → `0.0`.

**Fail pattern.** None — total and pure. Never panics, never `NaN`/`inf`, always
in `[0, 1]`. The first-occurrence rule guarantees `DCG@k ≤ IDCG@k` for any input
(including duplicates), so `nDCG ≤ 1`.

**Boundary note.** `nDCG = 1` requires **all** relevant keys to be present in the
effective top-k (i.e. `|relevant| ≤ effective_top`) **and no duplicate relevant key
in the effective top** (under first-occurrence, a duplicate relevant key contributes
`rel = 0` and reduces `DCG@k` below `IDCG@k`, so the presence condition is
necessary but not sufficient). If `|relevant| > effective_top` (e.g.
`results.len() < |relevant|`), `nDCG < 1` even for a perfectly-ranked retrieval,
because `IDCG@k` uses `min(k, |relevant|)` while `DCG@k` is capped by the number of
positions actually present.

### 5.4 `mrr_at_k`

```rust
pub fn mrr_at_k(
    results: &[RagResultItem],
    relevant: &HashSet<(DocumentId, NodeId)>,
    k: usize,
) -> f64
```

**Definition.** Let `r` = the 1-based rank of the **first** relevant item in the
effective top-k (the lowest 0-based index `i` with `results[i]` relevant, rank
`i + 1`). Then

```
MRR = 1 / r   if such an item exists
MRR = 0.0     otherwise (no relevant item in the effective top, or k = 0)
```

**Return shape.** A single `f64` in `[0, 1]`. No relevant item in the effective
top → `0.0`. `k = 0` → `0.0`.

**Fail pattern.** None — total and pure. Never panics, never `NaN`/`inf`, always
in `[0, 1]`.

---

## 6. Documented valid/happy + fail states per metric fn (TestWriter assertion guide)

Hand-computable reference values use the pinned conventions (§4). In every example
below, `results` is a list of `RagResultItem` whose keys are denoted by letters
(`A`, `B`, …) and `relevant` is the `HashSet` of the listed keys.

**Assertion tolerance (pinned):** values marked `≈` are asserted with an **absolute
tolerance of `1e-4`** (i.e. `|actual − expected| ≤ 1e-4`); values marked exact
(`**1.0**`, `**0.0**`, etc.) are asserted with exact `f64` equality.

### 6.1 `contextual_precision`

| input | expected | derivation |
| --- | --- | --- |
| `results=[B,D]`, `relevant={B,D}`, `k=2` | **1.0** | rank1 B relevant → `precision@1=1`; rank2 D relevant → `precision@2=2/2=1`; `R={1,2}`, `CP=(1/2)(1+1)=1.0` |
| `results=[A,B,C,D]`, `relevant={B,D}`, `k=4` | **0.5** | rank2 B relevant → `precision@2=1/2=0.5`; rank4 D relevant → `precision@4=2/4=0.5`; `R={2,4}`, `CP=(1/2)(0.5+0.5)=0.5` |
| `results=[B,A,C]`, `relevant={B}`, `k=3` | **1.0** | rank1 B relevant → `precision@1=1`; `R={1}`, `CP=1.0` |
| `results=[A,B,C]`, `relevant={B}`, `k=3` | **0.5** | rank2 B relevant → `precision@2=1/2=0.5`; `R={2}`, `CP=0.5` |
| `results=[B,A,C,D]`, `relevant={B,D}`, `k=2` | **1.0** | effective top `[B,A]`; rank1 B relevant → `precision@1=1`; `R={1}`, `CP=1.0` (D beyond top-k ignored) |
| `results=[B,D]`, `relevant={B,D}`, `k=10` | **1.0** | `effective_top=2`; same as the `k=2` case |
| `results=[B,A]`, `relevant={B}`, `k=1` | **1.0** | effective top `[B]`; rank1 relevant → `CP=1.0` |
| `results=[A,B]`, `relevant={B}`, `k=1` | **0.0** | effective top `[A]`; no relevant → `R` empty → `0.0` |
| `results=[A,B,C]`, `relevant={D}`, `k=3` | **0.0** | no relevant retrieved → `0.0` |
| `results=[A,B,C]`, `relevant={}`, `k=3` | **0.0** | empty relevant → `0.0` |
| `results=[]`, `relevant={B}`, `k=3` | **0.0** | empty results → `0.0` |
| `results=[B,A]`, `relevant={B}`, `k=0` | **0.0** | `k=0` → `0.0` |
| `results=[B,B]`, `relevant={B}`, `k=2` | **1.0** | rank1 B relevant → `precision@1=1`; rank2 B relevant → `precision@2=2/2=1`; `R={1,2}`, `CP=1.0` (duplicate counts at both positions) |
| `results=[A,C]`, `relevant={B}`, `k=2` | **0.0** | relevant not in results → `0.0` |
| `results=[B,A,C]`, `relevant={B,C}`, `k=1` | **1.0** | effective top `[B]`; rank1 relevant → `CP=1.0` (C beyond top-k ignored) |

### 6.2 `contextual_recall`

| input | expected | derivation |
| --- | --- | --- |
| `results=[B,D]`, `relevant={B,D}`, `k=2` | **1.0** | `top_k={B,D}`, `|∩|=2`, `|relevant|=2` → `1.0` |
| `results=[B,A,C]`, `relevant={B,D}`, `k=3` | **0.5** | `top_k={B,A,C}`, `|∩|=1`, `|relevant|=2` → `0.5` |
| `results=[B,A,C,D]`, `relevant={B,D}`, `k=1` | **0.5** | effective top `[B]`, `top_k={B}`, `|∩|=1`, `|relevant|=2` → `0.5` |
| `results=[B]`, `relevant={B,D}`, `k=10` | **0.5** | `effective_top=1`, `top_k={B}`, `|∩|=1`, `|relevant|=2` → `0.5` |
| `results=[B,A]`, `relevant={B}`, `k=1` | **1.0** | effective top `[B]`, `|∩|=1`, `|relevant|=1` → `1.0` |
| `results=[A,B]`, `relevant={B}`, `k=1` | **0.0** | effective top `[A]`, `|∩|=0` → `0.0` |
| `results=[A,B]`, `relevant={C}`, `k=2` | **0.0** | no relevant retrieved → `0.0` |
| `results=[A,B]`, `relevant={}`, `k=2` | **1.0** | empty relevant → vacuous `1.0` |
| `results=[]`, `relevant={B}`, `k=3` | **0.0** | empty results → `0.0` |
| `results=[B,A]`, `relevant={B}`, `k=0` | **0.0** | `k=0` → `0.0` |
| `results=[B,B]`, `relevant={B}`, `k=2` | **1.0** | `top_k={B}` (set), `|∩|=1`, `|relevant|=1` → `1.0` (duplicates do not inflate) |
| `results=[A,C]`, `relevant={B}`, `k=2` | **0.0** | relevant not in results → `0.0` |
| `results=[B,A,C]`, `relevant={B,C}`, `k=1` | **0.5** | effective top `[B]`, `top_k={B}`, `|∩|=1`, `|relevant|=2` → `0.5` |

### 6.3 `ndcg_at_k`

| input | expected | derivation |
| --- | --- | --- |
| `results=[B,D]`, `relevant={B,D}`, `k=2` | **1.0** | `DCG=1/log2(2)+1/log2(3)=1+0.6309=1.6309`; `IDCG=Σ_{i=1}^{min(2,2)}=1+0.6309=1.6309`; `nDCG=1.0` |
| `results=[A,B,C,D]`, `relevant={B,D}`, `k=4` | **≈0.6510** | `DCG=1/log2(3)+1/log2(5)=0.6309+0.4307=1.0616`; `IDCG=1+0.6309=1.6309`; `nDCG=1.0616/1.6309≈0.6510` |
| `results=[B,A,C]`, `relevant={B}`, `k=3` | **1.0** | `DCG=1/log2(2)=1`; `IDCG=Σ_{i=1}^{min(3,1)}=1`; `nDCG=1.0` |
| `results=[A,B,C]`, `relevant={B}`, `k=3` | **≈0.6309** | `DCG=1/log2(3)=0.6309`; `IDCG=1`; `nDCG≈0.6309` |
| `results=[B,A,C,D]`, `relevant={B,D}`, `k=2` | **≈0.6131** | effective top `[B,A]`; `DCG=1/log2(2)=1`; `IDCG=Σ_{i=1}^{min(2,2)}=1+0.6309=1.6309`; `nDCG=1/1.6309≈0.6131` |
| `results=[B,D]`, `relevant={B,D}`, `k=10` | **1.0** | `effective_top=2`; `DCG=IDCG=1.6309` → `1.0` |
| `results=[B,A]`, `relevant={B}`, `k=1` | **1.0** | effective top `[B]`; `DCG=1`; `IDCG=1` → `1.0` |
| `results=[A,B]`, `relevant={B}`, `k=1` | **0.0** | effective top `[A]`; `DCG=0`; `IDCG=1` → `0.0` |
| `results=[A,B]`, `relevant={C}`, `k=2` | **0.0** | `DCG=0`; `IDCG=1` → `0.0` |
| `results=[A,B]`, `relevant={}`, `k=2` | **0.0** | empty relevant → `IDCG=0` → `0.0` |
| `results=[]`, `relevant={B}`, `k=3` | **0.0** | `DCG=0`; `IDCG=Σ_{i=1}^{min(3,1)}=1` → `0.0` |
| `results=[B,A]`, `relevant={B}`, `k=0` | **0.0** | `k=0` → `0.0` |
| `results=[B,B]`, `relevant={B}`, `k=2` | **1.0** | first-occurrence: `DCG=1/log2(2)=1` (second B contributes `rel=0`); `IDCG=1` → `1.0` |
| `results=[B]`, `relevant={B,D}`, `k=10` | **≈0.6131** | `DCG=1`; `IDCG=Σ_{i=1}^{min(10,2)}=1+0.6309=1.6309`; `nDCG=1/1.6309≈0.6131` (D relevant but not retrieved) |

### 6.4 `mrr_at_k`

| input | expected | derivation |
| --- | --- | --- |
| `results=[B,A,C]`, `relevant={B}`, `k=3` | **1.0** | first relevant at rank 1 → `1/1=1.0` |
| `results=[A,B,C]`, `relevant={B}`, `k=3` | **0.5** | first relevant at rank 2 → `1/2=0.5` |
| `results=[A,C,B]`, `relevant={B}`, `k=3` | **≈0.3333** | first relevant at rank 3 → `1/3≈0.3333` |
| `results=[B,A,C,D]`, `relevant={D}`, `k=2` | **0.0** | effective top `[B,A]`; D beyond top-k → no relevant → `0.0` |
| `results=[A,B]`, `relevant={C}`, `k=2` | **0.0** | no relevant retrieved → `0.0` |
| `results=[A,B]`, `relevant={}`, `k=2` | **0.0** | empty relevant → `0.0` |
| `results=[]`, `relevant={B}`, `k=3` | **0.0** | empty results → `0.0` |
| `results=[B,A]`, `relevant={B}`, `k=0` | **0.0** | `k=0` → `0.0` |
| `results=[B,B]`, `relevant={B}`, `k=2` | **1.0** | first relevant at rank 1 → `1.0` |
| `results=[A,C]`, `relevant={B}`, `k=2` | **0.0** | relevant not in results → `0.0` |

**Cross-cutting TestWriter states (all four fns):**
- **`k > len`:** `effective_top = results.len()`; identical to `k = results.len()`.
- **`k = len`:** `effective_top = results.len()`; the full list is evaluated.
- **`k = 1`:** only the first item is considered.
- **Relevant beyond top-k:** ignored (never counted).
- **Relevant not in results:** never counted.
- **Duplicates:** per §4 (precision/MRR count each position; recall uses set
  semantics; nDCG uses first-occurrence).
- **No panic / no NaN:** every metric returns a finite `f64` in `[0, 1]` for every
  input combination above and for arbitrary generated inputs.

---

## 7. The `gnosis-eval` `[[bin]]` interface

### 7.1 CLI

```
gnosis-eval [--live] <corpus.json>
```

- `<corpus.json>` — path to a labeled corpus file matching the §8 schema.
- `--live` — use the **real** embedding provider for `vector`/`hybrid` cases
  (live data acquisition). **Default (no flag):** the **deterministic fake
  embedding provider** — reproducible in CI, no remote call.
- Equivalent env override: `GNOSIS_EVAL_LIVE=1` selects the real provider;
  `GNOSIS_EVAL_LIVE=0` or unset selects the fake provider. The flag and env are
  OR'd (flag wins if both present).

### 7.2 Behavior

1. Parse `<corpus.json>` into the §8 schema. A parse/shape error → print an error
   to stderr and exit non-zero. The `mode` field is parsed as a **lowercase string**
   and mapped to the `QueryMode` variant **manually** (see §8's `mode` serde note —
   it is **not** deserialized directly into `QueryMode`).
2. Seed a `Store` such that each case's labeled relevant nodes are retrievable by
   its `query` under its `mode` (the exact seeding is the Implementer's choice,
   but it must make the relevant set reachable so the metrics are meaningful). The
   store is built with the selected provider (fake or real) and the indexes the
   case's `mode` requires.
3. For each case, run `rag_query(query, {wiki_id, mode, top_k})` via the `RagStore`
   seam. A `rag_query` error → print the error for that case and continue (the
   case contributes no metric values; it is reported as `error`).
4. For each case that produced a `RagResult`, compute the four metrics over
   `result.results` and the case's `relevant` set with `k = top_k`.
5. Print the per-case + aggregate report (§7.3).
6. Exit `0` if every case ran (even if some cases errored); exit non-zero on a
   hard failure (corpus parse error, store-seeding error).

### 7.3 Report output shape

```
=== gnosis-eval report ===
case 1: query="<query>", wiki="<wiki_id>", mode=<flat|graph|vector|hybrid>, top_k=<n>
  contextual_precision = 0.5000
  contextual_recall    = 0.6667
  ndcg_at_k            = 0.6510
  mrr_at_k             = 0.5000
case 2: ... (or: case 2: ERROR <rag_query error>)
...
aggregate (mean over N cases):
  contextual_precision = 0.xxxx
  contextual_recall    = 0.xxxx
  ndcg_at_k            = 0.xxxx
  mrr_at_k             = 0.xxxx
```

- Per-case metric values are printed to 4 decimal places.
- The aggregate is the **arithmetic mean** of the per-case values over the cases
  that produced a `RagResult` (errored cases are excluded from the aggregate and
  counted separately).
- The exact spacing/format is not contract-critical beyond the field names and the
  per-case + aggregate structure; the TestWriter's harness smoke asserts the
  presence of the four metric field names and the aggregate block, not byte-exact
  formatting.

---

## 8. The labeled-corpus fixture `tests/fixtures/eval_corpus.json`

**Schema** (a JSON array of case objects):

```json
[
  {
    "query": "string (non-empty)",
    "wiki_id": "string",
    "mode": "flat" | "graph" | "vector" | "hybrid",
    "top_k": "integer, 1..=50",
    "relevant": [["doc-id", "node-id"], "..."]
  }
]
```

**`mode` serde note (pinned):** the corpus uses **lowercase** mode strings
(`"flat"`, `"graph"`, `"vector"`, `"hybrid"`), but `QueryMode` has **no
`#[serde(rename_all)]`**, so its serde representation is **PascalCase**
(`"Flat"`, `"Graph"`, `"Vector"`, `"Hybrid"`). The bin must **not** deserialize
`mode` directly into `QueryMode`; it parses the lowercase string and maps it to the
`QueryMode` variant **manually** (a `match` on the lowercase string). The TestWriter
must **not** assert that `serde_json::from_str::<QueryMode>("\"flat\"")` succeeds.

- `query` — the retrieval query text (non-empty).
- `wiki_id` — the wiki the case runs against.
- `mode` — one of the four `QueryMode` values.
- `top_k` — the `top_k` passed to `rag_query` and used as `k` for the metrics.
- `relevant` — the hand-authored ground-truth relevant-node set as an array of
  `[document_id, node_id]` string pairs.

**Fixture requirements (pinned):**
- Hand-authored ground truth (query → relevant node set), **not** a real
  benchmark.
- Spans **multiple modes** (at least `flat` and `graph`; `vector`/`hybrid` cases
  are present and run under the fake provider in CI).
- Contains **at least one empty-`relevant` case** (`"relevant": []`) to pin the
  empty-set conventions (§4) end-to-end through the bin.
- Contains at least one case where the relevant set is **not** fully retrieved
  (so the metrics are not all trivially 1.0).

---

## 9. Cross-references and ownership hand-off

- **Contract authority:** `docs/specs/gnosis.md` §7.8 (the gap), §4.5.1 (query
  modes), §4.6.1 (`ragQuery` surface + `RagResult`), §4.3.4 (audit log — the
  harness does **NOT** use it).
- **Gate record + scope guardrails:** `docs/specs/6-f6-eval-review.md`
  (`F6-EVAL-RE-SCOPED`); decision row `docs/decisions.md` `F6-EVAL-RE-SCOPED`.
- **PBT:** `docs/specs/6-f6-eval-property-register.md` (+ TestWriter's
  `tests/props_eval.rs`).
- **Conformance tests (TestWriter):** `tests/eval_harness.rs`.

**Suite-side (NOT Gnosis, per the review):**
1. The DeepEval Python harness.
2. Answer generation (the shell/LLM's job) — the engine produces no generated
   answer.
3. LLM-as-judge metrics — faithfulness, answer-relevancy.
4. Any user-facing retrieval-quality surface (D4 parity applies at the
   Astrographer shell, not the headless Gnosis).

**Doc reconciliation (from the review's residual risks):** the §7.8 wording that
the audit log "supplies the raw material" is the red herring the re-scope removes;
the corrected wording is that the harness **re-runs `rag_query` against a READY
engine**. This spec pins that corrected wording; the §7.8/`docs/next-steps.md`/
`docs/pending.md` reconciliation was completed by the **2026-09-09 F6 doc-review
pass** (`archive/reviews/2026-09-09-f6-eval-doc-review.md`).

---

## 10. What the TestWriter derives (red-set readiness)

- `eval_harness.rs`: the §6 hand-computed reference values for all four metrics
  (each row of §6.1–§6.4 as an exact/`≈` assertion — the `≈` values are asserted
  with the pinned absolute tolerance of `1e-4`, §6), plus the empty-set
  conventions (§4), the `k` boundaries (`k=0`, `k=1`, `k=len`, `k>len`), the
  duplicate-key conventions, and a harness smoke that runs the `gnosis-eval` bin
  against `tests/fixtures/eval_corpus.json` and asserts the report contains the
  four metric field names + the aggregate block.
- `props_eval.rs`: the 8 invariant rows of the PBT register
  (`docs/specs/6-f6-eval-property-register.md`), one `#[test]` per row, under the
  PBT-gate contract (deterministic pinned seed, ≤100 cases/row, ≤400 total,
  stop-after-5).
