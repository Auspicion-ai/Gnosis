# Gnosis — Integration Surfaces

Gnosis is a **pure backend** to the Auspicion Suite. Its integration surfaces
are the directed edges through which it gains and provides utility (design
constraint D3). The canonical direction + mechanism + value for each edge is in
`docs/specs/gnosis.md` §5 (Integration contract); these docs detail how Gnosis
integrates with each suite tool.

| Tool | Doc | Edge | Nature |
| --- | --- | --- | --- |
| **Astrographer** | `astrographer.md` | the **proxy seam** (§5.1) | The primary surface — the shell proxies Gnosis's API. Gnosis's front-end. |
| **Astral** | `astral.md` | Gnosis → Astral via the shell's push (§5.2) | Wiki publishing. |
| **Zodiac** | `zodiac.md` | Zodiac → Gnosis (§5.3) | A second RAG data source. |
| **Solomon** | `solomon.md` | Solomon ↔ Gnosis (§5.4) | Cross-instance search. |
| **Emerald** | `emerald.md` | Emerald ↔ Gnosis (§5.5) | Graph editing round-trip. |
| **Firmament** | `firmament.md` | Firmament → Gnosis (§5.6) | Secure remote bridge. |
| **Familiar** | `familiar.md` | Familiar → Gnosis (§5.7) | Knowledge retrieval + optional memory store. |

## The seam principle

Gnosis exposes its API (the `RagStore` + query/stream/engine-status surface,
`docs/specs/gnosis.md` §4.1.5/§4.6.1) to the **Astrographer shell**, which is
the single integration point for the rest of the suite. Other tools reach
Gnosis through the shell (or through Gnosis's own edges where a direct edge is
specified). Gnosis has **no MCP surface and no GUI surface** — it is headless.
