#!/usr/bin/env python3
"""
verify_upstream.py - Proof that every field path in our miner YAMLs actually
exists in the live upstream API responses.

Why this exists:
  Telegraph Tier A (Deterministic) intents score miners by exact-match against
  ground truth. If our `source_path` does not resolve to the real number, the
  scorer cannot find the answer and we score ~0. Most AI-generated miners fail
  here. This script captures a real response from each upstream API, asserts the
  exact dot-paths our YAML maps, and writes the raw response into
  miners/<slug>/fixtures/ as evidence.

Run:  python3 tools/verify_upstream.py
Exit code 0 = every mapped field path is present in live data.
"""
import json
import sys
import os
import urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FIX = lambda slug, name: os.path.join(ROOT, "miners", slug, "fixtures", name)


def get(url, timeout=15):
    req = urllib.request.Request(
        url,
        headers={
            "User-Agent": "Mozilla/5.0 (optivis-verify/1.0)",
            "Accept": "application/json",
        },
    )
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return json.loads(r.read().decode())


def save(slug, name, data):
    path = FIX(slug, name)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as f:
        json.dump(data, f, indent=2)
    print(f"  saved fixture -> {os.path.relpath(path, ROOT)}")


def resolve(obj, path):
    """Resolve a dot-path like 'coins.coingecko:solana.price' or 'pairs.0.priceUsd'."""
    cur = obj
    for part in path.split("."):
        if isinstance(cur, list):
            cur = cur[int(part)]
        elif isinstance(cur, dict):
            if part not in cur:
                raise KeyError(part)
            cur = cur[part]
        else:
            raise KeyError(f"cannot descend into {part} (at '{path}')")
    return cur


CHECKS = []


def check(slug, url, name, paths, quiet=False):
    print(f"[check] {slug} -> {name}")
    data = get(url)
    save(slug, name, data)
    ok = True
    for p in paths:
        try:
            val = resolve(data, p)
            shown = val if quiet else val
            # avoid dumping giant arrays (e.g. TVL history) to the terminal
            sval = repr(shown)
            if len(sval) > 80:
                sval = sval[:80] + "...(truncated)"
            print(f"    OK   {p} = {sval}")
        except Exception as e:
            ok = False
            print(f"    FAIL {p} ({e})")
    CHECKS.append(ok)
    return data


# ---------------------------------------------------------------------------
# 1) optivis-crypto-price
#    /price  -> CoinGecko simple/price (keyless, by symbol)
#    /pair   -> DexScreener best pair (keyless), indexed by Solana token
# ---------------------------------------------------------------------------
sol_addr = "So11111111111111111111111111111111111111112"
check(
    "optivis-crypto-price",
    "https://api.coingecko.com/api/v3/simple/price?symbols=sol,eth&vs_currencies=usd",
    "coingecko_price.json",
    ["sol.usd", "eth.usd"],
)
check(
    "optivis-crypto-price",
    f"https://api.dexscreener.com/latest/dex/tokens/{sol_addr}",
    "dexscreener_token.json",
    ["pairs.0.chainId", "pairs.0.priceUsd", "pairs.0.liquidity.usd",
     "pairs.0.volume.h24", "pairs.0.fdv", "pairs.0.pairCreatedAt",
     "pairs.0.baseToken.symbol"],
)

# ---------------------------------------------------------------------------
# 2) optivis-tvl
#    /protocol -> DefiLlama protocol TVL
#    /chain    -> DefiLlama chain TVL
#    /yields   -> DefiLlama top yields (bonus)
# ---------------------------------------------------------------------------
check(
    "optivis-tvl",
    "https://api.llama.fi/protocol/uniswap",
    "llama_protocol.json",
    ["name", "tvl", "chainTvls"],
    quiet=True,
)
check(
    "optivis-tvl",
    "https://api.llama.fi/v2/chains",
    "llama_chains.json",
    ["0.name", "0.tvl", "0.chainId"],
)
check(
    "optivis-tvl",
    "https://yields.llama.fi/pools",
    "llama_yields.json",
    ["data.0.chain", "data.0.project", "data.0.tvlUsd", "data.0.apy"],
)

# ---------------------------------------------------------------------------
# 3) optivis-gas
#    /gas -> publicnode Ethereum RPC eth_gasPrice (keyless)
# ---------------------------------------------------------------------------
import subprocess

def rpc_gas(url, timeout=15):
    # Use curl: the sandbox egress routes curl reliably to public RPC hosts
    # (raw urllib sockets to these hosts are intermittently blocked).
    body = json.dumps({"jsonrpc": "2.0", "method": "eth_gasPrice",
                        "params": [], "id": 1})
    r = subprocess.run(
        ["curl", "-s", "-m", str(timeout), "-X", "POST", url,
         "-H", "Content-Type: application/json", "-d", body],
        capture_output=True, text=True,
    )
    return json.loads(r.stdout)


print("[check] optivis-gas -> eth_gasPrice")
GAS_RPCS = [
    "https://eth.meowrpc.com",
    "https://ethereum-rpc.publicnode.com",
    "https://1rpc.io/eth",
]
gas = None
for rpc in GAS_RPCS:
    try:
        gas = rpc_gas(rpc)
        if "result" in gas:
            break
    except Exception as e:
        print(f"    rpc {rpc} failed: {e}")
        continue
ok = bool(gas) and "result" in gas and isinstance(gas.get("result"), str) \
    and gas["result"].startswith("0x")
save("optivis-gas", "eth_gas.json", gas or {"error": "all RPCs failed"})
print(f"    {'OK  ' if ok else 'FAIL'} result = {gas.get('result') if gas else None} (hex gwei)")
CHECKS.append(ok)

# ---------------------------------------------------------------------------
print("\n===== SUMMARY =====")
print(f"{sum(CHECKS)}/{len(CHECKS)} upstream checks passed")
sys.exit(0 if all(CHECKS) else 1)
