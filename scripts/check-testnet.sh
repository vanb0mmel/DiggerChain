#!/usr/bin/env bash
set -euo pipefail

if (( $# != 1 )); then
    echo 'Usage: bash scripts/check-testnet.sh RUN_DIRECTORY' >&2
    exit 2
fi
repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd -- "$repo_root"
forge="$HOME/.foundry/bin/forge"
compiler="$repo_root/target/solc-0.8.28"
if [[ ! -x "$forge" || ! -x "$compiler" ]]; then
    echo 'Foundry and the verified local Solidity 0.8.28 compiler are required. See experiments/proof-lab/EVM.md.' >&2
    exit 2
fi
python3 -m unittest discover -s scripts -p test_testnet_check.py -v
"$forge" test --root "$1" --use "$compiler" --offline -vv
python3 scripts/testnet_check.py "$1"
