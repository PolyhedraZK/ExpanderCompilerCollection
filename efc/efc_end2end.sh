#!/bin/bash

EPOCH="290001"

while [[ $# -gt 0 ]]; do
  key="$1"

  case $key in
    --epoch)
      EPOCH="$2"
      shift 
      shift 
      ;;
    *)
      echo "Unknown option, usage: <script> --epoch <number>"
      exit 1
      ;;
  esac
done

if [ -z "$EPOCH" ]; then
  echo "Usage: $0 --epoch <number>"
  exit 1
fi

echo "Running with epoch: $EPOCH"

start1=$(date +%s.%N)
# 1. Prepare assignment data
echo "Preparing assignment data, in the streamline mode, this should be done in advance"
./cmd -epoch "$EPOCH" || { echo "Failed to prepare assignment data"; exit 1; }

start2=$(date +%s.%N)
# 2. Generate the witnesses for end stage
echo "Preparing witnesses, in the streamline mode, this should be done in advance"
./efc -s end -e "$EPOCH" -m "8,8,8,8,2,8" || { echo "Failed to generate witnesses"; exit 1; }

start3=$(date +%s.%N)
# 3. Run the prover script
./end2end_prover.sh --epoch "$EPOCH" & || { echo "Failed to run prover script"; exit 1; }
# 4. Generate the witnesses for start stage
./efc -s start -e "$EPOCH" || { echo "Failed to generate witnesses"; exit 1; }

end1=$(date +%s.%N)
elapsed11=$(echo "$end1 - $start1" | bc)
elapsed21=$(echo "$end1 - $start2" | bc)
elapsed31=$(echo "$end1 - $start3" | bc)
echo "Elapsed time for the whole process: $elapsed11"
echo "Elapsed time after preparing assignment data: $elapsed21"
echo "Elapsed time after preparing witnesses: $elapsed31"

# 5. Run the verifier script
./end2end_verifier.sh --epoch "$EPOCH" || { echo "Failed to run verifier script"; exit 1; }