#!/usr/bin/env bash
set -euo pipefail
repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd -- "$repo_root"

for tool in cargo python timeout tee; do
    if ! command -v "$tool" >/dev/null; then
        echo "Missing required development tool: $tool" >&2
        exit 2
    fi
done
if [[ ! -x "$HOME/.foundry/bin/forge" || ! -x "$repo_root/target/solc-0.8.28" ]]; then
    echo 'The previously installed Foundry and local Solidity 0.8.28 compiler are required.' >&2
    exit 2
fi
mkdir -p "$repo_root/target"
run_dir=$(mktemp -d "$repo_root/target/smoke.XXXXXXXX")
trap 'printf "Artifacts: %s\n" "$run_dir"' EXIT
printf 'Smoke-test artifacts: %s\n' "$run_dir"

bash scripts/check-rewards.sh 2>&1 | tee "$run_dir/checks.log"
cargo build --release --locked -j 2 -p diggerchain-terminal 2>&1 | tee "$run_dir/game-build.log"
bash scripts/proof-release.sh setup "$run_dir/parameters" 2>&1 | tee "$run_dir/setup.log"
"$HOME/.foundry/bin/forge" build --root "$run_dir/parameters" \
    --use "$repo_root/target/solc-0.8.28" --offline 2>&1 | tee "$run_dir/verifier-build.log"
python scripts/reward_wallet.py "$run_dir/parameters" --prepare-only --any-account \
    --chain 46630 --save-path "$run_dir/player-{account}.json"
cp docs/SMOKE_TEST.md "$run_dir/README.md"
printf 'Preparation passed. Follow docs/SMOKE_TEST.md. Use a second terminal for gameplay and proof commands.\n'
printf 'Open http://127.0.0.1:8766 in MetaMask/Rabby browser. Ctrl+C stops only the local public page.\n'
"$repo_root/target/release/diggerchain" --wallet "$run_dir/parameters/wallet/index.html"
