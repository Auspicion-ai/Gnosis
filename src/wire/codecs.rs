//! §7.2 F2 — JSON codecs for `RagChunk`, `StoreError`, `RagResult` (all
//! envelope-wrapped). Contract: `docs/specs/engine-wire-contract.md` §6.
//!
//! **RED-stage stub skeleton** — bodies are placeholders the Implementer fills.

use crate::store::{RagChunk, RagResult, StoreError};
use crate::wire::decode::DecodeError;
use crate::wire::envelope::Envelope;

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

/// Encode a `RagChunk` into an envelope (payload = canonical chunk JSON).
pub fn encode_chunk(chunk: &RagChunk) -> Envelope {
    let _ = chunk;
    todo!("RED-stage stub: codecs::encode_chunk")
}

/// Decode an envelope back to the chunk (decode-then-validate; result-body
/// failures map into `EngineError`/`TraceUnavailable` routes).
pub fn decode_chunk(env: &Envelope) -> Result<RagChunk, DecodeError> {
    let _ = env;
    todo!("RED-stage stub: codecs::decode_chunk")
}

/// Standalone `StoreError` codec (payload = `{"code","message"}` envelope).
pub fn encode_error(err: &StoreError) -> Envelope {
    let _ = err;
    todo!("RED-stage stub: codecs::encode_error")
}

/// Decode a standalone error envelope back to the `StoreError`.
pub fn decode_error(env: &Envelope) -> Result<StoreError, DecodeError> {
    let _ = env;
    todo!("RED-stage stub: codecs::decode_error")
}

/// Encode a `RagResult` into an envelope (payload = the serde result body).
pub fn encode_result(res: &RagResult) -> Envelope {
    let _ = res;
    todo!("RED-stage stub: codecs::encode_result")
}

/// Decode the result envelope back to a validated `RagResult`.
pub fn decode_result(env: &Envelope) -> Result<RagResult, DecodeError> {
    let _ = env;
    todo!("RED-stage stub: codecs::decode_result")
}
