#!/bin/bash

scriptdir="${0%/*}"
. "$scriptdir/lib.sh"

typeogle '"date; sleep 3.5"'
args=( -c 'date; sleep 3.5' )
"$ogle" "${args[@]}" &
pid_ogle="$!"

sleep 15

kill "$pid_ogle"

printf '^C\n$ '
sleep 1
echo

