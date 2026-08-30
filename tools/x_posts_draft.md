Draft X (Twitter) posts for the Telegraph hackathon engagement component.
Post from @vip_ultr. Keep them short, credible, not hype. Replace <WALLET> if needed.

=== POST 1 — project intro (post early) ===
Building verifiable financial-data miners for @telegraphprotocol hackathon (Track 1 + Track 2).
Multi-source crypto pricing, currency exchange, financials and TVL — served on-chain, scored by WASM evaluators. Repo: github.com/vip-ultr/optivis-telegraph-miners

=== POST 2 — after registering scorer / progress ===
Shipped a no_std WASM scorer for CRYPTO_PRICE on @telegraphprotocol. Pure Rust, 0 imports, ~22KB. Iterating against the eval harness (separation margin is the gate). Track 2 is harder than it looks — the benchmark separates good from bad answers, not just right from wrong.

=== POST 3 — multi-intent leverage ===
One miner, many intents: our CoinGecko-backed miner serves CRYPTO_PRICE, CURRENCY_EXCHANGE and FINANCIAL_DATA from a single registration. Coverage beats single-intent depth when the score is averaged across intents. github.com/vip-ultr/optivis-telegraph-miners

=== POST 4 — final push (near deadline) ===
<24h left on @telegraphprotocol Season I. Our stack: 3 live miners (crypto/fx/financials, TVL, gas), a Rust WASM scorer, and an x402 payment loop paying $0.01 USDC per query. Verifiable signal infra, not a demo. Wish us luck.
