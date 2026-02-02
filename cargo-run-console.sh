#!/bin/bash

set -e -x

RUSTFLAGS="--cfg tokio_unstable" cargo run --features=console "$@"
