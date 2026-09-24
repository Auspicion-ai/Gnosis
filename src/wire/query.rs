//! §7.2 P2 U2 — the shared `ragQuery` decode seam.
//!
//! Contract: `docs/specs/p2-gnosis-server.md` §5.3/§5.4/§5.5 and
//! `docs/specs/engine-wire-contract.md` §4.5/§7.1.
//!
//! The signatures are the ones `p2` §9.5.1's surface table pins; the behavior:
//!
//! - `decode_query_request` is envelope-strict on the POST path (`schemaVersion`
//!   first, then `idFormat`, then an object `payload`), maps §5.3's 14 camelCase
//!   keys onto `RagQueryOptions` (wrongly-typed ⇒ the store `Option`'s `None`,
//!   never a decoder-supplied default — §5.3's two-layer bullet), maps `filters`
//!   member-by-member off the canonical §4.5.2 shape, and resolves the three
//!   enum tokens through the one resolver per family (§5.4);
//! - the three resolvers are the single ASCII-case-insensitive token rule: a
//!   member string ⇒ the typed enum, any other string ⇒ `ValidationError`;
//! - `request_decode_code` maps exactly §5.5's five request-decode variants;
//! - `encode_result_checked` never emits a body the result validator rejects.

use serde_json::Value;

use crate::store::{
    CompressionMode, EdgeKind, ExpandMode, MultiQueryOptions, NodeKind, QueryAuditFilters,
    QueryMode, RagQueryOptions, RagResult, ReferenceState, StoreError, SubTaskDagOptions,
};
use crate::wire::decode::{validate_rag_result, DecodeError};
use crate::wire::envelope::{Envelope, CURRENT_SCHEMA_VERSION, ID_FORMAT_OPAQUE_STRING_V1};

/// Which surface the shared query decoder is serving (§9.5.1's F6 signature
/// note): the POST body path or the SSE query-param path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryPath {
    Post,
    Sse,
}

/// The SSE half of the decoder's input — one `Option<&str>` per token the
/// pre-U4 SSE surface reads (`query`/`topK`/`mode`, §5.4's N1).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SseParams<'a> {
    pub query: Option<&'a str>,
    pub top_k: Option<&'a str>,
    pub mode: Option<&'a str>,
}

/// The decode-level error type (§9.5.1): a transport failure renders through the
/// shared decode-error renderer (400/422 + its transport code, §5.5); a
/// validation failure is the §11-mapped `ValidationError` (400
/// `validation_error`) the token resolver reports through (§5.4).
#[derive(Debug, Clone, PartialEq)]
pub enum QueryDecodeError {
    Transport(DecodeError),
    Validation(StoreError),
}

/// The shared `ragQuery` decoder (§5.3): envelope-strict on the POST path, the
/// camelCase 14-key payload mapped onto `RagQueryOptions`, the `mode` token
/// resolved through the one resolver on both paths.
pub fn decode_query_request(
    path: QueryPath,
    env: &Envelope,
    sse_params: SseParams<'_>,
) -> Result<(String, RagQueryOptions), QueryDecodeError> {
    match path {
        QueryPath::Post => decode_query_post(env),
        QueryPath::Sse => decode_query_sse(sse_params),
    }
}

