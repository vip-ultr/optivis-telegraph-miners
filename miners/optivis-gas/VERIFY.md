# VERIFY — optivis-gas

The gas value is proven against a live Ethereum JSON-RPC `eth_gasPrice` call.
Run `python3 tools/verify_upstream.py` to reproduce; the fixture is committed
under `fixtures/eth_gas.json`.

## Upstream — Ethereum JSON-RPC (keyless public RPC) — `/gas`
Method: `POST` `{jsonrpc:"2.0", method:"eth_gasPrice", params:[], id:1}`
RPC host (in YAML `base_url`): `https://eth.meowrpc.com`
Fallback RPCs tested: `https://ethereum-rpc.publicnode.com`, `https://1rpc.io/eth`
Fixture: `fixtures/eth_gas.json`

| YAML maps | Live path | Observed value |
|---|---|---|
| `gas_gwei` | `result` (hex wei) | `0xceb7d21` -> ~0.214 gwei |

The node converts the hex wei `result` to gwei for the `label_field: gas_gwei`
leaf, which the Deterministic GAS_PRICE scorer reads directly.

## Reliability note
Public RPCs are shared and occasionally rate-limit. The YAML sets
`cache_ttl_sec: 15` (gas changes fast but 15s is fine for agents) and a
circuit breaker (`circuit_threshold: 5`, `circuit_cooldown_seconds: 30`) so a
flaky RPC cannot cause a sustained routing revocation. If meowrpc is
persistently blocked, swap `base_url` to another keyless RPC and re-register.
