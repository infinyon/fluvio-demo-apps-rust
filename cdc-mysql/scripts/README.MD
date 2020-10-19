# MYSQL CDC Scripts

The CDC environment uses Docker to run 2 MySQL databases in parallel. The databases are mapped to local drive according to `data.base_path` parameter in the profile files.

**Prerequisites** 
* Fluvio CLI must be installed and accessible `fluvio` command.
* Docker must be installed.

**Tl;dr;** 

Run test:

```
cargo build -all
fluvio cluster install --local
./setup.sh
./run-test.sh
```

Clean up:

```
./cleanup.sh
fluvio cluster install --local
```

## Rust Test (in 6 steps)

### Build producer/consumer

Build producer/consumer images

```
$ cargo build -all
    ...
    Finished dev [unoptimized + debuginfo] target(s) in 0.14s
```

### Setup a Fluvio cluster

Ensure a fluvio cluster is running. To create a cluster run

```
# fluvio cluster install --local
...
waiting for spu to be provisioned
1 spus provisioned
```


### Setup CDC environment

Setup start two Docker containers, one for `producer` and the other for `consumer`

```
$ ./setup.sh 
Install 'mysql-producer' container...
 ✅ mkdir -p /Users/aj/mysql-cdc/mysql-producer - ok
 ✅ docker build . -t mysql-80 - ok
5cb7322cee3776cbc4a559a1ba2db3b9edd832f5d51d8938b24ed303c436ece4
 ✅ docker mysql-producer - running
 🎉 Done!
Install 'mysql-consumer' container...
 ✅ mkdir -p /Users/aj/mysql-cdc/mysql-consumer - ok
 ✅ docker build . -t mysql-80 - ok
65818b439df4a3412e2ae99994474ba78d085c9ac9eaaf9bab4da9cbd4e4e6ce
 ✅ docker mysql-consumer - running
 🎉 Done!
```


### Run Tests

Run sanity test:

```
$ ./run-test.sh 
Creating topic... rust-mysql-cdc
 ✅ fluvio topic create rust-mysql-cdc - ok
 ⌛ Waiting for MySQL
....
 ✅ mysql is running
 ✅ mysql> CREATE DATABASE flvDb; - ok
 ✅ mysql> use flvDb; CREATE TABLE pet (name VARCHAR(20), owner VARCHAR(20), species VARCHAR(20), sex CHAR(1), birth DATE); - ok
 ✅ mysql> use flvDb; INSERT INTO pet VALUES ('Puffball','Diane','hamster','f','1999-03-30'); - ok
 ✅ mysql> use flvDb; INSERT INTO pet VALUES ('Jack','Peter','dog','m','1999-03-30'); - ok
 ✅ mysql> use flvDb; UPDATE pet SET birth = '1989-08-31' WHERE name = 'Jack'; - ok
 ✅ mysql> use flvDb; ALTER TABLE pet ADD COLUMN last_vaccine DATE; - ok
 ✅ mysql> use flvDb; DELETE from pet where name='Puffball'; - ok
 ✅ mysql> use flvDb; INSERT INTO pet VALUES ('Spot', 'Jane', 'dog', 'm', '2010-11-2', Null); - ok
 ✅ mysql> use flvDb; UPDATE pet SET last_vaccine='2020-6-10' WHERE name='Spot'; - ok
 ✅ cargo run --bin cdc-producer -- producer_profile.toml' - ok
..........
 ✅ cdc-producer - stopped
 ✅ cargo run --bin cdc-consumer -- consumer_profile.toml' - ok
..........
 ✅ cdc-consumer - stopped
 ✅ mysql is running
Retrieve 'pet' from Producer ...
 ✅ mysql> use flvDb; select * from pet; - ok
Retrieve 'pet' from Consumer ...
 ✅ mysql> use flvDb; select * from pet; - ok
>> Leader:
name    owner   species sex     birth   last_vaccine
Jack    Peter   dog     m       1989-08-31      NULL
Spot    Jane    dog     m       2010-11-02      2020-06-10
>> Follower:
name    owner   species sex     birth   last_vaccine
Jack    Peter   dog     m       1989-08-31      NULL
Spot    Jane    dog     m       2010-11-02      2020-06-10
 ✅ result - ok
```
The test will leave the consumer & consumer database intact for manual inspection.


### Clean-up CDC environment

Clean-up script will remove containers and local directories.

```
./cleanup.sh 
Delete 'mysql-producer' container...
 ✅ docker stop mysql-producer - ok
 ✅ docker rm mysql-producer - ok
 ✅ rm -rf /Users/aj/mysql-cdc/mysql-producer - ok
 🎉 Done!
Delete 'mysql-consumer' container...
 ✅ docker stop mysql-consumer - ok
 ✅ docker rm mysql-consumer - ok
 ✅ rm -rf /Users/aj/mysql-cdc/mysql-consumer - ok
 🎉 Done! 
```

### Uninstall Cluster

Uninstall cluster to remove topic

```
# fluvio cluster install --local
...
topic.fluvio.infinyon.com "rust-mysql-cdc" deleted
deleting label 'app=spu' object persistentvolumeclaims in: default
No resources found
```


## Helper Commands (step by step)

The top level scripts utilize a series fo helper commands

### Create topic

Create topic on Fluvio

```
$ ./helpers/create-topic.sh
Creating topic... rust-mysql-cdc
 ✅ fluvio topic create rust-mysql-cdc - ok
```

