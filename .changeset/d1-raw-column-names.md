---
"workers-rs": minor
---

Add `D1PreparedStatement::raw_with_column_names`, which calls `raw({ columnNames: true })` and
returns the column names alongside the rows as `(Vec<String>, Vec<Vec<T>>)`. The existing `raw` is
unchanged.
