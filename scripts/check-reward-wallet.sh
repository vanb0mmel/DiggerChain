#!/usr/bin/env bash
set -euo pipefail
repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd -- "$repo_root"
if [[ $# -lt 1 ]]; then
    echo 'Usage: bash scripts/check-reward-wallet.sh PARAMETERS_DIRECTORY [wallet options]' >&2
    exit 2
fi
parameters=$(cd -- "$1" && pwd)
shift
"$HOME/.foundry/bin/forge" build --root "$parameters" --use "$repo_root/target/solc-0.8.28" --offline
"$HOME/.foundry/bin/forge" build --root "$repo_root/contracts" --use "$repo_root/target/solc-0.8.28" --offline
python scripts/reward_wallet.py "$parameters" "$@"
