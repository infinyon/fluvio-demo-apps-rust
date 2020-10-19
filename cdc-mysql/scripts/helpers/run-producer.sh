#!/bin/bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
ROOT_DIR="$DIR/../.."
PRODUCER_PROFILE="$ROOT_DIR/producer_profile.toml"

###
## Sleep routine
###
sleepWait()
{
    loop=0
    while [ $loop -lt 10 ];
    do
        echo -n "."
        let loop=loop+1
        sleep .5
    done
    echo
}

###
## Run Producer
###

eval "(RUST_LOG=debug cargo run --bin cdc-producer -- $PRODUCER_PROFILE)" &>/tmp/cdc-producer.log & disown;
echo " ✅ cargo run --bin cdc-producer -- producer_profile.toml' - ok"

sleepWait #allow producer time to complete

# Stop producer
producer_proc=$(ps | grep cdc-producer | grep -o -E '[0-9]+' | head -1 | sed -e 's/^0\+//')
eval "kill -9 $producer_proc"
echo " ✅ cdc-producer - stopped"