/// §5.3 — the POST path: the envelope check (version → id format → object
/// payload) precedes every payload reading; the token resolvers run inside the
/// decode, i.e. before the engine's `validate_rag_options` (§5.3's precedence).
fn decode_query_post(env: &Envelope) -> Result<(String, RagQueryOptions), QueryDecodeError> {
    if env.schema_version != CURRENT_SCHEMA_VERSION {
        return Err(QueryDecodeError::Transport(
            DecodeError::UnsupportedSchemaVersion(env.schema_version),
        ));
    }
    if env.id_format != ID_FORMAT_OPAQUE_STRING_V1 {
        return Err(QueryDecodeError::Transport(DecodeError::UnknownIdFormat(
            env.id_format.clone(),
        )));
    }
    let payload = env.payload.as_object().ok_or_else(|| {
        QueryDecodeError::Transport(DecodeError::InvalidEnvelope(
            "envelope payload must be a JSON object".to_string(),
        ))
    })?;

    // `query` is the decoder's separate return half and one of §5.3's two `Err`
    // exceptions: present-but-non-string ⇒ `Err(Validation(_))` (the pinned
    // per-key table), absent ⇒ `""` (the engine's FS-3 400 `validation_error`).
    let query = match payload.get("query") {
        None => String::new(),
        Some(Value::String(s)) => s.clone(),
        Some(_) => {
            return Err(QueryDecodeError::Validation(StoreError::ValidationError(
                "query must be a non-empty string".to_string(),
            )))
        }
    };

    // §5.4 — only a JSON string reaches a token resolver; every other value
    // (absent / non-string) decodes as absent ⇒ the store field stays `None`.
    let mode = token(payload.get("mode").and_then(Value::as_str), |raw| {
        resolve_query_mode(raw)
    })?;
    let expand = token(payload.get("expand").and_then(Value::as_str), |raw| {
        resolve_expand_mode(raw)
    })?;
    let compression = token(payload.get("compression").and_then(Value::as_str), |raw| {
        resolve_compression_mode(raw)
    })?;

    let options = RagQueryOptions {
        wiki_id: payload
            .get("wikiId")
            .and_then(Value::as_str)
            .map(|s| crate::store::WikiId(s.to_string())),
        top_k: payload.get("topK").and_then(Value::as_u64),
        // §5.3 — `filters` is the one schema-checked option: malformed ⇒ 400.
        filters: match payload.get("filters") {
            None | Some(Value::Null) => None,
            Some(v) => Some(parse_filters(v).map_err(QueryDecodeError::Validation)?),
        },
        mode,
        max_hops: payload.get("maxHops").and_then(Value::as_u64),
        expand,
        max_parent_context: payload.get("maxParentContext").and_then(Value::as_u64),
        multi_query: object_option(payload.get("multiQuery")).map(|(enabled, n)| {
            MultiQueryOptions {
                enabled,
                // an omitted `n` takes its documented default (3); a present
                // wrongly-typed `n` is not well-formed ⇒ the whole field is `None`
                n: n.unwrap_or(3),
            }
        }),
        compression,
        hyde: payload.get("hyde").and_then(Value::as_bool),
        binary_first_pass: payload.get("binaryFirstPass").and_then(Value::as_bool),
        binary_candidate_pool: payload.get("binaryCandidatePool").and_then(Value::as_u64),
        sub_task_dag: object_option(payload.get("subTaskDag"))
            .map(|(enabled, _)| SubTaskDagOptions { enabled }),
        // §5.3 — `requester` is not part of this request contract: never set.
        requester: None,
    };
    Ok((query, options))
}

/// §5.4 — the SSE path reads exactly `query`/`topK`/`mode` (N1); `env` is the
/// synthesized canonical envelope the handler builds and carries no payload.
fn decode_query_sse(
    sse_params: SseParams<'_>,
) -> Result<(String, RagQueryOptions), QueryDecodeError> {
    // An unparseable `topK` collapses onto the same `None` as an absent param
    // (§5.4's `topK`-unchanged note); `mode` goes through the one resolver.
    let options = RagQueryOptions {
        top_k: sse_params.top_k.and_then(|s| s.parse::<u64>().ok()),
        mode: token(sse_params.mode, resolve_query_mode)?,
        ..RagQueryOptions::default()
    };
    Ok((sse_params.query.unwrap_or("").to_string(), options))
}

/// Resolve one token input: absent ⇒ `None` (no resolver call), else the
/// resolver's outcome as the decode-level error (§5.4).
fn token<T, F>(raw: Option<&str>, resolve: F) -> Result<Option<T>, QueryDecodeError>
where
    F: FnOnce(Option<&str>) -> Result<T, StoreError>,
{
    match raw {
        None => Ok(None),
        Some(s) => resolve(Some(s))
            .map(Some)
            .map_err(QueryDecodeError::Validation),
    }
}

