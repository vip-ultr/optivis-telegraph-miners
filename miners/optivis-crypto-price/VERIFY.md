# VERIFY — optivis-crypto-price

Every `source_path` in `miner.yaml` is proven against a live response from the
upstream API, not guessed. Run `python3 tools/verify_upstream.py` to reproduce;
fixtures are committed under `fixtures/`.

## Upstream 1 — CoinGecko simple price (keyless) — `/price`
Query: `/api/v3/simple/price?symbols=sol,eth&vs_currencies=usd`
Fixture: `fixtures/coingecko_price.json`

| YAML maps | Live path | Observed value |
|---|---|---|
| `price_usd` (from `sol.usd`) | `sol.usd` | `95.67` |
| `price_usd` (from `eth.usd`) | `eth.usd` | `2445.03` |

## Upstream 2 — DexScreener token pairs (keyless) — `/pair`
Query: `/latest/dex/tokens/{So111...1112}` (Wrapped SOL, Solana-first default)
Fixture: `fixtures/dexscreener_token.json`

| YAML maps | Live path | Observed value |
|---|---|---|
| `pair.chain` | `pairs.0.chainId` | `fogo` |
| `pair.price_usd` | `pairs.0.priceUsd` | `0.008991` |
| `pair.liquidity_usd` | `pairs.0.liquidity.usd` | `157721.81` |
| `pair.volume_24h` | `pairs.0.volume.h24` | `20811.74` |
| `pair.fdv` | `pairs.0.fdv` | `90392428` |
| `pair.pair_age_hours` | `pairs.0.pairCreatedAt` | `1768479521000` (epoch ms) |
| `pair.base_symbol` | `pairs.0.baseToken.symbol` | `FOGO` |

## Tier A scoring note
CRYPTO_PRICE is a **Deterministic** intent: the scorer exact-matches the
returned number against ground truth. Because `label_field: price_usd` resolves
to a clean numeric leaf (`sol.usd`), the scorer can read the answer directly.
No LLM judge is involved.
