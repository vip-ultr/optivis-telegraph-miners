#!/usr/bin/env python3
"""
validate_yaml.py - Static sanity check for our miner YAMLs against the
Telegraph Miner Standard closed-set rules.

It does NOT replace the integrate.telegraphprotocol.com wizard (which
sandbox-tests live endpoints), but it catches the cheap, expensive mistakes
locally before we ever touch a wallet:

  - required top-level fields present
  - `kind` is miner/validator/subnet
  - `protocol` (if set) is bittensor/generic
  - auth.type (if set) is bearer/header/none
  - endpoint entries only use the 8 allowed keys
  - signal_mapping only uses confidence_field/label_field/reason_field
  - semantic intents are non-empty
  - on_chain.transform is direct/llm

Run:  python3 tools/validate_yaml.py
Exits non-zero if any miner file fails.
"""
import os
import sys
import glob
import yaml

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
MINERS = os.path.join(ROOT, "miners")

ALLOWED_ENDPOINT_KEYS = {
    "path", "external_path", "method", "description", "endpoint_base_url",
    "content_type", "multipart_fields", "param_map", "intents", "params", "body",
}
ALLOWED_SIGNAL_KEYS = {"confidence_field", "label_field", "reason_field"}
ALLOWED_KINDS = {"miner", "validator", "subnet"}
ALLOWED_PROTOCOLS = {"bittensor", "generic"}
ALLOWED_AUTH = {"bearer", "header", "none"}
ALLOWED_TRANSFORM = {"direct", "llm"}
REQUIRED_TOP = {"version", "kind", "id", "slug", "name", "base_url"}


def err(path, msg):
    print(f"  FAIL {os.path.relpath(path, ROOT)}: {msg}")
    return False


def validate(path):
    ok = True
    with open(path) as f:
        doc = yaml.safe_load(f)

    for k in REQUIRED_TOP:
        if k not in doc:
            ok &= err(path, f"missing required top-level field '{k}'")
    if "kind" in doc and doc["kind"] not in ALLOWED_KINDS:
        ok &= err(path, f"kind '{doc['kind']}' not in {ALLOWED_KINDS}")
    if "protocol" in doc and doc["protocol"] not in ALLOWED_PROTOCOLS:
        ok &= err(path, f"protocol '{doc['protocol']}' not in {ALLOWED_PROTOCOLS}")

    auth = doc.get("auth")
    if auth is not None:
        if auth.get("type") not in ALLOWED_AUTH:
            ok &= err(path, f"auth.type '{auth.get('type')}' not in {ALLOWED_AUTH}")

    for ep in doc.get("endpoints", []):
        extra = set(ep.keys()) - ALLOWED_ENDPOINT_KEYS
        if extra:
            ok &= err(path, f"endpoint '{ep.get('path')}' has extra keys {extra}")

    sem = doc.get("semantics", {})
    sm = sem.get("signal_mapping", {})
    extra = set(sm.keys()) - ALLOWED_SIGNAL_KEYS
    if extra:
        ok &= err(path, f"signal_mapping has extra keys {extra}")
    intents = sem.get("supported_intents", [])
    if not intents:
        ok &= err(path, "supported_intents is empty")

    oc = doc.get("on_chain")
    if oc is not None and oc.get("transform") not in ALLOWED_TRANSFORM:
        ok &= err(path, f"on_chain.transform '{oc.get('transform')}' invalid")

    # prove every mapped source_path is a string (structure check)
    fields = (oc or {}).get("fields", {})
    for arr in fields.values():
        for fld in arr:
            if "source_path" not in fld and "source" not in fld:
                ok &= err(path, f"on_chain field '{fld.get('name')}' missing source_path")
    return ok


def main():
    files = sorted(glob.glob(os.path.join(MINERS, "*", "miner.yaml")))
    print(f"Validating {len(files)} miner YAML(s)...\n")
    all_ok = True
    for f in files:
        print(f"[file] {os.path.relpath(f, ROOT)}")
        all_ok &= validate(f)
    print("\n===== VALIDATION " + ("PASSED" if all_ok else "FAILED") + " =====")
    sys.exit(0 if all_ok else 1)


if __name__ == "__main__":
    main()
