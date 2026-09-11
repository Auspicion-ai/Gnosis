//! §7.2 F2 — decode-then-validate (the FS-9 / FS-10 realization).
//! Contract: `docs/specs/engine-wire-contract.md` §7.

use crate::store::RagChunk;
use crate::store::{RagResult, RagTrace, StoreError};
use crate::wire::error::from_wire;

/// A decode/validation failure surfaced by the wire layer (§7).
#[derive(Debug, Clone, PartialEq)]
pub enum DecodeError {
    /// serde parse/shape failure.
    InvalidJson(String),
    /// chunk payload `"type"` not in {result, done, error}.
    UnknownType(String),
    /// well-formed result body, no `trace`.
    MissingTrace,
    /// `envelope.schema_version != CURRENT_SCHEMA_VERSION`.
    UnsupportedSchemaVersion(u32),
    /// `envelope.id_format` not a known value.
    UnknownIdFormat(String),
    /// missing `schema_version`/`id_format`/`payload`, or malformed SSE frame.
    InvalidEnvelope(String),
    /// error codec: no variant maps to this code.
    UnknownCode(String),
    /// §7.2 P1a — a CRUD request `"method"` value that names no `CrudMethod`
    /// variant (well-formed JSON, unrecognized method → transport 422).
    UnknownMethod(String),
    /// SSE `event:` line != data `"type"`.
    EventTypeMismatch { event: String, data_type: String },
    /// a valid structural body failed validation.
    ValidationFailed(ValidationFailure),
}

/// Post-decode invariant violations (§7).
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationFailure {
    /// `trace` absent.
    MissingTrace,
    /// `RagResult.engine != "gnosis"`.
    WrongEngine(String),
    /// `blocked_by` is `Some` but trace is not `RagTrace::Graph`.
    BlockedByWithoutGraphTrace,
}

/// Strict decode of a `RagResult` body from a JSON value (decode-then-validate
/// entry): structurally malformed → `InvalidJson`; well-formed but missing
/// `trace` → `MissingTrace`; otherwise `Ok`.
pub fn decode_rag_result(json: &serde_json::Value) -> Result<RagResult, DecodeError> {
    let obj = json
        .as_object()
        .ok_or_else(|| DecodeError::InvalidJson("result body must be a JSON object".to_string()))?;
    // A well-formed result body that lacks a `trace` is the FS-10 route.
    if !obj.contains_key("trace") {
        return Err(DecodeError::MissingTrace);
    }
    serde_json::from_value::<RagResult>(json.clone())
        .map_err(|e| DecodeError::InvalidJson(e.to_string()))
}

/// Post-decode validation of a `RagResult` against the §4.6.1 / §4.3.3
/// invariants (`engine == "gnosis"`, trace present, `blocked_by` ⇒
/// `RagTrace::Graph`).
pub fn validate_rag_result(res: &RagResult) -> Result<(), ValidationFailure> {
    if res.engine != "gnosis" {
        return Err(ValidationFailure::WrongEngine(res.engine.clone()));
    }
    if res.blocked_by.is_some() && !matches!(res.trace, RagTrace::Graph(_)) {
        return Err(ValidationFailure::BlockedByWithoutGraphTrace);
    }
    Ok(())
}

/// Decode the canonical chunk JSON (envelope payload or SSE data line).
pub fn decode_chunk_payload(payload: &serde_json::Value) -> Result<RagChunk, DecodeError> {
    let obj = payload.as_object().ok_or_else(|| {
        DecodeError::InvalidEnvelope("chunk payload must be a JSON object".to_string())
    })?;
    let ty = obj
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DecodeError::UnknownType("payload has no string \"type\"".to_string()))?;
    match ty {
        // §4.2 pins `RagChunk::Done` as exactly `{"type":"done"}` (a single
        // `type` key whose value is "done"). Any extra key — or a duplicated /
        // mis-shapen body — is a malformed done frame (the §13 cross-cutting
        // rule: decodes to `Ok(Done)` only when the payload is exactly
        // `{"type":"done"}`, else `InvalidEnvelope`/`UnknownType`).
        "done" => {
            if obj.len() == 1 {
                Ok(RagChunk::Done)
            } else {
                Err(DecodeError::InvalidEnvelope(
                    "done chunk must be exactly {\"type\":\"done\"}".to_string(),
                ))
            }
        }
        "error" => {
            let code = obj.get("code").and_then(|v| v.as_str()).ok_or_else(|| {
                DecodeError::InvalidJson("error payload missing \"code\"".to_string())
            })?;
            let message = obj.get("message").and_then(|v| v.as_str());
            let err = from_wire(code, message)
                .ok_or_else(|| DecodeError::UnknownCode(code.to_string()))?;
            Ok(RagChunk::Error(err))
        }
        "result" => {
            let body = obj.get("result").ok_or_else(|| {
                DecodeError::InvalidJson("result payload missing \"result\"".to_string())
            })?;
            let res = decode_rag_result(body)?;
            validate_rag_result(&res).map_err(DecodeError::ValidationFailed)?;
            Ok(RagChunk::Result(res))
        }
        other => Err(DecodeError::UnknownType(other.to_string())),
    }
}

/// Map a `DecodeError` to the wire outcome chunk (FS-9 / FS-10).
pub fn outcome_of(e: DecodeError) -> RagChunk {
    match e {
        // FS-10: a well-formed result body with no trace → TraceUnavailable.
        DecodeError::MissingTrace => RagChunk::Error(StoreError::TraceUnavailable),
        // FS-9: every other decode failure → EngineError.
        _ => RagChunk::Error(StoreError::EngineError),
    }
}
