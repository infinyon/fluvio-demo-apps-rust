#!/bin/bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

# Runs setup with both profiles
eval "$DIR//helpers/setup.sh  -f $DIR/../producer_profile.toml"
eval "$DIR//helpers/setup.sh  -f $DIR/../consumer_profile.toml"