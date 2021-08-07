#!/bin/bash

scriptdir="${0%/*}"
. "$scriptdir/lib.sh"

args=( -- ping -q -c1 -w5 myhost )
typeogle "${args[*]}"
"$ogle" "${args[@]}" &
pid_ogle="$!"

sleep 3

sed -i '$a\127.0.0.1 myhost' /etc/hosts
sleep 3

sed -i /myhost/d /etc/hosts

sleep 3

kill "$pid_ogle"

printf '^C\n$ '
sleep 1
echo

