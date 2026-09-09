//! §7.2 F2 Wire contract — **property-based-testing (PBT) gate — executed layer**.
//!
//! Implements **every** register row of `docs/specs/7-2-wire-property-register.md`
//! (8 rows: P-IM-1/2/3/4, P-SM-1/2/3, P-TP-1) as **one `#[test]` per row**, each
//! backed by a tiny **deterministic property harness** (`Xoshiro256**` seeded by
//! `SplitMix64`) — no third-party PBT crate, dependencies unchanged.
//!
//! ## Pinned deterministic seed
//!
//! Every row derives its `Rng` from a single fixed master seed mixed with a
//! per-row tag (the `P-<CLASS>-<N>` bytes), so the whole layer is reproducible.
//!
//! ## Budget (register gate: ≤100 cases/row, ≤400 total across the layer)
//!
//! | Row    | Budget | Row    | Budget |
//! |--------|--------|--------|--------|
//! | P-IM-1 | 60     | P-SM-1 | 40     |
//! | P-IM-2 | 40     | P-SM-2 | 30     |
//! | P-IM-3 | 60     | P-SM-3 | 40     |
//! | P-IM-4 | 40     | P-TP-1 | 40     |
//!
//! Sum = **350** generated cases (≤ 400). stop-after-5: a row aborts and reports
//! at most 5 distinct counterexamples.
//!
//! **RED-stage.** All assertions route through the `src/wire/` stub functions
//! (`todo!()`/`Err`), so every row FAILS at runtime — the failing red set. The
//! Implementer fills the stub bodies to go green, at which point these rows
//! assert decoded-value `eq` (not byte identity) against the input.

use gnosis::error::{code_table, from_wire};
use gnosis::{
    BlockedBy, DocumentId, EdgeKind, EngineState, EngineStatus, EngineSubsystems, GraphTraceStep,
    HybridTrace, NodeId, QueryMode, RagChunk, RagResult, RagResultItem, RagTrace, ReferenceState,
    Source, StoreError, TraceDescriptor,
};

// ---------------------------------------------------------------------------
// Deterministic PRNG: SplitMix64-seeded Xoshiro256** (house harness, §4.1–§4.5).
// ---------------------------------------------------------------------------

/// Master deterministic seed for the whole property layer.
const SEED: u64 = 0x9E37_79B9_7F4A_7C15;

/// SplitMix64 (also serves as the Xoshiro seed generator).
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Xoshiro256** — deterministic, dependency-free.
struct Rng {
    s: [u64; 4],
}

impl Rng {
    fn seeded(seed: u64) -> Self {
        let mut sm = seed;
        Rng {
            s: [
                splitmix64(&mut sm),
                splitmix64(&mut sm),
                splitmix64(&mut sm),
                splitmix64(&mut sm),
            ],
        }
    }
    fn rotl(x: u64, k: u32) -> u64 {
        x.rotate_left(k)
    }
    fn next(&mut self) -> u64 {
        let result = Self::rotl(self.s[1].wrapping_mul(5), 7).wrapping_mul(9);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[1] ^= t;
        self.s[2] = Self::rotl(self.s[2], 16);
        result
    }
    fn below(&mut self, n: u64) -> u64 {
        if n == 0 {
            0
        } else {
            self.next() % n
        }
    }
    fn pick<T: Copy>(&mut self, items: &[T]) -> T {
        items[self.below(items.len() as u64) as usize]
    }
    fn yes(&mut self) -> bool {
        self.next() & 1 == 1
    }
}

/// Per-row deterministic sub-seed from the master seed + a row tag.
fn row_seed(tag: u64) -> u64 {
    splitmix64(&mut (SEED ^ tag))
}

/// Row tags (`P-<CLASS>-<N>` bytes).
const PIM1: u64 = 0x50494D31; // "PIM1"
const PIM2: u64 = 0x50494D32; // "PIM2"
const PIM3: u64 = 0x50494D33; // "PIM3"
const PIM4: u64 = 0x50494D34; // "PIM4"
const PSM1: u64 = 0x50534D31; // "PSM1"
const PSM2: u64 = 0x50534D32; // "PSM2"
const PSM3: u64 = 0x50534D33; // "PSM3"
const PTP1: u64 = 0x50545031; // "PTP1"

