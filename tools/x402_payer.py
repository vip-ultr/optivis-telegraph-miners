#!/usr/bin/env python3
"""x402 v2 payer client for Telegraph devnet — Base Sepolia (EVM exact scheme).

Flow:
  1. POST query to /engine/v1/ask/{minerId} -> 402 with `Payment-Required` challenge
     (base64 JSON: x402Version, accepts[{scheme,network,asset,amount,payTo}])
  2. Build + sign a USDC ERC-20 `transfer(payTo, amount)` tx from the burner wallet.
  3. Re-POST with header:  Authorization: x402 <network> <base64(signed_tx)>
  4. Node verifies/broadcasts the payment and returns the miner's answer.

Key handling: reads BASE_SEPOLIA_PRIVATE_KEY from local .env (gitignored).
The key NEVER leaves this machine and is never printed or committed.

Usage:
  . .venv/bin/activate
  python3 tools/x402_payer.py            # runs the default probe set
  python3 tools/x402_payer.py 7311 "price of SOL in usd"
"""
import base64
import json
import os
import sys
import urllib.request
import urllib.error

from dotenv import load_dotenv
from eth_account import Account
from web3 import Web3

load_dotenv()

NODE = "https://devnode.telegraphprotocol.com"
USDC_ABI = [
    {  # ERC-20 transfer(address,uint256)
        "name": "transfer",
        "type": "function",
        "stateMutability": "nonpayable",
        "inputs": [
            {"name": "to", "type": "address"},
            {"name": "value", "type": "uint256"},
        ],
        "outputs": [{"name": "", "type": "bool"}],
    }
]

UA = {
    "Content-Type": "application/json",
    "User-Agent": "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 "
    "(KHTML, like Gecko) Chrome/124.0 Safari/537.36",
    "Accept": "application/json",
}

PROBES = [
    (7311, "What is the price of SOL in USD?"),
    (7311, "price of ETH and BONK in usd"),
    (7312, "What is the TVL of uniswap?"),
    (7312, "TVL of jupiter"),
]


def decode_challenge(b64str: str) -> dict:
    s = b64str.replace("-", "+").replace("_", "/")
    while len(s) % 4:
        s += "="
    return json.loads(base64.b64decode(s))


def build_payment_header(challenge: dict, w3: Web3, acct: Account) -> str:
    """Pick the Base Sepolia (eip155:84532) accept and sign a USDC transfer.

    x402 v2 'exact' EVM envelope:
      header name: X-PAYMENT
      header value: base64({ x402Version, scheme, network, payload:{ transaction } })
    where transaction is the 0x-prefixed signed raw tx.
    """
    accept = next(
        a for a in challenge["accepts"] if a["network"] == "eip155:84532"
    )
    asset = Web3.to_checksum_address(accept["asset"])
    pay_to = Web3.to_checksum_address(accept["payTo"])
    amount = int(accept["amount"])  # micro-USDC

    usdc = w3.eth.contract(address=asset, abi=USDC_ABI)
    tx = usdc.functions.transfer(pay_to, amount).build_transaction(
        {
            "from": acct.address,
            "nonce": w3.eth.get_transaction_count(acct.address),
            "gas": 100000,
            "gasPrice": w3.eth.gas_price,
            "chainId": 84532,
        }
    )
    signed = acct.sign_transaction(tx)
    raw = signed.raw_transaction.hex() if hasattr(signed, "raw_transaction") \
        else signed.rawTransaction.hex()
    envelope = {
        "x402Version": challenge.get("x402Version", 2),
        "scheme": "exact",
        "network": accept["network"],
        "payload": {"transaction": raw},
    }
    return base64.b64encode(json.dumps(envelope).encode()).decode()


def ask(miner_id: int, query: str, w3: Web3, acct: Account):
    path = f"/engine/v1/ask/{miner_id}"
    payload = json.dumps({"query": query}).encode()

    # 1) trigger 402
    req = urllib.request.Request(
        NODE + path, data=payload, headers=UA, method="POST"
    )
    try:
        with urllib.request.urlopen(req, timeout=30) as r:
            return r.status, r.read().decode()
    except urllib.error.HTTPError as e:
        if e.code != 402:
            return e.code, e.read().decode()[:600]
        chal_hdr = e.headers.get("Payment-Required") or ""
        if not chal_hdr:
            return e.code, "402 but no Payment-Required header: " + e.read().decode()[:300]
        challenge = decode_challenge(chal_hdr)
    except Exception as e:
        return None, f"ERR trigger: {e}"

    # 2) sign + pay
    try:
        auth = build_payment_header(challenge, w3, acct)
    except Exception as e:
        return None, f"ERR signing: {e}"
    accept_network = next(
        a for a in challenge["accepts"] if a["network"] == "eip155:84532"
    )["network"]

    # 3) re-POST with payment (x402 v2: Authorization: x402 <base64 json>)
    hdr = dict(UA)
    hdr["X-PAYMENT"] = auth
    hdr["Authorization"] = f"x402 {auth}"
    req2 = urllib.request.Request(
        NODE + path, data=payload, headers=hdr, method="POST"
    )
    try:
        with urllib.request.urlopen(req2, timeout=30) as r:
            return r.status, r.read().decode()
    except urllib.error.HTTPError as e:
        return e.code, e.read().decode()[:800]
    except Exception as e:
        return None, f"ERR pay: {e}"


def main():
    key = os.getenv("BASE_SEPOLIA_PRIVATE_KEY", "").strip()
    if not key or key == "0x":
        print("ERROR: set BASE_SEPOLIA_PRIVATE_KEY in .env (gitignored).")
        sys.exit(1)
    rpc = os.getenv("BASE_SEPOLIA_RPC_URL", "https://base-sepolia.public.blastapi.io")
    w3 = Web3(Web3.HTTPProvider(rpc, request_kwargs={"timeout": 15}))
    acct = Account.from_key(key)
    print(f"Payer: {acct.address}  (Base Sepolia)")
    print(f"USDC balance check: {w3.eth.get_balance(acct.address)} wei\n")

    probes = PROBES
    if len(sys.argv) >= 3:
        probes = [(int(sys.argv[1]), sys.argv[2])]

    for miner_id, query in probes:
        print(f"=== miner {miner_id}: {query} ===")
        status, body = ask(miner_id, query, w3, acct)
        print(f"HTTP {status}")
        try:
            print(json.dumps(json.loads(body), indent=1)[:900])
        except Exception:
            print(body[:900])
        print("-" * 50)


if __name__ == "__main__":
    main()
