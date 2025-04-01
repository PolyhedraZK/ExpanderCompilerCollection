#!/bin/bash

# Save current directory
ROOT_DIR=$(pwd)

# 1. Build the Rust program
cargo build --release || { echo "Rust build failed"; exit 1; }
sleep 2
# 2. Move compiled Rust binary to current directory
cp ../target/release/efc ./efc

# 3. Run all solver preparations in background (concurrently)
for circuit in shuffle permutationhash permutationquery hashtable blsverifier validatorsubtree merklesubtree; do
    ./efc -p "$circuit" &
    echo "Started ./efc -p $circuit in background"
    sleep 1
done

# 4. Clone the repo
cd ../../ || { echo "Failed to enter parent parent directory"; exit 1; }
rm -rf EthFullConsensus
git clone https://github.com/PolyhedraZK/EthFullConsensus.git
sleep 5
# 5. Enter the repo and checkout the branch
cd EthFullConsensus || { echo "Failed to enter EthFullConsensus"; exit 1; }
sleep 1
git checkout dev_pcs

# 6. Build the Go program
cd end2end/cmd || { echo "Go cmd dir not found"; exit 1; }
go build -o cmd || { echo "Go build failed"; exit 1; }
sleep 10
# 7. Move the compiled Go program to the original directory
mv cmd "$ROOT_DIR/"

# 8. Go back to efc root
cd "$ROOT_DIR" || exit 1
# 9. Optional: Wait for all background processes to complete
wait
echo "All background processes finished."
