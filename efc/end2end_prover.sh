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
mkdir ./proofs
mkdir ./proofs/${EPOCH}




EXPANDER_CMD="./expander-exec --fiat-shamir-hash SHA256 --poly-commitment-scheme Orion"

# Run the prover for permutationhashbit
CIRCUIT_FILE="./circuit_permutationhashbit_2097152.txt"
WITNESS_DIR="./witnesses/${EPOCH}/permutationhashbit_2097152"
PROOF_DIR="./proofs/${EPOCH}/permutationhashbit_2097152"
N=2
mkdir ./proofs/${EPOCH}/permutationhashbit_2097152

for i in $(seq 0 0); do
  WITNESS_FILE="$WITNESS_DIR/witness_${i}.txt"
  PROOF_FILE="$PROOF_DIR/proof_mpi${N}_${i}"
  mpiexec -n $N $EXPANDER_CMD prove \
    --circuit-file "$CIRCUIT_FILE" \
    --witness-file "$WITNESS_FILE" \
    --output-proof-file "$PROOF_FILE" &
done

# Run the prover for blsverifier (round1)
CIRCUIT_FILE="./circuit_blsverifier.txt"
WITNESS_DIR="./witnesses/${EPOCH}/blsverifier"
PROOF_DIR="./proofs/${EPOCH}/blsverifier"
N=8
mkdir ./proofs/${EPOCH}/blsverifier

for i in $(seq 0 7); do
  WITNESS_FILE="$WITNESS_DIR/witness_${i}.txt"
  PROOF_FILE="$PROOF_DIR/proof_mpi${N}_${i}"
  mpiexec -n $N $EXPANDER_CMD prove \
    --circuit-file "$CIRCUIT_FILE" \
    --witness-file "$WITNESS_FILE" \
    --output-proof-file "$PROOF_FILE" &
done

sleep 60 # wait for the permutationhashbit and blsverifier prover to load circuits and witnesses

# Run the prover for shuffle (round1)
CIRCUIT_FILE="./circuit_shuffle_512.txt"
WITNESS_DIR="./witnesses/${EPOCH}/shuffle_512"
PROOF_DIR="./proofs/${EPOCH}/shuffle_512"
N=8
mkdir ./proofs/${EPOCH}/shuffle_512

for i in $(seq 0 7); do
  WITNESS_FILE="$WITNESS_DIR/witness_${i}.txt"
  PROOF_FILE="$PROOF_DIR/proof_mpi${N}_${i}"
  mpiexec -n $N $EXPANDER_CMD prove \
    --circuit-file "$CIRCUIT_FILE" \
    --witness-file "$WITNESS_FILE" \
    --output-proof-file "$PROOF_FILE" &
done

#wait for the first round
MAX_PROC=80
INTERVAL=5
while true; do
    # get the number of expander-exec processes
    PROC_COUNT=$(pgrep -fc expander-exec)

    if [ "$PROC_COUNT" -lt "$MAX_PROC" ]; then
        break
    else
        sleep $INTERVAL
    fi
done

# Run the prover for shuffle (round2)
CIRCUIT_FILE="./circuit_shuffle_512.txt"
WITNESS_DIR="./witnesses/${EPOCH}/shuffle_512"
PROOF_DIR="./proofs/${EPOCH}/shuffle_512"
N=8
mkdir ./proofs/${EPOCH}/shuffle_512

for i in $(seq 8 15); do
  WITNESS_FILE="$WITNESS_DIR/witness_${i}.txt"
  PROOF_FILE="$PROOF_DIR/proof_mpi${N}_${i}"
  mpiexec -n $N $EXPANDER_CMD prove \
    --circuit-file "$CIRCUIT_FILE" \
    --witness-file "$WITNESS_FILE" \
    --output-proof-file "$PROOF_FILE" &
done

#wait for the first round of blsverifier
THRESHOLD=16
INTERVAL=5
while true; do
    # get the number of blsverifier prover processes
    count=$(ps aux | grep expander-exec | grep blsverifier | grep -v grep | wc -l)

    if [ "$count" -lt "$THRESHOLD" ]; then
        echo "Process count is below threshold. Exiting monitor."
        break
    fi
    sleep 5