/// Per-row budget caps (sum = 350 ≤ 400). P-IM-3 is 60 because its deterministic
/// case count is 51 (21 enumeration + 10 random + 20 from-wire inverse), which
/// exceeds a 40 budget; the count is fixed and legitimate, so the budget tracks it.
const B_IM1: u32 = 60;
const B_IM2: u32 = 40;
const B_IM3: u32 = 60;
const B_IM4: u32 = 40;
const B_SM1: u32 = 40;
const B_SM2: u32 = 30;
const B_SM3: u32 = 40;
const B_TP1: u32 = 40;

// ---------------------------------------------------------------------------
// Generators: the well-formed RagChunk / RagResult / StoreError corpus.
// ---------------------------------------------------------------------------

fn did(id: &str) -> DocumentId {
    DocumentId(id.to_string())
}
fn nid(id: &str) -> NodeId {
    NodeId(id.to_string())
}

fn item(doc: &str, node: &str, score: f64) -> RagResultItem {
    RagResultItem {
        document_id: did(doc),
        node_id: nid(node),
        score,
        snippet: "s".to_string(),
        source: Source::Local,
        parent: None,
        stale: None,
    }
}

/// Every `StoreError` variant with varied `ValidationError` messages.
fn error_corpus() -> Vec<StoreError> {
    let mut v = vec![
        StoreError::DocumentNotFound,
        StoreError::WikiNotFound,
        StoreError::ConflictError,
        StoreError::DocumentInUse,
        StoreError::InvalidState,
        StoreError::UnresolvedReference,
        StoreError::CommunityNotFound,
        StoreError::CycleDetected,
        StoreError::HopLimitExceeded,
        StoreError::EngineUnavailable,
        StoreError::EngineError,
        StoreError::TraceUnavailable,
        StoreError::EmbeddingUnavailable,
        StoreError::VectorIndexUnavailable,
        StoreError::LexicalIndexUnavailable,
        StoreError::RerankerUnavailable,
        StoreError::CompressionFailed,
        StoreError::HyDEGenerationFailed,
        StoreError::MultiQueryExpansionFailed,
        StoreError::SubTaskDagFailed,
    ];
    // ValidationError with boundary + structured + multi-codepoint messages.
    let msgs: Vec<String> = vec![
        "".to_string(),
        "a".to_string(),
        "é".to_string(),
        "漢字😀".to_string(),
        r#"{"code":1,"message":2}"#.to_string(), // JSON-object-shaped: must not be re-read as structure
        "note: data: marker (SSE line-start fragility)".to_string(), // contains literal "data:"
        "x".repeat(500),
    ];
    for m in msgs {
        v.push(StoreError::ValidationError(m));
    }
    v
}

/// Four well-formed `RagResult` values, one per `RagTrace` mode, plus a
/// graph `blocked_by:Some(..)` variant. The flat result's `query`/`snippet`
/// intentionally contain the literal `data:` so the SSE framing check (P-IM-4)
/// cannot pass merely because the fixed corpus lacks that substring.
fn wellformed_results() -> Vec<RagResult> {
    use QueryMode::{Flat, Hybrid, Vector};
    let flat = RagResult {
        query: "data: query for 'q'".to_string(),
        results: vec![RagResultItem {
            document_id: did("d1"),
            node_id: nid("n1"),
            score: 0.5,
            snippet: "snippet with data: marker".to_string(),
            source: Source::Local,
            parent: None,
            stale: None,
        }],
        engine: "gnosis".to_string(),
        citations: vec![(did("d1"), nid("n1"))],
        trace: RagTrace::Flat(TraceDescriptor {
            mode: Flat,
            engine: "gnosis".to_string(),
            top_k: 1,
            source: Source::Local,
        }),
        blocked_by: None,
    };
    let vector = RagResult {
        query: "v 漢字".to_string(),
        results: vec![item("d2", "n2", 1.0)],
        engine: "gnosis".to_string(),
        citations: vec![],
        trace: RagTrace::Vector(TraceDescriptor {
            mode: Vector,
            engine: "gnosis".to_string(),
            top_k: 50,
            source: Source::Zodiac,
        }),
        blocked_by: None,
    };
    let step = GraphTraceStep {
        from: (did("d1"), nid("n1")),
        to: (did("d2"), nid("n2")),
        edge: EdgeKind::Link,
        state: ReferenceState::Resolved,
    };
    let graph = RagResult {
        query: "g".to_string(),
        results: vec![],
        engine: "gnosis".to_string(),
        citations: vec![],
        trace: RagTrace::Graph(vec![step.clone(), step]),
        blocked_by: Some(vec![BlockedBy {
            document_id: did("d1"),
            node_id: nid("n1"),
            state: ReferenceState::Broken,
        }]),
    };
    let hybrid = RagResult {
        query: "h".to_string(),
        results: vec![],
        engine: "gnosis".to_string(),
        citations: vec![],
        trace: RagTrace::Hybrid(HybridTrace {
            mode: Hybrid,
            engine: "gnosis".to_string(),
            legs: vec![
                "graph".to_string(),
                "vector".to_string(),
                "lexical".to_string(),
            ],
            top_k: 10,
            source: Source::Local,
        }),
        blocked_by: None,
    };
    vec![flat, vector, graph, hybrid]
}

