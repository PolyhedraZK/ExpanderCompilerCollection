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

echo "Running Prover on epoch: $EPOCH"

# 1. Prepare assignment data
mpiexec -n 8 ./target/release/expander-exec --fiat-shamir-hash SHA256 --poly-commitment-scheme Orion --circuit-file ./circuit_hashtable256.txt prove --witness-file ./witnesses/hashtable256/ --output-proof-file ./proofs/hashtable256/
