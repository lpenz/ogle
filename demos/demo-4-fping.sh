#!/bin/bash

scriptdir="${0%/*}"
. "$scriptdir/lib.sh"

args=( --until-success -- fping myhost )
typeogle "${args[*]}"
"$ogle" "${args[@]}" &

sleep 3

sed -i '$a\127.0.0.1 myhost' /etc/hosts
sleep 2

sed -i /myhost/d /etc/hosts

wait

printf '\n$ '
sleep 1
echo