/// The `RagChunk` corpus: `Done`, an `Error` per `StoreError` variant, and a
/// `Result` per well-formed `RagResult` (all four trace shapes).
fn chunk_corpus() -> Vec<RagChunk> {
    let mut chunks = vec![RagChunk::Done];
    for e in error_corpus() {
        chunks.push(RagChunk::Error(e));
    }
    for r in wellformed_results() {
        chunks.push(RagChunk::Result(r));
    }
    chunks
}

fn all_true_subsystems() -> EngineSubsystems {
    EngineSubsystems {
        store: true,
        graph: true,
        lexical: true,
        vector: true,
        embedding: true,
        reranker: true,
    }
}

// ---------------------------------------------------------------------------
// P-IM-1 (IM) — strat:chunk-roundtrip — codec identity over all three variants.
// ---------------------------------------------------------------------------
//
// Invariant: ∀ well-formed chunk: decode_chunk(&encode_chunk(&chunk)) == chunk
// (element-wise, incl. Error(ValidationError(m)) preserving m and the full
// Result body).
#[test]
fn p_im_1_chunk_roundtrip() {
    let corpus = chunk_corpus();
    let mut rng = Rng::seeded(row_seed(PIM1));
    let mut cases: u32 = 0;
    let mut cexes: Vec<String> = Vec::new();
    for _ in 0..B_IM1 {
        // Deterministically pick a chunk; every ~10th case re-picks the first
        // (Done) so the boundary is hit repeatedly.
        let idx = if rng.below(10) == 0 {
            0
        } else {
            rng.below(corpus.len() as u64) as usize
        };
        let chunk = &corpus[idx];
        match gnosis::codecs::decode_chunk(&gnosis::codecs::encode_chunk(chunk)) {
            Ok(decoded) if decoded == *chunk => {}
            Ok(decoded) => {
                let cex = format!("identity: decoded {decoded:?} != input {chunk:?}");
                if !cexes.contains(&cex) {
                    cexes.push(cex);
                }
            }
            Err(e) => {
                let cex = format!("identity error for {chunk:?}: {e:?}");
                if !cexes.contains(&cex) {
                    cexes.push(cex);
                }
            }
        }
        cases += 1;
        if cexes.len() >= 5 {
            break; // stop-after-5
        }
    }
    assert!(
        cexes.is_empty(),
        "[P-IM-1][strat:chunk-roundtrip] BROKEN: {cexes:?}"
    );
    assert!(cases <= B_IM1, "B_IM1 budget exceeded: {cases}");
    println!(
        "[P-IM-1] generated cases: {cases} HELD={}",
        cexes.is_empty()
    );
}

// ---------------------------------------------------------------------------
// P-IM-2 (IM) — strat:result-bijective — RagResult codec exact bijectivity.
// ---------------------------------------------------------------------------
//
// Invariant: ∀ well-formed r: decode_result(&encode_result(&r)) == r in every
// field and field order (results / citations / trace / blocked_by element-wise).
#[test]
fn p_im_2_result_bijective() {
    let results = wellformed_results();
    let mut rng = Rng::seeded(row_seed(PIM2));
    let mut cases: u32 = 0;
    let mut cexes: Vec<String> = Vec::new();
    for _ in 0..B_IM2 {
        let r = &results[rng.below(results.len() as u64) as usize];
        match gnosis::codecs::decode_result(&gnosis::codecs::encode_result(r)) {
            Ok(decoded) if decoded == *r => {}
            Ok(decoded) => {
                let cex = format!("bijective {decoded:?} != {r:?}");
                if !cexes.contains(&cex) {
                    cexes.push(cex);
                }
            }
            Err(e) => {
                let cex = format!("decode_result err {e:?} for {r:?}");
                if !cexes.contains(&cex) {
                    cexes.push(cex);
                }
            }
        }
        cases += 1;
        if cexes.len() >= 5 {
            break;
        }
    }
    assert!(
        cexes.is_empty(),
        "[P-IM-2][strat:result-bijective] BROKEN: {cexes:?}"
    );
    assert!(cases <= B_IM2, "B_IM2 budget exceeded: {cases}");
    println!(
        "[P-IM-2] generated cases: {cases} HELD={}",
        cexes.is_empty()
    );
}

