# §7.8 F6 RAG evaluation — blind greens

Derived **from the documentation only** (`docs/specs/6-f6-eval-spec.md` §4–§8 and
`docs/specs/6-f6-eval-property-register.md`) by the Blind-Test Writer. Validation:
ran the built crate's integration suites (`CARGO_HOME="$PWD/.cargo-home" cargo
test --offline`, **387 tests green**, clippy/fmt clean) and ran an independent
throwaway `#[test]` (`tests/blind_eval_check.rs`, 17 assertions, since deleted)
that exercises the four metric fns through the public re-export surface
(`gnosis::{contextual_precision, contextual_recall, ndcg_at_k, mrr_at_k}`) plus a
live run of the `gnosis-eval` `[[bin]]` against
`tests/fixtures/eval_corpus.json`. **No behavior was verified by reading
`src/retrieval/eval.rs`, `src/bin/gnosis_eval.rs`, `tests/eval_harness.rs`, or
`tests/props_eval.rs`.**

Legend: `GREEN` = demonstrably exhibited by an independent check; `RED` = the
crate contradicts the docs; `NOT VERIFIED` = documented but not demonstrated.

## Metric happy paths (§6)

| Documented behavior | Expected observable result | Independent check | Status |
| --- | --- | --- | --- |
| all-relevant-at-top → precision/recall/nDCG/MRR all `1.0` | `results=[B,D]`, `relevant={B,D}`, `k=2` → all `1.0` | `assert_eq!` on all four fns == `1.0` | GREEN |
| `results=[B,D]`, `relevant={B,D}`, `k=2` → precision `1.0` | `CP=(1/2)(1+1)=1.0` | `assert_eq!(contextual_precision, 1.0)` | GREEN |
| `results=[B,D]`, `relevant={B,D}`, `k=2` → recall `1.0` | `top_k={B,D}`, `|∩|=2/2` | `assert_eq!(contextual_recall, 1.0)` | GREEN |
| `results=[B,D]`, `relevant={B,D}`, `k=2` → nDCG `1.0` | `DCG=IDCG=1.6309` | `assert_eq!(ndcg_at_k, 1.0)` | GREEN |
| `results=[B,D]`, `relevant={B,D}`, `k=2` → MRR `1.0` | first relevant at rank 1 | `assert_eq!(mrr_at_k, 1.0)` | GREEN |

## Metric fail-states (§6)

| Documented behavior | Expected observable result | Independent check | Status |
| --- | --- | --- | --- |
| no relevant retrieved → all `0.0` | `results=[A,B,C]`, `relevant={D}`, `k=3` → all `0.0` | `assert_eq!` on all four fns == `0.0` | GREEN |
| relevant not in results → all `0.0` | `results=[A,C]`, `relevant={B}`, `k=2` → all `0.0` | `assert_eq!` on all four fns == `0.0` | GREEN |

## Partial rankings — §6 reference values

| Documented behavior | Expected observable result | Independent check | Status |
| --- | --- | --- | --- |
| `results=[A,B,C,D]`, `relevant={B,D}`, `k=4` → precision `0.5` | `R={2,4}`, `CP=(1/2)(0.5+0.5)=0.5` | `assert_eq!(contextual_precision, 0.5)` | GREEN |
| `results=[A,B,C,D]`, `relevant={B,D}`, `k=4` → recall `1.0` | `top_k={A,B,C,D}`, `|∩|=2/2` | `assert_eq!(contextual_recall, 1.0)` | GREEN |
| `results=[A,B,C,D]`, `relevant={B,D}`, `k=4` → nDCG `≈0.6510` | `1.0616/1.6309≈0.6510` | `assert!(|ndcg_at_k − 0.6510| ≤ 1e-4)` | GREEN |
| `results=[A,B,C,D]`, `relevant={B,D}`, `k=4` → MRR `0.5` | first relevant at rank 2 → `1/2` | `assert_eq!(mrr_at_k, 0.5)` | GREEN |
| `results=[B,A,C,D]`, `relevant={B,D}`, `k=2` → nDCG `≈0.6131` | `1/1.6309≈0.6131` (D beyond top-k) | `assert!(|ndcg_at_k − 0.6131| ≤ 1e-4)` | GREEN |

## Boundaries (§4, §6 cross-cutting)

