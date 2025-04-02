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

echo "Running Verifier on epoch: $EPOCH"

EXPANDER_CMD="./expander-exec --fiat-shamir-hash SHA256 --poly-commitment-scheme Orion"

# Run the verifier for permutationhashbit
CIRCUIT_FILE="./circuit_permutationhashbit_2097152.txt"
WITNESS_DIR="./witnesses/permutationhashbit_2097152"
PROOF_DIR="./proofs/permutationhashbit_2097152"
N=2

for i in $(seq 0 0); do
  WITNESS_FILE="$WITNESS_DIR/witness_${i}.txt"
  PROOF_FILE="$PROOF_DIR/proof_mpi${N}_${i}"
  $EXPANDER_CMD verify \
    --circuit-file "$CIRCUIT_FILE" \
    --witness-file "$WITNESS_FILE" \
    --input-proof-file "$PROOF_FILE" \
    --mpi-size $N &
done

# Run the verifier for permutationquery
CIRCUIT_FILE="./circuit_permutationquery.txt"
WITNESS_DIR="./witnesses/permutationquery"
PROOF_DIR="./proofs/permutationquery"
N=8

for i in $(seq 0 0); do
  WITNESS_FILE="$WITNESS_DIR/witness_${i}.txt"
  PROOF_FILE="$PROOF_DIR/proof_mpi${N}_${i}"
  $EXPANDER_CMD verify \
    --circuit-file "$CIRCUIT_FILE" \
    --witness-file "$WITNESS_FILE" \
    --input-proof-file "$PROOF_FILE" \
    --mpi-size $N &
done


# Run the verifier for shuffle
CIRCUIT_FILE="./circuit_shuffle_512.txt"
WITNESS_DIR="./witnesses/shuffle_512"
PROOF_DIR="./proofs/shuffle_512"
N=8

for i in $(seq 0 15); do
  WITNESS_FILE="$WITNESS_DIR/witness_${i}.txt"
  PROOF_FILE="$PROOF_DIR/proof_mpi${N}_${i}"
  $EXPANDER_CMD verify \
    --circuit-file "$CIRCUIT_FILE" \
    --witness-file "$WITNESS_FILE" \
    --input-proof-file "$PROOF_FILE" \
    --mpi-size $N &
done

# Run the verifier for blsverifier
CIRCUIT_FILE="./circuit_blsverifier.txt"
WITNESS_DIR="./witnesses/blsverifier"
PROOF_DIR="./proofs/blsverifier"
N=8

for i in $(seq 0 15); do
  WITNESS_FILE="$WITNESS_DIR/witness_${i}.txt"
  PROOF_FILE="$PROOF_DIR/proof_mpi${N}_${i}"
  $EXPANDER_CMD verify \
    --circuit-file "$CIRCUIT_FILE" \
    --witness-file "$WITNESS_FILE" \
    --input-proof-file "$PROOF_FILE" \
    --mpi-size $N &
done

# Run the verifier for validatorsubtree
CIRCUIT_FILE="./circuit_validatorsubtree1024.txt"
WITNESS_DIR="./witnesses/validatorsubtree1024"
PROOF_DIR="./proofs/validatorsubtree1024"
N=8

for i in $(seq 0 15); do
  WITNESS_FILE="$WITNESS_DIR/witness_${i}.txt"
  PROOF_FILE="$PROOF_DIR/proof_mpi${N}_${i}"
  $EXPANDER_CMD verify \
    --circuit-file "$CIRCUIT_FILE" \
    --witness-file "$WITNESS_FILE" \
    --input-proof-file "$PROOF_FILE" \
    --mpi-size $N &
done

# Run the verifier for merklesubtree1024
CIRCUIT_FILE="./circuit_merklesubtree1024.txt"
WITNESS_DIR="./witnesses/merklesubtree1024"
PROOF_DIR="./proofs/merklesubtree1024"
N=1

for i in $(seq 0 0); do
  WITNESS_FILE="$WITNESS_DIR/witness_${i}.txt"
  PROOF_FILE="$PROOF_DIR/proof_mpi${N}_${i}"
  $EXPANDER_CMD verify \
    --circuit-file "$CIRCUIT_FILE" \
    --witness-file "$WITNESS_FILE" \
    --input-proof-file "$PROOF_FILE" \
    --mpi-size $N &
done

# Run the verifier for hashtable
CIRCUIT_FILE="./circuit_hashtable256.txt"
WITNESS_DIR="./witnesses/hashtable256"
PROOF_DIR="./proofs/hashtable256"
N=8

for i in $(seq 0 11); do
  if [ "$i" -eq 11 ]; then
    N=2
  else
    N=8
  fi
  WITNESS_FILE="$WITNESS_DIR/witness_${i}.txt"
  PROOF_FILE="$PROOF_DIR/proof_mpi${N}_${i}"
  $EXPANDER_CMD verify \
    --circuit-file "$CIRCUIT_FILE" \
    --witness-file "$WITNESS_FILE" \
    --input-proof-file "$PROOF_FILE" \
    --mpi-size $N &
done

wait
echo "All verifications completed"