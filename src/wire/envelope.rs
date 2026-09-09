//! §7.2 F2 — the versioned envelope `{schemaVersion, idFormat, payload}`.
//! Contract: `docs/specs/engine-wire-contract.md` §4.1 / §10.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::wire::codecs::WireCodecError;
use crate::wire::decode::DecodeError;

/// The wire envelope. snake_case **Rust** fields; camelCase on the wire via
/// `rename_all` (`schemaVersion` / `idFormat`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Envelope {
    pub schema_version: u32,
    pub id_format: String,
    pub payload: serde_json::Value,
}

/// The current schema version (all §12 golden vectors use `schemaVersion:1`).
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// The only implemented `id_format` ("opaque strings" — RFC-4122 deferred).
pub const ID_FORMAT_OPAQUE_STRING_V1: &str = "opaque-string-v1";

/// Returns `CURRENT_SCHEMA_VERSION`.
pub fn current_schema_version() -> u32 {
    CURRENT_SCHEMA_VERSION
}

impl Envelope {
    /// Serialize to compact JSON (serde_json::to_string); never fails for our
    /// payload types.
    pub fn to_json(&self) -> Result<String, WireCodecError> {
        serde_json::to_string(self).map_err(|e| WireCodecError::Serialize(e.to_string()))
    }

    /// Parse + shape-check a JSON string; `InvalidJson` on a parse error,
    /// `InvalidEnvelope` on a missing/typed-weak top-level field. Does **not**
    /// itself validate `schema_version`/`id_format` against the constants.
    pub fn from_json(s: &str) -> Result<Envelope, DecodeError> {
        let v: Value =
            serde_json::from_str(s).map_err(|e| DecodeError::InvalidJson(e.to_string()))?;
        let obj = v.as_object().ok_or_else(|| {
            DecodeError::InvalidEnvelope("envelope must be a JSON object".to_string())
        })?;
        let schema_version = obj
            .get("schemaVersion")
            .and_then(|x| x.as_u64())
            // Reject any value outside the `u32` range straight up instead of
            // truncating via `as u32` (which would turn `2^32 + 1` into a
            // passable `1`); a negative / over-sized `schemaVersion` is a
            // wrongly-typed field, not a wrap (§4.1 / §10 — `InvalidEnvelope`).
            .and_then(|u| u32::try_from(u).ok())
            .ok_or_else(|| {
                DecodeError::InvalidEnvelope("missing or non-u32 \"schemaVersion\"".to_string())
            })?;
        let id_format = obj
            .get("idFormat")
            .and_then(|x| x.as_str())
            .map(str::to_string)
            .ok_or_else(|| {
                DecodeError::InvalidEnvelope("missing or non-string \"idFormat\"".to_string())
            })?;
        let payload = obj
            .get("payload")
            .cloned()
            .ok_or_else(|| DecodeError::InvalidEnvelope("missing \"payload\"".to_string()))?;
        Ok(Envelope {
            schema_version,
            id_format,
            payload,
        })
    }

    /// Canonical envelope: `schema_version = 1`, `id_format = "opaque-string-v1"`.
    pub fn with_payload(payload: serde_json::Value) -> Envelope {
        Envelope {
            schema_version: current_schema_version(),
            id_format: ID_FORMAT_OPAQUE_STRING_V1.to_string(),
            payload,
        }
    }
}
