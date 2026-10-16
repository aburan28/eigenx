#!/bin/sh
# One KEM round trip on a Tier-B parameter set (L5, Q3, or Q5).
# L1/L3 are Tier A only; this script refuses them.
#
#   sh scripts/pq_roundtrip.sh L5
#   sh scripts/pq_roundtrip.sh Q5 8
set -eu
SET="${1:-L5}"
JOBS="${2:-4}"
case "$SET" in
  L5|Q1|Q3|Q5) ;;
  *) echo "pq_roundtrip: $SET is not a Tier-B set (use L5, Q3, or Q5)" >&2; exit 1 ;;
esac
ROOT=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
cd "$ROOT"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/eigenx-target}"
cargo build --release --bin eigenx
B="$CARGO_TARGET_DIR/release/eigenx"
D=$(mktemp -d)
trap 'rm -rf "$D"' EXIT
echo "set=$SET jobs=$JOBS"
"$B" params
/usr/bin/time -p "$B" keygen --set "$SET" --pk "$D/pk" --sk "$D/sk" --jobs "$JOBS"
/usr/bin/time -p "$B" encaps --pk "$D/pk" --ct "$D/ct" --key "$D/k1"
/usr/bin/time -p "$B" decaps --sk "$D/sk" --ct "$D/ct" --key "$D/k2"
cmp "$D/k1" "$D/k2"
echo "round trip ok ($SET)"