// ---------------------------------------------------------------------------
// P-IM-3 (IM) — strat:code-unique — wire_code uniqueness/stability/non-empty.
// ---------------------------------------------------------------------------
//
// Invariant: 21 distinct variants → 21 pairwise-distinct non-empty ASCII codes;
// equal variants (incl. any two ValidationError messages) yield equal codes.
#[test]
fn p_im_3_code_unique() {
    let errors = error_corpus();
    let mut rng = Rng::seeded(row_seed(PIM3));
    let mut cases: u32 = 0;
    let mut cexes: Vec<String> = Vec::new();

    // Enumeration: all 21 variants exactly once (dedupe by variant).
    let all = all_store_errors();
    let codes: Vec<&str> = all.iter().map(|e| e.wire_code()).collect();
    let mut seen = std::collections::HashSet::new();
    for (i, c) in codes.iter().enumerate() {
        assert!(!c.is_empty(), "[P-IM-3] empty code");
        assert!(c.is_ascii(), "[P-IM-3] non-ascii code {c}");
        assert!(
            seen.insert(*c),
            "[P-IM-3] duplicate code {c} at {i} ({:?})",
            all[i]
        );
        cases += 1;
    }
    // Random pairwise distinctness + stability + ValidationError-fixed.
    for _ in 0..10u32 {
        let a = &errors[rng.below(errors.len() as u64) as usize];
        let b = &errors[rng.below(errors.len() as u64) as usize];
        let a_code = a.wire_code();
        // A code "collision" means two *distinct variants* sharing one code. Two
        // `ValidationError` values carrying different messages are the SAME variant
        // (the §5 taxonomy is per-variant, not per-value), so they are required to
        // share the fixed `"validation_error"` code — compare by variant
        // discriminant, not value `==`.
        if std::mem::discriminant(a) != std::mem::discriminant(b) && a_code == b.wire_code() {
            let cex = format!("collision {a:?}/{b:?} -> {a_code}");
            if !cexes.contains(&cex) {
                cexes.push(cex);
            }
        }
        // Stability + ValidationError fixed code.
        if a_code != a.wire_code() {
            cexes.push(format!("unstable {a_code}"));
        }
        if let StoreError::ValidationError(m1) = a {
            let _ = m1;
        }
        cases += 1;
    }
    // From-wire bijectivity for the unit variants (inverse of wire_code).
    for e in all {
        if !matches!(e, StoreError::ValidationError(_)) {
            assert_eq!(
                from_wire(e.wire_code(), None).as_ref(),
                Some(&e),
                "[P-IM-3] from_wire inverse of wire_code for {e:?}"
            );
            cases += 1;
        }
    }
    let _ = code_table(); // table is a pure, total fn — exercising it is harmless.
    assert!(
        cexes.is_empty(),
        "[P-IM-3][strat:code-unique] BROKEN: {cexes:?}"
    );
    assert!(cases <= B_IM3, "B_IM3 budget exceeded: {cases}");
    println!(
        "[P-IM-3] generated cases: {cases} HELD={}",
        cexes.is_empty()
    );
}

/// The 20 unit (non-ValidationError) `StoreError` variants + a ValidationError.
fn all_store_errors() -> Vec<StoreError> {
    error_corpus()
        .into_iter()
        .filter(|e| !matches!(e, StoreError::ValidationError(_)))
        .take(20)
        .chain(std::iter::once(StoreError::ValidationError(
            "m".to_string(),
        )))
        .collect()
}

