#!/bin/bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

helpFunction()
{
   echo ""
   echo "Usage: $0 -n name -d data-path"
   echo -e "\t-n Container name"
   echo -e "\t-d Data path"
   exit 1 # Exit script after printing help
}

while getopts "n:d:" opt
do
   case "$opt" in
      n ) name="$OPTARG" ;;
      d ) path="$OPTARG" ;;
      ? ) helpFunction ;; # Print helpFunction in case parameter is non-existent
   esac
done

# Print helpFunction in case parameters are empty
if [ -z "$name" ] || [ -z "$path" ]
then
   echo "Invalid parameters";
   helpFunction
fi

# Stop container
eval "docker stop $name >/dev/null 2>&1"
RUNNING=$(docker inspect --format="{{.State.Running}}" $name 2> /dev/null)
if [ ! "$RUNNING" == "true" ]; then
   echo " ✅ docker stop $name - ok"
else
   echo " ❌ docker stop $name - failed"
   exit 1
fi

eval "docker rm $name >/dev/null 2>&1"
echo " ✅ docker rm $name - ok"

# Remove data file
eval "rm -rf $path >/dev/null 2>&1"
if [ ! -d `eval echo $path` ]; then
   echo " ✅ rm -rf $path - ok"
else
   echo " ❌ rm -rf $path - failed"
   exit 1
fi
