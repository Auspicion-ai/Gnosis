# Firmament — secure remote bridge

**Edge:** Firmament → Gnosis (`docs/specs/gnosis.md` §5.6).

## Nature

Firmament creates a **secure bridge** to a local Gnosis-backed instance so it
gains a reachable remote presence (tunnel/address-proxy, mTLS + optional
end-to-end mirroring) without compromising D2 local-first.

## Mechanism

Firmament bridges to the local instance (decisions FIRMAMENT-DATAFLOW,
FIRMAMENT-AUTH). The bridge credentials are **GUI-only at the shell** (§4.6.2).
A future Gnosis-backed instance can be reached remotely via the bridge.

## Value

A local Gnosis-backed instance gains a reachable remote presence without
compromising the local-first guarantee (D2).

## Contract refs

`docs/specs/gnosis.md` §5.6, decisions FIRMAMENT-AUTH / FIRMAMENT-DATAFLOW.

## Fail-states

Bridge auth failure / unreachable; bridge credentials are GUI-only at the shell
(security carve-out, decision D4-CARVEOUT-SCOPE).
