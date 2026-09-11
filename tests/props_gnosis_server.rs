//! §7.2 P2 — `gnosis-server` binary crate — **property-based-testing (PBT) gate —
//! executed layer** (`tests/props_gnosis_server.rs`).
//!
//! Implements **every** register row of `docs/specs/p2-gnosis-server.md` §5.x
//! (7 rows: P-IM-1..3, P-SM-1..3, P-TP-1) as **one `#[test]` per row**, each
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
//! | P-TP-1 | 40     |        |        |
//!
//! Sum = **310** generated cases (≤ 400). stop-after-5: a row aborts and reports
//! at most 5 distinct counterexamples.
//!
//! **RED-stage.** The P2 `gnosis-server` bin does NOT exist yet — the pure
//! server fns `gnosis::server_status`, `gnosis::request_decode_status` and
//! `gnosis::route_bijection` are ABSENT, so this suite FAILS TO COMPILE (the
//! missing-symbol red set). Every row is therefore **broken/compile-fail** until
//! the Implementer lands the least server code to green.

use gnosis::wire::crud::{
    decode_crud_request, decode_crud_response, encode_crud_error, encode_crud_request,
    encode_crud_response, CrudMethod, CrudRequestArgs, CrudResult, ENGINE_ENDPOINTS,
};
use gnosis::wire::decode::DecodeError;
use gnosis::{
    request_decode_status, route_bijection, server_status, CreateDocumentRequest, DocState,
    Document, DocumentId, DocumentList, DocumentSummary, Edge, EdgeKind, Graph,
    ListDocumentsFilter, Node, NodeId, NodeKind, ReferenceState, StoreError,
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
const PSM1: u64 = 0x50534D31; // "PSM1"
const PSM2: u64 = 0x50534D32; // "PSM2"
const PSM3: u64 = 0x50534D33; // "PSM3"
const PTP1: u64 = 0x50545031; // "PTP1"

/// Per-row budget caps (sum = 310 ≤ 400).
const B_IM1: u32 = 60;
const B_IM2: u32 = 40;
const B_IM3: u32 = 60;
const B_SM1: u32 = 40;
const B_SM2: u32 = 30;
const B_SM3: u32 = 40;
const B_TP1: u32 = 40;

// ---------------------------------------------------------------------------
// Fixtures / generators.
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

/// The 7 CRUD-reachable `StoreError` variant types (§7.1 / P1a §6.1), with the
/// `ValidationError` boundary `m` corpus (empty / single-byte / multi-codepoint)
/// per the register's generator-coverage note — the code stays `"validation_error"`.
fn crud_reachable_errors() -> Vec<StoreError> {
    vec![
        StoreError::DocumentNotFound,
        StoreError::WikiNotFound,
        StoreError::ValidationError("".to_string()),
        StoreError::ValidationError("x".to_string()),
        StoreError::ValidationError("漢字😀".to_string()),
        StoreError::ConflictError,
        StoreError::DocumentInUse,
        StoreError::InvalidState,
        StoreError::UnresolvedReference,
    ]
}

/// The 5 request-decode `DecodeError` variant types (§7.2 / P1a §6.2), with the
/// `UnknownMethod` boundary method-string corpus (empty / arbitrary / multi-
/// codepoint) per the register's generator-coverage note — always 422.
fn decode_error_variants() -> Vec<DecodeError> {
    vec![
        DecodeError::InvalidJson("bad json".to_string()),
        DecodeError::InvalidEnvelope("missing args".to_string()),
        DecodeError::UnknownMethod("".to_string()),
        DecodeError::UnknownMethod("bogus".to_string()),
        DecodeError::UnknownMethod("漢字😀".to_string()),
        DecodeError::UnsupportedSchemaVersion(99),
        DecodeError::UnknownIdFormat("uuid-v4".to_string()),
    ]
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

/// Boundary caller corpus (§5.x P-SM-3 generator-coverage note).
const CALLERS: &[&str] = &[
    "",
    "user:alice",
    "漢字😀",
    "caller",
    "body",
    "user:caller/body",
];

/// Boundary opaque-id corpus.
const IDS: &[&str] = &["", "d1", "550e8400-e29b-41d4-a716-446655440000", "漢字"];

/// Boundary title/name corpus (incl. a 200-char boundary string).
fn titles() -> Vec<String> {
    vec![
        "".to_string(),
        "a".to_string(),
        "漢字😀".to_string(),
        "x".repeat(200),
    ]
}

fn gen_caller(rng: &mut Rng) -> String {
    rng.pick(CALLERS).to_string()
}

fn gen_title(rng: &mut Rng) -> String {
    let t = titles();
    t[rng.below(t.len() as u64) as usize].clone()
}

fn gen_id(rng: &mut Rng) -> String {
    rng.pick(IDS).to_string()
}

fn gen_state(rng: &mut Rng) -> DocState {
    rng.pick(&[DocState::Draft, DocState::Published, DocState::Archived])
}

fn gen_tags(rng: &mut Rng) -> Option<Vec<String>> {
    match rng.below(4) {
        0 => None,
        1 => Some(vec![]),
        2 => Some(vec!["guide".to_string()]),
        _ => Some(vec!["a".to_string(), "漢字".to_string(), "c".to_string()]),
    }
}

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

/// A random `revision` over {0, non-zero}.
fn gen_revision(rng: &mut Rng) -> u64 {
    rng.pick(&[0u64, 1, 7, u64::MAX])
}

/// A random ISO-8601 timestamp string.
fn gen_timestamp(rng: &mut Rng) -> String {
    rng.pick(&[
        "2026-09-09T00:00:00Z",
        "2024-01-01T12:30:45Z",
        "2026-12-31T23:59:59Z",
    ])
    .to_string()
}

/// A random well-formed `Document` with the given `revision`/`state`.
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
        CreateDocument => CrudResult::Document(gen_doc(rng, 0, DocState::Draft)),
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

// ---------------------------------------------------------------------------
// P-IM-1 (IM) — strat:status-total — §11 status-mapping is total over the
// CRUD-reachable `StoreError` variants.
// ---------------------------------------------------------------------------
//
// Invariant: ∀ e in the 7 CRUD-reachable variants: `server_status(e)` is
// `Some((status, code))` with `status` in the §11 map and `code == e.wire_code()`;
// equal variants → equal `(status, code)`.
#[test]
fn p_im_1_status_total() {
    let mut rng = Rng::seeded(row_seed(PIM1));
    let errors = crud_reachable_errors();
    let mut cases: u32 = 0;
    let mut cexes: Vec<String> = Vec::new();
    for _ in 0..B_IM1 {
        // Enumerate each of the 7 variants at least once, then random picks.
        let e = if cases < errors.len() as u32 {
            errors[cases as usize].clone()
        } else {
            errors[rng.below(errors.len() as u64) as usize].clone()
        };
        match server_status(&e) {
            Some((status, code)) => {
                if code != e.wire_code() {
                    cexes.push(format!(
                        "wire-code mismatch for {e:?}: server {code} != wire_code {}",
                        e.wire_code()
                    ));
                }
                // Determinism: equal variants → equal (status, code).
                if server_status(&e) != Some((status, code)) {
                    cexes.push(format!("non-deterministic server_status for {e:?}"));
                }
            }
            None => cexes.push(format!("CRUD-reachable {e:?} unmapped")),
        }
        cases += 1;
        if cexes.len() >= 5 {
            break;
        }
    }
    assert!(
        cexes.is_empty(),
        "[P-IM-1][strat:status-total] BROKEN: {cexes:?}"
    );
    assert!(cases <= B_IM1, "B_IM1 budget exceeded: {cases}");
    println!(
        "[P-IM-1] generated cases: {cases} HELD={}",
        cexes.is_empty()
    );
}

// ---------------------------------------------------------------------------
// P-IM-2 (IM) — strat:decode-outcome-total — request-decode outcome mapping is
// total over the 5 `DecodeError` variants.
// ---------------------------------------------------------------------------
//
// Invariant: ∀ e: DecodeError (the 5 variants): `request_decode_status(e)` is
// `Some(status)` with `status ∈ {400, 422}`; `status != 502`; equal variants →
// equal status.
#[test]
fn p_im_2_decode_outcome_total() {
    let mut rng = Rng::seeded(row_seed(PIM2));
    let errors = decode_error_variants();
    let mut cases: u32 = 0;
    let mut cexes: Vec<String> = Vec::new();
    for _ in 0..B_IM2 {
        let e = if cases < errors.len() as u32 {
            errors[cases as usize].clone()
        } else {
            errors[rng.below(errors.len() as u64) as usize].clone()
        };
        match request_decode_status(&e) {
            Some(status) => {
                if status != 400 && status != 422 {
                    cexes.push(format!("status {status} for {e:?} not in {{400,422}}"));
                }
                if status == 502 {
                    cexes.push(format!("request-decode status for {e:?} is 502"));
                }
                if request_decode_status(&e) != Some(status) {
                    cexes.push(format!("non-deterministic request_decode_status for {e:?}"));
                }
            }
            None => cexes.push(format!("DecodeError {e:?} unmapped")),
        }
        cases += 1;
        if cexes.len() >= 5 {
            break;
        }
    }
    assert!(
        cexes.is_empty(),
        "[P-IM-2][strat:decode-outcome-total] BROKEN: {cexes:?}"
    );
    assert!(cases <= B_IM2, "B_IM2 budget exceeded: {cases}");
    println!(
        "[P-IM-2] generated cases: {cases} HELD={}",
        cexes.is_empty()
    );
}

// ---------------------------------------------------------------------------
// P-IM-3 (IM) — strat:route-bijection — endpoint routing is a bijection.
// ---------------------------------------------------------------------------
//
// Invariant: ∀ distinct (path_a, handler_a), (path_b, handler_b) in the routing
// table: path_a != path_b; handler_a != handler_b; the table has exactly 14
// rows, one per endpoint. The 11 CRUD paths equal the P1a `ENGINE_ENDPOINTS`
// paths verbatim.
#[test]
fn p_im_3_route_bijection() {
    let mut rng = Rng::seeded(row_seed(PIM3));
    let mut cases: u32 = 0;
    let mut cexes: Vec<String> = Vec::new();
    for _ in 0..B_IM3 {
        let table = route_bijection();
        if table.len() != 14 {
            cexes.push(format!("routing table has {} rows, expected 14", table.len()));
        }
        let mut paths: Vec<&str> = Vec::new();
        let mut handlers: Vec<&str> = Vec::new();
        for (path, handler) in table {
            if path.is_empty() {
                cexes.push("empty path".to_string());
            }
            if handler.is_empty() {
                cexes.push(format!("empty handler for {path}"));
            }
            if paths.contains(path) {
                cexes.push(format!("duplicate path {path}"));
            }
            if handlers.contains(handler) {
                cexes.push(format!("duplicate handler {handler}"));
            }
            paths.push(path);
            handlers.push(handler);
        }
        // The 11 CRUD paths equal the P1a ENGINE_ENDPOINTS paths verbatim.
        let engine_paths: Vec<&str> = ENGINE_ENDPOINTS.iter().map(|(p, _)| *p).collect();
        let crud_paths: Vec<&str> = paths
            .iter()
            .copied()
            .filter(|p| engine_paths.contains(p))
            .collect();
        if crud_paths.len() != 11 {
            cexes.push(format!("{} CRUD paths in routing table, expected 11", crud_paths.len()));
        }
        for p in &engine_paths {
            if !crud_paths.contains(p) {
                cexes.push(format!("CRUD path {p} missing from routing table"));
            }
        }
        cases += 1;
        if cexes.len() >= 5 {
            break;
        }
    }
    assert!(
        cexes.is_empty(),
        "[P-IM-3][strat:route-bijection] BROKEN: {cexes:?}"
    );
    assert!(cases <= B_IM3, "B_IM3 budget exceeded: {cases}");
    println!(
        "[P-IM-3] generated cases: {cases} HELD={}",
        cexes.is_empty()
    );
}

// ---------------------------------------------------------------------------
// P-SM-1 (SM) — strat:status-determinism — status-mapping determinism +
// wire-code fidelity.
// ---------------------------------------------------------------------------
//
// Invariant: ∀ e in the 7 CRUD-reachable variants: `server_status(e) ==
// server_status(e)` on repeat; `server_status(e).code == e.wire_code()`.
#[test]
fn p_sm_1_status_determinism() {
    let mut rng = Rng::seeded(row_seed(PSM1));
    let errors = crud_reachable_errors();
    let mut cases: u32 = 0;
    let mut cexes: Vec<String> = Vec::new();
    for _ in 0..B_SM1 {
        let e = if cases < errors.len() as u32 {
            errors[cases as usize].clone()
        } else {
            errors[rng.below(errors.len() as u64) as usize].clone()
        };
        let first = server_status(&e);
        let second = server_status(&e);
        if first != second {
            cexes.push(format!("server_status({e:?}) not stable: {first:?} vs {second:?}"));
        }
        if let Some((_, code)) = first {
            if code != e.wire_code() {
                cexes.push(format!(
                    "wire-code fidelity for {e:?}: server {code} != {}",
                    e.wire_code()
                ));
            }
        } else {
            cexes.push(format!("CRUD-reachable {e:?} unmapped"));
        }
        cases += 1;
        if cexes.len() >= 5 {
            break;
        }
    }
    assert!(
        cexes.is_empty(),
        "[P-SM-1][strat:status-determinism] BROKEN: {cexes:?}"
    );
    assert!(cases <= B_SM1, "B_SM1 budget exceeded: {cases}");
    println!(
        "[P-SM-1] generated cases: {cases} HELD={}",
        cexes.is_empty()
    );
}

// ---------------------------------------------------------------------------
// P-SM-2 (SM) — strat:decode-outcome-determinism — request-decode outcome
// determinism + the 400/422 split is stable.
// ---------------------------------------------------------------------------
//
// Invariant: ∀ e: DecodeError (the 5 variants): `request_decode_status(e) ==
// request_decode_status(e)` on repeat; the variant→status mapping is a fixed
// pure function.
#[test]
fn p_sm_2_decode_outcome_determinism() {
    let mut rng = Rng::seeded(row_seed(PSM2));
    let errors = decode_error_variants();
    let mut cases: u32 = 0;
    let mut cexes: Vec<String> = Vec::new();
    for _ in 0..B_SM2 {
        let e = if cases < errors.len() as u32 {
            errors[cases as usize].clone()
        } else {
            errors[rng.below(errors.len() as u64) as usize].clone()
        };
        let first = request_decode_status(&e);
        let second = request_decode_status(&e);
        if first != second {
            cexes.push(format!(
                "request_decode_status({e:?}) not stable: {first:?} vs {second:?}"
            ));
        }
        if let Some(status) = first {
            if status != 400 && status != 422 {
                cexes.push(format!("status {status} for {e:?} not in {{400,422}}"));
            }
        } else {
            cexes.push(format!("DecodeError {e:?} unmapped"));
        }
        cases += 1;
        if cexes.len() >= 5 {
            break;
        }
    }
    assert!(
        cexes.is_empty(),
        "[P-SM-2][strat:decode-outcome-determinism] BROKEN: {cexes:?}"
    );
    assert!(cases <= B_SM2, "B_SM2 budget exceeded: {cases}");
    println!(
        "[P-SM-2] generated cases: {cases} HELD={}",
        cexes.is_empty()
    );
}

// ---------------------------------------------------------------------------
// P-SM-3 (SM) — strat:caller-threaded — the decode-layer caller validation.
// ---------------------------------------------------------------------------
//
// Invariant (re-scoped §8): the server threads the `caller` through the request
// decode layer only (engine-side RBAC enforcement is a PACKAGE item). The decode
// layer requires the `caller` on mutating methods (a mutating request with no
// `caller` → request-decode 400) and tolerates a `caller` on a read-only request
// (the read-only args carry no `caller` field; an extra one is ignored).
#[test]
fn p_sm_3_caller_threaded() {
    let mut rng = Rng::seeded(row_seed(PSM3));
    let mut cases: u32 = 0;
    let mut cexes: Vec<String> = Vec::new();
    for _ in 0..B_SM3 {
        // Mutating: a request with no `caller` → request-decode 400 (the decode
        // layer requires the caller on mutating methods).
        let m = rng.pick(&mutating_methods());
        let args = gen_args(&mut rng, &m);
        let mut env = encode_crud_request(m.clone(), args.clone());
        if let Some(args_obj) = env.payload.get_mut("args").and_then(|v| v.as_object_mut()) {
            args_obj.remove("caller");
        }
        match decode_crud_request(&env) {
            Ok(_) => cexes.push(format!(
                "mutating {m:?} with no caller decoded OK (must be request-decode 400)"
            )),
            Err(_) => {}
        }
        // Read-only (the arg-carrying methods): a `caller` is tolerated (decode
        // succeeds; the read-only args carry no caller field, so it is ignored).
        let ro = rng.pick(&[
            CrudMethod::GetDocument,
            CrudMethod::ListDocuments,
            CrudMethod::GetWiki,
        ]);
        let ro_args = gen_args(&mut rng, &ro);
        let mut env = encode_crud_request(ro.clone(), ro_args.clone());
        if let Some(args_obj) = env.payload.get_mut("args").and_then(|v| v.as_object_mut()) {
            args_obj.insert("caller".to_string(), serde_json::json!("user:alice"));
        }
        match decode_crud_request(&env) {
            Ok(_) => {}
            Err(e) => cexes.push(format!("read-only {ro:?} with a caller rejected: {e:?}")),
        }
        cases += 1;
        if cexes.len() >= 5 {
            break;
        }
    }
    assert!(
        cexes.is_empty(),
        "[P-SM-3][strat:caller-threaded] BROKEN: {cexes:?}"
    );
    assert!(cases <= B_SM3, "B_SM3 budget exceeded: {cases}");
    println!(
        "[P-SM-3] generated cases: {cases} HELD={}",
        cexes.is_empty()
    );
}

// ---------------------------------------------------------------------------
// P-TP-1 (TP) — strat:server-encode-validates — the server never emits a
// response envelope the client decoder rejects.
// ---------------------------------------------------------------------------
//
// Invariant: ∀ CRUD method + well-formed result/error:
// `decode_crud_response(&server_response(method, result_or_error)).is_ok()`.
#[test]
fn p_tp_1_server_encode_validates() {
    let mut rng = Rng::seeded(row_seed(PTP1));
    let errors = crud_reachable_errors();
    let mut cases: u32 = 0;
    let mut cexes: Vec<String> = Vec::new();
    for _ in 0..B_TP1 {
        let m = rng.pick(&all_methods());
        // A well-formed result envelope must be accepted by the client decoder.
        let result = gen_result(&mut rng, &m);
        match decode_crud_response(&encode_crud_response(m.clone(), result.clone())) {
            Ok(_) => {}
            Err(e) => cexes.push(format!(
                "decoder rejected server result envelope for {m:?}: {e:?}"
            )),
        }
        // A representative error envelope must be accepted by the client decoder.
        let e = errors[rng.below(errors.len() as u64) as usize].clone();
        match decode_crud_response(&encode_crud_error(m.clone(), &e)) {
            Err(gnosis::wire::crud::CrudResponseError::Store(_)) => {}
            other => cexes.push(format!(
                "decoder rejected server error envelope for {m:?}/{e:?}: {other:?}"
            )),
        }
        cases += 1;
        if cexes.len() >= 5 {
            break;
        }
    }
    assert!(
        cexes.is_empty(),
        "[P-TP-1][strat:server-encode-validates] BROKEN: {cexes:?}"
    );
    assert!(cases <= B_TP1, "B_TP1 budget exceeded: {cases}");
    println!(
        "[P-TP-1] generated cases: {cases} HELD={}",
        cexes.is_empty()
    );
}
