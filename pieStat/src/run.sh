#!/bin/bash
current_dir=$(basename "$PWD")
if [[ "$PWD" != */src ]]; then
    cp main.rs src/
else
    cd ..
fi
gnome-terminal -- bash -c "source \$HOME/.cargo/env; cargo run; exec bash"
