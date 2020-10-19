#!/bin/bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
CONSUMER_PROFILE="$DIR/../consumer_profile.toml"

eval "$DIR/wait-mysql.sh -f $CONSUMER_PROFILE"