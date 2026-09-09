//! Gnosis — the production graph/vector engine binary.
//!
//! A pure backend to the Astrographer shell. It serves the `RagStore` +
//! query/stream/engine-status API the shell proxies. It has no MCP/GUI surface.

use gnosis::store::RagStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Engine boot: load the store, build the lexical/vector indexes, then
    // serve the query/stream/engine-status API over the IPC/HTTP seam.
    // (Scaffold — the boot model is specified in docs/specs/gnosis.md §4.6.)
    println!("Gnosis engine starting — the headless graph/vector engine of the Auspicion Suite.");
    Ok(())
}
