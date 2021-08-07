#!/bin/bash

scriptdir="${0%/*}"
. "$scriptdir/lib.sh"

args=( --period 2 -- free -h )
typeogle "${args[*]}"
"$ogle" "${args[@]}" &
pid_ogle="$!"

sleep 6

cat /dev/zero > /tmp/z &
trap 'rm -f /tmp/z' EXIT
pid_cat="$!"
sleep 6

kill "$pid_cat"
sleep 1
rm -f /tmp/z

sleep 4

kill "$pid_ogle"

printf '^C\n$ '
sleep 1
echo

