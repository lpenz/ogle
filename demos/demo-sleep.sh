#!/bin/bash

CARGO_TARGET_DIR=${CARGO_TARGET_DIR-$PWD/target}
function typeogle {
    cmd="ogle $1"
    printf "$ "
    sleep 0.2
    for (( i=0; i<"${#cmd}"; i++ )); do
        sleep 0.1
        printf "%c" "${cmd:$i:1}"
    done
    echo
}

typeogle '-p 3 -c "date; sleep 3"'
(
"$CARGO_TARGET_DIR/debug/ogle" -p 3 -c 'date; sleep 3' &
pidogle="$!"
trap 'kill $pidogle; wait $pidogle' EXIT

sleep 16.5
)

printf '^C\n$ '
sleep 1