/// §5.3's rule for the two **object-valued** option keys (`multiQuery`,
/// `subTaskDag`), read at the **field** level (the two-layer bullet's
/// object-valued-option clause): a non-object — or an object with a **documented
/// member present and wrongly typed** (`enabled` for both keys, or `n` for
/// `multiQuery`) — decodes as **absent** ⇒ the field stays `None`. The decoder
/// never fabricates a default option object, and never answers with a present
/// object whose member it defaulted inside. A well-formed object yields its
/// member mapping, where the **member-level** defaults apply only **inside** such
/// an object: an omitted `enabled` reads as disabled and an omitted `n` as
/// `None` (the call site supplies its pinned `3`). Extra members are tolerated.
fn object_option(v: Option<&Value>) -> Option<(bool, Option<u64>)> {
    let obj = v?.as_object()?;
    let enabled = match obj.get("enabled") {
        Some(Value::Bool(enabled)) => *enabled,
        None => false,
        // a present wrongly-typed documented member ⇒ the whole option is absent
        Some(_) => return None,
    };
    let n = match obj.get("n") {
        // a present wrongly-typed documented member ⇒ the whole option is absent
        Some(member) => Some(member.as_u64()?),
        None => None,
    };
    Some((enabled, n))
}

/// §5.3 — the canonical §4.5.2 `filters` mapping, member-by-member onto the
/// frozen store types (never a `serde_json` pass-through into
/// `QueryAuditFilters`, whose camelCase keys could not bind and which has no
/// `deny_unknown_fields`: that would silently yield an all-`None` filter).
/// Unknown members are ignored; `{}` ⇒ an all-`None` (present, valid) filter.
fn parse_filters(v: &Value) -> Result<QueryAuditFilters, StoreError> {
    let invalid = |m: &str| StoreError::ValidationError(m.to_string());
    let obj = v
        .as_object()
        .ok_or_else(|| invalid("filters must be a JSON object"))?;
    let mut filters = QueryAuditFilters {
        node_kind: None,
        edge_type: None,
        target: None,
        state: None,
    };
    if let Some(member) = obj.get("nodeKind") {
        let raw = member
            .as_str()
            .ok_or_else(|| invalid("filters.nodeKind must be a string"))?;
        filters.node_kind = Some(match raw.to_ascii_lowercase().as_str() {
            "content" => NodeKind::Content,
            "fact" => NodeKind::Fact,
            "reference" => NodeKind::Reference,
            _ => {
                return Err(invalid(
                    "filters.nodeKind must be content, fact, or reference",
                ))
            }
        });
    }
    if let Some(member) = obj.get("edgeType") {
        let raw = member
            .as_str()
            .ok_or_else(|| invalid("filters.edgeType must be a string"))?;
        filters.edge_type = Some(match raw.to_ascii_lowercase().as_str() {
            "link" => EdgeKind::Link,
            "embed" => EdgeKind::Embed,
            "crosslink" => EdgeKind::Crosslink,
            _ => {
                return Err(invalid(
                    "filters.edgeType must be link, embed, or crosslink",
                ))
            }
        });
    }
    if let Some(member) = obj.get("state") {
        let raw = member
            .as_str()
            .ok_or_else(|| invalid("filters.state must be a string"))?;
        filters.state = Some(match raw.to_ascii_uppercase().as_str() {
            "FRESH" => ReferenceState::Fresh,
            "RESOLVED" => ReferenceState::Resolved,
            "STALE" => ReferenceState::Stale,
            "BROKEN" => ReferenceState::Broken,
            _ => {
                return Err(invalid(
                    "filters.state must be FRESH, RESOLVED, STALE, or BROKEN",
                ))
            }
        });
    }
    if let Some(member) = obj.get("target") {
        let t = member
            .as_object()
            .ok_or_else(|| invalid("filters.target must be an object"))?;
        let document_id = t.get("documentId").and_then(Value::as_str);
        let node_id = t.get("nodeId").and_then(Value::as_str);
        match (document_id, node_id) {
            (Some(d), Some(n)) => {
                filters.target = Some((
                    crate::store::DocumentId(d.to_string()),
                    crate::store::NodeId(n.to_string()),
                ))
            }
            _ => {
                return Err(invalid(
                    "filters.target must carry string documentId and nodeId",
                ))
            }
        }
    }
    Ok(filters)
}

