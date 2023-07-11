# Copyright 2020-2023 Manta Network.
# This file is part of Manta.
#
# Manta is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# Manta is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with Manta.  If not, see <http://www.gnu.org/licenses/>.

# This script has three parts which all use the Substrate runtime:
# - Pallet benchmarking to update the pallet weights
# - Overhead benchmarking for the Extrinsic and Block weights
# - Machine benchmarking
#
# Should be run on a reference machine to gain accurate benchmarks
# current reference machine:
# https://github.com/Manta-Network/Manta/blob/manta/.github/workflows/run_all_benchmarks.yml#L15
#
# Should be run from the root of the repo.

while getopts 'bfps:c:v' flag; do
  case "${flag}" in
    b)
      # Skip build.
      skip_build='true'
      ;;
    c)
      # Which chain spec to use.
      chain_spec="${OPTARG}"
      ;;
    f)
      # Fail if any sub-command in a pipe fails, not just the last one.
      set -o pipefail
      # Fail on undeclared variables.
      set -u
      # Fail if any sub-command fails.
      set -e
      # Fail on traps.
      set -E
      ;;
    p)
      # Start at pallet
      start_pallet="${OPTARG}"
      ;;
    s)
      # Storage snapshot url
      storage_folder="${OPTARG}"
      ;;
    v)
      # Echo all executed commands.
      set -x
      ;;
    *)
      # Exit early.
      echo "Bad options. Check Script."
      exit 1
      ;;
  esac
done


if [ "$skip_build" != true ]
then
  echo "[+] Compiling Manta benchmarks..."
  cargo build --profile=production --locked --features=runtime-benchmarks
fi

# The executable to use.
MANTA=./target/production/manta

# Manually exclude some pallets.
EXCLUDED_PALLETS=(
)

# Load all pallet names in an array.
ALL_PALLETS=($(
  $MANTA benchmark pallet --list --chain=$chain_spec |\
    tail -n+2 |\
    cut -d',' -f1 |\
    sort |\
    uniq
))

# Filter out the excluded pallets by concatenating the arrays and discarding duplicates.
PALLETS=($({ printf '%s\n' "${ALL_PALLETS[@]}" "${EXCLUDED_PALLETS[@]}"; } | sort | uniq -u))

echo "[+] Benchmarking ${#PALLETS[@]} Manta pallets by excluding ${#EXCLUDED_PALLETS[@]} from ${#ALL_PALLETS[@]}."

# Define the error file.
ERR_FILE="scripts/benchmarking/benchmarking_errors.txt"
# Delete the error file before each run.
rm -f $ERR_FILE

FRAME_WEIGHTS_OUTPUT="scripts/benchmarking/frame-weights-output"
# Delete the weights output folders before each run.
rm -R ${FRAME_WEIGHTS_OUTPUT}
# Create the weights output folders.
mkdir ${FRAME_WEIGHTS_OUTPUT}

XCM_WEIGHTS_OUTPUT="scripts/benchmarking/xcm-weights-output"
rm -R ${XCM_WEIGHTS_OUTPUT}
mkdir ${XCM_WEIGHTS_OUTPUT}

STORAGE_WEIGHTS_OUTPUT="scripts/benchmarking/rocksdb_weights.rs"
rm -f ${STORAGE_WEIGHTS_OUTPUT}

MACHINE_OUTPUT="scripts/benchmarking/machine_benchmark_result.txt"
rm -f $MACHINE_OUTPUT

# Benchmark each frame pallet.
for PALLET in "${PALLETS[@]}"; do
  # If `-p` is used, skip benchmarks until the start pallet.
  if [ ! -z "$start_pallet" ] && [ "$start_pallet" != "$PALLET" ]
  then
    echo "[+] Skipping ${PALLET}..."
    continue
  else
    unset start_pallet
  fi

  FOLDER="$(echo "${PALLET#*_}" | tr '_' '-')";
  WEIGHT_FILE="./${FRAME_WEIGHTS_OUTPUT}/${PALLET}.rs"
  TEMPLATE_FILE=".github/resources/frame-weight-template.hbs"
  echo "[+] Benchmarking $PALLET with weight file $WEIGHT_FILE";

  if [ "$PALLET" == "pallet_xcm_benchmarks::fungible" ] || [ "$PALLET" == "pallet_xcm_benchmarks::generic" ]
  then
    OUTPUT_NAME=$(echo $PALLET | sed -r 's/[:]+/_/g')
    WEIGHT_FILE="./${XCM_WEIGHTS_OUTPUT}/${OUTPUT_NAME}.rs"
    TEMPLATE_FILE=".github/resources/xcm-weight-template.hbs"
  fi

  OUTPUT=$(
    $MANTA benchmark pallet \
    --chain=$chain_spec \
    --steps=50 \
    --repeat=40 \
    --pallet="$PALLET" \
    --extrinsic="*" \
    --execution=wasm \
    --wasm-execution=compiled \
    --heap-pages=4096 \
    --output="$WEIGHT_FILE" \
    --template="$TEMPLATE_FILE" 2>&1
  )
  if [ $? -ne 0 ]; then
    echo "$OUTPUT" >> "$ERR_FILE"
    echo "[-] Failed to benchmark $PALLET. Error written to $ERR_FILE; continuing..."
  fi
done

echo "[+] Benchmarking the machine..."
OUTPUT=$(
  $MANTA benchmark machine --chain=$chain_spec --allow-fail 2>&1
)
# In any case don't write errors to the error file since they're not benchmarking errors.
echo "[x] Machine benchmark:\n$OUTPUT"
echo $OUTPUT >> $MACHINE_OUTPUT

# If `-s` is used, run the storage benchmark.
if [ ! -z "$storage_folder" ]; then
  OUTPUT=$(
  $MANTA benchmark storage \
    --chain=$chain_spec \
    --state-version=1 \
    --warmups=10 \
    --base-path=$storage_folder \
    --weight-path=./$STORAGE_WEIGHTS_OUTPUT 2>&1
  )
  if [ $? -ne 0 ]; then
    echo "$OUTPUT" >> "$ERR_FILE"
    echo "[-] Failed the storage benchmark. Error written to $ERR_FILE; continuing..."
  fi
else
  unset storage_folder
fi

# Check if the error file exists.
if [ -f "$ERR_FILE" ]; then
  echo "[-] Some benchmarks failed. See: $ERR_FILE"
  exit 1
else
  echo "[+] All benchmarks passed."
  exit 0
fi