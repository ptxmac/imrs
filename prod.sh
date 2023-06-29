#!/usr/bin/env bash
set -euo pipefail

pushd frontend
trunk build
popd

cargo run --bin server --release -- --port 8080 --static-dir ./dist
