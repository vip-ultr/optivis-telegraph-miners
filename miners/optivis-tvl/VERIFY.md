# VERIFY — optivis-tvl

Every `source_path` in `miner.yaml` is proven against a live response from
DefiLlama's keyless APIs. Run `python3 tools/verify_upstream.py` to reproduce;
fixtures are committed under `fixtures/`.

## Upstream 1 — Protocol TVL — `/protocol`
Query: `/protocol/uniswap`
Fixture: `fixtures/llama_protocol.json`

| YAML maps | Live path | Observed value |
|---|---|---|
| `tvl_usd` | `tvl` (current, last entry) | present, numeric |
| `chain_tvls` | `chainTvls` | object keyed by chain |

## Upstream 2 — Chain TVL — `/chain`
Query: `/v2/chains` (filtered to chain name at runtime)
Fixture: `fixtures/llama_chains.json`

| YAML maps | Live path | Observed value |
|---|---|---|
| `tvl_usd` (first match by name) | `0.tvl` | `78092.03` |
| `source` | (constant) | `defillama` |

## Upstream 3 — Yields — `/yields`
Query: `/pools` (optional `chain` filter)
Fixture: `fixtures/llama_yields.json`

| YAML maps | Live path | Observed value |
|---|---|---|
| `top_pool.chain` | `data.0.chain` | `Base` |
| `top_pool.project` | `data.0.project` | `aerodrome-slipstream` |
| `top_pool.tvl_usd` | `data.0.tvlUsd` | `31431600393` |
| `top_pool.apy` | `data.0.apy` | `0.00315` |

## Tier A scoring note
TVL_LOOKUP is a **Deterministic** intent. `label_field: tvl_usd` maps to a
clean numeric leaf, so the scorer reads the answer directly. The per-chain
breakdown and top-pool APY are bonus signal for agents that want depth beyond
the single number.