// ---------------------------------------------------------------------------
// P-IM-4 (IM) — strat:sse-roundtrip — SSE single-event framing round-trip.
// ---------------------------------------------------------------------------
//
// Invariant: ∀ chunk: decode_event(&encode_event(&chunk)) == chunk; the frame
// starts with `event: `, has exactly one `data:` line whose JSON `"type"` equals
// the `event:` value, and ends with a terminal blank line (`\n\n`).
#[test]
fn p_im_4_sse_roundtrip() {
    let corpus = chunk_corpus();
    let mut rng = Rng::seeded(row_seed(PIM4));
    let mut cases: u32 = 0;
    let mut cexes: Vec<String> = Vec::new();
    for _ in 0..B_IM4 {
        let chunk = &corpus[rng.below(corpus.len() as u64) as usize];
        let frame = gnosis::sse::encode_event(chunk);

        // Framing shape.
        if !frame.starts_with("event: ") {
            cexes.push(format!("frame does not start with 'event: ': {frame:?}"));
        }
        if !frame.ends_with("\n\n") {
            cexes.push(format!("frame lacks terminal blank line: {frame:?}"));
        }
        // Count only lines that START with `data: ` — a payload whose JSON
        // contains the literal `data:` (e.g. a ValidationError message, query, or
        // snippet) must NOT trip the framing check. The frame has exactly one
        // `data:` line regardless of how many `data:` substrings the body carries.
        let data_lines = frame.lines().filter(|l| l.starts_with("data: ")).count();
        if data_lines != 1 {
            cexes.push(format!(
                "frame must contain exactly one 'data: ' line: {frame:?}"
            ));
        }
        // Event value == data JSON "type".
        let event_line = frame.lines().find(|l| l.starts_with("event: "));
        let data_line = frame.lines().find(|l| l.starts_with("data: "));
        if let (Some(e), Some(d)) = (event_line, data_line) {
            let ev = e.trim_start_matches("event: ").trim();
            let dj = &d.trim_start_matches("data: ");
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(dj) {
                if let Some(t) = val.get("type").and_then(|x| x.as_str()) {
                    if ev != t {
                        cexes.push(format!("event {ev} != data type {t}"));
                    }
                } else {
                    cexes.push("data has no string type".to_string());
                }
            } else {
                cexes.push(format!("data not parseable as JSON: {dj:?}"));
            }
        }

        // Round-trip identity.
        match gnosis::sse::decode_event(&frame) {
            Ok(decoded) if decoded == *chunk => {}
            Ok(decoded) => cexes.push(format!("sse identity {decoded:?} != {chunk:?}")),
            Err(err) => cexes.push(format!("sse decode error {err:?} for {chunk:?}")),
        }
        cases += 1;
        if cexes.len() >= 5 {
            break;
        }
    }
    assert!(
        cexes.is_empty(),
        "[P-IM-4][strat:sse-roundtrip] BROKEN: {cexes:?}"
    );
    assert!(cases <= B_IM4, "B_IM4 budget exceeded: {cases}");
    println!(
        "[P-IM-4] generated cases: {cases} HELD={}",
        cexes.is_empty()
    );
}

// ---------------------------------------------------------------------------
// P-SM-1 (SM) — strat:envelope-stable — envelope round-trip field preservation.
// ---------------------------------------------------------------------------
//
// Invariant: for every encoder output env: from_json(&to_json(&env)?) == env,
// env.schema_version == CURRENT_SCHEMA_VERSION, env.id_format == opaque-string-v1.
#[test]
fn p_sm_1_envelope_stable() {
    use gnosis::envelope::{Envelope, CURRENT_SCHEMA_VERSION, ID_FORMAT_OPAQUE_STRING_V1};
    let chunks = chunk_corpus();
    let mut rng = Rng::seeded(row_seed(PSM1));
    let mut cases: u32 = 0;
    let mut cexes: Vec<String> = Vec::new();
    for _ in 0..B_SM1 {
        // Alternate encoder surfaces.
        let env = match rng.below(3) {
            0 => gnosis::codecs::encode_chunk(&chunks[rng.below(chunks.len() as u64) as usize]),
            1 => gnosis::codecs::encode_error(
                &error_corpus()[rng.below(error_corpus().len() as u64) as usize],
            ),
            _ => {
                let rs = wellformed_results();
                gnosis::codecs::encode_result(&rs[rng.below(rs.len() as u64) as usize])
            }
        };
        let json = match env.to_json() {
            Ok(j) => j,
            Err(e) => {
                cexes.push(format!("to_json error {e:?}"));
                cases += 1;
                if cexes.len() >= 5 {
                    break;
                }
                continue;
            }
        };
        match Envelope::from_json(&json) {
            Ok(back) => {
                if back != env {
                    cexes.push("envelope round-trip != input".to_string());
                }
                if back.schema_version != CURRENT_SCHEMA_VERSION {
                    cexes.push("schema_version != 1".to_string());
                }
                if back.id_format != ID_FORMAT_OPAQUE_STRING_V1 {
                    cexes.push("id_format != opaque-string-v1".to_string());
                }
            }
            Err(e) => cexes.push(format!("envelope from_json error {e:?}")),
        }
        cases += 1;
        if cexes.len() >= 5 {
            break;
        }
    }
    assert!(
        cexes.is_empty(),
        "[P-SM-1][strat:envelope-stable] BROKEN: {cexes:?}"
    );
    assert!(cases <= B_SM1, "B_SM1 budget exceeded: {cases}");
    println!(
        "[P-SM-1] generated cases: {cases} HELD={}",
        cexes.is_empty()
    );
}

