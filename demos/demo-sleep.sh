#!/bin/bash

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
ogle -p 3 -c 'date; sleep 3' &
pid_ogle="$!"

sleep 16.5

kill "$pid_ogle"

printf '^C\n$ '
sleep 1

