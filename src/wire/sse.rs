//! §7.2 F2 — single-event SSE framing `event: <type>\ndata: <json>\n\n`.
//! Contract: `docs/specs/engine-wire-contract.md` §4.4 / §8.
//!
//! **RED-stage stub skeleton** — bodies are placeholders the Implementer fills.

use crate::store::RagChunk;
use crate::wire::decode::DecodeError;

/// The single-event SSE `event:` type derived from a `RagChunk`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SseEventType {
    Result,
    Done,
    Error,
}

/// The `SseEventType` for a chunk (`Result`/`Done`/`Error`).
pub fn event_type(chunk: &RagChunk) -> SseEventType {
    let _ = chunk;
    todo!("RED-stage stub: sse::event_type")
}

/// Emit exactly one event: `event: <type>\ndata: <json>\n\n` (§4.4 framing).
pub fn encode_event(chunk: &RagChunk) -> String {
    let _ = chunk;
    todo!("RED-stage stub: sse::encode_event")
}

/// Decode a single SSE frame back to the chunk (round-trip; malformed frames /
/// event–data mismatch / bad JSON / result-body validation are `DecodeError`s).
pub fn decode_event(frame: &str) -> Result<RagChunk, DecodeError> {
    let _ = frame;
    todo!("RED-stage stub: sse::decode_event")
}
