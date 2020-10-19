#!/bin/bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
ROOT_DIR="$DIR/../.."
PRODUCER_PROFILE="$ROOT_DIR/producer_profile.toml"

eval "$DIR/wait-mysql.sh -f $PRODUCER_PROFILE"