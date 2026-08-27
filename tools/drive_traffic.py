#!/usr/bin/env python3
"""Drive live traffic to our registered miners to confirm they answer end-to-end
and to start building a scoring history on the devnet node.

Calls the Engine two ways:
  - auto-routed:  POST /engine/v1/ask            (intent classified by LLM)
  - direct:       POST /engine/v1/ask/{minerId}  (forces our miner)

If the x402 payment gate requires testnet USDC, the node returns 402 with a
payment payload; we print it verbatim so the user knows they need faucet funds.

No keys, no server. Pure requests against the public devnet node.
"""
import json
import sys
import urllib.request
import urllib.error

NODE = "https://devnode.telegraphprotocol.com"

# (miner_id, intent, query) — queries are realistic so the scorer has real data
PROBES = [
    (7311, "CRYPTO_PRICE",   "What is the price of SOL in USD?"),
    (7311, "CRYPTO_PRICE",   "price of ETH and BONK in usd"),
    (7312, "TVL_LOOKUP",     "What is the TVL of uniswap?"),
    (7312, "TVL_LOOKUP",     "TVL of jupiter"),
]


def post(path, payload):
    url = NODE + path
    data = json.dumps(payload).encode()
    req = urllib.request.Request(
        url, data=data, headers={"Content-Type": "application/json"}, method="POST"
    )
    try:
        with urllib.request.urlopen(req, timeout=30) as r:
            return r.status, r.read().decode()
    except urllib.error.HTTPError as e:
        return e.code, e.read().decode()
    except Exception as e:  # noqa
        return None, f"ERROR: {e}"


def main():
    print(f"Node: {NODE}\n")
    for miner_id, intent, query in PROBES:
        # 1) direct-routed (forces our miner)
        direct_path = f"/engine/v1/ask/{miner_id}"
        status, body = post(direct_path, {"query": query})
        print(f"--- DIRECT miner {miner_id} | {intent} ---")
        print(f"query: {query}")
        print(f"HTTP {status}")
        if status == 402:
            print(">>> x402 PAYMENT REQUIRED. Node needs testnet USDC.")
            print(body[:800])
            print()
            continue
        # pretty-print if JSON
        try:
            j = json.loads(body)
            print(json.dumps(j, indent=1)[:800])
        except Exception:
            print(body[:800])
        print()

        # 2) auto-routed (intent classified by node LLM)
        status2, body2 = post("/engine/v1/ask", {"query": query})
        print(f"--- AUTO  ({intent}) ---")
        print(f"HTTP {status2}")
        try:
            j2 = json.loads(body2)
            print(json.dumps(j2, indent=1)[:800])
        except Exception:
            print(body2[:800])
        print("=" * 60)
        print()


if __name__ == "__main__":
    main()