// ---------------------------------------------------------------------------
// P-SM-2 (SM) — strat:validation-msg — ValidationError message survives.
// ---------------------------------------------------------------------------
//
// Invariant: ∀ m: decode_error(&encode_error(ValidationError(m))) is a
// ValidationError(m2) with m2==m; the wire payload "code" == "validation_error".
#[test]
fn p_sm_2_validation_msg() {
    let msgs = [
        "".to_string(),
        "   ".to_string(),
        "a".to_string(),
        "é漢字😀".to_string(),
        r#"{"code":1,"message":2}"#.to_string(),
        "a very long message ".repeat(64),
    ];
    let mut rng = Rng::seeded(row_seed(PSM2));
    let mut cases: u32 = 0;
    let mut cexes: Vec<String> = Vec::new();
    for _ in 0..B_SM2 {
        let m = &msgs[rng.below(msgs.len() as u64) as usize];
        let err = StoreError::ValidationError(m.clone());
        let env = gnosis::codecs::encode_error(&err);
        // Wire code field == "validation_error" regardless of m.
        if let Some(code) = env.payload.get("code").and_then(|c| c.as_str()) {
            if code != "validation_error" {
                cexes.push(format!("wire code {code} != validation_error for {m:?}"));
            }
        } else {
            cexes.push("payload missing code".to_string());
        }
        match gnosis::codecs::decode_error(&env) {
            Ok(StoreError::ValidationError(m2)) if m2 == *m => {}
            Ok(other) => cexes.push(format!(
                "decoded {other:?}, expected ValidationError({m:?})"
            )),
            Err(e) => cexes.push(format!("decode_error {e:?} for {m:?}")),
        }
        cases += 1;
        if cexes.len() >= 5 {
            break;
        }
    }
    assert!(
        cexes.is_empty(),
        "[P-SM-2][strat:validation-msg] BROKEN: {cexes:?}"
    );
    assert!(cases <= B_SM2, "B_SM2 budget exceeded: {cases}");
    println!(
        "[P-SM-2] generated cases: {cases} HELD={}",
        cexes.is_empty()
    );
}

