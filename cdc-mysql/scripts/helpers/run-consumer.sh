#!/bin/bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
ROOT_DIR="$DIR/../.."
CONSUMER_PROFILE="$ROOT_DIR/consumer_profile.toml"

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
## Run Consumer
###

eval "(RUST_LOG=debug cargo run --bin cdc-consumer -- $CONSUMER_PROFILE)" &>/tmp/cdc-consumer.log & disown;
echo " ✅ cargo run --bin cdc-consumer -- consumer_profile.toml' - ok"

sleepWait #allow consumer time to complete

# Stop consumer
consumer_proc=$(ps | grep cdc-consumer | grep -o -E '[0-9]+' | head -1 | sed -e 's/^0\+//')
eval "kill -9 $consumer_proc"
echo " ✅ cdc-consumer - stopped"
