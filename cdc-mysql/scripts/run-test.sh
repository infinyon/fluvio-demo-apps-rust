#!/bin/bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
PRODUCER_PROFILE="$DIR/../producer_profile.toml"
CONSUMER_PROFILE="$DIR/../consumer_profile.toml"
MYSQL_HOST="0.0.0.0"
MYSQL_USER="fluvio"
MYSQL_PASSWORD="fluvio4cdc!"

SQL_COMMANDS=("CREATE DATABASE flvDb;")
SQL_COMMANDS+=("use flvDb; CREATE TABLE pet (name VARCHAR(20), owner VARCHAR(20), species VARCHAR(20), sex CHAR(1), birth DATE);")
SQL_COMMANDS+=("use flvDb; INSERT INTO pet VALUES ('Puffball','Diane','hamster','f','1999-03-30');")
SQL_COMMANDS+=("use flvDb; INSERT INTO pet VALUES ('Jack','Peter','dog','m','1999-03-30');")
SQL_COMMANDS+=("use flvDb; UPDATE pet SET birth = '1989-08-31' WHERE name = 'Jack';")
SQL_COMMANDS+=("use flvDb; ALTER TABLE pet ADD COLUMN color VARCHAR(20);")
SQL_COMMANDS+=("use flvDb; DELETE from pet where name='Puffball';")
SQL_COMMANDS+=("use flvDb; INSERT INTO pet VALUES ('Spot', 'Jane', 'dog', 'm', '2010-11-2', Null);")
SQL_COMMANDS+=("use flvDb; UPDATE pet SET color='White' WHERE name='Spot';")

###
## Sleep routine
###
sleepWait()
{
    loop=0
    while [ $loop -lt 10 ];
    do
        echo -n "."
        let loop=loop+1
        sleep .5
    done
    echo
}

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

# Retrieve topic from producer profile
topic=$(grep "^topic" $PRODUCER_PROFILE | cut -d' ' -d'=' -f2- | tr -d '"' | tr -d '[:space:]')
if [ -z "$topic" ]
then
   echo "Cannot find 'fluvio.topic' in '$PRODUCER_PROFILE'";
   exit 1
fi

###
## Connect to Producer MYSQL
###

# Retrieve mysql host port from producer profile
mysql_producer_port=$(grep "^host_port" $PRODUCER_PROFILE | cut -d' ' -d'=' -f2- | tr -d '"' | tr -d '[:space:]')
if [ -z "$mysql_producer_port" ]
then
   echo "Cannot find 'container.host_port' in '$PRODUCER_PROFILE'";
   exit 1
fi

# MYSQL producer 
mysql_producer="mysql -h $MYSQL_HOST -P $mysql_producer_port -u$MYSQL_USER -p$MYSQL_PASSWORD"

#Run SQL Commands
for sql in "${SQL_COMMANDS[@]}"
do
    result=$($mysql_producer -e "$sql" 2>/dev/null)
    ret_code=$?
    if [ $ret_code -gt 0 ]; then
        echo " ❌ mysql command '$mysql_producer -e \"$sql\"' - failed"
        exit 1
    else 
        echo " ✅ mysql> $sql - ok"
    fi
done

###
## Run Producer
###

eval "(cargo run --bin cdc-producer -- $PRODUCER_PROFILE)" &>/dev/null & disown;
echo " ✅ cargo run --bin cdc-producer -- producer_profile.toml' - ok"

sleepWait #allow producer time to complete

# Stop producer
producer_proc=$(ps | grep cdc-producer | grep -o -E '[0-9]+' | head -1 | sed -e 's/^0\+//')
eval "kill -9 $producer_proc"
echo " ✅ cdc-producer - stopped"

###
## Run Consumer
###

eval "(cargo run --bin cdc-consumer -- $CONSUMER_PROFILE)" &>/dev/null & disown;
echo " ✅ cargo run --bin cdc-consumer -- consumer_profile.toml' - ok"

sleepWait #allow consumer time to complete

# Stop consumer
consumer_proc=$(ps | grep cdc-consumer | grep -o -E '[0-9]+' | head -1 | sed -e 's/^0\+//')
eval "kill -9 $consumer_proc"
echo " ✅ cdc-consumer - stopped"

###
## Connect to Consumer MYSQL
###

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
result=$($mysql_consumer -e "$sql" 2>/dev/null)
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

expected="name	owner	species	sex	birth	color
Jack	Peter	dog	m	1989-08-31	NULL
Spot	Jane	dog	m	2010-11-02	NULL"

if [ "$result" == "$expected" ]; then
    echo " ✅ result - ok"
else 
    echo " ❌ result - failed"
    echo ">> Expected:"
    echo "$expected"
    echo ">> Found:"
    echo "$result"
fi