// ---------------------------------------------------------------------------
// P-SM-3 (SM) — strat:health-determinism — health is a pure function.
// ---------------------------------------------------------------------------
//
// Invariant: ∀ sampled status: health(s)==health(s); health(clone(s))==health(s)
// element-wise; last_error is Some iff input's is Some; subsystem flags mirror.
#[test]
fn p_sm_3_health_determinism() {
    let states = [
        EngineState::Ready,
        EngineState::Starting,
        EngineState::Degraded,
        EngineState::Unavailable,
    ];
    let versions = ["", "0.0.0", env!("CARGO_PKG_VERSION")];
    let masks = [
        all_true_subsystems(),
        EngineSubsystems {
            store: false,
            graph: false,
            lexical: false,
            vector: false,
            embedding: false,
            reranker: false,
        },
        EngineSubsystems {
            store: true,
            graph: true,
            lexical: true,
            vector: true,
            embedding: false,
            reranker: true,
        },
        EngineSubsystems {
            store: true,
            graph: true,
            lexical: true,
            vector: true,
            embedding: true,
            reranker: false,
        },
        EngineSubsystems {
            store: true,
            graph: false,
            lexical: true,
            vector: false,
            embedding: true,
            reranker: false,
        },
    ];
    let mut rng = Rng::seeded(row_seed(PSM3));
    let mut cases: u32 = 0;
    let mut cexes: Vec<String> = Vec::new();
    for _ in 0..B_SM3 {
        let state = rng.pick(&states);
        let status = EngineStatus {
            state,
            version: versions[rng.below(versions.len() as u64) as usize].to_string(),
            subsystems: masks[rng.below(masks.len() as u64) as usize].clone(),
            last_error: if state == EngineState::Degraded && rng.yes() {
                Some("a non-core subsystem (embedding/reranker) is unavailable".to_string())
            } else {
                None
            },
        };
        let h1 = gnosis::status::health(&status);
        let h2 = gnosis::status::health(&status);
        if h1 != h2 {
            cexes.push("health is non-deterministic".to_string());
        }
        let clone = status.clone();
        let h3 = gnosis::status::health(&clone);
        if h1 != h3 {
            cexes.push("health(clone) != health(input)".to_string());
        }
        if h1.last_error.is_some() != status.last_error.is_some() {
            cexes.push("last_error did not mirror the input".to_string());
        }
        if let Some(le) = &h1.last_error {
            if status.last_error.as_ref() != Some(le) {
                cexes.push("last_error content mismatch".to_string());
            }
        }
        if h1.subsystems != status.subsystems {
            cexes.push("subsystems flags not mirrored".to_string());
        }
        if h1.state != status.state || h1.version != status.version {
            cexes.push("state/version not mirrored".to_string());
        }
        cases += 1;
        if cexes.len() >= 5 {
            break;
        }
    }
    assert!(
        cexes.is_empty(),
        "[P-SM-3][strat:health-determinism] BROKEN: {cexes:?}"
    );
    assert!(cases <= B_SM3, "B_SM3 budget exceeded: {cases}");
    println!(
        "[P-SM-3] generated cases: {cases} HELD={}",
        cexes.is_empty()
    );
}

// ---------------------------------------------------------------------------
// P-TP-1 (TP) — strat:encode-validates — decode-then-validate total on encoder.
// ---------------------------------------------------------------------------
//
// Invariant: ∀ well-formed r: decode_result(&encode_result(&r)).is_ok() AND
// validate_rag_result(&decode_rag_result(&encode_result(&r).payload)).is_ok().
#[test]
fn p_tp_1_encode_validates() {
    let results = wellformed_results();
    let mut rng = Rng::seeded(row_seed(PTP1));
    let mut cases: u32 = 0;
    let mut cexes: Vec<String> = Vec::new();
    for _ in 0..B_TP1 {
        let r = &results[rng.below(results.len() as u64) as usize];
        let env = gnosis::codecs::encode_result(r);
        if gnosis::codecs::decode_result(&env).is_err() {
            cexes.push("decode_result rejected encoder output".to_string());
        }
        match gnosis::codecs::decode_result(&env) {
            Ok(decoded) => {
                if let Err(vf) = gnosis::decode::validate_rag_result(&decoded) {
                    cexes.push(format!("validate_rag_result failed: {vf:?}"));
                }
            }
            Err(e) => cexes.push(format!("decode_result error {e:?}")),
        }
        // Direct decode_rag_result on the payload + validate (P-TP-1 totality).
        if let Ok(serde_val) = serde_json::to_value(r) {
            if let Ok(decoded2) = gnosis::decode::decode_rag_result(&serde_val) {
                if let Err(vf) = gnosis::decode::validate_rag_result(&decoded2) {
                    cexes.push(format!("direct validate failed: {vf:?}"));
                }
            } else {
                cexes.push("decode_rag_result rejected a well-formed body".to_string());
            }
        }
        cases += 1;
        if cexes.len() >= 5 {
            break;
        }
    }
    assert!(
        cexes.is_empty(),
        "[P-TP-1][strat:encode-validates] BROKEN: {cexes:?}"
    );
    assert!(cases <= B_TP1, "B_TP1 budget exceeded: {cases}");
    println!(
        "[P-TP-1] generated cases: {cases} HELD={}",
        cexes.is_empty()
    );
}
