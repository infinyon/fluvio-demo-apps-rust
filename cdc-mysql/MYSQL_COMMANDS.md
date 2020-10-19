# Mysql Sample Commands

Sample command to demonstrate event propagation between two mysql instances.

Fluvio producer/consumer can propagate the following commands:

* CREATE database
* DROP database
* ALTER table
* INSERT into table
* UPDATE table rows
* DELETE table rows

Each command is translated into an event that is transmitted to follower instances in real-time.

## Table - User

```mysql
USE flvDb;

CREATE TABLE user (first_name VARCHAR(20), last_name VARCHAR(20),  sex CHAR(1), birth DATE);

INSERT INTO user VALUES ('John','Doe','m','2000-03-30');
```


## Table - Year

```mysql
USE flvDb;

CREATE TABLE year (y YEAR);

INSERT INTO year (y) VALUES (1998);

INSERT INTO year (y) VALUES (1999);

INSERT INTO year (y) VALUES (2009);

INSERT INTO year (y) VALUES (2020);

DELETE FROM year  WHERE y LIKE "19%";

DROP TABLE year;
```