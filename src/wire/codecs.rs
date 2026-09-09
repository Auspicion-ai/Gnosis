//! §7.2 F2 — JSON codecs for `RagChunk`, `StoreError`, `RagResult` (all
//! envelope-wrapped). Contract: `docs/specs/engine-wire-contract.md` §6.

use serde_json::Value;

use crate::store::{RagChunk, RagResult, StoreError};
use crate::wire::decode::{self, DecodeError};
use crate::wire::envelope::{current_schema_version, Envelope, ID_FORMAT_OPAQUE_STRING_V1};
use crate::wire::error::from_wire;

/// Internal serialization error (only reachable on a non-object payload map the
/// codecs never produce — a defensive enum, not a tested fail-path).
#[derive(Debug)]
pub enum WireCodecError {
    /// A serde serialization failure.
    Serialize(String),
}

impl std::fmt::Display for WireCodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "wire codec error: {}",
            match self {
                WireCodecError::Serialize(s) => s,
            }
        )
    }
}

impl std::error::Error for WireCodecError {}

/// Validate that the envelope's `schema_version`/`id_format` are the current,
/// supported values (a decode/transport concern, per §4.1/§10).
fn check_envelope(env: &Envelope) -> Result<(), DecodeError> {
    if env.schema_version != current_schema_version() {
        return Err(DecodeError::UnsupportedSchemaVersion(env.schema_version));
    }
    if env.id_format != ID_FORMAT_OPAQUE_STRING_V1 {
        return Err(DecodeError::UnknownIdFormat(env.id_format.clone()));
    }
    Ok(())
}

/// Build the canonical chunk JSON value (§4.2 — the discriminative `"type"`).
/// This is the single source of the chunk body, shared by `encode_chunk` and the
/// SSE `data:` line.
pub(crate) fn chunk_payload(chunk: &RagChunk) -> Value {
    match chunk {
        RagChunk::Result(r) => serde_json::json!({ "type": "result", "result": r }),
        RagChunk::Done => serde_json::json!({ "type": "done" }),
        RagChunk::Error(e) => serde_json::json!({
            "type": "error",
            "code": e.wire_code(),
            "message": format!("{}", e),
        }),
    }
}

/// Encode a `RagChunk` into an envelope (payload = canonical chunk JSON).
pub fn encode_chunk(chunk: &RagChunk) -> Envelope {
    Envelope::with_payload(chunk_payload(chunk))
}

/// Decode an envelope back to the chunk (decode-then-validate; result-body
/// failures map into `EngineError`/`TraceUnavailable` routes).
pub fn decode_chunk(env: &Envelope) -> Result<RagChunk, DecodeError> {
    check_envelope(env)?;
    decode::decode_chunk_payload(&env.payload)
}

/// Standalone `StoreError` codec (payload = `{"code","message"}` envelope).
pub fn encode_error(err: &StoreError) -> Envelope {
    Envelope::with_payload(serde_json::json!({
        "code": err.wire_code(),
        "message": format!("{}", err),
    }))
}

/// Decode a standalone error envelope back to the `StoreError`.
pub fn decode_error(env: &Envelope) -> Result<StoreError, DecodeError> {
    check_envelope(env)?;
    let obj = env.payload.as_object().ok_or_else(|| {
        DecodeError::InvalidJson("error payload must be a JSON object".to_string())
    })?;
    let code = obj.get("code").and_then(|c| c.as_str()).ok_or_else(|| {
        DecodeError::InvalidJson("error payload missing string \"code\"".to_string())
    })?;
    let message = obj.get("message").and_then(|m| m.as_str());
    from_wire(code, message).ok_or_else(|| DecodeError::UnknownCode(code.to_string()))
}

/// Encode a `RagResult` into an envelope (payload = the serde result body).
pub fn encode_result(res: &RagResult) -> Envelope {
    // `to_value` cannot fail here: `res` is engine-produced (never untrusted),
    // all fields serialize, and the property register excludes NaN/infinite f64
    // scores, so serde has nothing to reject.
    Envelope::with_payload(serde_json::to_value(res).expect("RagResult is Serialize"))
}

/// Decode the result envelope back to a validated `RagResult`.
pub fn decode_result(env: &Envelope) -> Result<RagResult, DecodeError> {
    check_envelope(env)?;
    let res = decode::decode_rag_result(&env.payload)?;
    decode::validate_rag_result(&res).map_err(DecodeError::ValidationFailed)?;
    Ok(res)
}
