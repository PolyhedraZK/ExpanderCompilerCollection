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

# 4. Clone the EthFullConsensus repo
cd ../../ || { echo "Failed to enter parent parent directory"; exit 1; }
rm -rf EthFullConsensus
git clone https://github.com/PolyhedraZK/EthFullConsensus.git
sleep 5
# 5. Enter the repo and checkout the branch
cd EthFullConsensus || { echo "Failed to enter EthFullConsensus"; exit 1; }
sleep 1
git checkout dev_pcs

# 6. Build the go program
cd end2end/cmd || { echo "Go cmd dir not found"; exit 1; }
go build -o cmd || { echo "Go build failed"; exit 1; }
sleep 10
# 7. Move the compiled go program to the original directory
mv cmd "$ROOT_DIR/"

# 8. Go back to efc root
cd "$ROOT_DIR" || exit 1

# 9. Debug all circuits on a default epoch
./cmd -epoch 290001 || { echo "Failed to run cmd"; exit 1; }
./efc -d 290001 &

# 10. Clone the Expander repo
cd ../../ || { echo "Failed to enter parent parent directory"; exit 1; }
rm -rf Expander
git clone https://github.com/PolyhedraZK/Expander.git
sleep 5

# 11. Enter the repo and checkout the branch
cd Expander || { echo "Failed to enter Expander"; exit 1; }
sleep 1
git checkout main

# 12. Build the rust program
RUSTFLAGS="-C target-cpu=native" cargo build --release -- all || { echo "Rust build failed"; exit 1; }
sleep 10
# 13. Move the compiled rust program to the original directory
cp target/release/expander-exec "$ROOT_DIR/"

# 14. Optional: Wait for all background processes to complete
wait
echo "All background processes finished."