### Setup producer/consumer

Setup producer

```
$ ./helpers/setup.sh -f ../producer_profile.toml
Install 'mysql-producer' container...
 ✅ mkdir -p /Users/aj/mysql-cdc/mysql-producer - ok
 ✅ docker build . -t mysql-80 - ok
9126224c09ddb1380454c8418ee5c4e8cbb59dc8ea02c8935e568f4b676299c6
 ✅ docker mysql-producer - running
 🎉 Done!
```

Setup consumer

```
$ ./helpers/setup.sh -f ../consumer_profile.toml
Install 'mysql-consumer' container...
 ✅ mkdir -p /Users/aj/mysql-cdc/mysql-consumer - ok
 ✅ docker build . -t mysql-80 - ok
c4b3508008cc0abad0fb2072a7978870c076e2e6b1c2a0773928f6b5113302cf
 ✅ docker mysql-consumer - running
 🎉 Done!
```

### Wait for Mysql 

Wait for mysql on the producer

```
 ./helpers/wait-mysql-producer.sh 
 ✅ mysql is running
 ```

 Wait for mysql on the consumer

 ```
 ./helpers/wait-mysql-consumer.sh 
 ✅ mysql is running
 ```

### Send MySQL Commands

Send commands to producer MYSQL

```
$ ./helpers/mysql-send-cmds.sh
 ✅ mysql> CREATE DATABASE flvDb; - ok
 ✅ mysql> use flvDb; CREATE TABLE pet (name VARCHAR(20), owner VARCHAR(20), species VARCHAR(20), sex CHAR(1), birth DATE); - ok
 ✅ mysql> use flvDb; INSERT INTO pet VALUES ('Puffball','Diane','hamster','f','1999-03-30'); - ok
 ✅ mysql> use flvDb; INSERT INTO pet VALUES ('Jack','Peter','dog','m','1999-03-30'); - ok
 ✅ mysql> use flvDb; UPDATE pet SET birth = '1989-08-31' WHERE name = 'Jack'; - ok
 ✅ mysql> use flvDb; ALTER TABLE pet ADD COLUMN last_vaccine DATE; - ok
 ✅ mysql> use flvDb; DELETE from pet where name='Puffball'; - ok
 ✅ mysql> use flvDb; INSERT INTO pet VALUES ('Spot', 'Jane', 'dog', 'm', '2010-11-2', Null); - ok
 ✅ mysql> use flvDb; UPDATE pet SET last_vaccine='2020-6-10' WHERE name='Spot'; - ok
```

### Run CDC Producer

CDC producer runs for 5 seconds then it is killed. 

```
 ./helpers/run-producer.sh 
 ✅ cargo run --bin cdc-producer -- producer_profile.toml' - ok
..........
 ✅ cdc-producer - stopped
```

The producer captures debugging information in `/tmp/cdc-producer.log`

```
$ tail /tmp/cdc-producer.log
Oct 19 09:48:50.060 DEBUG fluvio_socket::multiplexing: dispatcher: waiting for next response from stream 
Oct 19 09:48:50.060 DEBUG dispatch_loop{spec="Partition"}: fluvio::sync::controller: received request
...
```

### Run CDC Consumer

CDC producer runs for 5 seconds then it is killed. 

```
./helpers/run-consumer.sh 
 ✅ cargo run --bin cdc-consumer -- consumer_profile.toml' - ok
..........
 ✅ cdc-consumer - stopped
```

The producer captures debugging information in `/tmp/cdc-consumer.log`

```
$ tail  /tmp/cdc-consumer.log
Connecting to mysql database... 
./helpers/run-consumer.sh: line 25: 65189 Killed: 9               ( RUST_LOG=debug cargo run --bin cdc-consumer -- /Users/aj/projects/github/fluvio-demo-apps-rust/cdc-mysql/scripts/helpers/../../consumer_profile.toml )
```

### Validate MySQL Result

Queries both databases and compares the results

```
$ $ ./helpers/mysql-validate-result.sh
Retrieve 'pet' from Producer ...
 ✅ mysql> use flvDb; select * from pet; - ok
Retrieve 'pet' from Consumer ...
 ✅ mysql> use flvDb; select * from pet; - ok
>> Leader:
name    owner   species sex     birth   last_vaccine
Jack    Peter   dog     m       1989-08-31      NULL
Spot    Jane    dog     m       2010-11-02      2020-06-10
>> Follower:
name    owner   species sex     birth   last_vaccine
Jack    Peter   dog     m       1989-08-31      NULL
Spot    Jane    dog     m       2010-11-02      2020-06-10
 ✅ result - ok
```

### Clean-up producer/consumer

Remove producer container & directory

```
$ ./helpers/cleanup.sh -f ../producer_profile.toml
Delete 'mysql-producer' container...
 ✅ docker stop mysql-producer - ok
 ✅ docker rm mysql-producer - ok
 ✅ rm -rf /Users/aj/mysql-cdc/mysql-producer - ok
 🎉 Done!
```

Remove consumer container & directory

```
$ ./helpers/cleanup.sh -f ../consumer_profile.toml
Delete 'mysql-consumer' container...
 ✅ docker stop mysql-consumer - ok
 ✅ docker rm mysql-consumer - ok
 ✅ rm -rf /Users/aj/mysql-cdc/mysql-consumer - ok
 🎉 Done!
```