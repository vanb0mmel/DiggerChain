#!/usr/bin/env bash
set -euo pipefail
repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd -- "$repo_root"

case "${1:-}" in
    setup|prove) ;;
    *) echo 'Usage: bash scripts/proof-release.sh setup NEW_DIRECTORY' >&2
       echo '       bash scripts/proof-release.sh prove SAVE PARAMETERS CHAIN EPOCH RECIPIENT CONTRACT NEW_DIRECTORY' >&2
       exit 2 ;;
esac

cargo build --manifest-path experiments/proof-lab/Cargo.toml --bin diggerchain-proof --release --locked -j 2
echo 'Proof tool limits: 2 Rayon threads, 4 GiB virtual address space, 120 seconds.' >&2
(
    ulimit -v 4194304
    export RAYON_NUM_THREADS=2
    exec timeout 120 "$repo_root/experiments/proof-lab/target/release/diggerchain-proof" "$@"
)
