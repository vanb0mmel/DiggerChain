#!/usr/bin/env bash
set -euo pipefail
repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd -- "$repo_root"

cargo test --workspace --locked -j 2
RAYON_NUM_THREADS=2 cargo test --manifest-path experiments/proof-lab/Cargo.toml \
    --release --locked -j 2 -- --test-threads=1
"$HOME/.foundry/bin/forge" test --root contracts --use "$repo_root/target/solc-0.8.28" --offline -vv
python -m unittest discover -s scripts -p 'test_*.py' -v
