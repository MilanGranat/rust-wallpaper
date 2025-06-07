#!/bin/bash

set -e

echo "Building the project..."
if ! cargo build --release; then
    echo "Build failed. Aborting deployment."
    exit 1
fi

echo "Build succeeded."

echo "Stopping service..."
systemctl --user stop wallpaperd.service

echo "Updating binary..."
sudo cp target/release/rust-wallpaper /usr/local/bin/wallpaperd

echo "Starting service..."
systemctl --user start wallpaperd.service

echo "Deployment complete."
