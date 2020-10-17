#!/bin/bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

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

mysql_cmd="mysql -h $MYSQL_HOST -P $mysql_port -u$MYSQL_USER -p$MYSQL_PASSWORD"

sql="SHOW DATABASES";
result=$($mysql_cmd -e "$sql" 2>/dev/null)
ret_code=$?

if [ $ret_code -gt 0 ]; then
    echo " ❌ mysql is not running"
    exit 1
else 
    echo " ✅ mysql is running"
fi
