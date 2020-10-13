#!/bin/bash

set -x

fluvio cluster uninstall --local

docker kill mysql-consumer mysql-producer

docker rm mysql-consumer mysql-producer

docker rmi mysql-80

rm -rf ~/mysql-cdc/

fluvio cluster install --local

pushd docker

./install.sh -n mysql-producer -d ~/mysql-cdc/mysql-producer -p 3080 && ./install.sh -n mysql-consumer -d ~/mysql-cdc/mysql-consumer -p 3090

popd

sleep 1

fluvio topic create rust-mysql-cdc

docker ps -a
