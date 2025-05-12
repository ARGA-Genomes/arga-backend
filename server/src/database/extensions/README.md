# Extending the diesel query builder
Diesel goes above and beyond to make our SQL queries as type safe and correct as possible, allowing us to benefit from compile time guarantees about the correctness of a query. This is a significant value add, particularly for the ARGA project given how database intesive the application is.

Given that SQL itself is only verified at runtime this can be a tall order at times. This document aims to make it easier to extend the query builder in the best way and to explain why it should be done.

For best practices with diesel refer to: https://diesel.rs/guides/extending-diesel.html which is a good generic overview of how these piece will fit together


## Writing filters
Reference guide: https://diesel.rs/guides/composing-applications.html

The key takeaway from this guide that will impact us most is this excerpt:

> Boxing an expression also implies that it has no aggregate functions. You cannot box an aggregate expression in Diesel. As of Diesel 2.0, a boxed expression can only be used with exactly the from clause given. You cannot use a boxed expression for crates::table with an inner join to another table.

ARGA has a lot of tables and views that tries to aggregate the data into more convenient views. And these tables are the likely target for your next filter. If the view doesn't have a specific piece of datum and we don't want to extend it to include it then we will need to do an inner join in the query. For example:

``` rust
let records = taxa_tree_stats::table
    .inner_join(taxa::table.on(taxa_tree_stats::id.eq(taxa::id)))
    .filter(taxa::rank.eq("Genus"))
    .load::<MyRecord>(&mut conn)
    .await?;
```

We could create a filter called something like `with_genus_rank` to check against the taxa table and only pull genera taxa. However, because of the above excerpt we would need to create a filter specificall for a query on `taxa_tree_stats INNER JOIN taxa`, that is the query will only work for queries that _only_ use the table taxa_tree_stats joined with taxa. That greatly restricts what we can do with the filter but it does maintain the correctness guarantees of diesel avoiding a myriad of issues that can occur when composing SQL queries.

Rather than fighting against this and creating less safe but more generic queries we should lean in and accept the strict requirements of query composition. The hope being that we will compile a good set of filters that can compose together and still be specifically implemented for the most common tables we use.
