#!/bin/bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

helpFunction()
{
   echo ""
   echo "Usage: $0 -n name -d data-path -p port"
   echo -e "\t-n Container name"
   echo -e "\t-d Data path"
   echo -e "\t-p Port number"
   exit 1 # Exit script after printing help
}

while getopts "n:d:p:" opt
do
   case "$opt" in
      n ) name="$OPTARG" ;;
      d ) path="$OPTARG" ;;
      p ) port="$OPTARG" ;;
      ? ) helpFunction ;; # Print helpFunction in case parameter is non-existent
   esac
done

# Print helpFunction in case parameters are empty
if [ -z "$name" ] || [ -z "$path" ] || [ -z "$port" ]
then
   echo "Invalid parameters";
   helpFunction
fi

# Create directory
eval "mkdir -p $path"
if [ -d `eval echo $path` ]; then
   echo " ✅ mkdir -p $path - ok"
else
   echo " ❌ mkdir -p $path - failed"
   exit 1
fi

# Build docker image
eval "docker build $DIR -t mysql-80 2> /dev/null"
if [[ ! "$(docker images -q mysql-80 2> /dev/null)" == "" ]]; then
   echo " ✅ docker build . -t mysql-80 - ok"
else
   echo " ❌ docker build . -t mysql-80 - failed"
   exit 1
fi

# Run Image
eval "docker run -p $port:3306 \
    -v $path:/var/lib/mysql \
    -v scripts:/docker-entrypoint-initdb.d/ \
    --name $name \
    -e MYSQL_ROOT_PASSWORD=root \
    -d mysql-80 \
    --server-id=1 \
    --log-bin=/var/lib/mysql/binlog.index \
    --binlog-format=row \
#    --binlog-row-metadata=full \
    --default-authentication-plugin=mysql_native_password 2> /dev/null"
RUNNING=$(docker inspect --format="{{.State.Running}}" $name 2> /dev/null)
if [ "$RUNNING" == "true" ]; then
   echo " ✅ docker $name - running"
else
   echo " ❌ docker run failed - try again"
   exit 1
fi