| Documented behavior | Expected observable result | Independent check | Status |
| --- | --- | --- | --- |
| `k=0` → all `0.0` | `results=[B,A]`, `relevant={B}`, `k=0` → all `0.0` | `assert_eq!` on all four fns == `0.0` | GREEN |
| `k>len` → `effective_top=len` | `results=[B,D]`, `relevant={B,D}`, `k=10` → all `1.0` | `assert_eq!` on all four fns == `1.0` | GREEN |
| `k=1` → only first item considered | `results=[B,A]`, `relevant={B}`, `k=1` → all `1.0`; `results=[A,B]`, `k=1` → all `0.0` | `assert_eq!` on all four fns | GREEN |
| `k=len` → full list evaluated | `results=[B,A,C,D]`, `relevant={B,D}`, `k=2` → precision `1.0`, recall `0.5`, nDCG `≈0.6131`, MRR `1.0` | `assert_eq!`/`assert!(≤1e-4)` | GREEN |
| relevant beyond top-k ignored | `results=[B,A,C,D]`, `relevant={B,D}`, `k=2` → D ignored | precision `1.0`, recall `0.5`, nDCG `≈0.6131`, MRR `1.0` | GREEN |
| relevant not in results never counted | `results=[A,C]`, `relevant={B}`, `k=2` → all `0.0` | `assert_eq!` on all four fns == `0.0` | GREEN |
| duplicates: precision/MRR count each position | `results=[B,B]`, `relevant={B}`, `k=2` → precision `1.0`, MRR `1.0` | `assert_eq!` | GREEN |
| duplicates: recall uses set semantics | `results=[B,B]`, `relevant={B}`, `k=2` → recall `1.0` (not inflated) | `assert_eq!(contextual_recall, 1.0)` | GREEN |
| duplicates: nDCG first-occurrence | `results=[B,B]`, `relevant={B}`, `k=2` → nDCG `1.0` (second B contributes `rel=0`) | `assert_eq!(ndcg_at_k, 1.0)` | GREEN |

## Empty-set conventions (§4)

| Documented behavior | Expected observable result | Independent check | Status |
| --- | --- | --- | --- |
| empty `relevant` → precision `0.0` | `results=[A,B]`, `relevant={}`, `k=2` | `assert_eq!(contextual_precision, 0.0)` | GREEN |
| empty `relevant` → recall `1.0` (vacuous) | `results=[A,B]`, `relevant={}`, `k=2` | `assert_eq!(contextual_recall, 1.0)` | GREEN |
| empty `relevant` → nDCG `0.0` | `results=[A,B]`, `relevant={}`, `k=2` | `assert_eq!(ndcg_at_k, 0.0)` | GREEN |
| empty `relevant` → MRR `0.0` | `results=[A,B]`, `relevant={}`, `k=2` | `assert_eq!(mrr_at_k, 0.0)` | GREEN |
| empty `results` (relevant non-empty) → all `0.0` | `results=[]`, `relevant={B}`, `k=3` | `assert_eq!` on all four fns == `0.0` | GREEN |
| `k=0` → all `0.0` | `results=[B,A]`, `relevant={B}`, `k=0` | `assert_eq!` on all four fns == `0.0` | GREEN |
| empty-`relevant` precedence over `k=0` for recall | `results=[B,A]`, `relevant={}`, `k=0` → recall `1.0` (vacuous), others `0.0` | `assert_eq!(contextual_recall, 1.0)`; others `0.0` | GREEN |

## The `gnosis-eval` `[[bin]]` (§7)

| Documented behavior | Expected observable result | Independent check | Status |
| --- | --- | --- | --- |
| `gnosis-eval <corpus.json>` runs the labeled corpus and exits `0` | all 5 fixture cases ran | `Command::new(env!("CARGO_BIN_EXE_gnosis-eval")).arg(corpus)` → `status.success()` | GREEN |
| report contains the four metric field names | `contextual_precision`, `contextual_recall`, `ndcg_at_k`, `mrr_at_k` | `stdout.contains(field)` for each | GREEN |
| report contains the aggregate block | `aggregate (mean over N cases)` | `stdout.contains("aggregate")` | GREEN |
| report contains per-case structure | `case 1: …` | `stdout.contains("case 1")` | GREEN |
| empty-`relevant` case surfaces recall `1.0` end-to-end | fixture case 5 (`"relevant": []`) → recall `1.0000` | live run: `case 5 … contextual_recall = 1.0000` | GREEN |
| per-case values printed to 4 decimals | `0.5000`-style | live run output | GREEN |

## Corpus `mode` serde note (§8)

| Documented behavior | Expected observable result | Independent check | Status |
| --- | --- | --- | --- |
| `QueryMode` has no `#[serde(rename_all)]` → PascalCase serde | `serde_json::from_str::<QueryMode>("\"flat\"")` **fails**; `"Flat"` succeeds | `assert!(from_str::<QueryMode>("\"flat\"").is_err())`; `assert!(from_str::<QueryMode>("\"Flat\"").is_ok())` | GREEN |

## NOT VERIFIED — §7.8 F6

1. **`--live` real-provider path** (§7.1): the live embedding-provider data
   acquisition is not exercised (no live Ollama in CI); only the default
   deterministic fake provider was run. The `GNOSIS_EVAL_LIVE` env override and
   the flag/env OR semantics were not independently driven.
   **→ NOT VERIFIED** (documented; requires a live provider, out of the
   deterministic blind-check scope).
2. **`rag_query` per-case error handling** (§7.2 step 3 — a case that errors is
   reported as `ERROR` and excluded from the aggregate): no fixture case errors,
   so the error-reporting branch was not observed.
   **→ NOT VERIFIED** (documented; no erroring case in the fixture).
3. **Non-zero exit on hard failure** (§7.2 step 6 — corpus parse / store-seeding
   error): not driven with a malformed corpus.
   **→ NOT VERIFIED** (documented; would require a deliberately malformed
   fixture).