done
# Run the prover for blsverifier (round2)
CIRCUIT_FILE="./circuit_blsverifier.txt"
WITNESS_DIR="./witnesses/${EPOCH}/blsverifier"
PROOF_DIR="./proofs/${EPOCH}/blsverifier"
N=8
mkdir ./proofs/${EPOCH}/blsverifier
for i in $(seq 8 15); do
  WITNESS_FILE="$WITNESS_DIR/witness_${i}.txt"
  PROOF_FILE="$PROOF_DIR/proof_mpi${N}_${i}"
  mpiexec -n $N $EXPANDER_CMD prove \
    --circuit-file "$CIRCUIT_FILE" \
    --witness-file "$WITNESS_FILE" \
    --output-proof-file "$PROOF_FILE" &
done

#wait for the second round of shuffle
MAX_PROC=80
INTERVAL=5
while true; do
    # get the number of expander-exec processes
    PROC_COUNT=$(pgrep -fc expander-exec)

    if [ "$PROC_COUNT" -lt "$MAX_PROC" ]; then
        break
    else
        sleep $INTERVAL
    fi
done

# Run the prover for permutationquery
CIRCUIT_FILE="./circuit_permutationquery.txt"
WITNESS_DIR="./witnesses/${EPOCH}/permutationquery"
PROOF_DIR="./proofs/${EPOCH}/permutationquery"
N=8
mkdir ./proofs/${EPOCH}/permutationquery

for i in $(seq 0 0); do
  WITNESS_FILE="$WITNESS_DIR/witness_${i}.txt"
  PROOF_FILE="$PROOF_DIR/proof_mpi${N}_${i}"
  mpiexec -n $N $EXPANDER_CMD prove \
    --circuit-file "$CIRCUIT_FILE" \
    --witness-file "$WITNESS_FILE" \
    --output-proof-file "$PROOF_FILE" &
done

# Run the prover for hashtable
CIRCUIT_FILE="./circuit_hashtable256.txt"
WITNESS_DIR="./witnesses/${EPOCH}/hashtable256"
PROOF_DIR="./proofs/${EPOCH}/hashtable256"
N=8
mkdir ./proofs/${EPOCH}/hashtable256

for i in $(seq 0 11); do
  if [ "$i" -eq 11 ]; then
    N=2
  else
    N=8
  fi
  WITNESS_FILE="$WITNESS_DIR/witness_${i}.txt"
  PROOF_FILE="$PROOF_DIR/proof_mpi${N}_${i}"
  mpiexec -n $N $EXPANDER_CMD prove \
    --circuit-file "$CIRCUIT_FILE" \
    --witness-file "$WITNESS_FILE" \
    --output-proof-file "$PROOF_FILE" &
done

#wait for other processes
MAX_PROC=80
INTERVAL=5
while true; do
    # get the number of expander-exec processes
    PROC_COUNT=$(pgrep -fc expander-exec)

    if [ "$PROC_COUNT" -lt "$MAX_PROC" ]; then
        break
    else
        sleep $INTERVAL
    fi
done
# Run the prover for validatorsubtree
CIRCUIT_FILE="./circuit_validatorsubtree1024.txt"
WITNESS_DIR="./witnesses/${EPOCH}/validatorsubtree1024"
PROOF_DIR="./proofs/${EPOCH}/validatorsubtree1024"
N=8
mkdir ./proofs/${EPOCH}/validatorsubtree1024

for i in $(seq 0 15); do
  WITNESS_FILE="$WITNESS_DIR/witness_${i}.txt"
  PROOF_FILE="$PROOF_DIR/proof_mpi${N}_${i}"
  mpiexec -n $N $EXPANDER_CMD prove \
    --circuit-file "$CIRCUIT_FILE" \
    --witness-file "$WITNESS_FILE" \
    --output-proof-file "$PROOF_FILE" &
done


# Run the prover for merklesubtree1024
CIRCUIT_FILE="./circuit_merklesubtree1024.txt"
WITNESS_DIR="./witnesses/${EPOCH}/merklesubtree1024"
PROOF_DIR="./proofs/${EPOCH}/merklesubtree1024"
N=1
mkdir ./proofs/${EPOCH}/merklesubtree1024

for i in $(seq 0 0); do
  WITNESS_FILE="$WITNESS_DIR/witness_${i}.txt"
  PROOF_FILE="$PROOF_DIR/proof_mpi${N}_${i}"
  mpiexec -n $N $EXPANDER_CMD prove \
    --circuit-file "$CIRCUIT_FILE" \
    --witness-file "$WITNESS_FILE" \
    --output-proof-file "$PROOF_FILE" &
done

wait
echo "Prover finished for all circuits"