/// §5.4's single mode rule, shared by the POST and SSE paths.
pub fn resolve_query_mode(raw: Option<&str>) -> Result<QueryMode, StoreError> {
    let Some(s) = raw else {
        // Absent ⇒ the engine's query-time reading (`Flat`); the decoder writes
        // no default into the options (§5.3's two-layer bullet).
        return Ok(QueryMode::Flat);
    };
    match s.to_ascii_lowercase().as_str() {
        "flat" => Ok(QueryMode::Flat),
        "graph" => Ok(QueryMode::Graph),
        "vector" => Ok(QueryMode::Vector),
        "hybrid" => Ok(QueryMode::Hybrid),
        _ => Err(StoreError::ValidationError(format!(
            "mode must be flat, graph, vector, or hybrid (got {s:?})"
        ))),
    }
}

/// §5.4's `expand` token resolver (POST payload only).
pub fn resolve_expand_mode(raw: Option<&str>) -> Result<ExpandMode, StoreError> {
    let Some(s) = raw else {
        return Ok(ExpandMode::None);
    };
    match s.to_ascii_lowercase().as_str() {
        "none" => Ok(ExpandMode::None),
        "parent" => Ok(ExpandMode::Parent),
        _ => Err(StoreError::ValidationError(format!(
            "expand must be none or parent (got {s:?})"
        ))),
    }
}

/// §5.4's `compression` token resolver (POST payload only).
pub fn resolve_compression_mode(raw: Option<&str>) -> Result<CompressionMode, StoreError> {
    let Some(s) = raw else {
        return Ok(CompressionMode::None);
    };
    match s.to_ascii_lowercase().as_str() {
        "none" => Ok(CompressionMode::None),
        "filter" => Ok(CompressionMode::Filter),
        "extract" => Ok(CompressionMode::Extract),
        "graph" => Ok(CompressionMode::Graph),
        _ => Err(StoreError::ValidationError(format!(
            "compression must be none, filter, extract, or graph (got {s:?})"
        ))),
    }
}

/// §5.5's transport code mapping — total over the same 5-variant domain as
/// `request_decode_status`, `None` outside it.
pub fn request_decode_code(e: &DecodeError) -> Option<&'static str> {
    use DecodeError::*;
    let code = match e {
        InvalidJson(_) => "invalid_json",
        InvalidEnvelope(_) => "invalid_envelope",
        UnsupportedSchemaVersion(_) => "unsupported_schema_version",
        UnknownIdFormat(_) => "unknown_id_format",
        UnknownMethod(_) => "unknown_method",
        _ => return None,
    };
    Some(code)
}

/// §5.5's `message` rule: the carried string verbatim (which MAY be empty) for
/// the four string-carrying variants; the decimal render for
/// `UnsupportedSchemaVersion`.
pub fn request_decode_message(e: &DecodeError) -> String {
    match e {
        DecodeError::InvalidJson(m)
        | DecodeError::InvalidEnvelope(m)
        | DecodeError::UnknownIdFormat(m)
        | DecodeError::UnknownMethod(m) => m.clone(),
        DecodeError::UnsupportedSchemaVersion(v) => format!("unsupported schemaVersion: {v}"),
        other => format!("{other:?}"),
    }
}

/// §5.3's encoder/validator rule: the checked result encoder never emits a body
/// the result validator rejects (a rejected result is `StoreError::EngineError`
/// → 502, FS-9).
pub fn encode_result_checked(res: &RagResult) -> Result<Envelope, StoreError> {
    validate_rag_result(res).map_err(|_| StoreError::EngineError)?;
    Ok(crate::wire::codecs::encode_result(res))
}
