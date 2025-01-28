#!/bin/bash

t=0.3

set -e

for i in {1..3}; do
    echo "$i"
    sleep "$t"
done

echo
sleep "$t"

date

for i in {4..6}; do
    sleep "$t"
    echo "$i"
done
