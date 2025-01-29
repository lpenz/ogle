#!/bin/bash

set -e -x

for demoscript in ./demos/demo-*.sh; do
    rm -f demo.cast
    asciinema rec -c "$demoscript" demo.cast
    base="${demoscript%.*}"
    docker run --rm -v "$PWD:/data" docker.io/asciinema2/asciicast2gif -S1 demo.cast "${base}.gif"
done
rm -f demo.cast
