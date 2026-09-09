//! §7.2 F2 — schemaVersion-aware health serialization.
//! Contract: `docs/specs/engine-wire-contract.md` §9.
//!
//! **RED-stage stub skeleton** — the struct is real (serde derives), but the
//! `health` projection body is a placeholder the Implementer fills.

use serde::{Deserialize, Serialize};

use crate::store::{EngineState, EngineStatus, EngineSubsystems};
use crate::wire::envelope::{current_schema_version, ID_FORMAT_OPAQUE_STRING_V1};

/// The wire health report. snake_case **Rust** fields; camelCase top-level wire
/// keys (`schemaVersion` / `idFormat` / `lastError`) via `rename_all`; the
/// inner `subsystems` flags keep their single-word field names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthReport {
    pub schema_version: u32,
    pub id_format: String,
    pub state: EngineState,
    pub version: String,
    pub subsystems: EngineSubsystems,
    pub last_error: Option<String>,
}

/// Build a `HealthReport` canonical-envelope value for `status`: current schema
/// version + `opaque-string-v1` id format; maps state/version/subsystems/
/// last_error verbatim (faithful projection, never invents a `last_error`).
pub fn health(status: &EngineStatus) -> HealthReport {
    // RED-stage stub: the struct shape is real so the type-checks pass, but the
    // projection body is a placeholder the Implementer fills.
    let _ = (status, current_schema_version(), ID_FORMAT_OPAQUE_STRING_V1);
    todo!("RED-stage stub: status::health")
}
