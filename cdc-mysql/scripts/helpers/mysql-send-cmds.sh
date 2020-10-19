#!/bin/bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
ROOT_DIR="$DIR/../.."
PRODUCER_PROFILE="$ROOT_DIR/producer_profile.toml"
MYSQL_HOST="0.0.0.0"
MYSQL_USER="fluvio"
MYSQL_PASSWORD="fluvio4cdc!"

###
## SQL commands
###
SQL_COMMANDS=("CREATE DATABASE flvDb;")
SQL_COMMANDS+=("use flvDb; CREATE TABLE pet (name VARCHAR(20), owner VARCHAR(20), species VARCHAR(20), sex CHAR(1), birth DATE);")
SQL_COMMANDS+=("use flvDb; INSERT INTO pet VALUES ('Puffball','Diane','hamster','f','1999-03-30');")
SQL_COMMANDS+=("use flvDb; INSERT INTO pet VALUES ('Jack','Peter','dog','m','1999-03-30');")
SQL_COMMANDS+=("use flvDb; UPDATE pet SET birth = '1989-08-31' WHERE name = 'Jack';")
SQL_COMMANDS+=("use flvDb; ALTER TABLE pet ADD COLUMN last_vaccine DATE;")
SQL_COMMANDS+=("use flvDb; DELETE from pet where name='Puffball';")
SQL_COMMANDS+=("use flvDb; INSERT INTO pet VALUES ('Spot', 'Jane', 'dog', 'm', '2010-11-2', Null);")
SQL_COMMANDS+=("use flvDb; UPDATE pet SET last_vaccine='2020-6-10' WHERE name='Spot';")

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
