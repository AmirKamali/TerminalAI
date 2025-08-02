#!/bin/bash
# Build the package
./build.sh

# Install globally
sudo cp target/release/* /usr/local/bin/

# Or install for user
cp target/release/* ~/.local/bin/