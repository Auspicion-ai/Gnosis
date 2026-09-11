//! §7.2 P1a — Document-CRUD wire contract — **property-based-testing (PBT) gate —
//! executed layer** (`tests/props_crud_wire.rs`).
//!
//! Implements **every** register row of `docs/specs/p1a-document-crud-wire.md`
//! §5.x (8 rows: P-IM-1..4, P-SM-1..3, P-TP-1) as **one `#[test]` per row**, each
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
//! **RED-stage.** The P1a `src/wire/crud.rs` module does NOT exist yet — the
//! `gnosis::wire::crud::{…}` surface is absent, so this suite FAILS TO COMPILE
//! (the missing-symbol red set). The Implementer lands the least `src/wire/`
//! code to green, at which point these rows assert decoded-value `eq` (not byte
//! identity) against the input.

use gnosis::envelope::{Envelope, CURRENT_SCHEMA_VERSION, ID_FORMAT_OPAQUE_STRING_V1};
use gnosis::wire::crud::{
    encode_crud_error, encode_crud_request, encode_crud_response, decode_crud_request,
    decode_crud_response, validate_crud_result, CrudMethod, CrudRequestArgs, CrudResponseError,
    CrudResult, ENGINE_ENDPOINTS,
};
use gnosis::{
    CreateDocumentRequest, DocState, Document, DocumentId, DocumentList, DocumentSummary, Edge,
    EdgeKind, Graph, ListDocumentsFilter, Node, NodeId, NodeKind, ReferenceState, StoreError,
    UpdateDocumentRequest, Wiki, WikiId,
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

/// Per-row budget caps (sum = 350 ≤ 400).
const B_IM1: u32 = 60;
const B_IM2: u32 = 40;
const B_IM3: u32 = 60;
const B_IM4: u32 = 40;
const B_SM1: u32 = 40;
const B_SM2: u32 = 30;
const B_SM3: u32 = 40;
const B_TP1: u32 = 40;

// ---------------------------------------------------------------------------
// Fixtures / generators: the well-formed CRUD request/result corpus.
// ---------------------------------------------------------------------------

fn did(id: &str) -> DocumentId {
    DocumentId(id.to_string())
}
fn wid(id: &str) -> WikiId {
    WikiId(id.to_string())
}
fn nid(id: &str) -> NodeId {
    NodeId(id.to_string())
}

/// All 11 methods in a fixed order.
fn all_methods() -> Vec<CrudMethod> {
    use CrudMethod::*;
    vec![
        CreateDocument,
        GetDocument,
        UpdateDocument,
        DeleteDocument,
        PublishDocument,
        UnpublishDocument,
        ArchiveDocument,
        ListDocuments,
        CreateWiki,
        GetWiki,
        ListWikis,
    ]
}

/// The 7 mutating methods.
fn mutating_methods() -> Vec<CrudMethod> {
    use CrudMethod::*;
    vec![
        CreateDocument,
        UpdateDocument,
        DeleteDocument,
        PublishDocument,
        UnpublishDocument,
        ArchiveDocument,
        CreateWiki,
    ]
}

/// The 4 read-only methods.
fn read_only_methods() -> Vec<CrudMethod> {
    use CrudMethod::*;
    vec![GetDocument, ListDocuments, GetWiki, ListWikis]
}

/// Boundary caller corpus (§5.x P-IM-1 / P-SM-2).
const CALLERS: &[&str] = &[
    "",
    "user:alice",
    "漢字😀",
    "caller",
    "body",
    "user:caller/body",
];

/// Boundary opaque-id corpus.
const IDS: &[&str] = &[
    "",
    "d1",
    "550e8400-e29b-41d4-a716-446655440000",
    "漢字",
];

/// Boundary title/name corpus (incl. a 200-char boundary string).
fn titles() -> Vec<String> {
    vec![
        "".to_string(),
        "a".to_string(),
        "漢字😀".to_string(),
        "x".repeat(200),
    ]
}

/// A random caller string.
fn gen_caller(rng: &mut Rng) -> String {
    rng.pick(CALLERS).to_string()
}

/// A random title/name string (from the boundary corpus).
fn gen_title(rng: &mut Rng) -> String {
    let t = titles();
    t[rng.below(t.len() as u64) as usize].clone()
}

/// A random opaque id string.
fn gen_id(rng: &mut Rng) -> String {
    rng.pick(IDS).to_string()
}

/// A random `DocState`.
fn gen_state(rng: &mut Rng) -> DocState {
    rng.pick(&[DocState::Draft, DocState::Published, DocState::Archived])
}

/// A random tags vector (empty / one / many / None).
fn gen_tags(rng: &mut Rng) -> Option<Vec<String>> {
    match rng.below(4) {
        0 => None,
        1 => Some(vec![]),
        2 => Some(vec!["guide".to_string()]),
        _ => Some(vec!["a".to_string(), "漢字".to_string(), "c".to_string()]),
    }
}

/// A random graph (empty / one node / many nodes+edges).
fn gen_graph(rng: &mut Rng) -> Graph {
    match rng.below(3) {
        0 => Graph {
            nodes: vec![],
            edges: vec![],
        },
        1 => Graph {
            nodes: vec![Node {
                document_id: did("d1"),
                node_id: nid("n1"),
                kind: NodeKind::Content,
                value: Some("hello".to_string()),
                fact_key: None,
                target: None,
            }],
            edges: vec![Edge {
                source: (did("d1"), nid("n1")),
                target: (did("d1"), nid("n1")),
                kind: EdgeKind::DocHead,
                state: None,
                cross_wiki: false,
                relation_type: None,
            }],
        },
        _ => Graph {
            nodes: vec![
                Node {
                    document_id: did("d1"),
                    node_id: nid("n1"),
                    kind: NodeKind::Content,
                    value: Some("a".to_string()),
                    fact_key: None,
                    target: None,
                },
                Node {
                    document_id: did("d1"),
                    node_id: nid("n2"),
                    kind: NodeKind::Fact,
                    value: Some("b".to_string()),
                    fact_key: Some("fk".to_string()),
                    target: None,
                },
            ],
            edges: vec![
                Edge {
                    source: (did("d1"), nid("n1")),
                    target: (did("d1"), nid("n2")),
                    kind: EdgeKind::Link,
                    state: Some(ReferenceState::Resolved),
                    cross_wiki: false,
                    relation_type: None,
                },
                Edge {
                    source: (did("d1"), nid("n1")),
                    target: (did("d1"), nid("n1")),
                    kind: EdgeKind::DocHead,
                    state: None,
                    cross_wiki: false,
                    relation_type: None,
                },
            ],
        },
    }
}

/// A random well-formed `args` for the given method.
fn gen_args(rng: &mut Rng, m: &CrudMethod) -> CrudRequestArgs {
    use CrudMethod::*;
    match m {
        CreateDocument => CrudRequestArgs::CreateDocument {
            caller: gen_caller(rng),
            wiki_id: wid(&gen_id(rng)),
            body: CreateDocumentRequest {
                title: gen_title(rng),
                tags: gen_tags(rng),
                author: if rng.yes() {
                    Some("alice".to_string())
                } else {
                    None
                },
            },
        },
        GetDocument => CrudRequestArgs::GetDocument {
            document_id: did(&gen_id(rng)),
        },
        UpdateDocument => CrudRequestArgs::UpdateDocument {
            caller: gen_caller(rng),
            document_id: did(&gen_id(rng)),
            body: UpdateDocumentRequest {
                base_revision: rng.pick(&[0u64, 1, u64::MAX]),
                graph: gen_graph(rng),
                title: if rng.yes() {
                    Some(gen_title(rng))
                } else {
                    None
                },
                tags: gen_tags(rng),
            },
        },
        DeleteDocument => CrudRequestArgs::DeleteDocument {
            caller: gen_caller(rng),
            document_id: did(&gen_id(rng)),
        },
        PublishDocument => CrudRequestArgs::PublishDocument {
            caller: gen_caller(rng),
            document_id: did(&gen_id(rng)),
        },
        UnpublishDocument => CrudRequestArgs::UnpublishDocument {
            caller: gen_caller(rng),
            document_id: did(&gen_id(rng)),
        },
        ArchiveDocument => CrudRequestArgs::ArchiveDocument {
            caller: gen_caller(rng),
            document_id: did(&gen_id(rng)),
        },
        ListDocuments => CrudRequestArgs::ListDocuments {
            wiki_id: wid(&gen_id(rng)),
            body: ListDocumentsFilter {
                state: if rng.yes() {
                    Some(gen_state(rng))
                } else {
                    None
                },
                tag: if rng.yes() {
                    Some("tag".to_string())
                } else {
                    None
                },
                page: rng.pick(&[Some(1u64), Some(100), None]),
                page_size: rng.pick(&[Some(1u64), Some(100), None]),
            },
        },
        CreateWiki => CrudRequestArgs::CreateWiki {
            caller: gen_caller(rng),
            name: gen_title(rng),
        },
        GetWiki => CrudRequestArgs::GetWiki {
            wiki_id: wid(&gen_id(rng)),
        },
        ListWikis => CrudRequestArgs::ListWikis,
    }
}

/// A random `revision` over {0, non-zero} (§5.x P-IM-2 generator-coverage note).
fn gen_revision(rng: &mut Rng) -> u64 {
    rng.pick(&[0u64, 1, 7, u64::MAX])
}

/// A random ISO-8601 `created_at`/`updated_at` string (§5.x P-IM-2 note).
fn gen_timestamp(rng: &mut Rng) -> String {
    rng.pick(&[
        "2026-09-09T00:00:00Z",
        "2024-01-01T12:30:45Z",
        "2026-12-31T23:59:59Z",
    ])
    .to_string()
}

/// A random well-formed `Document` with the given `revision`/`state` and varied
/// ISO-8601 timestamps.
fn gen_doc(rng: &mut Rng, revision: u64, state: DocState) -> Document {
    Document {
        document_id: did(&gen_id(rng)),
        wiki_id: wid(&gen_id(rng)),
        revision,
        state,
        graph: gen_graph(rng),
        title: gen_title(rng),
        created_at: gen_timestamp(rng),
        updated_at: gen_timestamp(rng),
        tags: gen_tags(rng).unwrap_or_default(),
        author: if rng.yes() {
            Some("alice".to_string())
        } else {
            None
        },
    }
}

/// A random well-formed `CrudResult` for the given method (satisfying the
/// CRUD-specific invariants of §4.3 / §10).
fn gen_result(rng: &mut Rng, m: &CrudMethod) -> CrudResult {
    use CrudMethod::*;
    match m {
        // createDocument must yield revision == 0 and state == Draft.
        CreateDocument => CrudResult::Document(gen_doc(rng, 0, DocState::Draft)),
        // getDocument/updateDocument have no invariant: vary revision over
        // {0, non-zero} and state over all three DocState values.
        GetDocument => {
            let rev = gen_revision(rng);
            let st = gen_state(rng);
            CrudResult::Document(gen_doc(rng, rev, st))
        }
        UpdateDocument => {
            let rev = gen_revision(rng);
            let st = gen_state(rng);
            CrudResult::Document(gen_doc(rng, rev, st))
        }
        DeleteDocument => CrudResult::DeleteDocument,
        // publish/unpublish/archive pin the documented state; revision is free.
        PublishDocument => {
            let rev = gen_revision(rng);
            CrudResult::Document(gen_doc(rng, rev, DocState::Published))
        }
        UnpublishDocument => {
            let rev = gen_revision(rng);
            CrudResult::Document(gen_doc(rng, rev, DocState::Draft))
        }
        ArchiveDocument => {
            let rev = gen_revision(rng);
            CrudResult::Document(gen_doc(rng, rev, DocState::Archived))
        }
        ListDocuments => CrudResult::DocumentList(DocumentList {
            items: (0..rng.below(4))
                .map(|i| DocumentSummary {
                    document_id: did(&format!("d{i}")),
                    wiki_id: wid("w1"),
                    title: "t".to_string(),
                    state: gen_state(rng),
                    revision: rng.below(10),
                    updated_at: "2026-09-09T00:00:00Z".to_string(),
                })
                .collect(),
            total: rng.below(100),
            page: rng.pick(&[1u64, 2, 100]),
            page_size: rng.pick(&[1u64, 20, 100]),
        }),
        CreateWiki => CrudResult::Wiki(Wiki {
            wiki_id: wid(&gen_id(rng)),
            name: gen_title(rng),
        }),
        GetWiki => CrudResult::Wiki(Wiki {
            wiki_id: wid(&gen_id(rng)),
            name: gen_title(rng),
        }),
        ListWikis => CrudResult::WikiList(
            (0..rng.below(4))
                .map(|i| Wiki {
                    wiki_id: wid(&format!("w{i}")),
                    name: gen_title(rng),
                })
                .collect(),
        ),
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
    let msgs: Vec<String> = vec![
        "".to_string(),
        "a".to_string(),
        "é".to_string(),
        "漢字😀".to_string(),
        r#"{"code":1,"message":2}"#.to_string(), // JSON-object-shaped: must not be re-read as structure
        "x".repeat(500),
    ];
    for m in msgs {
        v.push(StoreError::ValidationError(m));
    }
    v
}

/// Extract the `caller` from a `CrudRequestArgs` (None for read-only variants).
fn caller_of(args: &CrudRequestArgs) -> Option<&str> {
    match args {
        CrudRequestArgs::CreateDocument { caller, .. }
        | CrudRequestArgs::UpdateDocument { caller, .. }
        | CrudRequestArgs::DeleteDocument { caller, .. }
        | CrudRequestArgs::PublishDocument { caller, .. }
        | CrudRequestArgs::UnpublishDocument { caller, .. }
        | CrudRequestArgs::ArchiveDocument { caller, .. }
        | CrudRequestArgs::CreateWiki { caller, .. } => Some(caller.as_str()),
        CrudRequestArgs::GetDocument { .. }
        | CrudRequestArgs::ListDocuments { .. }
        | CrudRequestArgs::GetWiki { .. }
        | CrudRequestArgs::ListWikis => None,
    }
}

// ---------------------------------------------------------------------------
// P-IM-1 (IM) — strat:crud-request-roundtrip — CRUD request round-trip identity.
// ---------------------------------------------------------------------------
//
// Invariant: ∀ (method, args): decode_crud_request(&encode_crud_request(method, args))
// == (method, args) — method ==, args element-wise == (caller, ids, full body).
#[test]
fn p_im_1_crud_request_roundtrip() {
    let mut rng = Rng::seeded(row_seed(PIM1));
    let mut cases: u32 = 0;
    let mut cexes: Vec<String> = Vec::new();
    for _ in 0..B_IM1 {
        let m = rng.pick(&all_methods());
        let args = gen_args(&mut rng, &m);
        let decoded = decode_crud_request(&encode_crud_request(m.clone(), args.clone()));
        match decoded {
            Ok((dm, da)) if dm == m && da == args => {}
            Ok((dm, da)) => cexes.push(format!(
                "round-trip mismatch for {m:?}: got ({dm:?}, {da:?})"
            )),
            Err(e) => cexes.push(format!("decode error {e:?} for {m:?}")),
        }
        cases += 1;
        if cexes.len() >= 5 {
            break;
        }
    }
    assert!(
        cexes.is_empty(),
        "[P-IM-1][strat:crud-request-roundtrip] BROKEN: {cexes:?}"
    );
    assert!(cases <= B_IM1, "B_IM1 budget exceeded: {cases}");
    println!(
        "[P-IM-1] generated cases: {cases} HELD={}",
        cexes.is_empty()
    );
}

// ---------------------------------------------------------------------------
// P-IM-2 (IM) — strat:crud-response-roundtrip — CRUD response round-trip identity.
// ---------------------------------------------------------------------------
//
// Invariant: ∀ well-formed (method, result): decode_crud_response(&encode_crud_response(method, result))
// == result — the CrudResult variant and every field ==.
#[test]
fn p_im_2_crud_response_roundtrip() {
    let mut rng = Rng::seeded(row_seed(PIM2));
    let mut cases: u32 = 0;
    let mut cexes: Vec<String> = Vec::new();
    for _ in 0..B_IM2 {
        let m = rng.pick(&all_methods());
        let result = gen_result(&mut rng, &m);
        let decoded = decode_crud_response(&encode_crud_response(m.clone(), result.clone()));
        match decoded {
            Ok(dr) if dr == result => {}
            Ok(dr) => cexes.push(format!(
                "response round-trip mismatch for {m:?}: got {dr:?}"
            )),
            Err(e) => cexes.push(format!("decode error {e:?} for {m:?}")),
        }
        cases += 1;
        if cexes.len() >= 5 {
            break;
        }
    }
    assert!(
        cexes.is_empty(),
        "[P-IM-2][strat:crud-response-roundtrip] BROKEN: {cexes:?}"
    );
    assert!(cases <= B_IM2, "B_IM2 budget exceeded: {cases}");
    println!(
        "[P-IM-2] generated cases: {cases} HELD={}",
        cexes.is_empty()
    );
}

// ---------------------------------------------------------------------------
// P-IM-3 (IM) — strat:crud-error-roundtrip — CRUD error round-trip.
// ---------------------------------------------------------------------------
//
// Invariant: ∀ (method, e): decode_crud_response(&encode_crud_error(method, e))
// matches Err(CrudResponseError::Store(e2)) with e2 == e.
#[test]
fn p_im_3_crud_error_roundtrip() {
    let mut rng = Rng::seeded(row_seed(PIM3));
    let errors = error_corpus();
    let methods = all_methods();
    let mut cases: u32 = 0;
    let mut cexes: Vec<String> = Vec::new();
    for _ in 0..B_IM3 {
        let m = rng.pick(&methods);
        let e = errors[rng.below(errors.len() as u64) as usize].clone();
        let decoded = decode_crud_response(&encode_crud_error(m.clone(), &e));
        match decoded {
            Err(CrudResponseError::Store(e2)) if e2 == e => {}
            Err(CrudResponseError::Store(e2)) => cexes.push(format!(
                "error round-trip mismatch for {m:?}: got {e2:?}, expected {e:?}"
            )),
            other => cexes.push(format!(
                "expected Err(Store(..)) for {m:?}/{e:?}, got {other:?}"
            )),
        }
        cases += 1;
        if cexes.len() >= 5 {
            break;
        }
    }
    assert!(
        cexes.is_empty(),
        "[P-IM-3][strat:crud-error-roundtrip] BROKEN: {cexes:?}"
    );
    assert!(cases <= B_IM3, "B_IM3 budget exceeded: {cases}");
    println!(
        "[P-IM-3] generated cases: {cases} HELD={}",
        cexes.is_empty()
    );
}

// ---------------------------------------------------------------------------
// P-IM-4 (IM) — strat:crud-method-unique — method discriminator uniqueness.
// ---------------------------------------------------------------------------
//
// Invariant: ∀ distinct a,b: a.method_str() != b.method_str(); !a.method_str().is_empty();
// a.method_str() == a.method_str() on repeat and for equal variants.
#[test]
fn p_im_4_crud_method_unique() {
    let mut rng = Rng::seeded(row_seed(PIM4));
    let methods = all_methods();
    let mut cases: u32 = 0;
    let mut cexes: Vec<String> = Vec::new();
    for _ in 0..B_IM4 {
        // Fixed-order pairwise check.
        for i in 0..methods.len() {
            for j in (i + 1)..methods.len() {
                let a = &methods[i];
                let b = &methods[j];
                let sa = a.method_str();
                let sb = b.method_str();
                if sa == sb {
                    cexes.push(format!("collision {a:?} == {b:?} == {sa}"));
                }
                if sa.is_empty() {
                    cexes.push(format!("empty method_str for {a:?}"));
                }
                if !sa.is_ascii() {
                    cexes.push(format!("non-ascii method_str {sa} for {a:?}"));
                }
                // Determinism: repeat yields the same string.
                if a.method_str() != sa {
                    cexes.push(format!("non-deterministic method_str for {a:?}"));
                }
            }
        }
        // Random permutation pairwise check.
        let mut perm: Vec<CrudMethod> = methods.clone();
        for i in (1..perm.len()).rev() {
            let j = rng.below((i + 1) as u64) as usize;
            perm.swap(i, j);
        }
        for i in 0..perm.len() {
            for j in (i + 1)..perm.len() {
                if perm[i].method_str() == perm[j].method_str() {
                    cexes.push(format!(
                        "permutation collision {:?} == {:?}",
                        perm[i],
                        perm[j]
                    ));
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
        "[P-IM-4][strat:crud-method-unique] BROKEN: {cexes:?}"
    );
    assert!(cases <= B_IM4, "B_IM4 budget exceeded: {cases}");
    println!(
        "[P-IM-4] generated cases: {cases} HELD={}",
        cexes.is_empty()
    );
}

// ---------------------------------------------------------------------------
// P-SM-1 (SM) — strat:crud-envelope-stable — envelope round-trip stability.
// ---------------------------------------------------------------------------
//
// Invariant: ∀ CRUD envelope env: from_json(&to_json(&env)).unwrap() == env;
// env.schema_version == CURRENT_SCHEMA_VERSION; env.id_format == ID_FORMAT_OPAQUE_STRING_V1.
#[test]
fn p_sm_1_crud_envelope_stable() {
    let mut rng = Rng::seeded(row_seed(PSM1));
    let methods = all_methods();
    let errors = error_corpus();
    let mut cases: u32 = 0;
    let mut cexes: Vec<String> = Vec::new();
    for _ in 0..B_SM1 {
        let m = rng.pick(&methods);
        let env = match rng.below(3) {
            0 => encode_crud_request(m.clone(), gen_args(&mut rng, &m)),
            1 => encode_crud_response(m.clone(), gen_result(&mut rng, &m)),
            _ => {
                let e = errors[rng.below(errors.len() as u64) as usize].clone();
                encode_crud_error(m.clone(), &e)
            }
        };
        if env.schema_version != CURRENT_SCHEMA_VERSION {
            cexes.push(format!("schema_version {} != 1", env.schema_version));
        }
        if env.id_format != ID_FORMAT_OPAQUE_STRING_V1 {
            cexes.push(format!("id_format {} != opaque-string-v1", env.id_format));
        }
        match Envelope::from_json(&env.to_json().unwrap()) {
            Ok(parsed) if parsed == env => {}
            Ok(parsed) => cexes.push(format!("envelope mismatch: {parsed:?} vs {env:?}")),
            Err(e) => cexes.push(format!("envelope from_json error {e:?}")),
        }
        cases += 1;
        if cexes.len() >= 5 {
            break;
        }
    }
    assert!(
        cexes.is_empty(),
        "[P-SM-1][strat:crud-envelope-stable] BROKEN: {cexes:?}"
    );
    assert!(cases <= B_SM1, "B_SM1 budget exceeded: {cases}");
    println!(
        "[P-SM-1] generated cases: {cases} HELD={}",
        cexes.is_empty()
    );
}

// ---------------------------------------------------------------------------
// P-SM-2 (SM) — strat:crud-caller-preserved — RBAC caller survives round-trip.
// ---------------------------------------------------------------------------
//
// Invariant: ∀ mutating (method, args): decoded caller == encoded caller;
// ∀ read-only (method, args): the args object has no "caller" key.
#[test]
fn p_sm_2_crud_caller_preserved() {
    let mut rng = Rng::seeded(row_seed(PSM2));
    let mut cases: u32 = 0;
    let mut cexes: Vec<String> = Vec::new();
    for _ in 0..B_SM2 {
        // Mutating: caller survives.
        let m = rng.pick(&mutating_methods());
        let args = gen_args(&mut rng, &m);
        let expected = caller_of(&args).unwrap().to_string();
        let decoded = decode_crud_request(&encode_crud_request(m.clone(), args.clone()));
        match decoded {
            Ok((_, da)) => match caller_of(&da) {
                Some(c) if c == expected => {}
                Some(c) => cexes.push(format!(
                    "caller mismatch for {m:?}: got {c:?}, expected {expected:?}"
                )),
                None => cexes.push(format!("mutating {m:?} decoded without caller")),
            },
            Err(e) => cexes.push(format!("decode error {e:?} for {m:?}")),
        }
        // Read-only: no "caller" key in the encoded args object.
        let ro = rng.pick(&read_only_methods());
        let ro_args = gen_args(&mut rng, &ro);
        let env = encode_crud_request(ro.clone(), ro_args);
        let args_obj = env.payload.get("args").and_then(|v| v.as_object());
        match args_obj {
            Some(obj) if obj.contains_key("caller") => {
                cexes.push(format!("read-only {ro:?} carries a caller key"))
            }
            Some(_) => {}
            None => cexes.push(format!("read-only {ro:?} has no args object")),
        }
        cases += 1;
        if cexes.len() >= 5 {
            break;
        }
    }
    assert!(
        cexes.is_empty(),
        "[P-SM-2][strat:crud-caller-preserved] BROKEN: {cexes:?}"
    );
    assert!(cases <= B_SM2, "B_SM2 budget exceeded: {cases}");
    println!(
        "[P-SM-2] generated cases: {cases} HELD={}",
        cexes.is_empty()
    );
}

// ---------------------------------------------------------------------------
// P-SM-3 (SM) — strat:crud-endpoint-unique — endpoint-path uniqueness + bijection.
// ---------------------------------------------------------------------------
//
// Invariant: the ENGINE_ENDPOINTS table is a bijection between the 11 paths and
// the 11 methods (pairwise-distinct paths, pairwise-distinct methods, 11 rows).
#[test]
fn p_sm_3_crud_endpoint_unique() {
    let mut rng = Rng::seeded(row_seed(PSM3));
    let mut cases: u32 = 0;
    let mut cexes: Vec<String> = Vec::new();
    for _ in 0..B_SM3 {
        if ENGINE_ENDPOINTS.len() != 11 {
            cexes.push(format!(
                "ENGINE_ENDPOINTS has {} rows, expected 11",
                ENGINE_ENDPOINTS.len()
            ));
        }
        let mut paths: Vec<&str> = Vec::new();
        let mut methods: Vec<CrudMethod> = Vec::new();
        for (path, m) in ENGINE_ENDPOINTS {
            if path.is_empty() {
                cexes.push("empty endpoint path".to_string());
            }
            if paths.contains(path) {
                cexes.push(format!("duplicate path {path}"));
            }
            if methods.contains(m) {
                cexes.push(format!("duplicate method {m:?}"));
            }
            paths.push(path);
            methods.push(m.clone());
        }
        // Bijection: every one of the 11 methods appears exactly once.
        for m in all_methods() {
            if !methods.contains(&m) {
                cexes.push(format!("method {m:?} missing from ENGINE_ENDPOINTS"));
            }
        }
        cases += 1;
        if cexes.len() >= 5 {
            break;
        }
    }
    assert!(
        cexes.is_empty(),
        "[P-SM-3][strat:crud-endpoint-unique] BROKEN: {cexes:?}"
    );
    assert!(cases <= B_SM3, "B_SM3 budget exceeded: {cases}");
    println!(
        "[P-SM-3] generated cases: {cases} HELD={}",
        cexes.is_empty()
    );
}

// ---------------------------------------------------------------------------
// P-TP-1 (TP) — strat:crud-encode-validates — decode-then-validate total on
// well-formed input.
// ---------------------------------------------------------------------------
//
// Invariant: ∀ well-formed (method, result): decode_crud_response(&encode_crud_response(method, result))
// is Ok AND validate_crud_result(method, &result) is Ok.
#[test]
fn p_tp_1_crud_encode_validates() {
    let mut rng = Rng::seeded(row_seed(PTP1));
    let mut cases: u32 = 0;
    let mut cexes: Vec<String> = Vec::new();
    for _ in 0..B_TP1 {
        let m = rng.pick(&all_methods());
        let result = gen_result(&mut rng, &m);
        if let Err(e) = validate_crud_result(&m, &result) {
            cexes.push(format!("validate rejected encoder input {m:?}: {e:?}"));
        }
        match decode_crud_response(&encode_crud_response(m.clone(), result.clone())) {
            Ok(_) => {}
            Err(e) => cexes.push(format!(
                "decoder rejected encoder output for {m:?}: {e:?}"
            )),
        }
        cases += 1;
        if cexes.len() >= 5 {
            break;
        }
    }
    assert!(
        cexes.is_empty(),
        "[P-TP-1][strat:crud-encode-validates] BROKEN: {cexes:?}"
    );
    assert!(cases <= B_TP1, "B_TP1 budget exceeded: {cases}");
    println!(
        "[P-TP-1] generated cases: {cases} HELD={}",
        cexes.is_empty()
    );
}
