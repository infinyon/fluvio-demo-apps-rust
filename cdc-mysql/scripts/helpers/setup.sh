#!/bin/bash
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
ROOT_DIR="$DIR/../.."

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

# Retrieve "data path"
dataPath=$(grep "^base_path" $profile | cut -d' ' -d'=' -f2- | tr -d '"' | tr -d '[:space:]')
if [ -z "$dataPath" ]
then
   echo "Cannot find 'data.base_path' in '$profile'";
   exit 1
fi

# Retrieve "container name"
containerName=$(grep "^name" $profile | cut -d' ' -d'=' -f2- | tr -d '"' | tr -d '[:space:]')
if [ -z "$containerName" ]
then
   echo "Cannot find 'container.name' in '$profile'";
   exit 1 
fi

# Retrieve "port number"
containerPort=$(grep "^host_port" $profile | cut -d' ' -d'=' -f2- | tr -d '[:space:]')
if [ -z "$containerPort" ]
then
   echo "Cannot find 'container.host_port' in '$profile'";
   exit 1 
fi

echo "Install '$containerName' container..."
eval "$ROOT_DIR/docker/install.sh -n $containerName -d $dataPath -p $containerPort"
echo " 🎉 Done!"