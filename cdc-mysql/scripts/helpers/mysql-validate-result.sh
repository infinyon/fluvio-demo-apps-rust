#!/bin/bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
ROOT_DIR="$DIR/../.."
PRODUCER_PROFILE="$ROOT_DIR/producer_profile.toml"
CONSUMER_PROFILE="$ROOT_DIR/consumer_profile.toml"
MYSQL_HOST="0.0.0.0"
MYSQL_USER="fluvio"
MYSQL_PASSWORD="fluvio4cdc!"

###
## Connect to Producer MYSQL
###

echo "Retrieve 'pet' from Producer ..."

# Retrieve mysql host port from producer profile
mysql_producer_port=$(grep "^host_port" $PRODUCER_PROFILE | cut -d' ' -d'=' -f2- | tr -d '"' | tr -d '[:space:]')
if [ -z "$mysql_producer_port" ]
then
   echo "Cannot find 'container.host_port' in '$PRODUCER_PROFILE'";
   exit 1
fi

# MYSQL producer 
mysql_producer="mysql -h $MYSQL_HOST -P $mysql_producer_port -u$MYSQL_USER -p$MYSQL_PASSWORD"

sql="use flvDb; select * from pet;"
producer_result=$($mysql_producer -e "$sql" 2>/dev/null)
ret_code=$?
if [ $ret_code -gt 0 ]; then
    echo " ❌ mysql command '$mysql_producer -e \"$sql\"' - failed"
    exit 1
else 
    echo " ✅ mysql> $sql - ok"
fi


###
## Connect to Consumer MYSQL
###

echo "Retrieve 'pet' from Consumer ..."

# Retrieve mysql host port from consumer profile
mysql_consumer_port=$(grep "^host_port" $CONSUMER_PROFILE | cut -d' ' -d'=' -f2- | tr -d '"' | tr -d '[:space:]')
if [ -z "$mysql_consumer_port" ]
then
   echo "Cannot find 'container.host_port' in '$CONSUMER_PROFILE'";
   exit 1
fi

# Connect to consumer
mysql_consumer="mysql -h $MYSQL_HOST -P $mysql_consumer_port -u$MYSQL_USER -p$MYSQL_PASSWORD"

# Compare pet table
sql="use flvDb; select * from pet;"
consumer_result=$($mysql_consumer -e "$sql" 2>/dev/null)
ret_code=$?
if [ $ret_code -gt 0 ]; then
    echo " ❌ mysql command '$mysql_consumer -e \"$sql\"' - failed"
    exit 1
else 
    echo " ✅ mysql> $sql - ok"
fi

###
## Compare results
###
echo ">> Leader:"
echo "$producer_result"
echo ">> Follower:"
echo "$consumer_result"

if [ "$consumer_result" == "$producer_result" ]; then
    echo " ✅ result - ok"
else 
    echo " ❌ result - failed"
fi
