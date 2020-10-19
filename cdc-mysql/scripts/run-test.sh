#!/bin/bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

# Create Fluvio Topic
eval "$DIR/helpers/create-topic.sh"

# Send MYSQL commands to Producer MYSQL
eval "$DIR/helpers/wait-mysql-producer.sh"
if [ $? -ne 0 ]; then exit; fi
eval "$DIR/helpers/mysql-send-cmds.sh"

# Run Producer & Consumer
eval "$DIR/helpers/run-producer.sh"
eval "$DIR/helpers/run-consumer.sh"

# Compare Results - Producer MYSQL & Consumer MYSQL
eval "$DIR/helpers/wait-mysql-consumer.sh"
if [ $? -ne 0 ]; then exit; fi
eval "$DIR/helpers/mysql-validate-result.sh"