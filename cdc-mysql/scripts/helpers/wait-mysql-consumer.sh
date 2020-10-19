#!/bin/bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
ROOT_DIR="$DIR/../.."
CONSUMER_PROFILE="$ROOT_DIR/consumer_profile.toml"

eval "$DIR/wait-mysql.sh -f $CONSUMER_PROFILE"