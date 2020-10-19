#!/bin/bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
PRODUCER_PROFILE="$DIR/../producer_profile.toml"

eval "$DIR/wait-mysql.sh -f $PRODUCER_PROFILE"