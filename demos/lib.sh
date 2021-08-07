
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

ogle=ogle
