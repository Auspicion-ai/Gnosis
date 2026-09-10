# §7.8 F6 RAG evaluation — Typed Property Register (PBT gate)

- **Unit:** §7.8 F6 RAG evaluation — `src/retrieval/eval.rs`. **Spec:**
  `docs/specs/6-f6-eval-spec.md` (the F6 behavior contract); the metric semantics
  are pinned in §4–§6 of that spec.
- **Scope/API:** the four pure deterministic metric fns
  `contextual_precision`/`contextual_recall`/`ndcg_at_k`/`mrr_at_k` over
  `&[RagResultItem]` + `&HashSet<(DocumentId, NodeId)>` + `k: usize`, re-exported
  from `src/lib.rs` (`pub use self::retrieval::{contextual_precision,
  contextual_recall, ndcg_at_k, mrr_at_k};`). Types reached as
  `gnosis::{RagResultItem, DocumentId, NodeId}`.
- **Exercised by:** the F6 test-suite — `tests/eval_harness.rs` (metric math +
  harness smoke) and `tests/props_eval.rs` (this register's executed layer).
- **Date:** 2026-09-09. **Role:** spec_writer (this register is the PBT-gate
  artifact **#1** of 3).
- **Status:** PBT-gate artifact #1 — the typed property register. Artifacts #2
  (executed property layer, TestWriter) and #3 (read-only PBT audit, adversarial
  reviewer) are separate passes.

## PBT-gate note

This register is the contract for the **property-based-testing gate** on the F6
metric unit. The TestWriter's executed layer runs under `cargo test` with a
**deterministic pinned seed**, **≤100 generated cases per register row**,
**≤400 total cases** across the unit's whole property layer, **stop-after-5**
(report ≤5 distinct held/broken counterexamples per row), and records each row as
**held** or **broken** together with its `Strategy-id`. The adversarial reviewer
then reads this register with the executed artifacts and performs a **read-only**
PBT audit (per-row over-strength reasoning, generator-coverage check, prose
counterexamples, negative-generator requests); reviewers never run generators.

Authoring rule: `Property-id` = `P-<CLASS>-<N>`, `CLASS ∈ {IM, SM, TP}`, **≤ 8
rows**; invariants are **crisp, universally-quantified, black-box-observable
through the crate's public metric surface only**, and are **TRUE of the current
GREEN implementation** (implied by the metric definitions in the F6 spec — they are
**not** pending features). **There are no fail-state rows** (no `§6`/`FS-*` rows)
and **no gap/parked rows** — invariants only.

**Reserved-variant discipline applied.** The metric fns are **pure and total** —
they take any `&[RagResultItem]`, any `&HashSet<(DocumentId, NodeId)>`, and any
`k: usize`, and return a finite `f64` in `[0, 1]`; they never panic and never
return `NaN`/`inf`. There is **no throw-path** to reserve: the fns have no
`Result`, no runtime, no `async`, and no fail-state. Every row below is therefore
an invariant over the pure metric surface, and the generator may freely produce
arbitrary (incl. empty, `k=0`, `k>len`, duplicate-key) inputs. **One exception:**
`P-SM-1`'s generator produces **distinct-key `results` only** (per that row's
scoping note — the nDCG monotonicity leg is valid only for distinct-key inputs).
The `gnosis-eval`
`[[bin]]`'s `rag_query` fail-states (e.g. `EngineUnavailable`,
`EmbeddingUnavailable`, `VectorIndexUnavailable`) are **out of scope** for this
register — no row exercises the bin's live data-acquisition path; the rows assert
the pure metric fns only.

## Property table

