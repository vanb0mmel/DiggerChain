#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd -- "$repo_root"

echo 'Checking shared gameplay and the preserved terminal prototype...'
cargo test --workspace --locked -j 2
echo 'Checking gameplay and ownership proof circuits and their rejection cases...'
RAYON_NUM_THREADS=2 cargo test --manifest-path experiments/proof-lab/Cargo.toml \
    --release --locked -j 2 -- --test-threads=1
echo 'Measuring the generated three-floor expedition at 512 successful actions...'
bash scripts/proof-lab.sh --generated 0 512
