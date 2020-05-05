# sql_builder_test

Just (for now) a silly experimental macro for _readable_ SQL builders

Example:

```
build_query!(
    "SELECT * FROM foo"
    "WHERE"
    if let Some(id) = opt_id {
        "foo.id = " id
    } else {
        "TRUE"
    }
    "ORDER BY bar"
)
```

## Goals
* SQL: As readable as possible
* Compile-time SQL injection safety (all SQL composed of e.g. string literals)
* Composable: Use helper macros for e.g. common subqueries

## Hairy goals
* Integrate with e.g. [SQLx](https://github.com/launchbadge/sqlx)
* Static cyclomatic complexity analysis (prepared statement caching?)
* Compile-time SQL syntax check using live database for at least a subset of the possible outputs (the shortest/longest one?)
