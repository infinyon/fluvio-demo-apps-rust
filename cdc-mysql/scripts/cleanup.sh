#!/bin/bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

# Clean-up with both profiles
eval "$DIR/helpers/cleanup.sh  -f $DIR/../producer_profile.toml"
eval "$DIR/helpers/cleanup.sh  -f $DIR/../consumer_profile.toml"