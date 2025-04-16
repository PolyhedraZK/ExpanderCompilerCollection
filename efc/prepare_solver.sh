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

# 4. make sure 7z is installed
sudo apt update
sudo apt install p7zip-full p7zip-rar
sleep 2
# 5. unzip to get go program
7z x cmd.7z

# 6. Go back to efc root
cd "$ROOT_DIR" || exit 1

# 7. Debug all circuits on a default epoch
./cmd -epoch 290001 || { echo "Failed to run cmd"; exit 1; }
./efc -d 290001 &

# 8. Clone the Expander repo
cd ../../ || { echo "Failed to enter parent parent directory"; exit 1; }
rm -rf Expander
git clone https://github.com/PolyhedraZK/Expander.git
sleep 2

# 9. Enter the repo and checkout the branch
cd Expander || { echo "Failed to enter Expander"; exit 1; }
sleep 1
git checkout main

# 10. Build the rust program
RUSTFLAGS="-C target-cpu=native" cargo build --release --all || { echo "Rust build failed"; exit 1; }
sleep 2
# 11. Move the compiled rust program to the original directory
cp target/release/expander-exec "$ROOT_DIR/"

# 12. Optional: Wait for all background processes to complete
wait
echo "All background processes finished."
