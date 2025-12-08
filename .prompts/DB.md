{change wanted}

in crates: {crate}

Ensure to review crates/$crate/dbs/*/schema.sql and crates/$crate/dbs/*/queries/*.sql. Follow this process to implement changes:
1. implement sql schema changes
2. add / edit queries in crates/$crate/dbs/*/queries/*.sql
3. use make file command to generate db migration for changes if there's a schema change
4. use make file command to generate new db hash
5. update crates/$crate/sqlc.yaml to map columns to specific rust types. Always use WrappedJsonValue, WrappedDatetime, WrappedUuid for json, date time, uuid columns. use custom rust enums for enum type columns from $crate, add them to logic folder in $crate if they dont exist.
6. run sqlc generate
7. update the repository trait in src/$crate/repository/mod.rs
8. implement any type conversions from the generated sqlc types to types in the logic folders in  src/$crate/repository/sqlite/raw_from.rs
9. implement the repository trait for sqlite using the generated functions
10. Add tests for the repository functions