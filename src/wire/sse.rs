//! §7.2 F2 — single-event SSE framing `event: <type>\ndata: <json>\n\n`.
//! Contract: `docs/specs/engine-wire-contract.md` §4.4 / §8.

use serde_json::Value;

use crate::store::RagChunk;
use crate::wire::codecs;
use crate::wire::decode::{decode_chunk_payload, DecodeError};

/// The single-event SSE `event:` type derived from a `RagChunk`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SseEventType {
    Result,
    Done,
    Error,
}

fn event_label(t: SseEventType) -> &'static str {
    match t {
        SseEventType::Result => "result",
        SseEventType::Done => "done",
        SseEventType::Error => "error",
    }
}

/// The `SseEventType` for a chunk (`Result`/`Done`/`Error`).
pub fn event_type(chunk: &RagChunk) -> SseEventType {
    match chunk {
        RagChunk::Result(_) => SseEventType::Result,
        RagChunk::Done => SseEventType::Done,
        RagChunk::Error(_) => SseEventType::Error,
    }
}

/// Emit exactly one event: `event: <type>\ndata: <json>\n\n` (§4.4 framing).
/// The `data:` line is the single-line canonical chunk JSON.
pub fn encode_event(chunk: &RagChunk) -> String {
    let t = event_label(event_type(chunk));
    let data = codecs::chunk_payload(chunk).to_string();
    format!("event: {t}\ndata: {data}\n\n")
}

/// Decode a single SSE frame back to the chunk (round-trip; malformed frames /
/// event–data mismatch / bad JSON / result-body validation are `DecodeError`s).
pub fn decode_event(frame: &str) -> Result<RagChunk, DecodeError> {
    // Expected exact: <event: t>\n<data: json>\n\n  → split yields 4 parts.
    let parts: Vec<&str> = frame.split('\n').collect();
    if parts.len() != 4
        || !parts[0].starts_with("event: ")
        || !parts[1].starts_with("data: ")
        || !parts[2].is_empty()
        || !parts[3].is_empty()
    {
        return Err(DecodeError::InvalidEnvelope(
            "SSE frame must be exactly 'event: <t>\\ndata: <json>\\n\\n'".to_string(),
        ));
    }
    let event = &parts[0]["event: ".len()..];
    let data = &parts[1]["data: ".len()..];

    let val: Value =
        serde_json::from_str(data).map_err(|e| DecodeError::InvalidJson(e.to_string()))?;
    let data_type = val
        .get("type")
        .and_then(|x| x.as_str())
        .ok_or_else(|| DecodeError::InvalidJson("data JSON has no string \"type\"".to_string()))?;
    if data_type != event {
        return Err(DecodeError::EventTypeMismatch {
            event: event.to_string(),
            data_type: data_type.to_string(),
        });
    }
    decode_chunk_payload(&val)
}
