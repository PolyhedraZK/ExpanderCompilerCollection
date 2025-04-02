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

# 1. Prepare assignment data
./cmd -epoch "$EPOCH" || { echo "Failed to prepare assignment data"; exit 1; }

# 2. Generate the witnesses for end stage
./efc -s end -e "$EPOCH" || { echo "Failed to generate witnesses"; exit 1; }

# 3. Run the prover script
./end2end_prover.sh --epoch "$EPOCH" || { echo "Failed to run prover script"; exit 1; }
# 4. Generate the witnesses for start stage
./efc -s start -e "$EPOCH" || { echo "Failed to generate witnesses"; exit 1; }