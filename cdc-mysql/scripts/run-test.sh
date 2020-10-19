#!/bin/bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

# Create Fluvio Topic
eval "$DIR/create-topic.sh"

# Send MYSQL commands to Producer MYSQL
eval "$DIR/wait-mysql-producer.sh"
if [ $? -ne 0 ]; then exit; fi
eval "$DIR/mysql-send-cmds.sh"

# Run Producer & Consumer
eval "$DIR/run-producer.sh"
eval "$DIR/run-consumer.sh"

# Compare Results - Producer MYSQL & Consumer MYSQL
eval "$DIR/wait-mysql-consumer.sh"
if [ $? -ne 0 ]; then exit; fi
eval "$DIR/mysql-validate-result.sh"