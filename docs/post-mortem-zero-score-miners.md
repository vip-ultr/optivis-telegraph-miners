# Post-mortem: why most miners score ~0 (and how we avoid it)

Observed on the live leaderboard (epoch 281): across many intents, the "#1"
miner scored near zero (e.g. all RESEARCH_QUERY miners at 0, several
STOCK_PRICE/TVL_LOOKUP miners at 0) while a few dominated (FRAUD_DETECTION
"anchor" at 0.99, CHAT_COMPLETION at 0.86+). The differentiator is not
effort — it is **whether the scorer can read the answer**.

## Failure modes (each is a 0)

1. **Wrong `signal_mapping` path.** Tier A exact-match reads `label_field`
   (or confidence/label) as the canonical answer. If the path points at a
   non-numeric or missing leaf, the scorer can't match → 0.
   *Us:* every path in `miner.yaml` is asserted by `verify_upstream.py` and
   committed as a fixture. `label_field` always resolves to a numeric leaf
   (`sol.usd`, `tvl` last entry, `gas_gwei`).

2. **200-status error bodies.** Some APIs return HTTP 200 with the real
   failure in the body. Without `errors.status_path`, the node records the
   error text as the signal → 0. *Us:* our upstreams (CoinGecko, DexScreener,
   DefiLlama, JSON-RPC) use real status codes; no `status_path` needed.

3. **Broken / rate-limited upstream.** If the endpoint 5xxs, the circuit
   breaker trips; sustained failure → spot-check revocation. *Us:* keyless
   reliable APIs + conservative `rate_limit_per_sec` + `cache_ttl_sec` +
   circuit breaker tuned per endpoint.

4. **Guessed params the Engine doesn't send.** The Engine calls our endpoint
   with specific param names. A mismatch → empty/undefined upstream query →
   null answer → 0. *Us:* `param_map` aliases our YAML param names to the
   upstream's; wizard sandbox-tests this before registration.

## Net effect

A miner that is *correct* on a Deterministic intent scores ~1.0 and, on an
empty intent, becomes rank #1 by default → 70% of routed traffic. The bar to
win here is correctness, not novelty. Our whole repo exists to make that
correctness provable.
