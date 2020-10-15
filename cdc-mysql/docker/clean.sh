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