#!/bin/bash

# Build the project
cargo build

# Run the splash screen in the background with a dummy target (sleep 10)
./target/debug/fast_splash sleep 10 &

# Wait a bit for the splash to start
sleep 1

# Run the tester
cargo run --bin tester