| Property-id | Class | Invariant | Strategy-id | Observable-as-property |
|---|---|---|---|---|
| `P-IM-1` | IM | **Boundedness.** For any `results: &[RagResultItem]`, any `relevant: &HashSet<(DocumentId, NodeId)>`, and any `k: usize`, each of `contextual_precision`, `contextual_recall`, `ndcg_at_k`, `mrr_at_k` returns a **finite `f64` in `[0, 1]`** — never `NaN`, never `inf`, never negative, never `> 1`. | `strat:metric-bounds` | ∀ generated `results`/`relevant`/`k`: `p = contextual_precision(&results, &relevant, k)`, `r = contextual_recall(&results, &relevant, k)`, `n = ndcg_at_k(&results, &relevant, k)`, `m = mrr_at_k(&results, &relevant, k)` — each of `p`,`r`,`n`,`m` is finite and `0.0 <= v <= 1.0`. |
| `P-IM-2` | IM | **Determinism (pure function).** Repeating each metric on the **same** `results`/`relevant`/`k` yields the **identical `f64`** value on every call. | `strat:metric-repeat` | For generated `results`/`relevant`/`k`: `contextual_precision(&results,&relevant,k) == contextual_precision(&results,&relevant,k)` (and likewise for the other three) — element-wise `==` on a repeat. |
| `P-IM-3` | IM | **Empty-result handling.** With `results` empty and `relevant` non-empty, all four metrics return **`0.0`** (the pinned convention) — no panic, no `NaN`. | `strat:empty-results` | For generated non-empty `relevant` and any `k`: `contextual_precision(&[], &relevant, k) == 0.0`, `contextual_recall(&[], &relevant, k) == 0.0`, `ndcg_at_k(&[], &relevant, k) == 0.0`, `mrr_at_k(&[], &relevant, k) == 0.0`. |
| `P-IM-4` | IM | **Empty-relevant-set handling.** With `relevant` empty, all four metrics are **defined** (no panic, no `NaN`) and return the pinned values: precision `0.0`, recall **`1.0`** (vacuous), nDCG `0.0`, MRR `0.0`. | `strat:empty-relevant` | For generated `results` and any `k`: `contextual_precision(&results, &HashSet::new(), k) == 0.0`, `contextual_recall(&results, &HashSet::new(), k) == 1.0`, `ndcg_at_k(&results, &HashSet::new(), k) == 0.0`, `mrr_at_k(&results, &HashSet::new(), k) == 0.0`. |
| `P-SM-1` | SM | **Rank-improvement monotonicity (distinct-key `results`).** Moving a relevant item to a **higher rank** (a lower 0-based index) in `results` does **not decrease** any of precision@k / recall@k / nDCG@k / MRR@k. **Scoped to `results` with distinct keys** (no `(document_id, node_id)` key appears twice): under the first-occurrence nDCG convention, nDCG monotonicity is **provably strict only for distinct-key inputs** — a duplicate relevant key moved up contributes `rel = 0` and can push other relevant keys down, decreasing nDCG (counterexample: `results=[A,B,C,D,B]`, `relevant={B,C,D}`, `k=5` → nDCG 0.7328; move the rank-5 duplicate B to rank 3 → `[A,B,B,C,D]` → nDCG 0.6798). The precision/recall/MRR legs hold for arbitrary inputs; the nDCG leg is scoped to distinct-key `results`. | `strat:rank-improve` | For generated `results`/`relevant`/`k` with at least one relevant item present and **`results` containing distinct keys only**, construct `results'` by moving one relevant item to a strictly higher rank (all other items' relative order preserved): `metric(results', relevant, k) >= metric(results, relevant, k)` for each of the four metrics. |
| `P-SM-2` | SM | **Recall@k monotonic in `k`; precision@k NOT monotonic.** `contextual_recall` is **non-decreasing** in `k` (a larger `k` never lowers recall). `contextual_precision` is **not** guaranteed monotonic in `k` — the generator must confirm the property only claims the recall direction, and must not assert a precision direction. | `strat:k-monotone` | For generated `results`/`relevant` and `k1 < k2`: `contextual_recall(&results, &relevant, k2) >= contextual_recall(&results, &relevant, k1)`. The generator additionally samples precision at `k1`/`k2` and asserts **no** universal precision direction is claimed (a counterexample where precision decreases with `k` is expected and is **not** a broken row). |
| `P-TP-1` | TP | **Perfect-retrieval bound.** When the effective top-k is **exactly** the relevant set, correctly ranked (all relevant keys present, `|relevant| <= effective_top`, no non-relevant item in the top), then precision@k = recall@k = nDCG@k = **`1.0`**, and MRR@k = **`1.0`** if the relevant set is non-empty. | `strat:perfect-retrieval` | For generated non-empty `relevant` and `k` with `|relevant| <= min(k, results.len())`, build `results` = the relevant keys in any order (all relevant, no non-relevant): `contextual_precision == 1.0`, `contextual_recall == 1.0`, `ndcg_at_k == 1.0`, `mrr_at_k == 1.0`. |
| `P-TP-2` | TP | **No-relevant-retrieved bound.** When no retrieved item is relevant (the effective top-k contains no key from `relevant`), then precision@k = recall@k = nDCG@k = MRR@k = **`0.0`**. | `strat:no-relevant` | For generated `results`/`relevant`/`k` where `relevant` is non-empty and **disjoint** from the effective top-k's key set: `contextual_precision == 0.0`, `contextual_recall == 0.0`, `ndcg_at_k == 0.0`, `mrr_at_k == 0.0`. |

**Class tally:** IM ×4, SM ×2, TP ×2 = **8 rows ≤ 8** ✔.

## Generator-coverage note (per row — boundary + adversarial input shapes)

- **`P-IM-1 — strat:metric-bounds`.** Generate arbitrary `results` (empty; single
  item; many items; all-relevant; no-relevant; mixed; **duplicate keys** at
  multiple positions) and arbitrary `relevant` (empty; non-empty; overlapping;
  disjoint; keys not present in `results`) and `k ∈ {0, 1, mid, results.len(),
  results.len()+1, large}`. Assert every metric is finite and in `[0, 1]` — this is
  the row that pins the duplicate-key conventions (§4 of the spec: precision/MRR
  count each position, recall uses set semantics, nDCG uses first-occurrence) so
  `nDCG <= 1` holds even with duplicates.
- **`P-IM-2 — strat:metric-repeat`.** Re-run each metric twice on the same inputs
  (including empty `results`, empty `relevant`, `k=0`, duplicate-key inputs) and
  assert bit-identical `f64` equality. The fns are pure, so this must always hold.
- **`P-IM-3 — strat:empty-results`.** `results = []` with non-empty `relevant` and
  `k ∈ {0, 1, 5, 50}`. Assert all four metrics are `0.0`. This pins the empty-result
  convention (§4).
- **`P-IM-4 — strat:empty-relevant`.** `relevant = HashSet::new()` with arbitrary
  `results` (empty and non-empty) and `k ∈ {0, 1, 5, 50}`. Assert precision `0.0`,
  recall `1.0` (vacuous), nDCG `0.0`, MRR `0.0`. This pins the empty-relevant
  convention (§4) — the recall `1.0` is the judgment call the review flagged.
- **`P-SM-1 — strat:rank-improve`.** Start from a `results` with at least one
  relevant item and **distinct keys only** — the generator must **avoid duplicate
  keys** in this row (per the row's scoping note; a duplicate relevant key moved up
  would contribute `rel = 0` and could decrease nDCG, so duplicates are excluded).
  Move a relevant item to a strictly higher rank (lower index), preserving the
  relative order of all other items. Include cases where the moved item crosses a
  non-relevant item, and where it moves to rank 1. Assert each of the four metrics
  is non-decreasing. (Recall is unaffected by rank — it is non-decreasing trivially;
  precision/nDCG/MRR are the discriminating checks. The nDCG leg is valid only
  because `results` is distinct-key.)
- **`P-SM-2 — strat:k-monotone`.** For `k1 < k2` (e.g. `0 < 1`, `1 < 5`, `5 < 50`),
  assert `contextual_recall(k2) >= contextual_recall(k1)`. The generator must also
  sample precision at both `k` and confirm a precision **decrease** is possible
  (e.g. `results=[B,A,C,D]`, `relevant={B,D}`, `k1=2` → effective top `[B,A]`,
  rank1 B relevant → `precision@1=1`, `R={1}`, `CP=1.0`; `k2=4` → effective top
  `[B,A,C,D]`, rank1 B relevant `precision@1=1`, rank4 D relevant
  `precision@4=2/4=0.5`, `R={1,4}`, `CP=(1/2)(1+0.5)=0.75` — precision **decreased**
  from `1.0` to `0.75`) — that decrease is **expected** and must **not** fail the
  row; the row asserts only the recall direction.
- **`P-TP-1 — strat:perfect-retrieval`.** Build `results` = exactly the relevant
  keys (in any order, all relevant, no non-relevant) with `|relevant| <= min(k,
  results.len())`. Include `|relevant| = 1`, `|relevant| = effective_top`, and
  `k` larger than `results.len()`. Assert precision/recall/nDCG = `1.0` and MRR =
  `1.0`. Do **not** include a case where `|relevant| > effective_top` (nDCG < 1
  there, per the spec's boundary note — that is a different state, not this row).
- **`P-TP-2 — strat:no-relevant`.** Build `results` whose effective top-k keys are
  **disjoint** from a non-empty `relevant` (e.g. `relevant` keys not in `results`,
  or only beyond the effective top-k). Include `k=0` (effective top empty → all
  `0.0`), `k=1`, and `k` larger than `results.len()`. Assert all four metrics are
  `0.0`.

## API notes for the TestWriter

- **Pure synchronous helpers.** The four metric fns are `pub` in
  `src/retrieval/eval.rs` and **re-exported from `src/lib.rs`** (`pub use
  self::retrieval::{contextual_precision, contextual_recall, ndcg_at_k,
  mrr_at_k};`). They take `&[RagResultItem]` and `&HashSet<(DocumentId, NodeId)>`
  and return `f64` directly — **no `Result`, no runtime, no `async`**. Every row
  asserts them directly and synchronously (`#[test]`, no runtime needed).
