# Catalytic Scylla blog

This small project which is used in the Scylla blog: https://www.scylladb.com/2022/02/15/introducing-catalytic-an-orm-designed-for-scylladb-and-cassandra-written-in-rust/

It demonstrates how you can:
- use the table_to_struct crate to generate code 
- create compile time safe queries 
- interact with the generated code using CRUD operations

Setup:

1. Make sure this table is in your database:
```sql
create keyspace scylla_mapping with replication = {
'class': 'NetworkTopologyStrategy',
'replication_factor': 2
};

use scylla_mapping;

create table person(name text, age int, email text, json_example text, primary key((name), age));
```

2. Add these environment variables:
```
TEST_DB_KEYSPACE_KEY=scylla_mapping
RUST_LOG=debug
GENERATED_DB_ENTITIES_PATH_PREFIX=crate::generated
```

3. Make sure your database is reachable from 127.0.0.1:9042 or change environment
variable 'SCYLLA_URI'

Now you should be able to run the application
