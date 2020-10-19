#!/bin/bash
WAIT_SEC=60

helpFunction()
{
   echo ""
   echo "Usage: $0 -f profile-file"
   echo -e "\t-f profile-file"
   exit 1 # Exit script after printing help
}

while getopts "f:" opt
do
   case "$opt" in
      f ) profile="$OPTARG" ;;
      ? ) helpFunction ;; # Print helpFunction in case parameter is non-existent
   esac
done

# Print helpFunction in case parameters are empty
if [ -z "$profile" ]
then
   echo "Profile file is a required parameter";
   helpFunction
fi

# Check if profile exists
if [ ! -f "$profile" ]; then
    echo "Profile '$profile' not found!"
    exit 1
fi

MYSQL_HOST="0.0.0.0"
MYSQL_USER="fluvio"
MYSQL_PASSWORD="fluvio4cdc!"

# Retrieve mysql host port from producer profile
mysql_port=$(grep "^host_port" $profile | cut -d' ' -d'=' -f2- | tr -d '"' | tr -d '[:space:]')
if [ -z "$mysql_port" ]
then
   echo "Cannot find 'container.host_port' in '$profile'";
   exit 1
fi

###
# Wait for mysql to come up
##

checkMysql() {
    mysql_cmd="mysql -h $MYSQL_HOST -P $mysql_port -u$MYSQL_USER -p$MYSQL_PASSWORD"

    sql="SHOW DATABASES";
    result=$($mysql_cmd -e "$sql" 2>/dev/null)
    ret_code=$?
}

sleepWait()
{
    loop=0
    while [ $loop -lt 2 ];
    do
        echo -n "."
        let loop=loop+1
        sleep .5
    done
}

waitSec=0
firstIteration=1
checkMysql
while [ $ret_code -ne 0 ] && [ $waitSec -lt $WAIT_SEC ];
do
    if [ $firstIteration -eq 1 ]; then echo " ⌛ Waiting for MySQL" ; fi

    sleepWait
    checkMysql

    firstIteration=0
    waitSec=$((waitSec+1))
done

if [ $firstIteration -eq 0 ]; then echo; fi

if [ $ret_code -gt 0 ]; then
    echo " ❌ mysql is not running"
    exit 1
else 
    echo " ✅ mysql is running"
fi