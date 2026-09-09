//! §7.2 F2 — `StoreError` → stable wire-code string + reverse lookup.
//! Contract: `docs/specs/engine-wire-contract.md` §5 (the 21-variant table).
//!
//! **RED-stage stub skeleton** — bodies are placeholders the Implementer fills.

use crate::store::StoreError;

/// One row of the §5 wire-code table (for docs / tests).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireCodeRow {
    pub code: &'static str,
    pub variant_name: &'static str,
    pub display: &'static str,
}

impl StoreError {
    /// The stable, unique, non-empty wire-code string for this variant (§5).
    pub fn wire_code(&self) -> &'static str {
        let _ = self;
        todo!("RED-stage stub: StoreError::wire_code")
    }
}

/// Maps a canonical wire code → the unit variant; for `"validation_error"`
/// requires/uses `message`; `None` for any unknown/foreign code (§5 reverse
/// lookup).
pub fn from_wire(code: &str, message: Option<&str>) -> Option<StoreError> {
    let _ = (code, message);
    todo!("RED-stage stub: error::from_wire")
}

/// The §5 table — the 21 (code, variant_name, display) rows.
pub fn code_table() -> &'static [WireCodeRow] {
    todo!("RED-stage stub: error::code_table")
}
