# F6 — RAG evaluation harness — PROPOSAL-REVIEW record

- **Unit:** §7.8 F6 — RAG evaluation harness (SHOULD-HAVE dev/QA gate).
- **Gate:** proposal-review gate (validity ∥ critique → architecture → change-analysis).
- **Date:** 2026-09-09.
- **Verdict:** **PASS — RE-SCOPED, delegable** (contingent on the user's go-ahead). The as-stated F6 (audit-log-fed, faithfulness/relevancy, DeepEval-in-Rust) is **INVALID**; the re-scoped F6 is a valid, delegable Gnosis-repo unit.
- **Status:** proposal approved by review (re-scoped); code/contract delegation awaits the user's go-ahead.

## What F6 is (as stated) and why it was re-scoped

§7.8 frames F6 as an offline RAG-evaluation harness (faithfulness / answer-relevancy /
contextual precision-recall, DeepEval-style, local-first) over the §4.3.4 query-audit-log
material. The proposal-review found the as-stated framing **invalid** on three facts:

1. **The audit log records only counts, not retrieved results.** `QueryAuditEntry`
   (`src/store/mod.rs:465`) is `{query, filters, mode, result_count, timestamp,
   requester}`; `rag_query` appends only `result_count` and discards the `RagResult`
   nodes/snippets/ranks. It is also in-memory/non-durable. Faithfulness/precision/recall
   need the retrieved nodes + ranks, which the audit log does not capture.
2. **The engine produces no generated answer.** `RagResult` carries retrieved
   nodes/snippets, no answer text. Faithfulness ("is the answer grounded in the
   retrieved nodes?") and answer-relevancy ("does the answer address the query?")
   require a generated answer, which is the shell/LLM's job, not Gnosis's.
3. **DeepEval is Python; Gnosis is Rust.** DeepEval's contextual precision/recall are
   LLM-as-judge metrics, not deterministic — a Rust in-repo metric must substitute
   ground-truth labels (query→relevant node set), a different metric family
   (nDCG/MRR@k-style).

## The re-scoped F6 (the approved deliverable)

A **deterministic, ground-truth-labeled contextual precision/recall module** — a
dev/QA gate, not a runtime contract element:

- **`src/retrieval/eval.rs`** — pure deterministic metric fns `contextual_precision`,
  `contextual_recall`, `ndcg_at_k`, `mrr_at_k` over `&[RagResultItem]` + a labeled
  relevant-node set, re-exported from `src/lib.rs`.
- **`src/bin/gnosis_eval.rs`** — a `[[bin]]` `gnosis-eval` that reads a labeled eval
  corpus (JSON fixture), seeds a `Store`, runs `rag_query` per case, computes the
  metrics, prints a per-case + aggregate report. Supports a deterministic fake
  embedding provider (reproducible CI) and a real provider (live vector/hybrid).
- **`tests/eval_harness.rs`** + **`tests/props_eval.rs`** — pins the metric math +
  harness smoke + the PBT layer.
- **`tests/fixtures/eval_corpus.json`** — the labeled corpus
  `[{query, wiki_id, mode, top_k, relevant: [[doc-id,node-id],…]}]`.

**Metric semantics (rank 1-based, position i → rank i+1, effective top = min(k, len)):**
- `contextual_precision` = `(1/|R|)·Σ_{i∈R} precision@i`; no relevant retrieved → 0.0.
- `contextual_recall` = `|relevant ∩ top-k| / |relevant|`; empty relevant → 1.0 (vacuous).
- `ndcg_at_k` = `DCG@k / IDCG@k`; IDCG=0 or k=0 → 0.0.
- `mrr_at_k` = `1/rank` of first relevant in top-k, else 0.0; k=0 → 0.0.

**Zero-runtime guardrail:** ZERO change to `QueryAuditEntry`/`rag_query`/`rag_stream`/
`get_query_audit_log` or any §4.3.4/§4.5/§4.6 surface; ZERO new runtime deps. The
harness re-runs `rag_query` against a READY engine (live data acquisition; deterministic
metric math offline) — it does **not** use the audit log.

**PBT gate applies** (PBT-GATE-MANDATORY): register pins P-IM-1 boundedness [0,1],
P-IM-2 determinism, P-IM-3 empty-result handling, P-IM-4 empty-relevant-set handling,
P-SM-1 rank-improvement monotonicity, P-SM-2 recall@k monotonic in k, P-TP-1
perfect-retrieval bound (=1), P-TP-2 no-relevant-retrieved bound (=0).

## Corrected unpark criterion

The prior criterion ("retrieval foundation shipped + audit log returns real entries")
was insufficient (the audit log returns only counts). The corrected sufficiency
criterion: (1) a **named artifact** (the eval module + `gnosis-eval` bin + test-suite);
(2) a **labeled eval set** (query→relevant-node fixture); (3) the **metric scope** pinned
(deterministic contextual precision/recall; faithfulness/relevancy out-of-repo);
(4) the **zero-runtime guardrail** stated. Met as a delegable unit now.

## Suite/dev-tool-side (NOT Gnosis)

The DeepEval Python harness, answer generation (the shell/LLM's job), the LLM-as-judge
metrics (faithfulness, answer-relevancy), and any user-facing retrieval-quality surface
(D4 parity applies at the Astrographer shell, not the headless Gnosis).

## Residual risks / open items (told to the user before approval)

- **Empty-relevant-set conventions are a judgment call** (recall 1.0 vacuous) — the
  spec-writer must pin them and a reviewer confirm.
- **The labeled corpus is hand-authored ground truth**, not a real benchmark; its
  quality bounds the gate's meaningfulness. Real benchmark data (BEIR-style) is out of
  scope.
- **"Offline" is overstated for data acquisition** — the metric math is deterministic,
  but acquiring results needs a READY engine + live provider; the deterministic fake
  provider is the CI path.
- **`gnosis-eval` is a dev/QA tool**, not shipped runtime — it adds a `[[bin]]` (no new
  deps).
- **Doc reconciliation needed:** spec §7.8 + `docs/next-steps.md`/`docs/pending.md`
  currently say the audit log "supplies the raw material" — that is the red herring the
  re-scope removes; the wording must be corrected to "re-runs `rag_query` against a
  READY engine."

## Next steps (after user go-ahead, gate order)

1. **SpecWriter** authors the eval spec (compile-horizon format) + the PBT register;
2. reviewer loop → empty;
3. **TestWriter** writes `eval_harness.rs` + `props_eval.rs` from the spec alone → red;
4. **Implementer** lands `src/retrieval/eval.rs` + `gnosis_eval` bin → green;
5. adversarial → blind-greens → trio green.
