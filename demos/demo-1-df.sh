#!/bin/bash

scriptdir="${0%/*}"
. "$scriptdir/lib.sh"

args=( -- df -h /tmp/ )
typeogle "${args[*]}"
"$ogle" "${args[@]}" &
pid_ogle="$!"

sleep 3

cat /dev/zero > /tmp/z &
trap 'rm -f /tmp/z' EXIT
pid_cat="$!"
sleep 3

kill "$pid_cat"
sleep 1
rm -f /tmp/z

sleep 2

kill "$pid_ogle"

printf '^C\n$ '
sleep 1
echo

