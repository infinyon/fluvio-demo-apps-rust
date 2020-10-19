#!/bin/bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
ROOT_DIR="$DIR/../.."
PRODUCER_PROFILE="$ROOT_DIR/producer_profile.toml"

###
## Create Fluvio Topic
###

# Retrieve topic from producer profile
topic=$(grep "^topic" $PRODUCER_PROFILE | cut -d' ' -d'=' -f2- | tr -d '"' | tr -d '[:space:]')
if [ -z "$topic" ]
then
   echo "Cannot find 'fluvio.topic' in '$PRODUCER_PROFILE'";
   exit 1
fi

# Create Fluvio Topic
echo "Creating topic... $topic"
has_topic=$(fluvio topic list | grep $topic | wc -l | tr -d " ")
if [ $has_topic -gt 0 ]; then
    echo " ❌ topic '$topic' already created - delete and run script again"
    exit 1
fi

eval $(fluvio topic create $topic  >/dev/null 2>&1)
has_topic=$(fluvio topic list | grep $topic | wc -l | tr -d " ")
if [ $has_topic -gt 0 ]; then
    echo " ✅ fluvio topic create $topic - ok"
else
    echo " ❌ fluvio topic create $topic - error"
    exit 1
fi
