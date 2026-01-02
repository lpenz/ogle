#!/bin/bash

t=0.3

set -e

for i in {1..3}; do
    echo "$i"
    sleep "$t"
done

t=$(date +%s)

exit $((t % 2))

