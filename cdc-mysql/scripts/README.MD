# MYSQL CDC Scripts

The CDC environment uses Docker to run 2 MySQL databases in parallel. The databases are mapped to local drive according to `data.base_path` parameter in the profile files.

The following scripts automate the setup and tear-down of this environment

## Setup CDC environment

To setup one environment at a time use `setup.sh`, to setup both environments use `setup-all.sh`

### Setup Consumer

```
$ ./setup.sh -f ../consumer_profile.toml
Install 'mysql-consumer' container...
 ✅ mkdir -p /Users/aj/mysql-cdc/mysql-consumer - ok
 ✅ docker build . -t mysql-80 - ok
2a3f566f716cfea547cf96fe57e100795de4bda924a5e96392a0125efaaed4d8
 ✅ docker mysql-consumer - running
 🎉 Done!
```

### Setup Producer

```
$ ./setup.sh -f ../producer_profile.toml 
Install 'mysql-producer' container...
 ✅ mkdir -p /Users/aj/mysql-cdc/mysql-producer - ok
 ✅ docker build . -t mysql-80 - ok
793765f9edecbe733a92c793124c9a8763f608c3571942440909755a3abda046
 ✅ docker mysql-producer - running
 🎉 Done!
```

### Setup Both

Setup-all runs both scripts with hardcoded profile files:

```
$ ./setup-all.sh
Install 'mysql-consumer' container...
 ✅ mkdir -p /Users/aj/mysql-cdc/mysql-consumer - ok
 ✅ docker build . -t mysql-80 - ok
78c1991ef20d8e99a074fce551e6e82467efc4c57e7cf8c5c157a29c4380134f
 ✅ docker mysql-consumer - running
 🎉 Done!
Install 'mysql-producer' container...
 ✅ mkdir -p /Users/aj/mysql-cdc/mysql-producer - ok
 ✅ docker build . -t mysql-80 - ok
5fa96c3873a8a1ab4b3ea4e039974291d09720e70ffa60e8ada9fdda1e1b9ff1
 ✅ docker mysql-producer - running
 🎉 Done!
```

**Note**: It takes about 60 seconds for mysql to come-up after the container is running. 

### Check if Mysql is running

Check producer Mysql

```
$ ./check-mysql.sh -f ../producer_profile.toml 
 ❌ mysql is not running
$ ./check-mysql.sh -f ../producer_profile.toml 
 ❌ mysql is not running
$ ./check-mysql.sh -f ../producer_profile.toml 
 ✅ mysql is running
```

Check consumer Mysql

```
$ ./check-mysql.sh -f ../consumer_profile.toml
 ✅ mysql is running

```

## Setup a Fluvio cluster

Ensure a fluvio cluster is running. To create a cluster run

```
# fluvio cluster install --local
```

## Run Tests

Run sanity test:

```
$ ./run-test.sh
Creating topic... rust-mysql-cdc
 ✅ fluvio topic create rust-mysql-cdc - ok
 ✅ mysql> CREATE DATABASE flvDb; - ok
 ✅ mysql> use flvDb; CREATE TABLE pet (name VARCHAR(20), owner VARCHAR(20), species VARCHAR(20), sex CHAR(1), birth DATE); - ok
 ✅ mysql> use flvDb; INSERT INTO pet VALUES ('Puffball','Diane','hamster','f','1999-03-30'); - ok
 ✅ mysql> use flvDb; INSERT INTO pet VALUES ('Jack','Peter','dog','m','1999-03-30'); - ok
 ✅ mysql> use flvDb; UPDATE pet SET birth = '1989-08-31' WHERE name = 'Jack'; - ok
 ✅ mysql> use flvDb; ALTER TABLE pet ADD COLUMN color VARCHAR(20); - ok
 ✅ mysql> use flvDb; DELETE from pet where name='Puffball'; - ok
 ✅ mysql> use flvDb; INSERT INTO pet VALUES ('Spot', 'Jane', 'dog', 'm', '2010-11-2', Null); - ok
 ✅ mysql> use flvDb; UPDATE pet SET color='White' WHERE name='Spot'; - ok
 ✅ cargo run --bin cdc-producer -- producer_profile.toml' - ok
..........
 ✅ cdc-producer - stopped
 ✅ cargo run --bin cdc-consumer -- consumer_profile.toml' - ok
..........
 ✅ cdc-consumer - stopped
 ✅ mysql> use flvDb; select * from pet; - ok
 ✅ result - ok
```

The test will leave the consumer database as is for manual inspection.


## Clean-up CDC environment

Similarly to the setup, there are also 3 clean-up scripts. The clean-up scripts also use the environment variables to identify the container and the data path directories is needs to delete.

### Clean-up Consumer

```
$ ./cleanup.sh -f ../consumer_profile.toml
Delete 'mysql-consumer' container...
 ✅ docker stop mysql-consumer - ok
 ✅ docker rm mysql-consumer - ok
 ✅ rm -rf /Users/aj/mysql-cdc/mysql-consumer - ok
 🎉 Done!
```

### Clean-up Producer

```
$ ./cleanup.sh -f ../producer_profile.toml
Delete 'mysql-producer' container...
 ✅ docker stop mysql-producer - ok
 ✅ docker rm mysql-producer - ok
 ✅ rm -rf /Users/aj/mysql-cdc/mysql-producer - ok
 🎉 Done!
```

### Clean-up Both

Cleanup-all runs both scripts with hardcoded profile files:

```
$ ./cleanup-all.sh
./cleanup-all.sh 
Delete 'mysql-consumer' container...
 ✅ docker stop mysql-consumer - ok
 ✅ docker rm mysql-consumer - ok
 ✅ rm -rf /Users/aj/mysql-cdc/mysql-consumer - ok
 🎉 Done!
Delete 'mysql-producer' container...
 ✅ docker stop mysql-producer - ok
 ✅ docker rm mysql-producer - ok
 ✅ rm -rf /Users/aj/mysql-cdc/mysql-producer - ok
 🎉 Done!
```