- **Constructing inputs.** `RagResultItem` is `{document_id: DocumentId, node_id:
  NodeId, score: f64, snippet: String, source: Source, parent: Option<RagParent>,
  stale: Option<bool>}`; only `document_id`/`node_id` matter to the metrics, so the
  generator may fill the rest with defaults (`score: 0.0`, `snippet: String::new()`,
  `source: Source::Local`, `parent: None`, `stale: None`). `DocumentId(pub String)`
  and `NodeId(pub String)` are single-field `String` newtypes; the generator
  produces distinct keys by varying the inner strings. `relevant` is a
  `HashSet<(DocumentId, NodeId)>`.
- **Pinned conventions the generator must honor (from the spec §4).** 1-based
  rank (0-based index `i` → rank `i+1`); effective top = `min(k, results.len())`;
  `log2(i+1)` denominator; empty-set conventions (empty relevant → precision 0,
  recall 1, nDCG 0, MRR 0; empty results → all 0; `k=0` → all 0); duplicate-key
  conventions (precision/MRR count each position, recall uses set semantics, nDCG
  uses first-occurrence). The generator may freely produce empty/`k=0`/`k>len`/
  duplicate inputs — the fns are total and never panic.
- **Out of scope for this register.** The `gnosis-eval` `[[bin]]`'s live
  data-acquisition path (`rag_query` fail-states like `EngineUnavailable`/
  `EmbeddingUnavailable`/`VectorIndexUnavailable`) is **not** exercised by any
  row — the rows assert the pure metric fns only. The bin's report shape and the
  corpus fixture are pinned in the F6 spec §7–§8 and asserted by
  `tests/eval_harness.rs`, not by this register.
