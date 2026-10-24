#!/usr/bin/env bash
set -euo pipefail

generated=false
if [[ "${1:-}" == --generated ]]; then
    generated=true
    shift
fi
if (( $# > 3 )) || [[ "$generated" == true && $# -gt 2 ]]; then
    echo 'Usage: bash scripts/proof-lab.sh [SEED] [MAX_STEPS] [ROUND_TRIPS]' >&2
    echo '       bash scripts/proof-lab.sh --generated [SEED] [MAX_STEPS]' >&2
    exit 2
fi

seed=${1:-0}
if [[ "$generated" == true ]]; then
    steps=${2:-512}
else
    steps=${2:-64}
fi
trips=${3:-0}
for value in "$seed" "$steps" "$trips"; do
    if [[ ! "$value" =~ ^[0-9]+$ ]]; then
        echo 'Seed, step bound, and round trips must be unsigned decimal integers.' >&2
        exit 2
    fi
done
if [[ "$generated" == true ]]; then
    if [[ "$steps" != 256 && "$steps" != 512 ]]; then
        echo 'The generated step bound must be 256 or 512.' >&2
        exit 2
    fi
elif [[ "$steps" != 64 && "$steps" != 128 && "$steps" != 256 ]]; then
    echo 'The step bound must be 64, 128, or 256.' >&2
    exit 2
fi
if [[ "$(uname -s)" != Linux ]]; then
    echo 'This resource-limited benchmark wrapper currently supports Linux only.' >&2
    exit 2
fi
for tool in cargo timeout sha256sum; do
    if ! command -v "$tool" >/dev/null; then
        echo "Required tool is missing: $tool" >&2
        exit 2
    fi
done
repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
manifest="$repo_root/experiments/proof-lab/Cargo.toml"
build_dir="$repo_root/experiments/proof-lab/target"
echo 'Building the separate proof experiment with two Cargo jobs...'
CARGO_TARGET_DIR="$build_dir" cargo build --release --locked -j 2 --manifest-path "$manifest"
mkdir -p -- "$repo_root/target"
run_dir=$(mktemp -d "$repo_root/target/proof-lab.XXXXXXXX")
sha256sum \
    "$manifest" \
    "$repo_root/experiments/proof-lab/Cargo.lock" \
    "$repo_root/experiments/proof-lab/src/"* \
    "$repo_root/Cargo.toml" \
    "$repo_root/rust-toolchain.toml" \
    "$repo_root/crates/diggerchain-core/Cargo.toml" \
    "$repo_root/crates/diggerchain-core/src/"*.rs \
    "$repo_root/scripts/proof-lab.sh" \
    > "$run_dir/source-sha256.txt"
echo "Artifacts: $run_dir"
echo 'Proof process limits: 2 Rayon threads, 4 GiB virtual address space, 120 seconds.'
resource_command=()
if [[ "$generated" == true ]]; then
    proof_arguments=(generated "$seed" "$steps" "$run_dir")
else
    proof_arguments=("$seed" "$steps" "$trips" "$run_dir")
fi
if [[ -x /usr/bin/time ]]; then
    resource_command=(/usr/bin/time -v -o "$run_dir/resources.txt")
else
    echo 'GNU time is optional; using native stage timers and Linux peak-RSS reporting.'
fi
set +e
(
    ulimit -c 0 || exit 2
    ulimit -v 4194304 || exit 2
    exec "${resource_command[@]}" \
        timeout --signal=TERM --kill-after=5s 120s \
        env RAYON_NUM_THREADS=2 \
        "$build_dir/release/diggerchain-proof-lab" "${proof_arguments[@]}"
)
status=$?
set -e
printf 'Proof process exit status: %s\n' "$status"
if [[ -f "$run_dir/resources.txt" ]]; then
    cat -- "$run_dir/resources.txt"
fi
printf 'Artifacts: %s\n' "$run_dir"
exit "$status"
